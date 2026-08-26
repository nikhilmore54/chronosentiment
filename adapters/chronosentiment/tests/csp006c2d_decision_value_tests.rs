//! CS-P-006-C.2-D — decision-value landscape. Not a new search.

use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_ARTIFACT_HASH, RESEARCH_DISCOVERY_DIR,
};
use chronosentiment_adapter::decision_support::dataset_partition::PartitionKind;
use chronosentiment_adapter::decision_support::decision_value_landscape::{
    action_value, analyze_landscape, best_action, landscape_row,
};
use chronosentiment_adapter::decision_support::recommendation_outcome::{
    DirectionalCall, RecommendationRow,
};
use chronosentiment_adapter::decision_support::DecisionAction;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn rec_row(
    partition: PartitionKind,
    recommendation: DecisionAction,
    raw: f64,
) -> RecommendationRow {
    RecommendationRow {
        timestamp: "2021-10-31T15:30:00Z".to_string(),
        instrument: "HDFCBANK.NS".to_string(),
        partition,
        trend_state: "Bearish".to_string(),
        momentum_state: "present".to_string(),
        volatility_state: "present".to_string(),
        recommendation,
        actual_forward_return: Some(raw),
        return_contribution: Some(if recommendation == DecisionAction::NoTrade {
            0.0
        } else if recommendation == DecisionAction::Short {
            -raw
        } else {
            raw
        }),
        directional_call: DirectionalCall::NotApplicable,
        long_alternative_return: Some(raw),
        short_alternative_return: Some(-raw),
        no_trade_winning_alternative: if raw > 0.0 {
            Some(DecisionAction::Long)
        } else if raw < 0.0 {
            Some(DecisionAction::Short)
        } else {
            None
        },
        horizon_days: 20,
        policy_artifact_hash: "test-sealed".to_string(),
    }
}

#[test]
fn contract_preserves_magnitude_without_a_band() {
    assert_eq!(action_value(DecisionAction::Long, 0.002), 0.002);
    assert_eq!(action_value(DecisionAction::Short, 0.002), -0.002);
    assert_eq!(action_value(DecisionAction::NoTrade, 0.002), 0.0);
    assert_eq!(action_value(DecisionAction::Long, -0.001), -0.001);
    assert_eq!(action_value(DecisionAction::Short, -0.001), 0.001);
    assert_eq!(best_action(0.12), DecisionAction::Long);
    assert_eq!(best_action(-0.30), DecisionAction::Short);
    assert_eq!(best_action(0.0), DecisionAction::NoTrade);

    let gain = landscape_row(&rec_row(
        PartitionKind::Development,
        DecisionAction::Long,
        0.002,
    ))
    .unwrap();
    assert!((gain.recommended_value - 0.002).abs() < 1e-15);
    assert!((gain.regret - 0.0).abs() < 1e-15);
    assert!(gain.recommended_is_unique_best);

    let loss = landscape_row(&rec_row(
        PartitionKind::Evaluation,
        DecisionAction::Long,
        -0.001,
    ))
    .unwrap();
    assert!((loss.recommended_value + 0.001).abs() < 1e-15);
    assert!((loss.regret - 0.002).abs() < 1e-15);
    assert_eq!(loss.best_action, DecisionAction::Short);
    assert!(!loss.recommended_is_unique_best);
}

#[test]
fn no_trade_is_never_uniquely_best_and_never_called_correct() {
    for raw in [-0.05, 0.05, 0.0] {
        let row = landscape_row(&rec_row(
            PartitionKind::Selection,
            DecisionAction::NoTrade,
            raw,
        ))
        .unwrap();
        assert_eq!(row.recommended_value, 0.0);
        assert!((row.regret - raw.abs()).abs() < 1e-15);
        assert!(!row.recommended_is_unique_best);
    }
}

#[test]
fn analysis_and_binary_do_not_search_or_invent_a_cutoff() {
    let files = [
        include_str!("../src/decision_support/decision_value_landscape.rs"),
        include_str!("../src/bin/csp006_decision_value.rs"),
    ];
    for src in files {
        assert!(!src.contains("evolve_on_development"));
        assert!(!src.contains("train_policy"));
        assert!(!src.contains("test_fitness"));
        assert!(!src.contains("CoralysPhase"));
        assert!(!src.contains("b5_strategy"));
        assert!(!src.contains("BORDERLINE"));
        assert!(!src.contains("borderline_x"));
        assert!(!src.contains("TRANSACTION_COST"));
        assert!(!src.contains("evolve_on_development_observed"));
    }
    let bin = include_str!("../src/bin/csp006_decision_value.rs");
    assert!(bin.contains("RESEARCH_DISCOVERY_ARTIFACT_HASH"));
    assert!(bin.contains("analyze_landscape"));
    assert!(bin.contains("used_as_coralys_fitness=false"));
}

#[test]
fn document_does_not_authorize_search_two_or_freeze_a_band() {
    let doc = include_str!("../../../docs/CS-P-006-C.2-D_DECISION_VALUE_LANDSCAPE.md");
    assert!(doc.contains(RESEARCH_DISCOVERY_ARTIFACT_HASH));
    assert!(doc.contains("not authorized") || doc.contains("Not Search #2"));
    assert!(doc.contains("No `±X%` borderline classifier is frozen"));
    assert!(doc.contains("NO_TRADE is **never** uniquely best"));
    assert!(doc.contains("They are **not** Coralys fitness"));
    assert!(doc.contains("R1. Can the state at T identify an action"));
    assert!(doc.contains("must **not** become another accuracy metric"));
    assert!(doc.contains("unique_best = true  → fitness 1"));
}

#[test]
fn certified_landscape_from_existing_recommendations() {
    let rec_path = workspace_root()
        .join(RESEARCH_DISCOVERY_DIR)
        .join("recommendations")
        .join("recommendations.json");
    if !rec_path.exists() {
        return;
    }
    let recommendations: Vec<RecommendationRow> =
        serde_json::from_str(&std::fs::read_to_string(rec_path).unwrap()).unwrap();
    let (rows, card) =
        analyze_landscape(RESEARCH_DISCOVERY_ARTIFACT_HASH, &recommendations).unwrap();
    assert_eq!(rows.len(), 273);
    assert_eq!(card.n_rows, 273);
    assert_eq!(card.development.n_acted, 49);
    assert_eq!(card.selection.n_acted, 39);
    assert_eq!(card.evaluation.n_acted, 33);
    assert_eq!(card.overall.n_stood_aside, 152);
    assert!(!card.search_two_authorized);
    assert!(!card.coralys_feedback);
    assert!(!card.borderline_band_frozen);
    assert!(!card.used_as_coralys_fitness);
    assert!(!card.cost_term_present);
    assert!(rows.iter().all(|r| r.regret >= -1e-15));
    assert!(rows
        .iter()
        .filter(|r| r.recommendation == DecisionAction::NoTrade)
        .all(|r| !r.recommended_is_unique_best && r.recommended_value == 0.0));
}
