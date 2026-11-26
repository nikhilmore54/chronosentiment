//! Outcome Engine v0.1 — measure what happened after a recorded decision.
//!
//! Consumes an immutable `DecisionLedger`. Never mutates `TradingDecision` or the ledger.
//! Never calls `decide_at`. Does not replace Knowledge Lake `validation::outcome`.
//! Does not recompute prices. Attaches existing B4 `knowledge_outcomes` rows
//! (5D / 10D / 20D / 60D) whose parent lake decision is at `as_of` and whose
//! horizon expiry is strictly after `as_of`. No performance scoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::backtest::{DecisionLedger, LedgerRecord};
use super::replay::ReplayError;
use super::DecisionAction;

pub const HORIZON_DAYS: [u32; 4] = [5, 10, 20, 60];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HorizonOutcome {
    pub horizon_days: u32,
    pub available: bool,
    pub lake_outcome_id: Option<Uuid>,
    pub lake_decision_id: Option<Uuid>,
    pub outcome_return: Option<f64>,
    pub entry_reached: Option<bool>,
    pub target_hit: Option<bool>,
    pub stop_hit: Option<bool>,
    pub exit_reason: Option<String>,
    pub mfe: Option<f64>,
    pub mae: Option<f64>,
    pub drawdown: Option<f64>,
    pub horizon_expiry_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionOutcomeBundle {
    pub ledger_decision_id: Uuid,
    pub instrument_id: Uuid,
    pub as_of_timestamp: DateTime<Utc>,
    pub action: DecisionAction,
    pub horizons: Vec<HorizonOutcome>,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeReport {
    pub bundles: Vec<DecisionOutcomeBundle>,
}

impl OutcomeReport {
    pub fn identity_hash(&self) -> String {
        let hashes: Vec<&str> = self
            .bundles
            .iter()
            .map(|b| b.content_hash.as_str())
            .collect();
        format!(
            "{:x}",
            Sha256::digest(&serde_json::to_vec(&hashes).unwrap())
        )
    }
}

/// Lake outcome row used for matching. Parent decision must be at T; expiry after T.
#[derive(Debug, Clone)]
pub struct LakeOutcomeRow {
    pub id: Uuid,
    pub lake_decision_id: Uuid,
    pub instrument_id: Uuid,
    pub decision_as_of: DateTime<Utc>,
    pub horizon: String,
    pub outcome_return: f64,
    pub entry_reached: bool,
    pub target_hit: bool,
    pub stop_hit: bool,
    pub exit_reason: String,
    pub mfe: f64,
    pub mae: f64,
    pub drawdown: f64,
    pub horizon_expiry_timestamp: DateTime<Utc>,
}

pub fn measure_record(record: &LedgerRecord, rows: &[LakeOutcomeRow]) -> DecisionOutcomeBundle {
    let mut horizons = Vec::with_capacity(HORIZON_DAYS.len());
    for days in HORIZON_DAYS {
        let label = format!("{days}D");
        let hit = rows.iter().find(|r| {
            r.instrument_id == record.instrument_id
                && r.decision_as_of == record.as_of_timestamp
                && r.horizon == label
                && r.horizon_expiry_timestamp > record.as_of_timestamp
        });
        horizons.push(match hit {
            Some(r) => HorizonOutcome {
                horizon_days: days,
                available: true,
                lake_outcome_id: Some(r.id),
                lake_decision_id: Some(r.lake_decision_id),
                outcome_return: Some(r.outcome_return),
                entry_reached: Some(r.entry_reached),
                target_hit: Some(r.target_hit),
                stop_hit: Some(r.stop_hit),
                exit_reason: Some(r.exit_reason.clone()),
                mfe: Some(r.mfe),
                mae: Some(r.mae),
                drawdown: Some(r.drawdown),
                horizon_expiry_timestamp: Some(r.horizon_expiry_timestamp),
            },
            None => HorizonOutcome {
                horizon_days: days,
                available: false,
                lake_outcome_id: None,
                lake_decision_id: None,
                outcome_return: None,
                entry_reached: None,
                target_hit: None,
                stop_hit: None,
                exit_reason: None,
                mfe: None,
                mae: None,
                drawdown: None,
                horizon_expiry_timestamp: None,
            },
        });
    }
    let content_hash = bundle_hash(record.decision_id, &horizons);
    DecisionOutcomeBundle {
        ledger_decision_id: record.decision_id,
        instrument_id: record.instrument_id,
        as_of_timestamp: record.as_of_timestamp,
        action: record.action,
        horizons,
        content_hash,
    }
}

pub fn measure_ledger(ledger: &DecisionLedger, rows: &[LakeOutcomeRow]) -> OutcomeReport {
    OutcomeReport {
        bundles: ledger
            .records
            .iter()
            .map(|r| measure_record(r, rows))
            .collect(),
    }
}

pub struct OutcomeEngine {
    pool: PgPool,
}

impl OutcomeEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn measure_ledger(
        &self,
        ledger: &DecisionLedger,
    ) -> Result<OutcomeReport, ReplayError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("SET TRANSACTION READ ONLY")
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT
                o.id,
                o.decision_id,
                d.instrument_id,
                d.evaluation_timestamp AS decision_as_of,
                o.horizon,
                o.outcome_return,
                o.entry_reached,
                o.target_hit,
                o.stop_hit,
                o.exit_reason,
                o.mfe,
                o.mae,
                o.drawdown,
                o.horizon_expiry_timestamp
            FROM knowledge_outcomes o
            JOIN knowledge_decisions d ON o.decision_id = d.id
            WHERE d.instrument_id IS NOT NULL
            ORDER BY o.id ASC
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;

        let lake: Vec<LakeOutcomeRow> = rows
            .into_iter()
            .map(|row| -> Result<LakeOutcomeRow, ReplayError> {
                Ok(LakeOutcomeRow {
                    id: row.try_get("id")?,
                    lake_decision_id: row.try_get("decision_id")?,
                    instrument_id: row.try_get("instrument_id")?,
                    decision_as_of: row.try_get("decision_as_of")?,
                    horizon: row.try_get("horizon")?,
                    outcome_return: row.try_get("outcome_return")?,
                    entry_reached: row.try_get("entry_reached")?,
                    target_hit: row.try_get("target_hit")?,
                    stop_hit: row.try_get("stop_hit")?,
                    exit_reason: row.try_get("exit_reason")?,
                    mfe: row.try_get("mfe")?,
                    mae: row.try_get("mae")?,
                    drawdown: row.try_get("drawdown")?,
                    horizon_expiry_timestamp: row.try_get("horizon_expiry_timestamp")?,
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(measure_ledger(ledger, &lake))
    }
}

fn bundle_hash(decision_id: Uuid, horizons: &[HorizonOutcome]) -> String {
    #[derive(Serialize)]
    struct Payload<'a> {
        decision_id: Uuid,
        horizons: &'a [HorizonOutcome],
    }
    let bytes = serde_json::to_vec(&Payload {
        decision_id,
        horizons,
    })
    .expect("outcome bundle serializes");
    format!("{:x}", Sha256::digest(&bytes))
}
