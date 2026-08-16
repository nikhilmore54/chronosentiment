//! CS-P-006-P — Decision Observatory. P.1 registry only. No Search #3.

use chronosentiment_adapter::decision_support::c3_rule_ecology::{
    SEARCH_THREE_AUTHORIZED, SEARCH_TWO_PROMOTION_STATUS,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
};
use chronosentiment_adapter::decision_support::observatory_registry::{
    candidate_c3_002, register_paper_policy, CANDIDATE_C3_002, CANDIDATE_C3_002_LABEL,
    OBSERVATORY_P2_STARTED,
};

#[test]
fn candidate_c3_002_is_paper_only_search_two() {
    let entry = candidate_c3_002();
    assert_eq!(entry.registry_id, CANDIDATE_C3_002);
    assert_eq!(entry.label, CANDIDATE_C3_002_LABEL);
    assert_eq!(entry.artifact_hash, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH);
    assert_eq!(entry.promotion_status, SEARCH_TWO_PROMOTION_STATUS);
    assert!(entry.paper_only);
    assert_eq!(entry.customer_facing_system, "ChronoSentiment");
    assert!(!entry.search_three_authorized);
    assert!(!entry.regime_persistence_experiment_authorized);
    assert!(!entry.real_capital_authorized);
    assert!(!SEARCH_THREE_AUTHORIZED);
    assert!(!REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED);
    assert!(!OBSERVATORY_P2_STARTED);
}

#[test]
fn registry_refuses_any_other_artifact() {
    assert!(register_paper_policy("not-search-two").is_err());
    assert!(register_paper_policy(RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH).is_ok());
}

#[test]
fn source_does_not_rename_the_candidate_a_strategy() {
    let src = include_str!("../src/decision_support/observatory_registry.rs");
    assert!(!src.contains("Strategy v2"));
    assert!(!src.contains("b5_strategy"));
    assert!(!src.contains("evolve_"));
    assert!(!src.contains("87%"));
}

#[test]
fn document_keeps_research_quarantined_and_the_product_paper_only() {
    let doc = include_str!("../../../docs/CS-P-006-P_DECISION_OBSERVATORY.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH));
    assert!(doc.contains("Candidate C3-002"));
    assert!(doc.contains("Search #3"));
    assert!(doc.contains("C.3-G remains a question") || doc.contains("C.3-G remains an unanswered"));
    assert!(doc.contains("paper"));
    assert!(doc.contains("Not Strategy v2"));
}
