use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::backtest::{DecisionLedger, LedgerRecord};
use chronosentiment_adapter::decision_support::outcome::{
    DecisionOutcomeBundle, HorizonOutcome, OutcomeReport, HORIZON_DAYS,
};
use chronosentiment_adapter::decision_support::performance::measure_performance;
use chronosentiment_adapter::decision_support::replay::UNFROZEN_ENGINE_VERSION;
use chronosentiment_adapter::decision_support::{DecisionAction, DecisionLineage};
use uuid::Uuid;

fn t(day: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, day, 15, 30, 0).unwrap()
}

fn rec(seq: u32, id: u128, day: u32, action: DecisionAction) -> LedgerRecord {
    LedgerRecord {
        sequence: seq,
        decision_id: Uuid::from_u128(id),
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        policy_name: "baseline.trend_mapping.v0".to_string(),
        instrument_id: Uuid::from_u128(3),
        as_of_timestamp: t(day),
        decision_timestamp: t(day),
        action,
        confidence: None,
        confidence_status: Default::default(),
        input_set_hash: "abc".to_string(),
        lineage: DecisionLineage {
            produced_by: "test".to_string(),
            consumed_artifact_ids: vec![Uuid::from_u128(1)],
            assessment_id: Some(Uuid::from_u128(1)),
            input_set_hash: "abc".to_string(),
        },
        content_hash: format!("dec{id}"),
        evidence: Default::default(),
    }
}

fn horizons(r5: Option<f64>) -> Vec<HorizonOutcome> {
    HORIZON_DAYS
        .iter()
        .map(|days| {
            let r = if *days == 5 { r5 } else { None };
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
                horizon_expiry_timestamp: r.map(|_| t(31) + chrono::Duration::days(*days as i64)),
            }
        })
        .collect()
}

fn bundle(id: u128, action: DecisionAction, r5: Option<f64>) -> DecisionOutcomeBundle {
    DecisionOutcomeBundle {
        ledger_decision_id: Uuid::from_u128(id),
        instrument_id: Uuid::from_u128(3),
        as_of_timestamp: t(31),
        action,
        horizons: horizons(r5),
        content_hash: format!("out{id}"),
    }
}

fn fixture() -> (DecisionLedger, OutcomeReport) {
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.records = vec![
        rec(1, 1, 31, DecisionAction::Long),
        rec(2, 2, 31, DecisionAction::NoTrade),
        rec(3, 3, 31, DecisionAction::Short),
    ];
    let outcomes = OutcomeReport {
        bundles: vec![
            bundle(1, DecisionAction::Long, Some(0.25)),
            bundle(2, DecisionAction::NoTrade, Some(0.5)),
            bundle(3, DecisionAction::Short, Some(-0.125)),
        ],
    };
    (ledger, outcomes)
}

#[test]
fn no_trade_is_not_a_zero_return_trade() {
    let (ledger, outcomes) = fixture();
    let report = measure_performance(&ledger, &outcomes);
    let h5 = &report.horizons[0];
    assert_eq!(h5.horizon_days, 5);
    assert_eq!(h5.trading.returns.n_observed, 2);
    assert_eq!(h5.trading.returns.cumulative_return, Some(0.125));
    assert_eq!(h5.trading.returns.mean, Some(0.0625));
    assert_ne!(h5.trading.returns.mean, Some(0.25 / 3.0));
    assert_ne!(h5.trading.returns.mean, Some((0.25 + 0.5 - 0.125) / 3.0));
    assert_eq!(h5.opportunity.returns.n_observed, 1);
    assert_eq!(h5.opportunity.returns.mean, Some(0.5));
    assert_eq!(h5.opportunity.returns.cumulative_return, Some(0.5));
}

#[test]
fn four_horizons_always_present_none_selected() {
    let (ledger, outcomes) = fixture();
    let report = measure_performance(&ledger, &outcomes);
    assert_eq!(
        report
            .horizons
            .iter()
            .map(|h| h.horizon_days)
            .collect::<Vec<_>>(),
        HORIZON_DAYS.to_vec()
    );
    assert!(report.horizons[1].trading.returns.mean.is_none());
    assert!(report.horizons[1].opportunity.returns.mean.is_none());
}

#[test]
fn behavior_counts_come_from_the_ledger() {
    let (ledger, outcomes) = fixture();
    let report = measure_performance(&ledger, &outcomes);
    assert_eq!(report.behavior.n_records, 3);
    assert_eq!(report.behavior.counts.long, 1);
    assert_eq!(report.behavior.counts.short, 1);
    assert_eq!(report.behavior.counts.no_trade, 1);
    assert_eq!(report.horizons[0].by_action.long.mean, Some(0.25));
    assert_eq!(report.horizons[0].by_action.short.mean, Some(-0.125));
    assert_eq!(report.horizons[0].by_action.no_trade.mean, Some(0.5));
}

#[test]
fn repeated_measurement_is_deterministic_and_does_not_mutate_inputs() {
    let (ledger, outcomes) = fixture();
    let ledger_hash = ledger.identity_hash();
    let outcome_hash = outcomes.identity_hash();
    let a = measure_performance(&ledger, &outcomes);
    let b = measure_performance(&ledger, &outcomes);
    assert_eq!(a.content_hash, b.content_hash);
    assert_eq!(a.ledger_identity_hash, ledger_hash);
    assert_eq!(a.outcome_identity_hash, outcome_hash);
    assert_eq!(ledger.identity_hash(), ledger_hash);
    assert_eq!(outcomes.identity_hash(), outcome_hash);
    assert_eq!(ledger.records.len(), 3);
}

#[test]
fn missing_horizon_is_unavailable_not_invented() {
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.records = vec![rec(1, 1, 31, DecisionAction::Long)];
    let outcomes = OutcomeReport {
        bundles: vec![bundle(1, DecisionAction::Long, None)],
    };
    let report = measure_performance(&ledger, &outcomes);
    assert_eq!(report.horizons[0].trading.returns.n_decisions, 1);
    assert_eq!(report.horizons[0].trading.returns.n_observed, 0);
    assert_eq!(report.horizons[0].trading.returns.n_unavailable, 1);
    assert!(report.horizons[0]
        .trading
        .returns
        .cumulative_return
        .is_none());
}

#[test]
fn win_rate_and_drawdown_use_trading_path_only() {
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.records = vec![
        rec(1, 1, 29, DecisionAction::Long),
        rec(2, 2, 30, DecisionAction::NoTrade),
        rec(3, 3, 31, DecisionAction::Long),
    ];
    let outcomes = OutcomeReport {
        bundles: vec![
            bundle(1, DecisionAction::Long, Some(0.25)),
            bundle(2, DecisionAction::NoTrade, Some(-0.5)),
            bundle(3, DecisionAction::Long, Some(-0.125)),
        ],
    };
    let report = measure_performance(&ledger, &outcomes);
    let t5 = &report.horizons[0].trading;
    assert_eq!(t5.returns.n_win, 1);
    assert_eq!(t5.returns.n_loss, 1);
    assert_eq!(t5.returns.win_rate, Some(0.5));
    assert_eq!(t5.risk.max_drawdown, Some(0.125));
    assert_eq!(t5.risk.worst_outcome, Some(-0.125));
    assert_eq!(
        report.horizons[0].opportunity.risk.worst_outcome,
        Some(-0.5)
    );
}
