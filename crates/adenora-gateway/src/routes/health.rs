use crate::state::AppState;
use axum::{Json, extract::State};
use serde_json::{json, Value};

pub async fn root() -> Json<Value> {
    Json(json!({
        "service": "adenora",
        "tagline": "Where Value Flows",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "ok"
    }))
}

pub async fn healthz() -> Json<Value> {
    Json(json!({ "ok": true }))
}

pub async fn status(State(state): State<AppState>) -> Json<Value> {
    let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let market_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM markets WHERE status = 'active'")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let trade_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trades")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let bot_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bots WHERE status = 'active'")
        .fetch_one(&state.db).await.unwrap_or((0,));
    let charity_total: (rust_decimal::Decimal,) = sqlx::query_as("SELECT COALESCE(SUM(total_received), 0) FROM charity_projects")
        .fetch_one(&state.db).await.unwrap_or((rust_decimal::Decimal::ZERO,));

    let engine_count = state.engines.read().await.len();

    Json(json!({
        "service": "adenora",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "ok",
        "stats": {
            "users": user_count.0,
            "active_markets": market_count.0,
            "total_trades": trade_count.0,
            "active_bots": bot_count.0,
            "charity_raised": charity_total.0,
            "active_engines": engine_count
        },
        "background_services": [
            "batch_auction_loop (500ms)",
            "settlement_loop (10s)",
            "draw_executor_loop (30s)",
            "leaderboard_loop (5min)",
            "recovery_loop (60s)"
        ]
    }))
}
