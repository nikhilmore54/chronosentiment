//! Forward/Paper adapter and append-only journal (CS-P-003).
//!
//! Observation system only. No broker, no capital, no tuning.
//! Decision logic is `decide_from_inputs` with an explicit `DecisionPolicy`.
//! Outcomes are measured from raw prices after T, not B4 lake rows.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::backtest::{DecisionLedger, LedgerRecord};
use super::observation_outcome::{measure_ledger_from_prices, PriceBar};
use super::outcome::OutcomeReport;
use super::performance::{measure_performance, PerformanceReport};
use super::policy::DecisionPolicy;
use super::replay::{
    decide_from_inputs, DecideAt, ReplayError, ReplayInputs, UNFROZEN_ENGINE_VERSION,
};
use super::TradingDecision;

pub const FORWARD_PRODUCER: &str = "csp003.forward_adapter";
pub const SESSION_SCHEMA: &str = "csp003.forward_session.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForwardSessionMeta {
    pub schema_version: String,
    pub engine_version: String,
    pub broker: bool,
}

impl Default for ForwardSessionMeta {
    fn default() -> Self {
        Self {
            schema_version: SESSION_SCHEMA.to_string(),
            engine_version: UNFROZEN_ENGINE_VERSION.to_string(),
            broker: false,
        }
    }
}

/// In-memory snapshot adapter. Caller must supply assessments/observations ≤ T.
pub struct ForwardSnapshot {
    pub inputs: ReplayInputs,
}

#[async_trait::async_trait]
impl DecideAt for ForwardSnapshot {
    async fn decide_at(
        &self,
        t: DateTime<Utc>,
        instrument_id: Uuid,
        engine_version: &str,
        policy: &dyn DecisionPolicy,
    ) -> Result<TradingDecision, ReplayError> {
        let mut inputs = self.inputs.clone();
        inputs.as_of = t;
        inputs.instrument_id = instrument_id;
        inputs.engine_version = engine_version.to_string();
        inputs.produced_by = FORWARD_PRODUCER.to_string();
        decide_from_inputs(inputs, policy)
    }
}

pub fn decide_forward<P: DecisionPolicy + ?Sized>(
    mut inputs: ReplayInputs,
    policy: &P,
) -> Result<TradingDecision, ReplayError> {
    if inputs.produced_by.trim().is_empty() {
        inputs.produced_by = FORWARD_PRODUCER.to_string();
    }
    decide_from_inputs(inputs, policy)
}

/// Append-only JSONL journal. Never rewrites a prior decision row.
pub struct ForwardJournal {
    pub root: PathBuf,
}

impl ForwardJournal {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ForwardError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        let meta_path = root.join("session.json");
        if meta_path.exists() {
            let meta: ForwardSessionMeta = serde_json::from_str(&fs::read_to_string(&meta_path)?)?;
            if meta.broker {
                return Err(ForwardError::BrokerForbidden);
            }
            if meta.engine_version != UNFROZEN_ENGINE_VERSION {
                return Err(ForwardError::EngineMismatch(meta.engine_version));
            }
        } else {
            fs::write(
                &meta_path,
                serde_json::to_vec_pretty(&ForwardSessionMeta::default())?,
            )?;
        }
        Ok(Self { root })
    }

    pub fn ledger_path(&self) -> PathBuf {
        self.root.join("ledger.jsonl")
    }

    pub fn prices_path(&self) -> PathBuf {
        self.root.join("prices.jsonl")
    }

    pub fn ticks_path(&self) -> PathBuf {
        self.root.join("ticks.jsonl")
    }

    pub fn load_ledger(&self) -> Result<DecisionLedger, ForwardError> {
        let path = self.ledger_path();
        let mut ledger = DecisionLedger::new(UNFROZEN_ENGINE_VERSION);
        if !path.exists() {
            return Ok(ledger);
        }
        let file = fs::File::open(&path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            ledger.records.push(serde_json::from_str(&line)?);
        }
        Ok(ledger)
    }

    /// Append if `decision_id` is new. Existing id is a no-op (retry-safe, not a rewrite).
    pub fn persist(&self, decision: TradingDecision) -> Result<LedgerRecord, ForwardError> {
        let mut ledger = self.load_ledger()?;
        if let Some(existing) = ledger
            .records
            .iter()
            .find(|r| r.decision_id == decision.decision_id)
        {
            return Ok(existing.clone());
        }
        if let Some(existing) = ledger.records.iter().find(|r| {
            r.instrument_id == decision.instrument_id
                && r.as_of_timestamp == decision.as_of_timestamp
        }) {
            return Ok(existing.clone());
        }
        ledger.append(decision);
        let record = ledger
            .records
            .last()
            .cloned()
            .ok_or(ForwardError::EmptyLedger)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.ledger_path())?;
        writeln!(file, "{}", serde_json::to_string(&record)?)?;
        Ok(record)
    }

    pub fn load_prices(&self) -> Result<Vec<PriceBar>, ForwardError> {
        let path = self.prices_path();
        let mut out = Vec::new();
        if !path.exists() {
            return Ok(out);
        }
        let file = fs::File::open(&path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            out.push(serde_json::from_str(&line)?);
        }
        Ok(out)
    }

    pub fn persist_prices(&self, bars: &[PriceBar]) -> Result<u32, ForwardError> {
        let existing = self.load_prices()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.prices_path())?;
        let mut added = 0u32;
        for bar in bars {
            let dup = existing.iter().any(|e| {
                e.effective_from == bar.effective_from && e.instrument_id == bar.instrument_id
            });
            if dup {
                continue;
            }
            writeln!(file, "{}", serde_json::to_string(bar)?)?;
            added += 1;
        }
        Ok(added)
    }

    pub fn append_tick_line(&self, line: &str) -> Result<(), ForwardError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.ticks_path())?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    pub fn measure(
        &self,
        prices: &[PriceBar],
        now: DateTime<Utc>,
    ) -> Result<OutcomeReport, ForwardError> {
        let ledger = self.load_ledger()?;
        Ok(measure_ledger_from_prices(&ledger, prices, now))
    }

    pub fn performance(
        &self,
        prices: &[PriceBar],
        now: DateTime<Utc>,
    ) -> Result<PerformanceReport, ForwardError> {
        let ledger = self.load_ledger()?;
        let outcomes = measure_ledger_from_prices(&ledger, prices, now);
        Ok(measure_performance(&ledger, &outcomes))
    }
}

#[derive(Debug)]
pub enum ForwardError {
    Io(std::io::Error),
    Json(serde_json::Error),
    BrokerForbidden,
    EngineMismatch(String),
    EmptyLedger,
}

impl std::fmt::Display for ForwardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Json(e) => write!(f, "{e}"),
            Self::BrokerForbidden => write!(f, "CS-P-003 forbids brokerage"),
            Self::EngineMismatch(v) => write!(f, "session engine_version {v} is not unfrozen-dev"),
            Self::EmptyLedger => write!(f, "ledger append produced no row"),
        }
    }
}

impl std::error::Error for ForwardError {}

impl From<std::io::Error> for ForwardError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ForwardError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn write_progress_report(path: &Path, report: &PerformanceReport) -> Result<(), ForwardError> {
    let md = format!(
        "# Forward/Paper progress snapshot\n\n\
         **Engine version: `{}`.** Decision Engine v1.0 is **not frozen**. \
         Not a strategy score. Not G-GATE. No brokerage.\n\n\
         Do not judge the system after a handful of observations. \
         5D–60D horizons must be allowed to elapse.\n\n\
         | Field | Value |\n|---|---|\n\
         | Decisions | `{}` |\n\
         | LONG | `{}` |\n\
         | SHORT | `{}` |\n\
         | NO_TRADE | `{}` |\n\
         | Performance hash | `{}` |\n\
         | Ledger hash | `{}` |\n",
        report.decision_engine_version,
        report.behavior.n_records,
        report.behavior.counts.long,
        report.behavior.counts.short,
        report.behavior.counts.no_trade,
        report.content_hash,
        report.ledger_identity_hash,
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, md)?;
    Ok(())
}
