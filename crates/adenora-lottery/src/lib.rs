pub mod draw;
pub mod scratch;
pub mod ticket;

use adenora_common::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A lottery game — the "Donate and Play" core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lottery {
    pub id: LotteryId,
    pub name: String,
    pub description: String,
    pub game_type: LotteryGameType,
    pub project_id: ProjectId,
    /// Country codes where this lottery is available
    pub country_codes: Vec<String>,
    pub ticket_price: Decimal,
    pub currency: adenora_common::currency::Currency,
    /// Percentage of ticket revenue paid out as prizes (typically ~50%)
    pub prize_pct: u32,
    /// Percentage to the funded project/charity
    pub project_pct: u32,
    /// Percentage to the company
    pub company_pct: u32,
    pub status: LotteryStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotteryStatus {
    Draft,
    Active,
    Suspended,
    Completed,
}

/// A scheduled draw event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Draw {
    pub id: DrawId,
    pub lottery_id: LotteryId,
    pub draw_number: u32,
    pub scheduled_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub winning_numbers: Option<Vec<u32>>,
    pub prize_pool: Decimal,
    pub winners: Vec<Winner>,
    pub status: DrawStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawStatus {
    Scheduled,
    Open,    // accepting tickets
    Closed,  // no more tickets, awaiting draw
    Drawn,   // numbers drawn, winners determined
    Paid,    // prizes distributed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Winner {
    pub user_id: UserId,
    pub ticket_id: Uuid,
    pub tier: PrizeTier,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrizeTier {
    Jackpot,
    Second,
    Third,
    Fourth,
    Fifth,
    Free,  // free entry for next draw
}

/// A ticket purchased by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub lottery_id: LotteryId,
    pub draw_id: DrawId,
    pub user_id: UserId,
    pub numbers: Vec<u32>,
    pub is_free_entry: bool,
    pub purchased_at: DateTime<Utc>,
}

/// Multi-country linked jackpot configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedJackpot {
    pub id: Uuid,
    pub name: String,
    pub linked_lottery_ids: Vec<LotteryId>,
    pub combined_pool: Decimal,
    pub country_codes: Vec<String>,
}
