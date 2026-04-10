use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub game_id: GameId,
    pub entry_fee: Decimal,
    pub prize_pool: Decimal,
    pub house_cut_pct: u32, // typically 1% to match prediction fee structure
    pub max_participants: u32,
    pub current_participants: u32,
    pub status: TournamentStatus,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentEntry {
    pub id: Uuid,
    pub tournament_id: TournamentId,
    pub user_id: UserId,
    pub score: Option<i64>,
    pub rank: Option<u32>,
    pub prize_amount: Option<Decimal>,
    pub entered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentPrizeStructure {
    /// Percentage of prize pool for each rank (1st, 2nd, 3rd, etc.)
    pub distribution: Vec<PrizeSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrizeSlot {
    pub rank_from: u32,
    pub rank_to: u32,
    pub pct: Decimal,
}

impl TournamentPrizeStructure {
    /// Standard prize distribution for tournaments.
    pub fn standard() -> Self {
        Self {
            distribution: vec![
                PrizeSlot {
                    rank_from: 1,
                    rank_to: 1,
                    pct: Decimal::new(50, 0),
                },
                PrizeSlot {
                    rank_from: 2,
                    rank_to: 2,
                    pct: Decimal::new(25, 0),
                },
                PrizeSlot {
                    rank_from: 3,
                    rank_to: 3,
                    pct: Decimal::new(15, 0),
                },
                PrizeSlot {
                    rank_from: 4,
                    rank_to: 10,
                    pct: Decimal::new(10, 0),
                },
            ],
        }
    }
}
