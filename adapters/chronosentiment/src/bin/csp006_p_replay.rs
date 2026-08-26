//! CS-P-006-P.H / P.H.1 Historical Decision Replay.
//!
//! Production Observatory against a historical clock. Same C3-002.
//! Horizon is 20 market sessions. Does not mutate the 14 August cohort.
//! Does not start C.3-G.

use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR, RESEARCH_SNAPSHOT_DIR,
    RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_historical::{
    parse_replay_clocks, refuse_prospective_output, render_replay_html, render_replay_report,
    replay_selected, DEFAULT_REPLAY_CLOCKS, LOOKAHEAD_BACKTEST_AUTHORIZED, PEEKED_RETURNS_AT_SEAL,
    PROSPECTIVE_COHORT_MUTATION_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_maturity::TRADING_SESSION_HORIZON_AUTHORIZED;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    refuse_prospective_output(&args.output.to_string_lossy())?;
    if args.output.ends_with("selected_policy.json") {
        return Err("refusing to overwrite selected_policy.json".into());
    }
    if LOOKAHEAD_BACKTEST_AUTHORIZED
        || PEEKED_RETURNS_AT_SEAL
        || PROSPECTIVE_COHORT_MUTATION_AUTHORIZED
    {
        return Err(
            "refusing a replay that authorizes lookahead, peeking, or prospective mutation".into(),
        );
    }
    if !TRADING_SESSION_HORIZON_AUTHORIZED {
        return Err("Replay v1 requires the 20 market-session Observatory contract".into());
    }

    let artifact: PolicyArtifact = serde_json::from_str(&fs::read_to_string(
        args.search_two.join("selected_policy.json"),
    )?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }
    let cache = load_required_yahoo_cache(&args.cache_dir).map_err(|e| e.to_string())?;
    let (ledger, report) =
        replay_selected(&artifact, &cache, &args.clocks, &args.instruments, args.now)?;

    fs::create_dir_all(&args.output)?;
    fs::write(
        args.output.join("ledger.json"),
        serde_json::to_vec_pretty(&ledger)?,
    )?;
    fs::write(args.output.join("REPORT.md"), render_replay_report(&report))?;
    fs::write(
        args.output.join("observatory.html"),
        render_replay_html(&ledger, &report, args.now),
    )?;
    fs::write(
        args.output.join("report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;

    println!("result=PASS");
    println!("path_kind={}", ledger.path_kind);
    println!("sealed={}", report.n_decisions);
    println!("observed={}", report.n_observed);
    println!(
        "horizon={} {}",
        report.horizon_duration_days, report.horizon_calendar_basis
    );
    println!("peeked_returns={}", report.peeked_returns);
    println!("determinism={}", report.determinism_pass);
    println!("lookahead_clean={}", report.lookahead_clean);
    println!("statistical_backtest={}", report.statistical_backtest);
    println!(
        "prospective_cohort_mutated={}",
        report.prospective_cohort_mutated
    );
    println!("search_three_authorized={}", ledger.search_three_authorized);
    println!("output={}", args.output.display());
    Ok(())
}

struct ReplayArgs {
    search_two: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
    now: DateTime<Utc>,
    clocks: Vec<DateTime<Utc>>,
    instruments: Vec<&'static str>,
}

fn parse_args() -> Result<ReplayArgs, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_two = None;
    let mut cache = None;
    let mut output = None;
    let mut now_raw = None;
    let mut clocks_raw = Vec::new();
    let mut instruments_raw = Vec::new();
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
            "--clock" => clocks_raw.push(args.next().ok_or("missing --clock")?),
            "--instrument" => instruments_raw.push(args.next().ok_or("missing --instrument")?),
            "--horizon" | "--horizon-days" => {
                return Err("horizon is frozen at 20 market sessions".into())
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
    let clocks = if clocks_raw.is_empty() {
        parse_replay_clocks(&DEFAULT_REPLAY_CLOCKS)?
    } else {
        let refs: Vec<&str> = clocks_raw.iter().map(String::as_str).collect();
        parse_replay_clocks(&refs)?
    };
    let instruments = if instruments_raw.is_empty() {
        RESEARCH_UNIVERSE.to_vec()
    } else {
        let mut chosen = Vec::new();
        for raw in &instruments_raw {
            let ticker = RESEARCH_UNIVERSE
                .iter()
                .copied()
                .find(|known| *known == raw.as_str())
                .ok_or_else(|| format!("{raw} is outside the seven-name paper universe"))?;
            chosen.push(ticker);
        }
        chosen
    };
    Ok(ReplayArgs {
        search_two: search_two.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR)),
        cache_dir: cache
            .unwrap_or_else(|| PathBuf::from(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache")),
        output: output.unwrap_or_else(|| {
            PathBuf::from("product_validation/CS-P-006/observatory/historical_replay_v1")
        }),
        now,
        clocks,
        instruments,
    })
}
