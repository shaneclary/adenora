use crate::auth_extractor::AuthUser;
use crate::state::AppState;
use axum::{Json, extract::State, http::StatusCode};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct DepositRequest {
    pub currency: String,
    pub amount: Decimal,
    pub method: String,
}

#[derive(Deserialize)]
pub struct WithdrawRequest {
    pub currency: String,
    pub amount: Decimal,
    pub method: String,
    pub destination: String,
}

pub async fn get_wallet(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(uuid::Uuid, String, Decimal, Decimal)> = sqlx::query_as(
        "SELECT id, currency, available, reserved FROM wallets WHERE user_id = $1"
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let wallets: Vec<Value> = rows
        .into_iter()
        .map(|(id, currency, available, reserved)| {
            json!({
                "id": id,
                "currency": currency,
                "available": available,
                "reserved": reserved,
                "total": available + reserved
            })
        })
        .collect();

    Ok(Json(json!({ "wallets": wallets })))
}

pub async fn deposit(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<DepositRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.amount <= Decimal::ZERO {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "amount must be positive"}))));
    }

    // Only accept currencies the platform actually supports.
    if adenora_common::currency::Currency::from_code(&body.currency).is_none() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "unsupported currency"}))));
    }

    // The limit checks and the credit run in one transaction with the user row
    // locked (SELECT ... FOR UPDATE). This both enforces every responsible-
    // gambling restriction and serializes concurrent deposits so two requests
    // cannot each read the same pre-deposit total and both slip under the limit.
    let mut tx = state.db.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let user: Option<(
        Option<chrono::DateTime<chrono::Utc>>, // self_exclusion_until
        Option<chrono::DateTime<chrono::Utc>>, // cooling_off_until
        Option<Decimal>,                       // daily_deposit_limit
        Option<Decimal>,                       // weekly_deposit_limit
        Option<Decimal>,                       // monthly_deposit_limit
    )> = sqlx::query_as(
        "SELECT self_exclusion_until, cooling_off_until,
                daily_deposit_limit, weekly_deposit_limit, monthly_deposit_limit
         FROM users WHERE id = $1 FOR UPDATE"
    )
    .bind(auth.user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let Some((self_excl, cooling_off, daily, weekly, monthly)) = user else {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "user not found"}))));
    };

    let now = chrono::Utc::now();
    if let Some(until) = self_excl {
        if now < until {
            return Err((StatusCode::FORBIDDEN, Json(json!({"error": "self-exclusion active", "until": until}))));
        }
    }
    if let Some(until) = cooling_off {
        if now < until {
            return Err((StatusCode::FORBIDDEN, Json(json!({"error": "cooling-off period active", "until": until}))));
        }
    }

    // Enforce daily / weekly / monthly deposit caps against the in-transaction totals.
    for (limit, interval, label) in [
        (daily, "0 days", "daily"),
        (weekly, "7 days", "weekly"),
        (monthly, "30 days", "monthly"),
    ] {
        let Some(limit) = limit else { continue };
        let used: (Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE user_id = $1 AND tx_type = 'deposit'
             AND created_at >= CURRENT_DATE - $2::interval"
        )
        .bind(auth.user_id)
        .bind(interval)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        if used.0 + body.amount > limit {
            return Err((StatusCode::BAD_REQUEST, Json(json!({
                "error": format!("{label} deposit limit exceeded"),
                "limit": limit,
                "deposited": used.0
            }))));
        }
    }

    // Upsert wallet
    sqlx::query(
        "INSERT INTO wallets (user_id, currency, available)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id, currency)
         DO UPDATE SET available = wallets.available + $3, updated_at = NOW()"
    )
    .bind(auth.user_id)
    .bind(&body.currency)
    .bind(body.amount)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    // Record transaction
    sqlx::query(
        "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, description)
         VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = $2), 'deposit', $3, $2, $4)"
    )
    .bind(auth.user_id)
    .bind(&body.currency)
    .bind(body.amount)
    .bind(format!("deposit via {}", body.method))
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    tx.commit().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "status": "completed",
        "amount": body.amount,
        "currency": body.currency,
        "fee": "0.00",
        "message": "deposits are always free"
    })))
}

pub async fn withdraw(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<WithdrawRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    use crate::services::payments;

    if body.amount <= Decimal::ZERO {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "amount must be positive"}))));
    }

    // Only accept currencies the platform supports.
    if adenora_common::currency::Currency::from_code(&body.currency).is_none() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "unsupported currency"}))));
    }

    // Validate the payout destination for the chosen rail so funds aren't debited
    // toward a malformed IBAN or crypto address.
    let method = body.method.to_lowercase();
    let destination_ok = if method.contains("crypto") || method.contains("usdc") || method.contains("eth") {
        payments::is_valid_eth_address(&body.destination)
    } else {
        payments::is_valid_iban(&body.destination)
    };
    if !destination_ok {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "invalid withdrawal destination"}))));
    }

    // Withdrawals are always allowed even during self-exclusion (player protection).

    let result = sqlx::query(
        "UPDATE wallets SET available = available - $3, updated_at = NOW()
         WHERE user_id = $1 AND currency = $2 AND available >= $3"
    )
    .bind(auth.user_id)
    .bind(&body.currency)
    .bind(body.amount)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "insufficient balance"}))));
    }

    sqlx::query(
        "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, description)
         VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = $2), 'withdrawal', $3, $2, $4)"
    )
    .bind(auth.user_id)
    .bind(&body.currency)
    .bind(body.amount)
    .bind(format!("withdrawal to {} via {}", body.destination, body.method))
    .execute(&state.db)
    .await
    .ok();

    Ok(Json(json!({
        "status": "processing",
        "amount": body.amount,
        "currency": body.currency,
        "destination": body.destination,
        "fee": "0.00",
        "message": "withdrawals are always free"
    })))
}
