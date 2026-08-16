//! CS-P-006-N — measurement harness. Not a search. C.3 not authorized.

use std::collections::BTreeMap;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR, RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::dataset_partition::PartitionKind;
use chronosentiment_adapter::decision_support::decision_value_harness::{
    action_values, measure_harness, search_admissible_protocol_value, ProtocolValue,
    C3_AUTHORIZED, ROWS_PER_SYMBOL_PER_SLICE,
};
use chronosentiment_adapter::decision_support::decision_value_landscape::landscape_row;
use chronosentiment_adapter::decision_support::observation_value::score_genome;
use chronosentiment_adapter::decision_support::policy_artifact::first_match_action;
use chronosentiment_adapter::decision_support::recommendation_outcome::{
    DirectionalCall, RecommendationRow,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::decision_support::observation_value::{ObservationRow, ObservationSlice};
use chrono::{TimeZone, Utc};
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use chronosentiment_adapter::metrics::concepts::Concept;
use uuid::Uuid;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn rec(
    instrument: &str,
    partition: PartitionKind,
    recommendation: DecisionAction,
    raw: f64,
) -> RecommendationRow {
    RecommendationRow {
        timestamp: "2021-10-31T15:30:00Z".to_string(),
        instrument: instrument.to_string(),
        partition,
        trend_state: "Bearish".to_string(),
        momentum_state: "present".to_string(),
        volatility_state: "present".to_string(),
        recommendation,
        actual_forward_return: Some(raw),
        return_contribution: None,
        directional_call: DirectionalCall::NotApplicable,
        long_alternative_return: Some(raw),
        short_alternative_return: Some(-raw),
        no_trade_winning_alternative: None,
        horizon_days: 20,
        policy_artifact_hash: RESEARCH_DISCOVERY_ARTIFACT_HASH.to_string(),
    }
}

#[test]
fn continuous_value_and_action_symmetry() {
    for raw in [0.001, 0.0873, -0.001, -0.0873, 0.0, 0.0001, -0.0001, 0.20, -0.30] {
        let (long, short, no_trade) = action_values(raw);
        assert_eq!(long, raw);
        assert_eq!(short, -raw);
        assert_eq!(no_trade, 0.0);
    }
}

#[test]
fn borderline_magnitudes_are_not_thresholded() {
    for raw in [0.0001, -0.0001, 0.001, -0.001] {
        let row = landscape_row(&rec("HDFCBANK.NS", PartitionKind::Development, DecisionAction::Long, raw))
            .unwrap();
        assert!((row.recommended_value - raw).abs() < 1e-15);
        assert_ne!(row.recommended_value, 0.0);
    }
}

#[test]
fn no_trade_enters_the_instrument_mean() {
    let mut map = BTreeMap::new();
    for (i, ticker) in RESEARCH_UNIVERSE.iter().enumerate() {
        if i == 0 {
            map.insert((*ticker).to_string(), vec![0.10, 0.0]);
        } else {
            map.insert((*ticker).to_string(), vec![0.0]);
        }
    }
    let protocol = ProtocolValue::from_per_instrument_v(&map).unwrap();
    let first = protocol.instrument_means[RESEARCH_UNIVERSE[0]];
    assert!((first - 0.05).abs() < 1e-15, "NO_TRADE=0 must dilute the instrument mean");
}

#[test]
fn protocol_value_is_mean_of_instrument_means_not_pooled_rows() {
    let mut map = BTreeMap::new();
    for (i, ticker) in RESEARCH_UNIVERSE.iter().enumerate() {
        if i == 0 {
            map.insert((*ticker).to_string(), vec![0.10]);
        } else if i == 1 {
            map.insert((*ticker).to_string(), vec![0.01; 9]);
        } else {
            map.insert((*ticker).to_string(), vec![0.0]);
        }
    }
    let protocol = ProtocolValue::from_per_instrument_v(&map).unwrap();
    let expected = (0.10 + 0.01 + 0.0 + 0.0 + 0.0 + 0.0 + 0.0) / 7.0;
    let pooled = (0.10 + 0.09) / 11.0;
    assert!((protocol.value - expected).abs() < 1e-15);
    assert!((protocol.value - pooled).abs() > 1e-6);
}

#[test]
fn evaluation_cannot_produce_search_admissible_protocol_value() {
    let recs: Vec<RecommendationRow> = RESEARCH_UNIVERSE
        .iter()
        .map(|t| rec(t, PartitionKind::Evaluation, DecisionAction::Long, 0.01))
        .collect();
    let rows: Vec<_> = recs.iter().filter_map(landscape_row).collect();
    let err = search_admissible_protocol_value(&rows, PartitionKind::Evaluation).unwrap_err();
    assert!(err.contains("evaluation cannot influence"));
}

#[test]
fn score_genome_still_rejects_evaluation() {
    let slice = ObservationSlice {
        kind: PartitionKind::Evaluation,
        rows: vec![],
    };
    let genome = chronosentiment_adapter::decision_support::policy_genome::RuleListGenome {
        rules: vec![],
        unmatched_action: DecisionAction::NoTrade,
    };
    assert!(score_genome(&genome, &slice).is_err());
}

#[test]
fn first_match_action_does_not_see_forward_returns() {
    let src = include_str!("../src/decision_support/policy_artifact.rs");
    let start = src.find("pub fn first_match_action").expect("first_match_action");
    let body = &src[start..start + 400];
    assert!(!body.contains("instrument_return"));
    assert!(!body.contains("forward"));
    assert!(!body.contains("actual_forward_return"));
}

#[test]
fn policy_action_is_independent_of_whether_outcome_is_present() {
    let t = Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap();
    let mut metrics = MetricReport::default();
    metrics.metrics.insert("ma_20".into(), MetricValue::Float(90.0));
    metrics.metrics.insert("ma_50".into(), MetricValue::Float(100.0));
    metrics.metrics.insert("roc_20".into(), MetricValue::Float(0.01));
    metrics.metrics.insert("atr_14".into(), MetricValue::Float(1.0));
    let profile = AssessmentEngine.assess_at(
        &metrics,
        &[Concept::Trend, Concept::Momentum, Concept::Volatility],
        t,
        Some(Uuid::from_u128(11)),
    );
    let with_outcome = ObservationRow {
        instrument: "HDFCBANK.NS".into(),
        as_of: t,
        profile: profile.clone(),
        instrument_return: Some(0.0873),
    };
    let without_outcome = ObservationRow {
        instrument: "HDFCBANK.NS".into(),
        as_of: t,
        profile,
        instrument_return: None,
    };
    let a = first_match_action(&[], DecisionAction::NoTrade, &with_outcome.profile);
    let b = first_match_action(&[], DecisionAction::NoTrade, &without_outcome.profile);
    assert_eq!(a, b);
    assert_eq!(a, DecisionAction::NoTrade);
}

#[test]
fn regret_and_unique_best_cannot_construct_protocol_value() {
    let harness = include_str!("../src/decision_support/decision_value_harness.rs");
    assert!(!harness.contains("impl From<DecisionDiagnostics>"));
    assert!(!harness.contains("from_regret"));
    assert!(!harness.contains("from_unique_best"));
    assert!(harness.contains("from_per_instrument_v"));
    assert!(!C3_AUTHORIZED);
}

#[test]
fn analysis_and_binary_do_not_evolve_or_authorize_c3() {
    let files = [
        include_str!("../src/decision_support/decision_value_harness.rs"),
        include_str!("../src/bin/csp006_decision_value_harness.rs"),
    ];
    for src in files {
        assert!(!src.contains("evolve_on_development"));
        assert!(!src.contains("train_policy"));
        assert!(!src.contains("CoralysPhase"));
        assert!(!src.contains("b5_strategy"));
    }
    let bin = include_str!("../src/bin/csp006_decision_value_harness.rs");
    assert!(bin.contains("RESEARCH_DISCOVERY_ARTIFACT_HASH"));
    assert!(bin.contains("c3_authorized=false"));
    assert!(bin.contains("refusing to overwrite Search #1"));
}

#[test]
fn document_requires_symbol_matrices_and_does_not_authorize_c3() {
    let doc = include_str!("../../../docs/CS-P-006-N_DECISION_VALUE_RESEARCH_HARNESS.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("Table A"));
    assert!(doc.contains("Table B"));
    assert!(doc.contains("C.3 is not authorized") || doc.contains("C.3 not authorized"));
    assert!(doc.contains("13 Development"));
}

#[test]
fn certified_symbol_matrices_when_recommendations_present() {
    let path = workspace_root()
        .join(RESEARCH_DISCOVERY_DIR)
        .join("recommendations")
        .join("recommendations.json");
    if !path.exists() {
        return;
    }
    let recs: Vec<RecommendationRow> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let (rows, report) = measure_harness(RESEARCH_DISCOVERY_ARTIFACT_HASH, &recs).unwrap();
    assert_eq!(rows.len(), 273);
    assert_eq!(report.table_a_decision_distribution.len(), 28);
    assert_eq!(report.table_b_decision_value.len(), 28);
    assert!(!report.c3_authorized);
    assert!(!report.evaluation.search_admissible);
    assert!(!report.all.search_admissible);
    assert!(report.development.search_admissible);
    assert!(report.selection.search_admissible);
    for ticker in RESEARCH_UNIVERSE {
        for slice in ["development", "selection", "evaluation"] {
            let a = report
                .table_a_decision_distribution
                .iter()
                .find(|r| r.instrument == *ticker && r.slice == slice)
                .unwrap();
            assert_eq!(a.total, ROWS_PER_SYMBOL_PER_SLICE);
            assert_eq!(a.n_long + a.n_short + a.n_no_trade, a.total);
        }
    }
    assert_eq!(
        measure_harness("not-search-one", &recs).unwrap_err(),
        "harness identity-gates Search #1; refusing a different artifact"
    );
}
