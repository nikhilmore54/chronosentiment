//! CS-P-006-C.3-R — one authorized Search #2. Search #1 stays immutable.

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::c3_implementation::{
    evolve_on_development_value, search_one_evidence_is_immutable, SEARCH_TWO_RUN_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::c3_run::{
    c3_run_is_authorized, decision_value_methodology_hash, evolve_decision_value_on_development,
    refuse_search_one_output, C3_RUN_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_DISCOVERY_METHODOLOGY_HASH,
    RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::dataset_partition::PartitionKind;
use chronosentiment_adapter::decision_support::decision_value_fitness::score_decision_value;
use chronosentiment_adapter::decision_support::observation_value::{ObservationRow, ObservationSlice};
use chronosentiment_adapter::decision_support::policy_discovery::methodology_hash;
use chronosentiment_adapter::decision_support::policy_genome::RuleListGenome;
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

fn profile() -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(110.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(100.0));
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(0.05));
    metrics
        .metrics
        .insert("atr_14".to_string(), MetricValue::Float(1.2));
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
            profile: profile(),
            instrument_return: Some(instrument_return),
        })
        .collect();
    ObservationSlice { kind, rows }
}

#[test]
fn c3i_gate_stays_closed_and_run_module_is_authorized() {
    assert!(!SEARCH_TWO_RUN_AUTHORIZED);
    assert!(C3_RUN_AUTHORIZED);
    assert!(c3_run_is_authorized());
    assert!(evolve_on_development_value(synthetic_slice(PartitionKind::Development, 0.01)).is_err());
}

#[test]
fn search_one_methodology_and_evidence_stay_immutable() {
    assert_eq!(methodology_hash(), RESEARCH_DISCOVERY_METHODOLOGY_HASH);
    assert_ne!(decision_value_methodology_hash(), RESEARCH_DISCOVERY_METHODOLOGY_HASH);
    let search_one = workspace_root().join(RESEARCH_DISCOVERY_DIR);
    if search_one.join("selected_policy.json").exists() {
        search_one_evidence_is_immutable(&search_one).unwrap();
    }
    assert!(refuse_search_one_output(&search_one).is_err());
    assert!(refuse_search_one_output(&PathBuf::from("product_validation/CS-P-006/discovery/20260814T195327Z")).is_err());
}

#[test]
fn evaluation_is_not_an_evolve_argument_and_cannot_be_scored() {
    let src = include_str!("../src/decision_support/c3_run.rs");
    assert!(src.contains("Evaluation is not an argument"));
    assert!(!src.contains("PartitionKind::Evaluation"));
    let genome = RuleListGenome {
        rules: vec![],
        unmatched_action: DecisionAction::Long,
    };
    assert!(score_decision_value(&genome, &synthetic_slice(PartitionKind::Evaluation, 0.01)).is_err());
}

#[test]
fn synthetic_run_completes_frozen_configuration() {
    let development = synthetic_slice(PartitionKind::Development, 0.03);
    let (evidence, archive) = evolve_decision_value_on_development(development).unwrap();
    assert_eq!(evidence.seed, 42);
    assert_eq!(evidence.population_size, 32);
    assert_eq!(evidence.generation_limit, 12);
    assert_eq!(evidence.generation_best_value.len(), 12);
    assert_eq!(evidence.n_instruments, 7);
    assert_eq!(evidence.horizon_days, 20);
    assert_eq!(archive.generations.len(), 12);
    assert!(evidence.n_living_candidates > 2);
}

#[test]
fn domain_names_stay_semantic() {
    let files = [
        include_str!("../src/decision_support/c3_run.rs"),
        include_str!("../src/bin/csp006_c3_search.rs"),
    ];
    for src in files {
        assert!(!src.contains("train_fitness"));
        assert!(!src.contains("validation_candidates"));
        assert!(!src.contains("test_score"));
        assert!(!src.contains("phase_c3_population"));
        assert!(!src.contains("CoralysPhase"));
        assert!(!src.contains("b5_strategy"));
    }
}

#[test]
fn document_authorizes_one_run_not_iteration() {
    let doc = include_str!("../../../docs/CS-P-006-C.3-R_SEARCH.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("one complete"));
    assert!(doc.contains("Do not iterate") || doc.contains("not permission to iterate"));
    assert!(doc.contains("development value"));
}
