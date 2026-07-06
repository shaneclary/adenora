use adenora_common::error::AdenoraError;
use super::CreateMarketRequest;

/// Validate a community market proposal before submission.
pub fn validate_proposal(req: &CreateMarketRequest) -> Result<(), AdenoraError> {
    if req.question.len() < 10 {
        return Err(AdenoraError::Internal(
            "question must be at least 10 characters".into(),
        ));
    }
    if req.question.len() > 500 {
        return Err(AdenoraError::Internal(
            "question must be at most 500 characters".into(),
        ));
    }
    if req.outcomes.len() < 2 {
        return Err(AdenoraError::Internal(
            "market must have at least 2 outcomes".into(),
        ));
    }
    if req.outcomes.len() > 20 {
        return Err(AdenoraError::Internal(
            "market must have at most 20 outcomes".into(),
        ));
    }
    if req.closes_at <= chrono::Utc::now() {
        return Err(AdenoraError::Internal(
            "close time must be in the future".into(),
        ));
    }
    if req.country_codes.is_empty() {
        return Err(AdenoraError::Internal(
            "must specify at least one country".into(),
        ));
    }
    Ok(())
}

/// Content policy — reject markets about individual death, suffering, etc.
pub fn passes_content_policy(question: &str, description: &str) -> bool {
    let banned_phrases = [
        "will die",
        "death of",
        "assassination",
        "suicide",
        "be killed",
        "murder of",
    ];
    let question_lower = question.to_lowercase();
    let description_lower = description.to_lowercase();
    !banned_phrases
        .iter()
        .any(|phrase| question_lower.contains(phrase) || description_lower.contains(phrase))
}
