//! CS-P-006-B — protocol freeze without invented split dates.

use chronosentiment_adapter::decision_support::csp006_protocol::{
    audit_b4_coverage, coralys_search_is_authorized, CoverageAudit,
    CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT, CHRONOLOGICAL_PARTITION_FROZEN, MAX_RULES_FIRST_DISCOVERY,
    RESEARCH_SNAPSHOT_CERTIFIED, RESEARCH_UNIVERSE,
};

#[test]
fn intended_universe_is_seven_named_instruments() {
    assert_eq!(RESEARCH_UNIVERSE.len(), 7);
    assert!(RESEARCH_UNIVERSE.contains(&"IDEA.NS"));
    assert!(RESEARCH_UNIVERSE.contains(&"MAHABANK.NS"));
    assert_eq!(MAX_RULES_FIRST_DISCOVERY, 16);
}

#[test]
fn certified_snapshot_is_five_and_must_not_stand_in_for_seven() {
    assert_eq!(CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT.len(), 5);
    assert!(!CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT.contains(&"IDEA.NS"));
    assert!(!CERTIFIED_FIVE_INSTRUMENT_SNAPSHOT.contains(&"MAHABANK.NS"));
}

#[test]
fn b4_coverage_audit_is_insufficient_and_must_not_stand_in_for_seven() {
    match audit_b4_coverage() {
        CoverageAudit::Insufficient { missing } => {
            assert!(missing.contains(&"IDEA.NS"));
            assert!(missing.contains(&"MAHABANK.NS"));
            assert_eq!(missing.len(), 2);
        }
        CoverageAudit::Sufficient => {
            panic!("must not treat five-instrument B4/CS-P-005 as the seven-name universe")
        }
    }
}

#[test]
fn research_snapshot_and_partition_authorize_search() {
    assert!(RESEARCH_SNAPSHOT_CERTIFIED);
    assert!(CHRONOLOGICAL_PARTITION_FROZEN);
    assert!(coralys_search_is_authorized());
}

#[test]
fn protocol_module_does_not_contain_a_search_engine_or_frozen_calendar_split() {
    let src = include_str!("../src/decision_support/csp006_protocol.rs");
    assert!(!src.contains("EvolutionEngine"));
    assert!(!src.contains("FitnessEvaluator"));
    assert!(!src.contains("TRAIN_START"));
    assert!(!src.contains("2022-01-01"));
    assert!(!src.contains("2023-01-01"));
    assert!(!src.contains("2024-01-01"));
}

#[test]
fn protocol_document_keeps_b4_insufficient_and_points_at_b1() {
    let doc = include_str!("../../../docs/CS-P-006-B_RESEARCH_PROTOCOL.md");
    assert!(doc.contains("INSUFFICIENT"));
    assert!(doc.contains("MAHABANK.NS"));
    assert!(doc.contains("CS-P-006-B.1"));
    assert!(!doc.contains("Train window: 2022-01-01"));
}

#[test]
fn partition_document_freezes_hash_and_equal_thirds() {
    let doc = include_str!("../../../docs/CS-P-006-B.1_CHRONOLOGICAL_PARTITION.md");
    assert!(doc.contains("4354c81ef546003b1d11ec98cba83dd5f8c56b13c8b6055b8451614abdc4cfca"));
    assert!(doc.contains("contiguous equal thirds"));
    assert!(doc.contains("PASS"));
    assert!(doc.contains("Not used: G-GATE 55/27/28"));
}

#[test]
fn protocol_does_not_encode_mtf_or_var_strategy_rules() {
    let doc = include_str!("../../../docs/CS-P-006-B_RESEARCH_PROTOCOL.md");
    assert!(!doc.contains("VaR"));
    assert!(!doc.contains("MTF"));
    assert!(!doc.contains("leverage"));
    assert!(!doc.contains("funding rate"));
    let src = include_str!("../src/decision_support/csp006_protocol.rs");
    assert!(!src.contains("VaR"));
    assert!(!src.contains("MTF"));
}
