use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::backtest::{DecisionLedger, LedgerRecord};
use chronosentiment_adapter::decision_support::forward::{
    decide_forward, ForwardJournal, FORWARD_PRODUCER,
};
use chronosentiment_adapter::decision_support::observation_outcome::{
    measure_ledger_from_prices, measure_record_from_prices, PriceBar,
};
use chronosentiment_adapter::decision_support::performance::measure_performance;
use chronosentiment_adapter::decision_support::policy::BaselineTrendMappingPolicy;
use chronosentiment_adapter::decision_support::replay::{
    ReplayAssessment, ReplayInputs, ReplayObservation, UNFROZEN_ENGINE_VERSION,
};
use chronosentiment_adapter::decision_support::{DecisionAction, DecisionLineage};
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use uuid::Uuid;

fn t(day: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, day, 15, 30, 0).unwrap()
}

fn rec(id: u128, day: u32, action: DecisionAction) -> LedgerRecord {
    LedgerRecord {
        sequence: 1,
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
            produced_by: FORWARD_PRODUCER.to_string(),
            consumed_artifact_ids: vec![Uuid::from_u128(1)],
            assessment_id: Some(Uuid::from_u128(1)),
            input_set_hash: "abc".to_string(),
        },
        content_hash: format!("dec{id}"),
        evidence: Default::default(),
    }
}

fn bar_at(ts: chrono::DateTime<Utc>, close: f64) -> PriceBar {
    PriceBar {
        effective_from: ts,
        close,
        instrument_id: None,
    }
}

fn bars() -> Vec<PriceBar> {
    vec![
        bar_at(t(20), 100.0),
        bar_at(t(25) + chrono::Duration::days(5), 110.0),
        bar_at(t(25) + chrono::Duration::days(10), 90.0),
        bar_at(t(25) + chrono::Duration::days(20), 120.0),
        bar_at(t(25) + chrono::Duration::days(60), 80.0),
        bar_at(t(25) + chrono::Duration::days(400), 999.0),
    ]
}

#[test]
fn long_and_short_are_both_measured_with_opposite_sign() {
    let now = t(25) + chrono::Duration::days(60);
    let long = measure_record_from_prices(&rec(1, 25, DecisionAction::Long), &bars(), now);
    let short = measure_record_from_prices(&rec(2, 25, DecisionAction::Short), &bars(), now);
    assert!(long.horizons.iter().all(|h| h.available));
    assert!(short.horizons.iter().all(|h| h.available));
    assert_eq!(long.horizons[0].outcome_return, Some(0.10));
    assert_eq!(short.horizons[0].outcome_return, Some(-0.10));
    assert_eq!(long.horizons[1].outcome_return, Some(-0.10));
    assert_eq!(short.horizons[1].outcome_return, Some(0.10));
    assert!(long.horizons.iter().all(|h| h.lake_outcome_id.is_none()));
}

#[test]
fn prices_after_now_are_excluded_and_unelapsed_horizons_are_unavailable() {
    let now = t(25) + chrono::Duration::days(6);
    let bundle = measure_record_from_prices(&rec(1, 25, DecisionAction::Long), &bars(), now);
    assert!(bundle.horizons[0].available);
    assert!(!bundle.horizons[1].available);
    assert!(!bundle.horizons[3].available);
    assert_ne!(
        bundle.horizons[0].outcome_return,
        Some((999.0 - 100.0) / 100.0)
    );
}

#[test]
fn no_trade_stores_unsigned_path_and_stays_out_of_trading_pnl() {
    let now = t(25) + chrono::Duration::days(5);
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.records = vec![
        rec(1, 25, DecisionAction::Long),
        rec(2, 25, DecisionAction::NoTrade),
    ];
    let outcomes = measure_ledger_from_prices(&ledger, &bars(), now);
    let report = measure_performance(&ledger, &outcomes);
    assert_eq!(report.horizons[0].trading.returns.n_observed, 1);
    assert_eq!(report.horizons[0].trading.returns.mean, Some(0.10));
    assert_eq!(report.horizons[0].opportunity.returns.mean, Some(0.10));
}

#[test]
fn measurement_does_not_mutate_ledger_and_is_deterministic() {
    let now = t(25) + chrono::Duration::days(5);
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.records = vec![rec(1, 25, DecisionAction::Short)];
    let hash = ledger.identity_hash();
    let a = measure_ledger_from_prices(&ledger, &bars(), now);
    let b = measure_ledger_from_prices(&ledger, &bars(), now);
    assert_eq!(a.identity_hash(), b.identity_hash());
    assert_eq!(ledger.identity_hash(), hash);
}

fn bullish_inputs(instrument_id: Uuid) -> ReplayInputs {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));
    let mut profile =
        AssessmentEngine.assess_at(&metrics, &[Concept::Trend], t(25), Some(instrument_id));
    let id = Uuid::from_u128(9);
    profile.metadata.artifact_id = id;
    ReplayInputs {
        instrument_id,
        as_of: t(25),
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        produced_by: FORWARD_PRODUCER.to_string(),
        assessments: vec![ReplayAssessment {
            id,
            evaluation_timestamp: t(25),
            signature_hash: profile.to_hash(),
            profile,
        }],
        lake_decisions: vec![],
        observations: vec![ReplayObservation {
            id: Uuid::from_u128(11),
            effective_from: t(25),
        }],
    }
}

#[test]
fn forward_decide_does_not_consume_future_observations() {
    let instrument_id = Uuid::from_u128(7);
    let mut inputs = bullish_inputs(instrument_id);
    let future_id = Uuid::from_u128(99);
    inputs.observations.push(ReplayObservation {
        id: future_id,
        effective_from: t(25) + chrono::Duration::days(1),
    });
    let d = decide_forward(inputs, &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(d.lineage.produced_by, FORWARD_PRODUCER);
    assert_eq!(d.engine_version, UNFROZEN_ENGINE_VERSION);
    assert!(!d.lineage.consumed_artifact_ids.contains(&future_id));
}

#[test]
fn journal_is_append_only_and_idempotent() {
    let dir = std::env::temp_dir().join(format!("csp003-{}", Uuid::from_u128(77)));
    let _ = std::fs::remove_dir_all(&dir);
    let journal = ForwardJournal::open(&dir).unwrap();
    let decision = decide_forward(
        bullish_inputs(Uuid::from_u128(7)),
        &BaselineTrendMappingPolicy,
    )
    .unwrap();
    let first = journal.persist(decision.clone()).unwrap();
    let second = journal.persist(decision).unwrap();
    assert_eq!(first.decision_id, second.decision_id);
    assert_eq!(journal.load_ledger().unwrap().records.len(), 1);
    assert_eq!(first.sequence, 1);
    let meta: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("session.json")).unwrap()).unwrap();
    assert_eq!(meta["broker"], false);
    let _ = std::fs::remove_dir_all(&dir);
}

fn synthetic_history(
    n: usize,
    last: chrono::DateTime<Utc>,
) -> Vec<chronosentiment_adapter::decision_support::forward_tick::DailyBar> {
    use chronosentiment_adapter::decision_support::forward_tick::DailyBar;
    (0..n)
        .map(|i| DailyBar {
            timestamp: last - chrono::Duration::days((n - 1 - i) as i64),
            close: 100.0 + i as f64,
        })
        .collect()
}

#[test]
fn tick_decides_only_the_latest_session_not_historical_replay() {
    use chronosentiment_adapter::decision_support::forward_tick::{
        decide_latest_session, latest_as_of,
    };
    let last = t(31);
    let mut bars = synthetic_history(60, last);
    bars.push(
        chronosentiment_adapter::decision_support::forward_tick::DailyBar {
            timestamp: last + chrono::Duration::days(1),
            close: 999.0,
        },
    );
    let now = last;
    let as_of = latest_as_of(&bars, now).unwrap();
    assert_eq!(as_of, last);
    let a = decide_latest_session("RELIANCE.NS", &bars, now, &BaselineTrendMappingPolicy).unwrap();
    let b = decide_latest_session("RELIANCE.NS", &bars, now, &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(a.as_of_timestamp, last);
    assert_eq!(a.decision_id, b.decision_id);
    assert_eq!(a.engine_version, UNFROZEN_ENGINE_VERSION);
    assert_ne!(a.as_of_timestamp, last + chrono::Duration::days(1));
}
