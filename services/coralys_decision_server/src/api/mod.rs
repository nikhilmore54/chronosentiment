//! API response types for the Coralys Decision Intelligence API.
//!
//! **Invariants:**
//! - Response types are derived from `DecisionRecord` — they do not create a
//!   second decision model.
//! - No confidence, probability, ranking, or allocation fields are added.
//! - No decision is reconstructed from C3-002 at request time; the API
//!   exposes the already-certified ledger record.

pub mod detail;
pub mod execution;
pub mod feed;
pub mod ingest;
pub mod outcome;
pub mod recommendations;
pub mod recommendations_v1;

use chrono::{DateTime, Utc};
use coralys_decision::record::{
    CertificationStatus, DecisionRecord, Direction, ExecutionStatus, OutcomeStatus,
    ReferenceRiskStatus,
};
use serde::{Deserialize, Serialize};

// ─── Shared response types ────────────────────────────────────────────────────

/// Serializable direction for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiDirection {
    Long,
    Short,
    NoTrade,
}

impl From<&Direction> for ApiDirection {
    fn from(d: &Direction) -> Self {
        match d {
            Direction::Long => ApiDirection::Long,
            Direction::Short => ApiDirection::Short,
            Direction::NoTrade => ApiDirection::NoTrade,
        }
    }
}

/// Serializable certification status for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiCertificationStatus {
    Certified,
    Pending,
    Failed,
}

impl From<&CertificationStatus> for ApiCertificationStatus {
    fn from(s: &CertificationStatus) -> Self {
        match s {
            CertificationStatus::Certified => ApiCertificationStatus::Certified,
            CertificationStatus::Pending => ApiCertificationStatus::Pending,
            CertificationStatus::Failed => ApiCertificationStatus::Failed,
        }
    }
}

/// Serializable execution status for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiExecutionStatus {
    NotRecorded,
    UserIgnored,
    UserExecuted,
    UserCancelled,
}

impl From<&ExecutionStatus> for ApiExecutionStatus {
    fn from(s: &ExecutionStatus) -> Self {
        match s {
            ExecutionStatus::NotRecorded => ApiExecutionStatus::NotRecorded,
            ExecutionStatus::UserIgnored => ApiExecutionStatus::UserIgnored,
            ExecutionStatus::UserExecuted => ApiExecutionStatus::UserExecuted,
            ExecutionStatus::UserCancelled => ApiExecutionStatus::UserCancelled,
        }
    }
}

/// Serializable outcome status for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiOutcomeStatus {
    Open,
    Target,
    ReferenceRisk,
    Horizon,
    UserClosed,
}

impl From<&OutcomeStatus> for ApiOutcomeStatus {
    fn from(s: &OutcomeStatus) -> Self {
        match s {
            OutcomeStatus::Open => ApiOutcomeStatus::Open,
            OutcomeStatus::Target => ApiOutcomeStatus::Target,
            OutcomeStatus::ReferenceRisk => ApiOutcomeStatus::ReferenceRisk,
            OutcomeStatus::Horizon => ApiOutcomeStatus::Horizon,
            OutcomeStatus::UserClosed => ApiOutcomeStatus::UserClosed,
        }
    }
}

/// Serializable reference risk status for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiReferenceRiskStatus {
    Reference,
}

impl From<&ReferenceRiskStatus> for ApiReferenceRiskStatus {
    fn from(s: &ReferenceRiskStatus) -> Self {
        match s {
            ReferenceRiskStatus::Reference => ApiReferenceRiskStatus::Reference,
        }
    }
}

// ─── Full decision response (used by Detail API) ──────────────────────────────

/// Complete decision response — exposes the certified ledger record verbatim.
///
/// **No fields are added beyond what the `DecisionRecord` contains.**
/// In particular, no confidence, probability, ranking, or allocation fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub identity: IdentityResponse,
    pub certification: CertificationResponse,
    pub decision: DecisionCoreResponse,
    pub reference_risk: ReferenceRiskResponse,
    pub execution: ExecutionResponse,
    pub outcome: OutcomeResponse,
    pub evidence: EvidenceResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub decision_id: String,
    pub instrument: String,
    pub decision_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationResponse {
    pub status: ApiCertificationStatus,
    pub policy_artifact_hash: String,
    pub execution_artifact_hash: Option<String>,
    pub decision_pipeline: String,
    pub certified_timestamp: DateTime<Utc>,
    pub data_snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionCoreResponse {
    pub direction: ApiDirection,
    pub trend: String,
    pub momentum: String,
    pub volatility: String,
    pub target_price: Option<f64>,
    /// ATR-14 in price units at decision time T. Null when unavailable.
    pub atr_14: Option<f64>,
    /// Last traded price / previous close at decision time T.
    pub reference_price: Option<f64>,
    /// Next NSE trading session date (YYYY-MM-DD) this decision applies to.
    pub effective_session: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceRiskResponse {
    pub boundary_price: Option<f64>,
    pub boundary_type: String,
    pub status: ApiReferenceRiskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub status: ApiExecutionStatus,
    pub execution_timestamp: Option<DateTime<Utc>>,
    /// Only present when the user explicitly supplied it.
    pub quantity: Option<f64>,
    /// Only present when the user explicitly supplied it.
    pub execution_price: Option<f64>,
    pub execution_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeResponse {
    pub status: ApiOutcomeStatus,
    pub exit_reason: Option<String>,
    pub exit_timestamp: Option<DateTime<Utc>>,
    pub exit_price: Option<f64>,
    pub realized_pnl: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceResponse {
    pub similar_decisions_count: Option<u32>,
    pub historical_target_rate: Option<f64>,
    pub median_mae_pct: Option<f64>,
    pub p90_mae_pct: Option<f64>,
    pub median_mfe_pct: Option<f64>,
    pub median_time_to_target_sessions: Option<f64>,
}

impl From<&DecisionRecord> for DecisionResponse {
    fn from(r: &DecisionRecord) -> Self {
        DecisionResponse {
            identity: IdentityResponse {
                decision_id: r.identity.decision_id.clone(),
                instrument: r.identity.instrument.clone(),
                decision_timestamp: r.identity.decision_timestamp,
            },
            certification: CertificationResponse {
                status: ApiCertificationStatus::from(&r.certification.status),
                policy_artifact_hash: r.certification.policy_artifact_hash.clone(),
                execution_artifact_hash: r.certification.execution_artifact_hash.clone(),
                decision_pipeline: r.certification.decision_pipeline.clone(),
                certified_timestamp: r.certification.certified_timestamp,
                data_snapshot_id: r.certification.data_snapshot_id.clone(),
            },
            decision: DecisionCoreResponse {
                direction: ApiDirection::from(&r.decision.direction),
                trend: r.decision.trend.clone(),
                momentum: r.decision.momentum.clone(),
                volatility: r.decision.volatility.clone(),
                target_price: r.decision.target_price,
                atr_14: r.decision.atr_14,
                reference_price: r.decision.reference_price,
                effective_session: r.decision.effective_session.clone(),
            },
            reference_risk: ReferenceRiskResponse {
                boundary_price: r.reference_risk.boundary_price,
                boundary_type: r.reference_risk.boundary_type.clone(),
                status: ApiReferenceRiskStatus::from(&r.reference_risk.status),
            },
            execution: ExecutionResponse {
                status: ApiExecutionStatus::from(&r.execution.status),
                execution_timestamp: r.execution.execution_timestamp,
                quantity: r.execution.quantity,
                execution_price: r.execution.execution_price,
                execution_source: r.execution.execution_source.clone(),
            },
            outcome: OutcomeResponse {
                status: ApiOutcomeStatus::from(&r.outcome.status),
                exit_reason: r.outcome.exit_reason.clone(),
                exit_timestamp: r.outcome.exit_timestamp,
                exit_price: r.outcome.exit_price,
                realized_pnl: r.outcome.realized_pnl,
            },
            evidence: EvidenceResponse {
                similar_decisions_count: r.evidence.similar_decisions_count,
                historical_target_rate: r.evidence.historical_target_rate,
                median_mae_pct: r.evidence.median_mae_pct,
                p90_mae_pct: r.evidence.p90_mae_pct,
                median_mfe_pct: r.evidence.median_mfe_pct,
                median_time_to_target_sessions: r.evidence.median_time_to_target_sessions,
            },
        }
    }
}
