//! Portfolio Replay v0.3 — Universe Robustness.
//!
//! Iterates over a set of `ContinuousPortfolioConfig` instances (universe sizes),
//! runs the frozen v0.2.1 engine for each, and writes a per-config archive plus
//! a cross-config robustness matrix.
//!
//! ## v0.3 contract (FROZEN — do not modify)
//!
//! ```text
//! Same engine          — run_continuous_portfolio_replay_with_config
//! Same historical period — PORTFOLIO_REPLAY_REQUESTED_CLOCK
//! Same initial capital — INITIAL_CAPITAL_INR (₹5,000)
//! Same C3-002          — RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH
//! Same Coralys v0      — CORALYS_EXEC_ARTIFACT_HASH
//! Same stop-loss       — enforced risk_boundary from CoralysExecutionIntent
//! Same allocation      — 10% of available cash per lot
//! Same lifecycle       — session-by-session, capital recycled on close
//!
//!              ONLY CHANGE: Universe size
//!                  │
//!        ┌─────────┼─────────┐
//!        ▼         ▼         ▼
//!       25        50        100
//!    instruments instruments instruments
//! ```
//!
//! ## Output per config
//!
//! ```text
//! {output_base}/v03_{label}/
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
//!   robustness_matrix.json    — all configs × all metrics
//!   ROBUSTNESS_REPORT.md      — human-readable matrix
//! ```
//!
//! ## Usage
//!
//! ```sh
//! cargo run --bin csp010_portfolio_v03 -- \
//!   --search-two product_validation/CS-P-006/discovery/20260815T051900Z_c3 \
//!   --cache-dir  product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache \
//!   --output-base historical_runs/portfolio_v03_universe_robustness
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
    refuse_v021_output, run_continuous_portfolio_replay_with_config, ContinuousPortfolioConfig,
    ContinuousPortfolioLedger, TradeLot, CONTINUOUS_REPLAY_VERSION,
};
use chronosentiment_adapter::decision_support::portfolio_replay_v0::INITIAL_CAPITAL_INR;
use chronosentiment_adapter::ingestion::yahoo::YahooHistoricalBar;

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
    pub entry_price: f64,
    pub stop_price: f64,
    pub exit_price: f64,
    pub entry_time: String,
    pub exit_time: String,
    pub holding_sessions: u32,
    pub realized_pnl_inr: f64,
    pub allocation_inr: f64,
    /// Gap magnitude: (exit_price - stop_price) / stop_price. Negative = gapped through.
    pub gap_magnitude_pct: f64,
    /// Post-stop max favorable excursion (highest close above stop within 5 sessions).
    pub post_stop_max_favorable_pct: Option<f64>,
    /// Whether price recovered to target within 3 sessions after stop.
    pub target_reached_after_stop: bool,
    /// Whether price recovered above stop level within 5 sessions.
    pub recovered_above_stop_within_5: bool,
    /// Whether price continued adverse for ≥5 sessions after stop.
    pub continued_adverse_5_sessions: bool,
    /// Stop tightness: |stop_price - entry_price| / entry_price.
    pub stop_tightness_pct: f64,
    /// Counterfactual P&L if held to horizon (hold-to-horizon close).
    pub counterfactual_pnl_inr: Option<f64>,
    /// Opportunity cost = counterfactual_pnl - realized_pnl (positive = stop cost us money).
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
    /// Sum of opportunity costs (positive = stops cost us money vs hold-to-horizon).
    pub total_opportunity_cost_inr: f64,
    /// Sum of realized P&L on stop lots.
    pub total_stop_realized_pnl_inr: f64,
    /// Net stop benefit = realized_pnl - counterfactual_pnl (positive = stops helped).
    pub net_stop_benefit_inr: f64,
    pub diagnostics: Vec<StopDiagnostic>,
}

/// Classify all Coralys STOP lots from a ledger.
///
/// Requires the bar cache to compute post-stop price paths.
fn classify_stops(
    ledger: &ContinuousPortfolioLedger,
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    config_label: &str,
) -> StopLossAnalysis {
    let stop_lots: Vec<&TradeLot> = ledger
        .coralys_arm
        .trade_log
        .iter()
        .filter(|l| matches!(l.exit_reason, Some(ExitReason::Stop)))
        .collect();

    let n = stop_lots.len();
    let mut diagnostics: Vec<StopDiagnostic> = Vec::with_capacity(n);

    for lot in &stop_lots {
        let stop_price = match lot.stop_price {
            Some(s) => s,
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

        // Gap magnitude: how far exit was from stop (negative = gapped through stop)
        let gap_magnitude_pct = if stop_price > 0.0 {
            (exit_price - stop_price) / stop_price
        } else {
            0.0
        };

        // Stop tightness: distance from entry to stop
        let stop_tightness_pct = if lot.entry_price > 0.0 {
            (stop_price - lot.entry_price).abs() / lot.entry_price
        } else {
            0.0
        };

        // Post-stop price path analysis
        let bars = cache.get(&lot.instrument);
        let (
            post_stop_max_favorable_pct,
            target_reached_after_stop,
            recovered_above_stop_within_5,
            continued_adverse_5_sessions,
            counterfactual_pnl_inr,
        ) = if let Some(bars) = bars {
            // Find bars after exit_time
            let exit_ts = chrono::DateTime::parse_from_rfc3339(&exit_time_str)
                .map(|t| t.with_timezone(&chrono::Utc))
                .ok();

            let post_stop_bars: Vec<&YahooHistoricalBar> = if let Some(exit_ts) = exit_ts {
                bars.iter()
                    .filter(|b| {
                        chrono::DateTime::from_timestamp(b.timestamp, 0)
                            .map(|t| t > exit_ts)
                            .unwrap_or(false)
                    })
                    .collect()
            } else {
                vec![]
            };

            // Max favorable within 5 sessions after stop
            let within_5: Vec<f64> = post_stop_bars
                .iter()
                .take(5)
                .map(|b| b.close)
                .collect();

            let max_favorable_pct = if !within_5.is_empty() && exit_price > 0.0 {
                let max_close = within_5.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                // For LONG: favorable = price going up above stop
                Some((max_close - exit_price) / exit_price)
            } else {
                None
            };

            // Target reached within 3 sessions after stop
            let target_reached = within_5.iter().take(3).any(|&c| c >= lot.target_price);

            // Recovered above stop within 5 sessions
            let recovered = within_5.iter().any(|&c| c > stop_price);

            // Continued adverse for ≥5 sessions (price stayed below stop)
            let continued_adverse = within_5.len() >= 5 && within_5.iter().all(|&c| c < stop_price);

            // Counterfactual: hold to horizon (last available bar in cache)
            let last_bar_close = bars.last().map(|b| b.close);
            let counterfactual = last_bar_close.map(|close| {
                let ret = match lot.direction.as_str() {
                    "LONG" => (close - lot.entry_price) / lot.entry_price,
                    "SHORT" => (lot.entry_price - close) / lot.entry_price,
                    _ => 0.0,
                };
                lot.allocation_inr * ret
            });

            (max_favorable_pct, target_reached, recovered, continued_adverse, counterfactual)
        } else {
            (None, false, false, false, None)
        };

        let opportunity_cost_inr = counterfactual_pnl_inr.map(|cf| cf - realized_pnl);

        // Classification (priority order: GAP_THROUGH → PREMATURE → TEMPORARY_EXCURSION
        //                                → STOP_TOO_TIGHT → DIRECTION_FAILURE → GENUINE_ADVERSE)
        let category = if gap_magnitude_pct < -0.005 {
            // Exited more than 0.5% below stop — gapped through
            StopCategory::GapThrough
        } else if target_reached_after_stop {
            // Price reached target within 3 sessions — premature stop
            StopCategory::PrematureStop
        } else if recovered_above_stop_within_5 {
            // Price recovered above stop within 5 sessions — temporary excursion
            StopCategory::TemporaryExcursion
        } else if stop_tightness_pct < 0.01 {
            // Stop was within 1% of entry — too tight
            StopCategory::StopTooTight
        } else if continued_adverse_5_sessions {
            // Price continued adverse — directional failure
            StopCategory::DirectionFailure
        } else {
            // None of the above — genuinely adverse
            StopCategory::GenuineAdverse
        };

        diagnostics.push(StopDiagnostic {
            trade_id: lot.trade_id.clone(),
            instrument: lot.instrument.clone(),
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
            target_reached_after_stop: target_reached_after_stop,
            recovered_above_stop_within_5: recovered_above_stop_within_5,
            continued_adverse_5_sessions: continued_adverse_5_sessions,
            stop_tightness_pct,
            counterfactual_pnl_inr,
            opportunity_cost_inr,
            category,
        });
    }

    // Aggregate counts
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

fn render_v03_report(ledger: &ContinuousPortfolioLedger, stop_analysis: &StopLossAnalysis, config_label: &str) -> String {
    let p = &ledger.pe2_summary;
    let c = &ledger.coralys_summary;
    let mut md = String::new();

    md.push_str(&format!("# Portfolio Replay v0.3 — Universe Robustness: {config_label}\n\n"));
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Experiment:** Universe robustness — frozen engine, variable universe  \n");
    md.push_str("**Contract:** Same engine/period/capital/C3-002/Coralys-v0/stop/allocation as v0.2.1  \n");
    md.push_str("**Only change:** Universe size  \n\n");

    md.push_str("## Setup\n\n");
    md.push_str(&format!("- Config label: `{config_label}`\n"));
    md.push_str(&format!("- Universe size: {} instruments\n", ledger.universe.len()));
    md.push_str(&format!("- Universe: {}\n", ledger.universe.join(", ")));
    md.push_str(&format!("- Certified T: {}\n", ledger.certified_t));
    md.push_str(&format!("- Sessions simulated: {}\n", ledger.n_sessions_simulated));
    md.push_str(&format!("- Initial capital: Rs.{:.2}\n", ledger.initial_capital_inr));
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

    md.push_str("## Portfolio Performance\n\n");
    md.push_str("| Metric | P.E.2 | Coralys v0 |\n");
    md.push_str("|--------|-------|------------|\n");
    md.push_str(&format!("| Final portfolio value | Rs.{:.2} | Rs.{:.2} |\n", p.final_portfolio_value_inr, c.final_portfolio_value_inr));
    md.push_str(&format!("| Total return | {:+.2}% | {:+.2}% |\n", p.total_return_pct * 100.0, c.total_return_pct * 100.0));
    md.push_str(&format!("| Realized P&L | Rs.{:+.2} | Rs.{:+.2} |\n", p.total_realized_pnl_inr, c.total_realized_pnl_inr));
    md.push_str(&format!("| Max drawdown | {:.2}% | {:.2}% |\n", p.max_drawdown_pct * 100.0, c.max_drawdown_pct * 100.0));
    md.push_str("\n");

    md.push_str("## Stop-Loss Behaviour\n\n");
    let sa = stop_analysis;
    md.push_str(&format!("- Total Coralys STOP exits: {}\n", sa.n_coralys_stops));
    if sa.n_coralys_stops > 0 {
        let stop_rate = c.n_stop as f64 / c.n_lots_opened as f64 * 100.0;
        md.push_str(&format!("- Stop rate: {:.1}% of lots\n", stop_rate));
        md.push_str(&format!("- GAP_THROUGH: {} ({:.1}%)\n", sa.n_gap_through, sa.pct_gap_through));
        md.push_str(&format!("- PREMATURE_STOP: {} ({:.1}%)\n", sa.n_premature, sa.pct_premature));
        md.push_str(&format!("- TEMPORARY_EXCURSION: {} ({:.1}%)\n", sa.n_temporary_excursion, sa.pct_temporary_excursion));
        md.push_str(&format!("- STOP_TOO_TIGHT: {} ({:.1}%)\n", sa.n_stop_too_tight, sa.pct_stop_too_tight));
        md.push_str(&format!("- DIRECTION_FAILURE: {} ({:.1}%)\n", sa.n_direction_failure, sa.pct_direction_failure));
        md.push_str(&format!("- GENUINE_ADVERSE: {} ({:.1}%)\n", sa.n_genuine_adverse, sa.pct_genuine_adverse));
        md.push_str(&format!("- Net stop benefit vs hold-to-horizon: Rs.{:+.2}\n", sa.net_stop_benefit_inr));
    }
    md.push_str("\n");

    md.push_str("## Integrity\n\n");
    md.push_str(&format!("{}\n", ledger.integrity_note));

    md
}

fn render_robustness_matrix(results: &[(String, &ContinuousPortfolioLedger, &StopLossAnalysis)]) -> String {
    let mut md = String::new();
    md.push_str("# Portfolio Replay v0.3 — Universe Robustness Matrix\n\n");
    md.push_str("**v0.2.1 baseline (7 instruments) is the reference row.**  \n");
    md.push_str("All other rows use the same frozen engine with expanded universes.  \n\n");

    md.push_str("## Coralys v0 Arm\n\n");
    md.push_str("| Config | Universe | Lots | Velocity | Return | Max DD | Stop% | Premature% | Excursion% | Genuine% | Stop Benefit |\n");
    md.push_str("|--------|----------|------|----------|--------|--------|-------|------------|------------|----------|--------------|\n");

    for (label, ledger, sa) in results {
        let c = &ledger.coralys_summary;
        let stop_rate = if c.n_lots_opened > 0 { c.n_stop as f64 / c.n_lots_opened as f64 * 100.0 } else { 0.0 };
        md.push_str(&format!(
            "| {} | {} | {} | {:.2}x | {:+.2}% | {:.2}% | {:.1}% | {:.1}% | {:.1}% | {:.1}% | Rs.{:+.0} |\n",
            label,
            ledger.universe.len(),
            c.n_lots_opened,
            c.capital_velocity_ratio,
            c.total_return_pct * 100.0,
            c.max_drawdown_pct * 100.0,
            stop_rate,
            sa.pct_premature,
            sa.pct_temporary_excursion,
            sa.pct_genuine_adverse,
            sa.net_stop_benefit_inr,
        ));
    }
    md.push_str("\n");

    md.push_str("## P.E.2 Arm\n\n");
    md.push_str("| Config | Universe | Lots | Velocity | Return | Max DD |\n");
    md.push_str("|--------|----------|------|----------|--------|--------|\n");
    for (label, ledger, _) in results {
        let p = &ledger.pe2_summary;
        md.push_str(&format!(
            "| {} | {} | {} | {:.2}x | {:+.2}% | {:.2}% |\n",
            label,
            ledger.universe.len(),
            p.n_lots_opened,
            p.capital_velocity_ratio,
            p.total_return_pct * 100.0,
            p.max_drawdown_pct * 100.0,
        ));
    }

    md
}

// ─── Universe definitions ─────────────────────────────────────────────────────

/// v0.3-A: 25-instrument universe (NSE large-cap expansion).
///
/// The 7 v0.2.1 instruments are included. Additional instruments require bar
/// data in the cache — missing instruments are silently skipped by the engine.
const V03_A_25: &[&str] = &[
    // v0.2.1 baseline (7)
    "HDFCBANK.NS", "RELIANCE.NS", "TCS.NS", "INFY.NS",
    "ICICIBANK.NS", "HINDUNILVR.NS", "ITC.NS",
    // NSE large-cap expansion (18)
    "KOTAKBANK.NS", "AXISBANK.NS", "SBIN.NS", "BAJFINANCE.NS",
    "BHARTIARTL.NS", "ASIANPAINT.NS", "MARUTI.NS", "TITAN.NS",
    "SUNPHARMA.NS", "WIPRO.NS", "HCLTECH.NS", "ULTRACEMCO.NS",
    "NESTLEIND.NS", "POWERGRID.NS", "NTPC.NS", "ONGC.NS",
    "TMPV.NS", "TATASTEEL.NS",
];

/// v0.3-B: 50-instrument universe (NSE mid-cap expansion).
const V03_B_50: &[&str] = &[
    // All 25 from v0.3-A
    "HDFCBANK.NS", "RELIANCE.NS", "TCS.NS", "INFY.NS",
    "ICICIBANK.NS", "HINDUNILVR.NS", "ITC.NS",
    "KOTAKBANK.NS", "AXISBANK.NS", "SBIN.NS", "BAJFINANCE.NS",
    "BHARTIARTL.NS", "ASIANPAINT.NS", "MARUTI.NS", "TITAN.NS",
    "SUNPHARMA.NS", "WIPRO.NS", "HCLTECH.NS", "ULTRACEMCO.NS",
    "NESTLEIND.NS", "POWERGRID.NS", "NTPC.NS", "ONGC.NS",
    "TMPV.NS", "TATASTEEL.NS",
    // Mid-cap expansion (25)
    "ADANIENT.NS", "ADANIPORTS.NS", "BAJAJFINSV.NS", "BPCL.NS",
    "BRITANNIA.NS", "CIPLA.NS", "COALINDIA.NS", "DIVISLAB.NS",
    "DRREDDY.NS", "EICHERMOT.NS", "GRASIM.NS", "HEROMOTOCO.NS",
    "HINDALCO.NS", "INDUSINDBK.NS", "JSWSTEEL.NS", "LT.NS",
    "M&M.NS", "PIDILITIND.NS", "SBILIFE.NS", "SHREECEM.NS",
    "SIEMENS.NS", "TECHM.NS", "TRENT.NS", "UPL.NS",
    "VEDL.NS",
];

/// v0.3-C: 100-instrument universe (NSE broad market).
const V03_C_100: &[&str] = &[
    // All 50 from v0.3-B
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
    // Broad market expansion (50)
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
];

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── Environment guard ─────────────────────────────────────────────────────
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    // ── Artifact hash guard ───────────────────────────────────────────────────
    if CORALYS_EXEC_ARTIFACT_HASH
        != "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
    {
        return Err(format!(
            "coralys artifact hash mismatch: {CORALYS_EXEC_ARTIFACT_HASH}"
        )
        .into());
    }

    // ── Load C3-002 artifact ──────────────────────────────────────────────────
    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }

    // ── Load Yahoo bar cache ──────────────────────────────────────────────────
    let cache = load_required_yahoo_cache(&args.cache_dir).map_err(|e| e.to_string())?;
    println!("Loaded bar cache: {} instruments", cache.len());

    // ── Build config list ─────────────────────────────────────────────────────
    // Filter each universe to only instruments present in the cache.
    // Missing instruments are noted but do not abort the run.
    let configs: Vec<(ContinuousPortfolioConfig, usize)> = vec![
        (ContinuousPortfolioConfig {
            universe: filter_to_cache(V03_A_25, &cache, args.strict),
            config_label: "v03_A_25".to_string(),
            contributions: vec![],
            allocation_model: chronosentiment_adapter::decision_support::portfolio_replay_v021::AllocationModel::EqualWeight,
            initial_capital_inr: 5000.0,
        }, V03_A_25.len()),
        (ContinuousPortfolioConfig {
            universe: filter_to_cache(V03_B_50, &cache, args.strict),
            config_label: "v03_B_50".to_string(),
            contributions: vec![],
            allocation_model: chronosentiment_adapter::decision_support::portfolio_replay_v021::AllocationModel::EqualWeight,
            initial_capital_inr: 5000.0,
        }, V03_B_50.len()),
        (ContinuousPortfolioConfig {
            universe: filter_to_cache(V03_C_100, &cache, args.strict),
            config_label: "v03_C_100".to_string(),
            contributions: vec![],
            allocation_model: chronosentiment_adapter::decision_support::portfolio_replay_v021::AllocationModel::EqualWeight,
            initial_capital_inr: 5000.0,
        }, V03_C_100.len()),
    ];

    // ── Run each config ───────────────────────────────────────────────────────
    let mut all_results: Vec<(String, ContinuousPortfolioLedger, StopLossAnalysis)> = Vec::new();

    for (config, requested_size) in &configs {
        let label = &config.config_label;
        let available = config.universe.len();

        // Strict mode: abort if available < requested
        if args.strict && available < *requested_size {
            return Err(format!(
                "strict mode: config {label} requested {requested_size} instruments but only {available} are in cache. \
                 Acquire bar data for the missing instruments and retry."
            ).into());
        }

        println!("\n─── Running config: {label} ({available}/{requested_size} instruments in cache) ───");

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
        // Archive dir uses label directly (no extra prefix) → v03_A_25, v03_B_50, v03_C_100
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
        let report_md = render_v03_report(&ledger, &stop_analysis, label);
        fs::write(archive_dir.join("REPORT.md"), report_md)?;

        // metadata.json
        let metadata = serde_json::json!({
            "experiment": "Portfolio Replay v0.3 — Universe Robustness",
            "config_label": label,
            "path_kind": CONTINUOUS_REPLAY_VERSION,
            "start_clock": ledger.start_clock,
            "certified_t": ledger.certified_t,
            "initial_capital_inr": INITIAL_CAPITAL_INR,
            "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
            "coralys_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
            "n_sessions_simulated": ledger.n_sessions_simulated,
            "universe_requested": config.universe.len(),
            "universe_available": ledger.universe.len(),
            "universe": ledger.universe,
            "v03_contract": {
                "engine": "run_continuous_portfolio_replay_with_config",
                "only_change": "universe",
                "frozen": ["C3-002", "Coralys-v0", "stop-loss", "allocation", "lifecycle", "historical-period"]
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
        println!("  result=PASS  config={label}");
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

        all_results.push((label.clone(), ledger, stop_analysis));
    }

    // ── Write cross-config robustness matrix ──────────────────────────────────
    fs::create_dir_all(&args.output_base)?;

    let matrix_refs: Vec<(String, &ContinuousPortfolioLedger, &StopLossAnalysis)> = all_results
        .iter()
        .map(|(l, ledger, sa)| (l.clone(), ledger, sa))
        .collect();

    let robustness_md = render_robustness_matrix(&matrix_refs);
    fs::write(args.output_base.join("ROBUSTNESS_REPORT.md"), robustness_md)?;

    // robustness_matrix.json — machine-readable cross-config summary
    let matrix_json: Vec<serde_json::Value> = all_results.iter().map(|(label, ledger, sa)| {
        let p = &ledger.pe2_summary;
        let c = &ledger.coralys_summary;
        let stop_rate = if c.n_lots_opened > 0 { c.n_stop as f64 / c.n_lots_opened as f64 * 100.0 } else { 0.0 };
        serde_json::json!({
            "config_label": label,
            "universe_size": ledger.universe.len(),
            "pe2": {
                "n_lots": p.n_lots_opened,
                "capital_velocity": p.capital_velocity_ratio,
                "total_return_pct": p.total_return_pct * 100.0,
                "max_drawdown_pct": p.max_drawdown_pct * 100.0,
            },
            "coralys": {
                "n_lots": c.n_lots_opened,
                "capital_velocity": c.capital_velocity_ratio,
                "total_return_pct": c.total_return_pct * 100.0,
                "max_drawdown_pct": c.max_drawdown_pct * 100.0,
                "stop_rate_pct": stop_rate,
                "pct_premature": sa.pct_premature,
                "pct_temporary_excursion": sa.pct_temporary_excursion,
                "pct_stop_too_tight": sa.pct_stop_too_tight,
                "pct_direction_failure": sa.pct_direction_failure,
                "pct_genuine_adverse": sa.pct_genuine_adverse,
                "net_stop_benefit_inr": sa.net_stop_benefit_inr,
            }
        })
    }).collect();
    fs::write(
        args.output_base.join("robustness_matrix.json"),
        serde_json::to_vec_pretty(&matrix_json)?,
    )?;

    println!("\n═══ v0.3 Universe Robustness — All configs complete ═══");
    println!("output_base={}", args.output_base.display());
    println!("ROBUSTNESS_REPORT.md written");
    println!("robustness_matrix.json written");

    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Filter a universe slice to only instruments present in the bar cache.
///
/// In non-strict mode: missing instruments are logged to stderr and skipped.
/// In strict mode: missing instruments are still logged; the caller checks and aborts.
fn filter_to_cache(
    universe: &[&str],
    cache: &BTreeMap<String, Vec<YahooHistoricalBar>>,
    strict: bool,
) -> Vec<String> {
    let available: Vec<String> = universe
        .iter()
        .filter(|&&sym| cache.contains_key(sym))
        .map(|s| s.to_string())
        .collect();
    let missing: Vec<&str> = universe.iter()
        .filter(|sym| !cache.contains_key(**sym))
        .copied()
        .collect();
    if !missing.is_empty() {
        let mode = if strict { "STRICT — will abort" } else { "skipped" };
        eprintln!(
            "  [cache] {} instruments not in cache ({}): {}",
            missing.len(),
            mode,
            missing.join(", ")
        );
    }
    available
}

// ─── Argument parsing ─────────────────────────────────────────────────────────

struct V03Args {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output_base: PathBuf,
    /// If true: abort when available instruments < requested. Use for official v0.3 runs.
    /// If false (default): silently degrade to available instruments (development mode).
    strict: bool,
}

fn parse_args() -> Result<V03Args, Box<dyn std::error::Error>> {
    let mut search_two: Option<PathBuf> = None;
    let mut cache_dir: Option<PathBuf> = None;
    let mut output_base: Option<PathBuf> = None;
    let mut strict = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-two" => {
                search_two = Some(PathBuf::from(
                    args.next().ok_or("--search-two requires a value")?,
                ));
            }
            "--cache-dir" => {
                cache_dir = Some(PathBuf::from(
                    args.next().ok_or("--cache-dir requires a value")?,
                ));
            }
            "--output-base" => {
                output_base = Some(PathBuf::from(
                    args.next().ok_or("--output-base requires a value")?,
                ));
            }
            "--strict" => {
                strict = true;
            }
            other => {
                return Err(format!("unknown argument: {other}").into());
            }
        }
    }

    let search_two = search_two.unwrap_or_else(|| {
        PathBuf::from("product_validation/CS-P-006/discovery/20260815T051900Z_c3")
    });
    let cache_dir = cache_dir.unwrap_or_else(|| {
        PathBuf::from(
            "product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache",
        )
    });
    let output_base = output_base.unwrap_or_else(|| {
        PathBuf::from("historical_runs/portfolio_v03_universe_robustness")
    });

    Ok(V03Args {
        search_two,
        cache_dir,
        output_base,
        strict,
    })
}