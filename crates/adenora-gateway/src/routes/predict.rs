use crate::auth_extractor::AuthUser;
use crate::state::AppState;
use adenora_common::fees;
use adenora_common::types::*;
use adenora_orderbook::book::Order;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

// ─── Simplified predict endpoint ────────────────────────────────────────────

#[derive(Deserialize)]
pub struct PredictRequest {
    pub market_id: Uuid,
    pub side: String,           // "yes" or "no"
    pub amount_eur: Decimal,    // e.g. 5.00
    pub public: Option<bool>,   // opt-in to public prediction
    pub comment: Option<String>,// "money where your mouth is" comment
}

/// POST /api/v1/predict — human-friendly prediction endpoint
///
/// Accepts { market_id, side, amount_eur } and internally translates
/// to contracts at the best available price.
pub async fn predict(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<PredictRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let err = |s: StatusCode, m: &str| (s, Json(json!({"error": m})));

    // Validate amount
    if req.amount_eur <= Decimal::ZERO {
        return Err(err(StatusCode::BAD_REQUEST, "amount must be positive"));
    }
    if req.amount_eur > Decimal::new(10000, 0) {
        return Err(err(StatusCode::BAD_REQUEST, "maximum prediction is €10,000"));
    }

    let side = match req.side.as_str() {
        "yes" => Side::Yes,
        "no" => Side::No,
        _ => return Err(err(StatusCode::BAD_REQUEST, "side must be 'yes' or 'no'")),
    };

    // KYC + self-exclusion check
    let user_check: Option<(String, Option<chrono::DateTime<Utc>>)> = sqlx::query_as(
        "SELECT kyc_status, self_exclusion_until FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    if let Some((kyc_status, exclusion_until)) = &user_check {
        if kyc_status != "verified" && kyc_status != "approved" {
            return Err(err(StatusCode::FORBIDDEN, "KYC verification required before predicting"));
        }
        if let Some(until) = exclusion_until {
            if Utc::now() < *until {
                return Err((StatusCode::FORBIDDEN, Json(json!({
                    "error": "self-exclusion active", "until": until
                }))));
            }
        }
    } else {
        return Err(err(StatusCode::UNAUTHORIZED, "user not found"));
    }

    // Check market is active + get charity project
    let market: Option<(String, Option<Uuid>, String)> = sqlx::query_as(
        "SELECT m.status, m.charity_project_id, COALESCE(cp.name, '')
         FROM markets m
         LEFT JOIN charity_projects cp ON m.charity_project_id = cp.id
         WHERE m.id = $1"
    )
    .bind(req.market_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let (status, _charity_project_id, charity_name) = market
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "market not found"))?;

    if status != "active" {
        return Err(err(StatusCode::BAD_REQUEST, "market is not active"));
    }

    // Get best price from the people-mode order book
    let engine = state.get_engine(req.market_id).await;
    let snapshots = engine.snapshots().await;
    let people_book = &snapshots.people;

    // Determine price: use best ask if available, otherwise default to 50
    let price_cents: u32 = match side {
        Side::Yes => people_book.best_ask.unwrap_or(50),
        Side::No => 100 - people_book.best_bid.unwrap_or(50),
    };

    // Clamp to valid range
    let price_cents = price_cents.max(1).min(99);
    let price_dollars = Decimal::new(price_cents as i64, 2);

    // Calculate contracts from euro amount
    // Each contract costs price_cents cents, so quantity = amount_eur / (price_cents / 100)
    let quantity = (req.amount_eur * Decimal::new(100, 0) / Decimal::from(price_cents))
        .floor()
        .to_u32()
        .unwrap_or(0);

    if quantity == 0 {
        return Err(err(StatusCode::BAD_REQUEST, "amount too small for current price"));
    }

    // Calculate actual cost and potential payout
    let actual_cost = Decimal::new(price_cents as i64 * quantity as i64, 2);
    let potential_payout = Decimal::new(quantity as i64, 0); // each winning contract = €1
    let profit = potential_payout - actual_cost;

    // Calculate fee and charity share
    let taker_fee = fees::calculate_taker_fee(quantity, price_dollars);
    let fee_split = fees::split_fee(taker_fee);

    // Atomic reserve funds (cost only — fee deducted at fill time)
    let reserved: Option<(Decimal,)> = sqlx::query_as(
        "UPDATE wallets SET available = available - $1, reserved = reserved + $1, updated_at = NOW()
         WHERE user_id = $2 AND currency = 'EUR' AND available >= $1
         RETURNING available"
    )
    .bind(actual_cost)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    if reserved.is_none() {
        return Err(err(StatusCode::PAYMENT_REQUIRED, "insufficient balance"));
    }

    // Build and submit order
    let order = Order {
        id: Uuid::new_v4(),
        user_id: auth.user_id,
        market_id: req.market_id,
        side,
        action: Action::Buy,
        price_cents,
        quantity,
        filled_quantity: 0,
        time_in_force: TimeInForce::Gtc,
        status: OrderStatus::Pending,
        mode: MarketMode::People,
        bot_id: None,
        created_at: Utc::now(),
    };

    let order_id = order.id;

    // Persist order
    sqlx::query(
        "INSERT INTO orders (id, user_id, market_id, side, action, price_cents, quantity, time_in_force, status, mode)
         VALUES ($1, $2, $3, $4, 'buy', $5, $6, 'gtc', 'pending', 'people')"
    )
    .bind(order.id)
    .bind(auth.user_id)
    .bind(req.market_id)
    .bind(format!("{:?}", side).to_lowercase())
    .bind(price_cents as i32)
    .bind(quantity as i32)
    .execute(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    // Submit to engine
    let result = engine.submit(order).await;
    let status_str = match &result {
        adenora_orderbook::engine::SubmitResult::Queued { .. } => "queued",
        adenora_orderbook::engine::SubmitResult::Executed { .. } => "matched",
    };

    // Record public prediction if opted in
    let is_public = req.public.unwrap_or(false);
    if is_public {
        let comment = req.comment.as_deref().unwrap_or("");
        // Sanitize comment: strip HTML/script tags
        let safe_comment: String = comment.chars()
            .filter(|c| *c != '<' && *c != '>')
            .take(500)
            .collect();

        sqlx::query(
            "INSERT INTO public_predictions (user_id, market_id, order_id, side, amount_eur, comment)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(auth.user_id)
        .bind(req.market_id)
        .bind(order_id)
        .bind(format!("{:?}", side).to_lowercase())
        .bind(actual_cost)
        .bind(&safe_comment)
        .execute(&state.db)
        .await
        .ok(); // non-critical — don't fail the prediction over a social feature
    }

    // Get user display name for the response confirmation.
    let display_name: String = sqlx::query_as::<_, (String,)>(
        "SELECT display_name FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .map(|(n,)| n)
    .unwrap_or_default();

    tracing::info!(
        user_id = %auth.user_id,
        market_id = %req.market_id,
        side = ?side,
        amount_eur = %actual_cost,
        quantity = quantity,
        price_cents = price_cents,
        public = is_public,
        "prediction placed"
    );

    Ok(Json(json!({
        "prediction_id": order_id,
        "status": status_str,
        "predictor": display_name,
        "side": format!("{:?}", side).to_lowercase(),
        "amount_eur": actual_cost.to_string(),
        "contracts": quantity,
        "price_cents": price_cents,
        "chance_pct": price_cents,
        "potential_payout_eur": potential_payout.to_string(),
        "potential_profit_eur": profit.to_string(),
        "fee_eur": taker_fee.to_string(),
        "charity_contribution_eur": fee_split.charity.to_string(),
        "charity_project": charity_name,
        "is_public": is_public,
        "message": "Your prediction is in!"
    })))
}

// ─── Market card (human-friendly single market) ─────────────────────────────

/// GET /api/v1/markets/{id}/card — human-friendly market data
pub async fn get_market_card(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let err = |s: StatusCode, m: &str| (s, Json(json!({"error": m})));

    let market: Option<(
        Uuid, String, String, String, String,
        chrono::DateTime<Utc>, Option<Uuid>, Option<String>, Option<String>,
    )> = sqlx::query_as(
        "SELECT m.id, m.question, m.description, m.category, m.status,
                m.closes_at, m.charity_project_id, cp.name, cp.category
         FROM markets m
         LEFT JOIN charity_projects cp ON m.charity_project_id = cp.id
         WHERE m.id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let (mid, question, description, category, status,
         closes_at, _charity_pid, charity_name, charity_cat) = market
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "market not found"))?;

    // Get price from engine
    let engine = state.get_engine(mid).await;
    let snap = engine.snapshots().await;
    let yes_price = snap.people.best_ask.unwrap_or(50);
    let no_price = 100 - yes_price;

    // Count participants
    let participants: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT user_id) FROM orders WHERE market_id = $1"
    )
    .bind(mid)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    // Time remaining
    let now = Utc::now();
    let remaining = closes_at - now;
    let time_left = if remaining.num_days() > 0 {
        format!("{} days", remaining.num_days())
    } else if remaining.num_hours() > 0 {
        format!("{} hours", remaining.num_hours())
    } else if remaining.num_minutes() > 0 {
        format!("{} min", remaining.num_minutes())
    } else {
        "closing soon".to_string()
    };

    // Payout per euro
    let yes_payout = if yes_price > 0 {
        Decimal::new(100, 2) / Decimal::new(yes_price as i64, 2)
    } else { Decimal::ZERO };
    let no_payout = if no_price > 0 {
        Decimal::new(100, 2) / Decimal::new(no_price as i64, 2)
    } else { Decimal::ZERO };

    // Recent public predictions (the debate feed)
    let debate: Vec<(String, String, Decimal, String, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT u.display_name, pp.side, pp.amount_eur, pp.comment, pp.created_at
         FROM public_predictions pp
         JOIN users u ON u.id = pp.user_id
         WHERE pp.market_id = $1
         ORDER BY pp.created_at DESC LIMIT 20"
    )
    .bind(mid)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let debate_json: Vec<Value> = debate.into_iter().map(|(name, side, amount, comment, at)| json!({
        "name": name,
        "side": side,
        "amount_eur": amount.to_string(),
        "comment": comment,
        "at": at,
    })).collect();

    Ok(Json(json!({
        "id": mid,
        "question": question,
        "description": description,
        "category": category,
        "status": status,
        "chance_yes_pct": yes_price,
        "chance_no_pct": no_price,
        "time_left": time_left,
        "closes_at": closes_at,
        "participants": participants.0,
        "charity_project": {
            "name": charity_name,
            "category": charity_cat,
        },
        "yes_payout_per_euro": yes_payout.round_dp(2).to_string(),
        "no_payout_per_euro": no_payout.round_dp(2).to_string(),
        "debate": debate_json,
    })))
}

// ─── Batch market cards ─────────────────────────────────────────────────────

/// GET /api/v1/markets/cards — all active markets in card format
pub async fn list_market_cards(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let err = |s: StatusCode, m: &str| (s, Json(json!({"error": m})));

    let markets: Vec<(
        Uuid, String, String, String,
        chrono::DateTime<Utc>, Option<String>,
    )> = sqlx::query_as(
        "SELECT m.id, m.question, m.category, m.status,
                m.closes_at, cp.name
         FROM markets m
         LEFT JOIN charity_projects cp ON m.charity_project_id = cp.id
         WHERE m.status = 'active'
         ORDER BY m.closes_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let now = Utc::now();
    let mut cards: Vec<Value> = Vec::with_capacity(markets.len());

    for (mid, question, category, _status, closes_at, charity_name) in &markets {
        let engine = state.get_engine(*mid).await;
        let snap = engine.snapshots().await;
        let yes_price = snap.people.best_ask.unwrap_or(50);

        let remaining = *closes_at - now;
        let time_left = if remaining.num_days() > 0 {
            format!("{}d", remaining.num_days())
        } else if remaining.num_hours() > 0 {
            format!("{}h", remaining.num_hours())
        } else {
            "soon".to_string()
        };

        cards.push(json!({
            "id": mid,
            "question": question,
            "category": category,
            "chance_yes_pct": yes_price,
            "time_left": time_left,
            "closes_at": closes_at,
            "charity_project": charity_name,
        }));
    }

    Ok(Json(json!({ "cards": cards, "total": cards.len() })))
}

// ─── Human-friendly predictions (portfolio) ─────────────────────────────────

/// GET /api/v1/predictions — your predictions in plain language
pub async fn get_predictions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let err = |s: StatusCode, m: &str| (s, Json(json!({"error": m})));

    let rows: Vec<(
        Uuid, Uuid, String, String, i32, Decimal, String,
        chrono::DateTime<Utc>, String, Option<String>,
    )> = sqlx::query_as(
        "SELECT p.id, p.market_id, m.question, p.side, p.quantity, p.avg_price, p.mode,
                m.closes_at, m.status, cp.name
         FROM positions p
         JOIN markets m ON m.id = p.market_id
         LEFT JOIN charity_projects cp ON m.charity_project_id = cp.id
         WHERE p.user_id = $1 AND p.quantity > 0
         ORDER BY m.closes_at ASC"
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let now = Utc::now();
    let mut total_invested = Decimal::ZERO;
    let mut total_potential = Decimal::ZERO;
    let mut predictions: Vec<Value> = Vec::new();

    for (pid, market_id, question, side, qty, avg_price, _mode,
         closes_at, market_status, charity_name) in &rows
    {
        let invested = *avg_price * Decimal::from(*qty);
        let payout = Decimal::from(*qty); // €1 per winning contract
        total_invested += invested;
        total_potential += payout;

        let remaining = *closes_at - now;
        let time_left = if remaining.num_days() > 0 {
            format!("{} days", remaining.num_days())
        } else if remaining.num_hours() > 0 {
            format!("{} hours", remaining.num_hours())
        } else {
            "closing soon".to_string()
        };

        let pred_status = match market_status.as_str() {
            "active" => "active",
            "settled" | "resolved" => "resolved",
            _ => "pending",
        };

        predictions.push(json!({
            "id": pid,
            "market_id": market_id,
            "question": question,
            "your_side": side,
            "amount_spent_eur": invested.round_dp(2).to_string(),
            "contracts": qty,
            "potential_win_eur": payout.to_string(),
            "chance_pct": (avg_price * Decimal::new(100, 0)).round().to_u32().unwrap_or(0),
            "status": pred_status,
            "time_left": time_left,
            "charity_project": charity_name,
        }));
    }

    // Get total charity contribution
    let charity_total: (Decimal,) = sqlx::query_as(
        "SELECT COALESCE(SUM(cl.amount), 0)
         FROM charity_ledger cl
         JOIN trades t ON cl.reference_id = t.id::text
         WHERE t.buyer_user_id = $1 OR t.seller_user_id = $1"
    )
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((Decimal::ZERO,));

    Ok(Json(json!({
        "predictions": predictions,
        "total": predictions.len(),
        "total_invested_eur": total_invested.round_dp(2).to_string(),
        "total_potential_win_eur": total_potential.round_dp(2).to_string(),
        "total_charity_contributed_eur": charity_total.0.round_dp(2).to_string(),
    })))
}

// ─── Impact feed ────────────────────────────────────────────────────────────

/// GET /api/v1/impact/feed — your charity impact timeline
pub async fn impact_feed(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let err = |s: StatusCode, m: &str| (s, Json(json!({"error": m})));

    let entries: Vec<(Decimal, String, String, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT cl.amount, cp.name, cl.description, cl.created_at
         FROM charity_ledger cl
         JOIN charity_projects cp ON cl.project_id = cp.id
         JOIN trades t ON cl.reference_id = t.id::text
         WHERE t.buyer_user_id = $1 OR t.seller_user_id = $1
         ORDER BY cl.created_at DESC LIMIT 50"
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let total: Decimal = entries.iter().map(|(a, ..)| *a).sum();

    let feed: Vec<Value> = entries.into_iter().map(|(amount, project, desc, at)| json!({
        "amount_eur": amount.round_dp(2).to_string(),
        "project_name": project,
        "description": desc,
        "date": at,
    })).collect();

    Ok(Json(json!({
        "total_contributed_eur": total.round_dp(2).to_string(),
        "entries": feed,
    })))
}

// ─── Public debate feed for a market ────────────────────────────────────────

/// GET /api/v1/markets/{id}/debate — public predictions with conviction amounts
pub async fn get_market_debate(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let err = |s: StatusCode, m: &str| (s, Json(json!({"error": m})));

    let predictions: Vec<(String, String, Decimal, String, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT u.display_name, pp.side, pp.amount_eur, pp.comment, pp.created_at
         FROM public_predictions pp
         JOIN users u ON u.id = pp.user_id
         WHERE pp.market_id = $1
         ORDER BY pp.amount_eur DESC"
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let yes_total: Decimal = predictions.iter()
        .filter(|(_, s, ..)| s == "yes")
        .map(|(_, _, a, ..)| *a)
        .sum();
    let no_total: Decimal = predictions.iter()
        .filter(|(_, s, ..)| s == "no")
        .map(|(_, _, a, ..)| *a)
        .sum();

    let feed: Vec<Value> = predictions.into_iter().map(|(name, side, amount, comment, at)| json!({
        "name": name,
        "side": side,
        "amount_eur": amount.round_dp(2).to_string(),
        "comment": if comment.is_empty() { None } else { Some(comment) },
        "at": at,
    })).collect();

    Ok(Json(json!({
        "market_id": id,
        "total_yes_eur": yes_total.round_dp(2).to_string(),
        "total_no_eur": no_total.round_dp(2).to_string(),
        "predictions": feed,
        "count": feed.len(),
    })))
}
