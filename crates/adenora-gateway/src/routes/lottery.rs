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
pub struct BuyTicketRequest {
    pub numbers: Vec<i32>,
}

pub async fn list_lotteries(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, String, String, String, Decimal, String, String, i32)> =
        sqlx::query_as(
            "SELECT id, name, description, game_type, ticket_price, currency, status, prize_pct
             FROM lotteries WHERE status = 'active' ORDER BY name"
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let lotteries: Vec<Value> = rows
        .into_iter()
        .map(|(id, name, desc, gtype, price, currency, status, pct)| {
            json!({ "id": id, "name": name, "description": desc, "game_type": gtype,
                     "ticket_price": price, "currency": currency, "status": status, "prize_pct": pct })
        })
        .collect();

    Ok(Json(json!({ "lotteries": lotteries, "total": lotteries.len() })))
}

pub async fn buy_ticket(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(lottery_id): Path<Uuid>,
    Json(body): Json<BuyTicketRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Check self-exclusion
    let exclusion: Option<(Option<chrono::DateTime<chrono::Utc>>,)> = sqlx::query_as(
        "SELECT self_exclusion_until FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some((Some(until),)) = exclusion {
        if chrono::Utc::now() < until {
            return Err((StatusCode::FORBIDDEN, Json(json!({
                "error": "self-exclusion active",
                "until": until
            }))));
        }
    }

    let lottery: Option<(Uuid, Decimal, String, Uuid, i32, i32, i32)> = sqlx::query_as(
        "SELECT id, ticket_price, currency, project_id, prize_pct, project_pct, company_pct
         FROM lotteries WHERE id = $1 AND status = 'active'"
    )
    .bind(lottery_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let (_, ticket_price, currency, project_id, prize_pct, project_pct, _company_pct) = lottery
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error": "lottery not found or not active"}))))?;

    let draw: Option<(Uuid, i32)> = sqlx::query_as(
        "SELECT id, draw_number FROM draws
         WHERE lottery_id = $1 AND status IN ('scheduled', 'open')
         ORDER BY scheduled_at ASC LIMIT 1"
    )
    .bind(lottery_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let (draw_id, draw_number) = draw
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({"error": "no open draw available"}))))?;

    // Check if free entry (after N consecutive losses)
    let consecutive_losses: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tickets t
         JOIN draws d ON d.id = t.draw_id
         LEFT JOIN winners w ON w.ticket_id = t.id
         WHERE t.user_id = $1 AND t.lottery_id = $2 AND d.status = 'paid' AND w.id IS NULL
         AND t.purchased_at > COALESCE(
             (SELECT MAX(w2.paid_at) FROM winners w2
              JOIN tickets t2 ON t2.id = w2.ticket_id
              WHERE t2.user_id = $1 AND t2.lottery_id = $2), '1970-01-01'
         )"
    )
    .bind(auth.user_id)
    .bind(lottery_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or((0,));

    let is_free = consecutive_losses.0 >= state.config.lottery.free_entry_after_losses as i64;

    if !is_free {
        // Debit wallet
        let result = sqlx::query(
            "UPDATE wallets SET available = available - $1, updated_at = NOW()
             WHERE user_id = $2 AND currency = $3 AND available >= $1"
        )
        .bind(ticket_price)
        .bind(auth.user_id)
        .bind(&currency)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

        if result.rows_affected() == 0 {
            return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "insufficient balance"}))));
        }
    }

    // Create ticket
    let ticket_id: (Uuid,) = sqlx::query_as(
        "INSERT INTO tickets (lottery_id, draw_id, user_id, numbers, is_free_entry)
         VALUES ($1, $2, $3, $4, $5) RETURNING id"
    )
    .bind(lottery_id)
    .bind(draw_id)
    .bind(auth.user_id)
    .bind(&body.numbers)
    .bind(is_free)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    if !is_free {
        // Split revenue
        let prize_share = ticket_price * Decimal::from(prize_pct) / Decimal::from(100);
        let project_share = ticket_price * Decimal::from(project_pct) / Decimal::from(100);

        sqlx::query("UPDATE draws SET prize_pool = prize_pool + $1 WHERE id = $2")
            .bind(prize_share).bind(draw_id).execute(&state.db).await.ok();

        sqlx::query(
            "INSERT INTO charity_ledger (project_id, source, amount, currency, description, reference_id)
             VALUES ($1, 'lottery_revenue', $2, $3, $4, $5)"
        )
        .bind(project_id).bind(project_share).bind(&currency)
        .bind(format!("Donate and Play — draw #{draw_number}"))
        .bind(ticket_id.0.to_string())
        .execute(&state.db).await.ok();

        sqlx::query("UPDATE charity_projects SET total_received = total_received + $1 WHERE id = $2")
            .bind(project_share).bind(project_id).execute(&state.db).await.ok();

        // Transaction record
        sqlx::query(
            "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, reference_type, reference_id, description)
             VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = $2 LIMIT 1), 'ticket', $3, $2, 'ticket', $4, $5)"
        )
        .bind(auth.user_id).bind(&currency).bind(ticket_price).bind(ticket_id.0)
        .bind(format!("lottery ticket draw #{draw_number}"))
        .execute(&state.db).await.ok();
    }

    Ok(Json(json!({
        "status": "purchased",
        "ticket_id": ticket_id.0,
        "draw_number": draw_number,
        "numbers": body.numbers,
        "is_free_entry": is_free,
        "price": if is_free { Decimal::ZERO } else { ticket_price },
        "message": if is_free {
            "free entry earned from consecutive plays!"
        } else {
            "Donate and Play — your contribution supports humanitarian projects"
        }
    })))
}

pub async fn list_draws(
    State(state): State<AppState>,
    Path(lottery_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows: Vec<(Uuid, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, Option<Vec<i32>>, Decimal, String)> =
        sqlx::query_as(
            "SELECT id, draw_number, scheduled_at, executed_at, winning_numbers, prize_pool, status
             FROM draws WHERE lottery_id = $1 ORDER BY draw_number DESC LIMIT 50"
        )
        .bind(lottery_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))))?;

    let draws: Vec<Value> = rows.into_iter().map(|(id, num, sched, exec, nums, pool, status)| {
        json!({ "id": id, "draw_number": num, "scheduled_at": sched, "executed_at": exec,
                 "winning_numbers": nums, "prize_pool": pool, "status": status })
    }).collect();

    Ok(Json(json!({ "lottery_id": lottery_id, "draws": draws })))
}
