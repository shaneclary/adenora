use crate::evidence;
use crate::resolution;
use crate::dispute;
use adenora_common::types::*;
use uuid::Uuid;

#[test]
fn test_resolve_unanimous_yes() {
    let market_id = Uuid::new_v4();
    let evidence = vec![
        evidence::create_evidence(market_id, "source_a", "https://a.com", serde_json::json!({}), MarketOutcome::Yes, 0.95),
        evidence::create_evidence(market_id, "source_b", "https://b.com", serde_json::json!({}), MarketOutcome::Yes, 0.90),
        evidence::create_evidence(market_id, "source_c", "https://c.com", serde_json::json!({}), MarketOutcome::Yes, 0.92),
    ];

    let result = resolution::resolve_from_evidence(&evidence, 3).unwrap();
    assert!(matches!(result, MarketOutcome::Yes));
}

#[test]
fn test_resolve_unanimous_no() {
    let market_id = Uuid::new_v4();
    let evidence = vec![
        evidence::create_evidence(market_id, "a", "https://a.com", serde_json::json!({}), MarketOutcome::No, 0.95),
        evidence::create_evidence(market_id, "b", "https://b.com", serde_json::json!({}), MarketOutcome::No, 0.90),
        evidence::create_evidence(market_id, "c", "https://c.com", serde_json::json!({}), MarketOutcome::No, 0.92),
    ];

    let result = resolution::resolve_from_evidence(&evidence, 3).unwrap();
    assert!(matches!(result, MarketOutcome::No));
}

#[test]
fn test_resolve_insufficient_sources() {
    let market_id = Uuid::new_v4();
    let evidence = vec![
        evidence::create_evidence(market_id, "a", "https://a.com", serde_json::json!({}), MarketOutcome::Yes, 0.95),
        evidence::create_evidence(market_id, "b", "https://b.com", serde_json::json!({}), MarketOutcome::Yes, 0.90),
    ];

    let result = resolution::resolve_from_evidence(&evidence, 3);
    assert!(result.is_err()); // Need 3, have 2
}

#[test]
fn test_resolve_no_consensus() {
    let market_id = Uuid::new_v4();
    let evidence = vec![
        evidence::create_evidence(market_id, "a", "https://a.com", serde_json::json!({}), MarketOutcome::Yes, 0.95),
        evidence::create_evidence(market_id, "b", "https://b.com", serde_json::json!({}), MarketOutcome::No, 0.90),
        evidence::create_evidence(market_id, "c", "https://c.com", serde_json::json!({}), MarketOutcome::Void, 0.80),
    ];

    // No 3 sources agree on the same outcome
    let result = resolution::resolve_from_evidence(&evidence, 3);
    assert!(result.is_err());
}

#[test]
fn test_resolve_majority_with_dissent() {
    let market_id = Uuid::new_v4();
    let evidence = vec![
        evidence::create_evidence(market_id, "a", "https://a.com", serde_json::json!({}), MarketOutcome::Yes, 0.95),
        evidence::create_evidence(market_id, "b", "https://b.com", serde_json::json!({}), MarketOutcome::Yes, 0.90),
        evidence::create_evidence(market_id, "c", "https://c.com", serde_json::json!({}), MarketOutcome::Yes, 0.85),
        evidence::create_evidence(market_id, "d", "https://d.com", serde_json::json!({}), MarketOutcome::No, 0.70),
    ];

    // 3 out of 4 say Yes — meets min_sources=3
    let result = resolution::resolve_from_evidence(&evidence, 3).unwrap();
    assert!(matches!(result, MarketOutcome::Yes));
}

#[test]
fn test_file_dispute() {
    let market_id = Uuid::new_v4();
    let challenger_id = Uuid::new_v4();

    let dispute = dispute::file_dispute(
        market_id,
        challenger_id,
        "Resolution source was incorrect".into(),
        vec!["https://proof.com/evidence".into()],
    );

    assert_eq!(dispute.market_id, market_id);
    assert_eq!(dispute.challenger_id, challenger_id);
    assert!(matches!(dispute.status, crate::DisputeStatus::Filed));
    assert!(dispute.resolution.is_none());
}

#[test]
fn test_jury_vote_majority() {
    let market_id = Uuid::new_v4();
    let mut d = dispute::file_dispute(market_id, Uuid::new_v4(), "test".into(), vec![]);

    // Vote 6 times for Yes — should reach majority (6 out of 11)
    for _ in 0..6 {
        let vote = crate::JuryVote {
            juror_id: Uuid::new_v4(),
            vote: MarketOutcome::Yes,
            reasoning: None,
            voted_at: chrono::Utc::now(),
        };
        let result = dispute::record_vote(&mut d, vote);

        if d.jury_votes.len() >= 6 {
            assert!(result.is_some());
            let resolution = result.unwrap();
            assert!(matches!(resolution.outcome, MarketOutcome::Yes));
            return;
        }
    }

    panic!("should have resolved after 6 votes");
}

#[test]
fn test_jury_selection_excludes() {
    let all_users: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();
    let exclude = vec![all_users[0], all_users[1]];

    let jury = dispute::select_jury(&all_users, &exclude);
    assert!(!jury.contains(&all_users[0]));
    assert!(!jury.contains(&all_users[1]));
    assert!(jury.len() <= 11);
}

#[test]
fn test_evidence_confidence_clamped() {
    let ev = evidence::create_evidence(
        Uuid::new_v4(), "test", "https://test.com",
        serde_json::json!({}), MarketOutcome::Yes, 1.5,
    );
    assert!(ev.confidence <= 1.0);

    let ev2 = evidence::create_evidence(
        Uuid::new_v4(), "test", "https://test.com",
        serde_json::json!({}), MarketOutcome::No, -0.5,
    );
    assert!(ev2.confidence >= 0.0);
}
