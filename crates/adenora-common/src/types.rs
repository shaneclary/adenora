use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;
pub type MarketId = Uuid;
pub type OrderId = Uuid;
pub type TradeId = Uuid;
pub type BatchId = Uuid;
pub type BotId = Uuid;
pub type ProjectId = Uuid;
pub type TournamentId = Uuid;
pub type GameId = Uuid;
pub type LotteryId = Uuid;
pub type DrawId = Uuid;
pub type DisputeId = Uuid;
pub type EvidenceId = Uuid;

/// Price in cents (0-100 for prediction markets).
/// Stored as Decimal for precision in fee calculations.
pub type PriceCents = Decimal;

/// Monetary amount in the smallest currency unit (e.g. cents for EUR).
pub type Amount = Decimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Yes,
    No,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Yes => Side::No,
            Side::No => Side::Yes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    /// Immediate or cancel — fill what you can, cancel the rest
    Ioc,
    /// Good till cancelled — resting limit order
    Gtc,
    /// Fill or kill — all or nothing
    Fok,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketMode {
    /// Batch auction matching (500ms windows) — humans only, fair pricing, rate-limited
    People,
    /// Continuous matching, no rate limits — open to humans and bots alike
    /// Humans enter willingly knowing speed matters; bots permitted
    Unlimited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    /// Community-proposed, awaiting approval
    Proposed,
    /// Approved and accepting orders
    Active,
    /// Trading halted, awaiting resolution
    Closed,
    /// Oracle collecting evidence
    Resolving,
    /// Dispute period active
    Disputed,
    /// Final outcome determined, positions settled
    Settled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketOutcome {
    Yes,
    No,
    /// Multi-outcome markets: specific outcome index
    Index(u32),
    /// Market voided — all positions refunded
    Void,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// In the batch queue (people mode) or resting on the book (bot mode)
    Pending,
    /// Partially filled
    PartialFill,
    /// Fully filled
    Filled,
    /// Cancelled by user
    Cancelled,
    /// Expired (market closed)
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketCategory {
    Geopolitics,
    Economy,
    Sports,
    EuPolicy,
    Tech,
    Weather,
    Entertainment,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LotteryGameType {
    /// Classic number draw (lotto)
    Draw,
    /// Instant scratch card
    Scratch,
    /// 50/50 raffle
    FiftyFifty,
    /// Prize raffle (vacations, cars, etc.)
    PrizeRaffle,
    /// Numbers game
    Numbers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TournamentStatus {
    Registration,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamped<T> {
    pub data: T,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
