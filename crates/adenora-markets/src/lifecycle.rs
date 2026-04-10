use adenora_common::error::AdenoraError;
use adenora_common::types::*;
use super::Market;

/// Validate a market status transition.
pub fn validate_transition(
    current: MarketStatus,
    next: MarketStatus,
) -> Result<(), AdenoraError> {
    let valid = match (current, next) {
        (MarketStatus::Proposed, MarketStatus::Active) => true,
        (MarketStatus::Proposed, MarketStatus::Settled) => true, // rejected -> void
        (MarketStatus::Active, MarketStatus::Closed) => true,
        (MarketStatus::Closed, MarketStatus::Resolving) => true,
        (MarketStatus::Resolving, MarketStatus::Settled) => true,
        (MarketStatus::Resolving, MarketStatus::Disputed) => true,
        (MarketStatus::Disputed, MarketStatus::Resolving) => true,  // dispute resolved
        (MarketStatus::Disputed, MarketStatus::Settled) => true,    // dispute settled
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(AdenoraError::Internal(format!(
            "invalid market transition: {:?} -> {:?}",
            current, next
        )))
    }
}

/// Check if a market should auto-close based on time.
pub fn should_close(market: &Market) -> bool {
    market.status == MarketStatus::Active && chrono::Utc::now() >= market.closes_at
}
