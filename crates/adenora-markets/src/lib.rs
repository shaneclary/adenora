pub mod lifecycle;
pub mod proposal;

#[cfg(test)]
mod tests;

use adenora_common::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: MarketId,
    pub question: String,
    pub description: String,
    pub category: MarketCategory,
    pub outcomes: Vec<String>,
    pub status: MarketStatus,
    pub creator_id: UserId,
    pub resolution_source: Option<String>,
    pub resolution_criteria: Option<String>,
    pub outcome: Option<MarketOutcome>,
    /// Countries where this market is available (geofencing)
    pub country_codes: Vec<String>,
    pub opens_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMarketRequest {
    pub question: String,
    pub description: String,
    pub category: MarketCategory,
    pub outcomes: Vec<String>,
    pub resolution_source: Option<String>,
    pub resolution_criteria: Option<String>,
    pub country_codes: Vec<String>,
    pub closes_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketProposal {
    pub id: Uuid,
    pub proposer_id: UserId,
    pub request: CreateMarketRequest,
    pub status: ProposalStatus,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
}
