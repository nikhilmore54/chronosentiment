//! Portfolio Historical Replay v0.2 — Horizon Matrix.
//!
//! Runs the same two-arm portfolio comparison (P.E.2 vs Coralys v0) across
//! four evaluation horizons: 5, 10, 15, and 20 sessions.
//!
//! Purpose: characterize how Coralys v0 positions behave across different
//! observation horizons, with particular attention to adverse/favorable
//! excursions and unresolved positions, to provide empirical evidence for
//! future stop-loss and exit strategy design.
//!
//! Execution contracts are FROZEN — identical to v0.1:
//!   - P.E.2 arm: fixed +5% target, no stop, 20-session execution max
//!   - Coralys v0 arm: ATR/TMV target + enforced risk_boundary stop, 20-session max
//!
//! ## Usage
//!
//! ```sh
//! cargo run --bin csp008_portfolio_replay_v02 -- \
//!   --search-two product_validation/CS-P-006/discovery/20260815T051900Z_c3 \
//!   --cache-dir product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/yahoo_cache \
//!   --output historical_runs/portfolio_replay_v02_horizon_matrix_2026-08-16
//! ```
//!
//! ## Output
//!
//! ```
//! historical_runs/portfolio_replay_v02_horizon_matrix_2026-08-16/
//!   horizon_5s/
//!     portfolio_replay_ledger.json
//!     portfolio_comparison.json
//!     portfolio_REPORT.md
//!     metadata.json
//!   horizon_10s/
//!     ...
//!   horizon_15s/
//!     ...
//!   horizon_20s/
//!     ...
//!   matrix_summary.json
//!   matrix_REPORT.md
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::coralys_execution_model::CORALYS_EXEC_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::portfolio_replay_v0::{
    run_portfolio_replay, ArmSummary, PortfolioReplayConfig, INITIAL_CAPITAL_INR,
    PORTFOLIO_REPLAY_REQUESTED_CLOCK,
};

// ─── Horizon matrix ───────────────────────────────────────────────────────────

const EVALUATION_HORIZONS: &[u32] = &[5, 10, 15, 20];
const EXPERIMENT_VERSION: &str = "portfolio_replay_v02_horizon_matrix";

// ─── Output guard ─────────────────────────────────────────────────────────────

fn refuse_v02_output(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let forbidden = [
        "pe2_control",
        "pe3_coralys",
        "portfolio_comparison_pe2_vs_pe3",
    ];
    for f in &forbidden {
        if path.contains(f) {
            return Err(format!(
                "v0.2 binary refuses to write to immutable v0.1 archive path: {path}"
            )
            .into());
        }
    }
    Ok(())
}

// ─── Matrix row ───────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
struct MatrixRow {
    evaluation_horizon_sessions: u32,
    pe2: ArmSummary,
    coralys: ArmSummary,
}

// ─── Args ─────────────────────────────────────────────────────────────────────

struct Args {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut search_two = None;
    let mut cache_dir = None;
    let mut output = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--search-two" => {
                i += 1;
                search_two = Some(PathBuf::from(&args[i]));
            }
            "--cache-dir" => {
                i += 1;
                cache_dir = Some(PathBuf::from(&args[i]));
            }
            "--output" => {
                i += 1;
                output = Some(PathBuf::from(&args[i]));
            }
            _ => {}
        }
        i += 1;
    }
    Ok(Args {
        search_two: search_two.ok_or("--search-two required")?,
        cache_dir: cache_dir.ok_or("--cache-dir required")?,
        output: output.ok_or("--output required")?,
    })
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── Environment guard ─────────────────────────────────────────────────────
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    // ── Output path guard ─────────────────────────────────────────────────────
    refuse_v02_output(&args.output.to_string_lossy())?;

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
        return Err(format!("coralys artifact hash mismatch: {CORALYS_EXEC_ARTIFACT_HASH}").into());
    }

    // ── Load Yahoo bar cache ──────────────────────────────────────────────────
    let cache = load_required_yahoo_cache(&args.cache_dir).map_err(|e| e.to_string())?;

    // ── Create output root ────────────────────────────────────────────────────
    fs::create_dir_all(&args.output)?;

    // ── Run horizon matrix ────────────────────────────────────────────────────
    let mut matrix: Vec<MatrixRow> = Vec::new();

    for &horizon in EVALUATION_HORIZONS {
        let label = format!("v0_2_horizon_{horizon}s");
        let config = PortfolioReplayConfig {
            universe: RESEARCH_UNIVERSE.iter().map(|s| s.to_string()).collect(),
            start_clock: PORTFOLIO_REPLAY_REQUESTED_CLOCK.to_string(),
            end_clock: None,
            initial_capital_inr: INITIAL_CAPITAL_INR,
            evaluation_horizon_sessions: horizon,
            experiment_label: label.clone(),
        };

        let ledger = run_portfolio_replay(&artifact, &cache, &config)
            .map_err(|e| format!("horizon {horizon}s: {e}"))?;

        // Write per-horizon archive
        let horizon_dir = args.output.join(format!("horizon_{horizon}s"));
        fs::create_dir_all(&horizon_dir)?;

        fs::write(
            horizon_dir.join("portfolio_replay_ledger.json"),
            serde_json::to_string_pretty(&ledger)?,
        )?;
        fs::write(
            horizon_dir.join("portfolio_comparison.json"),
            serde_json::to_string_pretty(&ledger.comparison)?,
        )?;

        let report = build_horizon_report(&ledger, horizon);
        fs::write(horizon_dir.join("portfolio_REPORT.md"), &report)?;

        let metadata = serde_json::json!({
            "experiment_version": EXPERIMENT_VERSION,
            "evaluation_horizon_sessions": horizon,
            "experiment_label": label,
            "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
            "coralys_exec_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
            "initial_capital_inr": INITIAL_CAPITAL_INR,
            "start_clock": PORTFOLIO_REPLAY_REQUESTED_CLOCK,
            "universe": RESEARCH_UNIVERSE,
            "status": "IMMUTABLE",
        });
        fs::write(
            horizon_dir.join("metadata.json"),
            serde_json::to_string_pretty(&metadata)?,
        )?;

        println!(
            "horizon={horizon}s  P.E.2={:.2}%  Coralys={:.2}%  TARGET_pe2={}  TARGET_coralys={}  STOP_coralys={}",
            ledger.comparison.pe2_arm.total_return_pct * 100.0,
            ledger.comparison.coralys_arm.total_return_pct * 100.0,
            ledger.comparison.pe2_arm.n_target,
            ledger.comparison.coralys_arm.n_target,
            ledger.comparison.coralys_arm.n_stop,
        );

        matrix.push(MatrixRow {
            evaluation_horizon_sessions: horizon,
            pe2: ledger.comparison.pe2_arm.clone(),
            coralys: ledger.comparison.coralys_arm.clone(),
        });
    }

    // ── Write matrix summary ──────────────────────────────────────────────────
    fs::write(
        args.output.join("matrix_summary.json"),
        serde_json::to_string_pretty(&matrix)?,
    )?;

    let matrix_report = build_matrix_report(&matrix);
    fs::write(args.output.join("matrix_REPORT.md"), &matrix_report)?;

    let root_metadata = serde_json::json!({
        "experiment_version": EXPERIMENT_VERSION,
        "evaluation_horizons": EVALUATION_HORIZONS,
        "c3_002_artifact_hash": RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH,
        "coralys_exec_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
        "initial_capital_inr": INITIAL_CAPITAL_INR,
        "start_clock": PORTFOLIO_REPLAY_REQUESTED_CLOCK,
        "universe": RESEARCH_UNIVERSE,
        "status": "IMMUTABLE",
        "purpose": "Characterize Coralys v0 position behaviour across evaluation horizons. Observe frozen strategy. Do not optimize.",
    });
    fs::write(
        args.output.join("metadata.json"),
        serde_json::to_string_pretty(&root_metadata)?,
    )?;

    println!("\nresult=PASS");
    println!("output={}", args.output.display());

    Ok(())
}

// ─── Report builders ──────────────────────────────────────────────────────────

fn build_horizon_report(
    ledger: &chronosentiment_adapter::decision_support::portfolio_replay_v0::PortfolioReplayLedger,
    horizon: u32,
) -> String {
    let c = &ledger.comparison;
    format!(
        "# Portfolio Replay v0.2 — Horizon {horizon} Sessions\n\n\
         Evaluation horizon: {horizon} sessions\n\
         Execution contracts: FROZEN (P.E.2 = fixed +5% no stop; Coralys v0 = ATR/TMV + enforced stop)\n\n\
         ## P.E.2 Arm\n\n\
         - Return: {:.2}%\n\
         - TARGET: {} | STOP: {} | HORIZON: {}\n\
         - Avg hold: {:.1} sessions\n\
         - Max drawdown: {:.2}%\n\
         - Capital utilization: {:.1}%\n\n\
         ## Coralys v0 Arm\n\n\
         - Return: {:.2}%\n\
         - TARGET: {} | STOP: {} | HORIZON: {}\n\
         - Avg hold: {:.1} sessions\n\
         - Max drawdown: {:.2}%\n\
         - Capital utilization: {:.1}%\n",
        c.pe2_arm.total_return_pct * 100.0,
        c.pe2_arm.n_target,
        c.pe2_arm.n_stop,
        c.pe2_arm.n_horizon,
        c.pe2_arm.avg_holding_sessions.unwrap_or(0.0),
        c.pe2_arm.max_drawdown_pct * 100.0,
        c.pe2_arm.capital_utilization_pct * 100.0,
        c.coralys_arm.total_return_pct * 100.0,
        c.coralys_arm.n_target,
        c.coralys_arm.n_stop,
        c.coralys_arm.n_horizon,
        c.coralys_arm.avg_holding_sessions.unwrap_or(0.0),
        c.coralys_arm.max_drawdown_pct * 100.0,
        c.coralys_arm.capital_utilization_pct * 100.0,
    )
}

fn build_matrix_report(matrix: &[MatrixRow]) -> String {
    let mut out = String::from(
        "# Portfolio Replay v0.2 — Horizon Matrix\n\n\
         Execution contracts: FROZEN throughout.\n\
         Purpose: characterize Coralys v0 position behaviour across evaluation horizons.\n\
         Do not optimize based on these results.\n\n\
         ## Return Matrix\n\n\
         | Horizon | P.E.2 Return | Coralys Return | P.E.2 TARGET | Coralys TARGET | Coralys STOP | Coralys HORIZON | P.E.2 MaxDD | Coralys MaxDD | P.E.2 AvgHold | Coralys AvgHold |\n\
         |---------|-------------|----------------|-------------|----------------|-------------|-----------------|------------|--------------|--------------|----------------|\n",
    );
    for row in matrix {
        out.push_str(&format!(
            "| {}s | {:.2}% | {:.2}% | {} | {} | {} | {} | {:.2}% | {:.2}% | {:.1}s | {:.1}s |\n",
            row.evaluation_horizon_sessions,
            row.pe2.total_return_pct * 100.0,
            row.coralys.total_return_pct * 100.0,
            row.pe2.n_target,
            row.coralys.n_target,
            row.coralys.n_stop,
            row.coralys.n_horizon,
            row.pe2.max_drawdown_pct * 100.0,
            row.coralys.max_drawdown_pct * 100.0,
            row.pe2.avg_holding_sessions.unwrap_or(0.0),
            row.coralys.avg_holding_sessions.unwrap_or(0.0),
        ));
    }
    out.push_str(
        "\n## Notes\n\n\
         - All results are exploratory (n=7 instruments, single historical window).\n\
         - Execution horizon (20 sessions max hold) is unchanged across all rows.\n\
         - Evaluation horizon controls how many sessions are observed for exit scanning.\n\
         - HORIZON count = positions that reached the evaluation horizon without TARGET or STOP.\n\
         - Coralys stop is enforced in all rows (stop_authorized=true).\n",
    );
    out
}
