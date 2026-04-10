use crate::state::AppState;
use axum::{Json, extract::State, http::StatusCode};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, String, String, String, rust_decimal::Decimal, String, bool)> =
        sqlx::query_as(
            "SELECT id, name, description, category, total_received, currency, is_active
             FROM charity_projects WHERE is_active = true
             ORDER BY total_received DESC"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let projects: Vec<Value> = rows
        .into_iter()
        .map(|(id, name, desc, cat, received, currency, active)| {
            json!({
                "id": id,
                "name": name,
                "description": desc,
                "category": cat,
                "total_received": received,
                "currency": currency,
                "is_active": active
            })
        })
        .collect();

    Ok(Json(json!({ "projects": projects, "total": projects.len() })))
}

pub async fn get_ledger(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, Uuid, String, rust_decimal::Decimal, String, String, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT cl.id, cl.project_id, cl.source, cl.amount, cl.currency, cl.description, cl.created_at
             FROM charity_ledger cl
             ORDER BY cl.created_at DESC
             LIMIT 200"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let entries: Vec<Value> = rows
        .into_iter()
        .map(|(id, proj_id, source, amount, currency, desc, created)| {
            json!({
                "id": id,
                "project_id": proj_id,
                "source": source,
                "amount": amount,
                "currency": currency,
                "description": desc,
                "created_at": created
            })
        })
        .collect();

    let total: (rust_decimal::Decimal,) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount), 0) FROM charity_ledger"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or((rust_decimal::Decimal::ZERO,));

    Ok(Json(json!({
        "entries": entries,
        "total_distributed": total.0,
        "message": "every cent is tracked and auditable"
    })))
}

pub async fn funding_summary(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let total: (rust_decimal::Decimal,) = sqlx::query_as(
        "SELECT COALESCE(SUM(total_received), 0) FROM charity_projects"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or((rust_decimal::Decimal::ZERO,));

    let by_source: Vec<(String, rust_decimal::Decimal)> = sqlx::query_as(
        "SELECT source, COALESCE(SUM(amount), 0) FROM charity_ledger GROUP BY source"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut from_predictions = rust_decimal::Decimal::ZERO;
    let mut from_lottery = rust_decimal::Decimal::ZERO;
    let mut from_tournaments = rust_decimal::Decimal::ZERO;
    let mut from_donations = rust_decimal::Decimal::ZERO;
    let mut from_help = rust_decimal::Decimal::ZERO;

    for (source, amount) in &by_source {
        match source.as_str() {
            "prediction_fee" => from_predictions = *amount,
            "lottery_revenue" => from_lottery = *amount,
            "tournament_fee" => from_tournaments = *amount,
            "direct_donation" => from_donations = *amount,
            "help_game" => from_help = *amount,
            _ => {}
        }
    }

    let project_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM charity_projects WHERE is_active = true"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    Ok(Json(json!({
        "total_raised": total.0,
        "from_predictions": from_predictions,
        "from_lottery": from_lottery,
        "from_tournaments": from_tournaments,
        "from_donations": from_donations,
        "from_help_games": from_help,
        "projects_funded": project_count.0
    })))
}
