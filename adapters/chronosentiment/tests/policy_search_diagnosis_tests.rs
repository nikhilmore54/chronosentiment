//! CS-P-006-C.1 — Search #1 diagnosis invariants.
//!
//! Diagnosis inspects the sealed artifact. It must not evolve or retune.

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
use chronosentiment_adapter::decision_support::policy_search_diagnosis::diagnose_sealed_artifact;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn diagnosis_sources_do_not_evolve_or_promote() {
    let files = [
        include_str!("../src/decision_support/policy_search_diagnosis.rs"),
        include_str!("../src/bin/csp006_search_diagnosis.rs"),
    ];
    for src in files {
        assert!(!src.contains("evolve_on_development"));
        assert!(!src.contains("select_on_selection"));
        assert!(!src.contains("BaselineTrendMappingPolicy"));
        assert!(!src.contains("CoralysPhase"));
        assert!(!src.contains("b5_strategy"));
    }
}

#[test]
fn sealed_search_one_has_no_short_and_matches_frozen_hash_when_present() {
    let root = workspace_root();
    let artifact_path = root
        .join(RESEARCH_DISCOVERY_DIR)
        .join("selected_policy.json");
    let cache_dir = root.join(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache");
    if !artifact_path.exists() || !cache_dir.join("HDFCBANK.NS.json").exists() {
        return;
    }
    let artifact: PolicyArtifact =
        serde_json::from_str(&std::fs::read_to_string(artifact_path).unwrap()).unwrap();
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
    let report =
        diagnose_sealed_artifact(&artifact, &development, &selection, &evaluation, 12, 2, 2)
            .unwrap();
    assert_eq!(report.artifact_hash, RESEARCH_DISCOVERY_ARTIFACT_HASH);
    assert_eq!(report.development.actions.short, 0);
    assert_eq!(report.selection.actions.short, 0);
    assert_eq!(report.evaluation.actions.short, 0);
    assert!(!report.representation.selected_uses_momentum);
    assert!(!report.representation.selected_emits_short);
    assert!(!report.archive.population_diversity_recorded);
}

#[test]
fn diagnosis_document_does_not_authorize_search_two_or_promotion() {
    let doc = include_str!("../../../docs/CS-P-006-C.1_SEARCH_DIAGNOSIS.md");
    assert!(doc.contains("9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0"));
    assert!(doc.contains("failed generalization"));
    assert!(doc.contains("Search #2 not authorized"));
    assert!(doc.contains("**not** promoted"));
}
