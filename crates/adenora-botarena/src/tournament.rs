use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A bot trading tournament — scheduled competition on specific markets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTournament {
    pub id: TournamentId,
    pub name: String,
    pub description: String,
    pub market_ids: Vec<MarketId>,
    pub entry_fee: Decimal,
    pub prize_pool: Decimal,
    pub max_bots: u32,
    pub registered_bots: Vec<BotId>,
    pub status: TournamentStatus,
    pub ranking_metric: RankingMetric,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub results: Vec<BotTournamentResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankingMetric {
    /// Total P&L across tournament markets
    TotalPnl,
    /// Risk-adjusted returns (Sharpe)
    SharpeRatio,
    /// Win rate percentage
    WinRate,
    /// Combined score (P&L * win_rate)
    Combined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTournamentResult {
    pub bot_id: BotId,
    pub rank: u32,
    pub total_pnl: Decimal,
    pub trades: u32,
    pub win_rate: f64,
    pub prize: Decimal,
}
