//! CS-P-006-P.E.3.H Historical replay under coralys-exec-v0.
//!
//! Runs `replay_historical_pe3()` against the frozen C3-002 artifact and
//! writes the P.E.3 historical ledger to a separate output directory.
//!
//! ## What this binary does
//!
//! 1. Loads the C3-002 artifact (identity-gated).
//! 2. Loads the Yahoo historical bar cache.
//! 3. Calls `replay_historical_pe3()`.
//! 4. Writes the P.E.3 historical ledger to the output directory.
//!
//! ## What this binary does NOT do
//!
//! - Does NOT write to any P.E.2 path.
//! - Does NOT modify the C3-002 artifact.
//! - Does NOT modify coralys-exec-v0 multipliers.
//! - Does NOT fall back to +5% for ATR-invalid positions.
//!
//! ## Usage
//!
//! ```sh
//! cargo run --bin csp006_p_replay_pe3 -- \
//!   --search-two product_validation/CS-P-006/observatory \
//!   --cache-dir product_validation/CS-P-006/yahoo_cache \
//!   --output historical_runs/pe3_coralys_v0_2026-08-16/execution_ledger
//! ```
//!
//! ## Output
//!
//! ```
//! historical_runs/pe3_coralys_v0_2026-08-16/execution_ledger/
//!   pe3_historical_ledger.json   — full P.E.3 historical ledger
//!   pe3_execution_report.json    — summary report
//!   pe3_REPORT.md                — human-readable report
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::coralys_execution_model::CORALYS_EXEC_ARTIFACT_HASH;
use chronosentiment_adapter::decision_support::observatory_historical_pe3::{
    refuse_historical_pe3_output, replay_historical_pe3, HISTORICAL_PE3_PATH_KIND,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── Environment guard ─────────────────────────────────────────────────────
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    // ── Output path guard ─────────────────────────────────────────────────────
    refuse_historical_pe3_output(&args.output.to_string_lossy())?;

    // ── Load C3-002 artifact ──────────────────────────────────────────────────
    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }

    // ── Coralys artifact hash guard ───────────────────────────────────────────
    // This is also checked inside replay_historical_pe3(), but we surface it early.
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

    // ── Run P.E.3 historical replay ───────────────────────────────────────────
    let ledger = replay_historical_pe3(&artifact, &cache)?;

    // ── Validate output ───────────────────────────────────────────────────────
    if ledger.path_kind != HISTORICAL_PE3_PATH_KIND {
        return Err(format!(
            "unexpected path_kind: expected {HISTORICAL_PE3_PATH_KIND}, got {}",
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
    if ledger.execution_contract == "targeted_execution_v0_fixed_5pct_20_sessions" {
        return Err("P.E.3 ledger carries P.E.2 execution_contract — contamination detected".into());
    }

    // ── Write output ──────────────────────────────────────────────────────────
    fs::create_dir_all(&args.output)?;

    let ledger_path = args.output.join("pe3_historical_ledger.json");
    refuse_historical_pe3_output(&ledger_path.to_string_lossy())?;
    fs::write(&ledger_path, serde_json::to_vec_pretty(&ledger)?)?;

    // Summary report (JSON)
    let report = serde_json::json!({
        "path_kind": ledger.path_kind,
        "execution_contract": ledger.execution_contract,
        "execution_contract_label": ledger.execution_contract_label,
        "coralys_model_id": ledger.coralys_model_id,
        "coralys_model_version": ledger.coralys_model_version,
        "coralys_artifact_hash": ledger.coralys_artifact_hash,
        "requested_clock": ledger.requested_clock,
        "certified_t": ledger.certified_t,
        "max_holding_sessions": ledger.max_holding_sessions,
        "n_decisions": ledger.n_decisions,
        "n_pe3_eligible": ledger.n_pe3_eligible,
        "n_excluded_no_atr": ledger.n_excluded_no_atr,
        "n_target": ledger.n_target,
        "n_risk": ledger.n_risk,
        "n_horizon": ledger.n_horizon,
        "n_no_trade": ledger.n_no_trade,
        "n_ambiguous": ledger.n_ambiguous,
        "determinism_pass": ledger.determinism_pass,
        "lookahead_clean": ledger.lookahead_clean,
        "poison_test_pass": ledger.poison_test_pass,
        "peeked_returns_at_seal": ledger.peeked_returns_at_seal,
        "statistical_backtest": ledger.statistical_backtest,
        "retrospective_characterization": ledger.retrospective_characterization,
        "lifecycle_validation": ledger.lifecycle_validation,
    });
    fs::write(
        args.output.join("pe3_execution_report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;

    // Human-readable report (Markdown)
    let md = render_pe3_report(&ledger);
    fs::write(args.output.join("pe3_REPORT.md"), md)?;

    // ── Print summary ─────────────────────────────────────────────────────────
    println!("result=PASS");
    println!("path_kind={}", ledger.path_kind);
    println!("execution_contract={}", ledger.execution_contract);
    println!("coralys_artifact_hash={}", ledger.coralys_artifact_hash);
    println!("certified_t={}", ledger.certified_t);
    println!("n_decisions={}", ledger.n_decisions);
    println!("n_pe3_eligible={}", ledger.n_pe3_eligible);
    println!("n_excluded_no_atr={}", ledger.n_excluded_no_atr);
    println!("n_target={}", ledger.n_target);
    println!("n_risk={}", ledger.n_risk);
    println!("n_horizon={}", ledger.n_horizon);
    println!("n_no_trade={}", ledger.n_no_trade);
    println!("n_ambiguous={}", ledger.n_ambiguous);
    println!("determinism={}", ledger.determinism_pass);
    println!("lookahead_clean={}", ledger.lookahead_clean);
    println!("poison_test_pass={}", ledger.poison_test_pass);
    println!("retrospective_characterization={}", ledger.retrospective_characterization);
    println!("output={}", args.output.display());

    Ok(())
}

// ─── Report rendering ─────────────────────────────────────────────────────────

fn render_pe3_report(
    ledger: &chronosentiment_adapter::decision_support::observatory_historical_pe3::HistoricalPe3Ledger,
) -> String {
    let mut md = String::new();
    md.push_str("# P.E.3 Historical Replay Report — coralys-exec-v0\n\n");
    md.push_str("**Document type:** Product validation evidence  \n");
    md.push_str("**Parent:** CS-P-006-P.E.3.H  \n");
    md.push_str("**Execution model:** coralys-exec-v0 (ATR-anchored, TMV-scaled)  \n");
    md.push_str("**Does not:** modify C3-002, modify coralys-exec-v0 multipliers, fall back to +5%, touch P.E.2 ledger  \n\n");
    md.push_str(&format!(
        "- coralys artifact hash: `{}`\n",
        ledger.coralys_artifact_hash
    ));
    md.push_str(&format!("- path kind: `{}`\n", ledger.path_kind));
    md.push_str(&format!("- execution contract: `{}`\n", ledger.execution_contract));
    md.push_str(&format!("- certified T: {}\n", ledger.certified_t));
    md.push_str(&format!("- requested clock: {}\n", ledger.requested_clock));
    md.push_str(&format!("- peeked_returns_at_seal: {}\n", ledger.peeked_returns_at_seal));
    md.push_str(&format!(
        "- statistical backtest: {}\n\n",
        if ledger.statistical_backtest { "DONE" } else { "not done" }
    ));
    md.push_str(&format!("- decisions: {}\n", ledger.n_decisions));
    md.push_str(&format!("- P.E.3 eligible (ATR available): {}\n", ledger.n_pe3_eligible));
    md.push_str(&format!("- excluded (ATR unavailable): {}\n", ledger.n_excluded_no_atr));
    md.push_str(&format!("- NO_TRADE: {}\n", ledger.n_no_trade));
    md.push_str(&format!("- TARGET: {}\n", ledger.n_target));
    md.push_str(&format!("- RISK (stop): {}\n", ledger.n_risk));
    md.push_str(&format!("- HORIZON: {}\n", ledger.n_horizon));
    md.push_str(&format!("- AMBIGUOUS: {}\n\n", ledger.n_ambiguous));
    md.push_str(&format!("- determinism: {}\n", ledger.determinism_pass));
    md.push_str(&format!("- lookahead_clean: {}\n", ledger.lookahead_clean));
    md.push_str(&format!("- poison_test_pass: {}\n\n", ledger.poison_test_pass));
    md.push_str(&format!(
        "- retrospective_characterization: {}\n",
        ledger.retrospective_characterization
    ));
    md.push_str("\n## Lifecycle validation\n\n");
    md.push_str(&ledger.lifecycle_validation);
    md.push('\n');
    md
}

// ─── Argument parsing ─────────────────────────────────────────────────────────

struct ReplayPe3Args {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<ReplayPe3Args, Box<dyn std::error::Error>> {
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
        PathBuf::from("product_validation/CS-P-006/observatory")
    });
    let cache_dir = cache_dir.unwrap_or_else(|| {
        PathBuf::from("product_validation/CS-P-006/yahoo_cache")
    });
    let output = output.unwrap_or_else(|| {
        PathBuf::from("historical_runs/pe3_coralys_v0_2026-08-16/execution_ledger")
    });

    Ok(ReplayPe3Args {
        search_two,
        cache_dir,
        output,
    })
}