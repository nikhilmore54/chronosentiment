//! Product-mode Decision contract (CS-P-001 deliverable 1, CS-P-002).
//!
//! Both the replay adapter and the live adapter must emit this object.
//! It is not the Knowledge Lake `reasoning::decision::Decision` artifact (B3/B4 persist).
//!
//! ChronoSentiment evaluates a versioned `DecisionPolicy`. It does not discover
//! policies. Coralys is the learning/search system (CS-P-006). CS-P-006-A is the
//! consumption contract (`policy_artifact`); search is not this module.
//!
//! Decision Engine v1.0 is **not frozen**. `engine_version` is caller-supplied.
//! Outcomes do not belong on this object.

pub mod backtest;
pub mod c3_comparison;
pub mod c3_implementation;
pub mod c3_rule_ecology;
pub mod c3_rule_persistence;
pub mod c3_run;
pub mod c3_state_landscape;
pub mod coralys_execution_model;
pub mod csp006_protocol;
pub mod csp006_snapshot;
pub mod csp007_protocol;
pub mod dataset_partition;
pub mod decision_intent;
pub mod decision_value_fitness;
pub mod decision_value_harness;
pub mod decision_value_landscape;
pub mod enrichment_certify;
pub mod execution_feedback;
pub mod factor_availability;
pub mod factor_ecology;
pub mod forward;
pub mod forward_tick;
pub mod lab_context;
pub mod laboratory;
pub mod observation_outcome;
pub mod observation_value;
pub mod observatory_execution;
pub mod observatory_historical;
pub mod observatory_historical_pe2;
pub mod observatory_historical_pe3;
pub mod observatory_live_execution;
pub mod observatory_live_execution_pe3;
pub mod observatory_maturity;
pub mod observatory_prospective;
pub mod observatory_registry;
pub mod observatory_slice;
pub mod outcome;
pub mod performance;
pub mod policy;
pub mod policy_artifact;
pub mod policy_discovery;
pub mod policy_genome;
pub mod policy_handoff;
pub mod policy_search_diagnosis;
pub mod population_ecology;
pub mod portfolio_replay_v0;
pub mod portfolio_replay_v021;
pub mod recommendation_outcome;
pub mod replay;
pub mod search_observability;
pub mod selection_decision_value;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// DTO schema for this object. Independent of engine version.
/// `csp004.decision.1` requires `policy_name` on the decision object.
/// Action mapping of the baseline fixture is unchanged. This is not Decision Engine v1.0.
pub const OBJECT_SCHEMA_VERSION: &str = "csp004.decision.1";

const MIN_HORIZON_DAYS: u32 = 1;
const MAX_HORIZON_DAYS: u32 = 252;

/// First-class action. `NoTrade` is a decision, not the absence of one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionAction {
    Long,
    Short,
    NoTrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskInformation {
    pub level: RiskLevel,
    pub invalidation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionLineage {
    pub produced_by: String,
    pub consumed_artifact_ids: Vec<Uuid>,
    pub assessment_id: Option<Uuid>,
    /// Hash of the information set available at `as_of_timestamp` (inputs ≤ T only).
    pub input_set_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionProvenance {
    pub object_schema_version: String,
    pub content_hash: String,
}

/// Decision-level confidence. Assessment numeric scores are not this field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfidenceStatus {
    #[default]
    Unavailable,
    Available,
}

/// Factor copied from the assessment at T. `assessment_confidence` is not decision confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceFactor {
    pub concept: String,
    pub present: bool,
    pub direction: Option<String>,
    pub strength: Option<String>,
    pub assessment_confidence: Option<f64>,
}

/// Auditable adapter evidence. Not a strategy score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionEvidence {
    pub mapping_rule: String,
    pub diagnostics: String,
    pub factors: Vec<EvidenceFactor>,
    /// Concepts the policy used to choose the action. Unused factors stay on `factors` for audit.
    #[serde(default)]
    pub consumed_concepts: Vec<String>,
}

impl Default for DecisionEvidence {
    fn default() -> Self {
        Self {
            mapping_rule: String::new(),
            diagnostics: String::new(),
            factors: Vec::new(),
            consumed_concepts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradingDecision {
    pub decision_id: Uuid,
    pub engine_version: String,
    pub policy_name: String,
    pub instrument_id: Uuid,
    pub as_of_timestamp: DateTime<Utc>,
    pub action: DecisionAction,
    pub confidence: Option<f64>,
    pub confidence_status: ConfidenceStatus,
    pub horizon_trading_days: u32,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
    pub evidence: DecisionEvidence,
    pub risk: RiskInformation,
    pub lineage: DecisionLineage,
    pub provenance: DecisionProvenance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionDraft {
    pub engine_version: String,
    pub policy_name: String,
    pub instrument_id: Uuid,
    pub as_of_timestamp: DateTime<Utc>,
    pub action: DecisionAction,
    pub confidence: Option<f64>,
    pub confidence_status: ConfidenceStatus,
    pub horizon_trading_days: u32,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
    pub evidence: DecisionEvidence,
    pub risk: RiskInformation,
    pub lineage: DecisionLineage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionContractError {
    EmptyEngineVersion,
    EmptyPolicyName,
    ConfidenceOutOfBounds,
    ConfidenceStatusMismatch,
    HorizonOutOfBounds,
    EmptyProducer,
    EmptyInputSetHash,
    MissingLineage,
}

impl std::fmt::Display for DecisionContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyEngineVersion => write!(f, "engine_version must be non-empty"),
            Self::EmptyPolicyName => write!(f, "policy_name must be non-empty"),
            Self::ConfidenceOutOfBounds => {
                write!(f, "available confidence must be finite and in [0, 1]")
            }
            Self::ConfidenceStatusMismatch => write!(
                f,
                "UNAVAILABLE requires confidence=null; AVAILABLE requires a [0, 1] value"
            ),
            Self::HorizonOutOfBounds => write!(
                f,
                "horizon_trading_days must be in {MIN_HORIZON_DAYS}..={MAX_HORIZON_DAYS}"
            ),
            Self::EmptyProducer => write!(f, "lineage.produced_by must be non-empty"),
            Self::EmptyInputSetHash => write!(f, "lineage.input_set_hash must be non-empty"),
            Self::MissingLineage => {
                write!(
                    f,
                    "lineage requires assessment_id or at least one consumed artifact"
                )
            }
        }
    }
}

impl std::error::Error for DecisionContractError {}

/// Identity is policy consumption, not the full diagnostic evidence blob.
/// Unused factors, outcomes, wall-clock persistence, and fabricated assessment
/// scores are excluded.
#[derive(Serialize)]
struct IdentityPayload<'a> {
    object_schema_version: &'a str,
    engine_version: &'a str,
    policy_name: &'a str,
    instrument_id: Uuid,
    as_of_timestamp: DateTime<Utc>,
    action: DecisionAction,
    confidence: Option<f64>,
    confidence_status: ConfidenceStatus,
    horizon_trading_days: u32,
    rationale: &'a str,
    mapping_rule: &'a str,
    consumed_concepts: &'a [String],
    consumed_factors: &'a [EvidenceFactor],
    risk: &'a RiskInformation,
    produced_by: &'a str,
    assessment_id: Option<Uuid>,
}

impl TradingDecision {
    pub fn try_from_draft(mut draft: DecisionDraft) -> Result<Self, DecisionContractError> {
        draft.lineage.consumed_artifact_ids.sort_unstable();
        draft.evidence_refs.sort();
        draft
            .evidence
            .factors
            .sort_by(|a, b| a.concept.cmp(&b.concept));
        draft.evidence.consumed_concepts.sort();
        validate(&draft)?;

        let consumed_factors: Vec<EvidenceFactor> = draft
            .evidence
            .factors
            .iter()
            .filter(|f| {
                draft
                    .evidence
                    .consumed_concepts
                    .iter()
                    .any(|c| c == &f.concept)
            })
            .map(|f| EvidenceFactor {
                assessment_confidence: None,
                ..f.clone()
            })
            .collect();

        let payload = IdentityPayload {
            object_schema_version: OBJECT_SCHEMA_VERSION,
            engine_version: &draft.engine_version,
            policy_name: &draft.policy_name,
            instrument_id: draft.instrument_id,
            as_of_timestamp: draft.as_of_timestamp,
            action: draft.action,
            confidence: draft.confidence,
            confidence_status: draft.confidence_status,
            horizon_trading_days: draft.horizon_trading_days,
            rationale: &draft.rationale,
            mapping_rule: &draft.evidence.mapping_rule,
            consumed_concepts: &draft.evidence.consumed_concepts,
            consumed_factors: &consumed_factors,
            risk: &draft.risk,
            produced_by: &draft.lineage.produced_by,
            assessment_id: draft.lineage.assessment_id,
        };
        let bytes = serde_json::to_vec(&payload).expect("decision identity payload serializes");
        let digest = Sha256::digest(&bytes);
        let content_hash = format!("{digest:x}");
        let mut id_bytes = [0u8; 16];
        id_bytes.copy_from_slice(&digest[..16]);
        let decision_id = Uuid::from_bytes(id_bytes);

        Ok(Self {
            decision_id,
            engine_version: draft.engine_version,
            policy_name: draft.policy_name,
            instrument_id: draft.instrument_id,
            as_of_timestamp: draft.as_of_timestamp,
            action: draft.action,
            confidence: draft.confidence,
            confidence_status: draft.confidence_status,
            horizon_trading_days: draft.horizon_trading_days,
            rationale: draft.rationale,
            evidence_refs: draft.evidence_refs,
            evidence: draft.evidence,
            risk: draft.risk,
            lineage: draft.lineage,
            provenance: DecisionProvenance {
                object_schema_version: OBJECT_SCHEMA_VERSION.to_string(),
                content_hash,
            },
        })
    }
}

fn validate(draft: &DecisionDraft) -> Result<(), DecisionContractError> {
    if draft.engine_version.trim().is_empty() {
        return Err(DecisionContractError::EmptyEngineVersion);
    }
    if draft.policy_name.trim().is_empty() {
        return Err(DecisionContractError::EmptyPolicyName);
    }
    match (draft.confidence_status, draft.confidence) {
        (ConfidenceStatus::Unavailable, None) => {}
        (ConfidenceStatus::Available, Some(x)) if x.is_finite() && (0.0..=1.0).contains(&x) => {}
        (ConfidenceStatus::Available, Some(_)) => {
            return Err(DecisionContractError::ConfidenceOutOfBounds)
        }
        _ => return Err(DecisionContractError::ConfidenceStatusMismatch),
    }
    if !(MIN_HORIZON_DAYS..=MAX_HORIZON_DAYS).contains(&draft.horizon_trading_days) {
        return Err(DecisionContractError::HorizonOutOfBounds);
    }
    if draft.lineage.produced_by.trim().is_empty() {
        return Err(DecisionContractError::EmptyProducer);
    }
    if draft.lineage.input_set_hash.trim().is_empty() {
        return Err(DecisionContractError::EmptyInputSetHash);
    }
    if draft.lineage.assessment_id.is_none() && draft.lineage.consumed_artifact_ids.is_empty() {
        return Err(DecisionContractError::MissingLineage);
    }
    Ok(())
}
