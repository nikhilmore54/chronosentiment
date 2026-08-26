//! CS-P-006-C — Coralys TMV discovery invariants.
//!
//! Search may use development and selection outcomes. Evaluation is forbidden
//! as a search input. Same certified development state + frozen seed must
//! reproduce the same PolicyArtifact identity.

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    coralys_search_is_authorized, MAX_RULES_FIRST_DISCOVERY, RESEARCH_DISCOVERY_ARTIFACT_HASH,
    RESEARCH_DISCOVERY_DIR, RESEARCH_DISCOVERY_METHODOLOGY_HASH, RESEARCH_SNAPSHOT_DIR,
    RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::observation_value::{
    build_observation_slice, score_genome, ObservationRow, ObservationSlice, DISCOVERY_HORIZON_DAYS,
};
use chronosentiment_adapter::decision_support::policy_discovery::{
    evolve_on_development, select_on_selection, FROZEN_SEED,
};
use chronosentiment_adapter::decision_support::policy_genome::RuleListGenome;
use chronosentiment_adapter::decision_support::policy_handoff::evaluate_sealed_candidate;
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
fn search_is_authorized_and_horizon_is_not_the_lake_series() {
    assert!(coralys_search_is_authorized());
    assert_eq!(DISCOVERY_HORIZON_DAYS, 20);
    assert_ne!(DISCOVERY_HORIZON_DAYS, 60);
    assert_eq!(FROZEN_SEED, 42);
}

#[test]
fn score_genome_rejects_the_evaluation_slice() {
    let genome = RuleListGenome {
        rules: vec![],
        unmatched_action: DecisionAction::NoTrade,
    };
    let err = score_genome(&genome, &synthetic_slice(PartitionKind::Evaluation, 0.01))
        .expect_err("evaluation must be forbidden to search scoring");
    assert!(err.contains("must not score the evaluation slice"));
}

#[test]
fn score_genome_permits_development_and_selection() {
    let genome = RuleListGenome {
        rules: vec![],
        unmatched_action: DecisionAction::Long,
    };
    assert!(score_genome(&genome, &synthetic_slice(PartitionKind::Development, 0.02)).is_ok());
    assert!(score_genome(&genome, &synthetic_slice(PartitionKind::Selection, 0.01)).is_ok());
}

#[test]
fn same_seed_twice_reproduces_policy_artifact_identity() {
    let development = synthetic_slice(PartitionKind::Development, 0.03);
    let selection = synthetic_slice(PartitionKind::Selection, -0.01);
    let (evidence_a, candidates_a) = evolve_on_development(development.clone()).unwrap();
    let selected_a = select_on_selection(&candidates_a, &development, &selection).unwrap();
    let (evidence_b, candidates_b) = evolve_on_development(development.clone()).unwrap();
    let selected_b = select_on_selection(&candidates_b, &development, &selection).unwrap();
    assert_eq!(
        selected_a.artifact.artifact_hash, selected_b.artifact.artifact_hash,
        "same development state + frozen seed must seal the same artifact"
    );
    assert_eq!(
        evidence_a.development_best_fitness,
        evidence_b.development_best_fitness
    );
    assert!(selected_a.genome.rules.len() <= MAX_RULES_FIRST_DISCOVERY);
    for rule in &selected_a.artifact.rules {
        for pred in &rule.when {
            if pred.concept == "Volatility" {
                assert!(pred.direction.is_none());
            }
        }
    }
}

#[test]
fn search_evidence_does_not_carry_evaluation_fitness() {
    let development = synthetic_slice(PartitionKind::Development, 0.02);
    let (evidence, _) = evolve_on_development(development).unwrap();
    let json = serde_json::to_value(&evidence).unwrap();
    let obj = json.as_object().unwrap();
    assert!(obj.contains_key("development_best_fitness"));
    assert!(!obj.contains_key("evaluation"));
    assert!(!obj.contains_key("evaluation_fitness"));
    assert!(!obj.contains_key("test_fitness"));
    let blob = serde_json::to_string(&evidence).unwrap();
    assert!(!blob.contains("evaluation_fitness"));
    assert!(!blob.contains("test_fitness"));
}

#[test]
fn handoff_scores_evaluation_without_going_through_score_genome() {
    let development = synthetic_slice(PartitionKind::Development, 0.03);
    let selection = synthetic_slice(PartitionKind::Selection, 0.01);
    let evaluation = synthetic_slice(PartitionKind::Evaluation, -0.02);
    let (_, candidates) = evolve_on_development(development.clone()).unwrap();
    let selected = select_on_selection(&candidates, &development, &selection).unwrap();
    let handoff = evaluate_sealed_candidate(&selected.artifact, &evaluation).unwrap();
    assert_eq!(handoff.artifact_hash, selected.artifact.artifact_hash);
    assert!(score_genome(&selected.genome, &evaluation).is_err());
}

#[test]
fn domain_modules_do_not_encode_phase_or_baseline_seed_names() {
    let files = [
        include_str!("../src/decision_support/policy_genome.rs"),
        include_str!("../src/decision_support/policy_discovery.rs"),
        include_str!("../src/decision_support/observation_value.rs"),
        include_str!("../src/decision_support/policy_handoff.rs"),
        include_str!("../src/bin/csp006_policy_discovery.rs"),
    ];
    for src in files {
        for forbidden in [
            "train_policy",
            "validation_candidate",
            "test_fitness",
            "cs006_phase_c_population",
            "b5_strategy",
            "CoralysPhase",
            "BaselineTrendMappingPolicy",
        ] {
            assert!(
                !src.contains(forbidden),
                "discovery sources must not contain {forbidden}"
            );
        }
        assert!(!src.contains("G-GATE"));
        assert!(!src.contains("unavailable=85"));
        assert!(!src.contains("110 LONG"));
    }
}

#[test]
fn certified_snapshot_same_seed_reproduces_artifact_when_cache_present() {
    let cache_dir = workspace_root()
        .join(RESEARCH_SNAPSHOT_DIR)
        .join("yahoo_cache");
    if !cache_dir.join("HDFCBANK.NS.json").exists() {
        return;
    }
    let cache = load_required_yahoo_cache(&cache_dir).expect("certified yahoo cache");
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
    let (e1, c1) = evolve_on_development(development.clone()).unwrap();
    let s1 = select_on_selection(&c1, &development, &selection).unwrap();
    let (e2, c2) = evolve_on_development(development.clone()).unwrap();
    let s2 = select_on_selection(&c2, &development, &selection).unwrap();
    assert_eq!(s1.artifact.artifact_hash, s2.artifact.artifact_hash);
    assert_eq!(e1.methodology_hash, e2.methodology_hash);
    assert!(s1.genome.rules.len() <= MAX_RULES_FIRST_DISCOVERY);
}

#[test]
fn first_run_on_disk_artifact_matches_frozen_identity_when_present() {
    let path = workspace_root()
        .join(RESEARCH_DISCOVERY_DIR)
        .join("selected_policy.json");
    if !path.exists() {
        return;
    }
    let artifact: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(artifact["artifact_hash"], RESEARCH_DISCOVERY_ARTIFACT_HASH);
    assert_eq!(
        artifact["methodology_hash"],
        RESEARCH_DISCOVERY_METHODOLOGY_HASH
    );
    assert_eq!(artifact["discovery_engine"], "coralys.moga.rulelist.v0");
    assert_eq!(artifact["schema_version"], "csp006a.policy_artifact.1");
}

#[test]
fn discovery_document_records_the_frozen_run_without_promotion() {
    let doc = include_str!("../../../docs/CS-P-006-C_POLICY_DISCOVERY.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains(RESEARCH_DISCOVERY_METHODOLOGY_HASH));
    assert!(doc.contains("Do not retune"));
    assert!(doc.contains("not a promoted ChronoSentiment strategy"));
}
