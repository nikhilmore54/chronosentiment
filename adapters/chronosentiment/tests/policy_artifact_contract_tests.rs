//! CS-P-006-A — Policy Artifact consumption contract.
//!
//! Fixtures prove the evaluator. They are not Coralys-discovered candidates
//! and must not be promoted as the ChronoSentiment strategy.

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::policy_artifact::{
    certified_factor_definitions, certified_input_schema, ArtifactDecisionPolicy, DecisionRule,
    FactorPredicate, PolicyArtifact, PolicyArtifactError, TrainingProvenance,
    CONTRACT_FIXTURE_ENGINE, CONTRACT_FIXTURE_METHODOLOGY, POLICY_ARTIFACT_SCHEMA_VERSION,
};
use chronosentiment_adapter::decision_support::replay::{
    decide_from_inputs, ReplayAssessment, ReplayInputs, UNFROZEN_ENGINE_VERSION,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::metrics::concepts::Concept;
use chronosentiment_adapter::reasoning::assessment::AssessmentEngine;
use coralys_moga::runtime::optimization::metric::{MetricReport, MetricValue};
use std::fs;
use uuid::Uuid;

fn t() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 10, 31, 15, 30, 0).unwrap()
}

fn profile(
    ma20: f64,
    ma50: f64,
    roc: f64,
    atr: Option<f64>,
) -> chronosentiment_adapter::reasoning::assessment::AssessmentProfile {
    let mut metrics = MetricReport::default();
    metrics
        .metrics
        .insert("ma_20".to_string(), MetricValue::Float(ma20));
    metrics
        .metrics
        .insert("ma_50".to_string(), MetricValue::Float(ma50));
    metrics
        .metrics
        .insert("roc_20".to_string(), MetricValue::Float(roc));
    if let Some(v) = atr {
        metrics
            .metrics
            .insert("atr_14".to_string(), MetricValue::Float(v));
    }
    AssessmentEngine.assess_at(
        &metrics,
        &[Concept::Trend, Concept::Momentum, Concept::Volatility],
        t(),
        Some(Uuid::from_u128(7)),
    )
}

fn fixture_draft() -> PolicyArtifact {
    PolicyArtifact {
        schema_version: POLICY_ARTIFACT_SCHEMA_VERSION.to_string(),
        policy_id: "csp006a.test".to_string(),
        policy_version: "v0".to_string(),
        discovery_engine: CONTRACT_FIXTURE_ENGINE.to_string(),
        discovery_run_id: "csp006a.contract".to_string(),
        input_schema: certified_input_schema(),
        factor_definitions: certified_factor_definitions(),
        action_space: vec![
            DecisionAction::Long,
            DecisionAction::Short,
            DecisionAction::NoTrade,
        ],
        rules: vec![],
        unmatched_action: DecisionAction::NoTrade,
        training_provenance: TrainingProvenance {
            protocol_document_id: "CS-P-006-B-pending".to_string(),
            train: None,
            validation: None,
            test: None,
        },
        allowed_information_timestamp: Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap(),
        artifact_hash: String::new(),
        methodology_hash: CONTRACT_FIXTURE_METHODOLOGY.to_string(),
    }
}

fn replay_inputs(
    profile: chronosentiment_adapter::reasoning::assessment::AssessmentProfile,
) -> ReplayInputs {
    let id = Uuid::from_u128(1);
    let mut profile = profile;
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

fn policy_from(draft: PolicyArtifact) -> ArtifactDecisionPolicy {
    ArtifactDecisionPolicy::try_from_artifact(draft).expect("seal contract fixture")
}

#[test]
fn empty_rules_emit_unmatched_no_trade() {
    let policy = policy_from(fixture_draft());
    let decision = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    assert_eq!(decision.action, DecisionAction::NoTrade);
    assert_eq!(decision.policy_name, "csp006a.test@v0");
    assert_eq!(
        decision.confidence_status,
        chronosentiment_adapter::decision_support::ConfidenceStatus::Unavailable
    );
    assert!(decision.confidence.is_none());
    assert!(decision
        .evidence
        .mapping_rule
        .contains(&policy.artifact().artifact_hash));
}

#[test]
fn checked_in_empty_rules_fixture_seals() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/contracts/csp006a.policy_artifact.empty_rules.json");
    let raw = fs::read_to_string(&path).expect("fixture present");
    let draft: PolicyArtifact = serde_json::from_str(&raw).expect("fixture parses");
    let sealed = draft.seal().expect("empty-rules fixture seals");
    assert!(sealed.rules.is_empty());
    assert_eq!(sealed.unmatched_action, DecisionAction::NoTrade);
    assert_eq!(sealed.discovery_engine, CONTRACT_FIXTURE_ENGINE);
    assert!(!sealed.artifact_hash.is_empty());
}

#[test]
fn explicit_rule_can_emit_long() {
    let mut draft = fixture_draft();
    draft.policy_id = "csp006a.match_long".to_string();
    draft.rules = vec![DecisionRule {
        when: vec![FactorPredicate {
            concept: "Trend".to_string(),
            present: Some(true),
            direction: Some("Bullish".to_string()),
        }],
        action: DecisionAction::Long,
    }];
    let policy = policy_from(draft);
    let bullish = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    assert_eq!(bullish.action, DecisionAction::Long);
    assert_eq!(
        bullish.evidence.consumed_concepts,
        vec!["Trend".to_string()]
    );
    let bearish = decide_from_inputs(
        replay_inputs(profile(2000.0, 2050.0, 1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    assert_eq!(bearish.action, DecisionAction::NoTrade);
}

#[test]
fn conjunction_can_represent_no_trade_without_prescribing_it() {
    let mut draft = fixture_draft();
    draft.policy_id = "csp006a.represent_selectivity".to_string();
    draft.rules = vec![DecisionRule {
        when: vec![
            FactorPredicate {
                concept: "Trend".to_string(),
                present: Some(true),
                direction: Some("Bullish".to_string()),
            },
            FactorPredicate {
                concept: "Momentum".to_string(),
                present: Some(true),
                direction: Some("Negative".to_string()),
            },
        ],
        action: DecisionAction::NoTrade,
    }];
    let policy = policy_from(draft);
    let mixed = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, -1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    assert_eq!(mixed.action, DecisionAction::NoTrade);
    assert_eq!(
        mixed.evidence.consumed_concepts,
        vec!["Momentum".to_string(), "Trend".to_string()]
    );
}

#[test]
fn first_matching_rule_wins() {
    let mut draft = fixture_draft();
    draft.rules = vec![
        DecisionRule {
            when: vec![FactorPredicate {
                concept: "Trend".to_string(),
                present: Some(true),
                direction: Some("Bullish".to_string()),
            }],
            action: DecisionAction::NoTrade,
        },
        DecisionRule {
            when: vec![FactorPredicate {
                concept: "Trend".to_string(),
                present: Some(true),
                direction: Some("Bullish".to_string()),
            }],
            action: DecisionAction::Long,
        },
    ];
    let policy = policy_from(draft);
    let decision = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    assert_eq!(decision.action, DecisionAction::NoTrade);
}

#[test]
fn same_artifact_and_state_are_deterministic() {
    let policy = policy_from(fixture_draft());
    let a = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    let b = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    assert_eq!(a.decision_id, b.decision_id);
    assert_eq!(a.provenance.content_hash, b.provenance.content_hash);
}

#[test]
fn different_rules_change_artifact_hash_and_decision_identity() {
    let empty = policy_from(fixture_draft());
    let mut with_rule = fixture_draft();
    with_rule.rules = vec![DecisionRule {
        when: vec![FactorPredicate {
            concept: "Trend".to_string(),
            present: Some(true),
            direction: Some("Bullish".to_string()),
        }],
        action: DecisionAction::Long,
    }];
    let with_rule = policy_from(with_rule);
    assert_ne!(
        empty.artifact().artifact_hash,
        with_rule.artifact().artifact_hash
    );
    let d0 = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &empty,
    )
    .unwrap();
    let d1 = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &with_rule,
    )
    .unwrap();
    assert_ne!(d0.decision_id, d1.decision_id);
}

#[test]
fn handwritten_and_grid_engines_are_rejected() {
    for engine in ["chronosentiment.handwritten", "threshold.grid", "manual"] {
        let mut draft = fixture_draft();
        draft.discovery_engine = engine.to_string();
        assert_eq!(
            draft.seal().unwrap_err(),
            PolicyArtifactError::ForbiddenEngine,
            "{engine}"
        );
    }
}

#[test]
fn coralys_engine_without_windows_is_rejected() {
    let mut draft = fixture_draft();
    draft.discovery_engine = "coralys.moga".to_string();
    assert_eq!(
        draft.seal().unwrap_err(),
        PolicyArtifactError::DiscoveredWithoutWindows
    );
}

#[test]
fn incomplete_windows_are_rejected() {
    let mut draft = fixture_draft();
    draft.discovery_engine = "coralys.moga".to_string();
    draft.training_provenance.train = Some(
        chronosentiment_adapter::decision_support::policy_artifact::SplitWindow {
            inclusive_start: Utc.with_ymd_and_hms(1990, 1, 1, 0, 0, 0).unwrap(),
            exclusive_end: Utc.with_ymd_and_hms(1991, 1, 1, 0, 0, 0).unwrap(),
        },
    );
    assert_eq!(
        draft.seal().unwrap_err(),
        PolicyArtifactError::WindowIncomplete
    );
}

#[test]
fn missing_no_trade_in_action_space_is_rejected() {
    let mut draft = fixture_draft();
    draft.action_space = vec![DecisionAction::Long, DecisionAction::Short];
    assert_eq!(
        draft.seal().unwrap_err(),
        PolicyArtifactError::IncompleteActionSpace
    );
}

#[test]
fn volatility_direction_predicate_is_rejected() {
    let mut draft = fixture_draft();
    draft.rules = vec![DecisionRule {
        when: vec![FactorPredicate {
            concept: "Volatility".to_string(),
            present: Some(true),
            direction: Some("High".to_string()),
        }],
        action: DecisionAction::NoTrade,
    }];
    assert_eq!(
        draft.seal().unwrap_err(),
        PolicyArtifactError::VolatilityDirectionForbidden
    );
}

#[test]
fn hash_mismatch_is_rejected() {
    let mut sealed = fixture_draft().seal().unwrap();
    sealed.artifact_hash = "deadbeef".to_string();
    assert_eq!(
        ArtifactDecisionPolicy::try_from_artifact(sealed).unwrap_err(),
        PolicyArtifactError::HashMismatch
    );
}

#[test]
fn fabricated_assessment_confidence_is_not_copied() {
    let policy = policy_from(fixture_draft());
    let decision = decide_from_inputs(
        replay_inputs(profile(2100.0, 2050.0, 1.0, Some(2.0))),
        &policy,
    )
    .unwrap();
    for factor in &decision.evidence.factors {
        assert!(
            factor.assessment_confidence.is_none(),
            "{} leaked assessment_confidence",
            factor.concept
        );
    }
}

#[test]
fn artifact_struct_has_no_outcome_fields() {
    let src = include_str!("../src/decision_support/policy_artifact.rs");
    let lower = src.to_lowercase();
    assert!(!lower.contains("outcome_id"));
    assert!(!lower.contains("knowledge_outcomes"));
    assert!(!lower.contains("measure_performance"));
    assert!(!src.contains("EvolutionEngine"));
    assert!(!src.contains("FitnessEvaluator"));
    assert!(!src.contains("rand::"));
}

#[test]
fn product_decide_signature_still_excludes_outcomes() {
    let replay = include_str!("../src/decision_support/replay.rs");
    assert!(replay.contains("policy.decide(&latest.profile, t)"));
    assert!(!replay.contains("policy.decide(&latest.profile, t,"));
}
