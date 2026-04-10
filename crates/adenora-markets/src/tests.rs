use crate::lifecycle;
use crate::proposal;
use adenora_common::types::*;

#[test]
fn test_valid_transitions() {
    // Happy path
    assert!(lifecycle::validate_transition(MarketStatus::Proposed, MarketStatus::Active).is_ok());
    assert!(lifecycle::validate_transition(MarketStatus::Active, MarketStatus::Closed).is_ok());
    assert!(lifecycle::validate_transition(MarketStatus::Closed, MarketStatus::Resolving).is_ok());
    assert!(lifecycle::validate_transition(MarketStatus::Resolving, MarketStatus::Settled).is_ok());

    // Dispute path
    assert!(lifecycle::validate_transition(MarketStatus::Resolving, MarketStatus::Disputed).is_ok());
    assert!(lifecycle::validate_transition(MarketStatus::Disputed, MarketStatus::Resolving).is_ok());
    assert!(lifecycle::validate_transition(MarketStatus::Disputed, MarketStatus::Settled).is_ok());

    // Reject proposal
    assert!(lifecycle::validate_transition(MarketStatus::Proposed, MarketStatus::Settled).is_ok());
}

#[test]
fn test_invalid_transitions() {
    assert!(lifecycle::validate_transition(MarketStatus::Active, MarketStatus::Proposed).is_err());
    assert!(lifecycle::validate_transition(MarketStatus::Settled, MarketStatus::Active).is_err());
    assert!(lifecycle::validate_transition(MarketStatus::Closed, MarketStatus::Active).is_err());
    assert!(lifecycle::validate_transition(MarketStatus::Proposed, MarketStatus::Resolving).is_err());
}

#[test]
fn test_proposal_validation() {
    use crate::CreateMarketRequest;

    let valid = CreateMarketRequest {
        question: "Will the price of Bitcoin exceed $100K by end of 2026?".into(),
        description: "Based on CoinMarketCap closing price.".into(),
        category: MarketCategory::Economy,
        outcomes: vec!["Yes".into(), "No".into()],
        resolution_source: Some("coinmarketcap.com".into()),
        resolution_criteria: Some("CMC BTC/USD price at 23:59 UTC Dec 31 2026".into()),
        country_codes: vec!["XK".into(), "AL".into()],
        closes_at: chrono::Utc::now() + chrono::Duration::days(30),
    };

    assert!(proposal::validate_proposal(&valid).is_ok());
}

#[test]
fn test_proposal_too_short_question() {
    use crate::CreateMarketRequest;

    let invalid = CreateMarketRequest {
        question: "Short?".into(),
        description: "".into(),
        category: MarketCategory::Custom,
        outcomes: vec!["Yes".into(), "No".into()],
        resolution_source: None,
        resolution_criteria: None,
        country_codes: vec!["XK".into()],
        closes_at: chrono::Utc::now() + chrono::Duration::days(1),
    };

    assert!(proposal::validate_proposal(&invalid).is_err());
}

#[test]
fn test_proposal_needs_two_outcomes() {
    use crate::CreateMarketRequest;

    let invalid = CreateMarketRequest {
        question: "This question only has one outcome".into(),
        description: "".into(),
        category: MarketCategory::Custom,
        outcomes: vec!["Only one".into()],
        resolution_source: None,
        resolution_criteria: None,
        country_codes: vec!["XK".into()],
        closes_at: chrono::Utc::now() + chrono::Duration::days(1),
    };

    assert!(proposal::validate_proposal(&invalid).is_err());
}

#[test]
fn test_content_policy_blocks_death_markets() {
    assert!(!proposal::passes_content_policy("Will person X will die before 2027?", ""));
    assert!(!proposal::passes_content_policy("The assassination of leader Y", ""));
    assert!(!proposal::passes_content_policy("Will someone be killed in the conflict?", ""));
    assert!(!proposal::passes_content_policy("Betting on the death of a public figure", ""));
}

#[test]
fn test_content_policy_allows_normal_markets() {
    assert!(proposal::passes_content_policy("Will Bitcoin exceed $100K?", ""));
    assert!(proposal::passes_content_policy("Will Albania join the EU by 2030?", ""));
    assert!(proposal::passes_content_policy("Will Kosovo win any medal at the Olympics?", ""));
}
