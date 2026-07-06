pub mod dispute;
pub mod evidence;
pub mod resolution;

#[cfg(test)]
mod tests;

use adenora_common::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A piece of evidence collected from a data source for market resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleEvidence {
    pub id: EvidenceId,
    pub market_id: MarketId,
    pub source_name: String,
    pub source_url: String,
    pub raw_data: serde_json::Value,
    pub interpreted_outcome: MarketOutcome,
    pub confidence: f64,
    pub collected_at: DateTime<Utc>,
}

/// A dispute filed against a market resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispute {
    pub id: DisputeId,
    pub market_id: MarketId,
    pub challenger_id: UserId,
    pub reason: String,
    pub evidence_links: Vec<String>,
    pub status: DisputeStatus,
    /// The panel of jurors eligible to vote on this dispute. Only these users'
    /// votes are accepted, and each may vote at most once.
    pub selected_jury: Vec<UserId>,
    pub jury_votes: Vec<JuryVote>,
    pub resolution: Option<DisputeResolution>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    /// Filed, awaiting jury selection
    Filed,
    /// Jury selected, voting in progress
    Voting,
    /// Jury reached a verdict
    Resolved,
    /// Escalated to independent panel
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuryVote {
    pub juror_id: UserId,
    pub vote: MarketOutcome,
    pub reasoning: Option<String>,
    pub voted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolution {
    pub outcome: MarketOutcome,
    pub method: ResolutionMethod,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionMethod {
    /// Multi-source oracle agreement
    OracleConsensus,
    /// Community jury majority vote
    JuryVerdict,
    /// Independent panel decision
    PanelDecision,
    /// Market voided
    Void,
}
