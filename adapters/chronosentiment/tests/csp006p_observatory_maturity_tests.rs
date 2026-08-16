//! CS-P-006-P maturity path. No early peek. No C3-002 retune. No Search #3.

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_SNAPSHOT_DIR;
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_maturity::{
    append_matured_observation, days_remaining, nth_market_session_after, observation_due_at,
    observation_window_closed, require_window_closed, ui_lifecycle_status,
    INTERMEDIATE_INTERPRETATION_AUTHORIZED, OBSERVATORY_MATURITY_STARTED,
    POLICY_RETUNE_FROM_PROSPECTIVE_AUTHORIZED, UNIVERSE_EXPANSION_AUTHORIZED,
    UI_STATUS_OUTCOME_DUE,
};
use chronosentiment_adapter::decision_support::observatory_slice::{
    empty_ledger, observe_outcome, seal_into_ledger, SealedDecisionRecord, UI_STATUS_OBSERVING,
};
use chronosentiment_adapter::decision_support::DecisionAction;

fn paper_decision(time: &str) -> SealedDecisionRecord {
    SealedDecisionRecord {
        decision_id: "maturity-test".into(),
        instrument: "INFY.NS".into(),
        decision_time: time.into(),
        state: chronosentiment_adapter::decision_support::observatory_slice::certified_tmv_state(
            "Bullish",
            "Positive",
            "present",
        ),
        action: DecisionAction::Long,
        policy_id: "C3-002".into(),
        policy_artifact_sha256: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        engine_version: "unfrozen-dev".into(),
        horizon_days: 20,
        sealed_status: "OPEN".into(),
        paper_only: true,
    }
}

#[test]
fn maturity_protections_stay_closed() {
    assert!(OBSERVATORY_MATURITY_STARTED);
    assert!(!INTERMEDIATE_INTERPRETATION_AUTHORIZED);
    assert!(!POLICY_RETUNE_FROM_PROSPECTIVE_AUTHORIZED);
    assert!(!UNIVERSE_EXPANSION_AUTHORIZED);
}

#[test]
fn fourteen_aug_tick_is_still_observing_on_fifteen_aug() {
    let decision = paper_decision("2026-08-14T03:45:00+00:00");
    let now = Utc.with_ymd_and_hms(2026, 8, 15, 6, 30, 0).unwrap();
    assert!(!observation_window_closed(&decision, now).unwrap());
    assert_eq!(days_remaining(&decision, now).unwrap(), 20);
    assert!(
        chronosentiment_adapter::decision_support::observatory_maturity::format_observation_close(
            &decision
        )
        .contains("11 Sep 2026, 03:45 UTC")
    );
    assert_eq!(
        observation_due_at(&decision).unwrap(),
        Utc.with_ymd_and_hms(2026, 9, 11, 3, 45, 0).unwrap()
    );
    assert!(require_window_closed(&decision, now).is_err());
    let mut ledger = empty_ledger();
    seal_into_ledger(&mut ledger, decision.clone()).unwrap();
    assert_eq!(
        ui_lifecycle_status(&ledger, &decision.decision_id, now),
        UI_STATUS_OBSERVING
    );
    let observation = observe_outcome(&decision, "2026-08-15T06:30:00Z", 0.01).unwrap();
    assert!(append_matured_observation(&mut ledger, &decision, observation, now).is_err());
    assert!(ledger.observations.is_empty());
}

#[test]
fn window_closed_becomes_outcome_due_and_may_append() {
    let decision = paper_decision("2026-08-14T03:45:00+00:00");
    let now = Utc.with_ymd_and_hms(2026, 9, 11, 3, 45, 0).unwrap();
    assert!(observation_window_closed(&decision, now).unwrap());
    let mut ledger = empty_ledger();
    seal_into_ledger(&mut ledger, decision.clone()).unwrap();
    assert_eq!(
        ui_lifecycle_status(&ledger, &decision.decision_id, now),
        UI_STATUS_OUTCOME_DUE
    );
    let observation = observe_outcome(&decision, "2026-09-03T03:45:00Z", -0.012).unwrap();
    append_matured_observation(&mut ledger, &decision, observation, now).unwrap();
    assert_eq!(ledger.decisions[0].sealed_status, "OPEN");
    assert_eq!(ledger.decisions[0].policy_artifact_sha256, RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH);
}

#[test]
fn source_does_not_peek_or_retune() {
    let src = include_str!("../src/decision_support/observatory_maturity.rs");
    let bin = include_str!("../src/bin/csp006_p_observe.rs");
    for text in [src, bin] {
        assert!(!text.contains("evolve_"));
        assert!(!text.contains("regime_detector"));
        assert!(!text.contains("Strategy v2"));
        assert!(!text.contains("YahooProvider"));
    }
}

#[test]
fn document_records_maturity_and_three_layers() {
    let doc = include_str!("../../../docs/CS-P-006-P_DECISION_OBSERVATORY.md");
    assert!(doc.contains("OUTCOME DUE") || doc.contains("maturity"));
    assert!(doc.contains("Intelligence") || doc.contains("three"));
    assert!(doc.contains("No early peek. No retrospective edits."));
    assert!(doc.contains("evidence dashboard"));
    assert!(doc.contains("C.3-G remains a question") || doc.contains("C.3-G remains an unanswered"));
}

#[test]
fn twenty_market_sessions_follow_the_exchange_calendar() {
    let cache_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(RESEARCH_SNAPSHOT_DIR)
        .join("yahoo_cache");
    if !cache_dir.exists() {
        return;
    }
    let cache = load_required_yahoo_cache(&cache_dir).unwrap();
    let bars = cache.get("INFY.NS").unwrap();
    let t = Utc.with_ymd_and_hms(2026, 5, 15, 3, 45, 0).unwrap();
    assert_eq!(
        nth_market_session_after(bars, t, 20).unwrap(),
        Utc.with_ymd_and_hms(2026, 6, 12, 3, 45, 0).unwrap()
    );
    let june = Utc.with_ymd_and_hms(2026, 6, 12, 3, 45, 0).unwrap();
    assert_eq!(
        nth_market_session_after(bars, june, 20).unwrap(),
        Utc.with_ymd_and_hms(2026, 7, 10, 3, 45, 0).unwrap()
    );
    let live = Utc.with_ymd_and_hms(2026, 8, 14, 3, 45, 0).unwrap();
    assert!(nth_market_session_after(bars, live, 20).is_none());
}
