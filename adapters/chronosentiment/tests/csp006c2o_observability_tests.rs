//! CS-P-006-C.2-O — observability must not change search identity.

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_SNAPSHOT_DIR, RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::observation_value::{
    build_observation_slice, score_genome, ObservationRow, ObservationSlice,
};
use chronosentiment_adapter::decision_support::policy_discovery::{
    evolve_on_development, evolve_on_development_observed, select_and_observe, select_on_selection,
};
use chronosentiment_adapter::decision_support::policy_genome::RuleListGenome;
use chronosentiment_adapter::decision_support::search_observability::{
    archive_satisfies_contract, per_instrument_scores,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use uuid::Uuid;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn profile(
    ma20: f64,
    ma50: f64,
    roc: f64,
    atr: Option<f64>,
) -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(ma20));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(ma50));
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(roc));
    if let Some(v) = atr {
        metrics
            .metrics
            .insert("atr_14".to_string(), MetricValue::Float(v));
    }
    AssessmentEngine.assess_at(
        &metrics,
        &[Concept::Trend, Concept::Momentum, Concept::Volatility],
        t,
        Some(Uuid::from_u128(7)),
    )
}

fn synthetic_slice(kind: PartitionKind, instrument_return: f64) -> ObservationSlice {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let rows = RESEARCH_UNIVERSE
        .iter()
        .map(|ticker| ObservationRow {
            instrument: (*ticker).to_string(),
            as_of: t,
            profile: profile(110.0, 100.0, 0.05, Some(1.2)),
            instrument_return: Some(instrument_return),
        })
        .collect();
    ObservationSlice { kind, rows }
}

#[test]
fn observability_on_and_off_seal_the_same_artifact() {
    let development = synthetic_slice(PartitionKind::Development, 0.03);
    let selection = synthetic_slice(PartitionKind::Selection, -0.01);
    let (_, off_candidates) = evolve_on_development(development.clone()).unwrap();
    let off = select_on_selection(&off_candidates, &development, &selection).unwrap();
    let (_, on_candidates, archive) =
        evolve_on_development_observed(development.clone()).unwrap();
    let (on, archive) =
        select_and_observe(&on_candidates, &development, &selection, archive).unwrap();
    assert_eq!(off.artifact.artifact_hash, on.artifact.artifact_hash);
    assert!(archive_satisfies_contract(&archive));
    assert!(archive.selected_instruments.is_some());
    assert!(!archive.offspring.is_empty());
}

#[test]
fn observability_must_not_score_evaluation() {
    let genome = RuleListGenome {
        rules: vec![],
        unmatched_action: DecisionAction::NoTrade,
    };
    let evaluation = synthetic_slice(PartitionKind::Evaluation, 0.01);
    assert!(per_instrument_scores(&genome, &evaluation).is_err());
    assert!(score_genome(&genome, &evaluation).is_err());
}

#[test]
fn domain_observability_names_stay_off_the_core_types() {
    let src = include_str!("../src/decision_support/search_observability.rs");
    assert!(!src.contains("train_policy"));
    assert!(!src.contains("test_fitness"));
    assert!(!src.contains("CoralysPhase"));
    assert!(!src.contains("b5_strategy"));
}

#[test]
fn certified_observe_on_off_match_search_one_when_cache_present() {
    let cache_dir = workspace_root()
        .join(RESEARCH_SNAPSHOT_DIR)
        .join("yahoo_cache");
    if !cache_dir.join("HDFCBANK.NS.json").exists() {
        return;
    }
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
    let (_, off_c) = evolve_on_development(development.clone()).unwrap();
    let off = select_on_selection(&off_c, &development, &selection).unwrap();
    let (_, on_c, archive) = evolve_on_development_observed(development.clone()).unwrap();
    let (on, archive) = select_and_observe(&on_c, &development, &selection, archive).unwrap();
    assert_eq!(off.artifact.artifact_hash, on.artifact.artifact_hash);
    assert_eq!(off.artifact.artifact_hash, RESEARCH_DISCOVERY_ARTIFACT_HASH);
    assert!(archive_satisfies_contract(&archive));
}
