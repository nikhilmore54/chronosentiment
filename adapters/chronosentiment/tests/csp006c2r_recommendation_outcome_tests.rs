//! CS-P-006-C.2-R — sealed-artifact recommendation scorecard. Not a new search.

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_SNAPSHOT_DIR,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::dataset_partition::{
    certified_research_partition, PartitionKind,
};
use chronosentiment_adapter::decision_support::observation_value::{
    build_observation_slice, ObservationRow, ObservationSlice,
};
use chronosentiment_adapter::decision_support::policy_artifact::{
    DecisionRule, FactorPredicate, PolicyArtifact, TrainingProvenance,
};
use chronosentiment_adapter::decision_support::recommendation_outcome::{
    score_recommendations, DirectionalCall,
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

fn profile(ma20: f64, ma50: f64) -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
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
        .insert("roc_20".to_string(), MetricValue::Float(0.01));
    metrics
        .metrics
        .insert("atr_14".to_string(), MetricValue::Float(1.0));
    AssessmentEngine.assess_at(
        &metrics,
        &[Concept::Trend, Concept::Momentum, Concept::Volatility],
        t,
        Some(Uuid::from_u128(11)),
    )
}

fn sealed_bearish_long() -> PolicyArtifact {
    let artifact = PolicyArtifact {
        schema_version: "csp006a.policy_artifact.1".to_string(),
        policy_id: "test.scorecard".to_string(),
        policy_version: "v0".to_string(),
        discovery_engine: "coralys.moga.rulelist.v0".to_string(),
        discovery_run_id: "test".to_string(),
        input_schema: vec![
            "Trend".to_string(),
            "Momentum".to_string(),
            "Volatility".to_string(),
        ],
        factor_definitions: vec![],
        action_space: vec![
            DecisionAction::Long,
            DecisionAction::Short,
            DecisionAction::NoTrade,
        ],
        rules: vec![DecisionRule {
            when: vec![FactorPredicate {
                concept: "Trend".to_string(),
                present: Some(true),
                direction: Some("Bearish".to_string()),
            }],
            action: DecisionAction::Long,
        }],
        unmatched_action: DecisionAction::NoTrade,
        training_provenance: TrainingProvenance::default(),
        allowed_information_timestamp: Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap(),
        artifact_hash: "test-sealed".to_string(),
        methodology_hash: "test".to_string(),
    };
    artifact
}

fn slice(kind: PartitionKind, bearish: bool, ret: f64) -> ObservationSlice {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    ObservationSlice {
        kind,
        rows: vec![ObservationRow {
            instrument: "HDFCBANK.NS".to_string(),
            as_of: t,
            profile: if bearish {
                profile(90.0, 100.0)
            } else {
                profile(110.0, 100.0)
            },
            instrument_return: Some(ret),
        }],
    }
}

#[test]
fn long_is_correct_only_when_the_market_rose() {
    let artifact = sealed_bearish_long();
    let (rows, card) = score_recommendations(
        &artifact,
        &slice(PartitionKind::Development, true, 0.04),
        &slice(PartitionKind::Selection, true, -0.03),
        &slice(PartitionKind::Evaluation, false, 0.02),
    )
    .unwrap();
    assert_eq!(rows[0].recommendation, DecisionAction::Long);
    assert_eq!(rows[0].directional_call, DirectionalCall::Correct);
    assert_eq!(rows[1].recommendation, DecisionAction::Long);
    assert_eq!(rows[1].directional_call, DirectionalCall::Incorrect);
    assert_eq!(rows[2].recommendation, DecisionAction::NoTrade);
    assert_eq!(rows[2].directional_call, DirectionalCall::NotApplicable);
    assert_eq!(rows[2].no_trade_winning_alternative, Some(DecisionAction::Long));
    assert!(!card.search_two_authorized);
    assert!(!card.coralys_feedback);
}

#[test]
fn no_trade_is_never_marked_correct() {
    let artifact = sealed_bearish_long();
    let (rows, _) = score_recommendations(
        &artifact,
        &slice(PartitionKind::Development, false, -0.05),
        &slice(PartitionKind::Selection, false, 0.05),
        &slice(PartitionKind::Evaluation, false, 0.0),
    )
    .unwrap();
    assert!(rows.iter().all(|r| r.recommendation == DecisionAction::NoTrade));
    assert!(rows
        .iter()
        .all(|r| r.directional_call == DirectionalCall::NotApplicable));
}

#[test]
fn analysis_and_binary_do_not_search() {
    let files = [
        include_str!("../src/decision_support/recommendation_outcome.rs"),
        include_str!("../src/bin/csp006_recommendation_outcome.rs"),
    ];
    for src in files {
        assert!(!src.contains("evolve_on_development"));
        assert!(!src.contains("train_policy"));
        assert!(!src.contains("test_fitness"));
        assert!(!src.contains("CoralysPhase"));
        assert!(!src.contains("b5_strategy"));
    }
    let bin = include_str!("../src/bin/csp006_recommendation_outcome.rs");
    assert!(bin.contains("RESEARCH_DISCOVERY_ARTIFACT_HASH"));
    assert!(bin.contains("score_recommendations"));
}

#[test]
fn document_does_not_authorize_search_two() {
    let doc = include_str!("../../../docs/CS-P-006-C.2-R_RECOMMENDATION_OUTCOME.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("not authorized") || doc.contains("Not Search #2"));
    assert!(doc.contains("Generalization: FAIL"));
    assert!(doc.contains("NO_TRADE is standing aside"));
}

#[test]
fn on_disk_scorecard_matches_search_one_when_present() {
    let path = workspace_root()
        .join(RESEARCH_DISCOVERY_DIR)
        .join("recommendations")
        .join("scorecard.json");
    if !path.exists() {
        return;
    }
    let card: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(card["policy_artifact_hash"], RESEARCH_DISCOVERY_ARTIFACT_HASH);
    assert_eq!(card["n_recommendations"], 273);
    assert_eq!(card["overall"]["long"]["n"], 121);
    assert_eq!(card["overall"]["short"]["n"], 0);
    assert_eq!(card["overall"]["no_trade"]["n"], 152);
    assert_eq!(card["generalization"], "FAIL");
    assert_eq!(card["search_two_authorized"], false);
    assert_eq!(card["coralys_feedback"], false);
}

#[test]
fn certified_search_one_scorecard_when_cache_present() {
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
    assert_eq!(artifact.artifact_hash, RESEARCH_DISCOVERY_ARTIFACT_HASH);
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
    let (rows, card) =
        score_recommendations(&artifact, &development, &selection, &evaluation).unwrap();
    assert_eq!(rows.len(), 273);
    assert_eq!(card.overall.short.n, 0);
    assert_eq!(card.development.long.n, 49);
    assert_eq!(card.selection.long.n, 39);
    assert_eq!(card.evaluation.long.n, 33);
    assert!((card.development.protocol_mean_signed_traded_return - 0.016325).abs() < 1e-5);
    assert!((card.selection.protocol_mean_signed_traded_return - 0.019938).abs() < 1e-5);
    assert!((card.evaluation.protocol_mean_signed_traded_return + 0.000229).abs() < 1e-5);
    assert_eq!(card.generalization, "FAIL");
    assert!(rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::NoTrade)
        .all(|r| r.directional_call == DirectionalCall::NotApplicable));
}
