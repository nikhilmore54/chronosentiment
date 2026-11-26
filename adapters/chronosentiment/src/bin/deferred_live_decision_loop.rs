//! Decision loop operator — synthetic scenarios, a cached session tape, or a live JSONL inbox.
//!
//! ```text
//! # Cached replay:
//! cargo run -p chronosentiment_adapter --bin deferred_live_decision_loop -- \
//!   --session 2026-09-07 --dataset datasets/live005_20260907.json
//!
//! # Live polling inbox (Python sidecar writes observations; Rust polls every 30s):
//! cargo run -p chronosentiment_adapter --bin deferred_live_decision_loop -- \
//!   --session 2026-09-22 --dataset datasets/live005_20260922.json \
//!   --inbox /tmp/live_obs.jsonl \
//!   --ledger live_capture/ledger/asof_prospective_20260922.jsonl
//! ```
//!
//! Inbox mode: polls the JSONL file every 30 s, ingests new lines, and terminates
//! when the inbox has had no new observations for STALE_TIMEOUT_SECS (default 5 min
//! after 15:30 IST or 10 min absolute).
//! On termination the final report is written to stdout AND appended as a JSONL
//! record (with session metadata) to `--ledger`.

use std::io::{BufRead, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chronosentiment_adapter::product::{
    lifecycle_scenario_tape, load_intraday_briefs, run_cached_session, AsOfSessionDriver,
    DecisionLoopRuntime, DeferredLiveConfig, LifecycleScenario, MarketObservation,
};
use chronosentiment_adapter::product::intraday_decision::{DecisionBrief, ExecutionFacts};
use chronosentiment_adapter::product::live_observation::parse_observation_line;

/// Stop polling if no new observations arrive for this long after market close.
const POST_CLOSE_STALE_SECS: u64 = 300; // 5 minutes
/// Absolute maximum stale time regardless of market status.
const MAX_STALE_SECS: u64 = 600; // 10 minutes
/// Inbox poll interval.
const POLL_INTERVAL_SECS: u64 = 30;
/// NSE market close: 15:30 IST = 10:00 UTC.
const MARKET_CLOSE_UTC_HOUR: u32 = 10;
const MARKET_CLOSE_UTC_MIN: u32 = 0;

const TICKER: &str = "JUBLFOOD_NS";
const SNAP: i64 = 1_000_000;
const FILL: f64 = 480.5;

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if let Some(date) = arg_value(&raw, "--session") {
        if let Err(e) = run_session(&date, &raw) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    let scenario = arg_value(&raw, "--scenario").unwrap_or_else(|| "tape-exhausted".into());
    match scenario.as_str() {
        "target" => play_lifecycle(LifecycleScenario::Target),
        "stop" => play_lifecycle(LifecycleScenario::Stop),
        "horizon" => play_lifecycle(LifecycleScenario::Horizon),
        "before-snap" => play_steps(&[obs(SNAP - 60, FILL)]),
        "at-snap" => play_steps(&[obs(SNAP, FILL)]),
        "after-snap" => play_steps(&[obs(SNAP + 60, FILL)]),
        "same-unix" => play_steps(&[
            obs(SNAP, FILL),
            obs(SNAP + 60, 479.0),
            MarketObservation {
                ticker: TICKER.into(),
                unix: SNAP + 60,
                price: 478.0,
                high: Some(478.5),
                low: Some(477.5),
            },
        ]),
        _ => play_steps(&[obs(SNAP, FILL)]),
    }
}

fn run_session(date: &str, raw: &[String]) -> Result<(), String> {
    let dataset = arg_value(raw, "--dataset").unwrap_or_else(|| {
        std::env::var("INTRADAY_DATASET_PATH")
            .unwrap_or_else(|_| "datasets/p4_opportunity_dataset.json".into())
    });
    let ledger = arg_value(raw, "--ledger");
    let briefs = load_intraday_briefs(&dataset)?;
    if let Some(inbox) = arg_value(raw, "--inbox") {
        return run_live_inbox(&briefs, date, &inbox, ledger.as_deref());
    }
    let cache = PathBuf::from(arg_value(raw, "--cache-dir").unwrap_or_else(|| {
        "intraday_capture/yahoo_cache_1m".into()
    }));
    let report = run_cached_session(&briefs, date, &cache, 0.0)?;
    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    println!("{json}");
    if let Some(ledger_path) = ledger.as_deref() {
        append_to_ledger(ledger_path, date, &json)?;
    }
    Ok(())
}

/// Poll-loop inbox consumer.
///
/// Reads new lines from `inbox` every `POLL_INTERVAL_SECS` seconds.
/// Terminates when no new observations arrive for `POST_CLOSE_STALE_SECS` after
/// market close or `MAX_STALE_SECS` in absolute terms.
/// Final report is written to stdout and optionally appended to `ledger_path`.
fn run_live_inbox(
    briefs: &[DecisionBrief],
    date: &str,
    inbox: &str,
    ledger_path: Option<&str>,
) -> Result<(), String> {
    let frozen_armed_ids: Vec<String> =
        chronosentiment_adapter::product::select_session_briefs(briefs, date)
            .into_iter()
            .map(|b| b.id)
            .collect();
    let mut driver = AsOfSessionDriver::new(DeferredLiveConfig::default());
    driver.install_session(briefs, date);

    // Track inbox byte offset for incremental reads.
    let mut inbox_offset: u64 = 0;
    let mut last_new_obs = Instant::now();
    let mut total_ingested: usize = 0;
    let mut round: usize = 0;

    eprintln!("[inbox] Polling {inbox} every {POLL_INTERVAL_SECS}s  session={date}");
    eprintln!("[inbox] T0 population: {} briefs", briefs.len());

    loop {
        round += 1;

        // Read new lines since last offset.
        let new_count = poll_inbox_file(inbox, &mut inbox_offset, &mut driver)?;
        if new_count > 0 {
            total_ingested += new_count;
            last_new_obs = Instant::now();
            eprintln!("[inbox] round={round} +{new_count} obs  total={total_ingested}");
        } else {
            eprintln!("[inbox] round={round} no new obs  total={total_ingested}");
        }

        // Termination: stale for too long.
        let stale = last_new_obs.elapsed();
        let post_close = is_past_market_close();
        let stale_limit = if post_close {
            Duration::from_secs(POST_CLOSE_STALE_SECS)
        } else {
            Duration::from_secs(MAX_STALE_SECS)
        };

        if stale >= stale_limit {
            eprintln!(
                "[inbox] Stale for {:.0}s (limit={:.0}s, post_close={post_close}). Finalising.",
                stale.as_secs_f64(),
                stale_limit.as_secs_f64(),
            );
            break;
        }

        std::thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }

    driver.mark_tape_exhausted();
    let report = driver.snapshot_report(date, frozen_armed_ids);
    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    println!("{json}");
    eprintln!("[inbox] Report written to stdout  positions={}", report.positions.len());

    if let Some(lp) = ledger_path {
        append_to_ledger(lp, date, &json)?;
        eprintln!("[inbox] Appended to ledger: {lp}");
    }
    Ok(())
}

/// Read new lines from `inbox` starting at `offset`, ingest each as a MarketObservation.
/// Updates `offset` in place. Returns count of valid observations ingested.
fn poll_inbox_file(
    inbox: &str,
    offset: &mut u64,
    driver: &mut AsOfSessionDriver,
) -> Result<usize, String> {
    if !std::path::Path::new(inbox).exists() {
        return Ok(0);
    }
    let mut file = std::fs::File::open(inbox)
        .map_err(|e| format!("open inbox {inbox}: {e}"))?;
    let meta = file.metadata().map_err(|e| format!("stat inbox: {e}"))?;
    if meta.len() < *offset {
        // File was truncated/rotated — reset.
        *offset = 0;
    }
    if meta.len() == *offset {
        return Ok(0); // No new bytes.
    }
    file.seek(SeekFrom::Start(*offset))
        .map_err(|e| format!("seek inbox: {e}"))?;
    let reader = std::io::BufReader::new(&file);
    let mut count = 0;
    let mut bytes_consumed: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(|e| format!("read inbox line: {e}"))?;
        let bytes = line.len() as u64 + 1; // +1 for newline
        bytes_consumed += bytes;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(obs) = parse_observation_line(line) {
            driver.ingest(obs);
            count += 1;
        }
    }
    *offset += bytes_consumed;
    Ok(count)
}

/// True if current UTC time is past NSE market close (10:00 UTC = 15:30 IST).
fn is_past_market_close() -> bool {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs_of_day = secs % 86400;
    let close_secs = (MARKET_CLOSE_UTC_HOUR as u64) * 3600 + (MARKET_CLOSE_UTC_MIN as u64) * 60;
    secs_of_day >= close_secs
}

/// Append final run report as a JSONL record to `ledger_path`.
/// Creates parent directories and the file if needed.
fn append_to_ledger(ledger_path: &str, date: &str, report_json: &str) -> Result<(), String> {
    use std::io::Write;
    if let Some(parent) = std::path::Path::new(ledger_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create ledger dir: {e}"))?;
    }
    // Parse inner report to wrap with metadata.
    let inner: serde_json::Value = serde_json::from_str(report_json)
        .map_err(|e| format!("parse report for ledger: {e}"))?;
    let record = serde_json::json!({
        "schema": "asof_prospective_run_v1",
        "session_date": date,
        "generated_at": chrono_now_iso(),
        "run": inner,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path)
        .map_err(|e| format!("open ledger {ledger_path}: {e}"))?;
    writeln!(file, "{}", serde_json::to_string(&record).map_err(|e| e.to_string())?)
        .map_err(|e| format!("write ledger: {e}"))?;
    Ok(())
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as ISO8601 UTC (no external dep needed for a simple timestamp).
    let days = secs / 86400;
    let secs_of_day = secs % 86400;
    let h = secs_of_day / 3600;
    let m = (secs_of_day % 3600) / 60;
    let s = secs_of_day % 60;
    // Days since Unix epoch → calendar date (Gregorian).
    let (yr, mo, dy) = days_to_ymd(days);
    format!("{yr:04}-{mo:02}-{dy:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    // Rata Die algorithm (simple, no dep).
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn arg_value(raw: &[String], flag: &str) -> Option<String> {
    raw.iter()
        .position(|a| a == flag)
        .and_then(|i| raw.get(i + 1).cloned())
}

fn play_lifecycle(kind: LifecycleScenario) {
    let (brief, tape, cfg) = lifecycle_scenario_tape(kind, 0.0);
    let mut rt = DecisionLoopRuntime::new(cfg);
    rt.arm(brief);
    rt.play_tape(tape);
    emit(rt.surfaces());
}

fn play_steps(observations: &[MarketObservation]) {
    let mut rt = DecisionLoopRuntime::new(DeferredLiveConfig {
        strict_t0_admission: false,
        ..Default::default()
    });
    rt.arm(act_brief());
    for obs in observations {
        rt.step(obs.clone());
    }
    if observations.len() == 1 {
        rt.mark_tape_exhausted();
    }
    emit(rt.surfaces());
}

fn emit(surfaces: &[chronosentiment_adapter::product::DecisionSurface]) {
    for surface in surfaces {
        println!("{}", serde_json::to_string(surface).expect("DecisionSurface json"));
    }
}

fn obs(unix: i64, price: f64) -> MarketObservation {
    MarketObservation {
        ticker: TICKER.into(),
        unix,
        price,
        high: Some(price),
        low: Some(price),
    }
}

fn act_brief() -> DecisionBrief {
    DecisionBrief {
        id: "LIVE-HARNESS-JUBLFOOD_NS".into(),
        ticker: TICKER.into(),
        date: "2026-09-07".into(),
        direction: "SHORT".into(),
        oqs: 53,
        h60_class: "WAIT".into(),
        reference_price: Some(475.4),
        entry_price: Some(473.0),
        execution: ExecutionFacts {
            snap_unix: Some(SNAP),
            adaptive_target: Some(450.0),
            adaptive_risk: Some(490.0),
            adaptive_horizon_sessions: Some(1.0),
            target_distance_abs: None,
            target_distance_pct: None,
            risk_distance_abs: None,
            risk_distance_pct: None,
            expected_move_pct: None,
            current_price: None,
            last_tick_unix: None,
            freshness: "STALE".into(),
            horizon_elapsed: Some(false),
        },
        entry_state: "WAIT-HIGH".into(),
        entry_action: "ACT".into(),
        entry_confidence: "HIGH".into(),
        entry_horizon: "H300".into(),
        entry_why: "frozen session brief".into(),
        entry_risk: "frozen session brief".into(),
        h120_state: "WAIT-HIGH".into(),
        h120_action: "ACT".into(),
        h120_confidence: "HIGH".into(),
        h120_horizon: "H300".into(),
        h120_why: "frozen session brief".into(),
        h120_risk: "frozen session brief".into(),
        h15_ret: None,
        h30_ret: None,
        h60_ret: None,
        h120_ret: None,
        h180_ret: None,
        h300_ret: None,
        mfe_h60: None,
        mfe_h120: None,
        outcome: None,
        pnl: None,
        hist_win: None,
        hist_pf: None,
        hist_med: None,
    }
}
