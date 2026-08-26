//! CS-P-006-P.H Historical Observatory Replay.
//! Same engine as prospective. No lookahead. Not C.3-G. No Search #3.

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR, RESEARCH_SNAPSHOT_DIR,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_historical::{
    decision_time_bars, generate_historical_replay_decision, observe_if_due, parse_replay_clocks,
    refuse_prospective_output, render_replay_html, render_replay_report, replay_cohort,
    C3G_EXPERIMENT_AUTHORIZED, DEFAULT_REPLAY_CLOCKS, HISTORICAL_REPLAY_PATH_KIND,
    HISTORICAL_REPLAY_STARTED, LOOKAHEAD_BACKTEST_AUTHORIZED, PEEKED_RETURNS_AT_SEAL,
    PROSPECTIVE_COHORT_MUTATION_AUTHORIZED, SEARCH_THREE_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_maturity::{
    append_matured_observation, ui_lifecycle_status, UI_STATUS_OUTCOME_DUE,
};
use chronosentiment_adapter::decision_support::observatory_maturity::{
    HORIZON_CALENDAR_BASIS, TRADING_SESSION_HORIZON_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_slice::{
    empty_ledger, observe_outcome, UI_STATUS_OBSERVING,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_c3_002() -> Option<PolicyArtifact> {
    let path = workspace_root()
        .join(RESEARCH_DISCOVERY_TWO_DIR)
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
    let cache_dir = workspace_root()
        .join(RESEARCH_SNAPSHOT_DIR)
        .join("yahoo_cache");
    if !cache_dir.exists() {
        return None;
    }
    Some(load_required_yahoo_cache(&cache_dir).unwrap())
}

#[test]
fn historical_replay_stays_quarantined() {
    assert!(HISTORICAL_REPLAY_STARTED);
    assert!(!PEEKED_RETURNS_AT_SEAL);
    assert!(!LOOKAHEAD_BACKTEST_AUTHORIZED);
    assert!(!PROSPECTIVE_COHORT_MUTATION_AUTHORIZED);
    assert!(!C3G_EXPERIMENT_AUTHORIZED);
    assert!(!SEARCH_THREE_AUTHORIZED);
    assert_eq!(HORIZON_CALENDAR_BASIS, "TRADING_DAYS");
    assert!(TRADING_SESSION_HORIZON_AUTHORIZED);
    assert!(refuse_prospective_output(
        "product_validation/CS-P-006/observatory/historical_replay_v0"
    )
    .is_err());
    assert!(
        refuse_prospective_output("product_validation/CS-P-006/observatory/prospective").is_err()
    );
}

#[test]
fn generate_decision_cannot_take_returns() {
    let decide = include_str!("../src/decision_support/observatory_slice.rs");
    let replay = include_str!("../src/decision_support/observatory_historical.rs");
    let policy = include_str!("../src/decision_support/policy_artifact.rs");
    let start = decide.find("pub fn generate_decision").unwrap();
    let body = &decide[start..start + 1200];
    assert!(body.contains("trend: &str"));
    assert!(!body.contains("realized_return"));
    assert!(!body.contains("future_return"));
    assert!(!body.contains("YahooHistoricalBar"));
    assert!(!replay.contains("evolve_"));
    assert!(!replay.contains("regime_detector"));
    assert!(!replay.contains("Strategy v2"));
    let first_match = policy.find("pub fn first_match_action_from_tmv").unwrap();
    let first_body = &policy[first_match..first_match + 900];
    assert!(!first_body.contains("realized_return"));
    assert!(!first_body.contains("forward_return"));
    assert!(!first_body.contains("YahooHistoricalBar"));
}

#[test]
fn determinism_and_poisoned_future_do_not_change_the_decision() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let Some(cache) = load_cache() else {
        return;
    };
    let bars = cache.get("INFY.NS").unwrap();
    let t = Utc.with_ymd_and_hms(2026, 6, 14, 3, 45, 0).unwrap();
    let a = generate_historical_replay_decision(&artifact, "INFY.NS", bars, t).unwrap();
    let b = generate_historical_replay_decision(&artifact, "INFY.NS", bars, t).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        a.policy_artifact_sha256,
        RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
    );
    assert_eq!(a.decision_time, "2026-06-12T03:45:00+00:00");
    let json = serde_json::to_string(&a).unwrap();
    for forbidden in [
        "future_return",
        "\"outcome\"",
        "regret",
        "evaluation_score",
        "confidence",
        "realized_return",
    ] {
        assert!(
            !json.contains(forbidden),
            "{forbidden} leaked onto the decision"
        );
    }
    let known = decision_time_bars(bars, t);
    assert!(known.iter().all(|bar| bar.timestamp <= t.timestamp()));
    let from_known = generate_historical_replay_decision(&artifact, "INFY.NS", &known, t).unwrap();
    assert_eq!(a, from_known);
    let mut poisoned = bars.clone();
    for bar in &mut poisoned {
        if bar.timestamp > t.timestamp() {
            bar.adj_close = 1_000_000.0;
            bar.close = 1_000_000.0;
        }
    }
    let from_poisoned =
        generate_historical_replay_decision(&artifact, "INFY.NS", &poisoned, t).unwrap();
    assert_eq!(a.decision_id, from_poisoned.decision_id);
    assert_eq!(a.state, from_poisoned.state);
    assert_eq!(a.action, from_poisoned.action);
}

#[test]
fn evidence_cannot_appear_before_the_window_closes() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let Some(cache) = load_cache() else {
        return;
    };
    let bars = cache.get("INFY.NS").unwrap();
    let t = Utc.with_ymd_and_hms(2026, 6, 14, 3, 45, 0).unwrap();
    let decision = generate_historical_replay_decision(&artifact, "INFY.NS", bars, t).unwrap();
    let mut ledger =
        chronosentiment_adapter::decision_support::observatory_historical::empty_replay_ledger();
    chronosentiment_adapter::decision_support::observatory_slice::seal_into_ledger(
        &mut ledger,
        decision.clone(),
    )
    .unwrap();
    assert_eq!(ledger.path_kind, HISTORICAL_REPLAY_PATH_KIND);
    let mid = Utc.with_ymd_and_hms(2026, 6, 20, 3, 45, 0).unwrap();
    assert_eq!(
        ui_lifecycle_status(&ledger, &decision.decision_id, mid),
        UI_STATUS_OBSERVING
    );
    assert!(observe_if_due(&mut ledger, &decision, bars, mid)
        .unwrap()
        .is_none());
    let peek = observe_outcome(&decision, "2026-06-20T03:45:00Z", 0.5).unwrap();
    assert!(append_matured_observation(&mut ledger, &decision, peek, mid).is_err());
    assert!(ledger.observations.is_empty());
    let closed = Utc.with_ymd_and_hms(2026, 7, 10, 3, 45, 0).unwrap();
    assert_eq!(
        ui_lifecycle_status(&ledger, &decision.decision_id, closed),
        UI_STATUS_OUTCOME_DUE
    );
    let attached = observe_if_due(&mut ledger, &decision, bars, closed)
        .unwrap()
        .expect("window closed");
    assert_eq!(attached.observation_status, "COMPLETED");
    assert_eq!(ledger.decisions[0].sealed_status, "OPEN");
    assert!(!serde_json::to_string(&ledger.decisions[0])
        .unwrap()
        .contains("realized_return"));
}

#[test]
fn default_cohort_is_fourteen_closed_windows_and_does_not_touch_prospective() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let Some(cache) = load_cache() else {
        return;
    };
    let clocks = parse_replay_clocks(&DEFAULT_REPLAY_CLOCKS).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 15, 6, 30, 0).unwrap();
    let (ledger, report) = replay_cohort(&artifact, &cache, &clocks, now).unwrap();
    assert_eq!(ledger.decisions.len(), 14);
    assert_eq!(ledger.observations.len(), 14);
    assert!(report.determinism_pass);
    assert!(report.lookahead_clean);
    assert!(!report.peeked_returns);
    assert!(!report.prospective_cohort_mutated);
    assert_eq!(report.horizon_calendar_basis, "TRADING_DAYS");
    assert_eq!(report.horizon_unit, "MARKET_SESSIONS");
    assert_eq!(
        report.replay_contract,
        "historical_replay_v1_20_market_sessions"
    );
    assert_eq!(report.horizon_duration_days, 20);
    assert!(!report.statistical_backtest);
    assert!(report.trading_session_horizon_authorized);
    assert!(
        report
            .ticks
            .iter()
            .any(|t| t.session_resolved_from_request
                && t.decision_time == "2026-06-12T03:45:00+00:00")
    );
    let html = render_replay_html(&ledger, &report, now);
    assert!(html.contains("20 market sessions"));
    assert!(html.contains("Requested observation clock"));
    assert!(html.contains("Certified market timestamp"));
    assert!(html.contains("without access to information after T"));
    assert!(!html.contains("observed after 20D"));
    let md = render_replay_report(&report);
    assert!(md.contains("unit = MARKET_SESSIONS"));
    assert!(md.contains("historical_replay_v1_20_market_sessions"));
    assert!(md.contains("statistical strategy backtest: not done"));
    assert!(md.contains("not a homepage metric"));
    assert!(md.contains("backtesting mechanism"));
    assert!(
        md.contains("Replay integrity is not strategy validation")
            || md.contains("replay integrity ≠ strategy validation")
    );
    assert!(md.contains("Replay v0"));
    assert!(ledger.decisions.iter().any(|d| d.instrument == "IDEA.NS"));
    assert!(ledger
        .decisions
        .iter()
        .any(|d| d.instrument == "MAHABANK.NS"));
    let prospective_path =
        workspace_root().join("product_validation/CS-P-006/observatory/prospective/ledger.json");
    if prospective_path.exists() {
        let prospective: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&prospective_path).unwrap()).unwrap();
        assert_eq!(prospective["path_kind"], "prospective_paper_clock");
        assert_eq!(prospective["observations"].as_array().unwrap().len(), 0);
        assert_eq!(prospective["decisions"].as_array().unwrap().len(), 7);
    }
    let other = empty_ledger();
    assert_ne!(other.path_kind, HISTORICAL_REPLAY_PATH_KIND);
}

#[test]
fn document_is_not_c3g_and_keeps_the_live_cohort_untouched() {
    let doc = include_str!("../../../docs/CS-P-006-P.H_HISTORICAL_REPLAY.md");
    assert!(doc.contains("historical clock") || doc.contains("Historical Observatory Replay"));
    assert!(doc.contains("not C.3-G") || doc.contains("Not C.3-G"));
    assert!(doc.contains("peeked_returns"));
    assert!(doc.contains("14 August") || doc.contains("prospective"));
    assert!(
        doc.contains("no lookahead") || doc.contains("No-lookahead") || doc.contains("lookahead")
    );
    let capability = include_str!("../../../docs/CS-P-006-P.H.1_DECISION_EVIDENCE_ENGINE.md");
    assert!(capability.contains("20 calendar days"));
    assert!(capability.contains("CALENDAR_DAYS"));
    assert!(capability.contains("latest certified market session"));
    assert!(capability.contains("Statistical C3-002 performance study"));
    assert!(capability.contains("without access to information after T"));
    let v1 = include_str!("../../../docs/CS-P-006-P.H.2_MARKET_SESSION_HORIZON.md");
    assert!(v1.contains("MARKET_SESSIONS"));
    assert!(v1.contains("historical_replay_v0"));
    assert!(v1.contains("historical_replay_v1"));
    assert!(v1.contains("Not reinterpreted"));
    let dashboard = include_str!("../../../docs/CS-P-006-P.H.3_DECISION_EVIDENCE_DASHBOARD.md");
    assert!(dashboard.contains("backtesting mechanism"));
    assert!(dashboard.contains("not yet a statistical strategy backtest"));
    assert!(dashboard.contains("Replay integrity"));
    assert!(dashboard.contains("IDEA and MAHABANK"));
    assert!(!dashboard.contains("Search #3 is authorized"));
    let v0 = include_str!(
        "../../../product_validation/CS-P-006/observatory/historical_replay_v0/REPORT.md"
    );
    assert!(v0.contains("CALENDAR_DAYS"));
    assert!(!v0.contains("MARKET_SESSIONS"));
    assert!(v0.contains("4 Jun 2026"));
    let v1_report = include_str!(
        "../../../product_validation/CS-P-006/observatory/historical_replay_v1/REPORT.md"
    );
    assert!(v1_report.contains("MARKET_SESSIONS"));
    assert!(v1_report.contains("12 Jun 2026"));
    assert!(v1_report.contains("10 Jul 2026"));
    assert!(!v1_report.contains("Observation closes 4 Jun"));
    assert!(!v1_report.contains("Observation closes 2 Jul"));
}
