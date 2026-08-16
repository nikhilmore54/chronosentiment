//! CS-P-006-M — decision-value model is protocol, not a new search.

use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_ARTIFACT_HASH;

#[test]
fn document_does_not_authorize_search_two_or_advantage_fitness() {
    let doc = include_str!("../../../docs/CS-P-006-M_DECISION_VALUE_MODEL.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("not authorized") || doc.contains("Search #2"));
    assert!(doc.contains("They are not Search #1 fitness and they are not authorized as Search #2 fitness"));
    assert!(doc.contains("OPEN. Must not be fitted to this landscape"));
    assert!(doc.contains("CS-P-006-M.1"));
    assert!(!doc.contains("train_policy"));
    assert!(!doc.contains("CoralysPhase"));
    assert!(!doc.contains("b5_strategy"));
}
