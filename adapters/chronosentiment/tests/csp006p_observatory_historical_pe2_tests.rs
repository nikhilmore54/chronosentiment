//! CS-P-006-P.E.2.H historical time-machine. Does not reopen the P.E.2 spec.

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::observatory_execution::{
    ExitReason, TriggerType, EXECUTION_TARGET_PCT,
};
use chronosentiment_adapter::decision_support::observatory_historical_pe2::{
    refuse_historical_pe2_output, render_historical_pe2_html, render_historical_pe2_report,
    replay_historical_pe2, HISTORICAL_PE2_PATH_KIND, HISTORICAL_PE2_REQUESTED_CLOCK,
};
use chronosentiment_adapter::decision_support::observatory_live_execution::LIVE_EXECUTION_STATUS_AWAITING;
use chronosentiment_adapter::decision_support::DecisionAction;

#[test]
fn refuses_protected_outputs() {
    assert!(refuse_historical_pe2_output(
        "product_validation/CS-P-006/observatory/prospective"
    )
    .is_err());
    assert!(refuse_historical_pe2_output(
        "product_validation/CS-P-006/observatory/prospective_execution_v0"
    )
    .is_err());
    assert!(refuse_historical_pe2_output(
        "product_validation/CS-P-006/observatory/targeted_execution_v0"
    )
    .is_err());
    assert!(refuse_historical_pe2_output(
        "product_validation/CS-P-006/observatory/historical_replay_v0"
    )
    .is_err());
    assert!(refuse_historical_pe2_output(
        "product_validation/CS-P-006/observatory/historical_replay_v1"
    )
    .is_err());
    assert!(refuse_historical_pe2_output(
        "product_validation/CS-P-006/observatory/historical_pe2_replay"
    )
    .is_ok());
}

#[test]
fn july_fifteen_lifecycle_validates_without_lookahead() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let Some(cache) = load_cache() else {
        return;
    };
    let ledger = replay_historical_pe2(&artifact, &cache).unwrap();
    let again = replay_historical_pe2(&artifact, &cache).unwrap();
    assert_eq!(ledger.lifecycle_validation, "PASS");
    assert_eq!(ledger.path_kind, HISTORICAL_PE2_PATH_KIND);
    assert!(ledger.certified_t.starts_with("2026-07-15T03:45:00"));
    assert_eq!(ledger.requested_clock, Utc.with_ymd_and_hms(2026, 7, 15, 3, 45, 0).unwrap().to_rfc3339());
    assert_eq!(ledger.n_decisions, 7);
    assert_eq!(ledger.n_execution_intents, 7);
    assert_eq!(ledger.n_target + ledger.n_horizon, 7);
    assert!(ledger.determinism_pass);
    assert!(ledger.lookahead_clean);
    assert!(ledger.poison_test_pass);
    assert!(!ledger.peeked_returns_at_seal);
    assert!(!ledger.prospective_cohort_mutated);
    assert!(!ledger.protected_artifacts_mutated);
    assert!(!ledger.statistical_backtest);
    assert!((ledger.target_pct - EXECUTION_TARGET_PCT).abs() < 1e-12);
    assert_eq!(ledger.max_holding_sessions, 20);
    assert!(ledger.records.iter().any(|r| r.instrument == "IDEA.NS"));
    assert!(ledger.records.iter().any(|r| r.instrument == "MAHABANK.NS"));
    assert!(ledger.records.iter().all(|r| {
        r.decision.policy_artifact_sha256 == RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
            && r.intent.sealed_at_t
            && (r.intent.target_pct - 0.05).abs() < 1e-12
            && r.intent.max_holding_sessions == 20
            && r.decision.decision_time.starts_with("2026-07-15T03:45:00")
            && r.determinism_pass
            && r.lookahead_clean
            && r.poison_test_pass
            && !matches!(r.exit.exit_reason, ExitReason::Observing)
    }));
    for record in &ledger.records {
        match (record.decision.action, record.exit.exit_reason, record.exit.trigger_type) {
            (DecisionAction::Long, ExitReason::Target, Some(TriggerType::HighReached | TriggerType::GapThrough)) => {}
            (DecisionAction::Short, ExitReason::Target, Some(TriggerType::LowReached | TriggerType::GapThrough)) => {}
            (_, ExitReason::Horizon, Some(TriggerType::SessionClose)) => {
                assert_eq!(record.exit.holding_sessions, Some(20));
            }
            other => panic!("unexpected trigger pairing: {other:?}"),
        }
    }
    assert_eq!(
        ledger.n_gap_through + ledger.n_high_reached + ledger.n_low_reached + ledger.n_session_close,
        7
    );
    assert_eq!(ledger.n_decisions, again.n_decisions);
    assert_eq!(
        ledger.records.iter().map(|r| r.decision.decision_id.clone()).collect::<Vec<_>>(),
        again.records.iter().map(|r| r.decision.decision_id.clone()).collect::<Vec<_>>()
    );
    assert_eq!(
        ledger.records.iter().map(|r| r.intent.intent_hash.clone()).collect::<Vec<_>>(),
        again.records.iter().map(|r| r.intent.intent_hash.clone()).collect::<Vec<_>>()
    );

    let html = render_historical_pe2_html(&ledger);
    assert!(html.contains("Historical P.E.2 lifecycle validation"));
    assert!(html.contains("NOT PERFORMED"));
    assert!(html.contains("AWAITING_NEXT_SESSION"));
    assert!(html.contains("14-Aug live cohort"));
    assert!(!html.contains("Sharpe"));
    assert!(!html.contains("CAGR"));
    let report = render_historical_pe2_report(&ledger);
    assert!(report.contains("Statistical strategy backtest: **NOT PERFORMED**"));
    assert!(report.contains("Historical P.E.2 lifecycle validation: **PASS**"));
    assert!(!report.contains("mean V"));
}

#[test]
fn document_does_not_reopen_pe2_or_start_pe3() {
    let doc = include_str!("../../../docs/CS-P-006-P.E.2.H_HISTORICAL_LIFECYCLE_VALIDATION.md");
    assert!(doc.contains(HISTORICAL_PE2_REQUESTED_CLOCK) || doc.contains("15 Jul 2026"));
    assert!(doc.contains("AWAITING_NEXT_SESSION"));
    assert!(doc.contains("**PASS**"));
    assert!(doc.contains("NOT PERFORMED") || doc.contains("not a statistical"));
    assert!(doc.contains("Old evidence cannot silently become new evidence"));
    let pe2 = include_str!("../../../docs/CS-P-006-P.E.2_LIVE_EXECUTION_OBSERVATION.md");
    assert!(pe2.contains("Frozen"));
    assert!(pe2.contains("AWAITING_NEXT_SESSION"));
    let sidecar = include_str!(
        "../../../product_validation/CS-P-006/observatory/historical_pe2_replay/REPORT.md"
    );
    assert!(sidecar.contains("Historical P.E.2 lifecycle validation: **PASS**"));
    assert!(sidecar.contains("Statistical strategy backtest: **NOT PERFORMED**"));
    assert!(sidecar.contains("2026-07-15T03:45:00"));
    assert!(sidecar.contains("GAP_THROUGH"));
    assert!(sidecar.contains("HIGH_REACHED"));
    assert!(sidecar.contains("LOW_REACHED"));
    assert!(sidecar.contains("SESSION_CLOSE"));
    assert!(sidecar.contains("not product claims"));
    assert!(sidecar.contains("NOT PERFORMED"));
}

#[test]
fn protected_objects_remain_byte_for_byte() {
    assert_eq!(
        sha256("product_validation/CS-P-006/observatory/prospective/ledger.json"),
        "e25d3750b65657ae45d4c994ebe51cdf9396798e090bbc36dbd016d40c24a740"
    );
    assert_eq!(
        sha256("product_validation/CS-P-006/observatory/historical_replay_v0/ledger.json"),
        "06a899a5a606425a7145d679ab48089a799e53f9b70a8a901a3be084aad74cc0"
    );
    assert_eq!(
        sha256("product_validation/CS-P-006/observatory/historical_replay_v1/ledger.json"),
        "dc11ecb7619176f7d0af4ba3abfd5319be7ff0be6b418a5e825199df4fa32b92"
    );
    assert_eq!(
        sha256("product_validation/CS-P-006/observatory/targeted_execution_v0/report.json"),
        "e90d23ca40a283e985bdf37f972b537a925dc71e17b0a05fad0c8227255ec3cb"
    );
    let live = std::fs::read_to_string(
        repo("product_validation/CS-P-006/observatory/prospective_execution_v0/ledger.json"),
    )
    .unwrap();
    let live_json: serde_json::Value = serde_json::from_str(&live).unwrap();
    assert_eq!(live_json["seal_status"], LIVE_EXECUTION_STATUS_AWAITING);
    assert_eq!(live_json["n_decisions"], 0);
    assert_eq!(
        sha256("product_validation/CS-P-006/observatory/prospective_execution_v0/ledger.json"),
        "1d8bffc698ed3862bdcfc04ea26e8239754ade160f3037fd0af094a9ad416b0d"
    );
    assert_eq!(
        sha256("product_validation/CS-P-006/discovery/20260815T051900Z_c3/selected_policy.json"),
        "825063b418486ab9038bef0acea9b8b78204f5ffa6e24e4bd0adaae52b3c460e"
    );
}

fn sha256(rel: &str) -> String {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(repo(rel)).unwrap();
    format!("{:x}", Sha256::digest(&bytes))
}

fn repo(rel: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(rel)
}

fn load_c3_002() -> Option<chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact>
{
    let path = repo(
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
    let cache_dir = repo(
        chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_SNAPSHOT_DIR,
    )
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
