pub mod registry;
pub mod spectator;
pub mod tournament;

use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A registered trading bot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bot {
    pub id: BotId,
    pub owner_id: UserId,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub is_open_source: bool,
    pub source_url: Option<String>,
    pub stats: BotStats,
    pub status: BotStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotStatus {
    Active,
    Suspended,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStats {
    pub total_trades: u64,
    pub total_volume: Decimal,
    pub total_pnl: Decimal,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub markets_traded: u32,
    pub tournaments_entered: u32,
    pub tournaments_won: u32,
    pub updated_at: DateTime<Utc>,
}

impl Default for BotStats {
    fn default() -> Self {
        Self {
            total_trades: 0,
            total_volume: Decimal::ZERO,
            total_pnl: Decimal::ZERO,
            win_rate: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            markets_traded: 0,
            tournaments_entered: 0,
            tournaments_won: 0,
            updated_at: Utc::now(),
        }
    }
}

/// A bot vs bot cage match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CageMatch {
    pub id: Uuid,
    pub market_id: MarketId,
    pub bot_a: BotId,
    pub bot_b: BotId,
    pub bot_a_pnl: Decimal,
    pub bot_b_pnl: Decimal,
    pub winner: Option<BotId>,
    pub status: CageMatchStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CageMatchStatus {
    Scheduled,
    Live,
    Completed,
}
