//! CS-P-006-P maturity tick.
//!
//! Refreshes Observatory countdown. Appends observations only after the
//! window has closed. Does not peek at returns while OBSERVING.
//! Does not retune C3-002. Does not start C.3-G.

use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_SNAPSHOT_DIR;
use chronosentiment_adapter::decision_support::csp006_snapshot::load_required_yahoo_cache;
use chronosentiment_adapter::decision_support::observatory_maturity::{
    observation_due_at_with_bars, observation_window_closed_with_bars, sessions_remaining,
    ui_lifecycle_status_with_bars, INTERMEDIATE_INTERPRETATION_AUTHORIZED,
    POLICY_RETUNE_FROM_PROSPECTIVE_AUTHORIZED,
};
use chronosentiment_adapter::decision_support::observatory_slice::{
    render_product_html, ObservatoryLedger,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (historical_dir, prospective_dir, now) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if INTERMEDIATE_INTERPRETATION_AUTHORIZED || POLICY_RETUNE_FROM_PROSPECTIVE_AUTHORIZED {
        return Err("refusing a maturity tick that authorizes interpretation or retune".into());
    }
    let historical_s = historical_dir.to_string_lossy();
    let prospective_s = prospective_dir.to_string_lossy();
    if historical_s.contains("historical_replay_v0") || prospective_s.contains("historical_replay_v0")
    {
        return Err("refusing to overwrite Replay v0".into());
    }
    if historical_s.contains("historical_replay_v1") {
        return Err("refusing to overwrite Replay v1 HTML with the combined product view".into());
    }

    let historical: ObservatoryLedger = serde_json::from_str(&fs::read_to_string(
        historical_dir.join("ledger.json"),
    )?)?;
    let prospective: ObservatoryLedger = serde_json::from_str(&fs::read_to_string(
        prospective_dir.join("ledger.json"),
    )?)?;

    let cache_dir = PathBuf::from(RESEARCH_SNAPSHOT_DIR).join("yahoo_cache");
    let cache = load_required_yahoo_cache(&cache_dir).ok();

    let mut observing = 0u32;
    let mut due = 0u32;
    let mut earliest_due = None;
    for decision in &prospective.decisions {
        let bars = cache
            .as_ref()
            .and_then(|c| c.get(&decision.instrument))
            .map(|v| v.as_slice());
        let status = ui_lifecycle_status_with_bars(
            &prospective,
            &decision.decision_id,
            now,
            bars,
        );
        let due_at = observation_due_at_with_bars(decision, bars)?;
        let remain = sessions_remaining(decision, now, bars)?;
        let closed = observation_window_closed_with_bars(decision, now, bars)?;
        if closed {
            due += 1;
        } else {
            observing += 1;
        }
        earliest_due = Some(earliest_due.map_or(due_at, |e: DateTime<Utc>| e.min(due_at)));
        println!(
            "maturity ticker={} status={} due={} remaining_sessions={} window_closed={}",
            decision.instrument,
            status,
            due_at.to_rfc3339(),
            remain,
            closed
        );
    }

    let html = render_product_html(&historical, Some(&prospective), now);
    fs::write(prospective_dir.join("observatory.html"), &html)?;
    fs::write(historical_dir.join("observatory.html"), &html)?;

    println!("result=PASS");
    println!("attached=0");
    println!("observing={observing}");
    println!("outcome_due={due}");
    println!(
        "earliest_due={}",
        earliest_due
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| "none".into())
    );
    println!("peeked_returns=false");
    println!("policy_retuned=false");
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, DateTime<Utc>), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut historical = None;
    let mut prospective = None;
    let mut now_raw = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--historical-dir" => {
                historical = Some(PathBuf::from(args.next().ok_or("missing --historical-dir")?))
            }
            "--prospective-dir" => {
                prospective =
                    Some(PathBuf::from(args.next().ok_or("missing --prospective-dir")?))
            }
            "--now" => now_raw = Some(args.next().ok_or("missing --now")?),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    let now = match now_raw {
        Some(s) => s.parse().map_err(|e| format!("--now must be RFC3339: {e}"))?,
        None => Utc::now(),
    };
    Ok((
        historical.unwrap_or_else(|| PathBuf::from("product_validation/CS-P-006/observatory")),
        prospective.unwrap_or_else(|| {
            PathBuf::from("product_validation/CS-P-006/observatory/prospective")
        }),
        now,
    ))
}
