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
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PlaceOrderRequest {
    pub market_id: Uuid,
    pub side: String,
    pub action: String,
    pub price_cents: u32,
    pub quantity: u32,
    pub time_in_force: Option<String>,
    pub mode: Option<String>,
    pub bot_id: Option<Uuid>,
}

pub async fn place_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<PlaceOrderRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate price
    if body.price_cents < state.config.trading.min_price_cents
        || body.price_cents > state.config.trading.max_price_cents
    {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!(
            "price must be {}-{} cents",
            state.config.trading.min_price_cents,
            state.config.trading.max_price_cents
        )}))));
    }

    if body.quantity == 0 || body.quantity > state.config.trading.max_contracts_per_order {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!(
            "quantity must be 1-{}",
            state.config.trading.max_contracts_per_order
        )}))));
    }

    let side = match body.side.as_str() {
        "yes" => Side::Yes,
        "no" => Side::No,
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "side must be 'yes' or 'no'"})))),
    };

    let action = match body.action.as_str() {
        "buy" => Action::Buy,
        "sell" => Action::Sell,
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "action must be 'buy' or 'sell'"})))),
    };

    let tif = match body.time_in_force.as_deref().unwrap_or("ioc") {
        "ioc" => TimeInForce::Ioc,
        "gtc" => TimeInForce::Gtc,
        "fok" => TimeInForce::Fok,
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "time_in_force must be 'ioc', 'gtc', or 'fok'"})))),
    };

    let (mode, mode_db, mode_response) = match body.mode.as_deref().unwrap_or("people") {
        "people" | "human" => {
            if auth.is_bot {
                return Err((StatusCode::FORBIDDEN, Json(json!({
                    "error": "bot accounts cannot trade in Human Markets — use 'ultimate'"
                }))));
            }
            (MarketMode::People, "people", "human")
        }
        "unlimited" | "ultimate" | "bot" => (MarketMode::Unlimited, "unlimited", "ultimate"),
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "mode must be 'people', 'human', 'unlimited', 'ultimate', or 'bot'"})))),
    };

    // Enforce KYC + self-exclusion + responsible gambling
    let user_check: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT kyc_status, self_exclusion_until FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if let Some((kyc_status, exclusion_until)) = &user_check {
        // KYC must be verified to trade
        if kyc_status != "verified" && kyc_status != "approved" {
            return Err((StatusCode::FORBIDDEN, Json(json!({
                "error": "KYC verification required before trading",
                "kyc_status": kyc_status
            }))));
        }
        // Self-exclusion check
        if let Some(until) = exclusion_until {
            if chrono::Utc::now() < *until {
                return Err((StatusCode::FORBIDDEN, Json(json!({
                    "error": "self-exclusion active",
                    "until": until
                }))));
            }
        }
    } else {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "user not found"}))));
    }

    // Verify market is active
    let market_status: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM markets WHERE id = $1"
    )
    .bind(body.market_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    match market_status {
        Some((s,)) if s == "active" => {}
        Some(_) => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "market is not active"})))),
        None => return Err((StatusCode::NOT_FOUND, Json(json!({"error": "market not found"})))),
    }

    // Check balance — cost is price_cents * quantity for buys
    let cost_cents = if action == Action::Buy {
        body.price_cents as i64 * body.quantity as i64
    } else {
        0 // Selling existing position
    };

    if action == Action::Sell {
        // Sells must be backed by an existing position on the same side/mode.
        // Without this check a user could sell contracts they never bought and
        // receive the proceeds for free (naked selling).
        let held: Option<(i32,)> = sqlx::query_as(
            "SELECT quantity FROM positions
             WHERE user_id = $1 AND market_id = $2 AND side = $3 AND mode = $4"
        )
        .bind(auth.user_id)
        .bind(body.market_id)
        .bind(&body.side)
        .bind(mode_db)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        let held_qty = held.map(|(q,)| q).unwrap_or(0);
        if held_qty < body.quantity as i32 {
            return Err((StatusCode::BAD_REQUEST, Json(json!({
                "error": "insufficient position to sell",
                "held": held_qty,
                "requested": body.quantity
            }))));
        }
    }

    if cost_cents > 0 {
        let cost = Decimal::new(cost_cents, 2);

        // Atomic check-and-reserve — single UPDATE prevents race conditions
        let reserved: Option<(Decimal,)> = sqlx::query_as(
            "UPDATE wallets SET available = available - $1, reserved = reserved + $1, updated_at = NOW()
             WHERE user_id = $2 AND currency = 'EUR' AND available >= $1
             RETURNING available"
        )
        .bind(cost)
        .bind(auth.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        if reserved.is_none() {
            // Get current balance for error message
            let available: Decimal = sqlx::query_as::<_, (Decimal,)>(
                "SELECT COALESCE(available, 0) FROM wallets WHERE user_id = $1 AND currency = 'EUR'"
            )
            .bind(auth.user_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .map(|(a,)| a)
            .unwrap_or(Decimal::ZERO);

            return Err((StatusCode::BAD_REQUEST, Json(json!({
                "error": "insufficient balance",
                "need": cost.to_string(),
                "have": available.to_string()
            }))));
        }
    }

    let order = Order {
        id: Uuid::new_v4(),
        user_id: auth.user_id,
        market_id: body.market_id,
        side,
        action,
        price_cents: body.price_cents,
        quantity: body.quantity,
        filled_quantity: 0,
        time_in_force: tif,
        status: OrderStatus::Pending,
        mode,
        bot_id: body.bot_id,
        created_at: Utc::now(),
    };

    let _order_id = order.id;

    // Persist order
    sqlx::query(
        "INSERT INTO orders (id, user_id, market_id, side, action, price_cents, quantity, time_in_force, status, mode, bot_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(order.id)
    .bind(auth.user_id)
    .bind(order.market_id)
    .bind(&body.side)
    .bind(&body.action)
    .bind(order.price_cents as i32)
    .bind(order.quantity as i32)
    .bind(body.time_in_force.as_deref().unwrap_or("ioc"))
    .bind("pending")
    .bind(mode_db)
    .bind(order.bot_id)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    // Submit to engine
    let engine = state.get_engine(body.market_id).await;
    let result = engine.submit(order).await;

    match result {
        adenora_orderbook::engine::SubmitResult::Queued { order_id } => {
            Ok(Json(json!({
                "status": "queued",
                "order_id": order_id,
                "mode": mode_response,
                "message": "order queued for next batch auction"
            })))
        }
        adenora_orderbook::engine::SubmitResult::Executed { order_id, trades, cancelled } => {
            // Persist fills first, then release funds for anything the engine
            // dropped (unfilled IOC/FOK, self-trade) using post-fill quantities.
            persist_trades(&state, &trades).await;
            if !cancelled.is_empty() {
                release_cancelled_orders(&state, &cancelled).await;
            }

            Ok(Json(json!({
                "status": "executed",
                "order_id": order_id,
                "mode": mode_response,
                "trades": trades.len(),
                "fills": trades.iter().map(|t| json!({
                    "trade_id": t.id,
                    "price_cents": t.price_cents,
                    "quantity": t.quantity,
                    "fee": if t.buyer_user_id == auth.user_id { t.buyer_fee.to_string() } else { t.seller_fee.to_string() }
                })).collect::<Vec<_>>()
            })))
        }
    }
}

/// Release reserved funds and mark cancelled for orders the matching engine
/// dropped internally (self-trade prevention, unfilled IOC/FOK). Only buys
/// reserve funds; the unfilled portion is what gets released, computed from the
/// current `filled_quantity` (so this must run after `persist_trades`).
pub async fn release_cancelled_orders(state: &AppState, cancelled: &[adenora_common::types::OrderId]) {
    for order_id in cancelled {
        let row: Option<(Uuid, i32, i32, i32, String)> = sqlx::query_as(
            "SELECT user_id, price_cents, quantity, filled_quantity, action
             FROM orders WHERE id = $1 AND status IN ('pending', 'partial_fill')"
        )
        .bind(order_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();

        let Some((user_id, price_cents, quantity, filled, action)) = row else { continue };

        sqlx::query("UPDATE orders SET status = 'cancelled', updated_at = NOW() WHERE id = $1")
            .bind(order_id)
            .execute(&state.db)
            .await
            .ok();

        let unfilled = quantity - filled;
        if action == "buy" && unfilled > 0 {
            let release = Decimal::new(price_cents as i64 * unfilled as i64, 2);
            sqlx::query(
                "UPDATE wallets SET reserved = GREATEST(reserved - $1, 0), available = available + $1, updated_at = NOW()
                 WHERE user_id = $2 AND currency = 'EUR'"
            )
            .bind(release)
            .bind(user_id)
            .execute(&state.db)
            .await
            .ok();
        }
    }
}

/// Persist trades to DB, update positions, record fees in charity ledger.
pub async fn persist_trades(state: &AppState, trades: &[adenora_orderbook::book::Trade]) {
    for trade in trades {
        // Insert trade
        if let Err(e) = sqlx::query(
            "INSERT INTO trades (id, market_id, batch_id, buyer_order_id, seller_order_id, buyer_user_id, seller_user_id, side, price_cents, quantity, buyer_fee, seller_fee, mode)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(trade.id)
        .bind(trade.market_id)
        .bind(trade.batch_id)
        .bind(trade.buyer_order_id)
        .bind(trade.seller_order_id)
        .bind(trade.buyer_user_id)
        .bind(trade.seller_user_id)
        .bind(format!("{:?}", trade.side).to_lowercase())
        .bind(trade.price_cents as i32)
        .bind(trade.quantity as i32)
        .bind(trade.buyer_fee)
        .bind(trade.seller_fee)
        .bind(format!("{:?}", trade.mode).to_lowercase())
        .execute(&state.db)
        .await {
            tracing::error!(trade_id = %trade.id, error = %e, "failed to persist trade");
            continue; // Skip dependent operations if trade insert failed
        }

        // Update buyer position (increase)
        let side_str = format!("{:?}", trade.side).to_lowercase();
        let mode_str = format!("{:?}", trade.mode).to_lowercase();
        let price_dec = Decimal::new(trade.price_cents as i64, 2);

        if let Err(e) = sqlx::query(
            "INSERT INTO positions (user_id, market_id, side, quantity, avg_price, mode)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id, market_id, side, mode)
             DO UPDATE SET
                 avg_price = (positions.avg_price * positions.quantity + $5 * $4) / (positions.quantity + $4),
                 quantity = positions.quantity + $4,
                 updated_at = NOW()"
        )
        .bind(trade.buyer_user_id)
        .bind(trade.market_id)
        .bind(&side_str)
        .bind(trade.quantity as i32)
        .bind(price_dec)
        .bind(&mode_str)
        .execute(&state.db)
        .await {
            tracing::error!(trade_id = %trade.id, user_id = %trade.buyer_user_id, error = %e, "failed to update buyer position");
        }

        // Debit buyer: release the reserved trade value and take the fee from
        // available. Only `trade_value` was ever reserved (fees are charged at
        // fill time), so debiting the fee from `reserved` would drive it negative.
        let trade_value = Decimal::new(trade.price_cents as i64 * trade.quantity as i64, 2);
        if let Err(e) = sqlx::query(
            "UPDATE wallets SET reserved = GREATEST(reserved - $1, 0), available = available - $2, updated_at = NOW()
             WHERE user_id = $3 AND currency = 'EUR'"
        )
        .bind(trade_value)
        .bind(trade.buyer_fee)
        .bind(trade.buyer_user_id)
        .execute(&state.db)
        .await {
            tracing::error!(trade_id = %trade.id, error = %e, "failed to debit buyer funds");
        }

        // Decrement the seller's position on the side they sold. Without this a
        // user could sell the same position repeatedly and be paid again at
        // settlement for contracts they no longer hold.
        if let Err(e) = sqlx::query(
            "UPDATE positions SET quantity = GREATEST(quantity - $1, 0), updated_at = NOW()
             WHERE user_id = $2 AND market_id = $3 AND side = $4 AND mode = $5"
        )
        .bind(trade.quantity as i32)
        .bind(trade.seller_user_id)
        .bind(trade.market_id)
        .bind(&side_str)
        .bind(&mode_str)
        .execute(&state.db)
        .await {
            tracing::error!(trade_id = %trade.id, error = %e, "failed to decrement seller position");
        }

        if let Err(e) = sqlx::query(
            "UPDATE wallets SET available = available + $1, updated_at = NOW()
             WHERE user_id = $2 AND currency = 'EUR'"
        )
        .bind(trade_value - trade.seller_fee)
        .bind(trade.seller_user_id)
        .execute(&state.db)
        .await {
            tracing::error!(trade_id = %trade.id, error = %e, "failed to credit seller");
        }

        // Record fees in charity ledger
        let total_fee = trade.buyer_fee + trade.seller_fee;
        let fee_split = fees::split_fee(total_fee);

        if fee_split.charity > Decimal::ZERO {
            let project_id: Option<(Uuid,)> = sqlx::query_as(
                "SELECT charity_project_id FROM markets WHERE id = $1 AND charity_project_id IS NOT NULL"
            )
            .bind(trade.market_id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);

            if let Some((pid,)) = project_id {
                if let Err(e) = sqlx::query(
                    "INSERT INTO charity_ledger (project_id, source, amount, currency, description, reference_id)
                     VALUES ($1, 'prediction_fee', $2, 'EUR', $3, $4)"
                )
                .bind(pid)
                .bind(fee_split.charity)
                .bind(format!("fee split from trade on market {}", trade.market_id))
                .bind(trade.id.to_string())
                .execute(&state.db)
                .await {
                    tracing::error!(trade_id = %trade.id, error = %e, "failed to write charity ledger entry");
                }

                if let Err(e) = sqlx::query("UPDATE charity_projects SET total_received = total_received + $1 WHERE id = $2")
                    .bind(fee_split.charity)
                    .bind(pid)
                    .execute(&state.db)
                    .await {
                    tracing::error!(trade_id = %trade.id, error = %e, "failed to update charity project total");
                }
            }
        }

        // Update order statuses
        for (order_id, filled) in [
            (trade.buyer_order_id, trade.quantity),
            (trade.seller_order_id, trade.quantity),
        ] {
            if let Err(e) = sqlx::query(
                "UPDATE orders SET filled_quantity = filled_quantity + $1,
                 status = CASE WHEN filled_quantity + $1 >= quantity THEN 'filled' ELSE 'partial_fill' END,
                 updated_at = NOW()
                 WHERE id = $2"
            )
            .bind(filled as i32)
            .bind(order_id)
            .execute(&state.db)
            .await {
                tracing::error!(order_id = %order_id, error = %e, "failed to update order fill status");
            }
        }

        // Record transactions for audit trail
        for (user_id, amount, fee, tx_type) in [
            (trade.buyer_user_id, trade_value, trade.buyer_fee, "trade_buy"),
            (trade.seller_user_id, trade_value, trade.seller_fee, "trade_sell"),
        ] {
            if let Err(e) = sqlx::query(
                "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, reference_type, reference_id, description)
                 VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = 'EUR' LIMIT 1), $2, $3, 'EUR', 'trade', $4, $5)"
            )
            .bind(user_id)
            .bind(tx_type)
            .bind(amount)
            .bind(trade.id)
            .bind(format!("{} {}x @ {}c, fee {}", tx_type, trade.quantity, trade.price_cents, fee))
            .execute(&state.db)
            .await {
                tracing::error!(trade_id = %trade.id, user_id = %user_id, error = %e, "failed to record transaction audit trail");
            }
        }
    }
}

pub async fn cancel_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Verify ownership
    let row: Option<(Uuid, String, Uuid, i32, i32, i32, String)> = sqlx::query_as(
        "SELECT market_id, mode, user_id, price_cents, quantity, filled_quantity, action
         FROM orders WHERE id = $1 AND status IN ('pending', 'partial_fill')"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let (market_id, mode_str, owner_id, price_cents, quantity, filled, action) = row
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error": "order not found or not cancellable"}))))?;

    if owner_id != auth.user_id && !auth.is_admin {
        return Err((StatusCode::FORBIDDEN, Json(json!({"error": "not your order"}))));
    }

    let mode = if mode_str == "unlimited" { MarketMode::Unlimited } else { MarketMode::People };
    let engine = state.get_engine(market_id).await;
    engine.cancel(id, mode).await;

    sqlx::query("UPDATE orders SET status = 'cancelled', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .ok();

    // Release reserved funds for unfilled portion — only buys reserve funds.
    // Sells reserve nothing, so releasing on cancel would credit money that was
    // never reserved. Release to the order's owner, not the caller (an admin may
    // cancel another user's order).
    let unfilled = quantity - filled;
    let released = if action == "buy" && unfilled > 0 {
        let release = Decimal::new(price_cents as i64 * unfilled as i64, 2);
        sqlx::query(
            "UPDATE wallets SET reserved = GREATEST(reserved - $1, 0), available = available + $1, updated_at = NOW()
             WHERE user_id = $2 AND currency = 'EUR'"
        )
        .bind(release)
        .bind(owner_id)
        .execute(&state.db)
        .await
        .ok();
        release
    } else {
        Decimal::ZERO
    };

    Ok(Json(json!({
        "order_id": id,
        "status": "cancelled",
        "released_funds": released
    })))
}

pub async fn get_positions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, Uuid, String, i32, Decimal, String)> = sqlx::query_as(
        "SELECT p.id, p.market_id, p.side, p.quantity, p.avg_price, p.mode
         FROM positions p
         WHERE p.user_id = $1 AND p.quantity > 0
         ORDER BY p.updated_at DESC"
    )
    .bind(auth.user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut total_value = Decimal::ZERO;
    let mut positions = Vec::new();

    for (id, market_id, side, qty, avg_price, mode) in rows {
        let value = avg_price * Decimal::from(qty);
        total_value += value;
        positions.push(json!({
            "id": id,
            "market_id": market_id,
            "side": side,
            "quantity": qty,
            "avg_price": avg_price,
            "value": value,
            "mode": mode
        }));
    }

    Ok(Json(json!({
        "positions": positions,
        "total": positions.len(),
        "total_value": total_value
    })))
}
