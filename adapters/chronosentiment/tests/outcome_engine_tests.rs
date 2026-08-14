use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::backtest::{DecisionLedger, LedgerRecord};
use chronosentiment_adapter::decision_support::outcome::{
    measure_ledger, measure_record, LakeOutcomeRow, HORIZON_DAYS,
};
use chronosentiment_adapter::decision_support::replay::UNFROZEN_ENGINE_VERSION;
use chronosentiment_adapter::decision_support::{DecisionAction, DecisionLineage};
use uuid::Uuid;

fn t0() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
}

fn record() -> LedgerRecord {
    LedgerRecord {
        sequence: 1,
        decision_id: Uuid::from_u128(7),
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        instrument_id: Uuid::from_u128(3),
        as_of_timestamp: t0(),
        decision_timestamp: t0(),
        action: DecisionAction::Long,
        confidence: None,
        confidence_status: Default::default(),
        input_set_hash: "abc".to_string(),
        lineage: DecisionLineage {
            produced_by: "test".to_string(),
            consumed_artifact_ids: vec![Uuid::from_u128(1)],
            assessment_id: Some(Uuid::from_u128(1)),
            input_set_hash: "abc".to_string(),
        },
        content_hash: "dec".to_string(),
        evidence: Default::default(),
    }
}

fn row(horizon: &str, decision_as_of: chrono::DateTime<Utc>, id: u128) -> LakeOutcomeRow {
    LakeOutcomeRow {
        id: Uuid::from_u128(id),
        lake_decision_id: Uuid::from_u128(50),
        instrument_id: Uuid::from_u128(3),
        decision_as_of,
        horizon: horizon.to_string(),
        outcome_return: 0.01,
        entry_reached: true,
        target_hit: false,
        stop_hit: false,
        exit_reason: "Expired".to_string(),
        mfe: 0.02,
        mae: -0.01,
        drawdown: 0.01,
        horizon_expiry_timestamp: decision_as_of
            + chrono::Duration::days(horizon.trim_end_matches('D').parse::<i64>().unwrap()),
    }
}

#[test]
fn four_horizons_and_lineage_to_ledger_decision() {
    let rec = record();
    let rows = vec![
        row("5D", t0(), 10),
        row("10D", t0(), 11),
        row("20D", t0(), 12),
        row("60D", t0(), 13),
    ];
    let bundle = measure_record(&rec, &rows);
    assert_eq!(bundle.ledger_decision_id, rec.decision_id);
    assert_eq!(bundle.as_of_timestamp, rec.as_of_timestamp);
    assert_eq!(bundle.action, rec.action);
    assert_eq!(
        bundle
            .horizons
            .iter()
            .map(|h| h.horizon_days)
            .collect::<Vec<_>>(),
        HORIZON_DAYS.to_vec()
    );
    assert!(bundle.horizons.iter().all(|h| h.available));
    assert_eq!(bundle.horizons[0].lake_outcome_id, Some(Uuid::from_u128(10)));
}

#[test]
fn future_parent_decision_is_excluded() {
    let rec = record();
    let mut rows = vec![row("5D", t0(), 10)];
    let future = t0() + chrono::Duration::days(365);
    rows.push(row("5D", future, 99));
    let bundle = measure_record(&rec, &rows);
    assert_eq!(bundle.horizons[0].lake_outcome_id, Some(Uuid::from_u128(10)));
    assert!(!bundle
        .horizons
        .iter()
        .any(|h| h.lake_outcome_id == Some(Uuid::from_u128(99))));
}

#[test]
fn repeated_measurement_is_deterministic_and_does_not_mutate_ledger() {
    let rec = record();
    let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
    ledger.records.push(rec.clone());
    let rows = vec![row("5D", t0(), 10)];
    let a = measure_ledger(&ledger, &rows);
    let b = measure_ledger(&ledger, &rows);
    assert_eq!(a.identity_hash(), b.identity_hash());
    assert_eq!(ledger.records[0].content_hash, "dec");
    assert_eq!(ledger.records.len(), 1);
}

#[test]
fn missing_horizon_is_unavailable_not_invented() {
    let rec = record();
    let bundle = measure_record(&rec, &[row("5D", t0(), 10)]);
    assert!(bundle.horizons[0].available);
    assert!(!bundle.horizons[1].available);
    assert!(bundle.horizons[1].outcome_return.is_none());
}

#[test]
fn expiry_not_after_as_of_is_excluded() {
    let rec = record();
    let mut expired = row("5D", t0(), 10);
    expired.horizon_expiry_timestamp = t0();
    let bundle = measure_record(&rec, &[expired]);
    assert!(!bundle.horizons[0].available);
    assert!(bundle.horizons[0].outcome_return.is_none());
}

#[test]
fn no_trade_still_receives_lake_outcomes() {
    let mut rec = record();
    rec.action = DecisionAction::NoTrade;
    let bundle = measure_record(&rec, &[row("5D", t0(), 10)]);
    assert_eq!(bundle.action, DecisionAction::NoTrade);
    assert!(bundle.horizons[0].available);
    assert_eq!(bundle.horizons[0].outcome_return, Some(0.01));
}
