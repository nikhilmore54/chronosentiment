//! Replay/backtest orchestration: T1…Tn → TradingDecision → immutable ledger.
//!
//! Does not compute outcomes, scores, or charts. Does not freeze Decision Engine v1.0.
//! Decision logic is delegated to `DecideAt` (the Replay Adapter).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::policy::DecisionPolicy;
use super::replay::{DecideAt, ReplayAdapter, ReplayError, UNFROZEN_ENGINE_VERSION};
use super::TradingDecision;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayTick {
    pub as_of: DateTime<Utc>,
    pub instrument_id: Uuid,
}

/// One append-only ledger row. `decision_timestamp` equals as-of T in replay (not wall clock).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerRecord {
    pub sequence: u32,
    pub decision_id: Uuid,
    pub engine_version: String,
    #[serde(default)]
    pub policy_name: String,
    pub instrument_id: Uuid,
    pub as_of_timestamp: DateTime<Utc>,
    pub decision_timestamp: DateTime<Utc>,
    pub action: super::DecisionAction,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub confidence_status: super::ConfidenceStatus,
    pub input_set_hash: String,
    pub lineage: super::DecisionLineage,
    pub content_hash: String,
    #[serde(default)]
    pub evidence: super::DecisionEvidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionLedger {
    pub engine_version: String,
    pub records: Vec<LedgerRecord>,
}

impl DecisionLedger {
    pub fn new(engine_version: impl Into<String>) -> Self {
        Self {
            engine_version: engine_version.into(),
            records: Vec::new(),
        }
    }

    pub fn append(&mut self, decision: TradingDecision) {
        let sequence = (self.records.len() as u32) + 1;
        self.records.push(LedgerRecord {
            sequence,
            decision_id: decision.decision_id,
            engine_version: decision.engine_version.clone(),
            policy_name: decision.policy_name.clone(),
            instrument_id: decision.instrument_id,
            as_of_timestamp: decision.as_of_timestamp,
            decision_timestamp: decision.as_of_timestamp,
            action: decision.action,
            confidence: decision.confidence,
            confidence_status: decision.confidence_status,
            input_set_hash: decision.lineage.input_set_hash.clone(),
            lineage: decision.lineage,
            content_hash: decision.provenance.content_hash,
            evidence: decision.evidence,
        });
    }

    pub fn identity_hash(&self) -> String {
        let hashes: Vec<&str> = self
            .records
            .iter()
            .map(|r| r.content_hash.as_str())
            .collect();
        let bytes = serde_json::to_vec(&hashes).expect("ledger hashes serialize");
        format!("{:x}", Sha256::digest(&bytes))
    }
}

pub async fn run_replay_backtest<D: DecideAt>(
    adapter: &D,
    ticks: &[ReplayTick],
    engine_version: &str,
    policy: &dyn DecisionPolicy,
) -> Result<DecisionLedger, ReplayError> {
    let mut ordered = ticks.to_vec();
    ordered.sort_by(|a, b| {
        a.as_of
            .cmp(&b.as_of)
            .then(a.instrument_id.cmp(&b.instrument_id))
    });

    let mut ledger = DecisionLedger::new(engine_version);
    for tick in ordered {
        let decision = adapter
            .decide_at(tick.as_of, tick.instrument_id, engine_version, policy)
            .await?;
        ledger.append(decision);
    }
    Ok(ledger)
}

/// Drive the existing Replay Adapter from B4 assessment timestamps. No new decision logic.
pub async fn populate_ledger_from_assessment_schedule(
    adapter: &ReplayAdapter,
    engine_version: &str,
    policy: &dyn DecisionPolicy,
) -> Result<DecisionLedger, ReplayError> {
    let schedule = adapter.assessment_schedule().await?;
    let ticks: Vec<ReplayTick> = schedule
        .into_iter()
        .map(|(as_of, instrument_id)| ReplayTick {
            as_of,
            instrument_id,
        })
        .collect();
    run_replay_backtest(adapter, &ticks, engine_version, policy).await
}

pub fn unfrozen_engine_version() -> &'static str {
    UNFROZEN_ENGINE_VERSION
}
