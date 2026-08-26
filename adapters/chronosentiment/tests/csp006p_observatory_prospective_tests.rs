//! CS-P-006-P prospective C3-002. No outcomes at seal. No Search #3. No C.3-G.

use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR, RESEARCH_SNAPSHOT_DIR,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_prospective::{
    empty_prospective_ledger, generate_prospective_decision, seal_prospective,
    PROSPECTIVE_NOT_CSP003_VALIDATION, PROSPECTIVE_PATH_KIND,
};
use chronosentiment_adapter::decision_support::observatory_slice::{
    observe_outcome, render_product_html, ui_decision_status, OBSERVATORY_PROSPECTIVE_STARTED,
    UI_STATUS_OBSERVING,
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

#[test]
fn prospective_is_started_and_is_not_csp003_validation() {
    assert!(OBSERVATORY_PROSPECTIVE_STARTED);
    assert!(PROSPECTIVE_NOT_CSP003_VALIDATION);
}

#[test]
fn prospective_seal_has_no_outcome_and_status_is_observing() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let cache_dir = workspace_root()
        .join(RESEARCH_SNAPSHOT_DIR)
        .join("yahoo_cache");
    if !cache_dir.exists() {
        return;
    }
    let cache = load_required_yahoo_cache(&cache_dir).unwrap();
    let bars = cache.get("INFY.NS").unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 15, 12, 0, 0).unwrap();
    let decision = generate_prospective_decision(&artifact, "INFY.NS", bars, now).unwrap();
    assert_eq!(decision.policy_id, "C3-002");
    assert_eq!(
        decision.policy_artifact_sha256,
        RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
    );
    assert_eq!(decision.sealed_status, "OPEN");
    assert!(decision.paper_only);
    let json = serde_json::to_string(&decision).unwrap();
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
    let mut ledger = empty_prospective_ledger();
    assert_eq!(ledger.path_kind, PROSPECTIVE_PATH_KIND);
    assert!(seal_prospective(&mut ledger, decision.clone()).unwrap());
    assert!(!seal_prospective(&mut ledger, decision.clone()).unwrap());
    assert!(ledger.observations.is_empty());
    assert_eq!(
        ui_decision_status(&ledger, &decision.decision_id),
        UI_STATUS_OBSERVING
    );
    let observation = observe_outcome(&decision, "2026-09-04T12:00:00Z", 0.01).unwrap();
    assert!(
        chronosentiment_adapter::decision_support::observatory_slice::append_observation(
            &mut ledger,
            observation
        )
        .is_ok()
    );
    assert!(seal_prospective(&mut ledger, decision).is_err());
}

#[test]
fn product_html_keeps_historical_pass_separate_from_profit() {
    let historical_path =
        workspace_root().join("product_validation/CS-P-006/observatory/ledger.json");
    if !historical_path.exists() {
        return;
    }
    let historical: chronosentiment_adapter::decision_support::observatory_slice::ObservatoryLedger =
        serde_json::from_str(&std::fs::read_to_string(historical_path).unwrap()).unwrap();
    let html = render_product_html(&historical, None, chrono::Utc::now());
    assert!(html.contains("lifecycle PASS"));
    assert!(html.contains("not a profitability claim"));
    assert!(html.contains("OBSERVED"));
    assert!(!html.contains("sealed status OPEN · derived COMPLETED"));
}

#[test]
fn source_does_not_start_search_three_or_attach_outcomes() {
    let src = include_str!("../src/decision_support/observatory_prospective.rs");
    let bin = include_str!("../src/bin/csp006_p_prospective.rs");
    for text in [src, bin] {
        assert!(!text.contains("evolve_"));
        assert!(!text.contains("Strategy v2"));
        assert!(!text.contains("regime_detector"));
        assert!(!text.contains("observe_outcome"));
    }
}

#[test]
fn document_records_the_lifecycle_pass_and_prospective_gate() {
    let doc = include_str!("../../../docs/CS-P-006-P_DECISION_OBSERVATORY.md");
    assert!(doc.contains("lifecycle PASS") || doc.contains("91 / 91"));
    assert!(doc.contains("not a profitability claim") || doc.contains("Not a profitability claim"));
    assert!(doc.contains("prospective"));
    assert!(
        doc.contains("C.3-G remains a question") || doc.contains("C.3-G remains an unanswered")
    );
}
