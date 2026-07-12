use crate::auth_extractor::AuthUser;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

/// Row for the market list query (no resolved outcome column).
type MarketListRow = (
    Uuid, String, String, String, Value, String,
    chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>,
);

/// Row for a single market including its resolved outcome.
type MarketDetailRow = (
    Uuid, String, String, String, Value, String, Option<String>,
    chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>,
);

#[derive(Deserialize)]
pub struct MarketQuery {
    pub status: Option<String>,
    // Interface scaffolding: retained for planned category/country market filtering.
    #[allow(dead_code)]
    pub category: Option<String>,
    #[allow(dead_code)]
    pub country: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_markets(
    State(state): State<AppState>,
    Query(q): Query<MarketQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = q.limit.unwrap_or(50).min(100);
    let offset = q.offset.unwrap_or(0);
    let status = q.status.unwrap_or_else(|| "active".to_string());

    let rows: Vec<MarketListRow> =
        sqlx::query_as(
            "SELECT id, question, description, category, outcomes, status, opens_at, closes_at
             FROM markets
             WHERE status = $1
             ORDER BY closes_at ASC
             LIMIT $2 OFFSET $3"
        )
        .bind(&status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let markets: Vec<Value> = rows
        .into_iter()
        .map(|(id, question, desc, cat, outcomes, status, opens, closes)| {
            json!({
                "id": id,
                "question": question,
                "description": desc,
                "category": cat,
                "outcomes": outcomes,
                "status": status,
                "opens_at": opens,
                "closes_at": closes
            })
        })
        .collect();

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM markets WHERE status = $1")
        .bind(&status)
        .fetch_one(&state.db)
        .await
        .unwrap_or((0,));

    Ok(Json(json!({
        "markets": markets,
        "total": total.0,
        "limit": limit,
        "offset": offset
    })))
}

pub async fn get_market(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let row: Option<MarketDetailRow> =
        sqlx::query_as(
            "SELECT id, question, description, category, outcomes, status, outcome, opens_at, closes_at
             FROM markets WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    match row {
        Some((id, q, desc, cat, outcomes, status, outcome, opens, closes)) => {
            Ok(Json(json!({
                "id": id,
                "question": q,
                "description": desc,
                "category": cat,
                "outcomes": outcomes,
                "status": status,
                "outcome": outcome,
                "opens_at": opens,
                "closes_at": closes
            })))
        }
        None => Err((StatusCode::NOT_FOUND, Json(json!({"error": "market not found"})))),
    }
}

pub async fn get_orderbook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let engine = state.get_engine(id).await;
    let snapshots = engine.snapshots().await;

    Ok(Json(json!({
        "market_id": id,
        "people_book": {
            "bids": snapshots.people.bids,
            "asks": snapshots.people.asks,
            "best_bid": snapshots.people.best_bid,
            "best_ask": snapshots.people.best_ask,
            "spread": snapshots.people.spread
        },
        "bot_book": {
            "bids": snapshots.bot.bids,
            "asks": snapshots.bot.asks,
            "best_bid": snapshots.bot.best_bid,
            "best_ask": snapshots.bot.best_ask,
            "spread": snapshots.bot.spread
        }
    })))
}

#[derive(Deserialize, Serialize)]
pub struct ProposeMarketRequest {
    pub question: String,
    pub description: String,
    pub category: String,
    pub outcomes: Option<Vec<String>>,
    pub closes_at: String,
    pub country_codes: Option<Vec<String>>,
}

pub async fn propose_market(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ProposeMarketRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate required fields
    if body.question.trim().is_empty() || body.question.len() > 500 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "question must be 1-500 characters"}))));
    }
    if body.description.trim().is_empty() || body.description.len() > 5000 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "description must be 1-5000 characters"}))));
    }

    // Content policy — reject markets about individual death, suffering, assassination, etc.
    if !adenora_markets::proposal::passes_content_policy(&body.question, &body.description) {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "market proposal violates content policy"}))));
    }

    let request_json = serde_json::to_value(&body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let proposal_id: (Uuid,) = sqlx::query_as(
        "INSERT INTO market_proposals (proposer_id, request) VALUES ($1, $2) RETURNING id"
    )
    .bind(auth.user_id)
    .bind(&request_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    tracing::info!(
        user_id = %auth.user_id,
        proposal_id = %proposal_id.0,
        question = %body.question,
        "market proposal submitted"
    );

    Ok(Json(json!({
        "status": "proposed",
        "proposal_id": proposal_id.0,
        "message": "market proposal submitted for review"
    })))
}
