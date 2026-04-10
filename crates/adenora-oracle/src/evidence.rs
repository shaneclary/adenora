use super::OracleEvidence;
use adenora_common::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Create an evidence record from a data source fetch.
pub fn create_evidence(
    market_id: MarketId,
    source_name: &str,
    source_url: &str,
    raw_data: serde_json::Value,
    interpreted_outcome: MarketOutcome,
    confidence: f64,
) -> OracleEvidence {
    OracleEvidence {
        id: Uuid::new_v4(),
        market_id,
        source_name: source_name.to_string(),
        source_url: source_url.to_string(),
        raw_data,
        interpreted_outcome,
        confidence: confidence.clamp(0.0, 1.0),
        collected_at: Utc::now(),
    }
}
