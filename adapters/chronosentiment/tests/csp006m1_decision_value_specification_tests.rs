//! CS-P-006-M.1 — decision-value specification. Not a new search.

use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_ARTIFACT_HASH;

#[test]
fn specification_freezes_continuous_v_and_forbids_regret_fitness() {
    let spec = include_str!("../../../docs/CS-P-006-M.1_DECISION_VALUE_SPECIFICATION.md");
    assert!(spec.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(spec.contains("V(LONG)     =  R"));
    assert!(spec.contains("V(SHORT)    = −R"));
    assert!(spec.contains("V(NO_TRADE) =  0"));
    assert!(spec.contains("20 calendar days"));
    assert!(spec.contains("independent decision opportunity"));
    assert!(spec.contains("mean of seven per-instrument means"));
    assert!(spec.contains("cost_term_present = false"));
    assert!(spec.contains("not an assumption that markets are frictionless"));
    assert!(spec.contains("It is **not**"));
    assert!(spec.contains("`−regret`"));
    assert!(spec.contains("unique_best = true  → fitness 1"));
    assert!(spec.contains("Evaluation outcomes"));
    assert!(spec.contains("not authorized") || spec.contains("Not Search #2"));
    assert!(!spec.contains("train_policy"));
    assert!(!spec.contains("CoralysPhase"));
    assert!(!spec.contains("b5_strategy"));
}

#[test]
fn harness_spec_does_not_evolve_or_authorize_search_two() {
    let harness = include_str!("../../../docs/CS-P-006-N_DECISION_VALUE_RESEARCH_HARNESS.md");
    assert!(harness.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(harness.contains("C.3 is not authorized") || harness.contains("C.3 not authorized"));
    assert!(harness.contains("Table A"));
    assert!(harness.contains("does not") || harness.contains("Does not"));
    assert!(harness.contains("evolve") || harness.contains("evolve_on_development"));
    assert!(!harness.contains("train_policy"));
    assert!(!harness.contains("b5_strategy"));
}
