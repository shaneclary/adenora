use crate::auth_extractor::AuthUser;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterBotRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_open_source: Option<bool>,
    pub source_url: Option<String>,
}

pub async fn list_bots(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, String, Option<String>, String, i64, Decimal, f64)> =
        sqlx::query_as(
            "SELECT id, name, description, status, total_trades, total_pnl, win_rate
             FROM bots WHERE status = 'active' ORDER BY total_pnl DESC LIMIT 100"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let bots: Vec<Value> = rows.into_iter().map(|(id, name, desc, status, trades, pnl, wr)| {
        json!({ "id": id, "name": name, "description": desc, "status": status,
                 "total_trades": trades, "total_pnl": pnl, "win_rate": wr })
    }).collect();

    Ok(Json(json!({ "bots": bots, "total": bots.len() })))
}

pub async fn register_bot(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<RegisterBotRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let max_len = state.config.bot_arena.bot_name_max_length;

    if body.name.is_empty() || body.name.len() > max_len {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": format!("bot name must be 1-{max_len} characters")}))));
    }

    if !body.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "bot name: alphanumeric, hyphens, underscores only"}))));
    }

    // Check bot count limit
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM bots WHERE owner_id = $1 AND status = 'active'"
    )
    .bind(auth.user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    if count.0 >= state.config.bot_arena.max_bots_per_user as i64 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({
            "error": format!("max {} bots per user", state.config.bot_arena.max_bots_per_user)
        }))));
    }

    let is_open_source = body.is_open_source.unwrap_or(false);

    let bot_id: (Uuid,) = sqlx::query_as(
        "INSERT INTO bots (owner_id, name, description, is_open_source, source_url)
         VALUES ($1, $2, $3, $4, $5) RETURNING id"
    )
    .bind(auth.user_id)
    .bind(&body.name)
    .bind(&body.description)
    .bind(is_open_source)
    .bind(&body.source_url)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        let msg = if e.to_string().contains("unique") { "bot name already taken" } else { "registration failed" };
        (StatusCode::CONFLICT, Json(json!({"error": msg})))
    })?;

    Ok(Json(json!({
        "status": "registered",
        "bot_id": bot_id.0,
        "name": body.name,
        "message": "bot registered — unleash it on Bot Battle markets"
    })))
}

pub async fn get_bot(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let row: Option<(Uuid, String, Option<String>, bool, String, i64, Decimal, Decimal, f64, f64, f64, i32, i32, i32)> =
        sqlx::query_as(
            "SELECT id, name, description, is_open_source, status,
                    total_trades, total_volume, total_pnl, win_rate, sharpe_ratio, max_drawdown,
                    markets_traded, tournaments_entered, tournaments_won
             FROM bots WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    match row {
        Some((id, name, desc, open_src, status, trades, vol, pnl, wr, sharpe, dd, mkts, te, tw)) => {
            Ok(Json(json!({
                "id": id, "name": name, "description": desc, "is_open_source": open_src, "status": status,
                "stats": { "total_trades": trades, "total_volume": vol, "total_pnl": pnl,
                           "win_rate": wr, "sharpe_ratio": sharpe, "max_drawdown": dd,
                           "markets_traded": mkts, "tournaments_entered": te, "tournaments_won": tw }
            })))
        }
        None => Err((StatusCode::NOT_FOUND, Json(json!({"error": "bot not found"})))),
    }
}

pub async fn bot_leaderboard(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, String, Decimal, f64, i64, f64)> = sqlx::query_as(
        "SELECT id, name, total_pnl, win_rate, total_trades, sharpe_ratio
         FROM bots WHERE status = 'active' AND total_trades > 0
         ORDER BY total_pnl DESC LIMIT 50"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let entries: Vec<Value> = rows.into_iter().enumerate().map(|(i, (id, name, pnl, wr, trades, sharpe))| {
        json!({ "rank": i + 1, "bot_id": id, "name": name, "total_pnl": pnl,
                 "win_rate": wr, "total_trades": trades, "sharpe_ratio": sharpe })
    }).collect();

    Ok(Json(json!({ "scope": "all_time", "bots": entries, "total": entries.len() })))
}

pub async fn list_bot_tournaments(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, String, Decimal, Decimal, i32, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT id, name, entry_fee, prize_pool, max_bots, status, starts_at, ends_at
             FROM bot_tournaments ORDER BY starts_at DESC LIMIT 20"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let tournaments: Vec<Value> = rows.into_iter().map(|(id, name, fee, pool, max, status, start, end)| {
        json!({ "id": id, "name": name, "entry_fee": fee, "prize_pool": pool,
                 "max_bots": max, "status": status, "starts_at": start, "ends_at": end })
    }).collect();

    Ok(Json(json!({ "tournaments": tournaments, "total": tournaments.len() })))
}
