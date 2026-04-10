use super::OracleEvidence;
use adenora_common::error::AdenoraError;
use adenora_common::types::MarketOutcome;

/// Determine market outcome from collected evidence.
/// Requires min_sources agreeing on the same outcome.
pub fn resolve_from_evidence(
    evidence: &[OracleEvidence],
    min_sources: u32,
) -> Result<MarketOutcome, AdenoraError> {
    if (evidence.len() as u32) < min_sources {
        return Err(AdenoraError::InsufficientOracleSources {
            need: min_sources,
            have: evidence.len() as u32,
        });
    }

    // Count votes per outcome
    let mut counts: std::collections::HashMap<String, (u32, MarketOutcome)> =
        std::collections::HashMap::new();

    for ev in evidence {
        let key = format!("{:?}", ev.interpreted_outcome);
        let entry = counts
            .entry(key)
            .or_insert((0, ev.interpreted_outcome));
        entry.0 += 1;
    }

    // Find majority outcome
    let (majority_count, majority_outcome) = counts
        .values()
        .max_by_key(|(count, _)| *count)
        .map(|(count, outcome)| (*count, *outcome))
        .unwrap_or((0, MarketOutcome::Void));

    if majority_count >= min_sources {
        Ok(majority_outcome)
    } else {
        // No consensus — cannot resolve automatically
        Err(AdenoraError::InsufficientOracleSources {
            need: min_sources,
            have: majority_count,
        })
    }
}
