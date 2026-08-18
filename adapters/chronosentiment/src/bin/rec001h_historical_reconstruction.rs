//! REC-001-H — Historical Decision Reconstruction
//!
//! Builds a leakage-free, ticker-aware historical decision dataset for the
//! 102-stock universe. For every qualifying historical session T, reconstructs
//! what Coralys C3-002 would have known at T (trend, momentum, volatility,
//! ATR-14, volume, relative_volume_20, direction) and computes forward MFE/MAE
//! over 1–10 sessions plus outcome classification.
//!
//! ## Leakage guarantee
//!
//! For a decision at session T:
//! - Feature vector uses only bars with timestamp ≤ T.
//! - MFE/MAE/outcome use only bars with timestamp > T (sessions T+1 … T+10).
//! - No future information leaks into the feature vector.
//!
//! ## Output
//!
//! One JSONL file per ticker in `datasets/recommendation/historical/`.
//! Each line is a `HistoricalDecisionRecord` (see schema below).
//!
//! ## Volume governance
//!
//! Volume and relative_volume_20 are stored but are NOT used by
//! Recommendation Engine v0. They are captured now because they are cheap
//! to collect and expensive to reconstruct later. Volume-conditioned
//! recommendations require separate validation before becoming operational.
//!
//! ## Usage
//!
//! ```
//! cargo run -p chronosentiment_adapter --bin rec001h_historical_reconstruction -- \
//!   --universe datasets/universes/coralys_102_v1.json \
//!   --output datasets/recommendation/historical
//! ```

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::enrichment_certify::assess_from_bars_at_t;
use chronosentiment_adapter::decision_support::forward_tick::instrument_id_for;
use chronosentiment_adapter::decision_support::observatory_prospective::certified_tmv_from_profile;
use chronosentiment_adapter::decision_support::observatory_slice::generate_decision;
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::{YahooHistoricalBar, YahooProvider};
use chronosentiment_adapter::instrument::Instrument;
use serde::{Deserialize, Serialize};

// ─── Constants ────────────────────────────────────────────────────────────────

const C3_002_POLICY_ARTIFACT_HASH: &str =
    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121";

/// Minimum bars before T required to compute ATR-14 and relative_volume_20.
/// We need at least 20 bars of history before we can produce a valid decision.
const MIN_HISTORY_BARS: usize = 20;

/// Number of forward sessions for MFE/MAE computation.
const FORWARD_SESSIONS: usize = 10;

/// ATR multiplier for target (same as C3-002 v0 geometry).
/// t_mul = 2 × r_mul, so R:R ≈ 2.0.
const TARGET_ATR_MULT: f64 = 2.0;
const RISK_ATR_MULT: f64 = 1.0;

// ─── Output schema ────────────────────────────────────────────────────────────

/// One historical decision observation. All fields use only information
/// available at or before session T. MFE/MAE/outcome use only T+1…T+10.
#[derive(Debug, Serialize, Deserialize)]
struct HistoricalDecisionRecord {
    // Identity
    ticker: String,
    date: String,           // YYYY-MM-DD — session T
    timestamp_unix: i64,    // Unix timestamp of session T

    // Market state at T (feature vector — no future leakage)
    reference_price: f64,   // close at T
    open: f64,
    high: f64,
    low: f64,
    volume: f64,
    relative_volume_20: f64, // volume / median(prev 20 sessions volume)

    // Coralys state at T
    trend: String,
    momentum: String,
    volatility: String,
    direction: String,      // LONG / SHORT / NO_TRADE
    atr_14: f64,            // ATR-14 at T

    // Geometry (C3-002 v0 fixed geometry)
    target_distance_pct: Option<f64>,  // % from reference_price to indicative target
    risk_distance_pct: Option<f64>,    // % from reference_price to indicative risk
    indicative_target: Option<f64>,
    indicative_risk: Option<f64>,

    // Forward outcome (T+1 … T+10 only — no leakage)
    mfe_pct: [f64; 10],     // max favourable excursion at each session
    mae_pct: [f64; 10],     // max adverse excursion at each session
    sessions_available: usize, // how many forward sessions were available

    // Outcome classification
    outcome: String,        // TARGET_BEFORE_RISK / RISK_BEFORE_TARGET / HORIZON / INSUFFICIENT_DATA
    sessions_to_outcome: Option<usize>, // session at which outcome was determined
}

// ─── ATR-14 computation ───────────────────────────────────────────────────────

/// Compute ATR-14 using the standard Wilder true-range formula.
/// Requires at least 15 bars (14 TR values + 1 seed).
/// Returns None if insufficient data.
fn compute_atr14(bars: &[YahooHistoricalBar]) -> Option<f64> {
    if bars.len() < 15 {
        return None;
    }
    let n = bars.len();
    // True range for each bar (except the first)
    let mut trs: Vec<f64> = Vec::with_capacity(n - 1);
    for i in 1..n {
        let high = bars[i].high;
        let low = bars[i].low;
        let prev_close = bars[i - 1].close;
        let tr = (high - low)
            .max((high - prev_close).abs())
            .max((low - prev_close).abs());
        trs.push(tr);
    }
    // Wilder smoothing: seed = simple average of first 14 TRs
    let seed: f64 = trs[..14].iter().sum::<f64>() / 14.0;
    let mut atr = seed;
    for &tr in &trs[14..] {
        atr = (atr * 13.0 + tr) / 14.0;
    }
    Some(atr)
}

/// Compute relative volume: today's volume / median of previous N sessions' volume.
fn relative_volume(bars: &[YahooHistoricalBar], today_idx: usize, window: usize) -> f64 {
    if today_idx == 0 {
        return 1.0;
    }
    let start = if today_idx >= window { today_idx - window } else { 0 };
    let prev_vols: Vec<f64> = bars[start..today_idx]
        .iter()
        .map(|b| b.volume)
        .filter(|v| *v > 0.0)
        .collect();
    if prev_vols.is_empty() {
        return 1.0;
    }
    let mut sorted = prev_vols.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    if median <= 0.0 {
        return 1.0;
    }
    bars[today_idx].volume / median
}

// ─── Forward MFE/MAE/outcome ──────────────────────────────────────────────────

/// Compute MFE/MAE for each of the next `n` sessions after `t_idx`.
///
/// For LONG: MFE = (high - entry) / entry, MAE = (low - entry) / entry
/// For SHORT: MFE = (entry - low) / entry, MAE = (entry - high) / entry
///
/// Returns (mfe_pct[10], mae_pct[10], sessions_available).
fn compute_forward_excursions(
    bars: &[YahooHistoricalBar],
    t_idx: usize,
    entry: f64,
    direction: &str,
) -> ([f64; 10], [f64; 10], usize) {
    let mut mfe = [0.0f64; 10];
    let mut mae = [0.0f64; 10];
    let available = (bars.len() - t_idx - 1).min(FORWARD_SESSIONS);
    for i in 0..available {
        let bar = &bars[t_idx + 1 + i];
        let (fav, adv) = if direction == "LONG" {
            ((bar.high - entry) / entry * 100.0, (bar.low - entry) / entry * 100.0)
        } else {
            // SHORT: favourable = price falls, adverse = price rises
            ((entry - bar.low) / entry * 100.0, (entry - bar.high) / entry * 100.0)
        };
        // Cumulative MFE/MAE: max favourable / min adverse seen so far
        mfe[i] = if i == 0 { fav.max(0.0) } else { mfe[i - 1].max(fav.max(0.0)) };
        mae[i] = if i == 0 { adv.min(0.0) } else { mae[i - 1].min(adv.min(0.0)) };
    }
    (mfe, mae, available)
}

/// Classify outcome using C3-002 v0 geometry.
///
/// For LONG:
///   target = entry * (1 + target_pct)
///   risk   = entry * (1 - risk_pct)
///
/// For SHORT:
///   target = entry * (1 - target_pct)
///   risk   = entry * (1 + risk_pct)
///
/// Scan forward bars in order. First exit wins.
fn classify_outcome(
    bars: &[YahooHistoricalBar],
    t_idx: usize,
    entry: f64,
    direction: &str,
    target_price: f64,
    risk_price: f64,
) -> (String, Option<usize>) {
    let available = (bars.len() - t_idx - 1).min(FORWARD_SESSIONS);
    if available == 0 {
        return ("INSUFFICIENT_DATA".to_string(), None);
    }
    for i in 0..available {
        let bar = &bars[t_idx + 1 + i];
        let (target_hit, risk_hit) = if direction == "LONG" {
            (bar.high >= target_price, bar.low <= risk_price)
        } else {
            (bar.low <= target_price, bar.high >= risk_price)
        };
        // If both hit on the same bar, use open to determine which came first.
        // Conservative: if open is already beyond risk, risk hit first.
        if target_hit && risk_hit {
            let open_beyond_risk = if direction == "LONG" {
                bar.open <= risk_price
            } else {
                bar.open >= risk_price
            };
            if open_beyond_risk {
                return ("RISK_BEFORE_TARGET".to_string(), Some(i + 1));
            } else {
                return ("TARGET_BEFORE_RISK".to_string(), Some(i + 1));
            }
        }
        if target_hit {
            return ("TARGET_BEFORE_RISK".to_string(), Some(i + 1));
        }
        if risk_hit {
            return ("RISK_BEFORE_TARGET".to_string(), Some(i + 1));
        }
    }
    ("HORIZON".to_string(), None)
}

// ─── Argument parsing ─────────────────────────────────────────────────────────

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let args: Vec<String> = env::args().collect();
    let mut universe_path: Option<PathBuf> = None;
    let mut output_path: Option<PathBuf> = None;
    let mut search_two = PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR);

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--universe" => {
                i += 1;
                universe_path = Some(PathBuf::from(
                    args.get(i).ok_or("missing --universe value")?,
                ));
            }
            "--output" => {
                i += 1;
                output_path = Some(PathBuf::from(
                    args.get(i).ok_or("missing --output value")?,
                ));
            }
            "--search-two" => {
                i += 1;
                search_two = PathBuf::from(args.get(i).ok_or("missing --search-two value")?);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }

    let universe = universe_path.ok_or("--universe is required")?;
    let output = output_path.unwrap_or_else(|| PathBuf::from("datasets/recommendation/historical"));
    Ok((search_two, universe, output))
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_two, universe_path, output_dir) = parse_args()?;

    // Load and verify C3-002 artifact
    let artifact: PolicyArtifact =
        serde_json::from_str(&fs::read_to_string(search_two.join("selected_policy.json"))?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }
    if artifact.artifact_hash != C3_002_POLICY_ARTIFACT_HASH {
        return Err("C3_002_POLICY_ARTIFACT_HASH mismatch".into());
    }

    // Load universe
    let raw = fs::read_to_string(&universe_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    let tickers: Vec<String> = parsed["instruments"]
        .as_array()
        .ok_or("universe JSON must have 'instruments' array")?
        .iter()
        .map(|v| v.as_str().ok_or("instrument must be a string").map(|s| s.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    println!(
        "rec001h universe={} count={} output={}",
        universe_path.display(),
        tickers.len(),
        output_dir.display()
    );

    // Create output directory
    fs::create_dir_all(&output_dir)?;

    let yahoo = YahooProvider::new();
    let mut total_records = 0u64;
    let mut total_tickers = 0u32;
    let mut skipped_tickers = 0u32;

    for ticker in &tickers {
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: uuid::Uuid::nil(),
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: Utc::now(),
        };

        let bars = match yahoo.fetch_historical(&instrument, TimeRange::FiveYears).await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("skip ticker={ticker} reason=yahoo_error error={e}");
                skipped_tickers += 1;
                continue;
            }
        };

        // Filter out bars with zero/invalid close
        let bars: Vec<YahooHistoricalBar> = bars
            .into_iter()
            .filter(|b| b.close > 0.0 && b.open > 0.0 && b.high > 0.0 && b.low > 0.0)
            .collect();

        if bars.len() < MIN_HISTORY_BARS + 1 {
            eprintln!(
                "skip ticker={ticker} reason=insufficient_bars bars={}",
                bars.len()
            );
            skipped_tickers += 1;
            continue;
        }

        let instrument_id = instrument_id_for(ticker);
        let mut records: Vec<HistoricalDecisionRecord> = Vec::new();

        // Walk each bar from MIN_HISTORY_BARS onward as the decision point T.
        // We need at least 1 forward bar for any outcome, so stop at len-2.
        for t_idx in MIN_HISTORY_BARS..bars.len().saturating_sub(1) {
            let t_bar = &bars[t_idx];
            let t_ts = match Utc.timestamp_opt(t_bar.timestamp, 0).single() {
                Some(ts) => ts,
                None => continue,
            };

            // Feature vector: bars[0..=t_idx] only (no future leakage)
            let history = &bars[..=t_idx];

            // ATR-14 at T
            let atr_14 = match compute_atr14(history) {
                Some(a) => a,
                None => continue,
            };

            // Relative volume (20-session window)
            let rel_vol = relative_volume(&bars, t_idx, 20);

            // Coralys state at T via assess_from_bars_at_t
            let (profile, _n, _max_from) = assess_from_bars_at_t(history, t_ts, instrument_id);
            let (trend, momentum, volatility) = certified_tmv_from_profile(&profile);

            // C3-002 direction at T
            let decision = match generate_decision(
                &artifact,
                ticker,
                &t_ts.to_rfc3339(),
                &trend,
                &momentum,
                &volatility,
            ) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("skip ticker={ticker} t={t_ts} reason=decision_error error={e}");
                    continue;
                }
            };

            let direction = match decision.action {
                DecisionAction::Long => "LONG",
                DecisionAction::Short => "SHORT",
                DecisionAction::NoTrade => "NO_TRADE",
            };

            let entry = t_bar.close;
            let date = t_ts.format("%Y-%m-%d").to_string();

            // Geometry (C3-002 v0 fixed ATR geometry)
            let (target_distance_pct, risk_distance_pct, indicative_target, indicative_risk) =
                if direction != "NO_TRADE" && atr_14 > 0.0 && entry > 0.0 {
                    let target_pct = (atr_14 * TARGET_ATR_MULT) / entry;
                    let risk_pct = (atr_14 * RISK_ATR_MULT) / entry;
                    let (tgt, rsk) = if direction == "LONG" {
                        (entry * (1.0 + target_pct), entry * (1.0 - risk_pct))
                    } else {
                        (entry * (1.0 - target_pct), entry * (1.0 + risk_pct))
                    };
                    (
                        Some(target_pct * 100.0),
                        Some(risk_pct * 100.0),
                        Some(tgt),
                        Some(rsk),
                    )
                } else {
                    (None, None, None, None)
                };

            // Forward MFE/MAE (T+1 … T+10)
            let (mfe_pct, mae_pct, sessions_available) =
                compute_forward_excursions(&bars, t_idx, entry, direction);

            // Outcome classification
            let (outcome, sessions_to_outcome) =
                if let (Some(tgt), Some(rsk)) = (indicative_target, indicative_risk) {
                    classify_outcome(&bars, t_idx, entry, direction, tgt, rsk)
                } else {
                    ("NO_GEOMETRY".to_string(), None)
                };

            records.push(HistoricalDecisionRecord {
                ticker: ticker.clone(),
                date,
                timestamp_unix: t_bar.timestamp,
                reference_price: entry,
                open: t_bar.open,
                high: t_bar.high,
                low: t_bar.low,
                volume: t_bar.volume,
                relative_volume_20: rel_vol,
                trend,
                momentum,
                volatility,
                direction: direction.to_string(),
                atr_14,
                target_distance_pct,
                risk_distance_pct,
                indicative_target,
                indicative_risk,
                mfe_pct,
                mae_pct,
                sessions_available,
                outcome,
                sessions_to_outcome,
            });
        }

        let n = records.len();
        if n == 0 {
            eprintln!("skip ticker={ticker} reason=no_records");
            skipped_tickers += 1;
            continue;
        }

        // Write JSONL output
        let safe_name = ticker.replace('.', "_").replace('/', "_");
        let out_path = output_dir.join(format!("{safe_name}.jsonl"));
        let mut lines = String::new();
        for rec in &records {
            lines.push_str(&serde_json::to_string(rec)?);
            lines.push('\n');
        }
        fs::write(&out_path, &lines)?;

        println!("ticker={ticker} records={n} path={}", out_path.display());
        total_records += n as u64;
        total_tickers += 1;
    }

    println!("---");
    println!("rec001h_complete=true");
    println!("tickers_processed={total_tickers}");
    println!("tickers_skipped={skipped_tickers}");
    println!("total_records={total_records}");
    println!("output={}", output_dir.display());

    Ok(())
}