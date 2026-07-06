use crate::state::AppState;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Background task: detect and recover from error states.
pub async fn recovery_loop(state: AppState) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        ticker.tick().await;

        // 1. Orphan orders — pending orders on settled/closed markets
        cleanup_orphan_orders(&state).await;

        // 2. Stuck reserved funds — reserved > 0 with no pending orders
        release_stuck_reserves(&state).await;

        // 3. Stale engine cleanup — remove engines for settled markets
        cleanup_stale_engines(&state).await;
    }
}

async fn cleanup_orphan_orders(state: &AppState) {
    let orphans: Vec<(Uuid, Uuid, i32, i32, Uuid, String)> = sqlx::query_as(
        "SELECT o.id, o.market_id, o.price_cents, o.quantity - o.filled_quantity as unfilled, o.user_id, o.action
         FROM orders o
         JOIN markets m ON m.id = o.market_id
         WHERE o.status IN ('pending', 'partial_fill')
         AND m.status NOT IN ('active', 'proposed')"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (order_id, market_id, price_cents, unfilled, user_id, action) in &orphans {
        // Cancel the order
        sqlx::query("UPDATE orders SET status = 'expired', updated_at = NOW() WHERE id = $1")
            .bind(order_id)
            .execute(&state.db)
            .await
            .ok();

        // Release reserved funds — only buys reserve funds; releasing on a sell
        // would credit money that was never reserved.
        if action == "buy" && *unfilled > 0 {
            let release = Decimal::new(*price_cents as i64 * *unfilled as i64, 2);
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

        tracing::warn!(order_id = %order_id, market_id = %market_id, "orphan order expired");
    }
}

async fn release_stuck_reserves(state: &AppState) {
    // Find users with reserved > 0 but no pending/partial_fill orders
    let stuck: Vec<(Uuid, Decimal)> = sqlx::query_as(
        "SELECT w.user_id, w.reserved
         FROM wallets w
         WHERE w.reserved > 0
         AND NOT EXISTS (
             SELECT 1 FROM orders o
             WHERE o.user_id = w.user_id
             AND o.status IN ('pending', 'partial_fill')
         )"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    for (user_id, reserved) in &stuck {
        sqlx::query(
            "UPDATE wallets SET available = available + reserved, reserved = 0, updated_at = NOW()
             WHERE user_id = $1 AND reserved > 0
             AND NOT EXISTS (
                 SELECT 1 FROM orders o
                 WHERE o.user_id = $1
                 AND o.status IN ('pending', 'partial_fill')
             )"
        )
        .bind(user_id)
        .execute(&state.db)
        .await
        .ok();

        tracing::warn!(user_id = %user_id, amount = %reserved, "released stuck reserved funds");
    }
}

async fn cleanup_stale_engines(state: &AppState) {
    let settled_ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM markets WHERE status IN ('settled', 'closed')
         AND updated_at < NOW() - INTERVAL '1 hour'"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let mut engines = state.engines.write().await;
    let before = engines.len();
    for (market_id,) in &settled_ids {
        engines.remove(market_id);
    }
    let removed = before - engines.len();
    if removed > 0 {
        tracing::debug!(removed = removed, "cleaned up stale market engines");
    }
}
