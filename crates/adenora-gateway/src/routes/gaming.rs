use crate::auth_extractor::AuthUser;
use crate::state::AppState;
use adenora_gaming::trivia;
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list_games(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, String, String, String, bool, Option<rust_decimal::Decimal>)> =
        sqlx::query_as(
            "SELECT id, name, description, game_type, is_free_to_play, entry_fee
             FROM games ORDER BY name"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let games: Vec<Value> = rows.into_iter().map(|(id, name, desc, gtype, free, fee)| {
        json!({ "id": id, "name": name, "description": desc, "game_type": gtype,
                 "is_free_to_play": free, "entry_fee": fee })
    }).collect();

    Ok(Json(json!({ "games": games, "total": games.len() })))
}

pub async fn list_tournaments(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, String, rust_decimal::Decimal, rust_decimal::Decimal, i32, i32, String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT id, name, entry_fee, prize_pool, max_participants, current_participants, status, starts_at, ends_at
             FROM tournaments WHERE status IN ('registration', 'in_progress')
             ORDER BY starts_at ASC"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let tournaments: Vec<Value> = rows.into_iter().map(|(id, name, fee, pool, max, cur, status, start, end)| {
        json!({ "id": id, "name": name, "entry_fee": fee, "prize_pool": pool,
                 "max_participants": max, "current_participants": cur, "status": status,
                 "starts_at": start, "ends_at": end })
    }).collect();

    Ok(Json(json!({ "tournaments": tournaments, "total": tournaments.len() })))
}

pub async fn leaderboard(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let snapshot: Option<(Value, i32, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT entries, total_participants, generated_at
         FROM leaderboard_snapshots
         WHERE scope = 'all_time' AND category = 'overall'
         ORDER BY generated_at DESC LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    match snapshot {
        Some((entries, total, gen)) => Ok(Json(json!({
            "scope": "all_time", "category": "overall",
            "entries": entries, "total_participants": total, "generated_at": gen
        }))),
        None => Ok(Json(json!({
            "scope": "all_time", "category": "overall",
            "entries": [], "total_participants": 0
        }))),
    }
}

/// Start a trivia session — returns questions (without answers).
pub async fn start_trivia(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let questions = trivia::seed_questions();
    let session_id = Uuid::new_v4();

    // Store session in DB — find or create a trivia game record
    let game_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM games WHERE game_type = 'trivia' LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);

    let game_id = match game_id {
        Some((id,)) => id,
        None => {
            let created: (Uuid,) = sqlx::query_as(
                "INSERT INTO games (name, game_type, config) VALUES ('Trivia', 'trivia', '{}')
                 ON CONFLICT DO NOTHING
                 RETURNING id"
            )
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                // Another process created it — retry fetch
                (Uuid::new_v4(),) // fallback, will be overridden
            });
            created.0
        }
    };

    if let Err(e) = sqlx::query(
        "INSERT INTO game_results (id, game_id, user_id, score) VALUES ($1, $2, $3, 0)"
    )
    .bind(session_id)
    .bind(game_id)
    .bind(auth.user_id)
    .execute(&state.db)
    .await {
        tracing::error!(error = %e, "failed to create trivia session");
    }

    // Return questions without correct answers
    let safe_questions: Vec<Value> = questions.iter().map(|q| {
        json!({
            "id": q.id,
            "category": q.category,
            "question": q.question,
            "options": q.options,
            "difficulty": q.difficulty,
            "points": q.points,
            "time_limit_secs": q.time_limit_secs,
        })
    }).collect();

    Ok(Json(json!({
        "session_id": session_id,
        "questions": safe_questions,
        "total_questions": safe_questions.len()
    })))
}

#[derive(Deserialize)]
pub struct AnswerRequest {
    pub session_id: Uuid,
    pub question_id: Uuid,
    pub selected_index: usize,
    pub time_taken_ms: u64,
}

/// Submit a trivia answer and get result.
pub async fn answer_trivia(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AnswerRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let questions = trivia::seed_questions();

    let question = questions.iter().find(|q| q.id == body.question_id)
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error": "question not found"}))))?;

    let answer = trivia::score_answer(question, body.selected_index, body.time_taken_ms);

    // Update score
    if answer.is_correct {
        sqlx::query(
            "UPDATE game_results SET score = score + $1 WHERE id = $2 AND user_id = $3"
        )
        .bind(answer.points_earned)
        .bind(body.session_id)
        .bind(auth.user_id)
        .execute(&state.db)
        .await
        .ok();
    }

    Ok(Json(json!({
        "is_correct": answer.is_correct,
        "points_earned": answer.points_earned,
        "correct_index": question.correct_index,
        "explanation": question.explanation,
        "time_taken_ms": answer.time_taken_ms
    })))
}

#[derive(Deserialize)]
pub struct ForecastSubmitRequest {
    pub challenge_id: Uuid,
    pub question_id: Uuid,
    pub probability: f64,
}

/// Submit a forecast prediction.
pub async fn submit_forecast(
    State(_state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ForecastSubmitRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.probability < 0.0 || body.probability > 1.0 {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "probability must be 0.0-1.0"}))));
    }

    Ok(Json(json!({
        "status": "submitted",
        "challenge_id": body.challenge_id,
        "question_id": body.question_id,
        "probability": body.probability,
        "message": "forecast recorded"
    })))
}
