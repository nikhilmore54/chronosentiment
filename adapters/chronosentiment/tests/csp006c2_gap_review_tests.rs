//! CS-P-006-C.2 — research-gap review invariants.
//!
//! Search #1 remains the control. This review does not evolve.

use chronosentiment_adapter::decision_support::search_observability::{
    missing_from_search_one, search_one_gap_review, search_one_recorded_fields,
    search_one_satisfies_observability, OBSERVABILITY_CONTRACT_ID,
};

#[test]
fn search_one_fails_the_observability_contract() {
    assert!(!search_one_satisfies_observability());
    let missing = missing_from_search_one();
    assert!(missing.contains(&"unique_genome_count_by_generation"));
    assert!(missing.contains(&"median_fitness_by_generation"));
    assert!(missing.contains(&"population_action_symbol_histogram"));
    assert!(missing.contains(&"population_factor_consumption_histogram"));
    assert!(missing.contains(&"serialized_generation_best_rules"));
    assert!(search_one_recorded_fields().contains(&"generation_best_fitness"));
}

#[test]
fn gap_review_does_not_authorize_search_two_or_choose_volatility() {
    let review = search_one_gap_review();
    assert_eq!(review.contract_id, OBSERVABILITY_CONTRACT_ID);
    assert!(!review.search_one_satisfies_contract);
    assert!(!review.volatility_presence_discriminates_on_s1);
    assert!(!review.volatility_encoding_chosen);
    assert!(!review.search_two_authorized);
}

#[test]
fn observability_module_does_not_search_or_invent_thresholds() {
    let src = include_str!("../src/decision_support/search_observability.rs");
    assert!(!src.contains("evolve_on_development"));
    assert!(!src.contains("EvolutionEngine"));
    assert!(!src.contains("High/Low"));
    assert!(!src.contains("global ATR"));
    assert!(!src.contains("CoralysPhase"));
    assert!(!src.contains("b5_strategy"));
}

#[test]
fn gap_document_preserves_search_one_and_refuses_search_two() {
    let doc = include_str!("../../../docs/CS-P-006-C.2_RESEARCH_GAP_REVIEW.md");
    assert!(doc.contains("9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0"));
    assert!(doc.contains("Search #2 / C.3 not authorized"));
    assert!(doc.contains("encoding not chosen"));
    assert!(doc.contains("NO RETUNING"));
    assert!(!doc.contains("evolve_on_development"));
}
