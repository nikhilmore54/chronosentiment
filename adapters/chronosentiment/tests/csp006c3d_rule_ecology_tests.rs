//! CS-P-006-C.3-D — live-rule ecology. No Search #3. Not a promotion.

use std::path::PathBuf;

use chronosentiment_adapter::decision_support::c3_rule_ecology::{
    analyze_live_rules, SEARCH_THREE_AUTHORIZED, SEARCH_TWO_PROMOTION_STATUS,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR,
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
fn search_two_is_a_candidate_not_a_strategy() {
    assert_eq!(SEARCH_TWO_PROMOTION_STATUS, "candidate_research_artifact");
    assert!(!SEARCH_THREE_AUTHORIZED);
}

#[test]
fn ecology_identity_gates_search_two_and_covers_all_rows() {
    let dir = workspace_root().join(RESEARCH_DISCOVERY_TWO_DIR);
    let recs = dir.join("recommendations").join("recommendations.json");
    if !recs.exists() {
        return;
    }
    let recommendations: Vec<RecommendationRow> =
        serde_json::from_str(&std::fs::read_to_string(recs).unwrap()).unwrap();
    let artifact: PolicyArtifact =
        serde_json::from_str(&std::fs::read_to_string(dir.join("selected_policy.json")).unwrap())
            .unwrap();
    let report = analyze_live_rules(&recommendations, &artifact).unwrap();
    assert_eq!(
        report.search_two_artifact_hash,
        RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
    );
    assert_eq!(report.n_rows, 273);
    assert_eq!(report.live_rules.iter().map(|r| r.n).sum::<u32>(), 273);
    assert_eq!(report.live_rules.len(), 3);
    assert!(!report.search_three_authorized);
    let otherwise = report
        .live_rules
        .iter()
        .find(|r| r.rule_index == 3)
        .unwrap();
    assert!(otherwise
        .fired_states
        .iter()
        .all(|s| s.trend_state == "Bullish"));
}

#[test]
fn analysis_does_not_evolve_or_rename_the_candidate() {
    let src = include_str!("../src/decision_support/c3_rule_ecology.rs");
    let bin = include_str!("../src/bin/csp006_c3_rule_ecology.rs");
    for text in [src, bin] {
        assert!(!text.contains("evolve_"));
        assert!(!text.contains("train_fitness"));
        assert!(!text.contains("phase_c3_population"));
        assert!(!text.contains("Strategy v2"));
        assert!(!text.contains("b5_strategy"));
    }
}

#[test]
fn document_keeps_search_two_a_candidate() {
    let doc = include_str!("../../../docs/CS-P-006-C.3-D_RULE_ECOLOGY.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH));
    assert!(doc.contains("candidate research artifact"));
    assert!(doc.contains("Search #3 is not authorized"));
    assert!(doc.contains("what `Bullish otherwise") || doc.contains("otherwise"));
}
