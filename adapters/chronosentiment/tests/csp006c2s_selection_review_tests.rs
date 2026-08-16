//! CS-P-006-C.2-S — selection and decision-value review. Not a new search.

use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_SNAPSHOT_DIR,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::observation_value::build_observation_slice;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::search_observability::SearchArchive;
use chronosentiment_adapter::decision_support::selection_decision_value::{
    review_selection, DEVELOPMENT_BEST_IDENTITY, SELECTED_IDENTITY,
};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn analysis_and_binary_do_not_search_or_invent_a_cutoff() {
    let files = [
        include_str!("../src/decision_support/selection_decision_value.rs"),
        include_str!("../src/bin/csp006_selection_review.rs"),
    ];
    for src in files {
        assert!(!src.contains("evolve_on_development"));
        assert!(!src.contains("train_policy"));
        assert!(!src.contains("CoralysPhase"));
        assert!(!src.contains("b5_strategy"));
        assert!(!src.contains("borderline_x"));
        assert!(!src.contains("TRANSACTION_COST"));
    }
    let bin = include_str!("../src/bin/csp006_selection_review.rs");
    assert!(bin.contains("RESEARCH_DISCOVERY_ARTIFACT_HASH"));
    assert!(bin.contains("review_selection"));
}

#[test]
fn document_does_not_authorize_search_two_or_freeze_a_band() {
    let doc = include_str!("../../../docs/CS-P-006-C.2-S_SELECTION_DECISION_VALUE.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("not authorized") || doc.contains("Not Search #2"));
    assert!(doc.contains("borderline"));
}

#[test]
fn certified_review_when_archive_and_cache_present() {
    let root = workspace_root();
    let search = root.join(RESEARCH_DISCOVERY_DIR);
    let archive_path = search.join("ecology").join("archive.json");
    let artifact_path = search.join("selected_policy.json");
    let cache_dir = root.join(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache");
    if !archive_path.exists() || !artifact_path.exists() || !cache_dir.join("HDFCBANK.NS.json").exists()
    {
        return;
    }
    let artifact: PolicyArtifact =
        serde_json::from_str(&std::fs::read_to_string(artifact_path).unwrap()).unwrap();
    assert_eq!(artifact.artifact_hash, RESEARCH_DISCOVERY_ARTIFACT_HASH);
    let archive: SearchArchive =
        serde_json::from_str(&std::fs::read_to_string(archive_path).unwrap()).unwrap();
    let cache = load_required_yahoo_cache(&cache_dir).unwrap();
    let partition = certified_research_partition();
    let development = build_observation_slice(
        &cache,
        &partition.development.timestamps,
        PartitionKind::Development,
    )
    .unwrap();
    let selection = build_observation_slice(
        &cache,
        &partition.selection.timestamps,
        PartitionKind::Selection,
    )
    .unwrap();
    let evaluation = build_observation_slice(
        &cache,
        &partition.evaluation.timestamps,
        PartitionKind::Evaluation,
    )
    .unwrap();
    let report = review_selection(
        &artifact.artifact_hash,
        &archive,
        &development,
        &selection,
        &evaluation,
    )
    .unwrap();
    assert_eq!(report.selected.identity, SELECTED_IDENTITY);
    assert_eq!(report.development_best.identity, DEVELOPMENT_BEST_IDENTITY);
    assert!(!report.selected.uses_momentum);
    assert!(report.development_best.uses_momentum);
    assert_eq!(report.bottleneck.n_candidates_presented_to_selection, 2);
    assert!(!report.bottleneck.protocol_requires_generation_best_only);
    assert!(!report.fitness.accuracy_is_the_objective);
    assert!(!report.borderline_boundary_frozen);
    assert!(!report.search_two_authorized);
    assert!(report.selected.evaluation.is_some());
    assert!(report.development_best.evaluation.is_some());
    assert!((report.selected.development.protocol_mean - 0.016325).abs() < 1e-5);
    assert!((report.selected.selection.protocol_mean - 0.019938).abs() < 1e-5);
    assert_eq!(report.bottleneck.n_that_beat_selected_on_selection, 0);
    assert!(!report.development_best.beats_selected_on_selection);
}
