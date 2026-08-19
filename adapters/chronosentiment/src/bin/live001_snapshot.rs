//! LIVE-001 — Live Market Snapshot
//!
//! Produces a persisted, immutable snapshot artifact containing all C3-002
//! inputs for the full instrument universe, fetched from live Yahoo Finance
//! data at a defined point in time.
//!
//! ## What this is
//!
//! LIVE-001 answers exactly one question:
//!
//! > **Can we produce a fresh, complete, timestamp-consistent C3-002 input
//! > snapshot for the full universe at a defined point in time?**
//!
//! It does NOT evaluate, decide, or score. It only acquires reality.
//! LIVE-002 will consume this artifact to interpret reality.
//!
//! ## Artifact schema
//!
//! ```json
//! {
//!   "snapshot_id":        "LIVE-20260819-0930-001",
//!   "snapshot_timestamp": "2026-08-19T03:45:00Z",
//!   "source_type":        "LIVE",
//!   "producer":           "live001_snapshot.v1",
//!   "acquired_at":        "2026-08-19T04:01:23Z",
//!   "universe_file":      "datasets/universes/coralys_102_v1.json",
//!   "n_instruments":      102,
//!   "n_complete":         98,
//!   "n_incomplete":       4,
//!   "completeness_status": "PARTIAL",
//!   "instruments": [
//!     {
//!       "ticker":              "RELIANCE.NS",
//!       "source":              "YahooFinance",
//!       "source_bar_timestamp": "2026-08-18T00:00:00Z",
//!       "acquisition_timestamp": "2026-08-19T04:01:05Z",
//!       "n_bars":              1234,
//!       "reference_price":     2900.0,
//!       "atr_14":              58.3,
//!       "trend":               "Bullish",
//!       "momentum":            "Positive",
//!       "volatility":          "Normal",
//!       "tmv_complete":        true,
//!       "completeness_status": "COMPLETE"
//!     }
//!   ]
//! }
//! ```
//!
//! ## Governance
//!
//! - Does NOT create decisions.
//! - Does NOT emit to the Decision Server.
//! - Does NOT modify any ledger.
//! - Does NOT use the historical CS-P-006 yahoo cache.
//! - source_type = LIVE distinguishes this from historical/control population.
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p chronosentiment_adapter --bin live001_snapshot -- \
//!   --universe datasets/universes/coralys_102_v1.json \
//!   --output product_validation/LIVE-001/snapshots \
//!   --now 2026-08-19T03:45:00Z
//! ```
//!
//! Set CHRONO_YAHOO_CACHE_DIR to a writable directory to cache fetched bars
//! (strongly recommended — avoids re-fetching on repeated runs):
//!
//! ```bash
//! CHRONO_YAHOO_CACHE_DIR=product_validation/LIVE-001/yahoo_cache \
//!   cargo run -p chronosentiment_adapter --bin live001_snapshot -- ...
//! ```

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use chronosentiment_adapter::decision_support::enrichment_certify::{
    assess_from_bars_at_t, metrics_from_bars_at_t,
};
use chronosentiment_adapter::decision_support::forward_tick::instrument_id_for;
use chronosentiment_adapter::decision_support::observatory_prospective::latest_session_at_or_before;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::YahooProvider;
use chronosentiment_adapter::instrument::Instrument;
use serde::{Deserialize, Serialize};

// ─── Artifact types ───────────────────────────────────────────────────────────

const PRODUCER: &str = "live001_snapshot.v1";
const SOURCE_TYPE: &str = "LIVE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentSnapshot {
    pub ticker: String,
    pub source: String,
    /// Timestamp of the most recent bar used (≤ snapshot_timestamp).
    pub source_bar_timestamp: Option<DateTime<Utc>>,
    /// Wall-clock time when this instrument's bars were acquired.
    pub acquisition_timestamp: DateTime<Utc>,
    pub n_bars: usize,
    pub reference_price: Option<f64>,
    pub atr_14: Option<f64>,
    pub trend: Option<String>,
    pub momentum: Option<String>,
    pub volatility: Option<String>,
    pub tmv_complete: bool,
    pub completeness_status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSnapshot {
    pub snapshot_id: String,
    pub snapshot_timestamp: DateTime<Utc>,
    pub source_type: String,
    pub producer: String,
    pub acquired_at: DateTime<Utc>,
    pub universe_file: String,
    pub n_instruments: usize,
    pub n_complete: usize,
    pub n_incomplete: usize,
    pub n_error: usize,
    pub completeness_status: String,
    pub instruments: Vec<InstrumentSnapshot>,
}

// ─── Universe loading ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct UniverseFile {
    instruments: Vec<String>,
}

fn load_universe(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("cannot read universe file {}: {e}", path.display()))?;
    let universe: UniverseFile = serde_json::from_str(&raw)
        .map_err(|e| format!("universe JSON parse error: {e}"))?;
    Ok(universe.instruments)
}

// ─── Snapshot ID ─────────────────────────────────────────────────────────────

fn make_snapshot_id(ts: DateTime<Utc>) -> String {
    format!("LIVE-{}", ts.format("%Y%m%d-%H%M"))
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // Load universe.
    let tickers = load_universe(&args.universe)?;
    println!(
        "[live001] universe={} n_instruments={}",
        args.universe.display(),
        tickers.len()
    );

    let snapshot_id = make_snapshot_id(args.now);
    println!("[live001] snapshot_id={snapshot_id}");
    println!("[live001] snapshot_timestamp={}", args.now);

    let yahoo = YahooProvider::new();
    let mut instrument_snapshots: Vec<InstrumentSnapshot> = Vec::new();

    for ticker in &tickers {
        let acquisition_timestamp = Utc::now();

        // Build a minimal Instrument for the Yahoo provider.
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: uuid::Uuid::nil(),
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: args.now,
        };

        // Fetch bars (incremental — uses CHRONO_YAHOO_CACHE_DIR if set).
        let bars = match yahoo
            .fetch_historical(&instrument, TimeRange::FiveYears)
            .await
        {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[live001] error ticker={ticker} error={e}");
                instrument_snapshots.push(InstrumentSnapshot {
                    ticker: ticker.clone(),
                    source: "YahooFinance".to_string(),
                    source_bar_timestamp: None,
                    acquisition_timestamp,
                    n_bars: 0,
                    reference_price: None,
                    atr_14: None,
                    trend: None,
                    momentum: None,
                    volatility: None,
                    tmv_complete: false,
                    completeness_status: "ERROR".to_string(),
                    error: Some(e.to_string()),
                });
                continue;
            }
        };

        // Find the latest session at or before snapshot_timestamp.
        let t = latest_session_at_or_before(&bars, args.now).unwrap_or(args.now);

        // Source bar timestamp — the most recent bar used.
        let source_bar_timestamp = bars
            .iter()
            .filter(|b| b.timestamp <= t.timestamp())
            .last()
            .and_then(|b| Utc.timestamp_opt(b.timestamp, 0).single());

        // Reference price — last close ≤ T.
        let reference_price = bars
            .iter()
            .filter(|b| b.timestamp <= t.timestamp())
            .last()
            .and_then(|b| if b.close > 0.0 { Some(b.close) } else { None });

        // ATR-14.
        let instrument_id = instrument_id_for(ticker);
        let metrics = metrics_from_bars_at_t(&bars, t, instrument_id);
        let atr_14 = metrics.get_float("atr_14");

        // TMV assessment (trend, momentum, volatility).
        let (profile, _, _) = assess_from_bars_at_t(&bars, t, instrument_id);
        let trend = profile
            .factor_status
            .iter()
            .find(|s| s.concept == chronosentiment_adapter::metrics::concepts::Concept::Trend)
            .map(|s| format!("{:?}", s.availability));
        let momentum = profile
            .factor_status
            .iter()
            .find(|s| s.concept == chronosentiment_adapter::metrics::concepts::Concept::Momentum)
            .map(|s| format!("{:?}", s.availability));
        let volatility = profile
            .factor_status
            .iter()
            .find(|s| s.concept == chronosentiment_adapter::metrics::concepts::Concept::Volatility)
            .map(|s| format!("{:?}", s.availability));

        let tmv_complete = trend.is_some() && momentum.is_some() && volatility.is_some()
            && reference_price.is_some()
            && atr_14.is_some();

        let completeness_status = if tmv_complete {
            "COMPLETE"
        } else {
            "INCOMPLETE"
        };

        println!(
            "[live001] ticker={ticker} ref={reference_price:?} atr_14={atr_14:?} tmv={tmv_complete} status={completeness_status}"
        );

        instrument_snapshots.push(InstrumentSnapshot {
            ticker: ticker.clone(),
            source: "YahooFinance".to_string(),
            source_bar_timestamp,
            acquisition_timestamp,
            n_bars: bars.len(),
            reference_price,
            atr_14,
            trend,
            momentum,
            volatility,
            tmv_complete,
            completeness_status: completeness_status.to_string(),
            error: None,
        });
    }

    // Aggregate completeness.
    let n_complete = instrument_snapshots
        .iter()
        .filter(|i| i.completeness_status == "COMPLETE")
        .count();
    let n_error = instrument_snapshots
        .iter()
        .filter(|i| i.completeness_status == "ERROR")
        .count();
    let n_incomplete = instrument_snapshots.len() - n_complete - n_error;

    let completeness_status = if n_error == 0 && n_incomplete == 0 {
        "COMPLETE"
    } else if n_complete > 0 {
        "PARTIAL"
    } else {
        "FAILED"
    };

    let acquired_at = Utc::now();

    let snapshot = LiveSnapshot {
        snapshot_id: snapshot_id.clone(),
        snapshot_timestamp: args.now,
        source_type: SOURCE_TYPE.to_string(),
        producer: PRODUCER.to_string(),
        acquired_at,
        universe_file: args.universe.to_string_lossy().to_string(),
        n_instruments: instrument_snapshots.len(),
        n_complete,
        n_incomplete,
        n_error,
        completeness_status: completeness_status.to_string(),
        instruments: instrument_snapshots,
    };

    // Persist snapshot artifact.
    fs::create_dir_all(&args.output)?;
    let artifact_path = args.output.join(format!("{snapshot_id}.json"));
    fs::write(&artifact_path, serde_json::to_vec_pretty(&snapshot)?)?;

    // Also write a `latest.json` symlink-equivalent (overwrite).
    let latest_path = args.output.join("latest.json");
    fs::write(&latest_path, serde_json::to_vec_pretty(&snapshot)?)?;

    println!("[live001] result={completeness_status}");
    println!("[live001] n_instruments={}", snapshot.n_instruments);
    println!("[live001] n_complete={n_complete}");
    println!("[live001] n_incomplete={n_incomplete}");
    println!("[live001] n_error={n_error}");
    println!("[live001] artifact={}", artifact_path.display());
    println!("[live001] latest={}", latest_path.display());

    if completeness_status == "FAILED" {
        return Err("snapshot completeness=FAILED — no instruments produced complete C3-002 inputs".into());
    }

    Ok(())
}

// ─── Arg parsing ─────────────────────────────────────────────────────────────

struct Args {
    universe: PathBuf,
    output: PathBuf,
    now: DateTime<Utc>,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut universe = None;
    let mut output = None;
    let mut now_raw = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--universe" => {
                universe = Some(PathBuf::from(args.next().ok_or("missing --universe")?))
            }
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            "--now" => now_raw = Some(args.next().ok_or("missing --now")?),
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }

    let now = match now_raw {
        Some(s) => s.parse().map_err(|e| format!("--now must be RFC3339: {e}"))?,
        None => Utc::now(),
    };

    Ok(Args {
        universe: universe
            .unwrap_or_else(|| PathBuf::from("datasets/universes/coralys_102_v1.json")),
        output: output
            .unwrap_or_else(|| PathBuf::from("product_validation/LIVE-001/snapshots")),
        now,
    })
}