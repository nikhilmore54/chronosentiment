//! CS-P-006-P.E.2 live execution observation.
//!
//! Seals Execution Contract v0 on the next session after 14 August.
//! Does not mutate the 14 August cohort or the P.E.1 sidecar.
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
    C3G_EXPERIMENT_AUTHORIZED, SEARCH_THREE_AUTHORIZED, STOP_EXIT_AUTHORIZED,
    TARGET_PATH_OPTIMIZATION_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_live_execution::{
    refuse_live_execution_output, render_live_execution_html, render_live_execution_report,
    run_live_execution, CONTINUOUS_SESSION_SEAL_AUTHORIZED,
    FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED, LIVE_YAHOO_FETCH_AUTHORIZED,
    PE1_SIDECAR_MUTATION_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    refuse_live_execution_output(&args.output.to_string_lossy())?;
    if FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED
        || PE1_SIDECAR_MUTATION_AUTHORIZED
        || CONTINUOUS_SESSION_SEAL_AUTHORIZED
        || LIVE_YAHOO_FETCH_AUTHORIZED
        || TARGET_PATH_OPTIMIZATION_AUTHORIZED
        || STOP_EXIT_AUTHORIZED
        || SEARCH_THREE_AUTHORIZED
        || C3G_EXPERIMENT_AUTHORIZED
    {
        return Err(
            "refusing a live execution run that opens research or mutates protected ledgers".into(),
        );
    }

    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }
    let cache = load_required_yahoo_cache(&args.cache_dir).map_err(|e| e.to_string())?;
    let existing = if args.output.join("ledger.json").exists() {
        Some(serde_json::from_str(&fs::read_to_string(
            args.output.join("ledger.json"),
        )?)?)
    } else {
        None
    };
    let ledger = run_live_execution(&artifact, &cache, args.now, existing)?;

    fs::create_dir_all(&args.output)?;
    fs::write(
        args.output.join("ledger.json"),
        serde_json::to_vec_pretty(&ledger)?,
    )?;
    fs::write(
        args.output.join("REPORT.md"),
        render_live_execution_report(&ledger),
    )?;
    fs::write(
        args.output.join("evidence.html"),
        render_live_execution_html(&ledger),
    )?;
    fs::write(
        args.output.join("CONTRACT.txt"),
        "Execution Contract v0 — live observation\nC3-002 chooses direction only.\nThe 14-August cohort was sealed without an execution intent and remains untouched.\nP.E.2 will attach Execution Contract v0 only to the next eligible cohort at T.\nP.E.1 targeted_execution_v0 is frozen.\nAWAITING_NEXT_SESSION until a certified session after 2026-08-14T03:45:00Z.\n",
    )?;

    println!("result=PASS");
    println!("path_kind={}", ledger.path_kind);
    println!("seal_status={}", ledger.seal_status);
    println!("decisions={}", ledger.n_decisions);
    println!("observing={}", ledger.n_observing);
    println!("target={}", ledger.n_target);
    println!("horizon={}", ledger.n_horizon);
    println!(
        "fourteen_aug_cohort_mutated={}",
        ledger.fourteen_aug_cohort_mutated
    );
    println!("pe1_sidecar_mutated={}", ledger.pe1_sidecar_mutated);
    println!("output={}", args.output.display());
    Ok(())
}

struct LiveArgs {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
    now: DateTime<Utc>,
}

fn parse_args() -> Result<LiveArgs, Box<dyn std::error::Error>> {
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
    Ok(LiveArgs {
        search_two: search_two.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR)),
        cache_dir: cache
            .unwrap_or_else(|| PathBuf::from(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache")),
        output: output.unwrap_or_else(|| {
            PathBuf::from("product_validation/CS-P-006/observatory/prospective_execution_v0")
        }),
        now,
    })
}
