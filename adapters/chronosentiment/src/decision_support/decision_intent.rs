//! CS-P-006-P.E.3 pairing helper: Decision stays C3-002; execution intent is separate.
//!
//! P.E.2 freezes the fixed +5% execution intent as the control. P.E.3 (not started)
//! would attach a Coralys-derived execution intent. This module does not search,
//! does not peek after T, and does not rewrite P.E.1 or P.E.2.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use super::enrichment_certify::{assess_from_bars_at_t, bars_at_or_before};
use super::forward_tick::instrument_id_for;
use super::observatory_execution::{
    EXECUTION_CONTRACT_ID, EXECUTION_TARGET_PCT, TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use super::observatory_slice::{action_label, SealedDecisionRecord, OBSERVATORY_HORIZON_DAYS};
use super::recommendation_outcome::tmv_labels;

pub const CORALYS_TARGET_GENERATION_STARTED: bool = true;
pub const CORALYS_TARGET_SEARCH_AUTHORIZED: bool = false;
pub const TARGET_LOOKAHEAD_AUTHORIZED: bool = false;
pub const ASYMMETRIC_TARGET_AUTHORIZED: bool = false;
pub const HORIZON_SEARCH_AUTHORIZED: bool = false;
pub const CORALYS_TARGET_ARTIFACT_PRESENT: bool = true; // artifact hash: 3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f (frozen 2026-08-16)
pub const TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED: bool = false;
pub const TARGET_SOURCE_FIXED: &str = "deterministic_policy_parameter";
pub const TARGET_SOURCE_CORALYS: &str = "coralys_state_at_t";
pub const CORALYS_MODEL_NONE: &str = "none";

pub const AUTHORIZED_TARGET_INPUTS: [&str; 6] = [
    "certified_tmv_labels",
    "state_hash",
    "bars_at_or_before_T",
    "enrichment_metrics_from_bars_le_T",
    "c3_002_direction",
    "frozen_coralys_target_artifact_id",
];

pub const FORBIDDEN_TARGET_INPUTS: [&str; 8] = [
    "bars_after_T",
    "realized_future_return",
    "realized_V",
    "target_hit",
    "path_optimized_hit_rate",
    "per_name_hindsight_target",
    "coralys_evolved_after_T",
    "new_indicator_families",
];

/// Frozen generator identity. No instance exists until P.E.3 is authorized
/// and `CORALYS_TARGET_ARTIFACT_PRESENT` is true. This is the contract, not
/// a target algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoralysTargetGeneratorArtifact {
    pub artifact_id: String,
    pub content_hash: String,
    pub generator_id: String,
    pub generator_version: String,
    pub methodology_hash: String,
    pub effective_timestamp: String,
    pub input_schema: Vec<String>,
    pub output_schema: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetBasis {
    pub target_model: String,
    pub expected_move: Option<f64>,
    pub state_regime: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionIntent {
    pub instrument: String,
    pub decision_time: String,
    pub direction: String,
    pub target_pct: f64,
    pub horizon_sessions: u32,
    pub state_hash: String,
    pub direction_policy_id: String,
    pub direction_artifact_sha256: String,
    pub coralys_model_id: String,
    pub coralys_model_version: String,
    pub target_source: String,
    pub target_basis: TargetBasis,
    pub intent_hash: String,
    pub sealed_at_t: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertifiedExecutionState {
    pub instrument: String,
    pub decision_time: String,
    pub trend: String,
    pub momentum: String,
    pub volatility: String,
    pub state_hash: String,
}

pub fn refuse_if_research_opened() -> Result<(), String> {
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED
        || TARGET_LOOKAHEAD_AUTHORIZED
        || CORALYS_TARGET_SEARCH_AUTHORIZED
        || ASYMMETRIC_TARGET_AUTHORIZED
        || HORIZON_SEARCH_AUTHORIZED
        || TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED
    {
        return Err(
            "refusing Experiment B while lookahead, search, or profile expansion is open".into(),
        );
    }
    Ok(())
}

pub fn certified_execution_state(
    instrument: &str,
    bars: &[YahooHistoricalBar],
    t: DateTime<Utc>,
) -> Result<CertifiedExecutionState, String> {
    refuse_if_research_opened()?;
    if TARGET_LOOKAHEAD_AUTHORIZED {
        return Err("refusing a target state that may look after T".into());
    }
    let known = bars_at_or_before(bars, t);
    let instrument_id = instrument_id_for(instrument);
    let (profile, _n, max_from) = assess_from_bars_at_t(&known, t, instrument_id);
    if let Some(max_from) = max_from {
        if max_from > t {
            return Err(format!(
                "{instrument} target state included a bar after T ({max_from} > {t})"
            ));
        }
    }
    let (trend, momentum, volatility) = tmv_labels(&profile);
    let state = super::observatory_slice::certified_tmv_state(&trend, &momentum, &volatility);
    Ok(CertifiedExecutionState {
        instrument: instrument.to_string(),
        decision_time: t.to_rfc3339(),
        trend,
        momentum,
        volatility,
        state_hash: state.state_hash,
    })
}

pub fn seal_experiment_a_intent(decision: &SealedDecisionRecord) -> Result<DecisionIntent, String> {
    refuse_if_research_opened()?;
    if decision.policy_artifact_sha256 != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("DecisionIntent identity-gates C3-002".into());
    }
    let basis = TargetBasis {
        target_model: EXECUTION_CONTRACT_ID.to_string(),
        expected_move: Some(EXECUTION_TARGET_PCT),
        state_regime: Some(format!(
            "{} / {} / {}",
            decision.state.trend, decision.state.momentum, decision.state.volatility
        )),
    };
    Ok(hash_intent(DecisionIntent {
        instrument: decision.instrument.clone(),
        decision_time: decision.decision_time.clone(),
        direction: action_label(decision.action).to_string(),
        target_pct: EXECUTION_TARGET_PCT,
        horizon_sessions: OBSERVATORY_HORIZON_DAYS,
        state_hash: decision.state.state_hash.clone(),
        direction_policy_id: decision.policy_id.clone(),
        direction_artifact_sha256: decision.policy_artifact_sha256.clone(),
        coralys_model_id: CORALYS_MODEL_NONE.to_string(),
        coralys_model_version: "n/a".into(),
        target_source: TARGET_SOURCE_FIXED.to_string(),
        target_basis: basis,
        intent_hash: String::new(),
        sealed_at_t: true,
    }))
}

pub fn seal_experiment_b_intent(
    decision: &SealedDecisionRecord,
    _state: &CertifiedExecutionState,
    coralys_model_id: &str,
    coralys_model_version: &str,
) -> Result<DecisionIntent, String> {
    refuse_if_research_opened()?;
    if !CORALYS_TARGET_ARTIFACT_PRESENT || CORALYS_TARGET_SEARCH_AUTHORIZED {
        return Err(
            "Experiment B refuses to emit a Coralys target until a frozen target artifact exists and search stays closed"
                .into(),
        );
    }
    let _ = (decision, coralys_model_id, coralys_model_version);
    Err("Coralys target artifact is not present in this freeze".into())
}

fn hash_intent(mut intent: DecisionIntent) -> DecisionIntent {
    let identity = serde_json::json!({
        "coralys_model_id": intent.coralys_model_id,
        "coralys_model_version": intent.coralys_model_version,
        "decision_time": intent.decision_time,
        "direction": intent.direction,
        "direction_artifact_sha256": intent.direction_artifact_sha256,
        "horizon_sessions": intent.horizon_sessions,
        "instrument": intent.instrument,
        "state_hash": intent.state_hash,
        "target_pct": format!("{:.8}", intent.target_pct),
        "target_source": intent.target_source,
    });
    intent.intent_hash = format!("{:x}", Sha256::digest(identity.to_string().as_bytes()));
    intent
}
