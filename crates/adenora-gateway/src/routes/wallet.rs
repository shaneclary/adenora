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

    // Check gambling limits
    let limits: Option<(Option<Decimal>,)> = sqlx::query_as(
        "SELECT daily_deposit_limit FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some((Some(daily_limit),)) = limits {
        let today_deposits: (Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE user_id = $1 AND tx_type = 'deposit'
             AND created_at >= CURRENT_DATE"
        )
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or((Decimal::ZERO,));

        if today_deposits.0 + body.amount > daily_limit {
            return Err((StatusCode::BAD_REQUEST, Json(json!({
                "error": "daily deposit limit exceeded",
                "limit": daily_limit,
                "deposited_today": today_deposits.0
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
    .execute(&state.db)
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
    .execute(&state.db)
    .await
    .ok();

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
    if body.amount <= Decimal::ZERO {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "amount must be positive"}))));
    }

    // Check self-exclusion
    let exclusion: Option<(Option<chrono::DateTime<chrono::Utc>>,)> = sqlx::query_as(
        "SELECT self_exclusion_until FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    // Withdrawals are always allowed even during self-exclusion (player protection)

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
