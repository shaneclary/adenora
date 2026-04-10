use crate::state::AppState;
use serde_json::json;

/// Background task: snapshot leaderboards periodically.
pub async fn leaderboard_loop(state: AppState) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(300)); // every 5 min

    loop {
        ticker.tick().await;

        // Prediction leaderboard — by total P&L
        if let Ok(rows) = sqlx::query_as::<_, (uuid::Uuid, String, rust_decimal::Decimal, i64)>(
            "SELECT u.id, u.display_name, COALESCE(SUM(t.quantity * t.price_cents), 0) as volume, COUNT(t.id) as trades
             FROM users u
             JOIN trades t ON t.buyer_user_id = u.id OR t.seller_user_id = u.id
             GROUP BY u.id, u.display_name
             ORDER BY volume DESC
             LIMIT 100"
        )
        .fetch_all(&state.db)
        .await
        {
            let entries: Vec<serde_json::Value> = rows.iter().enumerate().map(|(i, (id, name, vol, trades))| {
                json!({"rank": i+1, "user_id": id, "display_name": name, "volume": vol, "trades": trades})
            }).collect();

            let total = entries.len() as i32;

            sqlx::query(
                "INSERT INTO leaderboard_snapshots (scope, category, entries, total_participants)
                 VALUES ('all_time', 'overall', $1, $2)"
            )
            .bind(json!(entries))
            .bind(total)
            .execute(&state.db)
            .await
            .ok();
        }

        // Bot leaderboard — already denormalized in bots table, just snapshot
        if let Ok(rows) = sqlx::query_as::<_, (uuid::Uuid, String, rust_decimal::Decimal, f64, i64)>(
            "SELECT id, name, total_pnl, win_rate, total_trades
             FROM bots WHERE status = 'active' AND total_trades > 0
             ORDER BY total_pnl DESC LIMIT 50"
        )
        .fetch_all(&state.db)
        .await
        {
            let entries: Vec<serde_json::Value> = rows.iter().enumerate().map(|(i, (id, name, pnl, wr, trades))| {
                json!({"rank": i+1, "bot_id": id, "name": name, "total_pnl": pnl, "win_rate": wr, "trades": trades})
            }).collect();

            let total = entries.len() as i32;

            sqlx::query(
                "INSERT INTO leaderboard_snapshots (scope, category, entries, total_participants)
                 VALUES ('all_time', 'bot_battle', $1, $2)"
            )
            .bind(json!(entries))
            .bind(total)
            .execute(&state.db)
            .await
            .ok();
        }

        // Trivia leaderboard — by total score
        if let Ok(rows) = sqlx::query_as::<_, (uuid::Uuid, String, i64, i32)>(
            "SELECT u.id, u.display_name, COALESCE(SUM(gr.score), 0) as total_score, COUNT(gr.id) as games
             FROM users u
             JOIN game_results gr ON gr.user_id = u.id
             GROUP BY u.id, u.display_name
             HAVING COUNT(gr.id) > 0
             ORDER BY total_score DESC
             LIMIT 100"
        )
        .fetch_all(&state.db)
        .await
        {
            let entries: Vec<serde_json::Value> = rows.iter().enumerate().map(|(i, (id, name, score, games))| {
                json!({"rank": i+1, "user_id": id, "display_name": name, "total_score": score, "games_played": games})
            }).collect();

            let total = entries.len() as i32;

            sqlx::query(
                "INSERT INTO leaderboard_snapshots (scope, category, entries, total_participants)
                 VALUES ('all_time', 'gaming', $1, $2)"
            )
            .bind(json!(entries))
            .bind(total)
            .execute(&state.db)
            .await
            .ok();
        }

        tracing::debug!("leaderboard snapshots updated");
    }
}
