//! INTRA-006 — Intraday Resolution Observer (INTRADAY-001)
//!
//! Consumes the existing TIME-009 ledger entries and resolves their outcomes
//! using 1-minute OHLCV bars at fixed horizons (H15, H30, H120, H180, H240, H300).
//!
//! ## What this does
//!
//! - Reads all TIME-009 ledger entries from `live_capture/ledger/entries/`
//! - For each entry, loads 1-minute bars from the intraday cache
//! - Resolves target/risk hit at each horizon using the protocol-specified rules
//! - Writes observation artifacts to `time_machine/analysis/INTRADAY001/observations/`
//!
//! ## Governance
//!
//! - Does NOT modify TIME-009 ledger, daily observer, or daily observations
//! - Does NOT modify the 1-minute or 5-minute caches
//! - COMPLETE observations are immutable (idempotency: never overwritten)
//! - Temporal firewall: only bars strictly AFTER source_snapshot_timestamp are used
//!
//! ## Usage
//!
//! ```bash
//! CHRONO_YAHOO_CACHE_1M_DIR=intraday_capture/yahoo_cache_1m \
//!   cargo run -p chronosentiment_adapter --bin intraday001_observe -- \
//!   --ledger  live_capture/ledger \
//!   --output  time_machine/analysis/INTRADAY001/observations \
//!   --cache1m intraday_capture/yahoo_cache_1m
//! ```

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};

const PRODUCER: &str = "intraday001_observe.v1";

/// Horizons in trading minutes (pre-specified in INTRADAY-001 protocol §4).
const HORIZONS: &[(u32, &str)] = &[
    (15, "H15"),
    (30, "H30"),
    (120, "H120"),
    (180, "H180"),
    (240, "H240"),
    (300, "H300"),
];

/// NSE session: 09:15–15:30 IST = 03:45–10:00 UTC
const NSE_OPEN_HOUR_UTC: u32 = 3;
const NSE_OPEN_MIN_UTC: u32 = 45;
const NSE_CLOSE_HOUR_UTC: u32 = 10;
const NSE_CLOSE_MIN_UTC: u32 = 0;

// ─── Input schema (TIME-009 ledger entry) ─────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
struct LedgerEntry {
    decision_id: String,
    admitted_at: String,
    certification_id: String,
    certification_status: String,
    certified_at: String,
    recommendation_id: String,
    recommended_at: String,
    source_snapshot_id: String,
    source_snapshot_timestamp: String,
    source_state_id: String,
    c3_002_artifact_hash: String,
    ticker: String,
    direction: String,
    action: String,
    reference_price: Option<f64>,
    adaptive_target: Option<f64>,
    adaptive_risk: Option<f64>,
    adaptive_horizon_sessions: Option<f64>,
    evidence_class: String,
    vol_regime: String,
    volume_regime: String,
    degradation_level: String,
    sample_size: usize,
    target_rate: f64,
    rank_score: f64,
}

// ─── Intraday bar ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Bar {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    adj_close: f64,
    volume: f64,
}

// ─── Output schema ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
struct IntradayObservation {
    observation_id: String,
    decision_id: String,
    horizon_label: String,
    horizon_minutes: u32,
    observed_at: String,
    producer: String,
    observation_status: String,

    // T0 provenance (verbatim from ledger)
    admitted_at: String,
    source_snapshot_timestamp: String,
    certification_id: String,
    certification_status: String,
    recommendation_id: String,

    // Decision fields
    ticker: String,
    direction: String,
    action: String,
    evidence_class: String,
    reference_price: Option<f64>,
    adaptive_target: Option<f64>,
    adaptive_risk: Option<f64>,

    // Outcome (None if PENDING)
    exit_reason: Option<String>,
    target_reached: Option<bool>,
    risk_reached: Option<bool>,
    realized_return: Option<f64>,
    actual_mfe: Option<f64>,
    actual_mae: Option<f64>,
    exit_bar_index: Option<usize>,
    n_bars_inspected: Option<usize>,
    exit_price: Option<f64>,
    eligible_for_primary_comparison: Option<bool>,
    n_1m_bars_after_t0: usize,
}

// ─── Args ─────────────────────────────────────────────────────────────────────

struct Args {
    ledger: PathBuf,
    output: PathBuf,
    cache1m: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut ledger = PathBuf::from("live_capture/ledger");
    let mut output = PathBuf::from("time_machine/analysis/INTRADAY001/observations");
    let mut cache1m = PathBuf::from(
        std::env::var("CHRONO_YAHOO_CACHE_1M_DIR")
            .unwrap_or_else(|_| "intraday_capture/yahoo_cache_1m".to_string()),
    );
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ledger" => {
                i += 1;
                ledger = PathBuf::from(&args[i]);
            }
            "--output" => {
                i += 1;
                output = PathBuf::from(&args[i]);
            }
            "--cache1m" => {
                i += 1;
                cache1m = PathBuf::from(&args[i]);
            }
            _ => {}
        }
        i += 1;
    }
    Ok(Args { ledger, output, cache1m })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn load_ledger_entries(
    ledger_dir: &PathBuf,
) -> Result<Vec<LedgerEntry>, Box<dyn std::error::Error>> {
    let entries_dir = ledger_dir.join("entries");
    let mut entries = Vec::new();
    if !entries_dir.exists() {
        return Ok(entries);
    }
    for entry in fs::read_dir(&entries_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        match serde_json::from_str::<LedgerEntry>(&content) {
            Ok(rec) => entries.push(rec),
            Err(e) => {
                eprintln!("[intra006] WARN: cannot parse {}: {e}", path.display());
            }
        }
    }
    entries.sort_by(|a, b| a.ticker.cmp(&b.ticker).then(a.admitted_at.cmp(&b.admitted_at)));
    Ok(entries)
}

/// Load 1-minute bars for a ticker from the intraday cache.
fn load_1m_cache(cache_dir: &PathBuf, ticker_ns: &str) -> Vec<Bar> {
    // ticker in ledger is TICKER_NS (underscore); cache files are TICKER.NS (dot)
    let ticker_dot = ticker_ns.replace("_NS", ".NS");
    let path = cache_dir.join(format!("{ticker_dot}.json"));
    if !path.exists() {
        return vec![];
    }
    match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<Vec<Bar>>(&bytes).unwrap_or_default(),
        Err(_) => vec![],
    }
}

/// Load all existing COMPLETE observation IDs for idempotency.
fn load_complete_ids(output_dir: &PathBuf) -> HashSet<String> {
    let mut ids = HashSet::new();
    if !output_dir.exists() {
        return ids;
    }
    for entry in fs::read_dir(output_dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if val.get("observation_status").and_then(|v| v.as_str()) == Some("COMPLETE") {
                    if let Some(id) = val.get("observation_id").and_then(|v| v.as_str()) {
                        ids.insert(id.to_string());
                    }
                }
            }
        }
    }
    ids
}

/// Returns true if the given UTC timestamp falls within an NSE trading session
/// (Mon–Fri, 03:45–10:00 UTC).
fn is_nse_trading_minute(ts: i64) -> bool {
    let dt = match chrono::DateTime::from_timestamp(ts, 0) {
        Some(d) => d,
        None => return false,
    };
    let weekday = dt.weekday();
    if matches!(weekday, Weekday::Sat | Weekday::Sun) {
        return false;
    }
    let h = dt.hour();
    let m = dt.minute();
    let total_min = h * 60 + m;
    let open_min = NSE_OPEN_HOUR_UTC * 60 + NSE_OPEN_MIN_UTC;
    let close_min = NSE_CLOSE_HOUR_UTC * 60 + NSE_CLOSE_MIN_UTC;
    total_min >= open_min && total_min < close_min
}

/// Count trading minutes from T0 (exclusive) to a bar timestamp (inclusive).
/// Only counts minutes within NSE sessions.
fn trading_minutes_from_t0(t0_ts: i64, bar_ts: i64, bars_after_t0: &[&Bar]) -> u32 {
    // Count bars strictly after T0 up to and including bar_ts that are in-session.
    bars_after_t0
        .iter()
        .filter(|b| b.timestamp <= bar_ts && is_nse_trading_minute(b.timestamp))
        .count() as u32
}

/// Resolve outcome for a single decision at a given horizon using 1-minute bars.
/// Returns (exit_reason, target_reached, risk_reached, realized_return, mfe, mae,
///          exit_bar_index, n_bars_inspected, exit_price, eligible).
#[allow(clippy::too_many_arguments)]
fn resolve_outcome(
    direction: &str,
    reference_price: f64,
    adaptive_target: f64,
    adaptive_risk: f64,
    action: &str,
    evidence_class: &str,
    certification_status: &str,
    bars_after_t0: &[&Bar],
    horizon_minutes: u32,
) -> (String, bool, bool, f64, f64, f64, usize, usize, f64, bool) {
    let is_long = direction == "LONG";

    // NO_TRADE: action is NoTrade
    if action == "NoTrade" {
        return (
            "NO_TRADE".to_string(),
            false,
            false,
            0.0,
            0.0,
            0.0,
            0,
            0,
            reference_price,
            false,
        );
    }

    // Collect bars within the horizon (trading minutes)
    let mut horizon_bars: Vec<&Bar> = Vec::new();
    let mut trading_min_count = 0u32;
    for bar in bars_after_t0 {
        if !is_nse_trading_minute(bar.timestamp) {
            continue;
        }
        trading_min_count += 1;
        horizon_bars.push(bar);
        if trading_min_count >= horizon_minutes {
            break;
        }
    }

    // INSUFFICIENT_DATA: fewer than 5 bars
    if horizon_bars.len() < 5 {
        return (
            "INSUFFICIENT_DATA".to_string(),
            false,
            false,
            0.0,
            0.0,
            0.0,
            0,
            horizon_bars.len(),
            reference_price,
            false,
        );
    }

    let n_bars = horizon_bars.len();
    let mut target_bar_idx: Option<usize> = None;
    let mut risk_bar_idx: Option<usize> = None;
    let mut mfe: f64 = 0.0;
    let mut mae: f64 = 0.0;

    for (idx, bar) in horizon_bars.iter().enumerate() {
        // MFE/MAE tracking
        let favorable = if is_long {
            (bar.high - reference_price) / reference_price
        } else {
            (reference_price - bar.low) / reference_price
        };
        let adverse = if is_long {
            (bar.low - reference_price) / reference_price
        } else {
            (reference_price - bar.high) / reference_price
        };
        if favorable > mfe {
            mfe = favorable;
        }
        if adverse < mae {
            mae = adverse;
        }

        // Target hit check
        if target_bar_idx.is_none() {
            let target_hit = if is_long {
                bar.high >= adaptive_target
            } else {
                bar.low <= adaptive_target
            };
            if target_hit {
                target_bar_idx = Some(idx);
            }
        }

        // Risk hit check
        if risk_bar_idx.is_none() {
            let risk_hit = if is_long {
                bar.low <= adaptive_risk
            } else {
                bar.high >= adaptive_risk
            };
            if risk_hit {
                risk_bar_idx = Some(idx);
            }
        }
    }

    let (exit_reason, exit_bar_idx, target_reached, risk_reached) =
        match (target_bar_idx, risk_bar_idx) {
            (Some(t), Some(r)) if t == r => {
                // Same bar — AMBIGUOUS
                ("AMBIGUOUS".to_string(), t, true, true)
            }
            (Some(t), Some(r)) if t < r => {
                // Target first
                ("TARGET".to_string(), t, true, false)
            }
            (Some(_t), Some(r)) => {
                // Risk first
                ("RISK".to_string(), r, false, true)
            }
            (Some(t), None) => ("TARGET".to_string(), t, true, false),
            (None, Some(r)) => ("RISK".to_string(), r, false, true),
            (None, None) => ("HORIZON".to_string(), n_bars - 1, false, false),
        };

    let exit_bar = horizon_bars[exit_bar_idx];
    let exit_price = exit_bar.close;
    let realized_return = if is_long {
        (exit_price - reference_price) / reference_price
    } else {
        (reference_price - exit_price) / reference_price
    };

    // Eligibility (protocol §7)
    let eligible = matches!(certification_status, "CERTIFIED" | "DEGRADED")
        && matches!(evidence_class, "Favourable" | "Mixed")
        && !matches!(
            exit_reason.as_str(),
            "AMBIGUOUS" | "INSUFFICIENT_DATA" | "NO_TRADE"
        );

    (
        exit_reason,
        target_reached,
        risk_reached,
        realized_return,
        mfe,
        mae,
        exit_bar_idx + 1,
        n_bars,
        exit_price,
        eligible,
    )
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    fs::create_dir_all(&args.output)?;

    let run_id = format!(
        "INTRADAY001-{}",
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
    );
    println!("[intra006] producer={PRODUCER}");
    println!("[intra006] run_id={run_id}");
    println!("[intra006] cache1m={}", args.cache1m.display());

    let entries = load_ledger_entries(&args.ledger)?;
    println!("[intra006] n_ledger_entries={}", entries.len());

    let complete_ids = load_complete_ids(&args.output);
    println!("[intra006] n_already_complete={}", complete_ids.len());

    let mut n_complete = 0usize;
    let mut n_pending = 0usize;
    let mut n_no_bars = 0usize;
    let mut n_skipped = 0usize;
    let mut n_invalid_gap = 0usize;

    // Maximum gap in seconds between T0 and first bar after T0 before we
    // declare the observation INVALID_DATA_GAP.
    // A Friday market-close T0 (15:30 IST = 10:00 UTC) has its next bar on
    // Monday open (09:15 IST = 03:45 UTC) — a gap of ~65.5 hours (2.73 days).
    // We allow up to 4 calendar days (345600s) to accommodate any weekend gap.
    const MAX_GAP_SECONDS: i64 = 4 * 24 * 3600; // 4 calendar days

    for entry in &entries {
        // Parse T0
        let t0: DateTime<Utc> = match entry.source_snapshot_timestamp.parse() {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[intra006] WARN: cannot parse source_snapshot_timestamp for {}: {e}",
                    entry.decision_id
                );
                continue;
            }
        };
        let t0_ts = t0.timestamp();

        // Load 1-minute bars
        let all_bars = load_1m_cache(&args.cache1m, &entry.ticker);

        // Bars strictly after T0 (temporal firewall AC-T9-02 equivalent)
        let bars_after_t0: Vec<&Bar> = all_bars.iter().filter(|b| b.timestamp > t0_ts).collect();
        let n_bars_after = bars_after_t0.len();

        if n_bars_after == 0 {
            n_no_bars += 1;
        }

        // ── Data-gap guard ────────────────────────────────────────────────────
        // If the first bar after T0 is more than MAX_GAP_SECONDS away, the
        // 1-minute cache does not cover the session immediately following the
        // decision. Resolving outcomes using bars from a later session would
        // produce invalid observations. Mark all horizons INVALID_DATA_GAP.
        let has_data_gap = bars_after_t0
            .first()
            .map(|b| b.timestamp - t0_ts > MAX_GAP_SECONDS)
            .unwrap_or(false);

        if has_data_gap {
            let first_bar_ts = bars_after_t0.first().map(|b| b.timestamp).unwrap_or(0);
            let gap_days = (first_bar_ts - t0_ts) / 86400;
            for &(_horizon_min, horizon_label) in HORIZONS {
                let observation_id = format!(
                    "INTRADAY001-OBS-{}-{}",
                    entry.decision_id, horizon_label
                );
                // Skip if already written (any status)
                let path = args.output.join(format!("{observation_id}.json"));
                if path.exists() {
                    n_skipped += 1;
                    continue;
                }
                let obs = serde_json::json!({
                    "observation_id": observation_id,
                    "decision_id": entry.decision_id,
                    "horizon_label": horizon_label,
                    "observed_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string(),
                    "producer": PRODUCER,
                    "observation_status": "INVALID_DATA_GAP",
                    "admitted_at": entry.admitted_at,
                    "source_snapshot_timestamp": entry.source_snapshot_timestamp,
                    "ticker": entry.ticker,
                    "direction": entry.direction,
                    "evidence_class": entry.evidence_class,
                    "gap_days": gap_days,
                    "n_1m_bars_after_t0": n_bars_after,
                    "eligible_for_primary_comparison": false,
                    "invalidation_reason": format!(
                        "First 1m bar after T0 is {}d away — outside Yahoo 1m retention window",
                        gap_days
                    )
                });
                fs::write(&path, serde_json::to_vec_pretty(&obs)?)?;
                n_invalid_gap += 1;
            }
            eprintln!(
                "[intra006] INVALID_DATA_GAP ticker={} gap_days={} id={}",
                entry.ticker, gap_days, entry.decision_id
            );
            continue;
        }

        let ref_price = match entry.reference_price {
            Some(p) if p > 0.0 => p,
            _ => {
                eprintln!(
                    "[intra006] WARN: no reference_price for {} — skipping",
                    entry.decision_id
                );
                continue;
            }
        };
        let target = entry.adaptive_target.unwrap_or(ref_price * 1.05);
        let risk = entry.adaptive_risk.unwrap_or(ref_price * 0.95);

        for &(horizon_min, horizon_label) in HORIZONS {
            let observation_id = format!(
                "INTRADAY001-OBS-{}-{}",
                entry.decision_id, horizon_label
            );

            // Idempotency: skip if already written (COMPLETE or PENDING)
            if complete_ids.contains(&observation_id) {
                n_skipped += 1;
                continue;
            }

            let observed_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();

            // Check if we have enough bars to resolve this horizon
            let trading_bars_in_horizon: Vec<&Bar> = bars_after_t0
                .iter()
                .copied()
                .filter(|b| is_nse_trading_minute(b.timestamp))
                .take(horizon_min as usize)
                .collect();

            let has_full_horizon = trading_bars_in_horizon.len() >= horizon_min as usize;

            if !has_full_horizon && n_bars_after < 5 {
                // PENDING — not enough data yet
                let obs = IntradayObservation {
                    observation_id: observation_id.clone(),
                    decision_id: entry.decision_id.clone(),
                    horizon_label: horizon_label.to_string(),
                    horizon_minutes: horizon_min,
                    observed_at,
                    producer: PRODUCER.to_string(),
                    observation_status: "PENDING".to_string(),
                    admitted_at: entry.admitted_at.clone(),
                    source_snapshot_timestamp: entry.source_snapshot_timestamp.clone(),
                    certification_id: entry.certification_id.clone(),
                    certification_status: entry.certification_status.clone(),
                    recommendation_id: entry.recommendation_id.clone(),
                    ticker: entry.ticker.clone(),
                    direction: entry.direction.clone(),
                    action: entry.action.clone(),
                    evidence_class: entry.evidence_class.clone(),
                    reference_price: entry.reference_price,
                    adaptive_target: entry.adaptive_target,
                    adaptive_risk: entry.adaptive_risk,
                    exit_reason: None,
                    target_reached: None,
                    risk_reached: None,
                    realized_return: None,
                    actual_mfe: None,
                    actual_mae: None,
                    exit_bar_index: None,
                    n_bars_inspected: None,
                    exit_price: None,
                    eligible_for_primary_comparison: None,
                    n_1m_bars_after_t0: n_bars_after,
                };
                let path = args.output.join(format!("{observation_id}.json"));
                fs::write(&path, serde_json::to_vec_pretty(&obs)?)?;
                n_pending += 1;
                println!(
                    "[intra006] PENDING ticker={} id={} horizon={} n_bars_after_t0={}",
                    entry.ticker, entry.decision_id, horizon_label, n_bars_after
                );
                continue;
            }

            // Resolve outcome
            let (exit_reason, target_reached, risk_reached, realized_return, mfe, mae,
                 exit_bar_idx, n_bars_inspected, exit_price, eligible) = resolve_outcome(
                &entry.direction,
                ref_price,
                target,
                risk,
                &entry.action,
                &entry.evidence_class,
                &entry.certification_status,
                &bars_after_t0,
                horizon_min,
            );

            let obs = IntradayObservation {
                observation_id: observation_id.clone(),
                decision_id: entry.decision_id.clone(),
                horizon_label: horizon_label.to_string(),
                horizon_minutes: horizon_min,
                observed_at,
                producer: PRODUCER.to_string(),
                observation_status: "COMPLETE".to_string(),
                admitted_at: entry.admitted_at.clone(),
                source_snapshot_timestamp: entry.source_snapshot_timestamp.clone(),
                certification_id: entry.certification_id.clone(),
                certification_status: entry.certification_status.clone(),
                recommendation_id: entry.recommendation_id.clone(),
                ticker: entry.ticker.clone(),
                direction: entry.direction.clone(),
                action: entry.action.clone(),
                evidence_class: entry.evidence_class.clone(),
                reference_price: entry.reference_price,
                adaptive_target: entry.adaptive_target,
                adaptive_risk: entry.adaptive_risk,
                exit_reason: Some(exit_reason.clone()),
                target_reached: Some(target_reached),
                risk_reached: Some(risk_reached),
                realized_return: Some(realized_return),
                actual_mfe: Some(mfe),
                actual_mae: Some(mae),
                exit_bar_index: Some(exit_bar_idx),
                n_bars_inspected: Some(n_bars_inspected),
                exit_price: Some(exit_price),
                eligible_for_primary_comparison: Some(eligible),
                n_1m_bars_after_t0: n_bars_after,
            };

            let path = args.output.join(format!("{observation_id}.json"));
            fs::write(&path, serde_json::to_vec_pretty(&obs)?)?;
            n_complete += 1;
            println!(
                "[intra006] COMPLETE ticker={} horizon={} exit={} eligible={}",
                entry.ticker, horizon_label, exit_reason, eligible
            );
        }
    }

    println!("[intra006] result=OK");
    println!("[intra006] run_id={run_id}");
    println!("[intra006] n_complete={n_complete}");
    println!("[intra006] n_pending={n_pending}");
    println!("[intra006] n_invalid_data_gap={n_invalid_gap}");
    println!("[intra006] n_no_bars={n_no_bars}");
    println!("[intra006] n_skipped_already_complete={n_skipped}");

    Ok(())
}