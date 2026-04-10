use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Real-time event for spectator WebSocket feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpectatorEvent {
    /// A trade was executed in bot mode
    BotTrade {
        market_id: MarketId,
        buyer_bot: Option<BotId>,
        seller_bot: Option<BotId>,
        side: Side,
        price_cents: u32,
        quantity: u32,
        timestamp: DateTime<Utc>,
    },
    /// Order book update in bot mode
    BookUpdate {
        market_id: MarketId,
        best_bid: Option<u32>,
        best_ask: Option<u32>,
        spread: Option<i32>,
        timestamp: DateTime<Utc>,
    },
    /// Bot leaderboard position changed
    RankChange {
        bot_id: BotId,
        old_rank: u32,
        new_rank: u32,
        pnl: Decimal,
    },
    /// Cage match update
    CageMatchUpdate {
        match_id: uuid::Uuid,
        bot_a_pnl: Decimal,
        bot_b_pnl: Decimal,
        trades_count: u64,
    },
}
