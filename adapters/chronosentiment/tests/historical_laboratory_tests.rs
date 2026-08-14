use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::backtest::{DecisionLedger, LedgerRecord};
use chronosentiment_adapter::decision_support::lab_context::context_from_profile;
use chronosentiment_adapter::decision_support::laboratory::{
    calendar_year_folds, run_laboratory, DecisionContext, LaboratoryInput,
};
use chronosentiment_adapter::decision_support::outcome::{
    DecisionOutcomeBundle, HorizonOutcome, OutcomeReport, HORIZON_DAYS,
};
use chronosentiment_adapter::decision_support::replay::UNFROZEN_ENGINE_VERSION;
use chronosentiment_adapter::decision_support::{DecisionAction, DecisionLineage};
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::{
    AssessmentProfile, Direction, DomainAssessment, Strength,
};
use chronosentiment_adapter::repository::knowledge::ArtifactMetadata;
use uuid::Uuid;

fn t(year: i32, month: u32, day: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, 15, 30, 0).unwrap()
}

fn rec(
    seq: u32,
    id: u128,
    instrument: u128,
    as_of: chrono::DateTime<Utc>,
    action: DecisionAction,
) -> LedgerRecord {
    LedgerRecord {
        sequence: seq,
        decision_id: Uuid::from_u128(id),
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        instrument_id: Uuid::from_u128(instrument),
        as_of_timestamp: as_of,
        decision_timestamp: as_of,
        action,
        confidence: None,
        confidence_status: Default::default(),
        input_set_hash: "abc".to_string(),
        lineage: DecisionLineage {
            produced_by: "csp002.replay_adapter".to_string(),
            consumed_artifact_ids: vec![Uuid::from_u128(1)],
            assessment_id: Some(Uuid::from_u128(1)),
            input_set_hash: "abc".to_string(),
        },
        content_hash: format!("dec{id}"),
        evidence: Default::default(),
    }
}

fn horizons(r60: Option<f64>) -> Vec<HorizonOutcome> {
    HORIZON_DAYS
        .iter()
        .map(|days| {
            let r = if *days == 60 { r60 } else { None };
            HorizonOutcome {
                horizon_days: *days,
                available: r.is_some(),
                lake_outcome_id: r.map(|_| Uuid::from_u128(*days as u128)),
                lake_decision_id: r.map(|_| Uuid::from_u128(50)),
                outcome_return: r,
                entry_reached: r.map(|_| true),
                target_hit: Some(false),
                stop_hit: Some(false),
                exit_reason: r.map(|_| "Expired".to_string()),
                mfe: None,
                mae: None,
                drawdown: None,
                horizon_expiry_timestamp: r.map(|_| t(2024, 12, 31)),
            }
        })
        .collect()
}

fn bundle(
    id: u128,
    instrument: u128,
    as_of: chrono::DateTime<Utc>,
    action: DecisionAction,
    r60: Option<f64>,
) -> DecisionOutcomeBundle {
    DecisionOutcomeBundle {
        ledger_decision_id: Uuid::from_u128(id),
        instrument_id: Uuid::from_u128(instrument),
        as_of_timestamp: as_of,
        action,
        horizons: horizons(r60),
        content_hash: format!("out{id}"),
    }
}

fn ctx(id: u128, label: &str, trend: &str) -> DecisionContext {
    DecisionContext {
        decision_id: Uuid::from_u128(id),
        instrument_label: label.to_string(),
        trend: Some(trend.to_string()),
        trend_strength: Some("Strong".to_string()),
        momentum: Some("Positive".to_string()),
        momentum_strength: Some("Moderate".to_string()),
        volatility: None,
        confidence_status: Some("Unavailable".to_string()),
        mapping_rule: None,
    }
}

fn fixture() -> (DecisionLedger, OutcomeReport, Vec<DecisionContext>) {
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.records = vec![
        rec(1, 1, 10, t(2021, 10, 31), DecisionAction::Long),
        rec(2, 2, 10, t(2022, 6, 30), DecisionAction::Long),
        rec(3, 3, 10, t(2023, 6, 30), DecisionAction::Short),
        rec(4, 4, 11, t(2023, 6, 30), DecisionAction::Long),
        rec(5, 5, 10, t(2024, 6, 30), DecisionAction::Short),
    ];
    let outcomes = OutcomeReport {
        bundles: vec![
            bundle(1, 10, t(2021, 10, 31), DecisionAction::Long, Some(0.10)),
            bundle(2, 10, t(2022, 6, 30), DecisionAction::Long, Some(-0.05)),
            bundle(3, 10, t(2023, 6, 30), DecisionAction::Short, None),
            bundle(4, 11, t(2023, 6, 30), DecisionAction::Long, Some(0.02)),
            bundle(5, 10, t(2024, 6, 30), DecisionAction::Short, None),
        ],
    };
    let context = vec![
        ctx(1, "AAA", "Bullish"),
        ctx(2, "AAA", "Bullish"),
        ctx(3, "AAA", "Bearish"),
        ctx(4, "BBB", "Bullish"),
        ctx(5, "AAA", "Bearish"),
    ];
    (ledger, outcomes, context)
}

#[test]
fn same_inputs_same_laboratory_hash() {
    let (ledger, outcomes, context) = fixture();
    let a = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    let b = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    assert_eq!(a.content_hash, b.content_hash);
    assert_eq!(a.decision_engine_version, UNFROZEN_ENGINE_VERSION);
}

#[test]
fn walk_forward_test_is_after_train_end() {
    let (ledger, outcomes, context) = fixture();
    let report = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    assert!(!report.walk_forward.is_empty());
    for fold in &report.walk_forward {
        assert_eq!(fold.test_start, fold.train_end);
        assert!(fold.test_end > fold.test_start);
        let train_times: Vec<_> = ledger
            .records
            .iter()
            .filter(|r| r.as_of_timestamp < fold.train_end)
            .map(|r| r.as_of_timestamp)
            .collect();
        let test_times: Vec<_> = ledger
            .records
            .iter()
            .filter(|r| r.as_of_timestamp >= fold.test_start && r.as_of_timestamp < fold.test_end)
            .map(|r| r.as_of_timestamp)
            .collect();
        for ts in &test_times {
            assert!(*ts >= fold.train_end);
            assert!(!train_times.contains(ts));
        }
    }
}

#[test]
fn calendar_folds_cover_2022_2024_from_late_2021() {
    let folds = calendar_year_folds(t(2021, 10, 31), t(2024, 12, 31));
    let names: Vec<_> = folds.iter().map(|f| f.0.as_str()).collect();
    assert_eq!(names, vec!["test_2022", "test_2023", "test_2024"]);
}

#[test]
fn short_missing_outcomes_stay_unavailable() {
    let (ledger, outcomes, context) = fixture();
    let report = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    assert_eq!(report.coverage.n_short, 2);
    assert_eq!(report.coverage.n_short_with_outcome, 0);
    assert!(report.coverage.short_unevaluated);
    let short_base = report
        .vs_baseline
        .iter()
        .find(|r| r.action == DecisionAction::Short && r.horizon_days == 60)
        .unwrap();
    assert_eq!(short_base.n_observed, 0);
    assert!(short_base.mean.is_none());
}

#[test]
fn no_trade_zero_count_is_unevaluable_not_zero_return() {
    let (ledger, outcomes, context) = fixture();
    let report = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    assert_eq!(report.behavior.counts.no_trade, 0);
    assert!(report.coverage.no_trade_unevaluable);
    let nt = report
        .vs_baseline
        .iter()
        .find(|r| r.action == DecisionAction::NoTrade && r.horizon_days == 60)
        .unwrap();
    assert_eq!(nt.n_decisions, 0);
    assert!(nt.mean.is_none());
}

#[test]
fn long_bullish_slice_is_not_the_aggregate() {
    let (ledger, outcomes, context) = fixture();
    let report = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    let long_bull = report
        .stratification
        .iter()
        .find(|s| s.dimension == "action+trend" && s.value == "LONG+Bullish")
        .unwrap();
    assert_eq!(long_bull.n_records, 3);
    let h60 = long_bull
        .performance
        .horizons
        .iter()
        .find(|h| h.horizon_days == 60)
        .unwrap();
    assert_eq!(h60.trading.returns.n_observed, 3);
    assert_eq!(
        h60.trading.returns.mean,
        Some((0.10 - 0.05 + 0.02) / 3.0)
    );
}

#[test]
fn transitions_and_streaks_are_per_instrument() {
    let (ledger, outcomes, context) = fixture();
    let report = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    assert_eq!(report.behavior.transitions.long_to_long, 1);
    assert_eq!(report.behavior.transitions.long_to_short, 1);
    assert_eq!(report.behavior.transitions.short_to_short, 1);
    assert!(report.behavior.streak_lengths.contains_key(&1));
}

#[test]
fn context_from_profile_does_not_invent_volatility() {
    let mut metadata = ArtifactMetadata::mock();
    metadata.evaluation_timestamp = t(2022, 1, 1);
    let profile = AssessmentProfile {
        metadata,
        instrument_id: Some(Uuid::from_u128(10)),
        assessments: vec![DomainAssessment {
            concept: Concept::Trend,
            direction: Direction::Bullish,
            strength: Some(Strength::Strong),
            maturity: None,
            persistence: None,
            confidence: 0.82,
            uncertainty: 0.18,
            uncertainty_reason: None,
            supporting_metrics: vec!["ma_20".to_string()],
            contradicting_metrics: vec![],
        }],
        factor_status: vec![],
    };
    let row = context_from_profile(Uuid::from_u128(1), "AAA".to_string(), Some(&profile));
    assert_eq!(row.trend.as_deref(), Some("Bullish"));
    assert!(row.volatility.is_none());
}

#[test]
fn laboratory_does_not_change_engine_version() {
    let (ledger, outcomes, context) = fixture();
    let report = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    assert_eq!(report.decision_engine_version, ledger.engine_version);
    assert_eq!(report.ledger_identity_hash, ledger.identity_hash());
}
