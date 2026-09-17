//! INTRA-001 — Intraday Bar Fetch (INTRADAY-001)
//!
//! Fetches and caches 1-minute, 5-minute, and 15-minute OHLCV bars from Yahoo Finance
//! for all instruments in the universe. This is the data acquisition step
//! for the INTRADAY-001 parallel observation experiment.
//!
//! ## What this does
//!
//! - Fetches `interval=5m` bars → `CHRONO_YAHOO_CACHE_5M_DIR/<TICKER.NS>.json`
//! - Fetches `interval=1m` bars → `CHRONO_YAHOO_CACHE_1M_DIR/<TICKER.NS>.json`
//! - Incremental: only fetches bars after the last stored timestamp
//! - Does NOT modify the daily Yahoo cache (`CHRONO_YAHOO_CACHE_DIR`)
//! - Does NOT modify TIME-009 ledger, observations, or daily observer
//!
//! ## Yahoo Finance intraday retention
//!
//! - `interval=1m`: last ~7 days
//! - `interval=5m`: last ~60 days
//!
//! ## Usage
//!
//! ```bash
//! CHRONO_YAHOO_CACHE_1M_DIR=intraday_capture/yahoo_cache_1m \
//! CHRONO_YAHOO_CACHE_5M_DIR=intraday_capture/yahoo_cache_5m \
//!   cargo run -p chronosentiment_adapter --bin intraday001_fetch -- \
//!   --universe datasets/universes/coralys_102_v1.json
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::ingestion::yahoo::{IntradayInterval, fetch_intraday_bars};
use chronosentiment_adapter::instrument::Instrument;
use serde::Deserialize;

const PRODUCER: &str = "intraday001_fetch.v1";

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

struct Args {
    universe: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut universe = PathBuf::from("datasets/universes/coralys_102_v1.json");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--universe" => {
                i += 1;
                universe = PathBuf::from(&args[i]);
            }
            _ => {}
        }
        i += 1;
    }
    Ok(Args { universe })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let tickers = load_universe(&args.universe)?;

    println!("[intra001] producer={PRODUCER}");
    println!("[intra001] universe={} n_instruments={}", args.universe.display(), tickers.len());

    let cache_1m = std::env::var("CHRONO_YAHOO_CACHE_1M_DIR").unwrap_or_default();
    let cache_5m = std::env::var("CHRONO_YAHOO_CACHE_5M_DIR").unwrap_or_default();
    let cache_15m = std::env::var("CHRONO_YAHOO_CACHE_15M_DIR").unwrap_or_default();

    if cache_1m.is_empty() {
        eprintln!("[intra001] WARNING: CHRONO_YAHOO_CACHE_1M_DIR not set — 1m bars will not be cached");
    } else {
        println!("[intra001] cache_1m={cache_1m}");
    }
    if cache_5m.is_empty() {
        eprintln!("[intra001] WARNING: CHRONO_YAHOO_CACHE_5M_DIR not set — 5m bars will not be cached");
    } else {
        println!("[intra001] cache_5m={cache_5m}");
    }
    if cache_15m.is_empty() {
        eprintln!("[intra001] WARNING: CHRONO_YAHOO_CACHE_15M_DIR not set — 15m bars will not be cached");
    } else {
        println!("[intra001] cache_15m={cache_15m}");
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36")
        .build()?;

    let mut n_1m_ok = 0usize;
    let mut n_1m_err = 0usize;
    let mut n_5m_ok = 0usize;
    let mut n_5m_err = 0usize;
    let mut n_15m_ok = 0usize;
    let mut n_15m_err = 0usize;

    for ticker in &tickers {
        // Fetch 5-minute bars
        if !cache_5m.is_empty() {
            match fetch_intraday_bars(&client, ticker, IntradayInterval::FiveMinute).await {
                Ok(bars) => {
                    println!("[intra001] 5m ticker={ticker} n_bars={}", bars.len());
                    n_5m_ok += 1;
                }
                Err(e) => {
                    eprintln!("[intra001] 5m ERROR ticker={ticker} error={e}");
                    n_5m_err += 1;
                }
            }
        }

        // Fetch 15-minute bars
        if !cache_15m.is_empty() {
            match fetch_intraday_bars(&client, ticker, IntradayInterval::FifteenMinute).await {
                Ok(bars) => {
                    println!("[intra001] 15m ticker={ticker} n_bars={}", bars.len());
                    n_15m_ok += 1;
                }
                Err(e) => {
                    eprintln!("[intra001] 15m ERROR ticker={ticker} error={e}");
                    n_15m_err += 1;
                }
            }
        }

        // Fetch 1-minute bars
        if !cache_1m.is_empty() {
            match fetch_intraday_bars(&client, ticker, IntradayInterval::OneMinute).await {
                Ok(bars) => {
                    println!("[intra001] 1m ticker={ticker} n_bars={}", bars.len());
                    n_1m_ok += 1;
                }
                Err(e) => {
                    eprintln!("[intra001] 1m ERROR ticker={ticker} error={e}");
                    n_1m_err += 1;
                }
            }
        }

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    println!("[intra001] result=OK");
    println!("[intra001] n_5m_ok={n_5m_ok} n_5m_err={n_5m_err}");
    println!("[intra001] n_15m_ok={n_15m_ok} n_15m_err={n_15m_err}");
    println!("[intra001] n_1m_ok={n_1m_ok} n_1m_err={n_1m_err}");

    Ok(())
}