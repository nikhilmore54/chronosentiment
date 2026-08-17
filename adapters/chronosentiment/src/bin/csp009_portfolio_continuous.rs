//! Portfolio Replay v0.2.1 — Continuous Lifecycle with Position Upgrades.
//!
//! Runs a session-by-session continuous portfolio simulation over the full
//! P.E.2 historical period. At each session:
//!
//!   1. Scan all open lots for exits (TARGET / STOP / HORIZON).
//!   2. For each instrument, generate a C3-002 decision.
//!   3. If LONG/SHORT and cash available, open a new lot.
//!
//! Multiple lots per instrument are allowed (position upgrades).
//! Capital is recycled when a lot closes.
//!
//! ## Usage
//!
//! ```sh
//! cargo run --bin csp009_portfolio_continuous -- \
//!   --search-two product_validation/CS-P-006/discovery/20260815T051900Z_c3 \
//!   --cache-dir product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache \
//!   --output historical_runs/portfolio_continuous_v021_2026-08-16
//! ```
//!
//! ## Output
//!
//! ```
//! historical_runs/portfolio_continuous_v021_2026-08-16/
//!   continuous_ledger.json    — full ledger with both arms + session snapshots
//!   continuous_REPORT.md      — human-readable report
//!   metadata.json             — experiment metadata
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::coralys_execution_model::CORALYS_EXEC_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::portfolio_replay_v021::{
    refuse_v021_output, run_continuous_portfolio_replay, ContinuousPortfolioLedger,
    CONTINUOUS_EXPERIMENT_ID, CONTINUOUS_REPLAY_VERSION,
};
use chronosentiment_adapter::decision_support::portfolio_replay_v0::INITIAL_CAPITAL_INR;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── Environment guard ─────────────────────────────────────────────────────
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    // ── Output path guard ─────────────────────────────────────────────────────
    refuse_v021_output(&args.output.to_string_lossy())?;

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

    // ── Run Portfolio Replay v0.2.1 ───────────────────────────────────────────
    println!("Running Portfolio Replay v0.2.1 — Continuous Lifecycle...");
    let ledger = run_continuous_portfolio_replay(&artifact, &cache)?;

    // ── Validate output ───────────────────────────────────────────────────────
    if ledger.path_kind != CONTINUOUS_REPLAY_VERSION {
        return Err(format!(
            "unexpected path_kind: expected {CONTINUOUS_REPLAY_VERSION}, got {}",
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
    let ledger_path = args.output.join("continuous_ledger.json");
    refuse_v021_output(&ledger_path.to_string_lossy())?;
    fs::write(&ledger_path, serde_json::to_vec_pretty(&ledger)?)?;

    // Human-readable report
    let md = render_continuous_report(&ledger);
    fs::write(args.output.join("continuous_REPORT.md"), md)?;

    // Metadata
    let metadata = serde_json::json!({
        "experiment": "Portfolio Replay v0.2.1 — Continuous Lifecycle",
        "experiment_id": CONTINUOUS_EXPERIMENT_ID,
        "path_kind": CONTINUOUS_REPLAY_VERSION,
        "start_clock": ledger.start_clock,
        "certified_t": ledger.certified_t,
        "initial_capital_inr": INITIAL_CAPITAL_INR,
        "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
        "coralys_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
        "n_sessions_simulated": ledger.n_sessions_simulated,
        "universe": ledger.universe,
        "status": "IMMUTABLE",
        "archived_at": "2026-08-16",
        "purpose": "Continuous portfolio lifecycle: session-by-session loop, capital recycling, \
            multiple lots per instrument (position upgrades allowed). \
            P.E.2 arm (fixed +5%, no stop) vs Coralys v0 arm (ATR/TMV + enforced stop). \
            Measures capital velocity and portfolio behaviour over the full P.E.2 period.",
        "arms": {
            "pe2": {
                "contract": ledger.pe2_summary.execution_contract,
                "n_lots_opened": ledger.pe2_summary.n_lots_opened,
                "n_target": ledger.pe2_summary.n_target,
                "n_stop": ledger.pe2_summary.n_stop,
                "n_horizon": ledger.pe2_summary.n_horizon,
                "capital_velocity_ratio": ledger.pe2_summary.capital_velocity_ratio,
                "total_return_pct": ledger.pe2_summary.total_return_pct,
            },
            "coralys_v0": {
                "contract": ledger.coralys_summary.execution_contract,
                "n_lots_opened": ledger.coralys_summary.n_lots_opened,
                "n_target": ledger.coralys_summary.n_target,
                "n_stop": ledger.coralys_summary.n_stop,
                "n_horizon": ledger.coralys_summary.n_horizon,
                "capital_velocity_ratio": ledger.coralys_summary.capital_velocity_ratio,
                "total_return_pct": ledger.coralys_summary.total_return_pct,
            }
        }
    });
    fs::write(
        args.output.join("metadata.json"),
        serde_json::to_vec_pretty(&metadata)?,
    )?;

    // ── Print summary ─────────────────────────────────────────────────────────
    let p = &ledger.pe2_summary;
    let c = &ledger.coralys_summary;

    println!("result=PASS");
    println!("experiment_id={}", CONTINUOUS_EXPERIMENT_ID);
    println!("certified_t={}", ledger.certified_t);
    println!("n_sessions_simulated={}", ledger.n_sessions_simulated);
    println!("initial_capital_inr={:.2}", INITIAL_CAPITAL_INR);
    println!();
    println!("=== P.E.2 ARM ===");
    println!("  contract={}", p.execution_contract);
    println!("  lots_opened={}", p.n_lots_opened);
    println!(
        "  TARGET={} STOP={} HORIZON={} OPEN={}",
        p.n_target, p.n_stop, p.n_horizon, p.n_open_at_end
    );
    println!("  final_value={:.2}", p.final_portfolio_value_inr);
    println!("  total_return={:+.2}%", p.total_return_pct * 100.0);
    println!("  realized_pnl={:+.2}", p.total_realized_pnl_inr);
    println!("  max_drawdown={:.2}%", p.max_drawdown_pct * 100.0);
    println!("  capital_velocity={:.2}x", p.capital_velocity_ratio);
    if let Some(avg) = p.avg_holding_sessions {
        println!("  avg_hold={:.1} sessions", avg);
    }
    println!();
    println!("=== CORALYS V0 ARM ===");
    println!("  contract={}", c.execution_contract);
    println!("  lots_opened={}", c.n_lots_opened);
    println!(
        "  TARGET={} STOP={} HORIZON={} OPEN={}",
        c.n_target, c.n_stop, c.n_horizon, c.n_open_at_end
    );
    println!("  final_value={:.2}", c.final_portfolio_value_inr);
    println!("  total_return={:+.2}%", c.total_return_pct * 100.0);
    println!("  realized_pnl={:+.2}", c.total_realized_pnl_inr);
    println!("  max_drawdown={:.2}%", c.max_drawdown_pct * 100.0);
    println!("  capital_velocity={:.2}x", c.capital_velocity_ratio);
    if let Some(avg) = c.avg_holding_sessions {
        println!("  avg_hold={:.1} sessions", avg);
    }
    println!();
    println!("output={}", args.output.display());

    Ok(())
}

// ─── Report rendering ─────────────────────────────────────────────────────────

fn render_continuous_report(ledger: &ContinuousPortfolioLedger) -> String {
    let p = &ledger.pe2_summary;
    let c = &ledger.coralys_summary;
    let mut md = String::new();

    md.push_str("# Portfolio Replay v0.2.1 — Continuous Lifecycle\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Experiment:** Continuous portfolio lifecycle with position upgrades  \n");
    md.push_str("**Design:** Session-by-session loop, capital recycling, multiple lots per instrument  \n");
    md.push_str("**Does not:** modify C3-002, modify coralys-exec-v0, touch any prior archive  \n\n");

    md.push_str("## Setup\n\n");
    md.push_str(&format!("- Certified T: {}\n", ledger.certified_t));
    md.push_str(&format!("- Sessions simulated: {}\n", ledger.n_sessions_simulated));
    md.push_str(&format!("- Initial capital: Rs.{:.2}\n", ledger.initial_capital_inr));
    md.push_str(&format!("- Universe: {}\n", ledger.universe.join(", ")));
    md.push_str(&format!("- C3-002 artifact: `{}`\n", ledger.c3_002_artifact_hash));
    md.push_str(&format!("- Coralys artifact: `{}`\n\n", ledger.coralys_artifact_hash));

    md.push_str("## Capital Velocity Comparison\n\n");
    md.push_str("| Metric | P.E.2 | Coralys v0 |\n");
    md.push_str("|--------|-------|------------|\n");
    md.push_str(&format!(
        "| Capital velocity | {:.2}x | {:.2}x |\n",
        p.capital_velocity_ratio, c.capital_velocity_ratio
    ));
    md.push_str(&format!(
        "| Lots opened | {} | {} |\n",
        p.n_lots_opened, c.n_lots_opened
    ));
    md.push_str(&format!(
        "| TARGET exits | {} | {} |\n",
        p.n_target, c.n_target
    ));
    md.push_str(&format!(
        "| STOP exits | {} | {} |\n",
        p.n_stop, c.n_stop
    ));
    md.push_str(&format!(
        "| HORIZON exits | {} | {} |\n",
        p.n_horizon, c.n_horizon
    ));
    md.push_str(&format!(
        "| Open at end | {} | {} |\n",
        p.n_open_at_end, c.n_open_at_end
    ));
    if let (Some(pa), Some(ca)) = (p.avg_holding_sessions, c.avg_holding_sessions) {
        md.push_str(&format!(
            "| Avg hold (sessions) | {:.1} | {:.1} |\n",
            pa, ca
        ));
    }
    md.push_str("\n");

    md.push_str("## Portfolio Performance\n\n");
    md.push_str("| Metric | P.E.2 | Coralys v0 |\n");
    md.push_str("|--------|-------|------------|\n");
    md.push_str(&format!(
        "| Final portfolio value | Rs.{:.2} | Rs.{:.2} |\n",
        p.final_portfolio_value_inr, c.final_portfolio_value_inr
    ));
    md.push_str(&format!(
        "| Total return | {:+.2}% | {:+.2}% |\n",
        p.total_return_pct * 100.0,
        c.total_return_pct * 100.0
    ));
    md.push_str(&format!(
        "| Realized P&L | Rs.{:+.2} | Rs.{:+.2} |\n",
        p.total_realized_pnl_inr, c.total_realized_pnl_inr
    ));
    md.push_str(&format!(
        "| Unrealized P&L | Rs.{:+.2} | Rs.{:+.2} |\n",
        p.total_unrealized_pnl_inr, c.total_unrealized_pnl_inr
    ));
    md.push_str(&format!(
        "| Max drawdown | {:.2}% (Rs.{:.2}) | {:.2}% (Rs.{:.2}) |\n",
        p.max_drawdown_pct * 100.0,
        p.max_drawdown_inr,
        c.max_drawdown_pct * 100.0,
        c.max_drawdown_inr
    ));
    md.push_str("\n");

    md.push_str("## Integrity\n\n");
    md.push_str(&format!("{}\n", ledger.integrity_note));

    md
}

// ─── Argument parsing ─────────────────────────────────────────────────────────

struct ContinuousReplayArgs {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<ContinuousReplayArgs, Box<dyn std::error::Error>> {
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
        PathBuf::from("historical_runs/portfolio_continuous_v021_2026-08-16")
    });

    Ok(ContinuousReplayArgs {
        search_two,
        cache_dir,
        output,
    })
}