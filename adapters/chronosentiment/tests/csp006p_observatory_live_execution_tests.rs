//! CS-P-006-P.E.2 live execution observation. No 14-Aug mutation. No C.3-G.

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::observatory_execution::{
    first_exit, refuse_protected_output, seal_execution_intent, ExitReason, TriggerType,
    TARGETED_EXECUTION_V0_FROZEN,
};
use chronosentiment_adapter::decision_support::observatory_live_execution::{
    is_protected_direction_only_clock, refuse_live_execution_output, render_live_execution_html,
    run_live_execution, CONTINUOUS_SESSION_SEAL_AUTHORIZED,
    FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED, LIVE_EXECUTION_STATUS_AWAITING,
    LIVE_YAHOO_FETCH_AUTHORIZED, PE1_SIDECAR_MUTATION_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_slice::SealedDecisionRecord;
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;

fn paper_decision(action: DecisionAction, time: &str) -> SealedDecisionRecord {
    SealedDecisionRecord {
        decision_id: "live-exec-test".into(),
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
fn live_protections_stay_closed() {
    assert!(!FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED);
    assert!(!PE1_SIDECAR_MUTATION_AUTHORIZED);
    assert!(!CONTINUOUS_SESSION_SEAL_AUTHORIZED);
    assert!(!LIVE_YAHOO_FETCH_AUTHORIZED);
    assert!(TARGETED_EXECUTION_V0_FROZEN);
    assert!(is_protected_direction_only_clock(
        Utc.with_ymd_and_hms(2026, 8, 14, 3, 45, 0).unwrap()
    ));
    assert!(!is_protected_direction_only_clock(
        Utc.with_ymd_and_hms(2026, 8, 17, 3, 45, 0).unwrap()
    ));
    assert!(
        refuse_live_execution_output("product_validation/CS-P-006/observatory/prospective")
            .is_err()
    );
    assert!(refuse_live_execution_output(
        "product_validation/CS-P-006/observatory/targeted_execution_v0"
    )
    .is_err());
    assert!(refuse_protected_output(
        "product_validation/CS-P-006/observatory/targeted_execution_v0"
    )
    .is_err());
    assert!(refuse_live_execution_output(
        "product_validation/CS-P-006/observatory/prospective_execution_v0"
    )
    .is_ok());
}

#[test]
fn high_reached_records_trigger_audit() {
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    let bars = vec![
        bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0),
        bar(2026, 5, 16, 101.0, 105.4, 100.5, 103.0),
    ];
    let exit = first_exit(&decision, &intent, &bars).unwrap();
    assert_eq!(exit.exit_reason, ExitReason::Target);
    assert_eq!(exit.trigger_type, Some(TriggerType::HighReached));
    assert_eq!(exit.trigger_session, Some(1));
    assert!((exit.trigger_price.unwrap() - 105.4).abs() < 1e-9);
    assert!((exit.execution_price.unwrap() - 105.0).abs() < 1e-9);
}

#[test]
fn gap_through_records_session_open() {
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    let bars = vec![
        bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0),
        bar(2026, 5, 16, 106.0, 107.0, 105.5, 106.5),
    ];
    let exit = first_exit(&decision, &intent, &bars).unwrap();
    assert_eq!(exit.trigger_type, Some(TriggerType::GapThrough));
    assert!((exit.execution_price.unwrap() - 106.0).abs() < 1e-9);
    assert!((exit.trigger_price.unwrap() - 106.0).abs() < 1e-9);
}

#[test]
fn short_low_reached_records_trigger_audit() {
    let decision = paper_decision(DecisionAction::Short, "2026-05-15T03:45:00+00:00");
    let intent = seal_execution_intent(&decision, 100.0, 0.05).unwrap();
    let bars = vec![
        bar(2026, 5, 15, 100.0, 101.0, 99.0, 100.0),
        bar(2026, 5, 16, 99.0, 99.5, 94.5, 97.0),
    ];
    let exit = first_exit(&decision, &intent, &bars).unwrap();
    assert_eq!(exit.trigger_type, Some(TriggerType::LowReached));
    assert!((exit.trigger_price.unwrap() - 94.5).abs() < 1e-9);
    assert!((exit.execution_price.unwrap() - 95.0).abs() < 1e-9);
}

#[test]
fn horizon_records_session_close() {
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
    assert_eq!(exit.trigger_type, Some(TriggerType::SessionClose));
    assert_eq!(exit.holding_sessions, Some(20));
    assert!((exit.execution_price.unwrap() - 100.20).abs() < 1e-9);
}

#[test]
fn certified_cache_awaits_the_next_session() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let Some(cache) = load_cache() else {
        return;
    };
    let now = Utc.with_ymd_and_hms(2026, 8, 15, 8, 30, 0).unwrap();
    let ledger = run_live_execution(&artifact, &cache, now, None).unwrap();
    assert_eq!(ledger.seal_status, LIVE_EXECUTION_STATUS_AWAITING);
    assert_eq!(ledger.n_decisions, 0);
    assert!(!ledger.fourteen_aug_cohort_mutated);
    assert!(!ledger.pe1_sidecar_mutated);
    assert!(ledger
        .certified_t
        .as_ref()
        .unwrap()
        .starts_with("2026-08-14T03:45:00"));
    let html = render_live_execution_html(&ledger);
    assert!(html.contains("AWAITING_NEXT_SESSION"));
    assert!(html.contains("14-August cohort was sealed without an execution intent"));
    assert!(html.contains("next eligible cohort"));
    assert!(html.contains("Decision + Execution Intent"));
    assert!(html.contains("14-Aug cohort"));
    assert!(html.contains("IDEA"));
    assert!(!html.contains("C3-002 target"));
    assert!(!html.contains("Sealed without an execution target"));
}

#[test]
fn next_session_seals_all_seven_including_idea_and_mahabank() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let Some(mut cache) = load_cache() else {
        return;
    };
    let next = Utc.with_ymd_and_hms(2026, 8, 17, 3, 45, 0).unwrap();
    for bars in cache.values_mut() {
        let last = bars.last().cloned().expect("cache bar");
        bars.push(YahooHistoricalBar {
            timestamp: next.timestamp(),
            open: last.adj_close,
            high: last.adj_close * 1.01,
            low: last.adj_close * 0.99,
            close: last.adj_close,
            adj_close: last.adj_close,
            volume: 1.0,
        });
    }
    let now = Utc.with_ymd_and_hms(2026, 8, 17, 12, 0, 0).unwrap();
    let ledger = run_live_execution(&artifact, &cache, now, None).unwrap();
    assert_eq!(ledger.n_decisions, 7);
    assert_eq!(ledger.n_observing, 7);
    assert!(ledger.records.iter().any(|r| r.instrument == "IDEA.NS"));
    assert!(ledger.records.iter().any(|r| r.instrument == "MAHABANK.NS"));
    assert!(ledger
        .records
        .iter()
        .all(|r| r.intent.sealed_at_t && r.intent.target_pct == 0.05));
    assert!(ledger
        .records
        .iter()
        .all(|r| r.exit.exit_reason == ExitReason::Observing));
    assert!(ledger.records.iter().all(|r| r.exit.trigger_type.is_none()));
    assert!(!ledger.peeked_returns_at_seal);
    let again = run_live_execution(&artifact, &cache, now, Some(ledger.clone())).unwrap();
    assert_eq!(again.n_decisions, 7);
}

#[test]
fn document_freezes_pe1_and_protects_fourteen_aug() {
    let doc = include_str!("../../../docs/CS-P-006-P.E.2_LIVE_EXECUTION_OBSERVATION.md");
    assert!(doc.contains("AWAITING_NEXT_SESSION"));
    assert!(doc.contains("trigger_type"));
    assert!(doc.contains("HIGH_REACHED"));
    assert!(doc.contains("GAP_THROUGH"));
    assert!(doc.contains("14-August cohort was sealed without an execution intent"));
    assert!(doc.contains("next eligible cohort"));
    assert!(doc.contains("Decision + Execution Intent"));
    assert!(doc.contains("14-Aug cohort"));
    assert!(!doc.contains("Sealed without an execution target"));
    assert!(doc.contains("FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED = false"));
    assert!(doc.contains("not C.3-G") || doc.contains("C.3-G"));
    assert!(doc.contains("Search #3"));
    let pe1 = include_str!("../../../docs/CS-P-006-P.E.1_EXECUTION_EVIDENCE_SURFACE.md");
    assert!(pe1.contains("Frozen"));
    assert!(doc.contains("Frozen"));
    assert!(
        doc.contains("not a test of whether 5% is a good target")
            || doc.contains("Not: is 5% a good target")
    );
    assert!(doc.contains("Execution Intent"));
    let pe3 = include_str!("../../../docs/CS-P-006-P.E.3_CORALYS_TARGET_DISCOVERY.md");
    assert!(pe3.contains("Specified"));
    assert!(pe3.contains("Frozen Coralys target artifact"));
    assert!(pe3.contains("hindsight optimization"));
    assert!(pe3.contains("CORALYS_TARGET_SEARCH_AUTHORIZED = false"));
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
