//! CS-P-006-C.3-C — sealed comparison. No Search #3.

use std::path::PathBuf;

use chronosentiment_adapter::decision_support::c3_comparison::compare_sealed_recommendations;
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
    RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
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
fn comparison_identity_gates_both_artifacts() {
    let one_dir = workspace_root().join(RESEARCH_DISCOVERY_DIR);
    let two_dir = workspace_root().join(RESEARCH_DISCOVERY_TWO_DIR);
    let one_recs = one_dir.join("recommendations").join("recommendations.json");
    let two_recs = two_dir.join("recommendations").join("recommendations.json");
    if !one_recs.exists() || !two_recs.exists() {
        return;
    }
    let one: Vec<RecommendationRow> =
        serde_json::from_str(&std::fs::read_to_string(one_recs).unwrap()).unwrap();
    let two: Vec<RecommendationRow> =
        serde_json::from_str(&std::fs::read_to_string(two_recs).unwrap()).unwrap();
    let art: PolicyArtifact = serde_json::from_str(
        &std::fs::read_to_string(two_dir.join("selected_policy.json")).unwrap(),
    )
    .unwrap();
    let report = compare_sealed_recommendations(&one, &two, &art).unwrap();
    assert_eq!(report.n_rows, 273);
    assert_eq!(report.search_one_artifact_hash, RESEARCH_DISCOVERY_ARTIFACT_HASH);
    assert_eq!(report.search_two_artifact_hash, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH);
    assert!(!report.search_three_authorized);
    assert!(!report.used_as_coralys_fitness);
    assert_eq!(
        report.no_trade_conversion.n_converted_to_long
            + report.no_trade_conversion.n_converted_to_short
            + report.no_trade_conversion.n_still_no_trade,
        report.no_trade_conversion.n_search_one_no_trade
    );
    assert_eq!(
        report.pairwise_all.search_two_better
            + report.pairwise_all.search_one_better
            + report.pairwise_all.tie,
        273
    );
}

#[test]
fn analysis_does_not_evolve_or_authorize_search_three() {
    let src = include_str!("../src/decision_support/c3_comparison.rs");
    let bin = include_str!("../src/bin/csp006_c3_comparison.rs");
    for text in [src, bin] {
        assert!(!text.contains("evolve_decision_value"));
        assert!(!text.contains("evolve_on_development"));
        assert!(!text.contains("train_fitness"));
        assert!(!text.contains("phase_c3_population"));
        assert!(!text.contains("CoralysPhase"));
        assert!(!text.contains("b5_strategy"));
    }
    assert!(src.contains("search_three_authorized: false"));
}

#[test]
fn document_freezes_both_searches_and_forbids_search_three() {
    let doc = include_str!("../../../docs/CS-P-006-C.3-C_COMPARATIVE_REVIEW.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains(RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH));
    assert!(doc.contains("No Search #3") || doc.contains("Search #3 is not authorized"));
    assert!(doc.contains("what did Coralys learn"));
}
