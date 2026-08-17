//! Portfolio Replay v0.4 — Allocation Model Experiment.
//!
//! Compares `EqualWeight` (v0.2.1/v0.3 baseline) against `MaxPerSymbol` (v0.4 experiment)
//! at 25 and 50 instruments, with ₹1,000,000 initial capital.
//!
//! ## Motivation
//!
//! v0.3 showed capital velocity collapse at 50 instruments under EqualWeight:
//! `cash / n_eligible` deploys 100% of capital in session 1 when signal density is high
//! (50 signals × 2% each = 100%). No capital remains for subsequent sessions.
//!
//! `MaxPerLot { max_per_lot_inr: 20_000 }` caps each lot at ₹20k (2% of ₹1M),
//! leaving undeployed capital available for future sessions.
//!
//! ## Experiment design
//!
//! ```text
//! Capital: ₹1,000,000 (same for all 4 configs)
//!
//!                  EqualWeight (control)    MaxPerLot ₹20k (experiment)
//!                  ─────────────────────    ───────────────────────────
//! 25 instruments   v04_A_25_equal           v04_B_25_max
//! 50 instruments   v04_C_50_equal           v04_D_50_max
//! ```
//!
//! ## v0.4 contract (FROZEN — do not modify)
//!
//! ```text
//! Same engine          — run_continuous_portfolio_replay_with_config
//! Same historical period — PORTFOLIO_REPLAY_REQUESTED_CLOCK
//! Same C3-002          — RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
//! Same Coralys v0      — CORALYS_EXEC_ARTIFACT_HASH
//! Same stop-loss       — enforced risk_boundary from CoralysExecutionIntent
//! Same lifecycle       — session-by-session, capital recycled on close
//!
//!              CHANGES vs v0.3:
//!                1. Initial capital: ₹5,000 → ₹1,000,000
//!                2. Allocation model: EqualWeight (control) vs MaxPerLot ₹20k (experiment)
//!                3. Universe: 25 and 50 instruments only (100 excluded — C3-002 scope)
//! ```
//!
//! ## Output per config
//!
//! ```text
//! {output_base}/{label}/
//!   continuous_ledger.json    — full ledger
//!   stop_loss_analysis.json   — auto-produced stop classification
//!   REPORT.md                 — human-readable report
//!   metadata.json             — experiment metadata
//! ```
//!
//! ## Cross-config output
//!
//! ```text
//! {output_base}/
//!   comparison_matrix.json    — all 4 configs × all metrics
//!   COMPARISON_REPORT.md      — human-readable side-by-side matrix
//! ```
//!
//! ## Usage
//!
//! ```sh
//! cargo run --bin csp011_portfolio_v04 -- \
//!   --search-two product_validation/CS-P-006/discovery/20260815T051900Z_c3 \
//!   --cache-dir  product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache \
//!   --output-base historical_runs/portfolio_v04_allocation_experiment
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::coralys_execution_model::CORALYS_EXEC_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_execution::ExitReason;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::portfolio_replay_v021::{
    refuse_v021_output, run_continuous_portfolio_replay_with_config, AllocationModel,
    ContinuousPortfolioConfig, ContinuousPortfolioLedger, TradeLot, CONTINUOUS_REPLAY_VERSION,
};
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;

// ─── v0.4 experiment constants ────────────────────────────────────────────────

/// Initial capital for all v0.4 configs (₹1,000,000).
pub const V04_INITIAL_CAPITAL_INR: f64 = 1_000_000.0;

/// MaxPerLot cap: ₹20,000 **per lot** = 2% of ₹1M.
///
/// Semantics: each individual lot opened in a session is capped at ₹20k, regardless of how
/// many lots are already open for the same instrument (position upgrades are allowed).
/// This is per-lot, not aggregate per symbol — consistent with the engine's multi-lot model.
///
/// At 50 instruments this deploys at most ₹1M total (50 × ₹20k), but only if all 50 signal
/// in the same session. In practice signal density is lower, so capital is preserved across
/// sessions — which is the fix for the v0.3 velocity collapse.
pub const V04_MAX_PER_LOT_INR: f64 = 20_000.0;

// ─── Stop classification ──────────────────────────────────────────────────────

/// Stop classification categories (mirrors the Python taxonomy from v0.2.1 analysis).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum StopCategory {
    /// Stop triggered on the same bar as entry (gap-through at open).
    GapThrough,
    /// Price recovered to target within 3 sessions after stop.
    PrematureStop,
    /// Price recovered above stop level within 5 sessions but did not reach target.
    TemporaryExcursion,
    /// Stop was within 1% of entry (stop too tight relative to noise).
    StopTooTight,
    /// Price continued adverse for ≥5 sessions after stop — stop was directionally correct.
    DirectionFailure,
    /// None of the above — stop was genuinely protective.
    GenuineAdverse,
}

impl StopCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            StopCategory::GapThrough => "GAP_THROUGH",
            StopCategory::PrematureStop => "PREMATURE_STOP",
            StopCategory::TemporaryExcursion => "TEMPORARY_EXCURSION",
            StopCategory::StopTooTight => "STOP_TOO_TIGHT",
            StopCategory::DirectionFailure => "DIRECTION_FAILURE",
            StopCategory::GenuineAdverse => "GENUINE_ADVERSE",
        }
    }
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
    /// Adverse gap magnitude: positive = gapped through stop adversely.
    /// Direction-normalized: LONG = (stop - exit)/stop, SHORT = (exit - stop)/stop.
    pub gap_magnitude_pct: f64,
    /// Max favorable excursion within 5 sessions after stop (direction-normalized).
    /// LONG: (max_close - exit) / exit. SHORT: (exit - min_close) / exit.
    pub post_stop_max_favorable_pct: Option<f64>,
    /// Whether price reached the original target within 3 sessions after stop (direction-aware).
    pub target_reached_after_stop: bool,
    /// Whether price recovered toward entry within 5 sessions after stop (direction-aware).
    pub recovered_after_stop_within_5: bool,
    /// Whether price continued adversely for all 5 sessions after stop (direction-aware).
    pub continued_adverse_5_sessions: bool,
    /// Stop distance from entry as fraction of entry price.
    pub stop_tightness_pct: f64,
    /// Counterfactual PnL if held to experiment end_clock (bounded by ledger end, not cache end).
    pub counterfactual_pnl_inr: Option<f64>,
    pub opportunity_cost_inr: Option<f64>,
    /// Primary category (derived from orthogonal flags with defined precedence).
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
    // Temporal boundary: derive experiment end from the last closed lot exit time across
    // both arms. This bounds post_stop_bars and counterfactual to the experiment window,
    // preventing look-ahead leakage from bars that extend beyond the last simulated session.
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
        let stop_price = match lot.stop_price {
            Some(p) => p,
            None => continue,
        };
        let exit_price = match lot.exit_price {
            Some(p) => p,
            None => continue,
        };
        let exit_time_str = match &lot.exit_time {
            Some(t) => t.clone(),
            None => continue,
        };
        let holding = lot.holding_sessions.unwrap_or(0);
        let realized_pnl = lot.realized_pnl_inr.unwrap_or(0.0);
        let is_long = lot.direction == "LONG";

        // Direction-normalized gap magnitude.
        // Positive = gapped adversely through stop.
        // LONG: stop was below entry; adverse gap = exit opened below stop → stop - exit > 0.
        // SHORT: stop was above entry; adverse gap = exit opened above stop → exit - stop > 0.
        let gap_magnitude_pct = if stop_price > 0.0 {
            if is_long {
                (stop_price - exit_price) / stop_price
            } else {
                (exit_price - stop_price) / stop_price
            }
        } else {
            0.0
        };

        let stop_tightness_pct = if lot.entry_price > 0.0 {
            (stop_price - lot.entry_price).abs() / lot.entry_price
        } else {
            0.0
        };

        let bars = cache.get(&lot.instrument);
        let (
            post_stop_max_favorable_pct,
            target_reached_after_stop,
            recovered_after_stop_within_5,
            continued_adverse_5_sessions,
            counterfactual_pnl_inr,
        ) = if let Some(bars) = bars {
            let exit_ts = chrono::DateTime::parse_from_rfc3339(&exit_time_str)
                .map(|t| t.with_timezone(&chrono::Utc))
                .ok();

            // Bound post-stop bars to [exit_time, end_clock] — no look-ahead beyond experiment.
            let post_stop_bars: Vec<&YahooHistoricalBar> = if let Some(exit_ts) = exit_ts {
                bars.iter()
                    .filter(|b| {
                        let bar_ts = chrono::DateTime::from_timestamp(b.timestamp, 0);
                        let after_exit = bar_ts.map(|t| t > exit_ts).unwrap_or(false);
                        let before_end = end_ts
                            .map(|end| bar_ts.map(|t| t <= end).unwrap_or(false))
                            .unwrap_or(true); // if end_ts unavailable, don't filter
                        after_exit && before_end
                    })
                    .collect()
            } else {
                vec![]
            };

            let within_5 = &post_stop_bars[..post_stop_bars.len().min(5)];

            // Direction-normalized MFE: favorable = toward target.
            // LONG: favorable = high prices. SHORT: favorable = low prices.
            let max_favorable_pct = if !within_5.is_empty() && exit_price > 0.0 {
                if is_long {
                    let max_close = within_5.iter().map(|b| b.close).fold(f64::NEG_INFINITY, f64::max);
                    Some((max_close - exit_price) / exit_price)
                } else {
                    let min_close = within_5.iter().map(|b| b.close).fold(f64::INFINITY, f64::min);
                    Some((exit_price - min_close) / exit_price)
                }
            } else {
                None
            };

            // Direction-normalized target: LONG target is above entry, SHORT target is below.
            let target_reached = if is_long {
                within_5.iter().take(3).any(|b| b.close >= lot.target_price)
            } else {
                within_5.iter().take(3).any(|b| b.close <= lot.target_price)
            };

            // Direction-normalized recovery: price moved back toward entry after stop.
            // LONG: price rose above entry. SHORT: price fell below entry.
            let recovered = if is_long {
                within_5.iter().any(|b| b.close > lot.entry_price)
            } else {
                within_5.iter().any(|b| b.close < lot.entry_price)
            };

            // Direction-normalized continued adverse: price kept moving against entry for all 5 bars.
            // LONG: all closes below entry. SHORT: all closes above entry.
            let continued_adverse = within_5.len() >= 5 && if is_long {
                within_5.iter().all(|b| b.close < lot.entry_price)
            } else {
                within_5.iter().all(|b| b.close > lot.entry_price)
            };

            // Counterfactual: use last bar within experiment end_clock (not cache end).
            let last_bar_in_window = post_stop_bars.last().map(|b| b.close);
            let counterfactual = last_bar_in_window.map(|close| {
                let ret = if is_long {
                    (close - lot.entry_price) / lot.entry_price
                } else {
                    (lot.entry_price - close) / lot.entry_price
                };
                lot.allocation_inr * ret
            });

            (max_favorable_pct, target_reached, recovered, continued_adverse, counterfactual)
        } else {
            (None, false, false, false, None)
        };

        let opportunity_cost_inr = counterfactual_pnl_inr.map(|cf| cf - realized_pnl);

        // Category precedence (orthogonal flags → single primary category):
        // 1. StopTooTight — structural issue, checked first regardless of outcome
        // 2. GapThrough   — execution issue (slippage beyond stop)
        // 3. PrematureStop — target reached after stop (stop was directionally wrong)
        // 4. TemporaryExcursion — recovered toward entry but didn't reach target
        // 5. DirectionFailure — continued adverse for all 5 sessions
        // 6. GenuineAdverse — none of the above
        let category = if stop_tightness_pct < 0.01 {
            StopCategory::StopTooTight
        } else if gap_magnitude_pct > 0.005 {
            StopCategory::GapThrough
        } else if target_reached_after_stop {
            StopCategory::PrematureStop
        } else if recovered_after_stop_within_5 {
            StopCategory::TemporaryExcursion
        } else if continued_adverse_5_sessions {
            StopCategory::DirectionFailure
        } else {
            StopCategory::GenuineAdverse
        };

        diagnostics.push(StopDiagnostic {
            trade_id: lot.trade_id.clone(),
            instrument: lot.instrument.clone(),
            direction: lot.direction.clone(),
            entry_price: lot.entry_price,
            stop_price,
            exit_price,
            entry_time: lot.entry_time.clone(),
            exit_time: exit_time_str,
            holding_sessions: holding,
            realized_pnl_inr: realized_pnl,
            allocation_inr: lot.allocation_inr,
            gap_magnitude_pct,
            post_stop_max_favorable_pct,
            target_reached_after_stop,
            recovered_after_stop_within_5: recovered_after_stop_within_5,
            continued_adverse_5_sessions,
            stop_tightness_pct,
            counterfactual_pnl_inr,
            opportunity_cost_inr,
            category,
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
        n_gap_through,
        n_premature,
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

// ─── Report rendering ─────────────────────────────────────────────────────────

fn render_v04_report(
    ledger: &ContinuousPortfolioLedger,
    stop_analysis: &StopLossAnalysis,
    config_label: &str,
    allocation_desc: &str,
) -> String {
    let p = &ledger.pe2_summary;
    let c = &ledger.coralys_summary;
    let mut md = String::new();

    md.push_str(&format!("# Portfolio Replay v0.4 — Allocation Experiment: {config_label}\n\n"));
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Experiment:** Allocation model comparison — EqualWeight vs MaxPerSymbol  \n");
    md.push_str(&format!("**Allocation:** {allocation_desc}  \n"));
    md.push_str(&format!("**Initial capital:** Rs.{:.0}  \n\n", V04_INITIAL_CAPITAL_INR));

    md.push_str("## Setup\n\n");
    md.push_str(&format!("- Config label: `{config_label}`\n"));
    md.push_str(&format!("- Universe size: {} instruments\n", ledger.universe.len()));
    md.push_str(&format!("- Universe: {}\n", ledger.universe.join(", ")));
    md.push_str(&format!("- Certified T: {}\n", ledger.certified_t));
    md.push_str(&format!("- Sessions simulated: {}\n", ledger.n_sessions_simulated));
    md.push_str(&format!("- Initial capital: Rs.{:.0}\n", ledger.initial_capital_inr));
    md.push_str(&format!("- Allocation model: {allocation_desc}\n"));
    md.push_str(&format!("- C3-002 artifact: `{}`\n", ledger.c3_002_artifact_hash));
    md.push_str(&format!("- Coralys artifact: `{}`\n\n", ledger.coralys_artifact_hash));

    md.push_str("## Capital Velocity\n\n");
    md.push_str("| Metric | P.E.2 | Coralys v0 |\n");
    md.push_str("|--------|-------|------------|\n");
    md.push_str(&format!("| Capital velocity | {:.2}x | {:.2}x |\n", p.capital_velocity_ratio, c.capital_velocity_ratio));
    md.push_str(&format!("| Lots opened | {} | {} |\n", p.n_lots_opened, c.n_lots_opened));
    md.push_str(&format!("| TARGET exits | {} | {} |\n", p.n_target, c.n_target));
    md.push_str(&format!("| STOP exits | {} | {} |\n", p.n_stop, c.n_stop));
    md.push_str(&format!("| HORIZON exits | {} | {} |\n", p.n_horizon, c.n_horizon));
    md.push_str(&format!("| Open at end | {} | {} |\n", p.n_open_at_end, c.n_open_at_end));
    if let (Some(pa), Some(ca)) = (p.avg_holding_sessions, c.avg_holding_sessions) {
        md.push_str(&format!("| Avg hold (sessions) | {:.1} | {:.1} |\n", pa, ca));
    }
    md.push_str("\n");

    md.push_str("## Returns\n\n");
    md.push_str("| Metric | P.E.2 | Coralys v0 |\n");
    md.push_str("|--------|-------|------------|\n");
    md.push_str(&format!("| Total return | {:+.2}% | {:+.2}% |\n",
        p.total_return_pct * 100.0, c.total_return_pct * 100.0));
    md.push_str(&format!("| Realized PnL | Rs.{:+.2} | Rs.{:+.2} |\n",
        p.total_realized_pnl_inr, c.total_realized_pnl_inr));
    md.push_str(&format!("| Unrealized PnL | Rs.{:+.2} | Rs.{:+.2} |\n",
        p.total_unrealized_pnl_inr, c.total_unrealized_pnl_inr));
    md.push_str(&format!("| Max drawdown | Rs.{:.2} ({:.2}%) | Rs.{:.2} ({:.2}%) |\n",
        p.max_drawdown_inr, p.max_drawdown_pct * 100.0,
        c.max_drawdown_inr, c.max_drawdown_pct * 100.0));
    md.push_str("\n");

    md.push_str("## Stop-Loss Analysis (Coralys arm)\n\n");
    md.push_str(&format!("- Total stops: {}\n", stop_analysis.n_coralys_stops));
    if stop_analysis.n_coralys_stops > 0 {
        md.push_str(&format!("- Premature: {:.1}% ({}/{})\n",
            stop_analysis.pct_premature, stop_analysis.n_premature, stop_analysis.n_coralys_stops));
        md.push_str(&format!("- Temporary excursion: {:.1}% ({}/{})\n",
            stop_analysis.pct_temporary_excursion, stop_analysis.n_temporary_excursion, stop_analysis.n_coralys_stops));
        md.push_str(&format!("- Stop too tight: {:.1}% ({}/{})\n",
            stop_analysis.pct_stop_too_tight, stop_analysis.n_stop_too_tight, stop_analysis.n_coralys_stops));
        md.push_str(&format!("- Direction failure: {:.1}% ({}/{})\n",
            stop_analysis.pct_direction_failure, stop_analysis.n_direction_failure, stop_analysis.n_coralys_stops));
        md.push_str(&format!("- Genuine adverse: {:.1}% ({}/{})\n",
            stop_analysis.pct_genuine_adverse, stop_analysis.n_genuine_adverse, stop_analysis.n_coralys_stops));
        md.push_str(&format!("- Net stop benefit: Rs.{:+.2}\n", stop_analysis.net_stop_benefit_inr));
    }
    md.push_str("\n");

    md
}

fn render_comparison_matrix(
    results: &[(String, String, &ContinuousPortfolioLedger, &StopLossAnalysis)],
) -> String {
    let mut md = String::new();

    md.push_str("# Portfolio Replay v0.4 — Allocation Model Comparison Matrix\n\n");
    md.push_str("**Experiment:** EqualWeight (control) vs MaxPerSymbol ₹20k (experiment)  \n");
    md.push_str(&format!("**Initial capital:** Rs.{:.0} for all configs  \n\n", V04_INITIAL_CAPITAL_INR));

    // P.E.2 arm table
    md.push_str("## P.E.2 Arm\n\n");
    md.push_str("| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity |\n");
    md.push_str("|--------|-------|----------|------|--------|------|---------|--------|----------|\n");
    for (label, alloc_desc, ledger, _) in results {
        let p = &ledger.pe2_summary;
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {:+.2}% | {:.2}x |\n",
            label, alloc_desc, ledger.universe.len(),
            p.n_lots_opened, p.n_target, p.n_stop, p.n_horizon,
            p.total_return_pct * 100.0, p.capital_velocity_ratio,
        ));
    }
    md.push_str("\n");

    // Coralys arm table
    md.push_str("## Coralys v0 Arm\n\n");
    md.push_str("| Config | Alloc | Universe | Lots | TARGET | STOP | HORIZON | Return | Velocity | Stop% | Premature% | Excursion% | Genuine% |\n");
    md.push_str("|--------|-------|----------|------|--------|------|---------|--------|----------|-------|------------|------------|----------|\n");
    for (label, alloc_desc, ledger, stop_analysis) in results {
        let c = &ledger.coralys_summary;
        let stop_rate = if c.n_lots_opened > 0 {
            c.n_stop as f64 / c.n_lots_opened as f64 * 100.0
        } else {
            0.0
        };
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {:+.2}% | {:.2}x | {:.1}% | {:.1}% | {:.1}% | {:.1}% |\n",
            label, alloc_desc, ledger.universe.len(),
            c.n_lots_opened, c.n_target, c.n_stop, c.n_horizon,
            c.total_return_pct * 100.0, c.capital_velocity_ratio,
            stop_rate,
            stop_analysis.pct_premature,
            stop_analysis.pct_temporary_excursion,
            stop_analysis.pct_genuine_adverse,
        ));
    }
    md.push_str("\n");

    // Velocity comparison: EqualWeight vs MaxPerSymbol
    md.push_str("## Velocity Comparison: EqualWeight vs MaxPerSymbol\n\n");
    md.push_str("| Universe | EqualWeight Velocity (PE2/Coralys) | MaxPerSymbol Velocity (PE2/Coralys) | Delta |\n");
    md.push_str("|----------|-------------------------------------|--------------------------------------|-------|\n");

    // Pair up: A/B = 25 instruments, C/D = 50 instruments
    let pairs: &[(&str, &str)] = &[
        ("v04_A_25_equal", "v04_B_25_max"),
        ("v04_C_50_equal", "v04_D_50_max"),
    ];
    for (eq_label, max_label) in pairs {
        let eq = results.iter().find(|(l, _, _, _)| l == eq_label);
        let mx = results.iter().find(|(l, _, _, _)| l == max_label);
        if let (Some((_, _, eq_ledger, _)), Some((_, _, mx_ledger, _))) = (eq, mx) {
            let universe_size = eq_ledger.universe.len();
            let eq_pe2_v = eq_ledger.pe2_summary.capital_velocity_ratio;
            let eq_cor_v = eq_ledger.coralys_summary.capital_velocity_ratio;
            let mx_pe2_v = mx_ledger.pe2_summary.capital_velocity_ratio;
            let mx_cor_v = mx_ledger.coralys_summary.capital_velocity_ratio;
            md.push_str(&format!(
                "| {} | {:.2}x / {:.2}x | {:.2}x / {:.2}x | PE2: {:+.2}x / Coralys: {:+.2}x |\n",
                universe_size,
                eq_pe2_v, eq_cor_v,
                mx_pe2_v, mx_cor_v,
                mx_pe2_v - eq_pe2_v,
                mx_cor_v - eq_cor_v,
            ));
        }
    }
    md.push_str("\n");

    md.push_str("## Interpretation\n\n");
    md.push_str("- **EqualWeight** deploys all available cash in every session with eligible signals.\n");
    md.push_str("  At high signal density (50 instruments), this exhausts capital in session 1.\n");
    md.push_str("- **MaxPerSymbol ₹20k** caps each lot at ₹20,000, leaving undeployed capital\n");
    md.push_str("  available for subsequent sessions. This should increase velocity at 50 instruments.\n");
    md.push_str("- If MaxPerSymbol velocity > EqualWeight velocity at 50 instruments, the hypothesis\n");
    md.push_str("  is confirmed: allocation policy (not capital amount) drives velocity collapse.\n\n");

    md
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    search_two_dir: PathBuf,
    cache_dir: PathBuf,
    output_base: PathBuf,
    strict: bool,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut search_two_dir: Option<PathBuf> = None;
    let mut cache_dir: Option<PathBuf> = None;
    let mut output_base: Option<PathBuf> = None;
    let mut strict = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--search-two" => {
                i += 1;
                search_two_dir = Some(PathBuf::from(&args[i]));
            }
            "--cache-dir" => {
                i += 1;
                cache_dir = Some(PathBuf::from(&args[i]));
            }
            "--output-base" => {
                i += 1;
                output_base = Some(PathBuf::from(&args[i]));
            }
            "--strict" => {
                strict = true;
            }
            other => {
                return Err(format!("unknown argument: {other}").into());
            }
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

// ─── Universe slices ──────────────────────────────────────────────────────────

/// 7-instrument RESEARCH_UNIVERSE (canonical baseline).
const UNIVERSE_7: &[&str] = &[
    "HDFCBANK.NS", "ICICIBANK.NS", "INFY.NS", "RELIANCE.NS",
    "TCS.NS", "IDEA.NS", "MAHABANK.NS",
];

/// First 27 instruments — v0.3-A base (25) + MAHABANK.NS + IDEA.NS.
const UNIVERSE_25: &[&str] = &[
    "RELIANCE.NS", "TCS.NS", "HDFCBANK.NS", "INFY.NS", "ICICIBANK.NS",
    "HINDUNILVR.NS", "ITC.NS", "SBIN.NS", "BHARTIARTL.NS", "KOTAKBANK.NS",
    "LT.NS", "AXISBANK.NS", "ASIANPAINT.NS", "MARUTI.NS", "TITAN.NS",
    "SUNPHARMA.NS", "WIPRO.NS", "ULTRACEMCO.NS", "BAJFINANCE.NS", "NESTLEIND.NS",
    "POWERGRID.NS", "NTPC.NS", "TECHM.NS", "HCLTECH.NS", "ONGC.NS",
    "MAHABANK.NS", "IDEA.NS",
];

/// 52 instruments — extends UNIVERSE_25 with 25 more + MAHABANK.NS + IDEA.NS.
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

/// 102 instruments — v0.3-C base (100) + MAHABANK.NS + IDEA.NS.
const UNIVERSE_100: &[&str] = &[
    "HDFCBANK.NS", "RELIANCE.NS", "TCS.NS", "INFY.NS",
    "ICICIBANK.NS", "HINDUNILVR.NS", "ITC.NS",
    "KOTAKBANK.NS", "AXISBANK.NS", "SBIN.NS", "BAJFINANCE.NS",
    "BHARTIARTL.NS", "ASIANPAINT.NS", "MARUTI.NS", "TITAN.NS",
    "SUNPHARMA.NS", "WIPRO.NS", "HCLTECH.NS", "ULTRACEMCO.NS",
    "NESTLEIND.NS", "POWERGRID.NS", "NTPC.NS", "ONGC.NS",
    "TMPV.NS", "TATASTEEL.NS",
    "ADANIENT.NS", "ADANIPORTS.NS", "BAJAJFINSV.NS", "BPCL.NS",
    "BRITANNIA.NS", "CIPLA.NS", "COALINDIA.NS", "DIVISLAB.NS",
    "DRREDDY.NS", "EICHERMOT.NS", "GRASIM.NS", "HEROMOTOCO.NS",
    "HINDALCO.NS", "INDUSINDBK.NS", "JSWSTEEL.NS", "LT.NS",
    "M&M.NS", "PIDILITIND.NS", "SBILIFE.NS", "SHREECEM.NS",
    "SIEMENS.NS", "TECHM.NS", "TRENT.NS", "UPL.NS",
    "VEDL.NS",
    "ABCAPITAL.NS", "ABFRL.NS", "ACC.NS", "AMBUJACEM.NS",
    "APOLLOHOSP.NS", "APOLLOTYRE.NS", "AUROPHARMA.NS", "BALKRISIND.NS",
    "BANDHANBNK.NS", "BANKBARODA.NS", "BERGEPAINT.NS", "BIOCON.NS",
    "BOSCHLTD.NS", "CANBK.NS", "CHOLAFIN.NS", "COLPAL.NS",
    "CONCOR.NS", "CUMMINSIND.NS", "DABUR.NS", "DLF.NS",
    "ESCORTS.NS", "EXIDEIND.NS", "FEDERALBNK.NS", "GAIL.NS",
    "GODREJCP.NS", "GODREJPROP.NS", "HAVELLS.NS", "HDFCAMC.NS",
    "HDFCLIFE.NS", "ICICIPRULI.NS", "IDFCFIRSTB.NS", "IGL.NS",
    "INDUSTOWER.NS", "IRCTC.NS", "JUBLFOOD.NS", "LICHSGFIN.NS",
    "LUPIN.NS", "MARICO.NS", "UNITDSPR.NS", "MFSL.NS",
    "MPHASIS.NS", "MRF.NS", "MUTHOOTFIN.NS", "NAUKRI.NS",
    "NMDC.NS", "PAGEIND.NS", "PIIND.NS", "PERSISTENT.NS",
    "PFC.NS", "PNB.NS",
    "TATACONSUM.NS",
    "MAHABANK.NS", "IDEA.NS",
];

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    println!("Portfolio Replay v0.4 — Allocation Model Experiment");
    println!("  search_two_dir : {}", args.search_two_dir.display());
    println!("  cache_dir      : {}", args.cache_dir.display());
    println!("  output_base    : {}", args.output_base.display());
    println!("  initial_capital: Rs.{:.0}", V04_INITIAL_CAPITAL_INR);
    println!("  max_per_lot    : Rs.{:.0}", V04_MAX_PER_LOT_INR);
    println!("  strict         : {}", args.strict);

    // ── Load artifact ─────────────────────────────────────────────────────────
    let artifact_path = args.search_two_dir.join("selected_policy.json");
    let artifact_bytes = fs::read(&artifact_path)
        .map_err(|e| format!("cannot read artifact at {}: {e}", artifact_path.display()))?;
    let artifact: PolicyArtifact = serde_json::from_slice(&artifact_bytes)
        .map_err(|e| format!("cannot parse artifact: {e}"))?;

    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err(format!(
            "artifact hash mismatch: expected {RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH}, got {}",
            artifact.artifact_hash
        ).into());
    }
    println!("  artifact_hash  : {} ✓", artifact.artifact_hash);

    // ── Load bar cache ────────────────────────────────────────────────────────
    println!("\nLoading bar cache from {} ...", args.cache_dir.display());
    let cache: BTreeMap<String, Vec<YahooHistoricalBar>> =
        load_required_yahoo_cache(&args.cache_dir)
            .map_err(|e| format!("cache load failed: {e}"))?;
    println!("  {} instruments loaded", cache.len());

    // ── Build configs ─────────────────────────────────────────────────────────
    // 8 configs: EqualWeight and MaxPerLot at 7, 27, 52, and 102 instruments.
    // All use Rs.1M initial capital.
    let configs: Vec<(String, String, ContinuousPortfolioConfig)> = vec![
        (
            "v04_G_7_equal".to_string(),
            "EqualWeight".to_string(),
            ContinuousPortfolioConfig::v03_universe(UNIVERSE_7, "v04_G_7_equal")
                .with_capital(V04_INITIAL_CAPITAL_INR),
        ),
        (
            "v04_H_7_max".to_string(),
            format!("MaxPerLot Rs.{:.0}", V04_MAX_PER_LOT_INR),
            ContinuousPortfolioConfig::v04_max_per_lot(
                UNIVERSE_7,
                "v04_H_7_max",
                V04_INITIAL_CAPITAL_INR,
                V04_MAX_PER_LOT_INR,
            ),
        ),
        (
            "v04_A_25_equal".to_string(),
            "EqualWeight".to_string(),
            ContinuousPortfolioConfig::v03_universe(UNIVERSE_25, "v04_A_25_equal")
                .with_capital(V04_INITIAL_CAPITAL_INR),
        ),
        (
            "v04_B_25_max".to_string(),
            format!("MaxPerLot Rs.{:.0}", V04_MAX_PER_LOT_INR),
            ContinuousPortfolioConfig::v04_max_per_lot(
                UNIVERSE_25,
                "v04_B_25_max",
                V04_INITIAL_CAPITAL_INR,
                V04_MAX_PER_LOT_INR,
            ),
        ),
        (
            "v04_C_50_equal".to_string(),
            "EqualWeight".to_string(),
            ContinuousPortfolioConfig::v03_universe(UNIVERSE_50, "v04_C_50_equal")
                .with_capital(V04_INITIAL_CAPITAL_INR),
        ),
        (
            "v04_D_50_max".to_string(),
            format!("MaxPerLot Rs.{:.0}", V04_MAX_PER_LOT_INR),
            ContinuousPortfolioConfig::v04_max_per_lot(
                UNIVERSE_50,
                "v04_D_50_max",
                V04_INITIAL_CAPITAL_INR,
                V04_MAX_PER_LOT_INR,
            ),
        ),
        (
            "v04_E_100_equal".to_string(),
            "EqualWeight".to_string(),
            ContinuousPortfolioConfig::v03_universe(UNIVERSE_100, "v04_E_100_equal")
                .with_capital(V04_INITIAL_CAPITAL_INR),
        ),
        (
            "v04_F_100_max".to_string(),
            format!("MaxPerLot Rs.{:.0}", V04_MAX_PER_LOT_INR),
            ContinuousPortfolioConfig::v04_max_per_lot(
                UNIVERSE_100,
                "v04_F_100_max",
                V04_INITIAL_CAPITAL_INR,
                V04_MAX_PER_LOT_INR,
            ),
        ),
    ];

    // ── Run each config ───────────────────────────────────────────────────────
    fs::create_dir_all(&args.output_base)?;

    let mut all_results: Vec<(String, String, ContinuousPortfolioLedger, StopLossAnalysis)> = Vec::new();

    for (label, alloc_desc, config) in &configs {
        let available = config.universe.iter().filter(|s| cache.contains_key(s.as_str())).count();
        let requested_size = config.universe.len();

        if args.strict && available < requested_size {
            return Err(format!(
                "strict mode: config {label} requested {requested_size} instruments but only {available} are in cache."
            ).into());
        }

        println!("\n─── Running config: {label} ({available}/{requested_size} instruments in cache) ───");
        println!("    allocation: {alloc_desc}");

        let ledger = run_continuous_portfolio_replay_with_config(&artifact, &cache, config)
            .map_err(|e| format!("config {label} failed: {e}"))?;

        if ledger.path_kind != CONTINUOUS_REPLAY_VERSION {
            return Err(format!(
                "unexpected path_kind for {label}: expected {CONTINUOUS_REPLAY_VERSION}, got {}",
                ledger.path_kind
            ).into());
        }

        // ── Stop classification ───────────────────────────────────────────────
        let stop_analysis = classify_stops(&ledger, &cache, label);

        // ── Write per-config archive ──────────────────────────────────────────
        let archive_dir = args.output_base.join(label);
        refuse_v021_output(&archive_dir.to_string_lossy())?;
        fs::create_dir_all(&archive_dir)?;

        // continuous_ledger.json
        let ledger_path = archive_dir.join("continuous_ledger.json");
        refuse_v021_output(&ledger_path.to_string_lossy())?;
        fs::write(&ledger_path, serde_json::to_vec_pretty(&ledger)?)?;

        // stop_loss_analysis.json
        fs::write(
            archive_dir.join("stop_loss_analysis.json"),
            serde_json::to_vec_pretty(&stop_analysis)?,
        )?;

        // REPORT.md
        let report_md = render_v04_report(&ledger, &stop_analysis, label, alloc_desc);
        fs::write(archive_dir.join("REPORT.md"), report_md)?;

        // metadata.json
        let alloc_model_json = match &config.allocation_model {
            AllocationModel::EqualWeight => serde_json::json!({ "model": "EqualWeight" }),
            AllocationModel::MaxPerLot { max_per_lot_inr } => serde_json::json!({
                "model": "MaxPerLot",
                "max_per_lot_inr": max_per_lot_inr,
                "semantics": "per-lot cap (not aggregate per symbol); position upgrades allowed",
            }),
        };
        let metadata = serde_json::json!({
            "experiment": "Portfolio Replay v0.4 — Allocation Model Experiment",
            "config_label": label,
            "path_kind": CONTINUOUS_REPLAY_VERSION,
            "start_clock": ledger.start_clock,
            "certified_t": ledger.certified_t,
            "initial_capital_inr": V04_INITIAL_CAPITAL_INR,
            "allocation_model": alloc_model_json,
            "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
            "coralys_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
            "n_sessions_simulated": ledger.n_sessions_simulated,
            "universe_requested": config.universe.len(),
            "universe_available": ledger.universe.len(),
            "universe": ledger.universe,
            "v04_contract": {
                "engine": "run_continuous_portfolio_replay_with_config",
                "changes_vs_v03": ["initial_capital_inr", "allocation_model"],
                "frozen": ["C3-002", "Coralys-v0", "stop-loss", "lifecycle", "historical-period"]
            },
            "stop_summary": {
                "n_stops": stop_analysis.n_coralys_stops,
                "n_premature": stop_analysis.n_premature,
                "n_temporary_excursion": stop_analysis.n_temporary_excursion,
                "n_genuine_adverse": stop_analysis.n_genuine_adverse,
                "net_stop_benefit_inr": stop_analysis.net_stop_benefit_inr,
            }
        });
        fs::write(
            archive_dir.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata)?,
        )?;

        // ── Print per-config summary ──────────────────────────────────────────
        let p = &ledger.pe2_summary;
        let c = &ledger.coralys_summary;
        println!("  result=PASS  config={label}  alloc={alloc_desc}");
        println!("  universe_available={}", ledger.universe.len());
        println!("  P.E.2:     lots={} TARGET={} STOP={} HORIZON={} return={:+.2}% velocity={:.2}x",
            p.n_lots_opened, p.n_target, p.n_stop, p.n_horizon,
            p.total_return_pct * 100.0, p.capital_velocity_ratio);
        println!("  Coralys:   lots={} TARGET={} STOP={} HORIZON={} return={:+.2}% velocity={:.2}x",
            c.n_lots_opened, c.n_target, c.n_stop, c.n_horizon,
            c.total_return_pct * 100.0, c.capital_velocity_ratio);
        let stop_rate = if c.n_lots_opened > 0 { c.n_stop as f64 / c.n_lots_opened as f64 * 100.0 } else { 0.0 };
        println!("  stop_rate={:.1}%  premature={:.1}%  excursion={:.1}%  genuine={:.1}%",
            stop_rate, stop_analysis.pct_premature, stop_analysis.pct_temporary_excursion,
            stop_analysis.pct_genuine_adverse);
        println!("  archive={}", archive_dir.display());

        all_results.push((label.clone(), alloc_desc.clone(), ledger, stop_analysis));
    }

    // ── Write cross-config comparison matrix ──────────────────────────────────
    println!("\n─── Writing comparison matrix ───");

    let results_ref: Vec<(String, String, &ContinuousPortfolioLedger, &StopLossAnalysis)> =
        all_results.iter().map(|(l, a, ledger, stop)| (l.clone(), a.clone(), ledger, stop)).collect();

    // comparison_matrix.json
    let matrix_json: Vec<serde_json::Value> = results_ref.iter().map(|(label, alloc_desc, ledger, stop)| {
        let p = &ledger.pe2_summary;
        let c = &ledger.coralys_summary;
        let stop_rate = if c.n_lots_opened > 0 { c.n_stop as f64 / c.n_lots_opened as f64 * 100.0 } else { 0.0 };
        serde_json::json!({
            "config_label": label,
            "allocation_model": alloc_desc,
            "universe_size": ledger.universe.len(),
            "pe2": {
                "n_lots_opened": p.n_lots_opened,
                "n_target": p.n_target,
                "n_stop": p.n_stop,
                "n_horizon": p.n_horizon,
                "total_return_pct": p.total_return_pct * 100.0,
                "capital_velocity_ratio": p.capital_velocity_ratio,
                "total_realized_pnl_inr": p.total_realized_pnl_inr,
            },
            "coralys": {
                "n_lots_opened": c.n_lots_opened,
                "n_target": c.n_target,
                "n_stop": c.n_stop,
                "n_horizon": c.n_horizon,
                "total_return_pct": c.total_return_pct * 100.0,
                "capital_velocity_ratio": c.capital_velocity_ratio,
                "total_realized_pnl_inr": c.total_realized_pnl_inr,
                "stop_rate_pct": stop_rate,
                "pct_premature": stop.pct_premature,
                "pct_temporary_excursion": stop.pct_temporary_excursion,
                "pct_stop_too_tight": stop.pct_stop_too_tight,
                "pct_direction_failure": stop.pct_direction_failure,
                "pct_genuine_adverse": stop.pct_genuine_adverse,
            }
        })
    }).collect();

    fs::write(
        args.output_base.join("comparison_matrix.json"),
        serde_json::to_vec_pretty(&matrix_json)?,
    )?;

    // COMPARISON_REPORT.md
    let comparison_md = render_comparison_matrix(&results_ref);
    fs::write(args.output_base.join("COMPARISON_REPORT.md"), comparison_md)?;

    println!("  comparison_matrix.json written");
    println!("  COMPARISON_REPORT.md written");
    println!("\n═══ v0.4 Allocation Experiment complete ═══");
    println!("  output_base={}", args.output_base.display());

    Ok(())
}