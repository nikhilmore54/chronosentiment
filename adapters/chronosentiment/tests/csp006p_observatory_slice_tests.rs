//! CS-P-006-P.3–P.6 — sealed-then-measured path. No Search #3. No C.3-G.

use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::observatory_registry::{
    CANDIDATE_C3_002, OBSERVATORY_VERTICAL_SLICE_STARTED,
};
use chronosentiment_adapter::decision_support::observatory_slice::{
    append_observation, empty_ledger, generate_decision, measure_decision_value, observe_outcome,
    render_observatory_html, seal_into_ledger, ui_decision_status, SealedDecisionRecord,
    OBSERVATION_STATUS_COMPLETED, OBSERVATORY_P7_STARTED, OBSERVATORY_PROSPECTIVE_STARTED,
    UI_STATUS_OBSERVED, UI_STATUS_OBSERVING,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::DecisionAction;

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
fn vertical_slice_is_started_and_stays_paper_only() {
    assert!(OBSERVATORY_VERTICAL_SLICE_STARTED);
    assert!(OBSERVATORY_P7_STARTED);
    assert!(OBSERVATORY_PROSPECTIVE_STARTED);
}

#[test]
fn sealed_record_has_two_identities_and_no_outcome_fields() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let decision = generate_decision(
        &artifact,
        "INFY.NS",
        "2024-06-30T15:30:00Z",
        "Bullish",
        "Positive",
        "present",
    )
    .unwrap();
    assert_eq!(decision.policy_id, CANDIDATE_C3_002);
    assert_eq!(
        decision.policy_artifact_sha256,
        RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
    );
    assert_eq!(decision.engine_version, "unfrozen-dev");
    assert_eq!(decision.horizon_days, 20);
    assert_eq!(decision.sealed_status, "OPEN");
    assert!(decision.paper_only);
    assert_eq!(decision.action, DecisionAction::Long);
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
    let again = generate_decision(
        &artifact,
        "INFY.NS",
        "2024-06-30T15:30:00Z",
        "Bullish",
        "Positive",
        "present",
    )
    .unwrap();
    assert_eq!(decision.decision_id, again.decision_id);
}

#[test]
fn bullish_negative_is_short_and_bearish_is_long() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let short = generate_decision(
        &artifact,
        "RELIANCE.NS",
        "2024-01-31T15:30:00Z",
        "Bullish",
        "Negative",
        "present",
    )
    .unwrap();
    assert_eq!(short.action, DecisionAction::Short);
    let long = generate_decision(
        &artifact,
        "TCS.NS",
        "2024-01-31T15:30:00Z",
        "Bearish",
        "Negative",
        "present",
    )
    .unwrap();
    assert_eq!(long.action, DecisionAction::Long);
}

#[test]
fn observation_is_append_only_and_does_not_rewrite_the_decision() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let decision = generate_decision(
        &artifact,
        "TCS.NS",
        "2024-03-31T15:30:00Z",
        "Bullish",
        "Negative",
        "present",
    )
    .unwrap();
    let before = serde_json::to_string(&decision).unwrap();
    let observation = observe_outcome(&decision, "2024-04-20T15:30:00Z", 0.0214).unwrap();
    let after: SealedDecisionRecord = serde_json::from_str(&before).unwrap();
    assert_eq!(decision, after);
    let measure = measure_decision_value(&decision, &observation).unwrap();
    assert!((measure.recommended_value + 0.0214).abs() < 1e-12);
    assert!(measure.decided_before_outcome);
    let mut ledger = empty_ledger();
    seal_into_ledger(&mut ledger, decision.clone()).unwrap();
    append_observation(&mut ledger, observation.clone()).unwrap();
    assert!(append_observation(&mut ledger, observation).is_err());
    assert_eq!(ledger.decisions[0].sealed_status, "OPEN");
    assert_eq!(
        ledger.observations[0].observation_status,
        OBSERVATION_STATUS_COMPLETED
    );
    assert_eq!(
        ui_decision_status(&ledger, &decision.decision_id),
        UI_STATUS_OBSERVED
    );
}

#[test]
fn source_does_not_start_search_three_or_a_regime_detector() {
    let src = include_str!("../src/decision_support/observatory_slice.rs");
    let bin = include_str!("../src/bin/csp006_p_observatory.rs");
    for text in [src, bin] {
        assert!(!text.contains("evolve_"));
        assert!(!text.contains("Strategy v2"));
        assert!(!text.contains("b5_strategy"));
        assert!(!text.contains("regime_detector"));
    }
}

#[test]
fn document_records_the_vertical_slice() {
    let doc = include_str!("../../../docs/CS-P-006-P_DECISION_OBSERVATORY.md");
    assert!(doc.contains("P.3"));
    assert!(doc.contains("P.7"));
    assert!(doc.contains("immutable"));
    assert!(doc.contains("Candidate C3-002"));
    assert!(doc.contains("OBSERVED"));
    assert!(
        doc.contains("C.3-G remains a question") || doc.contains("C.3-G remains an unanswered")
    );
}

#[test]
fn p7_ui_exposes_observed_not_sealed_open_derived_completed() {
    let Some(artifact) = load_c3_002() else {
        return;
    };
    let observing = generate_decision(
        &artifact,
        "INFY.NS",
        "2026-08-15T10:00:00Z",
        "Bullish",
        "Positive",
        "present",
    )
    .unwrap();
    let observed = generate_decision(
        &artifact,
        "MAHABANK.NS",
        "2024-12-31T15:30:00Z",
        "Bullish",
        "Negative",
        "present",
    )
    .unwrap();
    let mut ledger = empty_ledger();
    seal_into_ledger(&mut ledger, observing.clone()).unwrap();
    seal_into_ledger(&mut ledger, observed.clone()).unwrap();
    let observation = observe_outcome(&observed, "2025-01-20T15:30:00Z", -0.0161).unwrap();
    append_observation(&mut ledger, observation).unwrap();
    assert_eq!(
        ui_decision_status(&ledger, &observing.decision_id),
        UI_STATUS_OBSERVING
    );
    assert_eq!(
        ui_decision_status(&ledger, &observed.decision_id),
        UI_STATUS_OBSERVED
    );
    assert_eq!(ledger.decisions[1].sealed_status, "OPEN");
    let html = render_observatory_html(&ledger, chrono::Utc::now());
    assert!(html.contains("Decision status"));
    assert!(html.contains("OBSERVED"));
    assert!(html.contains("OBSERVING"));
    assert!(html.contains("id=\"observatory\""));
    assert!(html.contains("id=\"feed\""));
    assert!(html.contains("id=\"policy\""));
    assert!(html.contains("MAHABANK.NS"));
    assert!(html.contains("IDEA") || html.contains("heterogeneous"));
    assert!(!html.contains("sealed status OPEN · derived COMPLETED"));
    assert!(!html.contains("Strategy v2"));
    assert!(!html.contains("Future unknown"));
    assert!(!html.contains("future unknown"));
    assert!(html.contains("Outcome not yet observed"));
    assert!(html.contains("No early peek. No retrospective edits."));
    assert!(html.contains("20 market sessions") || html.contains("market session"));
    assert!(html.contains("without access to information after T"));
    assert!(html.contains("backtesting mechanism"));
    assert!(html.contains("not yet a statistical strategy backtest"));
    assert!(html.contains("Replay integrity is not strategy validation"));
    assert!(html.contains("Certified TMV"));
    assert!(html.contains("Time periods"));
    assert!(html.contains("+V"));
    assert!(!html.contains(">Mean V<"));
}
