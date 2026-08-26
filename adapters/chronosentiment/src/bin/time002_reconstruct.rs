//! TIME-002 — Point-in-Time Data Reconstruction.
//!
//! # Purpose
//!
//! Given a historical timestamp `T` and a universe of instruments, reconstruct
//! exactly what LIVE-001 would have captured at `T`, using **only** information
//! that was available at or before `T`.
//!
//! This is the temporal foundation of the Time Machine. It establishes the
//! historical reality that TIME-003 will then pass through the frozen Coralys
//! decision engine.
//!
//! # Architectural boundary
//!
//! TIME-002 establishes **what was knowable at T**. It does NOT decide anything.
//! No RecommendationEngine, no BUY/SELL/WATCH logic, no evidence classification.
//! That belongs in TIME-003.
//!
//! # Temporal invariants
//!
//! - T2-01: Every `source_timestamp ≤ T`
//! - T2-02: Every derived feature uses only data ≤ T
//! - T2-03: No `Utc::now()` — `HistoricalClock::replay(T)` only
//! - T2-04: Future cache records cannot contaminate reconstruction
//! - T2-05: `reconstruct(T, normal) == reconstruct(T, normal + future_poison)`
//! - T2-06: Same dataset + same T → deterministic output
//! - T2-07: Every universe instrument: COMPLETE / INCOMPLETE / ERROR
//!
//! # Data boundary rule
//!
//! ```text
//! Data(T) = { bar | bar.timestamp ≤ T }
//! ```
//!
//! Any bar with `timestamp > T` is excluded **before** any metric computation.
//! This is the raw-source boundary that makes T2-05 meaningful: poisoning a
//! future bar cannot alter the reconstruction because future bars are excluded
//! at the source layer, not the derived-feature layer.
//!
//! # Provenance artifact
//!
//! The output artifact carries a complete reconstruction provenance tuple:
//!
//! ```text
//! reconstruction_id        — unique ID for this reconstruction run
//! as_of                    — the historical timestamp T
//! universe_id              — identity of the instrument universe
//! data_source              — "YahooFinance/cache"
//! data_boundary_rule       — "bar.timestamp <= as_of"
//! source_dataset_hash      — SHA-256 of all bar timestamps used
//! clock_mode               — "REPLAY"
//! feature_pipeline_id      — "time002_reconstruct.v1"
//! accounting               — { n_complete, n_incomplete, n_error }
//! created_at               — wall-clock time of artifact generation
//!                            (MUST NOT participate in reconstructed state)
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run -p chronosentiment_adapter --bin time002_reconstruct -- \
//!   --as-of 2024-08-17T10:15:00Z \
//!   --universe datasets/universes/coralys_102_v1.json \
//!   --cache-dir product_validation/CS-P-006/snapshot/20260814T183851Z_100instrument/yahoo_cache \
//!   --output time_machine/reconstructions
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use chronosentiment_adapter::decision_support::enrichment_certify::{
    assess_from_bars_at_t, metrics_from_bars_at_t,
};
use chronosentiment_adapter::decision_support::forward_tick::instrument_id_for;
use chronosentiment_adapter::decision_support::observatory_prospective::latest_session_at_or_before;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::{YahooHistoricalBar, YahooProvider};
use chronosentiment_adapter::instrument::Instrument;
use chronosentiment_adapter::time_machine::clock::HistoricalClock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ─── Constants ────────────────────────────────────────────────────────────────

const FEATURE_PIPELINE_ID: &str = "time002_reconstruct.v1";
const DATA_SOURCE: &str = "YahooFinance/cache";
const DATA_BOUNDARY_RULE: &str = "bar.timestamp <= as_of";

// ─── Artifact types ───────────────────────────────────────────────────────────

/// Per-instrument reconstruction result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentReconstruction {
    pub ticker: String,
    pub source: String,
    /// Timestamp of the most recent bar used (≤ T). None if no bars ≤ T.
    pub source_bar_timestamp: Option<DateTime<Utc>>,
    /// Number of bars in the raw cache for this instrument.
    pub n_bars_total: usize,
    /// Number of bars with timestamp ≤ T (the only ones used).
    pub n_bars_at_t: usize,
    /// Number of bars excluded because timestamp > T (future-poison boundary).
    pub n_bars_excluded: usize,
    pub reference_price: Option<f64>,
    pub atr_14: Option<f64>,
    pub trend: Option<String>,
    pub momentum: Option<String>,
    pub volatility: Option<String>,
    pub tmv_complete: bool,
    /// COMPLETE | INCOMPLETE | ERROR
    pub status: String,
    pub error: Option<String>,
}

/// Accounting summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accounting {
    pub n_total: usize,
    pub n_complete: usize,
    pub n_incomplete: usize,
    pub n_error: usize,
}

/// Reconstruction provenance tuple.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionProvenance {
    pub reconstruction_id: String,
    pub as_of: DateTime<Utc>,
    pub universe_id: String,
    pub data_source: String,
    pub data_boundary_rule: String,
    pub source_dataset_hash: String,
    pub clock_mode: String,
    pub feature_pipeline_id: String,
    pub accounting: Accounting,
    /// Wall-clock time of artifact generation.
    /// MUST NOT participate in the reconstructed state.
    pub created_at: DateTime<Utc>,
}

/// The complete TIME-002 reconstruction artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionArtifact {
    pub provenance: ReconstructionProvenance,
    pub instruments: Vec<InstrumentReconstruction>,
}

// ─── Universe loading ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct UniverseFile {
    instruments: Vec<String>,
}

fn load_universe(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("cannot read universe file {}: {e}", path.display()))?;
    let universe: UniverseFile =
        serde_json::from_str(&raw).map_err(|e| format!("universe JSON parse error: {e}"))?;
    Ok(universe.instruments)
}

// ─── Source dataset hash ──────────────────────────────────────────────────────

/// Compute a deterministic SHA-256 hash over all bar timestamps used (≤ T).
/// This hash is part of the provenance tuple and allows downstream consumers
/// to verify that the same source data was used.
fn compute_source_hash(
    all_bars: &[(String, Vec<YahooHistoricalBar>)],
    as_of: DateTime<Utc>,
) -> String {
    let mut hasher = Sha256::new();
    // Sort by ticker for determinism.
    let mut sorted: Vec<&(String, Vec<YahooHistoricalBar>)> = all_bars.iter().collect();
    sorted.sort_by_key(|(ticker, _)| ticker.as_str());
    for (ticker, bars) in &sorted {
        hasher.update(ticker.as_bytes());
        hasher.update(b"|");
        // Only hash bars ≤ T (the boundary we enforce).
        let mut ts_at_t: Vec<i64> = bars
            .iter()
            .filter(|b| b.timestamp <= as_of.timestamp())
            .map(|b| b.timestamp)
            .collect();
        ts_at_t.sort_unstable();
        for ts in ts_at_t {
            hasher.update(ts.to_le_bytes());
        }
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

// ─── Reconstruction ID ────────────────────────────────────────────────────────

fn make_reconstruction_id(as_of: DateTime<Utc>, created_at: DateTime<Utc>) -> String {
    format!(
        "TIME002-{}-gen{}",
        as_of.format("%Y%m%dT%H%M%SZ"),
        created_at.format("%Y%m%dT%H%M%SZ")
    )
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    as_of: DateTime<Utc>,
    universe: PathBuf,
    cache_dir: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut as_of_str: Option<String> = None;
    let mut universe: Option<PathBuf> = None;
    let mut cache_dir: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--as-of" => {
                as_of_str = Some(args.next().ok_or("--as-of requires a value")?);
            }
            "--universe" => {
                universe = Some(PathBuf::from(
                    args.next().ok_or("--universe requires a value")?,
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

    let as_of_str = as_of_str.ok_or("--as-of is required")?;
    let as_of: DateTime<Utc> = as_of_str
        .parse()
        .map_err(|e| format!("--as-of must be RFC3339: {e}"))?;

    Ok(Args {
        as_of,
        universe: universe.ok_or("--universe is required")?,
        cache_dir: cache_dir.ok_or("--cache-dir is required")?,
        output: output.ok_or("--output is required")?,
    })
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // T2-03: Use HistoricalClock::replay(T) — never Utc::now() for the
    // reconstructed state. created_at is the only wall-clock value and it
    // must never participate in the reconstructed state.
    let clock = HistoricalClock::replay(args.as_of);
    let as_of = clock.now(); // always returns args.as_of — no wall-clock leakage

    // created_at records when the artifact was generated.
    // It is NOT part of the historical world.
    let created_at = Utc::now();

    println!("[time002] clock={clock}");
    println!("[time002] as_of={as_of}");
    println!("[time002] created_at={created_at} (artifact generation time — not historical state)");

    let tickers = load_universe(&args.universe)?;
    println!(
        "[time002] universe={} n_instruments={}",
        args.universe.display(),
        tickers.len()
    );

    // Set CHRONO_YAHOO_CACHE_DIR so YahooProvider reads from the historical cache.
    std::env::set_var("CHRONO_YAHOO_CACHE_DIR", &args.cache_dir);
    println!("[time002] cache_dir={}", args.cache_dir.display());

    let yahoo = YahooProvider::new();
    let mut instrument_results: Vec<InstrumentReconstruction> = Vec::new();
    let mut all_bars_for_hash: Vec<(String, Vec<YahooHistoricalBar>)> = Vec::new();

    for ticker in &tickers {
        // Build a minimal Instrument for the Yahoo provider.
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: uuid::Uuid::nil(),
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: as_of, // T2-03: use clock time, not Utc::now()
        };

        // Fetch all bars from cache.
        let all_bars = match yahoo
            .fetch_historical(&instrument, TimeRange::FiveYears)
            .await
        {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[time002] error ticker={ticker} error={e}");
                instrument_results.push(InstrumentReconstruction {
                    ticker: ticker.clone(),
                    source: DATA_SOURCE.to_string(),
                    source_bar_timestamp: None,
                    n_bars_total: 0,
                    n_bars_at_t: 0,
                    n_bars_excluded: 0,
                    reference_price: None,
                    atr_14: None,
                    trend: None,
                    momentum: None,
                    volatility: None,
                    tmv_complete: false,
                    status: "ERROR".to_string(),
                    error: Some(e.to_string()),
                });
                all_bars_for_hash.push((ticker.clone(), vec![]));
                continue;
            }
        };

        let n_bars_total = all_bars.len();

        // T2-01 + T2-04: Exclude ALL bars with timestamp > T.
        // This is the raw-source boundary. Future bars are excluded here,
        // before any metric computation. This is what makes T2-05 meaningful:
        // poisoning a future bar cannot alter the reconstruction because it
        // is excluded at this layer.
        let bars_at_t: Vec<YahooHistoricalBar> = all_bars
            .iter()
            .filter(|b| b.timestamp <= as_of.timestamp())
            .cloned()
            .collect();

        let n_bars_at_t = bars_at_t.len();
        let n_bars_excluded = n_bars_total - n_bars_at_t;

        // Record for provenance hash (uses bars_at_t only).
        all_bars_for_hash.push((ticker.clone(), bars_at_t.clone()));

        if bars_at_t.is_empty() {
            println!(
                "[time002] ticker={ticker} status=INCOMPLETE reason=no_bars_at_t \
                 n_total={n_bars_total} n_excluded={n_bars_excluded}"
            );
            instrument_results.push(InstrumentReconstruction {
                ticker: ticker.clone(),
                source: DATA_SOURCE.to_string(),
                source_bar_timestamp: None,
                n_bars_total,
                n_bars_at_t: 0,
                n_bars_excluded,
                reference_price: None,
                atr_14: None,
                trend: None,
                momentum: None,
                volatility: None,
                tmv_complete: false,
                status: "INCOMPLETE".to_string(),
                error: None,
            });
            continue;
        }

        // T2-02: All derived features computed exclusively from bars_at_t.
        // latest_session_at_or_before uses only bars_at_t — no future data.
        let t = latest_session_at_or_before(&bars_at_t, as_of).unwrap_or(as_of);

        // Source bar timestamp — most recent bar used (≤ T).
        let source_bar_timestamp = bars_at_t
            .iter()
            .filter(|b| b.timestamp <= t.timestamp())
            .last()
            .and_then(|b| Utc.timestamp_opt(b.timestamp, 0).single());

        // Reference price — last close ≤ T.
        let reference_price = bars_at_t
            .iter()
            .filter(|b| b.timestamp <= t.timestamp())
            .last()
            .and_then(|b| if b.close > 0.0 { Some(b.close) } else { None });

        // ATR-14 — computed from bars_at_t only.
        let instrument_id = instrument_id_for(ticker);
        let metrics = metrics_from_bars_at_t(&bars_at_t, t, instrument_id);
        let atr_14 = metrics.get_float("atr_14");

        // TMV assessment — computed from bars_at_t only.
        let (profile, _, _) = assess_from_bars_at_t(&bars_at_t, t, instrument_id);

        let trend = profile
            .assessments
            .iter()
            .find(|a| a.concept == chronosentiment_adapter::metrics::concepts::Concept::Trend)
            .map(|a| format!("{:?}", a.direction));
        let momentum = profile
            .assessments
            .iter()
            .find(|a| a.concept == chronosentiment_adapter::metrics::concepts::Concept::Momentum)
            .map(|a| format!("{:?}", a.direction));
        let volatility = profile
            .factor_status
            .iter()
            .find(|s| s.concept == chronosentiment_adapter::metrics::concepts::Concept::Volatility)
            .map(|s| format!("{:?}", s.availability));

        let tmv_complete = trend.is_some()
            && momentum.is_some()
            && volatility.is_some()
            && reference_price.is_some()
            && atr_14.is_some();

        // T2-07: Every instrument gets exactly one of COMPLETE / INCOMPLETE / ERROR.
        let status = if tmv_complete {
            "COMPLETE"
        } else {
            "INCOMPLETE"
        }
        .to_string();

        println!(
            "[time002] ticker={ticker} status={status} ref={reference_price:?} \
             atr_14={atr_14:?} n_at_t={n_bars_at_t} n_excluded={n_bars_excluded}"
        );

        instrument_results.push(InstrumentReconstruction {
            ticker: ticker.clone(),
            source: DATA_SOURCE.to_string(),
            source_bar_timestamp,
            n_bars_total,
            n_bars_at_t,
            n_bars_excluded,
            reference_price,
            atr_14,
            trend,
            momentum,
            volatility,
            tmv_complete,
            status,
            error: None,
        });
    }

    // ── Accounting (T2-07) ────────────────────────────────────────────────────

    let n_complete = instrument_results
        .iter()
        .filter(|i| i.status == "COMPLETE")
        .count();
    let n_incomplete = instrument_results
        .iter()
        .filter(|i| i.status == "INCOMPLETE")
        .count();
    let n_error = instrument_results
        .iter()
        .filter(|i| i.status == "ERROR")
        .count();
    let n_total = instrument_results.len();

    println!(
        "[time002] accounting: total={n_total} complete={n_complete} \
         incomplete={n_incomplete} error={n_error}"
    );

    // Verify accounting is exhaustive (T2-07).
    assert_eq!(
        n_complete + n_incomplete + n_error,
        n_total,
        "T2-07 FAIL: accounting is not exhaustive"
    );

    // ── Provenance ────────────────────────────────────────────────────────────

    let source_dataset_hash = compute_source_hash(&all_bars_for_hash, as_of);
    let reconstruction_id = make_reconstruction_id(as_of, created_at);

    let provenance = ReconstructionProvenance {
        reconstruction_id: reconstruction_id.clone(),
        as_of,
        universe_id: args.universe.to_string_lossy().to_string(),
        data_source: DATA_SOURCE.to_string(),
        data_boundary_rule: DATA_BOUNDARY_RULE.to_string(),
        source_dataset_hash: source_dataset_hash.clone(),
        clock_mode: clock.mode_label().to_string(),
        feature_pipeline_id: FEATURE_PIPELINE_ID.to_string(),
        accounting: Accounting {
            n_total,
            n_complete,
            n_incomplete,
            n_error,
        },
        created_at,
    };

    println!("[time002] reconstruction_id={reconstruction_id}");
    println!("[time002] source_dataset_hash={source_dataset_hash}");

    let artifact = ReconstructionArtifact {
        provenance,
        instruments: instrument_results,
    };

    // ── Write artifact ────────────────────────────────────────────────────────

    fs::create_dir_all(&args.output)?;

    let filename = format!("TIME002-{}.json", as_of.format("%Y%m%dT%H%M%SZ"));
    let artifact_path = args.output.join(&filename);
    let latest_path = args.output.join("latest.json");

    let json = serde_json::to_string_pretty(&artifact)?;
    fs::write(&artifact_path, &json)?;
    fs::write(&latest_path, &json)?;

    println!("[time002] artifact written: {}", artifact_path.display());
    println!("[time002] latest.json updated: {}", latest_path.display());
    println!("[time002] DONE complete={n_complete} incomplete={n_incomplete} error={n_error}");

    Ok(())
}
