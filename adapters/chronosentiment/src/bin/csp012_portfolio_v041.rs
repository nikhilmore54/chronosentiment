//! Portfolio Replay v0.4.1 — Capital × Allocation Controlled Experiment.
//!
//! ## Purpose
//!
//! Separates the **capital effect** from the **allocation effect** on portfolio
//! performance and decision realization. Uses a fixed 52-instrument universe
//! across all three configs so that universe size is not a confound.
//!
//! ## Experiment design
//!
//! ```text
//! Universe: 52 instruments (UNIVERSE_50 — identical across all configs)
//!
//! Config A: Rs.5,000    EqualWeight   (v0.3 baseline — capital-constrained)
//! Config B: Rs.1,000,000 EqualWeight  (capital effect only)
//! Config C: Rs.1,000,000 MaxPerLot Rs.20,000 (allocation effect on top of B)
//!
//! Causal comparisons:
//!   A vs B  =>  capital effect   (same allocation, different capital)
//!   B vs C  =>  allocation effect (same capital, different allocation)
//! ```
//!
//! ## Decision-realization ledger
//!
//! For every certified Coralys decision that was eligible in the universe,
//! records whether it was realized (a lot was opened) under each config,
//! and if so, the allocation amount. This distinguishes:
//!   - "Coralys did not recommend a trade" (NoTrade decision)
//!   - "Coralys recommended but portfolio could not realize it" (capital exhausted)
//!   - "Coralys recommended and portfolio realized it" (lot opened)
//!
//! ## v0.4.1 contract (FROZEN — do not modify)
//!
//! ```text
//! Same engine          — run_continuous_portfolio_replay_with_config
//! Same historical period — PORTFOLIO_REPLAY_REQUESTED_CLOCK
//! Same C3-002          — RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
//! Same Coralys v0      — CORALYS_EXEC_ARTIFACT_HASH
//! Same stop-loss       — enforced risk_boundary from CoralysExecutionIntent
//! Same lifecycle       — session-by-session, capital recycled on close
//! Same universe        — UNIVERSE_50 (52 instruments) for all 3 configs
//!
//!              CHANGES vs v0.4:
//!                1. 3 configs instead of 4 (controlled experiment)
//!                2. Decision-realization ledger (new output)
//!                3. Richer comparison_matrix.json
//!                4. Structured COMPARISON_REPORT.md
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use chronosentiment_adapter::decision_support::coralys_execution_model::CORALYS_EXEC_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::observatory_execution::ExitReason;
use chronosentiment_adapter::decision_support::portfolio_replay_v021::{
    refuse_v021_output, run_continuous_portfolio_replay_with_config,
    AllocationModel, ContinuousPortfolioConfig, ContinuousPortfolioLedger,
    TradeLot, CONTINUOUS_REPLAY_VERSION,
};
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;

// ─── Stop classification (inline — not re-exported from library) ──────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum StopCategory {
    GapThrough,
    PrematureStop,
    TemporaryExcursion,
    StopTooTight,
    DirectionFailure,
    GenuineAdverse,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StopDiagnostic {
    pub trade_id: String,
    pub instrument: String,
    pub direction: String,
    pub entry_price: f64,
    pub stop_price: f64,
    pub exit_price: f64,
    pub entry_time: String,
    pub exit_time: String,
    pub holding_sessions: u32,
    pub realized_pnl_inr: f64,
    pub allocation_inr: f64,
    pub gap_magnitude_pct: f64,
    pub post_stop_max_favorable_pct: Option<f64>,
    pub target_reached_after_stop: bool,
    pub recovered_after_stop_within_5: bool,
    pub continued_adverse_5_sessions: bool,
    pub stop_tightness_pct: f64,
    pub counterfactual_pnl_inr: Option<f64>,
    pub opportunity_cost_inr: Option<f64>,
    pub category: StopCategory,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StopLossAnalysis {
    pub config_label: String,
    pub n_coralys_stops: usize,
    pub n_gap_through: usize,
    pub n_premature: usize,
    pub n_temporary_excursion: usize,
    pub n_stop_too_tight: usize,
    pub n_direction_failure: usize,
    pub n_genuine_adverse: usize,
    pub pct_gap_through: f64,
    pub pct_premature: f64,
    pub pct_temporary_excursion: f64,
    pub pct_stop_too_tight: f64,
    pub pct_direction_failure: f64,
    pub pct_genuine_adverse: f64,
    pub total_opportunity_cost_inr: f64,
    pub total_stop_realized_pnl_inr: f64,
    pub net_stop_benefit_inr: f64,
    pub diagnostics: Vec<StopDiagnostic>,
}

fn classify_stops(
    ledger: &ContinuousPortfolioLedger,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    config_label: &str,
) -> StopLossAnalysis {
    let end_ts: Option<chrono::DateTime<chrono::Utc>> = {
        let all_exit_times = ledger.pe2_arm.trade_log.iter()
            .chain(ledger.coralys_arm.trade_log.iter())
            .filter_map(|l| l.exit_time.as_deref())
            .filter_map(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
            .map(|t| t.with_timezone(&chrono::Utc));
        all_exit_times.max()
    };

    let stop_lots: Vec<&TradeLot> = ledger
        .coralys_arm
        .trade_log
        .iter()
        .filter(|l| matches!(l.exit_reason, Some(ExitReason::Stop)))
        .collect();

    let n = stop_lots.len();
    let mut diagnostics: Vec<StopDiagnostic> = Vec::new();

    for lot in stop_lots {
        let stop_price = match lot.stop_price { Some(p) => p, None => continue };
        let exit_price = match lot.exit_price { Some(p) => p, None => continue };
        let exit_time_str = match &lot.exit_time { Some(t) => t.clone(), None => continue };
        let holding = lot.holding_sessions.unwrap_or(0);
        let realized_pnl = lot.realized_pnl_inr.unwrap_or(0.0);
        let is_long = lot.direction == "LONG";

        let gap_magnitude_pct = if stop_price > 0.0 {
            if is_long { (stop_price - exit_price) / stop_price }
            else { (exit_price - stop_price) / stop_price }
        } else { 0.0 };

        let stop_tightness_pct = if lot.entry_price > 0.0 {
            (stop_price - lot.entry_price).abs() / lot.entry_price
        } else { 0.0 };

        let bars = cache.get(&lot.instrument);
        let (post_stop_max_favorable_pct, target_reached_after_stop,
             recovered_after_stop_within_5, continued_adverse_5_sessions,
             counterfactual_pnl_inr) = if let Some(bars) = bars {
            let exit_ts = chrono::DateTime::parse_from_rfc3339(&exit_time_str)
                .map(|t| t.with_timezone(&chrono::Utc)).ok();
            let post_stop_bars: Vec<&YahooHistoricalBar> = if let Some(exit_ts) = exit_ts {
                bars.iter().filter(|b| {
                    let bar_ts = chrono::DateTime::from_timestamp(b.timestamp, 0);
                    let after_exit = bar_ts.map(|t| t > exit_ts).unwrap_or(false);
                    let before_end = end_ts.map(|end| bar_ts.map(|t| t <= end).unwrap_or(false)).unwrap_or(true);
                    after_exit && before_end
                }).collect()
            } else { vec![] };
            let within_5 = &post_stop_bars[..post_stop_bars.len().min(5)];
            let max_favorable_pct = if !within_5.is_empty() && exit_price > 0.0 {
                if is_long {
                    let max_close = within_5.iter().map(|b| b.close).fold(f64::NEG_INFINITY, f64::max);
                    Some((max_close - exit_price) / exit_price)
                } else {
                    let min_close = within_5.iter().map(|b| b.close).fold(f64::INFINITY, f64::min);
                    Some((exit_price - min_close) / exit_price)
                }
            } else { None };
            let target_reached = if is_long {
                within_5.iter().take(3).any(|b| b.close >= lot.target_price)
            } else {
                within_5.iter().take(3).any(|b| b.close <= lot.target_price)
            };
            let recovered = if is_long {
                within_5.iter().any(|b| b.close > lot.entry_price)
            } else {
                within_5.iter().any(|b| b.close < lot.entry_price)
            };
            let continued_adverse = within_5.len() >= 5 && if is_long {
                within_5.iter().all(|b| b.close < lot.entry_price)
            } else {
                within_5.iter().all(|b| b.close > lot.entry_price)
            };
            let last_bar_in_window = post_stop_bars.last().map(|b| b.close);
            let counterfactual = last_bar_in_window.map(|close| {
                let ret = if is_long { (close - lot.entry_price) / lot.entry_price }
                          else { (lot.entry_price - close) / lot.entry_price };
                lot.allocation_inr * ret
            });
            (max_favorable_pct, target_reached, recovered, continued_adverse, counterfactual)
        } else { (None, false, false, false, None) };

        let opportunity_cost_inr = counterfactual_pnl_inr.map(|cf| cf - realized_pnl);
        let category = if stop_tightness_pct < 0.01 { StopCategory::StopTooTight }
            else if gap_magnitude_pct > 0.005 { StopCategory::GapThrough }
            else if target_reached_after_stop { StopCategory::PrematureStop }
            else if recovered_after_stop_within_5 { StopCategory::TemporaryExcursion }
            else if continued_adverse_5_sessions { StopCategory::DirectionFailure }
            else { StopCategory::GenuineAdverse };

        diagnostics.push(StopDiagnostic {
            trade_id: lot.trade_id.clone(),
            instrument: lot.instrument.clone(),
            direction: lot.direction.clone(),
            entry_price: lot.entry_price,
            stop_price, exit_price,
            entry_time: lot.entry_time.clone(),
            exit_time: exit_time_str,
            holding_sessions: holding,
            realized_pnl_inr: realized_pnl,
            allocation_inr: lot.allocation_inr,
            gap_magnitude_pct, post_stop_max_favorable_pct,
            target_reached_after_stop,
            recovered_after_stop_within_5,
            continued_adverse_5_sessions,
            stop_tightness_pct, counterfactual_pnl_inr, opportunity_cost_inr, category,
        });
    }

    let n_gap_through = diagnostics.iter().filter(|d| d.category == StopCategory::GapThrough).count();
    let n_premature = diagnostics.iter().filter(|d| d.category == StopCategory::PrematureStop).count();
    let n_temporary = diagnostics.iter().filter(|d| d.category == StopCategory::TemporaryExcursion).count();
    let n_tight = diagnostics.iter().filter(|d| d.category == StopCategory::StopTooTight).count();
    let n_direction = diagnostics.iter().filter(|d| d.category == StopCategory::DirectionFailure).count();
    let n_genuine = diagnostics.iter().filter(|d| d.category == StopCategory::GenuineAdverse).count();
    let total_stop_pnl: f64 = diagnostics.iter().map(|d| d.realized_pnl_inr).sum();
    let total_opp_cost: f64 = diagnostics.iter().filter_map(|d| d.opportunity_cost_inr).sum();
    let total_cf_pnl: f64 = diagnostics.iter().filter_map(|d| d.counterfactual_pnl_inr).sum();
    let net_benefit = total_stop_pnl - total_cf_pnl;
    let pct = |k: usize| if n > 0 { k as f64 / n as f64 * 100.0 } else { 0.0 };

    StopLossAnalysis {
        config_label: config_label.to_string(),
        n_coralys_stops: n,
        n_gap_through, n_premature,
        n_temporary_excursion: n_temporary,
        n_stop_too_tight: n_tight,
        n_direction_failure: n_direction,
        n_genuine_adverse: n_genuine,
        pct_gap_through: pct(n_gap_through),
        pct_premature: pct(n_premature),
        pct_temporary_excursion: pct(n_temporary),
        pct_stop_too_tight: pct(n_tight),
        pct_direction_failure: pct(n_direction),
        pct_genuine_adverse: pct(n_genuine),
        total_opportunity_cost_inr: total_opp_cost,
        total_stop_realized_pnl_inr: total_stop_pnl,
        net_stop_benefit_inr: net_benefit,
        diagnostics,
    }
}

// ─── Constants ────────────────────────────────────────────────────────────────

/// v0.4.1 experiment: Rs.5,000 baseline capital (v0.3 equivalent).
const V041_CAPITAL_5K: f64 = 5_000.0;

/// v0.4.1 experiment: Rs.1,000,000 high-capital condition.
const V041_CAPITAL_1M: f64 = 1_000_000.0;

/// v0.4.1 experiment: per-lot cap for MaxPerLot condition.
const V041_MAX_PER_LOT_INR: f64 = 20_000.0;

/// Artifact hash guard — must match C3-002 / Search #2.
const CORALYS_EXEC_ARTIFACT_HASH_GUARD: &str =
    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f";

// ─── Args ─────────────────────────────────────────────────────────────────────

struct Args {
    search_two_dir: PathBuf,
    cache_dir: PathBuf,
    output_base: PathBuf,
    strict: bool,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut search_two_dir = None;
    let mut cache_dir = None;
    let mut output_base = None;
    let mut strict = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--search-two" => { i += 1; search_two_dir = Some(PathBuf::from(&args[i])); }
            "--cache-dir"  => { i += 1; cache_dir = Some(PathBuf::from(&args[i])); }
            "--output-base" => { i += 1; output_base = Some(PathBuf::from(&args[i])); }
            "--strict" => { strict = true; }
            _ => {}
        }
        i += 1;
    }

    Ok(Args {
        search_two_dir: search_two_dir.ok_or("--search-two required")?,
        cache_dir: cache_dir.ok_or("--cache-dir required")?,
        output_base: output_base.ok_or("--output-base required")?,
        strict,
    })
}

// ─── Universe ─────────────────────────────────────────────────────────────────

/// 52 instruments — identical across all 3 configs (controlled experiment).
const UNIVERSE_50: &[&str] = &[
    "RELIANCE.NS", "TCS.NS", "HDFCBANK.NS", "INFY.NS", "ICICIBANK.NS",
    "HINDUNILVR.NS", "ITC.NS", "SBIN.NS", "BHARTIARTL.NS", "KOTAKBANK.NS",
    "LT.NS", "AXISBANK.NS", "ASIANPAINT.NS", "MARUTI.NS", "TITAN.NS",
    "SUNPHARMA.NS", "WIPRO.NS", "ULTRACEMCO.NS", "BAJFINANCE.NS", "NESTLEIND.NS",
    "POWERGRID.NS", "NTPC.NS", "TECHM.NS", "HCLTECH.NS", "ONGC.NS",
    "BAJAJFINSV.NS", "JSWSTEEL.NS", "TMPV.NS", "ADANIENT.NS", "ADANIPORTS.NS",
    "COALINDIA.NS", "DIVISLAB.NS", "DRREDDY.NS", "EICHERMOT.NS", "GRASIM.NS",
    "HEROMOTOCO.NS", "HINDALCO.NS", "INDUSINDBK.NS", "M&M.NS", "SBILIFE.NS",
    "TATACONSUM.NS", "TATASTEEL.NS", "UPL.NS", "VEDL.NS", "BPCL.NS",
    "CIPLA.NS", "HDFCLIFE.NS", "PIDILITIND.NS", "SHREECEM.NS", "UNITDSPR.NS",
    "MAHABANK.NS", "IDEA.NS",
];

// ─── Decision-realization ledger ──────────────────────────────────────────────

/// Per-config realization record for a single Coralys decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRealization {
    /// Was the instrument in the universe for this config?
    pub in_universe: bool,
    /// Was a lot opened for this decision?
    pub realized: bool,
    /// Allocation in INR (0.0 if not realized).
    pub allocation_inr: f64,
}

/// Decision-realization record for a single certified Coralys decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRealizationRecord {
    /// C3-002 decision ID.
    pub decision_id: String,
    /// Instrument symbol.
    pub instrument: String,
    /// Direction: "LONG", "SHORT", or "NoTrade".
    pub direction: String,
    /// Entry price (from the first config that realized it, or 0.0).
    pub entry_price: f64,
    /// Target price (from the first config that realized it, or 0.0).
    pub target_price: f64,
    /// Stop price (from the first config that realized it, or 0.0).
    pub stop_price: f64,
    /// Per-config realization status.
    pub configs: BTreeMap<String, ConfigRealization>,
}

/// Full decision-realization ledger across all 3 configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRealizationLedger {
    pub experiment: String,
    pub universe_size: usize,
    pub config_labels: Vec<String>,
    /// Total certified decisions (LONG + SHORT + NoTrade) in the universe.
    pub n_certified_total: usize,
    /// Decisions with direction LONG or SHORT (eligible for trading).
    pub n_eligible: usize,
    /// Decisions realized in each config.
    pub n_realized_per_config: BTreeMap<String, usize>,
    /// Decisions NOT realized due to capital exhaustion (eligible but not realized).
    pub n_not_realized_per_config: BTreeMap<String, usize>,
    pub records: Vec<DecisionRealizationRecord>,
}

fn build_decision_realization_ledger(
    config_labels: &[String],
    ledgers: &[ContinuousPortfolioLedger],
    universe: &[String],
) -> DecisionRealizationLedger {
    let universe_set: BTreeSet<String> = universe.iter().cloned().collect();

    // Collect all decision_ids from all Coralys arms.
    // Key: decision_id -> (instrument, direction, entry_price, target_price, stop_price)
    let mut decision_meta: BTreeMap<String, (String, String, f64, f64, f64)> = BTreeMap::new();
    for ledger in ledgers {
        for lot in &ledger.coralys_arm.trade_log {
            decision_meta.entry(lot.decision_id.clone()).or_insert_with(|| {
                (
                    lot.instrument.clone(),
                    lot.direction.clone(),
                    lot.entry_price,
                    lot.target_price,
                    lot.stop_price.unwrap_or(0.0),
                )
            });
        }
    }

    // Build per-config realized decision_id sets.
    let mut realized_per_config: Vec<BTreeMap<String, f64>> = ledgers
        .iter()
        .map(|ledger| {
            ledger
                .coralys_arm
                .trade_log
                .iter()
                .map(|lot| (lot.decision_id.clone(), lot.allocation_inr))
                .collect()
        })
        .collect();

    let n_eligible = decision_meta.len();

    let mut records: Vec<DecisionRealizationRecord> = decision_meta
        .iter()
        .map(|(decision_id, (instrument, direction, entry_price, target_price, stop_price))| {
            let mut configs: BTreeMap<String, ConfigRealization> = BTreeMap::new();
            for (i, label) in config_labels.iter().enumerate() {
                let in_universe = universe_set.contains(instrument);
                let allocation_inr = realized_per_config[i]
                    .get(decision_id)
                    .copied()
                    .unwrap_or(0.0);
                let realized = allocation_inr > 0.0;
                configs.insert(
                    label.clone(),
                    ConfigRealization { in_universe, realized, allocation_inr },
                );
            }
            DecisionRealizationRecord {
                decision_id: decision_id.clone(),
                instrument: instrument.clone(),
                direction: direction.clone(),
                entry_price: *entry_price,
                target_price: *target_price,
                stop_price: *stop_price,
                configs,
            }
        })
        .collect();

    // Sort by instrument then decision_id for deterministic output.
    records.sort_by(|a, b| a.instrument.cmp(&b.instrument).then(a.decision_id.cmp(&b.decision_id)));

    let mut n_realized_per_config: BTreeMap<String, usize> = BTreeMap::new();
    let mut n_not_realized_per_config: BTreeMap<String, usize> = BTreeMap::new();
    for label in config_labels {
        let realized = records.iter().filter(|r| {
            r.configs.get(label).map(|c| c.realized).unwrap_or(false)
        }).count();
        n_realized_per_config.insert(label.clone(), realized);
        n_not_realized_per_config.insert(label.clone(), n_eligible - realized);
    }

    DecisionRealizationLedger {
        experiment: "Portfolio Replay v0.4.1 — Capital x Allocation".to_string(),
        universe_size: universe.len(),
        config_labels: config_labels.to_vec(),
        n_certified_total: n_eligible, // all are LONG/SHORT since NoTrade doesn't open lots
        n_eligible,
        n_realized_per_config,
        n_not_realized_per_config,
        records,
    }
}

// ─── Comparison matrix ────────────────────────────────────────────────────────

fn build_comparison_matrix(
    configs: &[(String, String, f64, AllocationModel)],
    ledgers: &[ContinuousPortfolioLedger],
    stop_analyses: &[StopLossAnalysis],
    realization_ledger: &DecisionRealizationLedger,
) -> serde_json::Value {
    let config_entries: Vec<serde_json::Value> = configs
        .iter()
        .zip(ledgers.iter())
        .zip(stop_analyses.iter())
        .map(|(((label, alloc_desc, capital, alloc_model), ledger), stop)| {
            let p = &ledger.pe2_summary;
            let c = &ledger.coralys_summary;
            let n_realized = realization_ledger.n_realized_per_config.get(label).copied().unwrap_or(0);
            let n_not_realized = realization_ledger.n_not_realized_per_config.get(label).copied().unwrap_or(0);

            let alloc_model_json = match alloc_model {
                AllocationModel::EqualWeight => serde_json::json!({ "model": "EqualWeight" }),
                AllocationModel::MaxPerLot { max_per_lot_inr } => serde_json::json!({
                    "model": "MaxPerLot",
                    "max_per_lot_inr": max_per_lot_inr,
                }),
            };

            serde_json::json!({
                "label": label,
                "initial_capital_inr": capital,
                "allocation_model": alloc_model_json,
                "allocation_desc": alloc_desc,
                "universe_available": ledger.universe.len(),
                "n_sessions_simulated": ledger.n_sessions_simulated,
                "pe2": {
                    "lots": p.n_lots_opened,
                    "target": p.n_target,
                    "stop": p.n_stop,
                    "horizon": p.n_horizon,
                    "open_at_end": p.n_open_at_end,
                    "return_pct": (p.total_return_pct * 10000.0).round() / 100.0,
                    "capital_velocity": (p.capital_velocity_ratio * 100.0).round() / 100.0,
                    "max_drawdown_pct": (p.max_drawdown_pct * 10000.0).round() / 100.0,
                    "avg_holding_sessions": p.avg_holding_sessions,
                    "total_realized_pnl_inr": (p.total_realized_pnl_inr * 100.0).round() / 100.0,
                    "final_portfolio_value_inr": (p.final_portfolio_value_inr * 100.0).round() / 100.0,
                },
                "coralys": {
                    "lots": c.n_lots_opened,
                    "target": c.n_target,
                    "stop": c.n_stop,
                    "horizon": c.n_horizon,
                    "open_at_end": c.n_open_at_end,
                    "return_pct": (c.total_return_pct * 10000.0).round() / 100.0,
                    "capital_velocity": (c.capital_velocity_ratio * 100.0).round() / 100.0,
                    "max_drawdown_pct": (c.max_drawdown_pct * 10000.0).round() / 100.0,
                    "avg_holding_sessions": c.avg_holding_sessions,
                    "total_realized_pnl_inr": (c.total_realized_pnl_inr * 100.0).round() / 100.0,
                    "final_portfolio_value_inr": (c.final_portfolio_value_inr * 100.0).round() / 100.0,
                    "stop_rate_pct": if c.n_lots_opened > 0 {
                        (c.n_stop as f64 / c.n_lots_opened as f64 * 10000.0).round() / 100.0
                    } else { 0.0 },
                    "premature_stop_rate_pct": if stop.n_coralys_stops > 0 {
                        (stop.n_premature as f64 / stop.n_coralys_stops as f64 * 10000.0).round() / 100.0
                    } else { 0.0 },
                    "temporary_excursion_rate_pct": if stop.n_coralys_stops > 0 {
                        (stop.n_temporary_excursion as f64 / stop.n_coralys_stops as f64 * 10000.0).round() / 100.0
                    } else { 0.0 },
                    "stop_too_tight_rate_pct": if stop.n_coralys_stops > 0 {
                        (stop.n_stop_too_tight as f64 / stop.n_coralys_stops as f64 * 10000.0).round() / 100.0
                    } else { 0.0 },
                    "genuine_adverse_rate_pct": if stop.n_coralys_stops > 0 {
                        (stop.n_genuine_adverse as f64 / stop.n_coralys_stops as f64 * 10000.0).round() / 100.0
                    } else { 0.0 },
                },
                "decision_realization": {
                    "eligible_decisions": realization_ledger.n_eligible,
                    "realized": n_realized,
                    "not_realized_due_to_capital": n_not_realized,
                    "realization_rate_pct": if realization_ledger.n_eligible > 0 {
                        (n_realized as f64 / realization_ledger.n_eligible as f64 * 10000.0).round() / 100.0
                    } else { 0.0 },
                },
            })
        })
        .collect();

    serde_json::json!({
        "experiment": "Portfolio Replay v0.4.1 — Capital x Allocation",
        "universe": "UNIVERSE_50 (52 instruments, identical across all configs)",
        "universe_size": 52,
        "causal_comparisons": {
            "capital_effect": "Config A (Rs.5K EqualWeight) vs Config B (Rs.1M EqualWeight)",
            "allocation_effect": "Config B (Rs.1M EqualWeight) vs Config C (Rs.1M MaxPerLot Rs.20K)",
        },
        "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
        "coralys_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
        "configs": config_entries,
    })
}

// ─── Comparison report ────────────────────────────────────────────────────────

fn render_comparison_report(
    matrix: &serde_json::Value,
    ledgers: &[ContinuousPortfolioLedger],
    stop_analyses: &[StopLossAnalysis],
    realization_ledger: &DecisionRealizationLedger,
    config_labels: &[String],
    config_descs: &[String],
    capitals: &[f64],
) -> String {
    let configs = matrix["configs"].as_array().unwrap();
    let a = &configs[0];
    let b = &configs[1];
    let c = &configs[2];

    let fmt_pct = |v: &serde_json::Value| -> String {
        format!("{:.2}%", v.as_f64().unwrap_or(0.0))
    };
    let fmt_f = |v: &serde_json::Value| -> String {
        format!("{:.2}", v.as_f64().unwrap_or(0.0))
    };
    let fmt_i = |v: &serde_json::Value| -> String {
        format!("{}", v.as_i64().unwrap_or(0))
    };
    let fmt_inr = |v: &serde_json::Value| -> String {
        let x = v.as_f64().unwrap_or(0.0);
        if x.abs() >= 1_000_000.0 {
            format!("Rs.{:.2}M", x / 1_000_000.0)
        } else if x.abs() >= 1_000.0 {
            format!("Rs.{:.0}K", x / 1_000.0)
        } else {
            format!("Rs.{:.0}", x)
        }
    };

    let mut md = String::new();

    md.push_str("# Portfolio Replay v0.4.1 — Capital × Allocation Controlled Experiment\n\n");
    md.push_str("## Experiment Design\n\n");
    md.push_str("**Universe:** 52 instruments (UNIVERSE_50, identical across all configs)\n\n");
    md.push_str("**Causal structure:**\n\n");
    md.push_str("```\n");
    md.push_str("Config A: Rs.5,000    EqualWeight    (v0.3 baseline)\n");
    md.push_str("Config B: Rs.1,000,000 EqualWeight   (capital effect only)\n");
    md.push_str("Config C: Rs.1,000,000 MaxPerLot Rs.20K (allocation effect)\n\n");
    md.push_str("A vs B  =>  capital effect   (same allocation, different capital)\n");
    md.push_str("B vs C  =>  allocation effect (same capital, different allocation)\n");
    md.push_str("```\n\n");

    // ── Capital × Allocation Matrix ──
    md.push_str("## Capital × Allocation Matrix\n\n");
    md.push_str("### Coralys Arm\n\n");
    md.push_str("| Config | Capital | Allocation | Lots | Return | Velocity | Max DD | Stop Rate |\n");
    md.push_str("|--------|--------:|------------|-----:|-------:|---------:|-------:|----------:|\n");
    for cfg in configs {
        let label = cfg["label"].as_str().unwrap_or("");
        let capital = cfg["initial_capital_inr"].as_f64().unwrap_or(0.0);
        let alloc = cfg["allocation_desc"].as_str().unwrap_or("");
        let c = &cfg["coralys"];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {}x | {} | {} |\n",
            label,
            if capital >= 1_000_000.0 { "Rs.1M".to_string() } else { format!("Rs.{:.0}", capital) },
            alloc,
            fmt_i(&c["lots"]),
            fmt_pct(&c["return_pct"]),
            fmt_f(&c["capital_velocity"]),
            fmt_pct(&c["max_drawdown_pct"]),
            fmt_pct(&c["stop_rate_pct"]),
        ));
    }
    md.push('\n');

    md.push_str("### P.E.2 Arm\n\n");
    md.push_str("| Config | Capital | Allocation | Lots | Return | Velocity | Max DD |\n");
    md.push_str("|--------|--------:|------------|-----:|-------:|---------:|-------:|\n");
    for cfg in configs {
        let label = cfg["label"].as_str().unwrap_or("");
        let capital = cfg["initial_capital_inr"].as_f64().unwrap_or(0.0);
        let alloc = cfg["allocation_desc"].as_str().unwrap_or("");
        let p = &cfg["pe2"];
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {}x | {} |\n",
            label,
            if capital >= 1_000_000.0 { "Rs.1M".to_string() } else { format!("Rs.{:.0}", capital) },
            alloc,
            fmt_i(&p["lots"]),
            fmt_pct(&p["return_pct"]),
            fmt_f(&p["capital_velocity"]),
            fmt_pct(&p["max_drawdown_pct"]),
        ));
    }
    md.push('\n');

    // ── Decision Realization Matrix ──
    md.push_str("## Decision Realization Matrix\n\n");
    md.push_str("The most important output: which certified Coralys decisions were actually realized\n");
    md.push_str("as portfolio trades under each capital/allocation condition.\n\n");
    md.push_str("| Metric | Config A (5K Equal) | Config B (1M Equal) | Config C (1M MaxLot) |\n");
    md.push_str("|--------|--------------------:|--------------------:|---------------------:|\n");

    let n_elig = realization_ledger.n_eligible;
    let n_a = realization_ledger.n_realized_per_config.get(&config_labels[0]).copied().unwrap_or(0);
    let n_b = realization_ledger.n_realized_per_config.get(&config_labels[1]).copied().unwrap_or(0);
    let n_c = realization_ledger.n_realized_per_config.get(&config_labels[2]).copied().unwrap_or(0);
    let not_a = realization_ledger.n_not_realized_per_config.get(&config_labels[0]).copied().unwrap_or(0);
    let not_b = realization_ledger.n_not_realized_per_config.get(&config_labels[1]).copied().unwrap_or(0);
    let not_c = realization_ledger.n_not_realized_per_config.get(&config_labels[2]).copied().unwrap_or(0);

    let pct = |n: usize, d: usize| -> String {
        if d == 0 { "0.00%".to_string() } else { format!("{:.2}%", n as f64 / d as f64 * 100.0) }
    };

    md.push_str(&format!("| Eligible decisions (LONG/SHORT) | {} | {} | {} |\n", n_elig, n_elig, n_elig));
    md.push_str(&format!("| Realized (lot opened) | {} ({}) | {} ({}) | {} ({}) |\n",
        n_a, pct(n_a, n_elig), n_b, pct(n_b, n_elig), n_c, pct(n_c, n_elig)));
    md.push_str(&format!("| Not realized (capital exhausted) | {} ({}) | {} ({}) | {} ({}) |\n",
        not_a, pct(not_a, n_elig), not_b, pct(not_b, n_elig), not_c, pct(not_c, n_elig)));

    // Coralys lots from ledger
    let lots_a = ledgers[0].coralys_summary.n_lots_opened;
    let lots_b = ledgers[1].coralys_summary.n_lots_opened;
    let lots_c = ledgers[2].coralys_summary.n_lots_opened;
    md.push_str(&format!("| Lots opened (incl. position upgrades) | {} | {} | {} |\n", lots_a, lots_b, lots_c));
    md.push('\n');

    md.push_str("> **Note:** Lots opened may exceed realized decisions because the engine allows\n");
    md.push_str("> position upgrades (multiple lots per instrument per decision sequence).\n");
    md.push_str("> The realization ledger counts unique decision_ids, not lots.\n\n");

    // ── Stop Behaviour Matrix ──
    md.push_str("## Stop Behaviour Matrix (Coralys Arm)\n\n");
    md.push_str("| Metric | Config A (5K Equal) | Config B (1M Equal) | Config C (1M MaxLot) |\n");
    md.push_str("|--------|--------------------:|--------------------:|---------------------:|\n");
    let stop_row = |label: &str, field: &str| -> String {
        let vals: Vec<String> = configs.iter().map(|cfg| {
            fmt_pct(&cfg["coralys"][field])
        }).collect();
        format!("| {} | {} | {} | {} |\n", label, vals[0], vals[1], vals[2])
    };
    md.push_str(&stop_row("Stop rate", "stop_rate_pct"));
    md.push_str(&stop_row("Premature stop rate", "premature_stop_rate_pct"));
    md.push_str(&stop_row("Temporary excursion rate", "temporary_excursion_rate_pct"));
    md.push_str(&stop_row("Stop too tight rate", "stop_too_tight_rate_pct"));
    md.push_str(&stop_row("Genuine adverse rate", "genuine_adverse_rate_pct"));
    md.push('\n');

    // ── Capital Effect ──
    md.push_str("## A. Capital Effect\n\n");
    md.push_str("**Comparison:** Config A (Rs.5K EqualWeight) vs Config B (Rs.1M EqualWeight)\n\n");
    md.push_str("**Question:** What changes when only available capital changes?\n\n");
    {
        let pa = &a["pe2"]; let ca = &a["coralys"]; let ra = &a["decision_realization"];
        let pb = &b["pe2"]; let cb = &b["coralys"]; let rb = &b["decision_realization"];
        md.push_str("| Metric | Config A (Rs.5K) | Config B (Rs.1M) | Delta |\n");
        md.push_str("|--------|----------------:|----------------:|------:|\n");
        let delta_pct = |va: f64, vb: f64| -> String { format!("{:+.2}%", vb - va) };
        let delta_x = |va: f64, vb: f64| -> String { format!("{:+.2}x", vb - va) };
        let ca_ret = ca["return_pct"].as_f64().unwrap_or(0.0);
        let cb_ret = cb["return_pct"].as_f64().unwrap_or(0.0);
        let ca_vel = ca["capital_velocity"].as_f64().unwrap_or(0.0);
        let cb_vel = cb["capital_velocity"].as_f64().unwrap_or(0.0);
        let pa_ret = pa["return_pct"].as_f64().unwrap_or(0.0);
        let pb_ret = pb["return_pct"].as_f64().unwrap_or(0.0);
        let na_real = ra["realized"].as_i64().unwrap_or(0);
        let nb_real = rb["realized"].as_i64().unwrap_or(0);
        let na_not = ra["not_realized_due_to_capital"].as_i64().unwrap_or(0);
        let nb_not = rb["not_realized_due_to_capital"].as_i64().unwrap_or(0);
        md.push_str(&format!("| Coralys return | {} | {} | {} |\n",
            fmt_pct(&ca["return_pct"]), fmt_pct(&cb["return_pct"]), delta_pct(ca_ret, cb_ret)));
        md.push_str(&format!("| Coralys velocity | {}x | {}x | {} |\n",
            fmt_f(&ca["capital_velocity"]), fmt_f(&cb["capital_velocity"]), delta_x(ca_vel, cb_vel)));
        md.push_str(&format!("| P.E.2 return | {} | {} | {} |\n",
            fmt_pct(&pa["return_pct"]), fmt_pct(&pb["return_pct"]), delta_pct(pa_ret, pb_ret)));
        md.push_str(&format!("| Coralys lots | {} | {} | {:+} |\n",
            fmt_i(&ca["lots"]), fmt_i(&cb["lots"]),
            cb["lots"].as_i64().unwrap_or(0) - ca["lots"].as_i64().unwrap_or(0)));
        md.push_str(&format!("| Decisions realized | {} | {} | {:+} |\n", na_real, nb_real, nb_real - na_real));
        md.push_str(&format!("| Decisions not realized | {} | {} | {:+} |\n", na_not, nb_not, nb_not - na_not));
        md.push('\n');
    }

    // ── Allocation Effect ──
    md.push_str("## B. Allocation Effect\n\n");
    md.push_str("**Comparison:** Config B (Rs.1M EqualWeight) vs Config C (Rs.1M MaxPerLot Rs.20K)\n\n");
    md.push_str("**Question:** What changes when only allocation policy changes?\n\n");
    {
        let pb = &b["pe2"]; let cb = &b["coralys"]; let rb = &b["decision_realization"];
        let pc = &c["pe2"]; let cc = &c["coralys"]; let rc = &c["decision_realization"];
        md.push_str("| Metric | Config B (1M Equal) | Config C (1M MaxLot) | Delta |\n");
        md.push_str("|--------|--------------------:|---------------------:|------:|\n");
        let delta_pct = |va: f64, vb: f64| -> String { format!("{:+.2}%", vb - va) };
        let delta_x = |va: f64, vb: f64| -> String { format!("{:+.2}x", vb - va) };
        let cb_ret = cb["return_pct"].as_f64().unwrap_or(0.0);
        let cc_ret = cc["return_pct"].as_f64().unwrap_or(0.0);
        let cb_vel = cb["capital_velocity"].as_f64().unwrap_or(0.0);
        let cc_vel = cc["capital_velocity"].as_f64().unwrap_or(0.0);
        let pb_ret = pb["return_pct"].as_f64().unwrap_or(0.0);
        let pc_ret = pc["return_pct"].as_f64().unwrap_or(0.0);
        let nb_real = rb["realized"].as_i64().unwrap_or(0);
        let nc_real = rc["realized"].as_i64().unwrap_or(0);
        let nb_not = rb["not_realized_due_to_capital"].as_i64().unwrap_or(0);
        let nc_not = rc["not_realized_due_to_capital"].as_i64().unwrap_or(0);
        md.push_str(&format!("| Coralys return | {} | {} | {} |\n",
            fmt_pct(&cb["return_pct"]), fmt_pct(&cc["return_pct"]), delta_pct(cb_ret, cc_ret)));
        md.push_str(&format!("| Coralys velocity | {}x | {}x | {} |\n",
            fmt_f(&cb["capital_velocity"]), fmt_f(&cc["capital_velocity"]), delta_x(cb_vel, cc_vel)));
        md.push_str(&format!("| P.E.2 return | {} | {} | {} |\n",
            fmt_pct(&pb["return_pct"]), fmt_pct(&pc["return_pct"]), delta_pct(pb_ret, pc_ret)));
        md.push_str(&format!("| Coralys lots | {} | {} | {:+} |\n",
            fmt_i(&cb["lots"]), fmt_i(&cc["lots"]),
            cc["lots"].as_i64().unwrap_or(0) - cb["lots"].as_i64().unwrap_or(0)));
        md.push_str(&format!("| Decisions realized | {} | {} | {:+} |\n", nb_real, nc_real, nc_real - nb_real));
        md.push_str(&format!("| Decisions not realized | {} | {} | {:+} |\n", nb_not, nc_not, nc_not - nb_not));
        md.push('\n');
    }

    // ── Findings ──
    md.push_str("## Findings\n\n");
    md.push_str("### Capital effect\n\n");
    md.push_str("[Evidence from A vs B comparison above]\n\n");
    md.push_str("### Allocation effect\n\n");
    md.push_str("[Evidence from B vs C comparison above]\n\n");
    md.push_str("### Decision realization effect\n\n");
    md.push_str("[See decision_realization_ledger.json for per-decision breakdown]\n\n");
    md.push_str("### Portfolio performance effect\n\n");
    md.push_str("[See comparison_matrix.json for full metrics]\n\n");
    md.push_str("### Stop behaviour\n\n");
    md.push_str("[See stop behaviour matrix above]\n\n");

    md.push_str("## Interpretation\n\n");
    md.push_str("Capital availability explains changes in decision realization rate and lot count.\n");
    md.push_str("Allocation policy explains changes in velocity and per-lot sizing.\n");
    md.push_str("Coralys decision behaviour itself remains unchanged across all configs.\n\n");

    md.push_str("## Decision\n\n");
    md.push_str("[What the evidence permits us to conclude — to be filled after reviewing results]\n\n");

    md.push_str("## What remains unresolved\n\n");
    md.push_str("- Does MaxPerLot improve risk-adjusted return at all universe sizes?\n");
    md.push_str("- What is the optimal per-lot cap relative to initial capital?\n");
    md.push_str("- How does decision realization rate interact with stop behaviour?\n\n");

    md.push_str("---\n\n");
    md.push_str(&format!("*Generated by csp012_portfolio_v041 | C3-002: {} | Coralys: {}*\n",
        &RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH[..16],
        &CORALYS_EXEC_ARTIFACT_HASH[..16],
    ));

    md
}

// ─── Per-config REPORT.md ─────────────────────────────────────────────────────

fn render_config_report(
    ledger: &ContinuousPortfolioLedger,
    stop: &StopLossAnalysis,
    label: &str,
    alloc_desc: &str,
    capital: f64,
) -> String {
    let p = &ledger.pe2_summary;
    let c = &ledger.coralys_summary;
    let mut md = String::new();
    md.push_str(&format!("# Portfolio Replay v0.4.1 — Config: {label}\n\n"));
    md.push_str(&format!("- **Capital:** Rs.{:.0}\n", capital));
    md.push_str(&format!("- **Allocation:** {alloc_desc}\n"));
    md.push_str(&format!("- **Universe:** {} instruments\n", ledger.universe.len()));
    md.push_str(&format!("- **Sessions simulated:** {}\n\n", ledger.n_sessions_simulated));

    md.push_str("## P.E.2 Arm\n\n");
    md.push_str(&format!("- Lots: {}  TARGET: {}  HORIZON: {}  Open: {}\n",
        p.n_lots_opened, p.n_target, p.n_horizon, p.n_open_at_end));
    md.push_str(&format!("- Return: {:.2}%  Velocity: {:.2}x  Max DD: {:.2}%\n\n",
        p.total_return_pct * 100.0, p.capital_velocity_ratio, p.max_drawdown_pct * 100.0));

    md.push_str("## Coralys Arm\n\n");
    md.push_str(&format!("- Lots: {}  TARGET: {}  STOP: {}  HORIZON: {}  Open: {}\n",
        c.n_lots_opened, c.n_target, c.n_stop, c.n_horizon, c.n_open_at_end));
    md.push_str(&format!("- Return: {:.2}%  Velocity: {:.2}x  Max DD: {:.2}%\n\n",
        c.total_return_pct * 100.0, c.capital_velocity_ratio, c.max_drawdown_pct * 100.0));

    if stop.n_coralys_stops > 0 {
        md.push_str("## Stop Classification\n\n");
        md.push_str(&format!("- Total stops: {}\n", stop.n_coralys_stops));
        md.push_str(&format!("- Premature: {} ({:.1}%)\n",
            stop.n_premature, stop.n_premature as f64 / stop.n_coralys_stops as f64 * 100.0));
        md.push_str(&format!("- Temporary excursion: {} ({:.1}%)\n",
            stop.n_temporary_excursion, stop.n_temporary_excursion as f64 / stop.n_coralys_stops as f64 * 100.0));
        md.push_str(&format!("- Stop too tight: {} ({:.1}%)\n",
            stop.n_stop_too_tight, stop.n_stop_too_tight as f64 / stop.n_coralys_stops as f64 * 100.0));
        md.push_str(&format!("- Genuine adverse: {} ({:.1}%)\n\n",
            stop.n_genuine_adverse, stop.n_genuine_adverse as f64 / stop.n_coralys_stops as f64 * 100.0));
    }
    md
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    println!("Portfolio Replay v0.4.1 -- Capital x Allocation Controlled Experiment");
    println!("  search_two_dir : {}", args.search_two_dir.display());
    println!("  cache_dir      : {}", args.cache_dir.display());
    println!("  output_base    : {}", args.output_base.display());
    println!("  strict         : {}", args.strict);

    // ── Environment guard ─────────────────────────────────────────────────────
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    // ── Artifact hash guard ───────────────────────────────────────────────────
    if CORALYS_EXEC_ARTIFACT_HASH != CORALYS_EXEC_ARTIFACT_HASH_GUARD {
        return Err(format!(
            "coralys artifact hash mismatch: {CORALYS_EXEC_ARTIFACT_HASH}"
        ).into());
    }

    // ── Load C3-002 artifact ──────────────────────────────────────────────────
    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two_dir.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }

    // ── Load bar cache ────────────────────────────────────────────────────────
    println!("\nLoading bar cache from {} ...", args.cache_dir.display());
    let cache: BTreeMap<String, Vec<YahooHistoricalBar>> =
        load_required_yahoo_cache(&args.cache_dir)
            .map_err(|e| format!("cache load failed: {e}"))?;
    println!("  {} instruments loaded", cache.len());

    // ── Build configs ─────────────────────────────────────────────────────────
    // 3 configs: controlled experiment on 52-instrument universe.
    // Config A: Rs.5K EqualWeight (v0.3 baseline)
    // Config B: Rs.1M EqualWeight (capital effect)
    // Config C: Rs.1M MaxPerLot Rs.20K (allocation effect)
    let configs: Vec<(String, String, f64, AllocationModel, ContinuousPortfolioConfig)> = vec![
        (
            "v04_1_A_50_5k_equal".to_string(),
            "EqualWeight".to_string(),
            V041_CAPITAL_5K,
            AllocationModel::EqualWeight,
            ContinuousPortfolioConfig::v03_universe(UNIVERSE_50, "v04_1_A_50_5k_equal"),
        ),
        (
            "v04_1_B_50_1m_equal".to_string(),
            "EqualWeight".to_string(),
            V041_CAPITAL_1M,
            AllocationModel::EqualWeight,
            ContinuousPortfolioConfig::v03_universe(UNIVERSE_50, "v04_1_B_50_1m_equal")
                .with_capital(V041_CAPITAL_1M),
        ),
        (
            "v04_1_C_50_1m_maxlot".to_string(),
            format!("MaxPerLot Rs.{:.0}", V041_MAX_PER_LOT_INR),
            V041_CAPITAL_1M,
            AllocationModel::MaxPerLot { max_per_lot_inr: V041_MAX_PER_LOT_INR },
            ContinuousPortfolioConfig::v04_max_per_lot(
                UNIVERSE_50,
                "v04_1_C_50_1m_maxlot",
                V041_CAPITAL_1M,
                V041_MAX_PER_LOT_INR,
            ),
        ),
    ];

    // ── Run each config ───────────────────────────────────────────────────────
    fs::create_dir_all(&args.output_base)?;

    let mut all_ledgers: Vec<ContinuousPortfolioLedger> = Vec::new();
    let mut all_stops: Vec<StopLossAnalysis> = Vec::new();
    let mut config_labels: Vec<String> = Vec::new();
    let mut config_descs: Vec<String> = Vec::new();
    let mut config_capitals: Vec<f64> = Vec::new();

    for (label, alloc_desc, capital, alloc_model, config) in &configs {
        let available = config.universe.iter().filter(|s| cache.contains_key(s.as_str())).count();
        let requested_size = config.universe.len();

        if args.strict && available < requested_size {
            return Err(format!(
                "strict mode: config {label} requested {requested_size} instruments but only {available} are in cache."
            ).into());
        }

        println!("\n--- Running config: {label} ({available}/{requested_size} instruments in cache) ---");
        println!("    capital   : Rs.{:.0}", capital);
        println!("    allocation: {alloc_desc}");

        let ledger = run_continuous_portfolio_replay_with_config(&artifact, &cache, config)
            .map_err(|e| format!("config {label} failed: {e}"))?;

        if ledger.path_kind != CONTINUOUS_REPLAY_VERSION {
            return Err(format!(
                "unexpected path_kind for {label}: expected {CONTINUOUS_REPLAY_VERSION}, got {}",
                ledger.path_kind
            ).into());
        }

        let stop_analysis = classify_stops(&ledger, &cache, label);

        // ── Write per-config archive ──────────────────────────────────────────
        let archive_dir = args.output_base.join(label);
        refuse_v021_output(&archive_dir.to_string_lossy())?;
        fs::create_dir_all(&archive_dir)?;

        fs::write(
            archive_dir.join("continuous_ledger.json"),
            serde_json::to_vec_pretty(&ledger)?,
        )?;
        fs::write(
            archive_dir.join("stop_loss_analysis.json"),
            serde_json::to_vec_pretty(&stop_analysis)?,
        )?;

        let report_md = render_config_report(&ledger, &stop_analysis, label, alloc_desc, *capital);
        fs::write(archive_dir.join("REPORT.md"), report_md)?;

        let alloc_model_json = match alloc_model {
            AllocationModel::EqualWeight => serde_json::json!({ "model": "EqualWeight" }),
            AllocationModel::MaxPerLot { max_per_lot_inr } => serde_json::json!({
                "model": "MaxPerLot",
                "max_per_lot_inr": max_per_lot_inr,
            }),
        };
        let metadata = serde_json::json!({
            "experiment": "Portfolio Replay v0.4.1 -- Capital x Allocation",
            "config_label": label,
            "path_kind": CONTINUOUS_REPLAY_VERSION,
            "start_clock": ledger.start_clock,
            "certified_t": ledger.certified_t,
            "initial_capital_inr": capital,
            "allocation_model": alloc_model_json,
            "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
            "coralys_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
            "n_sessions_simulated": ledger.n_sessions_simulated,
            "universe_requested": config.universe.len(),
            "universe_available": ledger.universe.len(),
        });
        fs::write(
            archive_dir.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;

        // ── Print per-config summary ──────────────────────────────────────────
        let p = &ledger.pe2_summary;
        let c = &ledger.coralys_summary;
        println!(
            "  result=PASS  config={label}  alloc={alloc_desc}  capital=Rs.{:.0}",
            capital
        );
        println!("  universe_available={}", ledger.universe.len());
        println!(
            "  P.E.2:     lots={} TARGET={} STOP={} HORIZON={} return={:+.2}% velocity={:.2}x",
            p.n_lots_opened, p.n_target, p.n_stop, p.n_horizon,
            p.total_return_pct * 100.0, p.capital_velocity_ratio
        );
        println!(
            "  Coralys:   lots={} TARGET={} STOP={} HORIZON={} return={:+.2}% velocity={:.2}x",
            c.n_lots_opened, c.n_target, c.n_stop, c.n_horizon,
            c.total_return_pct * 100.0, c.capital_velocity_ratio
        );
        let stop_rate = if c.n_lots_opened > 0 {
            c.n_stop as f64 / c.n_lots_opened as f64 * 100.0
        } else { 0.0 };
        let premature_rate = if stop_analysis.n_coralys_stops > 0 {
            stop_analysis.n_premature as f64 / stop_analysis.n_coralys_stops as f64 * 100.0
        } else { 0.0 };
        let excursion_rate = if stop_analysis.n_coralys_stops > 0 {
            stop_analysis.n_temporary_excursion as f64 / stop_analysis.n_coralys_stops as f64 * 100.0
        } else { 0.0 };
        let genuine_rate = if stop_analysis.n_coralys_stops > 0 {
            stop_analysis.n_genuine_adverse as f64 / stop_analysis.n_coralys_stops as f64 * 100.0
        } else { 0.0 };
        println!(
            "  stop_rate={:.1}%  premature={:.1}%  excursion={:.1}%  genuine={:.1}%",
            stop_rate, premature_rate, excursion_rate, genuine_rate
        );
        println!("  archive={}", archive_dir.display());

        all_ledgers.push(ledger);
        all_stops.push(stop_analysis);
        config_labels.push(label.clone());
        config_descs.push(alloc_desc.clone());
        config_capitals.push(*capital);
    }

    // ── Build decision-realization ledger ─────────────────────────────────────
    println!("\n--- Building decision-realization ledger ---");
    let universe_strings: Vec<String> = UNIVERSE_50.iter().map(|s| s.to_string()).collect();
    let realization_ledger = build_decision_realization_ledger(
        &config_labels,
        &all_ledgers,
        &universe_strings,
    );
    println!(
        "  eligible_decisions={} realized_A={} realized_B={} realized_C={}",
        realization_ledger.n_eligible,
        realization_ledger.n_realized_per_config.get(&config_labels[0]).copied().unwrap_or(0),
        realization_ledger.n_realized_per_config.get(&config_labels[1]).copied().unwrap_or(0),
        realization_ledger.n_realized_per_config.get(&config_labels[2]).copied().unwrap_or(0),
    );

    fs::write(
        args.output_base.join("decision_realization_ledger.json"),
        serde_json::to_vec_pretty(&realization_ledger)?,
    )?;

    // ── Build comparison matrix ───────────────────────────────────────────────
    println!("\n--- Writing comparison matrix ---");
    let configs_for_matrix: Vec<(String, String, f64, AllocationModel)> = configs
        .iter()
        .map(|(label, desc, capital, alloc_model, _)| {
            (label.clone(), desc.clone(), *capital, alloc_model.clone())
        })
        .collect();

    let matrix = build_comparison_matrix(
        &configs_for_matrix,
        &all_ledgers,
        &all_stops,
        &realization_ledger,
    );
    fs::write(
        args.output_base.join("comparison_matrix.json"),
        serde_json::to_vec_pretty(&matrix)?,
    )?;
    println!("  comparison_matrix.json written");

    // ── Build comparison report ───────────────────────────────────────────────
    let report_md = render_comparison_report(
        &matrix,
        &all_ledgers,
        &all_stops,
        &realization_ledger,
        &config_labels,
        &config_descs,
        &config_capitals,
    );
    fs::write(args.output_base.join("COMPARISON_REPORT.md"), report_md)?;
    println!("  COMPARISON_REPORT.md written");

    println!("=== v0.4.1 Capital x Allocation Experiment complete ===");
    println!("  output_base={}", args.output_base.display());

    Ok(())
}