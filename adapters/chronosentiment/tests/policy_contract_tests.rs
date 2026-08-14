//! CS-P-CLEAN-002 — explicit policy contract.
//!
//! ChronoSentiment cannot emit a product TradingDecision without a named policy.
//! BaselineTrendMappingPolicy is a fixture, not a silent default.

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::policy::{
    BaselineTrendMappingPolicy, DecisionPolicy, PolicyDecision, BASELINE_TREND_MAPPING_POLICY_NAME,
};
use chronosentiment_adapter::decision_support::replay::{
    decide_from_inputs, ReplayAssessment, ReplayInputs, UNFROZEN_ENGINE_VERSION,
};
use chronosentiment_adapter::decision_support::{
    ConfidenceStatus, DecisionAction, DecisionContractError, DecisionDraft, DecisionEvidence,
    DecisionLineage, EvidenceFactor, RiskInformation, RiskLevel, TradingDecision,
};
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::{AssessmentEngine, AssessmentProfile};
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use uuid::Uuid;

fn t() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
}

fn bullish_profile() -> AssessmentProfile {
    let mut metrics = MetricReport::default();
    metrics.metrics.insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics.metrics.insert("ma_50".to_string(), MetricValue::Float(2050.0));
    AssessmentEngine.assess_at(&metrics, &[Concept::Trend], t(), Some(Uuid::from_u128(7)))
}

fn inputs() -> ReplayInputs {
    let mut profile = bullish_profile();
    let id = Uuid::from_u128(1);
    profile.metadata.artifact_id = id;
    ReplayInputs {
        instrument_id: Uuid::from_u128(7),
        as_of: t(),
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        produced_by: chronosentiment_adapter::decision_support::replay::REPLAY_PRODUCER.to_string(),
        assessments: vec![ReplayAssessment {
            id,
            evaluation_timestamp: t(),
            signature_hash: profile.to_hash(),
            profile,
        }],
        lake_decisions: vec![],
        observations: vec![],
    }
}

struct NamedNoTradeA;
struct NamedNoTradeB;

fn no_trade_decision(name: &'static str) -> PolicyDecision {
    PolicyDecision {
        action: DecisionAction::NoTrade,
        mapping_rule: "always NO_TRADE".to_string(),
        action_reason: "fixture NO_TRADE".to_string(),
        diagnostics: format!("Policy={name}"),
        evidence_refs: vec![name.to_string()],
        factors: vec![],
        consumed_concepts: vec![],
    }
}

impl DecisionPolicy for NamedNoTradeA {
    fn name(&self) -> &'static str {
        "test.fixture_a.v0"
    }
    fn decide(&self, _: &AssessmentProfile, _: chrono::DateTime<Utc>) -> PolicyDecision {
        no_trade_decision(self.name())
    }
}

impl DecisionPolicy for NamedNoTradeB {
    fn name(&self) -> &'static str {
        "test.fixture_b.v0"
    }
    fn decide(&self, _: &AssessmentProfile, _: chrono::DateTime<Utc>) -> PolicyDecision {
        no_trade_decision(self.name())
    }
}

#[test]
fn explicit_baseline_is_named_and_reproducible() {
    let a = decide_from_inputs(inputs(), &BaselineTrendMappingPolicy).unwrap();
    let b = decide_from_inputs(inputs(), &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(a.policy_name, BASELINE_TREND_MAPPING_POLICY_NAME);
    assert_eq!(a.decision_id, b.decision_id);
    assert_eq!(a.as_of_timestamp, t());
    assert_eq!(a.confidence, None);
    assert_eq!(a.confidence_status, ConfidenceStatus::Unavailable);
}

#[test]
fn different_policy_names_change_identity_even_when_action_matches() {
    let a = decide_from_inputs(inputs(), &NamedNoTradeA).unwrap();
    let b = decide_from_inputs(inputs(), &NamedNoTradeB).unwrap();
    assert_eq!(a.action, DecisionAction::NoTrade);
    assert_eq!(b.action, DecisionAction::NoTrade);
    assert_ne!(a.policy_name, b.policy_name);
    assert_ne!(a.decision_id, b.decision_id);
    assert_ne!(a.provenance.content_hash, b.provenance.content_hash);
}

#[test]
fn empty_policy_name_is_rejected() {
    let draft = DecisionDraft {
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        policy_name: "   ".to_string(),
        instrument_id: Uuid::from_u128(7),
        as_of_timestamp: t(),
        action: DecisionAction::Long,
        confidence: None,
        confidence_status: ConfidenceStatus::Unavailable,
        horizon_trading_days: 5,
        rationale: "x".into(),
        evidence_refs: vec![],
        evidence: DecisionEvidence::default(),
        risk: RiskInformation {
            level: RiskLevel::Medium,
            invalidation: None,
        },
        lineage: DecisionLineage {
            produced_by: "test".into(),
            consumed_artifact_ids: vec![Uuid::from_u128(1)],
            assessment_id: Some(Uuid::from_u128(1)),
            input_set_hash: "abc".into(),
        },
    };
    assert_eq!(
        TradingDecision::try_from_draft(draft).unwrap_err(),
        DecisionContractError::EmptyPolicyName
    );
}

#[test]
fn assessment_confidence_does_not_enter_identity_or_decision_confidence() {
    let d = decide_from_inputs(inputs(), &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(d.confidence, None);
    assert_eq!(d.confidence_status, ConfidenceStatus::Unavailable);
    for f in &d.evidence.factors {
        assert_eq!(f.assessment_confidence, None);
    }

    let mut with_score = DecisionDraft {
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        policy_name: BASELINE_TREND_MAPPING_POLICY_NAME.to_string(),
        instrument_id: Uuid::from_u128(7),
        as_of_timestamp: t(),
        action: DecisionAction::Long,
        confidence: None,
        confidence_status: ConfidenceStatus::Unavailable,
        horizon_trading_days: 5,
        rationale: "Trend Bullish → LONG by mapping_rule".into(),
        evidence_refs: vec!["Trend".into()],
        evidence: DecisionEvidence {
            mapping_rule: "test".into(),
            diagnostics: "x".into(),
            consumed_concepts: vec!["Trend".into()],
            factors: vec![EvidenceFactor {
                concept: "Trend".into(),
                present: true,
                direction: Some("Bullish".into()),
                strength: None,
                assessment_confidence: Some(0.82),
            }],
        },
        risk: RiskInformation {
            level: RiskLevel::Medium,
            invalidation: None,
        },
        lineage: DecisionLineage {
            produced_by: "test".into(),
            consumed_artifact_ids: vec![Uuid::from_u128(1)],
            assessment_id: Some(Uuid::from_u128(1)),
            input_set_hash: "abc".into(),
        },
    };
    let honest = TradingDecision::try_from_draft(with_score.clone()).unwrap();
    with_score.evidence.factors[0].assessment_confidence = Some(0.11);
    let mutated = TradingDecision::try_from_draft(with_score).unwrap();
    assert_eq!(honest.decision_id, mutated.decision_id);
}

#[test]
fn future_inputs_cannot_change_an_explicit_policy_decision() {
    let base = decide_from_inputs(inputs(), &BaselineTrendMappingPolicy).unwrap();
    let mut later = inputs();
    later.observations.push(
        chronosentiment_adapter::decision_support::replay::ReplayObservation {
            id: Uuid::from_u128(99),
            effective_from: t() + chrono::Duration::days(1),
        },
    );
    let attacked = decide_from_inputs(later, &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(base.decision_id, attacked.decision_id);
    assert_eq!(base.action, attacked.action);
}

#[test]
fn replay_source_still_cannot_select_outcomes() {
    let src = include_str!("../src/decision_support/replay.rs");
    assert!(!src.contains("FROM knowledge_outcomes"));
    assert!(!src.contains("fn decide_from_inputs(inputs: ReplayInputs)"));
    assert!(src.contains("policy: &dyn DecisionPolicy") || src.contains("policy: &P"));
}
