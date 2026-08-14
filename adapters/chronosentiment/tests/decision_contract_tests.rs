use chrono::TimeZone;
use chronosentiment_adapter::decision_support::{
    ConfidenceStatus, DecisionAction, DecisionContractError, DecisionDraft, DecisionEvidence,
    DecisionLineage, RiskInformation, RiskLevel, TradingDecision,
};
use uuid::Uuid;

fn as_of() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2023, 7, 31, 15, 30, 0).unwrap()
}

fn draft(action: DecisionAction) -> DecisionDraft {
    DecisionDraft {
        engine_version: "unfrozen-dev".to_string(),
        instrument_id: Uuid::nil(),
        as_of_timestamp: as_of(),
        action,
        confidence: Some(0.68),
        confidence_status: ConfidenceStatus::Available,
        horizon_trading_days: 5,
        rationale: "Contract test".to_string(),
        evidence_refs: vec!["trend".to_string()],
        evidence: DecisionEvidence {
            mapping_rule: "test".to_string(),
            diagnostics: "Contract test".to_string(),
            factors: vec![],
            consumed_concepts: vec![],
        },
        risk: RiskInformation {
            level: RiskLevel::Medium,
            invalidation: Some("Close below X".to_string()),
        },
        lineage: DecisionLineage {
            produced_by: "decision_support.contract".to_string(),
            consumed_artifact_ids: vec![Uuid::from_u128(2), Uuid::from_u128(1)],
            assessment_id: Some(Uuid::from_u128(9)),
            input_set_hash: "abc".to_string(),
        },
    }
}

#[test]
fn no_trade_is_a_first_class_decision() {
    let d = TradingDecision::try_from_draft(draft(DecisionAction::NoTrade)).unwrap();
    assert_eq!(d.action, DecisionAction::NoTrade);
    let json = serde_json::to_value(&d).unwrap();
    assert_eq!(json["action"], "NO_TRADE");
    assert!(json.get("outcome_return").is_none());
}

#[test]
fn same_frozen_inputs_same_engine_same_identity() {
    let a = TradingDecision::try_from_draft(draft(DecisionAction::Long)).unwrap();
    let mut shuffled = draft(DecisionAction::Long);
    shuffled.lineage.consumed_artifact_ids = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
    let b = TradingDecision::try_from_draft(shuffled).unwrap();
    assert_eq!(a.decision_id, b.decision_id);
    assert_eq!(a.provenance.content_hash, b.provenance.content_hash);
    assert_eq!(a.as_of_timestamp, as_of());
}

#[test]
fn engine_version_is_part_of_identity() {
    let a = TradingDecision::try_from_draft(draft(DecisionAction::Long)).unwrap();
    let mut other = draft(DecisionAction::Long);
    other.engine_version = "unfrozen-dev-2".to_string();
    let b = TradingDecision::try_from_draft(other).unwrap();
    assert_ne!(a.decision_id, b.decision_id);
}

#[test]
fn as_of_is_caller_supplied_not_wall_clock() {
    let d = TradingDecision::try_from_draft(draft(DecisionAction::Short)).unwrap();
    assert_eq!(d.as_of_timestamp, as_of());
    assert!(d.as_of_timestamp < chrono::Utc::now() - chrono::Duration::days(365));
}

#[test]
fn rejects_missing_lineage_and_unbounded_confidence() {
    let mut no_lineage = draft(DecisionAction::Long);
    no_lineage.lineage.consumed_artifact_ids.clear();
    no_lineage.lineage.assessment_id = None;
    assert_eq!(
        TradingDecision::try_from_draft(no_lineage).unwrap_err(),
        DecisionContractError::MissingLineage
    );

    let mut bad_p = draft(DecisionAction::Long);
    bad_p.confidence = Some(1.2);
    assert_eq!(
        TradingDecision::try_from_draft(bad_p).unwrap_err(),
        DecisionContractError::ConfidenceOutOfBounds
    );
}

#[test]
fn unavailable_confidence_is_not_a_numeric_score() {
    let mut d = draft(DecisionAction::Long);
    d.confidence = None;
    d.confidence_status = ConfidenceStatus::Unavailable;
    let decision = TradingDecision::try_from_draft(d).unwrap();
    assert_eq!(decision.confidence, None);
    assert_eq!(decision.confidence_status, ConfidenceStatus::Unavailable);
    let json = serde_json::to_value(&decision).unwrap();
    assert!(json["confidence"].is_null());
    assert_eq!(json["confidence_status"], "UNAVAILABLE");
}

#[test]
fn unavailable_is_not_the_same_identity_as_constant_082() {
    let mut honest = draft(DecisionAction::Long);
    honest.confidence = None;
    honest.confidence_status = ConfidenceStatus::Unavailable;
    let mut pretend = draft(DecisionAction::Long);
    pretend.confidence = Some(0.82);
    pretend.confidence_status = ConfidenceStatus::Available;
    let a = TradingDecision::try_from_draft(honest).unwrap();
    let b = TradingDecision::try_from_draft(pretend).unwrap();
    assert_ne!(a.decision_id, b.decision_id);
}

#[test]
fn unavailable_cannot_carry_a_numeric_confidence() {
    let mut d = draft(DecisionAction::Long);
    d.confidence = Some(0.82);
    d.confidence_status = ConfidenceStatus::Unavailable;
    assert_eq!(
        TradingDecision::try_from_draft(d).unwrap_err(),
        DecisionContractError::ConfidenceStatusMismatch
    );
}
