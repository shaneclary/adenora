use adenora_common::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Create a ticket for a lottery draw.
pub fn create_ticket(
    lottery_id: LotteryId,
    draw_id: DrawId,
    user_id: UserId,
    numbers: Vec<u32>,
    is_free_entry: bool,
) -> super::Ticket {
    super::Ticket {
        id: Uuid::new_v4(),
        lottery_id,
        draw_id,
        user_id,
        numbers,
        is_free_entry,
        purchased_at: Utc::now(),
    }
}

/// Check if a user qualifies for a free entry (after N consecutive non-winning tickets).
pub fn qualifies_for_free_entry(
    consecutive_losses: u32,
    free_entry_threshold: u32,
) -> bool {
    free_entry_threshold > 0 && consecutive_losses >= free_entry_threshold
}
