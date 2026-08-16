//! CS-P-006-C.3-G — regime-persistence question. No experiment. No Search #3.

use chronosentiment_adapter::decision_support::c3_rule_ecology::SEARCH_THREE_AUTHORIZED;
use chronosentiment_adapter::decision_support::csp006_protocol::REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED;

#[test]
fn next_target_is_stated_and_not_started() {
    assert!(!REGIME_PERSISTENCE_EXPERIMENT_AUTHORIZED);
    assert!(!SEARCH_THREE_AUTHORIZED);
}

#[test]
fn document_does_not_authorize_search_three_or_a_detector() {
    let doc = include_str!("../../../docs/CS-P-006-C.3-G_REGIME_PERSISTENCE_QUESTION.md");
    assert!(doc.contains("experiment not authorized"));
    assert!(doc.contains("Search #3 is not authorized"));
    assert!(doc.contains("candidate research artifact"));
    assert!(doc.contains("information available at T"));
    assert!(!doc.contains("Strategy v2"));
    assert!(!doc.contains("b5_strategy"));
}
