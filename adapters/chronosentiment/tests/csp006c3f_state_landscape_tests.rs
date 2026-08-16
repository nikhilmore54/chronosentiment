//! CS-P-006-C.3-F — state × action landscape. No Search #3. No product claim.

use std::path::PathBuf;

use chronosentiment_adapter::decision_support::c3_rule_ecology::{
    SEARCH_THREE_AUTHORIZED, SEARCH_TWO_PROMOTION_STATUS,
};
use chronosentiment_adapter::decision_support::c3_state_landscape::{
    analyze_state_landscape, PASS_THRESHOLD_INTRODUCED, PRODUCT_CLAIM_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
    RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::recommendation_outcome::RecommendationRow;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn search_two_stays_a_candidate_without_a_product_claim() {
    assert_eq!(SEARCH_TWO_PROMOTION_STATUS, "candidate_research_artifact");
    assert!(!SEARCH_THREE_AUTHORIZED);
    assert!(!PASS_THRESHOLD_INTRODUCED);
    assert!(!PRODUCT_CLAIM_AUTHORIZED);
}

#[test]
fn landscape_covers_certified_states_and_keeps_actions_as_counterfactuals() {
    let one_path = workspace_root()
        .join(RESEARCH_DISCOVERY_DIR)
        .join("recommendations")
        .join("recommendations.json");
    let two_path = workspace_root()
        .join(RESEARCH_DISCOVERY_TWO_DIR)
        .join("recommendations")
        .join("recommendations.json");
    if !one_path.exists() || !two_path.exists() {
        return;
    }
    let search_one: Vec<RecommendationRow> =
        serde_json::from_str(&std::fs::read_to_string(one_path).unwrap()).unwrap();
    let search_two: Vec<RecommendationRow> =
        serde_json::from_str(&std::fs::read_to_string(two_path).unwrap()).unwrap();
    let report = analyze_state_landscape(&search_one, &search_two).unwrap();
    assert_eq!(report.search_one_artifact_hash, RESEARCH_DISCOVERY_ARTIFACT_HASH);
    assert_eq!(
        report.search_two_artifact_hash,
        RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
    );
    assert_eq!(report.n_rows, 273);
    assert_eq!(report.states.iter().map(|s| s.n).sum::<u32>(), 273);
    assert!(!report.search_three_authorized);
    assert!(!report.product_claim_authorized);
    for state in &report.states {
        assert!((state.overall.long + state.overall.short).abs() < 1e-12);
        assert_eq!(state.overall.no_trade, 0.0);
        assert!((state.evaluation.long + state.evaluation.short).abs() < 1e-12);
    }
}

#[test]
fn analysis_does_not_evolve_or_authorize_a_product_card() {
    let src = include_str!("../src/decision_support/c3_state_landscape.rs");
    let bin = include_str!("../src/bin/csp006_c3_state_landscape.rs");
    for text in [src, bin] {
        assert!(!text.contains("evolve_"));
        assert!(!text.contains("train_fitness"));
        assert!(!text.contains("phase_c3_population"));
        assert!(!text.contains("Strategy v2"));
        assert!(!text.contains("b5_strategy"));
        assert!(!text.contains("Outcomes that you can hope"));
    }
}

#[test]
fn document_keeps_the_landscape_off_the_product_path() {
    let doc = include_str!("../../../docs/CS-P-006-C.3-F_STATE_ACTION_LANDSCAPE.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH));
    assert!(doc.contains("candidate research artifact"));
    assert!(doc.contains("Search #3 is not authorized"));
    assert!(doc.contains("product claim") || doc.contains("product-facing"));
}
