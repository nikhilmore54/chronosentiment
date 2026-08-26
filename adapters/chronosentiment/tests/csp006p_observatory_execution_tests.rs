//! CS-P-006-P.E targeted execution. Target sealed at T. No C.3-G. No Search #3.

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::observatory_execution::ExitReason;
use chronosentiment_adapter::decision_support::observatory_execution::{
    first_exit, first_exit_with_optional_stop, refuse_protected_output, render_execution_report,
    replay_targeted_execution, seal_execution_intent, C3G_EXPERIMENT_AUTHORIZED,
    EXECUTION_CONTRACT_ID, EXECUTION_TARGET_PCT, SEARCH_THREE_AUTHORIZED, STOP_EXIT_AUTHORIZED,
    TARGETED_EXECUTION_STARTED, TARGETED_EXECUTION_V0_FROZEN, TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_slice::SealedDecisionRecord;
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;

fn paper_decision(action: DecisionAction, time: &str) -> SealedDecisionRecord {
    SealedDecisionRecord {
        decision_id: "exec-test".into(),
        instrument: "INFY.NS".into(),
        decision_time: time.into(),
        state: chronosentiment_adapter::decision_support::observatory_slice::certified_tmv_state(
            "Bullish", "Positive", "present",
        ),
        action,
        policy_id: "C3-002".into(),
        policy_artifact_sha256: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        engine_version: "unfrozen-dev".into(),
        horizon_days: 20,
        sealed_status: "OPEN".into(),
        paper_only: true,
    }
}

fn bar(
    year: i32,
    month: u32,
    day: u32,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
) -> YahooHistoricalBar {
    let ts = Utc
        .with_ymd_and_hms(year, month, day, 3, 45, 0)
        .unwrap()
        .timestamp();
    YahooHistoricalBar {
        timestamp: ts,
        open,
        high,
        low,
        close,
        adj_close: close,
        volume: 1.0,
    }
}

#[test]
fn execution_protections_stay_closed() {
    assert!(TARGETED_EXECUTION_STARTED);
    assert!(!TARGET_PATH_OPTIMIZATION_AUTHORIZED);
    assert!(!STOP_EXIT_AUTHORIZED);
    assert!(!SEARCH_THREE_AUTHORIZED);
    assert!(!C3G_EXPERIMENT_AUTHORIZED);
    assert_eq!(EXECUTION_TARGET_PCT, 0.05);
    assert!(
        refuse_protected_output("product_validation/CS-P-006/observatory/prospective").is_err()
    );
    assert!(refuse_protected_output(
        "product_validation/CS-P-006/observatory/historical_replay_v1"
    )
    .is_err());
    assert!(refuse_protected_output(
        "product_validation/CS-P-006/observatory/historical_replay_v0"
    )
    .is_err());
    assert!(TARGETED_EXECUTION_V0_FROZEN);
    assert!(refuse_protected_output(
        "product_validation/CS-P-006/observatory/targeted_execution_v0"
    )
    .is_err());
}

#[test]
fn target_is_sealed_at_t_and_intraday_high_counts() {
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    assert!(intent.sealed_at_t);
    assert_eq!(intent.target_price, 105.0);
    assert!(intent.stop_price.is_none());
    let json = serde_json::to_string(&intent).unwrap();
    assert!(!json.contains("realized_return"));
    assert!(!json.contains("future_return"));
    let bars = vec![
        bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0),
        bar(2026, 5, 16, 101.0, 105.4, 100.5, 103.0),
    ];
    let exit = first_exit(&decision, &intent, &bars).unwrap();
    assert_eq!(exit.exit_reason, ExitReason::Target);
    assert!(exit.target_hit);
    assert_eq!(exit.target_hit_session, Some(1));
    assert!((exit.exit_price.unwrap() - 105.0).abs() < 1e-9);
    assert!((exit.decision_value.unwrap() - 0.05).abs() < 1e-9);
}

#[test]
fn gap_through_target_fills_at_open() {
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    let bars = vec![
        bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0),
        bar(2026, 5, 16, 106.0, 107.0, 105.5, 106.5),
    ];
    let exit = first_exit(&decision, &intent, &bars).unwrap();
    assert_eq!(exit.exit_reason, ExitReason::Target);
    assert!((exit.exit_price.unwrap() - 106.0).abs() < 1e-9);
}

#[test]
fn short_intraday_low_counts_as_target() {
    let decision = paper_decision(DecisionAction::Short, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    assert_eq!(intent.target_price, 95.0);
    let bars = vec![
        bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0),
        bar(2026, 5, 16, 99.0, 99.5, 94.5, 97.0),
    ];
    let exit = first_exit(&decision, &intent, &bars).unwrap();
    assert_eq!(exit.exit_reason, ExitReason::Target);
    assert!((exit.exit_price.unwrap() - 95.0).abs() < 1e-9);
    assert!((exit.decision_value.unwrap() - 0.05).abs() < 1e-9);
}

#[test]
fn horizon_exit_when_target_never_prints() {
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    let mut bars = vec![bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0)];
    let start = Utc.with_ymd_and_hms(2026, 5, 15, 3, 45, 0).unwrap();
    for i in 1..=20 {
        let ts = start + chrono::Duration::days(i);
        bars.push(YahooHistoricalBar {
            timestamp: ts.timestamp(),
            open: 100.0,
            high: 101.0,
            low: 99.0,
            close: 100.0 + i as f64 * 0.01,
            adj_close: 100.0 + i as f64 * 0.01,
            volume: 1.0,
        });
    }
    let exit = first_exit(&decision, &intent, &bars).unwrap();
    assert_eq!(exit.exit_reason, ExitReason::Horizon);
    assert!(!exit.target_hit);
    assert_eq!(exit.holding_sessions, Some(20));
}

#[test]
fn same_bar_target_and_stop_is_ambiguous() {
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    let bars = vec![
        bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0),
        bar(2026, 5, 16, 100.0, 106.0, 96.0, 101.0),
    ];
    let exit = first_exit_with_optional_stop(&decision, &intent, &bars, Some(97.0), true).unwrap();
    assert_eq!(exit.exit_reason, ExitReason::Ambiguous);
    assert!(!exit.target_hit);
}

#[test]
fn no_trade_has_no_target_path() {
    let decision = paper_decision(DecisionAction::NoTrade, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    let exit = first_exit(&decision, &intent, &[]).unwrap();
    assert_eq!(exit.exit_reason, ExitReason::NoTrade);
    assert_eq!(exit.decision_value, Some(0.0));
}

#[test]
fn source_does_not_retune_or_peek_the_target() {
    let src = include_str!("../src/decision_support/observatory_execution.rs");
    assert!(!src.contains("evolve_"));
    assert!(!src.contains("regime_detector"));
    assert!(!src.contains("Strategy v2"));
    assert!(src.contains("TARGET_PATH_OPTIMIZATION_AUTHORIZED: bool = false"));
    assert!(src.contains("STOP_EXIT_AUTHORIZED: bool = false"));
}

#[test]
fn document_is_not_p7_and_does_not_reopen_research() {
    let doc = include_str!("../../../docs/CS-P-006-P.E_TARGETED_DECISION_EXECUTION.md");
    assert!(doc.contains("This document is **not** P.7") || doc.contains("not P.7"));
    assert!(doc.contains("sealed at T"));
    assert!(doc.contains("OHLC"));
    assert!(doc.contains("TARGET_PATH_OPTIMIZATION_AUTHORIZED = false"));
    assert!(doc.contains("not C.3-G") || doc.contains("C.3-G"));
    assert!(doc.contains("Search #3"));
    let surface = include_str!("../../../docs/CS-P-006-P.E.1_EXECUTION_EVIDENCE_SURFACE.md");
    assert!(surface.contains("Execution Contract v0"));
    assert!(surface.contains("target_pct = 5.0%"));
    assert!(surface.contains("Did the predefined target get reached"));
    assert!(surface.contains("Both are evidence"));
    assert!(surface.contains("Frozen"));
    assert!(!surface.contains("C3-002’s 5% target") || surface.contains("Do not say"));
}

#[test]
fn replay_report_names_the_contract() {
    let report = chronosentiment_adapter::decision_support::observatory_execution::TargetedExecutionReport {
        path_kind: "targeted_execution_replay".into(),
        execution_contract: EXECUTION_CONTRACT_ID.into(),
        target_source: "deterministic_policy_parameter".into(),
        target_pct: 0.05,
        max_holding_sessions: 20,
        stop_exit_authorized: false,
        target_path_optimization_authorized: false,
        n_decisions: 2,
        n_exits: 2,
        n_target: 1,
        n_horizon: 1,
        n_no_trade: 0,
        peeked_returns_at_seal: false,
        prospective_cohort_mutated: false,
        statistical_backtest: false,
        ticks: vec![
            chronosentiment_adapter::decision_support::observatory_execution::TargetedExecutionTick {
                instrument: "INFY.NS".into(),
                requested_clock: "2026-05-15T03:45:00+00:00".into(),
                decision_time: "2026-05-15T03:45:00+00:00".into(),
                decision_id: "demo-target".into(),
                direction: "LONG".into(),
                entry_price: 1095.30,
                target_pct: 0.05,
                target_price: 1150.06,
                target_hit: true,
                target_hit_session: Some(2),
                exit_price: Some(1150.06),
                exit_reason: ExitReason::Target,
                holding_sessions: Some(2),
                decision_value: Some(0.05),
                peeked_returns_at_seal: false,
            },
            chronosentiment_adapter::decision_support::observatory_execution::TargetedExecutionTick {
                instrument: "HDFCBANK.NS".into(),
                requested_clock: "2026-05-15T03:45:00+00:00".into(),
                decision_time: "2026-05-15T03:45:00+00:00".into(),
                decision_id: "demo-horizon".into(),
                direction: "LONG".into(),
                entry_price: 755.01,
                target_pct: 0.05,
                target_price: 792.76,
                target_hit: false,
                target_hit_session: None,
                exit_price: Some(759.88),
                exit_reason: ExitReason::Horizon,
                holding_sessions: Some(20),
                decision_value: Some(0.0064),
                peeked_returns_at_seal: false,
            },
        ],
    };
    let md = render_execution_report(&report);
    assert!(md.contains("backtesting mechanism"));
    assert!(md.contains("not yet a statistical strategy backtest"));
    assert!(md.contains(EXECUTION_CONTRACT_ID));
    assert!(md.contains("Execution Contract v0"));
    assert!(md.contains("C3-002 chooses direction only"));
    assert!(md.contains("C3-002 does not have a 5% target"));
    let html =
        chronosentiment_adapter::decision_support::observatory_execution::render_execution_html(
            &report,
        );
    assert!(html.contains("Execution Contract v0"));
    assert!(html.contains("<dt>Decision</dt>"));
    assert!(html.contains("<dt>Target</dt>"));
    assert!(html.contains("<dt>Maximum hold</dt>"));
    assert!(html.contains("<dt>Exit</dt>"));
    assert!(html.contains("<dt>Holding period</dt>"));
    assert!(html.contains("<dt>Realized decision value</dt>"));
    assert!(html.contains("TARGET"));
    assert!(html.contains("HORIZON"));
    assert!(html.contains("target_pct = 5.0%"));
    assert!(!html.contains("C3-002 target"));
}

#[test]
fn replay_uses_c3_002_and_does_not_touch_prospective() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let Some(cache) = load_cache() else {
        return;
    };
    let clocks =
        chronosentiment_adapter::decision_support::observatory_execution::default_execution_clocks(
        )
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 15, 6, 30, 0).unwrap();
    let (intents, report) = replay_targeted_execution(&artifact, &cache, &clocks, now).unwrap();
    assert_eq!(intents.len(), 14);
    assert_eq!(report.n_decisions, 14);
    assert!(report.n_exits <= 14);
    assert!(!report.peeked_returns_at_seal);
    assert!(!report.prospective_cohort_mutated);
    assert!(!report.statistical_backtest);
    assert!(intents
        .iter()
        .all(|i| i.sealed_at_t && i.target_pct == 0.05));
    assert!(intents.iter().any(|i| i.instrument == "IDEA.NS"));
    assert!(intents.iter().any(|i| i.instrument == "MAHABANK.NS"));
}

fn load_c3_002(
) -> Option<chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(
            chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_DIR,
        )
        .join("selected_policy.json");
    if !path.exists() {
        return None;
    }
    Some(serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
}

fn load_cache() -> Option<
    std::collections::BTreeMap<
        String,
        Vec<chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar>,
    >,
> {
    let cache_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_SNAPSHOT_DIR)
        .join("yahoo_cache");
    if !cache_dir.exists() {
        return None;
    }
    Some(
        chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache(
            &cache_dir,
        )
        .unwrap(),
    )
}
