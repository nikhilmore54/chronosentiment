//! CS-P-006-C.3 — protocol authorization. Search #2 is not started.

use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_ARTIFACT_HASH;

#[test]
fn protocol_authorizes_design_not_evolution() {
    let doc = include_str!("../../../docs/CS-P-006-C.3_PROTOCOL.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("C.3-R") || doc.contains("Search #2 not started"));
    assert!(doc.contains("Same TMV"));
    assert!(doc.contains("M.1 continuous V"));
    assert!(doc.contains("unique genomes observed during evolution"));
    assert!(doc.contains("Find a mapping from the certified state at T"));
    assert!(doc.contains("That presupposes a cutoff"));
    assert!(doc.contains("IDEA → don't trade") || doc.contains("Dropping or down-weighting IDEA"));
    assert!(
        doc.contains("Do not:** run Search #2")
            || doc.contains("does not:** run Search #2")
            || doc.contains("**Does not:** run Search #2")
    );
    assert!(!doc.contains("train_policy"));
    assert!(!doc.contains("CoralysPhase"));
    assert!(!doc.contains("b5_strategy"));
}

#[test]
fn no_implementation_starts_search_two() {
    let src = include_str!("../src/decision_support/decision_value_harness.rs");
    assert!(src.contains("C3_AUTHORIZED: bool = false"));
    assert!(!src.contains("evolve_on_development"));
}
