//! CS-P-006-P.E.2.H historical time-machine of the frozen P.E.2 control.
//!
//! Does not modify the P.E.2 specification. Does not mutate 14 August, P.E.1,
//! Replay v0/v1, or live prospective_execution_v0.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR, RESEARCH_SNAPSHOT_DIR,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_execution::{
    C3G_EXPERIMENT_AUTHORIZED, SEARCH_THREE_AUTHORIZED, STOP_EXIT_AUTHORIZED,
    TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_historical_pe2::{
    historical_pe2_contract_text, refuse_historical_pe2_output, render_historical_pe2_html,
    render_historical_pe2_report, replay_historical_pe2,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    refuse_historical_pe2_output(&args.output.to_string_lossy())?;
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED
        || STOP_EXIT_AUTHORIZED
        || SEARCH_THREE_AUTHORIZED
        || C3G_EXPERIMENT_AUTHORIZED
    {
        return Err("refusing a historical P.E.2 run that opens research".into());
    }

    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }
    let cache = load_required_yahoo_cache(&args.cache_dir).map_err(|e| e.to_string())?;
    let ledger = replay_historical_pe2(&artifact, &cache)?;

    fs::create_dir_all(&args.output)?;
    fs::write(
        args.output.join("ledger.json"),
        serde_json::to_vec_pretty(&ledger)?,
    )?;
    fs::write(
        args.output.join("REPORT.md"),
        render_historical_pe2_report(&ledger),
    )?;
    fs::write(
        args.output.join("evidence.html"),
        render_historical_pe2_html(&ledger),
    )?;
    fs::write(
        args.output.join("CONTRACT.txt"),
        historical_pe2_contract_text(),
    )?;

    println!("result={}", ledger.lifecycle_validation);
    println!("path_kind={}", ledger.path_kind);
    println!("requested_clock={}", ledger.requested_clock);
    println!("certified_t={}", ledger.certified_t);
    println!("intents={}", ledger.n_decisions);
    println!("execution_intents={}", ledger.n_execution_intents);
    println!("target={}", ledger.n_target);
    println!("horizon={}", ledger.n_horizon);
    println!("gap_through={}", ledger.n_gap_through);
    println!("high_reached={}", ledger.n_high_reached);
    println!("low_reached={}", ledger.n_low_reached);
    println!("session_close={}", ledger.n_session_close);
    println!(
        "determinism={}",
        if ledger.determinism_pass {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "no_lookahead={}",
        if ledger.lookahead_clean {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "poison_test={}",
        if ledger.poison_test_pass {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "prospective_cohort_mutated={}",
        ledger.prospective_cohort_mutated
    );
    println!(
        "protected_artifacts_mutated={}",
        ledger.protected_artifacts_mutated
    );
    println!(
        "historical_pe2_lifecycle_validation={}",
        ledger.lifecycle_validation
    );
    println!("statistical_strategy_backtest=NOT_PERFORMED");
    println!("output={}", args.output.display());
    if ledger.lifecycle_validation != "PASS" {
        return Err("historical P.E.2 lifecycle validation failed".into());
    }
    Ok(())
}

struct Args {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_two = None;
    let mut cache = None;
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-two-dir" => {
                search_two = Some(PathBuf::from(
                    args.next().ok_or("missing --search-two-dir")?,
                ))
            }
            "--yahoo-cache" => {
                cache = Some(PathBuf::from(args.next().ok_or("missing --yahoo-cache")?))
            }
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            "--target" | "--target-pct" | "--now" => {
                return Err(
                    "historical P.E.2 uses the frozen 15 Jul 2026 clock and +5.0% contract".into(),
                )
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok(Args {
        search_two: search_two.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR)),
        cache_dir: cache
            .unwrap_or_else(|| PathBuf::from(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache")),
        output: output.unwrap_or_else(|| {
            PathBuf::from("product_validation/CS-P-006/observatory/historical_pe2_replay")
        }),
    })
}
