use super::*;
use super::super::deferred_live::{LivePaperLedger, MarketObservation, LivePaperEvent, LivePaperPosition};
use super::super::paper_replay::PaperPosition;
use super::super::paper_lifecycle::{PaperWalkState, HorizonPolicy, ExitReason, PaperObservation};
use std::fs;

fn mock_pos(id: &str, ticker: &str, entry: f64, target: f64, stop: f64, unix: i64) -> LivePaperPosition {
    LivePaperPosition {
        paper: PaperPosition {
            decision_id: Some(id.to_string()),
            ticker: ticker.to_string(),
            date: "2026-09-14".to_string(),
            direction: "LONG".to_string(),
            paper_entry_price: entry,
            paper_target: target,
            paper_risk: stop,
            candidate_stop_price: Some(stop),
            paper_exit_price: None,
            exit_reason: "".to_string(),
            realized_return: None,
            bars_held: 0,
            ..Default::default()
        },
        walk: PaperWalkState::open("LONG", entry, target, Some(stop)),
        current_price: entry,
        opened_at: unix,
        last_tick_at: unix,
        horizon: HorizonPolicy::Unix { horizon_unix: unix + 300 * 60 },
        events: vec![],
    }
}

fn enter_event(ticker: &str, unix: i64) -> LivePaperEvent {
    LivePaperEvent { unix, kind: "PAPER_ENTER".to_string(), price: 100.0, note: Some("LONG".to_string()) }
}

fn exit_event(ticker: &str, unix: i64, kind: &str, price: f64) -> LivePaperEvent {
    LivePaperEvent { unix, kind: kind.to_string(), price, note: Some(kind.to_string()) }
}

#[test]
fn test_no_trigger_shadow_equals_baseline() {
    let mut val = E4ShadowValidator::new();
    let mut ledger = LivePaperLedger::new();
    ledger.positions.push(mock_pos("d1", "AAA", 100.0, 110.0, 90.0, 1000));
    
    val.step(&MarketObservation::last("AAA", 1000, 100.0), &[enter_event("AAA", 1000)], &ledger);
    assert_eq!(val.active_positions.len(), 1);
    
    // Close never reaches +0.50% (100.5)
    val.step(&MarketObservation::last("AAA", 1060, 100.2), &[], &ledger);
    
    // Target hit
    val.step(&MarketObservation { ticker: "AAA".into(), unix: 1120, price: 110.0, high: Some(110.0), low: Some(109.0) }, &[], &ledger);
    
    // Prod exits
    ledger.positions[0].paper.paper_exit_price = Some(110.0);
    ledger.positions[0].paper.exit_reason = "TARGET".to_string();
    ledger.positions[0].paper.realized_return = Some(0.1);
    ledger.positions[0].walk.exit = Some(super::super::paper_lifecycle::LifecycleExit { unix: 1120, price: 110.0, reason: ExitReason::Target });
    
    val.step(&MarketObservation::last("AAA", 1120, 110.0), &[exit_event("AAA", 1120, "TARGET", 110.0)], &ledger);
    
    // Flush happens when prod exits and both shadows exit.
    assert_eq!(val.active_positions.len(), 0); // flushed
    let content = fs::read_to_string("live_capture/ledger/shadow_ledger_e4_050_20260914.jsonl").unwrap_or_default();
    assert!(content.contains(r#""data_integrity":"COMPLETE""#));
}

#[test]
fn test_close_exactly_trigger() {
    let mut val = E4ShadowValidator::new();
    let mut ledger = LivePaperLedger::new();
    ledger.positions.push(mock_pos("d2", "BBB", 100.0, 110.0, 90.0, 1000));
    
    val.step(&MarketObservation::last("BBB", 1000, 100.0), &[enter_event("BBB", 1000)], &ledger);
    // Close exactly 100.5 -> trigger!
    val.step(&MarketObservation::last("BBB", 1060, 100.5), &[], &ledger);
    assert!(val.active_positions[0].triggered);
}

#[test]
fn test_high_trigger_close_below_no_trigger() {
    let mut val = E4ShadowValidator::new();
    let mut ledger = LivePaperLedger::new();
    ledger.positions.push(mock_pos("d3", "CCC", 100.0, 110.0, 90.0, 1000));
    
    val.step(&MarketObservation::last("CCC", 1000, 100.0), &[enter_event("CCC", 1000)], &ledger);
    // High is 100.6, but close is 100.4
    val.step(&MarketObservation { ticker: "CCC".into(), unix: 1060, price: 100.4, high: Some(100.6), low: Some(100.0) }, &[], &ledger);
    assert!(!val.active_positions[0].triggered);
}

#[test]
fn test_trigger_bar_stop_not_active() {
    let mut val = E4ShadowValidator::new();
    let mut ledger = LivePaperLedger::new();
    ledger.positions.push(mock_pos("d4", "DDD", 100.0, 110.0, 90.0, 1000));
    
    val.step(&MarketObservation::last("DDD", 1000, 100.0), &[enter_event("DDD", 1000)], &ledger);
    // Close is 100.5, Low is 100.0. If stop was active on SAME bar, it would hit 100.05.
    val.step(&MarketObservation { ticker: "DDD".into(), unix: 1060, price: 100.5, high: Some(100.5), low: Some(100.0) }, &[], &ledger);
    
    assert!(val.active_positions[0].triggered);
    assert_eq!(val.active_positions[0].shadow_exit_timestamp, None); // didn't exit on the same bar
}

#[test]
fn test_next_bar_crosses_stop_shadow_exit() {
    let mut val = E4ShadowValidator::new();
    let mut ledger = LivePaperLedger::new();
    ledger.positions.push(mock_pos("d5", "EEE", 100.0, 110.0, 90.0, 1000));
    
    val.step(&MarketObservation::last("EEE", 1000, 100.0), &[enter_event("EEE", 1000)], &ledger);
    // Trigger
    val.step(&MarketObservation::last("EEE", 1060, 100.5), &[], &ledger);
    // Next bar low hits 100.0. Stop is at 100.05
    val.step(&MarketObservation { ticker: "EEE".into(), unix: 1120, price: 100.0, high: Some(100.5), low: Some(100.0) }, &[], &ledger);
    
    let pos = &val.active_positions[0];
    assert_eq!(pos.shadow_exit_reason.as_deref(), Some("E4-PP-050"));
    assert_eq!(pos.shadow_exit_price, Some(100.05));
    assert_eq!(pos.baseline_exit_timestamp, None); // Baseline is still going
}

#[test]
fn test_reconciliation_failure() {
    let mut val = E4ShadowValidator::new();
    let mut ledger = LivePaperLedger::new();
    ledger.positions.push(mock_pos("d6", "FFF", 100.0, 110.0, 90.0, 1000));
    
    val.step(&MarketObservation::last("FFF", 1000, 100.0), &[enter_event("FFF", 1000)], &ledger);
    // Target hit
    val.step(&MarketObservation { ticker: "FFF".into(), unix: 1120, price: 110.0, high: Some(110.0), low: Some(109.0) }, &[], &ledger);
    
    // Prod exits with DIFFERENT price (simulating discrepancy)
    ledger.positions[0].paper.paper_exit_price = Some(110.1); // Mismatch!
    ledger.positions[0].paper.exit_reason = "TARGET".to_string();
    ledger.positions[0].paper.realized_return = Some(0.101);
    ledger.positions[0].walk.exit = Some(super::super::paper_lifecycle::LifecycleExit { unix: 1120, price: 110.1, reason: ExitReason::Target });
    
    val.step(&MarketObservation::last("FFF", 1120, 110.0), &[exit_event("FFF", 1120, "TARGET", 110.1)], &ledger);
    
    assert_eq!(val.active_positions.len(), 0);
    // It should have flushed with RECONCILIATION_FAILURE.
    let content = fs::read_to_string("live_capture/ledger/shadow_ledger_e4_050_20260914.jsonl").unwrap_or_default();
    assert!(content.contains(r#""data_integrity":"RECONCILIATION_FAILURE""#));
}
