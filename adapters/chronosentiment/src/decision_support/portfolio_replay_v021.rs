//! Portfolio Replay v0.2.1 — Continuous Lifecycle with Position Upgrades.
//!
//! ## Architecture
//!
//! This module is a **portfolio lifecycle orchestrator only**. It does NOT own:
//!   - C3-002 decision logic
//!   - Coralys target/stop calculation
//!   - Execution semantics
//!   - Exit ordering or lookahead rules
//!
//! Those remain canonical in their respective modules. This module orchestrates
//! multiple trades, capital recycling, and session-by-session lifecycle.
//!
//! ## Canonical execution pipeline (unchanged)
//!
//! ```text
//! C3-002 decision at T
//!        ↓
//! generate_historical_replay_decision()
//!        ↓
//! seal_execution_intent() / seal_coralys_execution_intent()
//!        ↓
//! SealedExecutionIntent stored in TradeLot
//!        ↓
//! first_exit_with_optional_stop() called each session
//!        ↓
//! TARGET / STOP / HORIZON / Observing
//!        ↓
//! capital released on close
//! ```
//!
//! ## What v0.2.1 adds over v0.1
//!
//!   - Session-by-session loop over the full P.E.2 historical period
//!   - Multiple lots per instrument (position upgrades / accumulation)
//!   - Capital recycled when a lot closes
//!   - Per-session portfolio snapshots
//!   - Capital velocity metric
//!
//! ## Capital model
//!
//!   - `initial_capital_inr`: starting capital (₹5,000 for this experiment)
//!   - `contribution_schedule`: list of (session_index, amount_inr) contributions
//!   - Per-lot allocation = available_cash / n_eligible_instruments_at_T
//!   - Minimum lot allocation: ₹100 (skip if cash too thin)
//!
//! ## Execution arms (FROZEN — do not modify)
//!
//!   - P.E.2 arm: fixed +5% target, no stop, 20-session max hold
//!   - Coralys v0 arm: ATR/TMV target + enforced risk_boundary stop, 20-session max hold

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::ingestion::yahoo::YahooHistoricalBar;

use super::coralys_execution_model::{
    seal_coralys_execution_intent, CoralysExecutionResult, CORALYS_EXEC_ARTIFACT_HASH,
    MAXIMUM_HOLD_SESSIONS,
};
use super::csp006_protocol::{RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE};
use super::observatory_execution::{
    entry_close, first_exit_with_optional_stop, seal_execution_intent, ExitReason,
    SealedExecutionIntent, EXECUTION_TARGET_PCT,
};
use super::observatory_historical::{decision_time_bars, generate_historical_replay_decision};
use super::observatory_live_execution_pe3::atr_14_at_t;
use super::observatory_prospective::latest_session_at_or_before;
use super::observatory_slice::SealedDecisionRecord;
use super::policy_artifact::PolicyArtifact;
use super::portfolio_replay_v0::{
    TradePath, CORALYS_ARM_CONTRACT, INITIAL_CAPITAL_INR, PE2_ARM_CONTRACT,
    PORTFOLIO_REPLAY_REQUESTED_CLOCK,
};
use super::DecisionAction;

// ─── Constants ────────────────────────────────────────────────────────────────

pub const CONTINUOUS_REPLAY_VERSION: &str = "portfolio_replay_v0_2_1";
pub const CONTINUOUS_EXPERIMENT_ID: &str = "portfolio_continuous_v021_2026-08-16";
/// Minimum INR per lot — skip allocation if cash is below this.
pub const MIN_LOT_ALLOCATION_INR: f64 = 100.0;

// ─── Contribution schedule ────────────────────────────────────────────────────

/// A scheduled capital contribution at a specific session index.
///
/// For this experiment: initial_capital=₹5,000 at session 0, no further contributions.
/// The engine supports arbitrary schedules for future experiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalContribution {
    /// 0-based session index at which to inject capital.
    pub session_index: u32,
    /// Amount to inject (INR).
    pub amount_inr: f64,
}

// ─── Allocation model ─────────────────────────────────────────────────────────

/// How per-lot capital is determined at the open phase of each session.
///
/// `EqualWeight` is the v0.2.1/v0.3 baseline — all available cash is split
/// equally across every eligible signal in the session.
///
/// `MaxPerSymbol` is the v0.4 experiment — each eligible signal receives at most
/// `max_per_symbol_inr`, leaving the remainder available for future sessions.
/// This prevents session-1 capital exhaustion when signal density is high.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "model")]
pub enum AllocationModel {
    /// v0.2.1/v0.3 baseline: `available_cash / n_eligible_signals`.
    /// Deploys all available cash in every session that has eligible signals.
    EqualWeight,
    /// v0.4 experiment: `min(max_per_symbol_inr, available_cash)` per signal.
    /// Leaves undeployed capital available for subsequent sessions.
    MaxPerSymbol {
        /// Maximum INR to allocate to a single lot.
        max_per_symbol_inr: f64,
    },
}

impl AllocationModel {
    /// Compute the per-lot allocation given available cash and number of eligible signals.
    pub fn per_lot_alloc(&self, available_cash: f64, n_eligible: usize) -> f64 {
        match self {
            AllocationModel::EqualWeight => {
                if n_eligible == 0 { 0.0 } else { available_cash / n_eligible as f64 }
            }
            AllocationModel::MaxPerSymbol { max_per_symbol_inr } => {
                available_cash.min(*max_per_symbol_inr)
            }
        }
    }
}

// ─── Portfolio replay config ──────────────────────────────────────────────────

/// Configuration for a continuous portfolio replay run.
///
/// Execution semantics (C3-002, Coralys v0, stop-loss, target) remain frozen.
/// Universe, initial capital, and allocation model are variable.
///
/// The v0.2.1 baseline uses `ContinuousPortfolioConfig::v021_baseline()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousPortfolioConfig {
    /// Instrument symbols to include. Must be a subset of the bar cache keys.
    pub universe: Vec<String>,
    /// Human-readable label for this configuration (used in archive metadata).
    pub config_label: String,
    /// Capital contribution schedule. Empty = no contributions beyond initial capital.
    pub contributions: Vec<CapitalContribution>,
    /// Initial capital in INR. Default: ₹5,000 (v0.2.1/v0.3 baseline).
    pub initial_capital_inr: f64,
    /// Allocation model. Default: EqualWeight (v0.2.1/v0.3 baseline).
    pub allocation_model: AllocationModel,
}

impl ContinuousPortfolioConfig {
    /// v0.2.1 baseline: 7-instrument RESEARCH_UNIVERSE, ₹5,000, EqualWeight.
    pub fn v021_baseline() -> Self {
        ContinuousPortfolioConfig {
            universe: RESEARCH_UNIVERSE.iter().map(|s| s.to_string()).collect(),
            config_label: "v021_baseline_7_instruments".to_string(),
            contributions: vec![],
            initial_capital_inr: 5_000.0,
            allocation_model: AllocationModel::EqualWeight,
        }
    }

    /// v0.3 config with a custom universe slice, ₹5,000, EqualWeight.
    pub fn v03_universe(instruments: &[&str], label: &str) -> Self {
        ContinuousPortfolioConfig {
            universe: instruments.iter().map(|s| s.to_string()).collect(),
            config_label: label.to_string(),
            contributions: vec![],
            initial_capital_inr: 5_000.0,
            allocation_model: AllocationModel::EqualWeight,
        }
    }

    /// v0.4 MaxPerSymbol config: custom universe, ₹1M initial capital, per-symbol cap.
    pub fn v04_max_per_symbol(
        instruments: &[&str],
        label: &str,
        initial_capital_inr: f64,
        max_per_symbol_inr: f64,
    ) -> Self {
        ContinuousPortfolioConfig {
            universe: instruments.iter().map(|s| s.to_string()).collect(),
            config_label: label.to_string(),
            contributions: vec![],
            initial_capital_inr,
            allocation_model: AllocationModel::MaxPerSymbol { max_per_symbol_inr },
        }
    }
}

// ─── Trade lot ────────────────────────────────────────────────────────────────

/// A single allocated lot — the atomic unit of the continuous portfolio.
///
/// Multiple lots per instrument are allowed. Each lot has its own entry,
/// target, stop, and exit. Lots are never merged.
///
/// The `decision` and `intent` fields are stored so that the canonical
/// `first_exit_with_optional_stop()` can be called at each subsequent session
/// without re-deriving execution semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLot {
    /// Unique lot identifier: "{arm}-{instrument}-seq{decision_sequence}".
    pub trade_id: String,
    /// C3-002 decision ID that triggered this lot.
    pub decision_id: String,
    /// Instrument symbol.
    pub instrument: String,
    /// "LONG" or "SHORT".
    pub direction: String,
    /// Session timestamp at which this lot was opened (RFC3339).
    pub entry_time: String,
    /// Entry price (close of the entry session).
    pub entry_price: f64,
    /// Capital allocated to this lot (INR).
    pub allocation_inr: f64,
    /// Fractional units = allocation_inr / entry_price.
    pub units: f64,
    /// Target return fraction (e.g. 0.05 = 5%).
    pub target_pct: f64,
    /// Absolute target price.
    pub target_price: f64,
    /// Stop return fraction (None for P.E.2 arm).
    pub stop_pct: Option<f64>,
    /// Absolute stop price (None for P.E.2 arm).
    pub stop_price: Option<f64>,
    /// Session timestamp at which this lot was closed (RFC3339). None if open.
    pub exit_time: Option<String>,
    /// Exit price. None if open.
    pub exit_price: Option<f64>,
    /// Exit reason. None if open.
    pub exit_reason: Option<ExitReason>,
    /// Number of sessions held. None if open.
    pub holding_sessions: Option<u32>,
    /// Realized return fraction. None if open.
    pub realized_return: Option<f64>,
    /// Realized P&L in INR. None if open.
    pub realized_pnl_inr: Option<f64>,
    /// Sequence number of this decision for this instrument in this arm (1-indexed).
    pub decision_sequence: u32,
    /// Trade-path diagnostics. None if open.
    pub trade_path: Option<TradePath>,
    /// Canonical decision record — stored for `first_exit_with_optional_stop`.
    /// Serialized as JSON string to avoid deep nesting in the ledger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_decision: Option<SealedDecisionRecord>,
    /// Canonical execution intent — stored for `first_exit_with_optional_stop`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sealed_intent: Option<SealedExecutionIntent>,
}

impl TradeLot {
    pub fn is_open(&self) -> bool {
        self.exit_time.is_none()
    }

    pub fn unrealized_pnl_inr(&self, mark_price: f64) -> f64 {
        if self.entry_price <= 0.0 || mark_price <= 0.0 {
            return 0.0;
        }
        let ret = match self.direction.as_str() {
            "LONG" => (mark_price - self.entry_price) / self.entry_price,
            "SHORT" => (self.entry_price - mark_price) / self.entry_price,
            _ => 0.0,
        };
        self.allocation_inr * ret
    }

    pub fn market_value_inr(&self, mark_price: f64) -> f64 {
        self.allocation_inr + self.unrealized_pnl_inr(mark_price)
    }
}

// ─── Portfolio snapshot ───────────────────────────────────────────────────────

/// State of a portfolio arm at a single session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    /// Session timestamp (RFC3339).
    pub session_time: String,
    /// Session index (0-based).
    pub session_index: u32,
    /// Cash available for new allocations (INR).
    pub cash_inr: f64,
    /// Total capital currently deployed in open lots (INR, at cost).
    pub invested_capital_inr: f64,
    /// Mark-to-market value of all open lots (INR).
    pub market_value_inr: f64,
    /// Total portfolio equity = cash + market_value (INR).
    pub total_equity_inr: f64,
    /// Cumulative realized P&L (INR).
    pub realized_pnl_inr: f64,
    /// Unrealized P&L on open lots (INR).
    pub unrealized_pnl_inr: f64,
    /// Number of open lots at this session.
    pub open_lots: u32,
    /// Number of lots opened this session.
    pub lots_opened_this_session: u32,
    /// Number of lots closed this session.
    pub lots_closed_this_session: u32,
    /// Instrument → current exposure in INR (mark-to-market).
    pub instrument_exposure: BTreeMap<String, f64>,
}

// ─── Continuous portfolio arm ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousPortfolioArm {
    pub arm_id: String,
    pub execution_contract: String,
    pub initial_capital_inr: f64,
    pub cash_inr: f64,
    /// All lots (open and closed).
    pub trade_log: Vec<TradeLot>,
    /// Per-session snapshots.
    pub session_snapshots: Vec<PortfolioSnapshot>,
    /// Cumulative realized P&L.
    pub total_realized_pnl_inr: f64,
    /// Peak portfolio equity (for drawdown).
    pub peak_equity_inr: f64,
    /// Maximum drawdown in INR.
    pub max_drawdown_inr: f64,
    /// Maximum drawdown as fraction of peak.
    pub max_drawdown_pct: f64,
    /// Per-instrument decision sequence counter.
    pub decision_sequence: BTreeMap<String, u32>,
}

impl ContinuousPortfolioArm {
    pub fn new(arm_id: &str, execution_contract: &str, initial_capital: f64) -> Self {
        ContinuousPortfolioArm {
            arm_id: arm_id.to_string(),
            execution_contract: execution_contract.to_string(),
            initial_capital_inr: initial_capital,
            cash_inr: initial_capital,
            trade_log: Vec::new(),
            session_snapshots: Vec::new(),
            total_realized_pnl_inr: 0.0,
            peak_equity_inr: initial_capital,
            max_drawdown_inr: 0.0,
            max_drawdown_pct: 0.0,
            decision_sequence: BTreeMap::new(),
        }
    }

    pub fn open_lots_count(&self) -> usize {
        self.trade_log.iter().filter(|l| l.is_open()).count()
    }

    pub fn invested_capital_inr(&self) -> f64 {
        self.trade_log
            .iter()
            .filter(|l| l.is_open())
            .map(|l| l.allocation_inr)
            .sum()
    }

    pub fn market_value_inr(&self, mark_prices: &BTreeMap<String, f64>) -> f64 {
        self.trade_log
            .iter()
            .filter(|l| l.is_open())
            .map(|l| {
                let mark = mark_prices.get(&l.instrument).copied().unwrap_or(l.entry_price);
                l.market_value_inr(mark)
            })
            .sum()
    }

    pub fn unrealized_pnl_inr(&self, mark_prices: &BTreeMap<String, f64>) -> f64 {
        self.trade_log
            .iter()
            .filter(|l| l.is_open())
            .map(|l| {
                let mark = mark_prices.get(&l.instrument).copied().unwrap_or(l.entry_price);
                l.unrealized_pnl_inr(mark)
            })
            .sum()
    }

    pub fn total_equity_inr(&self, mark_prices: &BTreeMap<String, f64>) -> f64 {
        self.cash_inr + self.market_value_inr(mark_prices)
    }

    pub fn instrument_exposure(
        &self,
        mark_prices: &BTreeMap<String, f64>,
    ) -> BTreeMap<String, f64> {
        let mut exp: BTreeMap<String, f64> = BTreeMap::new();
        for lot in self.trade_log.iter().filter(|l| l.is_open()) {
            let mark = mark_prices.get(&lot.instrument).copied().unwrap_or(lot.entry_price);
            *exp.entry(lot.instrument.clone()).or_insert(0.0) += lot.market_value_inr(mark);
        }
        exp
    }

    pub fn update_drawdown(&mut self, equity: f64) {
        if equity > self.peak_equity_inr {
            self.peak_equity_inr = equity;
        }
        let dd = self.peak_equity_inr - equity;
        if dd > self.max_drawdown_inr {
            self.max_drawdown_inr = dd;
            if self.peak_equity_inr > 0.0 {
                self.max_drawdown_pct = dd / self.peak_equity_inr;
            }
        }
    }

    pub fn next_decision_sequence(&mut self, instrument: &str) -> u32 {
        let seq = self.decision_sequence.entry(instrument.to_string()).or_insert(0);
        *seq += 1;
        *seq
    }

    pub fn n_lots_opened(&self) -> usize {
        self.trade_log.len()
    }

    pub fn n_target(&self) -> usize {
        self.trade_log
            .iter()
            .filter(|l| matches!(l.exit_reason, Some(ExitReason::Target)))
            .count()
    }

    pub fn n_stop(&self) -> usize {
        self.trade_log
            .iter()
            .filter(|l| matches!(l.exit_reason, Some(ExitReason::Stop)))
            .count()
    }

    pub fn n_horizon(&self) -> usize {
        self.trade_log
            .iter()
            .filter(|l| matches!(l.exit_reason, Some(ExitReason::Horizon)))
            .count()
    }

    pub fn n_ambiguous(&self) -> usize {
        self.trade_log
            .iter()
            .filter(|l| matches!(l.exit_reason, Some(ExitReason::Ambiguous)))
            .count()
    }

    pub fn holding_sessions_list(&self) -> Vec<u32> {
        self.trade_log.iter().filter_map(|l| l.holding_sessions).collect()
    }

    pub fn avg_holding_sessions(&self) -> Option<f64> {
        let list = self.holding_sessions_list();
        if list.is_empty() {
            return None;
        }
        Some(list.iter().sum::<u32>() as f64 / list.len() as f64)
    }

    pub fn median_holding_sessions(&self) -> Option<f64> {
        let mut list = self.holding_sessions_list();
        if list.is_empty() {
            return None;
        }
        list.sort_unstable();
        let n = list.len();
        if n % 2 == 0 {
            Some((list[n / 2 - 1] + list[n / 2]) as f64 / 2.0)
        } else {
            Some(list[n / 2] as f64)
        }
    }

    pub fn total_return_pct(&self, mark_prices: &BTreeMap<String, f64>) -> f64 {
        if self.initial_capital_inr <= 0.0 {
            return 0.0;
        }
        (self.total_equity_inr(mark_prices) - self.initial_capital_inr) / self.initial_capital_inr
    }
}

// ─── Continuous portfolio ledger ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousArmSummary {
    pub arm_id: String,
    pub execution_contract: String,
    pub initial_capital_inr: f64,
    pub final_cash_inr: f64,
    pub final_invested_inr: f64,
    pub final_portfolio_value_inr: f64,
    pub total_return_pct: f64,
    pub total_realized_pnl_inr: f64,
    pub total_unrealized_pnl_inr: f64,
    pub max_drawdown_inr: f64,
    pub max_drawdown_pct: f64,
    pub n_lots_opened: usize,
    pub n_target: usize,
    pub n_stop: usize,
    pub n_horizon: usize,
    pub n_ambiguous: usize,
    pub n_open_at_end: usize,
    pub avg_holding_sessions: Option<f64>,
    pub median_holding_sessions: Option<f64>,
    pub n_sessions_simulated: u32,
    /// Total capital deployed / initial capital. Measures capital recycling velocity.
    pub capital_velocity_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousPortfolioLedger {
    pub path_kind: String,
    pub experiment_id: String,
    pub start_clock: String,
    pub certified_t: String,
    pub initial_capital_inr: f64,
    pub coralys_artifact_hash: String,
    pub c3_002_artifact_hash: String,
    pub pe2_arm: ContinuousPortfolioArm,
    pub coralys_arm: ContinuousPortfolioArm,
    pub pe2_summary: ContinuousArmSummary,
    pub coralys_summary: ContinuousArmSummary,
    pub n_sessions_simulated: u32,
    pub universe: Vec<String>,
    pub integrity_note: String,
}

// ─── Output guard ─────────────────────────────────────────────────────────────

pub fn refuse_v021_output(path: &str) -> Result<(), String> {
    for forbidden in [
        "pe2_control_2026-08-16",
        "pe3_coralys_v0_2026-08-16",
        "observatory/prospective",
        "historical_replay_v0",
        "historical_replay_v1",
        "selected_policy.json",
        "portfolio_comparison_pe2_vs_pe3_2026-08-16",
        "portfolio_replay_v02_horizon_matrix_2026-08-16",
    ] {
        if path.contains(forbidden) {
            return Err(format!(
                "portfolio replay v0.2.1 refuses to write to protected path: {forbidden}"
            ));
        }
    }
    Ok(())
}

// ─── Helper: build arm summary ────────────────────────────────────────────────

fn build_continuous_arm_summary(
    arm: &ContinuousPortfolioArm,
    mark_prices: &BTreeMap<String, f64>,
    n_sessions: u32,
) -> ContinuousArmSummary {
    let n_open_at_end = arm.open_lots_count();
    let final_value = arm.total_equity_inr(mark_prices);
    let total_deployed: f64 = arm.trade_log.iter().map(|l| l.allocation_inr).sum();
    let velocity = if arm.initial_capital_inr > 0.0 {
        total_deployed / arm.initial_capital_inr
    } else {
        0.0
    };
    ContinuousArmSummary {
        arm_id: arm.arm_id.clone(),
        execution_contract: arm.execution_contract.clone(),
        initial_capital_inr: arm.initial_capital_inr,
        final_cash_inr: arm.cash_inr,
        final_invested_inr: arm.invested_capital_inr(),
        final_portfolio_value_inr: final_value,
        total_return_pct: arm.total_return_pct(mark_prices),
        total_realized_pnl_inr: arm.total_realized_pnl_inr,
        total_unrealized_pnl_inr: arm.unrealized_pnl_inr(mark_prices),
        max_drawdown_inr: arm.max_drawdown_inr,
        max_drawdown_pct: arm.max_drawdown_pct,
        n_lots_opened: arm.n_lots_opened(),
        n_target: arm.n_target(),
        n_stop: arm.n_stop(),
        n_horizon: arm.n_horizon(),
        n_ambiguous: arm.n_ambiguous(),
        n_open_at_end,
        avg_holding_sessions: arm.avg_holding_sessions(),
        median_holding_sessions: arm.median_holding_sessions(),
        n_sessions_simulated: n_sessions,
        capital_velocity_ratio: velocity,
    }
}

// ─── Main continuous replay ───────────────────────────────────────────────────

/// Run Portfolio Replay v0.2.1 — Continuous Lifecycle with Position Upgrades.
///
/// Session-by-session loop over the full P.E.2 historical period.
/// Uses the canonical execution pipeline unchanged:
///   - `generate_historical_replay_decision` for C3-002 decisions
///   - `seal_execution_intent` / `seal_coralys_execution_intent` for intents
///   - `first_exit_with_optional_stop` for exit scanning
///
/// Multiple lots per instrument are allowed (position upgrades).
/// Capital is recycled when a lot closes.
pub fn run_continuous_portfolio_replay(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
) -> Result<ContinuousPortfolioLedger, String> {
    run_continuous_portfolio_replay_with_contributions(artifact, cache, &[])
}

/// Run with a `ContinuousPortfolioConfig` — the v0.3/v0.4 entry point.
///
/// Universe, initial capital, and allocation model are taken from the config.
/// All execution semantics remain frozen.
pub fn run_continuous_portfolio_replay_with_config(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    config: &ContinuousPortfolioConfig,
) -> Result<ContinuousPortfolioLedger, String> {
    run_continuous_portfolio_replay_inner(
        artifact,
        cache,
        &config.universe,
        &config.contributions,
        config.initial_capital_inr,
        &config.allocation_model,
    )
}

/// Run with an explicit contribution schedule (v0.2.1 baseline — 7-instrument universe).
///
/// `contributions`: list of (session_index, amount_inr) injections.
/// For the v0.2.1 baseline experiment: pass an empty slice (₹5,000 at session 0
/// is the initial capital, not a contribution).
pub fn run_continuous_portfolio_replay_with_contributions(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    contributions: &[CapitalContribution],
) -> Result<ContinuousPortfolioLedger, String> {
    let universe: Vec<String> = RESEARCH_UNIVERSE.iter().map(|s| s.to_string()).collect();
    run_continuous_portfolio_replay_inner(
        artifact,
        cache,
        &universe,
        contributions,
        INITIAL_CAPITAL_INR,
        &AllocationModel::EqualWeight,
    )
}

/// Inner implementation — accepts explicit universe, contributions, initial capital, and allocation model.
///
/// All callers must go through one of the public wrappers above.
fn run_continuous_portfolio_replay_inner(
    artifact: &PolicyArtifact,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    universe: &[String],
    contributions: &[CapitalContribution],
    initial_capital_inr: f64,
    allocation_model: &AllocationModel,
) -> Result<ContinuousPortfolioLedger, String> {
    // Identity gates
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("continuous portfolio replay identity-gates C3-002".into());
    }
    if CORALYS_EXEC_ARTIFACT_HASH
        != "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
    {
        return Err(format!(
            "continuous portfolio replay coralys artifact hash mismatch: {CORALYS_EXEC_ARTIFACT_HASH}"
        ));
    }

    let universe: Vec<String> = universe.to_vec();

    // Resolve certified T (same as v0.1 baseline)
    let requested = DateTime::parse_from_rfc3339(PORTFOLIO_REPLAY_REQUESTED_CLOCK)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| format!("clock parse error: {e}"))?;

    let first_instrument = universe.first().ok_or("universe is empty")?;
    let first_bars = cache
        .get(first_instrument.as_str())
        .ok_or_else(|| format!("cache missing {first_instrument}"))?;
    let certified_t = latest_session_at_or_before(first_bars, requested)
        .ok_or_else(|| format!("no certified session for {first_instrument}"))?;

    // Collect all session timestamps from the first instrument's bars at or after certified_t.
    // These are the sessions we will simulate.
    let session_timestamps: Vec<DateTime<Utc>> = first_bars
        .iter()
        .filter_map(|b| {
            let ts = Utc.timestamp_opt(b.timestamp, 0).single()?;
            if ts >= certified_t { Some(ts) } else { None }
        })
        .collect();

    if session_timestamps.is_empty() {
        return Err("no sessions found at or after certified_t".into());
    }

    let mut pe2_arm = ContinuousPortfolioArm::new("pe2", PE2_ARM_CONTRACT, initial_capital_inr);
    let mut coralys_arm =
        ContinuousPortfolioArm::new("coralys_v0", CORALYS_ARM_CONTRACT, initial_capital_inr);

    let n_sessions = session_timestamps.len() as u32;

    // ── Session-by-session loop ───────────────────────────────────────────────
    for (session_idx, &session_t) in session_timestamps.iter().enumerate() {
        let session_idx_u32 = session_idx as u32;

        // Apply scheduled capital contributions
        for contrib in contributions {
            if contrib.session_index == session_idx_u32 {
                pe2_arm.cash_inr += contrib.amount_inr;
                coralys_arm.cash_inr += contrib.amount_inr;
            }
        }

        // Build mark prices for this session (close price of each instrument at session_t)
        let mut mark_prices: BTreeMap<String, f64> = BTreeMap::new();
        for inst in &universe {
            if let Some(bars) = cache.get(inst.as_str()) {
                if let Some(bar) = bars
                    .iter()
                    .filter(|b| Utc.timestamp_opt(b.timestamp, 0).single().map_or(false, |t| t <= session_t))
                    .last()
                {
                    mark_prices.insert(inst.clone(), bar.close);
                }
            }
        }

        // ── Step 1: Exit scan — call canonical first_exit_with_optional_stop ──
        // For each open lot, call the canonical exit function with the full bars
        // slice. If the result is not Observing, close the lot.

        struct PendingClose {
            trade_id: String,
            exit_price: f64,
            exit_time_str: String,
            exit_reason: ExitReason,
            holding_sessions: u32,
        }

        let mut pe2_closes: Vec<PendingClose> = Vec::new();
        let mut coralys_closes: Vec<PendingClose> = Vec::new();

        // P.E.2 exit scan
        for lot in pe2_arm.trade_log.iter().filter(|l| l.is_open()) {
            let (decision, intent) = match (&lot.sealed_decision, &lot.sealed_intent) {
                (Some(d), Some(i)) => (d, i),
                _ => continue,
            };
            let bars = match cache.get(&lot.instrument) {
                Some(b) => b.as_slice(),
                None => continue,
            };
            // P.E.2 arm: no stop authorized
            match first_exit_with_optional_stop(decision, intent, bars, None, false) {
                Ok(exit) => {
                    let reason = exit.exit_reason;
                    if !matches!(reason, ExitReason::Observing | ExitReason::NoTrade) {
                        if let (Some(ep), Some(et), Some(hs)) = (
                            exit.exit_price,
                            exit.exit_time.as_deref(),
                            exit.holding_sessions,
                        ) {
                            pe2_closes.push(PendingClose {
                                trade_id: lot.trade_id.clone(),
                                exit_price: ep,
                                exit_time_str: et.to_string(),
                                exit_reason: reason,
                                holding_sessions: hs,
                            });
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        // Coralys exit scan
        for lot in coralys_arm.trade_log.iter().filter(|l| l.is_open()) {
            let (decision, intent) = match (&lot.sealed_decision, &lot.sealed_intent) {
                (Some(d), Some(i)) => (d, i),
                _ => continue,
            };
            let bars = match cache.get(&lot.instrument) {
                Some(b) => b.as_slice(),
                None => continue,
            };
            // Coralys arm: stop authorized with risk_boundary
            match first_exit_with_optional_stop(decision, intent, bars, lot.stop_price, true) {
                Ok(exit) => {
                    let reason = exit.exit_reason;
                    if !matches!(reason, ExitReason::Observing | ExitReason::NoTrade) {
                        if let (Some(ep), Some(et), Some(hs)) = (
                            exit.exit_price,
                            exit.exit_time.as_deref(),
                            exit.holding_sessions,
                        ) {
                            coralys_closes.push(PendingClose {
                                trade_id: lot.trade_id.clone(),
                                exit_price: ep,
                                exit_time_str: et.to_string(),
                                exit_reason: reason,
                                holding_sessions: hs,
                            });
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        // ── Apply P.E.2 closes ────────────────────────────────────────────────
        let mut pe2_lots_closed = 0u32;
        for pending in &pe2_closes {
            if let Some(lot) = pe2_arm
                .trade_log
                .iter_mut()
                .find(|l| l.trade_id == pending.trade_id && l.is_open())
            {
                let ret = if lot.direction == "LONG" {
                    (pending.exit_price - lot.entry_price) / lot.entry_price
                } else {
                    (lot.entry_price - pending.exit_price) / lot.entry_price
                };
                let pnl = lot.allocation_inr * ret;
                let returned_cash = lot.allocation_inr + pnl;

                // Collect post-entry closes for TradePath
                let bars = cache.get(&lot.instrument).map(|v| v.as_slice()).unwrap_or(&[]);
                let lot_entry_t = DateTime::parse_from_rfc3339(&lot.entry_time)
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or(session_t);
                let exit_t = DateTime::parse_from_rfc3339(&pending.exit_time_str)
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or(session_t);
                let post_entry_closes: Vec<f64> = bars
                    .iter()
                    .filter_map(|b| {
                        let ts = Utc.timestamp_opt(b.timestamp, 0).single()?;
                        if ts > lot_entry_t && ts <= exit_t { Some(b.close) } else { None }
                    })
                    .collect();

                lot.exit_time = Some(pending.exit_time_str.clone());
                lot.exit_price = Some(pending.exit_price);
                lot.exit_reason = Some(pending.exit_reason);
                lot.holding_sessions = Some(pending.holding_sessions);
                lot.realized_return = Some(ret);
                lot.realized_pnl_inr = Some(pnl);
                lot.trade_path = Some(TradePath::compute(
                    lot.entry_price,
                    &post_entry_closes,
                    lot.direction == "LONG",
                    lot.stop_price,
                    lot.target_price,
                ));

                pe2_arm.cash_inr += returned_cash;
                pe2_arm.total_realized_pnl_inr += pnl;
                pe2_lots_closed += 1;
            }
        }

        // ── Apply Coralys closes ──────────────────────────────────────────────
        let mut coralys_lots_closed = 0u32;
        for pending in &coralys_closes {
            if let Some(lot) = coralys_arm
                .trade_log
                .iter_mut()
                .find(|l| l.trade_id == pending.trade_id && l.is_open())
            {
                let ret = if lot.direction == "LONG" {
                    (pending.exit_price - lot.entry_price) / lot.entry_price
                } else {
                    (lot.entry_price - pending.exit_price) / lot.entry_price
                };
                let pnl = lot.allocation_inr * ret;
                let returned_cash = lot.allocation_inr + pnl;

                let bars = cache.get(&lot.instrument).map(|v| v.as_slice()).unwrap_or(&[]);
                let lot_entry_t = DateTime::parse_from_rfc3339(&lot.entry_time)
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or(session_t);
                let exit_t = DateTime::parse_from_rfc3339(&pending.exit_time_str)
                    .map(|t| t.with_timezone(&Utc))
                    .unwrap_or(session_t);
                let post_entry_closes: Vec<f64> = bars
                    .iter()
                    .filter_map(|b| {
                        let ts = Utc.timestamp_opt(b.timestamp, 0).single()?;
                        if ts > lot_entry_t && ts <= exit_t { Some(b.close) } else { None }
                    })
                    .collect();

                lot.exit_time = Some(pending.exit_time_str.clone());
                lot.exit_price = Some(pending.exit_price);
                lot.exit_reason = Some(pending.exit_reason);
                lot.holding_sessions = Some(pending.holding_sessions);
                lot.realized_return = Some(ret);
                lot.realized_pnl_inr = Some(pnl);
                lot.trade_path = Some(TradePath::compute(
                    lot.entry_price,
                    &post_entry_closes,
                    lot.direction == "LONG",
                    lot.stop_price,
                    lot.target_price,
                ));

                coralys_arm.cash_inr += returned_cash;
                coralys_arm.total_realized_pnl_inr += pnl;
                coralys_lots_closed += 1;
            }
        }

        // ── Step 2: Entry scan — generate decisions and open new lots ─────────
        // For each instrument, generate a C3-002 decision at session_t using the
        // canonical decision pipeline. If LONG/SHORT and cash available, open a
        // new lot with a canonical SealedExecutionIntent.

        struct NewLotPlan {
            instrument: String,
            decision: SealedDecisionRecord,
            entry_price: f64,
            atr: Option<f64>,
            direction: String,
        }

        let mut new_lot_plans: Vec<NewLotPlan> = Vec::new();

        for inst in &universe {
            let bars = match cache.get(inst.as_str()) {
                Some(b) => b,
                None => continue,
            };

            // Generate C3-002 decision at session_t using the canonical pipeline
            let decision = match generate_historical_replay_decision(artifact, inst.as_str(), bars, session_t) {
                Ok(d) => d,
                Err(_) => continue,
            };

            if decision.action == DecisionAction::NoTrade {
                continue;
            }

            // Entry price = close at session_t
            let entry_price = match entry_close(bars, session_t) {
                Some(p) => p,
                None => continue,
            };

            let known = decision_time_bars(bars, session_t);
            let atr = atr_14_at_t(&known, session_t);

            let direction = match decision.action {
                DecisionAction::Long => "LONG",
                DecisionAction::Short => "SHORT",
                DecisionAction::NoTrade => unreachable!(),
            };

            new_lot_plans.push(NewLotPlan {
                instrument: inst.clone(),
                decision,
                entry_price,
                atr,
                direction: direction.to_string(),
            });
        }

        let n_eligible = new_lot_plans.len();

        // ── Open P.E.2 lots ───────────────────────────────────────────────────
        let mut pe2_lots_opened = 0u32;
        if n_eligible > 0 && pe2_arm.cash_inr >= MIN_LOT_ALLOCATION_INR {
            let alloc = allocation_model.per_lot_alloc(pe2_arm.cash_inr, n_eligible).max(0.0);
            if alloc >= MIN_LOT_ALLOCATION_INR {
                for plan in &new_lot_plans {
                    // Build canonical P.E.2 execution intent
                    let intent = match seal_execution_intent(
                        &plan.decision,
                        plan.entry_price,
                        EXECUTION_TARGET_PCT,
                    ) {
                        Ok(i) => i,
                        Err(_) => continue,
                    };

                    let seq = pe2_arm.next_decision_sequence(&plan.instrument);
                    let trade_id = format!(
                        "pe2-{}-seq{}",
                        plan.instrument.replace(".NS", ""),
                        seq
                    );

                    pe2_arm.trade_log.push(TradeLot {
                        trade_id,
                        decision_id: plan.decision.decision_id.clone(),
                        instrument: plan.instrument.clone(),
                        direction: plan.direction.clone(),
                        entry_time: session_t.to_rfc3339(),
                        entry_price: plan.entry_price,
                        allocation_inr: alloc,
                        units: alloc / plan.entry_price,
                        target_pct: intent.target_pct,
                        target_price: intent.target_price,
                        stop_pct: None,
                        stop_price: None,
                        exit_time: None,
                        exit_price: None,
                        exit_reason: None,
                        holding_sessions: None,
                        realized_return: None,
                        realized_pnl_inr: None,
                        decision_sequence: seq,
                        trade_path: None,
                        sealed_decision: Some(plan.decision.clone()),
                        sealed_intent: Some(intent),
                    });
                    pe2_arm.cash_inr -= alloc;
                    pe2_lots_opened += 1;
                }
            }
        }

        // ── Open Coralys lots ─────────────────────────────────────────────────
        let mut coralys_lots_opened = 0u32;
        if n_eligible > 0 && coralys_arm.cash_inr >= MIN_LOT_ALLOCATION_INR {
            let alloc = allocation_model.per_lot_alloc(coralys_arm.cash_inr, n_eligible).max(0.0);
            if alloc >= MIN_LOT_ALLOCATION_INR {
                for plan in &new_lot_plans {
                    let coralys_result = match seal_coralys_execution_intent(
                        &plan.instrument,
                        &plan.decision.decision_time,
                        &plan.decision.decision_time,
                        &plan.direction,
                        plan.entry_price,
                        plan.atr,
                        &plan.decision.state.trend,
                        &plan.decision.state.momentum,
                        &plan.decision.state.state_hash,
                    ) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };

                    match coralys_result {
                        CoralysExecutionResult::Intent(ci) => {
                            // Build a SealedExecutionIntent for canonical exit scanning
                            let intent = SealedExecutionIntent {
                                decision_id: plan.decision.decision_id.clone(),
                                instrument: plan.instrument.clone(),
                                decision_time: plan.decision.decision_time.clone(),
                                action: plan.direction.clone(),
                                entry_price: plan.entry_price,
                                target_pct: ci.target_pct,
                                target_price: ci.target_price,
                                stop_pct: Some(ci.risk_pct),
                                stop_price: Some(ci.risk_boundary),
                                max_holding_sessions: MAXIMUM_HOLD_SESSIONS,
                                target_source: ci.target_basis.clone(),
                                execution_contract: CORALYS_ARM_CONTRACT.to_string(),
                                sealed_at_t: true,
                                intent_hash: String::new(),
                            };

                            let seq = coralys_arm.next_decision_sequence(&plan.instrument);
                            let trade_id = format!(
                                "coralys-{}-seq{}",
                                plan.instrument.replace(".NS", ""),
                                seq
                            );

                            coralys_arm.trade_log.push(TradeLot {
                                trade_id,
                                decision_id: plan.decision.decision_id.clone(),
                                instrument: plan.instrument.clone(),
                                direction: plan.direction.clone(),
                                entry_time: session_t.to_rfc3339(),
                                entry_price: plan.entry_price,
                                allocation_inr: alloc,
                                units: alloc / plan.entry_price,
                                target_pct: ci.target_pct,
                                target_price: ci.target_price,
                                stop_pct: Some(ci.risk_pct),
                                stop_price: Some(ci.risk_boundary),
                                exit_time: None,
                                exit_price: None,
                                exit_reason: None,
                                holding_sessions: None,
                                realized_return: None,
                                realized_pnl_inr: None,
                                decision_sequence: seq,
                                trade_path: None,
                                sealed_decision: Some(plan.decision.clone()),
                                sealed_intent: Some(intent),
                            });
                            coralys_arm.cash_inr -= alloc;
                            coralys_lots_opened += 1;
                        }
                        CoralysExecutionResult::Invalid { .. } => {
                            // ATR unavailable — skip this instrument for Coralys arm
                        }
                    }
                }
            }
        }

        // ── Step 3: Record session snapshots ──────────────────────────────────
        let pe2_equity = pe2_arm.total_equity_inr(&mark_prices);
        pe2_arm.update_drawdown(pe2_equity);
        let pe2_snap = PortfolioSnapshot {
            session_time: session_t.to_rfc3339(),
            session_index: session_idx_u32,
            cash_inr: pe2_arm.cash_inr,
            invested_capital_inr: pe2_arm.invested_capital_inr(),
            market_value_inr: pe2_arm.market_value_inr(&mark_prices),
            total_equity_inr: pe2_equity,
            realized_pnl_inr: pe2_arm.total_realized_pnl_inr,
            unrealized_pnl_inr: pe2_arm.unrealized_pnl_inr(&mark_prices),
            open_lots: pe2_arm.open_lots_count() as u32,
            lots_opened_this_session: pe2_lots_opened,
            lots_closed_this_session: pe2_lots_closed,
            instrument_exposure: pe2_arm.instrument_exposure(&mark_prices),
        };
        pe2_arm.session_snapshots.push(pe2_snap);

        let coralys_equity = coralys_arm.total_equity_inr(&mark_prices);
        coralys_arm.update_drawdown(coralys_equity);
        let coralys_snap = PortfolioSnapshot {
            session_time: session_t.to_rfc3339(),
            session_index: session_idx_u32,
            cash_inr: coralys_arm.cash_inr,
            invested_capital_inr: coralys_arm.invested_capital_inr(),
            market_value_inr: coralys_arm.market_value_inr(&mark_prices),
            total_equity_inr: coralys_equity,
            realized_pnl_inr: coralys_arm.total_realized_pnl_inr,
            unrealized_pnl_inr: coralys_arm.unrealized_pnl_inr(&mark_prices),
            open_lots: coralys_arm.open_lots_count() as u32,
            lots_opened_this_session: coralys_lots_opened,
            lots_closed_this_session: coralys_lots_closed,
            instrument_exposure: coralys_arm.instrument_exposure(&mark_prices),
        };
        coralys_arm.session_snapshots.push(coralys_snap);
    }
    // ── End session loop ──────────────────────────────────────────────────────

    // Final mark prices (last session)
    let mut final_marks: BTreeMap<String, f64> = BTreeMap::new();
    for inst in &universe {
        if let Some(bars) = cache.get(inst.as_str()) {
            if let Some(bar) = bars.last() {
                final_marks.insert(inst.clone(), bar.close);
            }
        }
    }

    let pe2_summary = build_continuous_arm_summary(&pe2_arm, &final_marks, n_sessions);
    let coralys_summary = build_continuous_arm_summary(&coralys_arm, &final_marks, n_sessions);

    Ok(ContinuousPortfolioLedger {
        path_kind: CONTINUOUS_REPLAY_VERSION.to_string(),
        experiment_id: CONTINUOUS_EXPERIMENT_ID.to_string(),
        start_clock: PORTFOLIO_REPLAY_REQUESTED_CLOCK.to_string(),
        certified_t: certified_t.to_rfc3339(),
        initial_capital_inr: INITIAL_CAPITAL_INR,
        coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
        c3_002_artifact_hash: RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH.to_string(),
        pe2_arm,
        coralys_arm,
        pe2_summary,
        coralys_summary,
        n_sessions_simulated: n_sessions,
        universe,
        integrity_note: concat!(
            "v0.2.1 continuous lifecycle -- multiple lots per instrument allowed, ",
            "capital recycled on exit, session-by-session loop over full P.E.2 period. ",
            "Canonical execution pipeline: generate_historical_replay_decision -> ",
            "seal_execution_intent/seal_coralys_execution_intent -> ",
            "first_exit_with_optional_stop. ",
            "Execution arms frozen: P.E.2 = +5% target / no stop / 20s max; ",
            "Coralys v0 = ATR/TMV target / risk_boundary stop / 20s max."
        ).to_string(),
    })
}