//! CS-P-006-P.E Targeted Decision Execution replay.
//!
//! Seals a +5% target at T and records the first OHLC exit.
//! Does not mutate Replay v0/v1 or the 14 August cohort.
//! Does not start C.3-G or Search #3.

use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR, RESEARCH_SNAPSHOT_DIR,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_execution::{
    default_execution_clocks, refuse_protected_output, render_execution_html,
    render_execution_report, replay_targeted_execution, C3G_EXPERIMENT_AUTHORIZED,
    SEARCH_THREE_AUTHORIZED, STOP_EXIT_AUTHORIZED, TARGETED_EXECUTION_V0_FROZEN,
    TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    refuse_protected_output(&args.output.to_string_lossy())?;
    if TARGETED_EXECUTION_V0_FROZEN
        && args
            .output
            .to_string_lossy()
            .contains("targeted_execution_v0")
    {
        return Err("P.E.1 sidecar targeted_execution_v0 is frozen".into());
    }
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED
        || STOP_EXIT_AUTHORIZED
        || SEARCH_THREE_AUTHORIZED
        || C3G_EXPERIMENT_AUTHORIZED
    {
        return Err(
            "refusing an execution run that opens research or path-optimizes the target".into(),
        );
    }

    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }
    let cache = load_required_yahoo_cache(&args.cache_dir).map_err(|e| e.to_string())?;
    let (intents, report) = replay_targeted_execution(&artifact, &cache, &args.clocks, args.now)?;

    fs::create_dir_all(&args.output)?;
    fs::write(
        args.output.join("intents.json"),
        serde_json::to_vec_pretty(&intents)?,
    )?;
    fs::write(
        args.output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(
        args.output.join("REPORT.md"),
        render_execution_report(&report),
    )?;
    fs::write(
        args.output.join("evidence.html"),
        render_execution_html(&report),
    )?;
    fs::write(
        args.output.join("CONTRACT.txt"),
        "Execution Contract v0\nC3-002 chooses direction only.\ntarget_pct = 5.0% belongs to this contract, not to C3-002.\nTarget sealed at T. Replay v0/v1 and the 14 August cohort are not rewritten.\n",
    )?;

    println!("result=PASS");
    println!("path_kind={}", report.path_kind);
    println!("contract={}", report.execution_contract);
    println!("sealed_intents={}", intents.len());
    println!("exits={}", report.n_exits);
    println!("target={}", report.n_target);
    println!("horizon={}", report.n_horizon);
    println!("peeked_returns_at_seal={}", report.peeked_returns_at_seal);
    println!(
        "prospective_cohort_mutated={}",
        report.prospective_cohort_mutated
    );
    println!("statistical_backtest={}", report.statistical_backtest);
    println!("output={}", args.output.display());
    Ok(())
}

struct ExecArgs {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
    now: DateTime<Utc>,
    clocks: Vec<DateTime<Utc>>,
}

fn parse_args() -> Result<ExecArgs, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_two = None;
    let mut cache = None;
    let mut output = None;
    let mut now_raw = None;
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
            "--now" => now_raw = Some(args.next().ok_or("missing --now")?),
            "--target" | "--target-pct" => {
                return Err("target_pct is sealed on execution contract v0 at +5.0%".into())
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    let now = match now_raw {
        Some(s) => s
            .parse()
            .map_err(|e| format!("--now must be RFC3339: {e}"))?,
        None => Utc::now(),
    };
    Ok(ExecArgs {
        search_two: search_two.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR)),
        cache_dir: cache
            .unwrap_or_else(|| PathBuf::from(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache")),
        output: output.unwrap_or_else(|| {
            PathBuf::from("product_validation/CS-P-006/observatory/targeted_execution_v0")
        }),
        now,
        clocks: default_execution_clocks()?,
    })
}
