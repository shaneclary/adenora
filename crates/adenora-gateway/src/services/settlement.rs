use crate::state::AppState;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Settle a market: pay out winning positions, refund losers nothing (binary outcome).
/// Called after oracle resolution is finalized (dispute window closed).
pub async fn settle_market(state: &AppState, market_id: Uuid) -> anyhow::Result<u32> {
    // Get market outcome
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT status, COALESCE(outcome, '') FROM markets WHERE id = $1"
    )
    .bind(market_id)
    .fetch_optional(&state.db)
    .await?;

    let (status, outcome) = row.ok_or_else(|| anyhow::anyhow!("market not found"))?;

    if status != "resolving" && status != "disputed" {
        return Err(anyhow::anyhow!("market is not in resolving/disputed state"));
    }

    if outcome.is_empty() {
        return Err(anyhow::anyhow!("market has no outcome set"));
    }

    let winning_side = match outcome.as_str() {
        "yes" => "yes",
        "no" => "no",
        "void" => {
            // Void: refund all positions at avg_price
            return void_market(state, market_id).await;
        }
        _ => return Err(anyhow::anyhow!("unsupported outcome: {outcome}")),
    };

    // Get all positions for this market
    let positions: Vec<(Uuid, Uuid, String, i32, Decimal, String)> = sqlx::query_as(
        "SELECT id, user_id, side, quantity, avg_price, mode
         FROM positions WHERE market_id = $1 AND quantity > 0"
    )
    .bind(market_id)
    .fetch_all(&state.db)
    .await?;

    let mut settled_count = 0u32;

    // All payouts, position zeroing, and the market-status flip run in one
    // transaction. If any step fails the whole thing rolls back, so the 10s
    // settlement loop can safely retry without double-paying winners it had
    // already credited before the failure.
    let mut tx = state.db.begin().await?;

    for (pos_id, user_id, side, qty, avg_price, mode) in &positions {
        let payout_per_contract = if side == winning_side {
            // Winner: receives $1.00 per contract
            Decimal::ONE
        } else {
            // Loser: receives nothing
            Decimal::ZERO
        };

        let total_payout = payout_per_contract * Decimal::from(*qty);
        let cost_basis = *avg_price * Decimal::from(*qty);

        if total_payout > Decimal::ZERO {
            // Credit winner
            sqlx::query(
                "UPDATE wallets SET available = available + $1, updated_at = NOW()
                 WHERE user_id = $2 AND currency = 'EUR'"
            )
            .bind(total_payout)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

            // Record transaction
            sqlx::query(
                "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, reference_type, reference_id, description)
                 VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = 'EUR' LIMIT 1),
                         'prize', $2, 'EUR', 'market', $3, $4)"
            )
            .bind(user_id)
            .bind(total_payout)
            .bind(market_id)
            .bind(format!("settlement payout: {} side won, {}x @ $1.00, P&L: {}", winning_side, qty, total_payout - cost_basis))
            .execute(&mut *tx)
            .await?;
        }

        // Zero out position
        sqlx::query("UPDATE positions SET quantity = 0, updated_at = NOW() WHERE id = $1")
            .bind(pos_id)
            .execute(&mut *tx)
            .await?;

        let _ = mode;
        settled_count += 1;
    }

    // Mark market as settled
    sqlx::query(
        "UPDATE markets SET status = 'settled', resolved_at = NOW(), updated_at = NOW() WHERE id = $1"
    )
    .bind(market_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Remove engine from memory
    state.engines.write().await.remove(&market_id);

    tracing::info!(market_id = %market_id, outcome = %outcome, positions = settled_count, "market settled");

    Ok(settled_count)
}

/// Void a market: refund all positions at their cost basis.
async fn void_market(state: &AppState, market_id: Uuid) -> anyhow::Result<u32> {
    let positions: Vec<(Uuid, Uuid, i32, Decimal)> = sqlx::query_as(
        "SELECT id, user_id, quantity, avg_price
         FROM positions WHERE market_id = $1 AND quantity > 0"
    )
    .bind(market_id)
    .fetch_all(&state.db)
    .await?;

    let mut count = 0u32;

    // Single transaction so a mid-loop failure rolls back and a retry cannot
    // double-refund positions already credited.
    let mut tx = state.db.begin().await?;

    for (pos_id, user_id, qty, avg_price) in &positions {
        let refund = *avg_price * Decimal::from(*qty);

        sqlx::query(
            "UPDATE wallets SET available = available + $1, updated_at = NOW()
             WHERE user_id = $2 AND currency = 'EUR'"
        )
        .bind(refund)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, reference_type, reference_id, description)
             VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = 'EUR' LIMIT 1),
                     'refund', $2, 'EUR', 'market', $3, 'market voided — full refund')"
        )
        .bind(user_id)
        .bind(refund)
        .bind(market_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("UPDATE positions SET quantity = 0, updated_at = NOW() WHERE id = $1")
            .bind(pos_id)
            .execute(&mut *tx)
            .await?;

        count += 1;
    }

    sqlx::query(
        "UPDATE markets SET status = 'settled', outcome = 'void', resolved_at = NOW(), updated_at = NOW() WHERE id = $1"
    )
    .bind(market_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    state.engines.write().await.remove(&market_id);

    tracing::info!(market_id = %market_id, positions = count, "market voided — all positions refunded");

    Ok(count)
}

/// Background task: check for markets that need auto-closing or settlement.
pub async fn settlement_loop(state: AppState) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(10));

    loop {
        ticker.tick().await;

        // Auto-close markets past their close time
        let to_close: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM markets WHERE status = 'active' AND closes_at <= NOW()"
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for (market_id,) in to_close {
            sqlx::query("UPDATE markets SET status = 'closed', updated_at = NOW() WHERE id = $1")
                .bind(market_id)
                .execute(&state.db)
                .await
                .ok();
            tracing::info!(market_id = %market_id, "market auto-closed");
        }

        // Check for resolved markets past dispute window ready for settlement
        let dispute_hours = state.config.trading.dispute_window_hours as i64;
        let to_settle: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM markets
             WHERE status = 'resolving' AND outcome IS NOT NULL
             AND updated_at + make_interval(hours => $1) <= NOW()
             AND id NOT IN (SELECT market_id FROM disputes WHERE status IN ('filed', 'voting'))"
        )
        .bind(dispute_hours)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for (market_id,) in to_settle {
            if let Err(e) = settle_market(&state, market_id).await {
                tracing::error!(market_id = %market_id, error = %e, "settlement failed");
            }
        }
    }
}
