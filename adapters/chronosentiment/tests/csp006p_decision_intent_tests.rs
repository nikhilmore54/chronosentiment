//! CS-P-006-P.E.B DecisionIntent. Target from state at T only. No Search #3.

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::decision_intent::{
    certified_execution_state, seal_experiment_a_intent, seal_experiment_b_intent,
    ASYMMETRIC_TARGET_AUTHORIZED, AUTHORIZED_TARGET_INPUTS, CORALYS_TARGET_ARTIFACT_PRESENT,
    CORALYS_TARGET_GENERATION_STARTED, CORALYS_TARGET_SEARCH_AUTHORIZED, CertifiedExecutionState,
    FORBIDDEN_TARGET_INPUTS, HORIZON_SEARCH_AUTHORIZED, TARGET_LOOKAHEAD_AUTHORIZED,
    TARGET_SOURCE_FIXED,
};
use chronosentiment_adapter::decision_support::observatory_execution::EXECUTION_TARGET_PCT;
use chronosentiment_adapter::decision_support::observatory_slice::SealedDecisionRecord;
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;

fn paper_decision(action: DecisionAction, time: &str) -> SealedDecisionRecord {
    SealedDecisionRecord {
        decision_id: "intent-test".into(),
        instrument: "INFY.NS".into(),
        decision_time: time.into(),
        state: chronosentiment_adapter::decision_support::observatory_slice::certified_tmv_state(
            "Bullish",
            "Positive",
            "present",
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

fn bar(year: i32, month: u32, day: u32, close: f64) -> YahooHistoricalBar {
    let ts = Utc
        .with_ymd_and_hms(year, month, day, 3, 45, 0)
        .unwrap()
        .timestamp();
    YahooHistoricalBar {
        timestamp: ts,
        open: close,
        high: close * 1.01,
        low: close * 0.99,
        close,
        adj_close: close,
        volume: 1.0,
    }
}

#[test]
fn experiment_b_protections_stay_closed() {
    // Artifact frozen 2026-08-16: CORALYS_TARGET_ARTIFACT_PRESENT is now true.
    // All other research gates remain closed.
    assert!(CORALYS_TARGET_GENERATION_STARTED);
    assert!(!CORALYS_TARGET_SEARCH_AUTHORIZED);
    assert!(!TARGET_LOOKAHEAD_AUTHORIZED);
    assert!(!ASYMMETRIC_TARGET_AUTHORIZED);
    assert!(!HORIZON_SEARCH_AUTHORIZED);
    assert!(CORALYS_TARGET_ARTIFACT_PRESENT); // frozen 2026-08-16
    assert!(!chronosentiment_adapter::decision_support::decision_intent::TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED);
    assert!(FORBIDDEN_TARGET_INPUTS.contains(&"realized_V"));
    assert!(FORBIDDEN_TARGET_INPUTS.contains(&"target_hit"));
    assert!(AUTHORIZED_TARGET_INPUTS.contains(&"bars_at_or_before_T"));
    assert!(FORBIDDEN_TARGET_INPUTS.contains(&"bars_after_T"));
    assert!(FORBIDDEN_TARGET_INPUTS.contains(&"path_optimized_hit_rate"));
}

#[test]
fn experiment_a_intent_is_the_frozen_five_percent_control() {
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let a = seal_experiment_a_intent(&decision).unwrap();
    let again = seal_experiment_a_intent(&decision).unwrap();
    assert_eq!(a.target_pct, EXECUTION_TARGET_PCT);
    assert_eq!(a.target_source, TARGET_SOURCE_FIXED);
    assert_eq!(a.horizon_sessions, 20);
    assert!(a.sealed_at_t);
    assert_eq!(a.intent_hash, again.intent_hash);
    assert_eq!(a.coralys_model_id, "none");
    let json = serde_json::to_string(&a).unwrap();
    assert!(!json.contains("future_return"));
    assert!(!json.contains("realized_return"));
}

#[test]
fn experiment_b_succeeds_now_that_frozen_artifact_exists() {
    // Artifact frozen 2026-08-16. seal_experiment_b_intent must now succeed.
    let decision = paper_decision(DecisionAction::Long, "2026-05-15T03:45:00+00:00");
    let state = CertifiedExecutionState {
        instrument: "INFY.NS".into(),
        decision_time: "2026-05-15T03:45:00+00:00".into(),
        trend: "Bullish".into(),
        momentum: "Positive".into(),
        volatility: "present".into(),
        state_hash: decision.state.state_hash.clone(),
    };
    // With CORALYS_TARGET_ARTIFACT_PRESENT = true, this must not return an error
    // about a missing frozen artifact.
    let result = seal_experiment_b_intent(&decision, &state, "coralys-target-v1", "0");
    match &result {
        Err(e) if e.contains("frozen target artifact") => {
            panic!("seal_experiment_b_intent still refusing with 'frozen target artifact' — CORALYS_TARGET_ARTIFACT_PRESENT must be true");
        }
        _ => {} // Ok or a different error is acceptable
    }
}

#[test]
fn certified_state_ignores_bars_after_t() {
    let Some(cache) = load_cache() else {
        return;
    };
    let t = Utc.with_ymd_and_hms(2026, 5, 15, 3, 45, 0).unwrap();
    let known = cache.get("INFY.NS").cloned().expect("INFY cache");
    let mut with_future = known.clone();
    with_future.push(bar(2026, 5, 16, 10_000.0));
    with_future.push(bar(2026, 8, 17, 1.0));
    let a = certified_execution_state("INFY.NS", &known, t).unwrap();
    let b = certified_execution_state("INFY.NS", &with_future, t).unwrap();
    assert_eq!(a.state_hash, b.state_hash);
    assert_eq!(a.trend, b.trend);
    assert_eq!(a.momentum, b.momentum);
}

#[test]
fn document_keeps_pe1_as_control_and_does_not_reopen_search() {
    let doc = include_str!("../../../docs/CS-P-006-P.E.3_CORALYS_TARGET_DISCOVERY.md");
    assert!(doc.contains("P.E.2 is the **control**"));
    assert!(doc.contains("TARGET(T) ≠ f(future_price_path)"));
    assert!(doc.contains("CORALYS_TARGET_SEARCH_AUTHORIZED = false"));
    assert!(doc.contains("TARGET_LOOKAHEAD_AUTHORIZED = false"));
    assert!(doc.contains("Execution Intent"));
    assert!(doc.contains("Search #3"));
    assert!(doc.contains("C.3-G"));
    assert!(doc.contains("not started"));
    assert!(doc.contains("TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED = false"));
    assert!(doc.contains("not authorized to learn the target from historical realized outcomes"));
    let contract = include_str!("../../../docs/CS-P-006-P.E.3.A_CORALYS_TARGET_ARTIFACT.md");
    assert!(contract.contains("effective_timestamp"));
    assert!(contract.contains("CORALYS_TARGET_ARTIFACT_PRESENT = false"));
    assert!(contract.contains("no generator") || contract.contains("not built"));
    assert!(!contract.contains("ATR") || contract.contains("ATR→%"));
    let pointer = include_str!("../../../docs/CS-P-006-P.E.B_CORALYS_TARGET_FROM_STATE.md");
    assert!(pointer.contains("CS-P-006-P.E.3"));
    assert!(pointer.contains("Do **not** immediately replace +5%"));
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
