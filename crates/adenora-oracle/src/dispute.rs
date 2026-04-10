use super::*;
use adenora_common::error::AdenoraError;
use adenora_common::types::*;
use chrono::Utc;
use uuid::Uuid;

const JURY_SIZE: usize = 11;

/// File a dispute against a market resolution. Free — no bond required.
pub fn file_dispute(
    market_id: MarketId,
    challenger_id: UserId,
    reason: String,
    evidence_links: Vec<String>,
) -> Dispute {
    Dispute {
        id: Uuid::new_v4(),
        market_id,
        challenger_id,
        reason,
        evidence_links,
        status: DisputeStatus::Filed,
        jury_votes: Vec::new(),
        resolution: None,
        created_at: Utc::now(),
        resolved_at: None,
    }
}

/// Select random jurors from verified users (excluding challenger and market participants).
pub fn select_jury(
    verified_user_ids: &[UserId],
    exclude: &[UserId],
) -> Vec<UserId> {
    use rand::seq::SliceRandom;

    let eligible: Vec<UserId> = verified_user_ids
        .iter()
        .filter(|id| !exclude.contains(id))
        .copied()
        .collect();

    let mut rng = rand::thread_rng();
    let count = JURY_SIZE.min(eligible.len());
    eligible.choose_multiple(&mut rng, count).copied().collect()
}

/// Record a jury vote and check if we have a verdict.
pub fn record_vote(dispute: &mut Dispute, vote: JuryVote) -> Option<DisputeResolution> {
    dispute.jury_votes.push(vote);

    // Need majority (>50%) to resolve
    let majority_needed = (JURY_SIZE / 2) + 1;

    if dispute.jury_votes.len() < majority_needed {
        return None;
    }

    // Count votes per outcome
    let mut counts: std::collections::HashMap<String, (usize, MarketOutcome)> =
        std::collections::HashMap::new();

    for v in &dispute.jury_votes {
        let key = format!("{:?}", v.vote);
        let entry = counts.entry(key).or_insert((0, v.vote));
        entry.0 += 1;
    }

    let (max_count, max_outcome) = counts
        .values()
        .max_by_key(|(c, _)| *c)
        .map(|(c, o)| (*c, *o))
        .unwrap_or((0, MarketOutcome::Void));

    if max_count >= majority_needed {
        let resolution = DisputeResolution {
            outcome: max_outcome,
            method: ResolutionMethod::JuryVerdict,
            notes: format!("jury voted {}/{}", max_count, dispute.jury_votes.len()),
        };
        dispute.status = DisputeStatus::Resolved;
        dispute.resolution = Some(resolution.clone());
        dispute.resolved_at = Some(Utc::now());
        Some(resolution)
    } else if dispute.jury_votes.len() >= JURY_SIZE {
        // All votes in but no majority — escalate
        dispute.status = DisputeStatus::Escalated;
        None
    } else {
        None
    }
}
