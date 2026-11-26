//! # DEPRECATED / NON-AUTHORITATIVE ARCHIVED COMPONENT
//!
//! **STATUS: QUARANTINED / HISTORICAL REFERENCE ONLY**
//!
//! - NOT AN ACTIVE RP-615 COMPONENT
//! - NOT AN EXECUTION AUTHORITY
//! - NOT A PROSPECTIVE VALIDATION PATH
//!
//! ### Architectural Disposition & Migration:
//! This legacy daemon implemented Yahoo Finance polling (`query1.finance.yahoo.com`),
//! which has been superseded and deprecated.
//!
//! The single authoritative prospective crypto shadow validation pipeline is:
//! - **Daemon:** `scripts/run_rp615_crypto_shadow_daemon.py` (Binance live feed)
//! - **Telemetry:** `live_capture/shadow_validation/rp615_crypto_shadow_telemetry.json`
//! - **Metrics:** `scripts/emit_rp615_crypto_live_metrics.py`
//!
//! Do not compile or deploy this file in production environments.
//!
//! --- Historical Source Below ---

use chronosentiment_adapter::product::{DecisionCardViewModel, DecisionTape, ResolutionState};
use chronosentiment_adapter::decision_support::{BarObservation, ConfigATrajectoryInterpreter};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

const CRYPTO_UNIVERSE: [&str; 8] = [
    "BTC-USD",
    "ETH-USD",
    "BNB-USD",
    "XRP-USD",
    "SOL-USD",
    "ADA-USD",
    "LINK-USD",
    "LTC-USD",
];

const POLL_INTERVAL_SECS: u64 = 60;

fn fetch_bars_yahoo(symbol: &str, interval: &str, range: &str) -> Vec<BarObservation> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval={}",
        symbol, range, interval
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0")
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let resp = match client.get(&url).send() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let json: Value = match resp.json() {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let result = &json["chart"]["result"][0];
    let timestamps = match result["timestamp"].as_array() {
        Some(ts) => ts,
        None => return Vec::new(),
    };

    let quote = &result["indicators"]["quote"][0];
    let opens = quote["open"].as_array();
    let highs = quote["high"].as_array();
    let lows = quote["low"].as_array();
    let closes = quote["close"].as_array();
    let volumes = quote["volume"].as_array();

    let mut bars = Vec::new();

    for i in 0..timestamps.len() {
        let ts = match timestamps[i].as_i64() {
            Some(t) => t,
            None => continue,
        };

        let open = opens.and_then(|a| a.get(i)).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let high = highs.and_then(|a| a.get(i)).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let low = lows.and_then(|a| a.get(i)).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let close = closes.and_then(|a| a.get(i)).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let volume = volumes.and_then(|a| a.get(i)).and_then(|v| v.as_f64()).unwrap_or(0.0);

        if close == 0.0 {
            continue;
        }

        let dt_str = chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

        bars.push(BarObservation {
            timestamp: ts,
            datetime: dt_str,
            open,
            high,
            low,
            close,
            volume,
        });
    }

    bars
}

fn run_single_pass(interpreter: &ConfigATrajectoryInterpreter) {
    let now_utc = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    println!("[{}] Executing Rust Native RP-601 Continuous Acquisition Pass...", now_utc);

    let mut cards = Vec::new();
    let mut replay_tapes = Vec::new();

    for &symbol in &CRYPTO_UNIVERSE {
        let bars_15m = fetch_bars_yahoo(symbol, "15m", "7d");
        thread::sleep(Duration::from_millis(150));
        let bars_30m = fetch_bars_yahoo(symbol, "30m", "14d");
        thread::sleep(Duration::from_millis(150));
        let bars_1d = fetch_bars_yahoo(symbol, "1d", "60d");
        thread::sleep(Duration::from_millis(150));

        // Evaluate latest card
        let eval_15m = interpreter.evaluate_stream(&bars_15m);
        let eval_30m = interpreter.evaluate_stream(&bars_30m);
        let eval_1d = interpreter.evaluate_stream(&bars_1d);

        let latest_price = bars_15m.last().map(|b| b.close).unwrap_or(0.0);
        let ts_now = chrono::Utc::now().timestamp();

        let card = DecisionCardViewModel::create_canonical(
            symbol.to_string(),
            ts_now,
            latest_price,
            ResolutionState { label: eval_15m.label, score: eval_15m.score },
            ResolutionState { label: eval_30m.label, score: eval_30m.score },
            ResolutionState { label: eval_1d.label, score: eval_1d.score },
            Some("LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO".into()),
            false,
        );

        cards.push(card);

        // Build historical replay event sequence natively in Rust
        let mut events = Vec::new();
        for offset in (0..8).rev() {
            let offset_bars_15 = if offset * 2 < bars_15m.len() {
                &bars_15m[..bars_15m.len() - offset * 2]
            } else {
                &bars_15m[..]
            };
            let offset_bars_30 = if offset < bars_30m.len() {
                &bars_30m[..bars_30m.len() - offset]
            } else {
                &bars_30m[..]
            };

            let ev_15 = interpreter.evaluate_stream(offset_bars_15);
            let ev_30 = interpreter.evaluate_stream(offset_bars_30);
            let ev_1d = interpreter.evaluate_stream(&bars_1d);

            let price_t = offset_bars_15.last().map(|b| b.close).unwrap_or(0.0);
            let ts_t = offset_bars_15.last().map(|b| b.timestamp).unwrap_or(ts_now);

            let evt_card = DecisionCardViewModel::create_canonical(
                symbol.to_string(),
                ts_t,
                price_t,
                ResolutionState { label: ev_15.label, score: ev_15.score },
                ResolutionState { label: ev_30.label, score: ev_30.score },
                ResolutionState { label: ev_1d.label, score: ev_1d.score },
                Some("LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO".into()),
                false,
            );
            events.push(evt_card);
        }

        let symbol_tape = DecisionTape::create_replay_tape(
            symbol.to_string(),
            "LIVE_CONTINUOUS_POLLING_DAEMON".into(),
            "v0.1.0 (FROZEN)".into(),
            "Config A (FROZEN)".into(),
            events,
        );
        replay_tapes.push(symbol_tape);
    }

    let mut tape = DecisionTape::new(
        format!("tape_crypto_live_{}", chrono::Utc::now().format("%Y%m%dT%H%M%SZ")),
        None,
        Some("LIQUID_CRYPTO_8".into()),
        "LIVE_CONTINUOUS_POLLING_DAEMON".into(),
        Some("LIVE — EXPERIMENTAL / UNVALIDATED FOR CRYPTO".into()),
        "v0.1.0 (FROZEN)".into(),
        "Config A (FROZEN)".into(),
        cards.clone(),
    );
    tape.tapes = Some(replay_tapes);

    let json_bytes = match serde_json::to_string_pretty(&tape) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize tape: {}", e);
            return;
        }
    };

    let root_path = PathBuf::from("/Users/nikhil/ChronoSentiment_MEGA_FINAL/rp601_live_crypto_decision_tapes.json");
    let ui_data_path = PathBuf::from("/Users/nikhil/ChronoSentiment_MEGA_FINAL/services/ui/src/data/rp601_live_crypto_decision_tapes.json");
    let ui_public_path = PathBuf::from("/Users/nikhil/ChronoSentiment_MEGA_FINAL/services/ui/public/rp601_live_crypto_decision_tapes.json");

    for p in &[root_path, ui_data_path, ui_public_path] {
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut f) = File::create(p) {
            let _ = f.write_all(json_bytes.as_bytes());
        }
    }

    let btc = cards.iter().find(|c| c.instrument == "BTC-USD");
    if let Some(c) = btc {
        println!(
            "[{}] Rust Engine Pass Complete. Updated {} cards & replay tapes. BTC-USD: ${:.2} | Decision: {}",
            now_utc,
            cards.len(),
            c.current_price,
            c.decision_state
        );
    }
}

fn main() {
    println!("Starting ChronoSentiment Rust Native Crypto Live Daemon (Interval: {}s)...", POLL_INTERVAL_SECS);
    let interpreter = ConfigATrajectoryInterpreter::new();

    loop {
        run_single_pass(&interpreter);
        thread::sleep(Duration::from_secs(POLL_INTERVAL_SECS));
    }
}
