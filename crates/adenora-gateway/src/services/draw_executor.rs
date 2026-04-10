use crate::state::AppState;
use adenora_lottery::draw;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Execute a lottery draw: generate winning numbers, find winners, distribute prizes.
pub async fn execute_draw(state: &AppState, draw_id: Uuid) -> anyhow::Result<u32> {
    // Get draw details
    let row: Option<(Uuid, Uuid, Decimal, String)> = sqlx::query_as(
        "SELECT d.id, d.lottery_id, d.prize_pool, l.game_type
         FROM draws d JOIN lotteries l ON l.id = d.lottery_id
         WHERE d.id = $1 AND d.status IN ('scheduled', 'open', 'closed')"
    )
    .bind(draw_id)
    .fetch_optional(&state.db)
    .await?;

    let (_draw_id, lottery_id, prize_pool, game_type) = row
        .ok_or_else(|| anyhow::anyhow!("draw not found or already executed"))?;

    // Generate winning numbers (6 numbers from 1-49 for standard lotto)
    let (num_count, num_max) = match game_type.as_str() {
        "draw" => (6, 49),
        "numbers" => (4, 9),
        "fifty_fifty" => (1, 2), // simple 50/50
        _ => (6, 49),
    };

    let winning_numbers = draw::generate_draw_numbers(num_count, 1, num_max);
    let winning_i32: Vec<i32> = winning_numbers.iter().map(|n| *n as i32).collect();

    // Atomically claim this draw — CAS guard prevents double execution
    let claimed = sqlx::query(
        "UPDATE draws SET winning_numbers = $1, executed_at = NOW(), status = 'drawn'
         WHERE id = $2 AND status IN ('scheduled', 'open', 'closed')
         AND winning_numbers IS NULL"
    )
    .bind(&winning_i32)
    .bind(draw_id)
    .execute(&state.db)
    .await?;

    if claimed.rows_affected() == 0 {
        tracing::warn!(draw_id = %draw_id, "draw already executed or claimed by another process");
        return Ok(0);
    }

    // Get all tickets for this draw
    let tickets: Vec<(Uuid, Uuid, Vec<i32>)> = sqlx::query_as(
        "SELECT id, user_id, numbers FROM tickets WHERE draw_id = $1"
    )
    .bind(draw_id)
    .fetch_all(&state.db)
    .await?;

    // Determine winners
    let mut winner_count = 0u32;
    let mut jackpot_winners = Vec::new();
    let mut tier_winners: Vec<(Uuid, Uuid, String, usize)> = Vec::new(); // ticket_id, user_id, tier, matches

    for (ticket_id, user_id, ticket_numbers) in &tickets {
        let ticket_u32: Vec<u32> = ticket_numbers.iter().map(|n| *n as u32).collect();
        let matches = draw::count_matches(&ticket_u32, &winning_numbers);

        if let Some(tier) = draw::prize_tier(matches, num_count) {
            let tier_str = format!("{:?}", tier).to_lowercase();
            tier_winners.push((*ticket_id, *user_id, tier_str.clone(), matches));

            if matches == num_count {
                jackpot_winners.push((*ticket_id, *user_id));
            }
        }
    }

    // Calculate prize distribution
    // Jackpot: 50% of pool, Second: 20%, Third: 15%, Fourth: 10%, Fifth: 5%
    let tier_pcts = [
        ("jackpot", Decimal::new(50, 0)),
        ("second", Decimal::new(20, 0)),
        ("third", Decimal::new(15, 0)),
        ("fourth", Decimal::new(10, 0)),
        ("fifth", Decimal::new(5, 0)),
    ];

    for (tier_name, pct) in &tier_pcts {
        let tier_pool = prize_pool * *pct / Decimal::from(100);
        let tier_count = tier_winners.iter().filter(|(_, _, t, _)| t == tier_name).count();

        if tier_count == 0 || tier_pool <= Decimal::ZERO {
            continue;
        }

        let per_winner = tier_pool / Decimal::from(tier_count as u32);

        for (ticket_id, user_id, tier, _) in &tier_winners {
            if tier != tier_name {
                continue;
            }

            // Record winner
            sqlx::query(
                "INSERT INTO winners (draw_id, user_id, ticket_id, tier, amount, paid_at)
                 VALUES ($1, $2, $3, $4, $5, NOW())"
            )
            .bind(draw_id)
            .bind(user_id)
            .bind(ticket_id)
            .bind(tier)
            .bind(per_winner)
            .execute(&state.db)
            .await?;

            // Credit wallet
            let lottery_currency: (String,) = sqlx::query_as(
                "SELECT currency FROM lotteries WHERE id = $1"
            )
            .bind(lottery_id)
            .fetch_one(&state.db)
            .await?;

            sqlx::query(
                "UPDATE wallets SET available = available + $1, updated_at = NOW()
                 WHERE user_id = $2 AND currency = $3"
            )
            .bind(per_winner)
            .bind(user_id)
            .bind(&lottery_currency.0)
            .execute(&state.db)
            .await?;

            // Record transaction
            sqlx::query(
                "INSERT INTO transactions (user_id, wallet_id, tx_type, amount, currency, reference_type, reference_id, description)
                 VALUES ($1, (SELECT id FROM wallets WHERE user_id = $1 AND currency = $2 LIMIT 1),
                         'prize', $3, $2, 'draw', $4, $5)"
            )
            .bind(user_id)
            .bind(&lottery_currency.0)
            .bind(per_winner)
            .bind(draw_id)
            .bind(format!("{} prize: {} {}", tier, per_winner, lottery_currency.0))
            .execute(&state.db)
            .await?;

            winner_count += 1;
        }
    }

    // Handle free entry winners (1 match)
    for (ticket_id, user_id, tier, _) in &tier_winners {
        if tier == "free" {
            sqlx::query(
                "INSERT INTO winners (draw_id, user_id, ticket_id, tier, amount)
                 VALUES ($1, $2, $3, 'free', 0)"
            )
            .bind(draw_id)
            .bind(user_id)
            .bind(ticket_id)
            .execute(&state.db)
            .await?;
        }
    }

    // Mark draw as paid
    sqlx::query("UPDATE draws SET status = 'paid' WHERE id = $1")
        .bind(draw_id)
        .execute(&state.db)
        .await?;

    tracing::info!(
        draw_id = %draw_id,
        tickets = tickets.len(),
        winners = winner_count,
        prize_pool = %prize_pool,
        winning_numbers = ?winning_numbers,
        "draw executed"
    );

    Ok(winner_count)
}

/// Background task: check for draws that need execution.
pub async fn draw_loop(state: AppState) {
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(30));

    loop {
        ticker.tick().await;

        // Find draws past their scheduled time that haven't been executed
        let due_draws: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM draws
             WHERE status IN ('scheduled', 'open', 'closed')
             AND scheduled_at <= NOW()
             ORDER BY scheduled_at ASC"
        )
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for (draw_id,) in due_draws {
            // Close ticket sales first
            sqlx::query("UPDATE draws SET status = 'closed' WHERE id = $1 AND status IN ('scheduled', 'open')")
                .bind(draw_id)
                .execute(&state.db)
                .await
                .ok();

            if let Err(e) = execute_draw(&state, draw_id).await {
                tracing::error!(draw_id = %draw_id, error = %e, "draw execution failed");
            }
        }
    }
}
