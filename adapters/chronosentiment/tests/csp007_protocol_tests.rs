//! CS-P-007 Statistical Strategy Validation. Specified, not run.

use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp007_protocol::{
    CS007_CONFIRMATORY_AFTER, CS007_DECISION_T_BEFORE, CS007_EVIDENCE_GATE_OPENED,
    CS007_HOMEPAGE_PERFORMANCE_AUTHORIZED, CS007_NULL_SEED, CS007_RETUNE_C3_002_AUTHORIZED,
    CS007_RUN_AUTHORIZED, CS007_SPECIFIED, CS007_TARGET_SEARCH_AUTHORIZED,
    CS007_UNIVERSE_MUTATION_AUTHORIZED, PE3_RUN_AUTHORIZED, REAL_CAPITAL_AUTHORIZED,
};

#[test]
fn cs007_stays_specified_and_unrun() {
    assert!(CS007_SPECIFIED);
    assert!(!CS007_RUN_AUTHORIZED);
    assert!(!CS007_RETUNE_C3_002_AUTHORIZED);
    assert!(!CS007_UNIVERSE_MUTATION_AUTHORIZED);
    assert!(!CS007_TARGET_SEARCH_AUTHORIZED);
    assert!(!CS007_HOMEPAGE_PERFORMANCE_AUTHORIZED);
    assert!(!CS007_EVIDENCE_GATE_OPENED);
    assert!(!PE3_RUN_AUTHORIZED);
    assert!(!REAL_CAPITAL_AUTHORIZED);
    assert_eq!(CS007_NULL_SEED, "cs-p-007-null-v0");
    assert_eq!(CS007_CONFIRMATORY_AFTER, "2024-12-31T15:30:00+00:00");
    assert_eq!(CS007_DECISION_T_BEFORE, "2026-08-14T03:45:00+00:00");
}

#[test]
fn document_defines_the_test_without_running_it() {
    let doc = include_str!("../../../docs/CS-P-007_STATISTICAL_STRATEGY_VALIDATION.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH));
    assert!(doc.contains("Specified — not run"));
    assert!(doc.contains("CS007_RUN_AUTHORIZED = false"));
    assert!(doc.contains("hash_direction_fixed_5pct"));
    assert!(doc.contains("always_long_horizon"));
    assert!(doc.contains("sign_flip_fixed_5pct"));
    assert!(doc.contains("one_position_per_name"));
    assert!(doc.contains("CS-P-007-G"));
    assert!(doc.contains("P.E.3"));
    assert!(doc.contains("IDEA"));
    assert!(doc.contains("MAHABANK"));
    assert!(doc.contains("FREEZE"));
    assert!(doc.contains("The Observatory **homepage stays clean**"));
    assert!(!doc.contains("train_policy"));
    let pe3 = include_str!("../../../docs/CS-P-006-P.E.3_CORALYS_TARGET_DISCOVERY.md");
    assert!(pe3.contains("CS-P-007") || pe3.contains("statistical"));
}
