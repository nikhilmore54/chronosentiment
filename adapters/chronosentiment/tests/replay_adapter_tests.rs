use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::policy::BaselineTrendMappingPolicy;
use chronosentiment_adapter::decision_support::replay::{
    decide_from_inputs, observations_at_or_before, ReplayAssessment, ReplayInputs,
    ReplayLakeDecision, ReplayObservation, UNFROZEN_ENGINE_VERSION,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use uuid::Uuid;

fn t0() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
}

fn bullish_profile(
    instrument_id: Uuid,
    dt: chrono::DateTime<Utc>,
) -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(2100.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));
    AssessmentEngine.assess_at(&metrics, &[Concept::Trend], dt, Some(instrument_id))
}

fn bearish_profile(
    instrument_id: Uuid,
    dt: chrono::DateTime<Utc>,
) -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(1900.0));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(2050.0));
    AssessmentEngine.assess_at(&metrics, &[Concept::Trend], dt, Some(instrument_id))
}

fn base_inputs(instrument_id: Uuid) -> ReplayInputs {
    let mut profile = bullish_profile(instrument_id, t0());
    let assessment_id = Uuid::from_u128(1);
    profile.metadata.artifact_id = assessment_id;
    ReplayInputs {
        instrument_id,
        as_of: t0(),
        engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
        produced_by: chronosentiment_adapter::decision_support::replay::REPLAY_PRODUCER.to_string(),
        assessments: vec![ReplayAssessment {
            id: assessment_id,
            evaluation_timestamp: t0(),
            signature_hash: profile.to_hash(),
            profile,
        }],
        lake_decisions: vec![],
        observations: vec![ReplayObservation {
            id: Uuid::from_u128(11),
            effective_from: t0(),
        }],
    }
}

#[test]
fn replay_is_deterministic_across_two_runs() {
    let instrument_id = Uuid::from_u128(7);
    let a = decide_from_inputs(base_inputs(instrument_id), &BaselineTrendMappingPolicy).unwrap();
    let b = decide_from_inputs(base_inputs(instrument_id), &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(a.as_of_timestamp, t0());
    assert_eq!(a.engine_version, UNFROZEN_ENGINE_VERSION);
    assert_eq!(a.action, DecisionAction::Long);
    assert_eq!(a.decision_id, b.decision_id);
    assert_eq!(a.provenance.content_hash, b.provenance.content_hash);
    assert_eq!(a.lineage.input_set_hash, b.lineage.input_set_hash);
    assert!(a.lineage.assessment_id.is_some());
}

#[test]
fn future_observation_is_excluded_from_input_set() {
    let instrument_id = Uuid::from_u128(7);
    let without_future =
        decide_from_inputs(base_inputs(instrument_id), &BaselineTrendMappingPolicy).unwrap();

    let mut with_future = base_inputs(instrument_id);
    with_future.observations.push(ReplayObservation {
        id: Uuid::from_u128(99),
        effective_from: t0() + chrono::Duration::days(1),
    });
    let filtered = observations_at_or_before(&with_future.observations, t0());
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, Uuid::from_u128(11));

    let with_future_decision =
        decide_from_inputs(with_future, &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(without_future.decision_id, with_future_decision.decision_id);
    assert_eq!(
        without_future.provenance.content_hash,
        with_future_decision.provenance.content_hash
    );
    assert!(!with_future_decision
        .lineage
        .consumed_artifact_ids
        .contains(&Uuid::from_u128(99)));
}

#[test]
fn future_assessment_cannot_change_action() {
    let instrument_id = Uuid::from_u128(7);
    let mut inputs = base_inputs(instrument_id);
    let future_dt = t0() + chrono::Duration::days(30);
    let future = bearish_profile(instrument_id, future_dt);
    inputs.assessments.push(ReplayAssessment {
        id: future.metadata.artifact_id,
        evaluation_timestamp: future_dt,
        signature_hash: future.to_hash(),
        profile: future,
    });
    let d = decide_from_inputs(inputs, &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(d.action, DecisionAction::Long);
    assert_eq!(d.as_of_timestamp, t0());
}

#[test]
fn no_trade_when_trend_is_absent() {
    let instrument_id = Uuid::from_u128(8);
    let mut profile = bullish_profile(instrument_id, t0());
    profile.assessments.clear();
    let d = decide_from_inputs(
        ReplayInputs {
            instrument_id,
            as_of: t0(),
            engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
            produced_by: chronosentiment_adapter::decision_support::replay::REPLAY_PRODUCER
                .to_string(),
            assessments: vec![ReplayAssessment {
                id: profile.metadata.artifact_id,
                evaluation_timestamp: t0(),
                signature_hash: "neutral".to_string(),
                profile,
            }],
            lake_decisions: vec![],
            observations: vec![],
        },
        &BaselineTrendMappingPolicy,
    )
    .unwrap();
    assert_eq!(d.action, DecisionAction::NoTrade);
}

#[test]
fn adapter_does_not_copy_assessment_confidence_as_decision_confidence() {
    let instrument_id = Uuid::from_u128(7);
    let d = decide_from_inputs(base_inputs(instrument_id), &BaselineTrendMappingPolicy).unwrap();
    assert_eq!(d.action, DecisionAction::Long);
    assert_eq!(d.confidence, None);
    assert_eq!(
        d.confidence_status,
        chronosentiment_adapter::decision_support::ConfidenceStatus::Unavailable
    );
    assert_eq!(
        d.evidence.mapping_rule,
        chronosentiment_adapter::decision_support::replay::TREND_MAPPING_RULE
    );
    let trend = d
        .evidence
        .factors
        .iter()
        .find(|f| f.concept == "Trend")
        .unwrap();
    assert!(trend.present);
    assert_eq!(trend.assessment_confidence, None);
    let momentum = d
        .evidence
        .factors
        .iter()
        .find(|f| f.concept == "Momentum")
        .unwrap();
    assert!(!momentum.present);
    let vol = d
        .evidence
        .factors
        .iter()
        .find(|f| f.concept == "Volatility")
        .unwrap();
    assert!(!vol.present);
    assert!(d.evidence.diagnostics.contains("UNAVAILABLE"));
    assert!(d.evidence.diagnostics.contains("LONG"));
}

#[test]
fn lake_decision_after_t_is_not_consumed() {
    let instrument_id = Uuid::from_u128(7);
    let mut inputs = base_inputs(instrument_id);
    let future_id = Uuid::from_u128(42);
    inputs.lake_decisions.push(ReplayLakeDecision {
        id: future_id,
        evaluation_timestamp: t0() + chrono::Duration::days(1),
    });
    let d = decide_from_inputs(inputs, &BaselineTrendMappingPolicy).unwrap();
    assert!(!d.lineage.consumed_artifact_ids.contains(&future_id));
}
