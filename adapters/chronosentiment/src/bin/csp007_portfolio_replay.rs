//! Portfolio Historical Replay v0.1 — P.E.2 vs Coralys v0 Portfolio Comparison.
//!
//! Runs two independent simulated portfolios over the exact P.E.2 historical
//! period (certified T = 2026-07-15T03:45:00Z):
//!
//!   - P.E.2 arm: fixed +5% target, no stop
//!   - Coralys v0 arm: ATR/TMV target + enforced risk_boundary stop
//!
//! Both start with Rs.5,000 initial capital. Capital is recycled after exits.
//!
//! ## Usage
//!
//! ```sh
//! cargo run --bin csp007_portfolio_replay -- \
//!   --search-two product_validation/CS-P-006/discovery/20260815T051900Z_c3 \
//!   --cache-dir product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache \
//!   --output historical_runs/portfolio_comparison_pe2_vs_pe3_2026-08-16
//! ```
//!
//! ## Output
//!
//! ```
//! historical_runs/portfolio_comparison_pe2_vs_pe3_2026-08-16/
//!   portfolio_replay_ledger.json   — full ledger with both arms
//!   portfolio_comparison.json      — comparison report
//!   portfolio_REPORT.md            — human-readable report
//!   metadata.json                  — experiment metadata
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::coralys_execution_model::CORALYS_EXEC_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::portfolio_replay_v0::{
    refuse_portfolio_replay_output, run_portfolio_replay, INITIAL_CAPITAL_INR,
    PORTFOLIO_REPLAY_EXPERIMENT_ID, PORTFOLIO_REPLAY_PATH_KIND,
    PORTFOLIO_REPLAY_REQUESTED_CLOCK,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── Environment guard ─────────────────────────────────────────────────────
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    // ── Output path guard ─────────────────────────────────────────────────────
    refuse_portfolio_replay_output(&args.output.to_string_lossy())?;

    // ── Load C3-002 artifact ──────────────────────────────────────────────────
    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }

    // ── Coralys artifact hash guard ───────────────────────────────────────────
    if CORALYS_EXEC_ARTIFACT_HASH
        != "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
    {
        return Err(format!(
            "coralys artifact hash mismatch: {CORALYS_EXEC_ARTIFACT_HASH}"
        )
        .into());
    }

    // ── Load Yahoo bar cache ──────────────────────────────────────────────────
    let cache = load_required_yahoo_cache(&args.cache_dir).map_err(|e| e.to_string())?;

    // ── Run Portfolio Replay v0.1 ─────────────────────────────────────────────
    let ledger = run_portfolio_replay(&artifact, &cache)?;

    // ── Validate output ───────────────────────────────────────────────────────
    if ledger.path_kind != PORTFOLIO_REPLAY_PATH_KIND {
        return Err(format!(
            "unexpected path_kind: expected {PORTFOLIO_REPLAY_PATH_KIND}, got {}",
            ledger.path_kind
        )
        .into());
    }
    if ledger.coralys_artifact_hash
        != "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
    {
        return Err(format!(
            "ledger coralys_artifact_hash mismatch: {}",
            ledger.coralys_artifact_hash
        )
        .into());
    }

    // ── Write output ──────────────────────────────────────────────────────────
    fs::create_dir_all(&args.output)?;

    // Full ledger
    let ledger_path = args.output.join("portfolio_replay_ledger.json");
    refuse_portfolio_replay_output(&ledger_path.to_string_lossy())?;
    fs::write(&ledger_path, serde_json::to_vec_pretty(&ledger)?)?;

    // Comparison report (JSON)
    let comparison_path = args.output.join("portfolio_comparison.json");
    refuse_portfolio_replay_output(&comparison_path.to_string_lossy())?;
    fs::write(
        &comparison_path,
        serde_json::to_vec_pretty(&ledger.comparison)?,
    )?;

    // Human-readable report (Markdown)
    let md = render_portfolio_report(&ledger);
    fs::write(args.output.join("portfolio_REPORT.md"), md)?;

    // Metadata
    let metadata = serde_json::json!({
        "experiment": "Portfolio Replay v0.1",
        "experiment_id": PORTFOLIO_REPLAY_EXPERIMENT_ID,
        "path_kind": PORTFOLIO_REPLAY_PATH_KIND,
        "requested_clock": PORTFOLIO_REPLAY_REQUESTED_CLOCK,
        "certified_t": ledger.certified_t,
        "initial_capital_inr": INITIAL_CAPITAL_INR,
        "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
        "coralys_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
        "status": "IMMUTABLE",
        "archived_at": "2026-08-16",
        "purpose": "Portfolio-level comparison: P.E.2 (fixed +5%, no stop) vs Coralys v0 (ATR/TMV + enforced stop). Same period, same decisions, same capital. Divergence is the product effect.",
        "arms": {
            "pe2": {
                "contract": ledger.pe2_arm.execution_contract,
                "n_positions": ledger.pe2_arm.n_positions_opened,
                "n_target": ledger.pe2_arm.n_target,
                "n_stop": ledger.pe2_arm.n_stop,
                "n_horizon": ledger.pe2_arm.n_horizon,
            },
            "coralys_v0": {
                "contract": ledger.coralys_arm.execution_contract,
                "n_positions": ledger.coralys_arm.n_positions_opened,
                "n_target": ledger.coralys_arm.n_target,
                "n_stop": ledger.coralys_arm.n_stop,
                "n_horizon": ledger.coralys_arm.n_horizon,
            }
        }
    });
    fs::write(
        args.output.join("metadata.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;

    // ── Print summary ─────────────────────────────────────────────────────────
    let c = &ledger.comparison;
    println!("result=PASS");
    println!("experiment_id={}", PORTFOLIO_REPLAY_EXPERIMENT_ID);
    println!("certified_t={}", ledger.certified_t);
    println!("initial_capital_inr={:.2}", INITIAL_CAPITAL_INR);
    println!();
    println!("=== P.E.2 ARM ===");
    println!("  contract={}", c.pe2_arm.execution_contract);
    println!("  positions={}", c.pe2_arm.n_positions_opened);
    println!("  TARGET={} STOP={} HORIZON={}", c.pe2_arm.n_target, c.pe2_arm.n_stop, c.pe2_arm.n_horizon);
    println!("  final_value={:.2}", c.pe2_arm.final_portfolio_value_inr);
    println!("  total_return={:+.2}%", c.pe2_arm.total_return_pct * 100.0);
    println!("  realized_pnl={:+.2}", c.pe2_arm.total_realized_pnl_inr);
    println!("  max_drawdown={:.2}%", c.pe2_arm.max_drawdown_pct * 100.0);
    if let Some(avg) = c.pe2_arm.avg_holding_sessions {
        println!("  avg_hold={:.1} sessions", avg);
    }
    println!();
    println!("=== CORALYS V0 ARM ===");
    println!("  contract={}", c.coralys_arm.execution_contract);
    println!("  positions={}", c.coralys_arm.n_positions_opened);
    println!("  TARGET={} STOP={} HORIZON={}", c.coralys_arm.n_target, c.coralys_arm.n_stop, c.coralys_arm.n_horizon);
    println!("  final_value={:.2}", c.coralys_arm.final_portfolio_value_inr);
    println!("  total_return={:+.2}%", c.coralys_arm.total_return_pct * 100.0);
    println!("  realized_pnl={:+.2}", c.coralys_arm.total_realized_pnl_inr);
    println!("  max_drawdown={:.2}%", c.coralys_arm.max_drawdown_pct * 100.0);
    if let Some(avg) = c.coralys_arm.avg_holding_sessions {
        println!("  avg_hold={:.1} sessions", avg);
    }
    println!();
    println!("output={}", args.output.display());

    Ok(())
}

// ─── Report rendering ─────────────────────────────────────────────────────────

fn render_portfolio_report(
    ledger: &chronosentiment_adapter::decision_support::portfolio_replay_v0::PortfolioReplayLedger,
) -> String {
    let c = &ledger.comparison;
    let mut md = String::new();
    md.push_str("# Portfolio Historical Replay v0.1\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Experiment:** Portfolio Replay v0.1 — P.E.2 vs Coralys v0  \n");
    md.push_str("**Does not:** modify C3-002, modify coralys-exec-v0, touch P.E.2 or P.E.3-B archives  \n\n");
    md.push_str("## Setup\n\n");
    md.push_str(&format!("- Certified T: {}\n", ledger.certified_t));
    md.push_str(&format!("- Initial capital: Rs.{:.2}\n", ledger.initial_capital_inr));
    md.push_str(&format!("- C3-002 artifact: `{}`\n", ledger.c3_002_artifact_hash));
    md.push_str(&format!("- Coralys artifact: `{}`\n\n", ledger.coralys_artifact_hash));
    md.push_str("## P.E.2 Arm (fixed +5%, no stop)\n\n");
    md.push_str(&format!("- Contract: `{}`\n", c.pe2_arm.execution_contract));
    md.push_str(&format!("- Positions opened: {}\n", c.pe2_arm.n_positions_opened));
    md.push_str(&format!("- TARGET: {} | STOP: {} | HORIZON: {} | AMBIGUOUS: {}\n",
        c.pe2_arm.n_target, c.pe2_arm.n_stop, c.pe2_arm.n_horizon, c.pe2_arm.n_ambiguous));
    md.push_str(&format!("- Final portfolio value: Rs.{:.2}\n", c.pe2_arm.final_portfolio_value_inr));
    md.push_str(&format!("- Total return: {:+.2}%\n", c.pe2_arm.total_return_pct * 100.0));
    md.push_str(&format!("- Realized P&L: Rs.{:+.2}\n", c.pe2_arm.total_realized_pnl_inr));
    md.push_str(&format!("- Unrealized P&L: Rs.{:+.2}\n", c.pe2_arm.total_unrealized_pnl_inr));
    md.push_str(&format!("- Max drawdown: {:.2}% (Rs.{:.2})\n", c.pe2_arm.max_drawdown_pct * 100.0, c.pe2_arm.max_drawdown_inr));
    if let Some(avg) = c.pe2_arm.avg_holding_sessions {
        md.push_str(&format!("- Avg holding: {:.1} sessions\n", avg));
    }
    md.push_str("\n## Coralys v0 Arm (ATR/TMV target + enforced stop)\n\n");
    md.push_str(&format!("- Contract: `{}`\n", c.coralys_arm.execution_contract));
    md.push_str(&format!("- Positions opened: {}\n", c.coralys_arm.n_positions_opened));
    md.push_str(&format!("- TARGET: {} | STOP: {} | HORIZON: {} | AMBIGUOUS: {}\n",
        c.coralys_arm.n_target, c.coralys_arm.n_stop, c.coralys_arm.n_horizon, c.coralys_arm.n_ambiguous));
    md.push_str(&format!("- Final portfolio value: Rs.{:.2}\n", c.coralys_arm.final_portfolio_value_inr));
    md.push_str(&format!("- Total return: {:+.2}%\n", c.coralys_arm.total_return_pct * 100.0));
    md.push_str(&format!("- Realized P&L: Rs.{:+.2}\n", c.coralys_arm.total_realized_pnl_inr));
    md.push_str(&format!("- Unrealized P&L: Rs.{:+.2}\n", c.coralys_arm.total_unrealized_pnl_inr));
    md.push_str(&format!("- Max drawdown: {:.2}% (Rs.{:.2})\n", c.coralys_arm.max_drawdown_pct * 100.0, c.coralys_arm.max_drawdown_inr));
    if let Some(avg) = c.coralys_arm.avg_holding_sessions {
        md.push_str(&format!("- Avg holding: {:.1} sessions\n", avg));
    }
    md.push_str("\n## Notes\n\n");
    md.push_str(&format!("{}\n\n", c.exploratory_note));
    md.push_str(&format!("{}\n", c.methodology_note));
    md
}

// ─── Argument parsing ─────────────────────────────────────────────────────────

struct PortfolioReplayArgs {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<PortfolioReplayArgs, Box<dyn std::error::Error>> {
    let mut search_two: Option<PathBuf> = None;
    let mut cache_dir: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

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
            "--output" => {
                output = Some(PathBuf::from(
                    args.next().ok_or("--output requires a value")?,
                ));
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
    let output = output.unwrap_or_else(|| {
        PathBuf::from("historical_runs/portfolio_comparison_pe2_vs_pe3_2026-08-16")
    });

    Ok(PortfolioReplayArgs {
        search_two,
        cache_dir,
        output,
    })
}