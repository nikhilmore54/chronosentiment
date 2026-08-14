//! Replay adapter: reconstruct information available at explicit T and emit `TradingDecision`.
//!
//! B4 is read-only. This module never writes. It never selects `knowledge_outcomes`.
//! Decision Engine v1.0 is not frozen.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::reasoning::assessment::AssessmentProfile;

use super::policy::DecisionPolicy;
use super::{
    ConfidenceStatus, DecisionDraft, DecisionEvidence, DecisionLineage, RiskInformation,
    RiskLevel, TradingDecision,
};

#[async_trait::async_trait]
pub trait DecideAt: Send + Sync {
    async fn decide_at(
        &self,
        t: DateTime<Utc>,
        instrument_id: Uuid,
        engine_version: &str,
        policy: &dyn DecisionPolicy,
    ) -> Result<TradingDecision, ReplayError>;
}

/// Caller-supplied until Decision Engine v1.0 is frozen.
pub const UNFROZEN_ENGINE_VERSION: &str = "unfrozen-dev";
pub const REPLAY_PRODUCER: &str = "csp004.adapter.v0.1";
/// Placeholder horizon until an engine version is frozen. Not a G-GATE parameter.
const UNFROZEN_HORIZON_DAYS: u32 = 5;
pub use super::policy::{
    BASELINE_TREND_MAPPING_POLICY_NAME, TREND_MAPPING_RULE,
};

#[derive(Debug)]
pub enum ReplayError {
    NoAssessmentAtT,
    Contract(super::DecisionContractError),
    Database(sqlx::Error),
    Profile(String),
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAssessmentAtT => write!(f, "no assessment with evaluation_timestamp <= T"),
            Self::Contract(e) => write!(f, "{e}"),
            Self::Database(e) => write!(f, "database: {e}"),
            Self::Profile(e) => write!(f, "assessment profile: {e}"),
        }
    }
}

impl std::error::Error for ReplayError {}

impl From<super::DecisionContractError> for ReplayError {
    fn from(value: super::DecisionContractError) -> Self {
        Self::Contract(value)
    }
}

impl From<sqlx::Error> for ReplayError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

/// Observation used only for the temporal firewall. B4 has no observations table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayObservation {
    pub id: Uuid,
    pub effective_from: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ReplayAssessment {
    pub id: Uuid,
    pub evaluation_timestamp: DateTime<Utc>,
    pub signature_hash: String,
    pub profile: AssessmentProfile,
}

#[derive(Debug, Clone)]
pub struct ReplayLakeDecision {
    pub id: Uuid,
    pub evaluation_timestamp: DateTime<Utc>,
}

/// Information set at T. Outcomes are intentionally absent.
#[derive(Debug, Clone)]
pub struct ReplayInputs {
    pub instrument_id: Uuid,
    pub as_of: DateTime<Utc>,
    pub engine_version: String,
    pub produced_by: String,
    pub assessments: Vec<ReplayAssessment>,
    pub lake_decisions: Vec<ReplayLakeDecision>,
    pub observations: Vec<ReplayObservation>,
}

#[derive(Serialize)]
struct InputSetPayload {
    instrument_id: Uuid,
    as_of: DateTime<Utc>,
    assessments: Vec<(Uuid, DateTime<Utc>, String)>,
    lake_decision_ids: Vec<Uuid>,
    observation_ids: Vec<Uuid>,
}

pub fn observations_at_or_before(
    observations: &[ReplayObservation],
    t: DateTime<Utc>,
) -> Vec<&ReplayObservation> {
    observations
        .iter()
        .filter(|o| o.effective_from <= t)
        .collect()
}

/// Reconstruct a `TradingDecision` at T using only inputs ≤ T and an explicit policy.
/// There is no default policy.
pub fn decide_from_inputs<P: DecisionPolicy + ?Sized>(
    inputs: ReplayInputs,
    policy: &P,
) -> Result<TradingDecision, ReplayError> {
    let t = inputs.as_of;
    let mut assessments: Vec<ReplayAssessment> = inputs
        .assessments
        .into_iter()
        .filter(|a| a.evaluation_timestamp <= t)
        .collect();
    assessments.sort_by(|a, b| {
        a.evaluation_timestamp
            .cmp(&b.evaluation_timestamp)
            .then(a.id.cmp(&b.id))
    });

    let mut lake_decisions: Vec<ReplayLakeDecision> = inputs
        .lake_decisions
        .into_iter()
        .filter(|d| d.evaluation_timestamp <= t)
        .collect();
    lake_decisions.sort_by(|a, b| {
        a.evaluation_timestamp
            .cmp(&b.evaluation_timestamp)
            .then(a.id.cmp(&b.id))
    });

    let mut observation_ids: Vec<Uuid> = observations_at_or_before(&inputs.observations, t)
        .into_iter()
        .map(|o| o.id)
        .collect();
    observation_ids.sort_unstable();

    let latest = assessments.last().ok_or(ReplayError::NoAssessmentAtT)?;
    let mapped = policy.decide(&latest.profile, t);

    let mut consumed: Vec<Uuid> = assessments.iter().map(|a| a.id).collect();
    consumed.extend(lake_decisions.iter().map(|d| d.id));
    consumed.extend(observation_ids.iter().copied());
    consumed.sort_unstable();
    consumed.dedup();

    let payload = InputSetPayload {
        instrument_id: inputs.instrument_id,
        as_of: t,
        assessments: assessments
            .iter()
            .map(|a| (a.id, a.evaluation_timestamp, a.signature_hash.clone()))
            .collect(),
        lake_decision_ids: lake_decisions.iter().map(|d| d.id).collect(),
        observation_ids,
    };
    let input_set_hash = sha256_json(&payload);

    let produced_by = if inputs.produced_by.trim().is_empty() {
        REPLAY_PRODUCER.to_string()
    } else {
        inputs.produced_by
    };

    let draft = DecisionDraft {
        engine_version: inputs.engine_version,
        policy_name: policy.name().to_string(),
        instrument_id: inputs.instrument_id,
        as_of_timestamp: t,
        action: mapped.action,
        confidence: None,
        confidence_status: ConfidenceStatus::Unavailable,
        horizon_trading_days: UNFROZEN_HORIZON_DAYS,
        rationale: mapped.action_reason.clone(),
        evidence_refs: mapped.evidence_refs,
        evidence: DecisionEvidence {
            mapping_rule: mapped.mapping_rule,
            diagnostics: mapped.diagnostics,
            factors: mapped.factors,
            consumed_concepts: mapped.consumed_concepts,
        },
        risk: RiskInformation {
            level: RiskLevel::Medium,
            invalidation: None,
        },
        lineage: DecisionLineage {
            produced_by,
            consumed_artifact_ids: consumed,
            assessment_id: Some(latest.id),
            input_set_hash,
        },
    };
    Ok(TradingDecision::try_from_draft(draft)?)
}

/// Read-only B4 (or compatible) replay. Opens `READ ONLY` transactions only.
pub struct ReplayAdapter {
    pool: PgPool,
}

impl ReplayAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn decide_at(
        &self,
        t: DateTime<Utc>,
        instrument_id: Uuid,
        engine_version: &str,
        policy: &dyn DecisionPolicy,
    ) -> Result<TradingDecision, ReplayError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await?;
        let inputs = load_inputs(&mut tx, t, instrument_id, engine_version).await?;
        tx.commit().await?;
        decide_from_inputs(inputs, policy)
    }
}

#[async_trait::async_trait]
impl DecideAt for ReplayAdapter {
    async fn decide_at(
        &self,
        t: DateTime<Utc>,
        instrument_id: Uuid,
        engine_version: &str,
        policy: &dyn DecisionPolicy,
    ) -> Result<TradingDecision, ReplayError> {
        ReplayAdapter::decide_at(self, t, instrument_id, engine_version, policy).await
    }
}

impl ReplayAdapter {
    /// Chronological unique (instrument, assessment as-of) ticks. Read-only. No decision logic.
    pub async fn assessment_schedule(
        &self,
    ) -> Result<Vec<(DateTime<Utc>, Uuid)>, ReplayError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT instrument_id, evaluation_timestamp
            FROM knowledge_assessments
            WHERE instrument_id IS NOT NULL
            GROUP BY instrument_id, evaluation_timestamp
            ORDER BY evaluation_timestamp ASC, instrument_id ASC
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;

        let mut ticks = Vec::with_capacity(rows.len());
        for row in rows {
            let instrument_id: Uuid = row.try_get("instrument_id")?;
            let as_of: DateTime<Utc> = row.try_get("evaluation_timestamp")?;
            ticks.push((as_of, instrument_id));
        }
        Ok(ticks)
    }
}

async fn load_inputs(
    tx: &mut Transaction<'_, Postgres>,
    t: DateTime<Utc>,
    instrument_id: Uuid,
    engine_version: &str,
) -> Result<ReplayInputs, ReplayError> {
    let assess_rows = sqlx::query_as::<_, AssessmentRow>(
        r#"
        SELECT id, evaluation_timestamp, signature_hash, profile_json
        FROM knowledge_assessments
        WHERE instrument_id = $1
          AND evaluation_timestamp <= $2
        ORDER BY evaluation_timestamp ASC, id ASC
        "#,
    )
    .bind(instrument_id)
    .bind(t)
    .fetch_all(&mut **tx)
    .await?;

    let mut assessments = Vec::new();
    for row in assess_rows {
        let profile: AssessmentProfile = serde_json::from_value(row.profile_json)
            .map_err(|e| ReplayError::Profile(e.to_string()))?;
        assessments.push(ReplayAssessment {
            id: row.id,
            evaluation_timestamp: row.evaluation_timestamp,
            signature_hash: row.signature_hash,
            profile,
        });
    }

    let decision_rows = sqlx::query_as::<_, DecisionRow>(
        r#"
        SELECT id, evaluation_timestamp
        FROM knowledge_decisions
        WHERE instrument_id = $1
          AND evaluation_timestamp <= $2
        ORDER BY evaluation_timestamp ASC, id ASC
        "#,
    )
    .bind(instrument_id)
    .bind(t)
    .fetch_all(&mut **tx)
    .await?;

    let lake_decisions = decision_rows
        .into_iter()
        .map(|r| ReplayLakeDecision {
            id: r.id,
            evaluation_timestamp: r.evaluation_timestamp,
        })
        .collect();

    Ok(ReplayInputs {
        instrument_id,
        as_of: t,
        engine_version: engine_version.to_string(),
        produced_by: REPLAY_PRODUCER.to_string(),
        assessments,
        lake_decisions,
        observations: Vec::new(),
    })
}

#[derive(sqlx::FromRow)]
struct AssessmentRow {
    id: Uuid,
    evaluation_timestamp: DateTime<Utc>,
    signature_hash: String,
    profile_json: serde_json::Value,
}

#[derive(sqlx::FromRow)]
struct DecisionRow {
    id: Uuid,
    evaluation_timestamp: DateTime<Utc>,
}

fn sha256_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("input set serializes");
    let digest = Sha256::digest(&bytes);
    format!("{digest:x}")
}
