use serde_json::Value;
use std::{process::Command, thread, time::Duration};

use chronosentiment_core::ga::*;
use chronosentiment_core::{from_real, Candle, MarketEvent, MarketEventType, Side, PRICE_SCALE};

fn fetch_live_candles(symbol: &str, interval: &str, n: usize) -> Vec<Value> {
    let script_path = format!(
        "{}/scripts/fetch_candles.py",
        std::env::current_dir().unwrap().display()
    );

    let output = Command::new("python3")
        .arg(script_path)
        .arg(symbol)
        .arg(interval)
        .arg(n.to_string())
        .output()
        .expect("failed to fetch candles");

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout).unwrap_or_default()
}

fn json_to_candles(data: Vec<Value>) -> Vec<Candle> {
    data.into_iter()
        .map(|c| Candle {
            timestamp: c["timestamp"].as_u64().unwrap_or(0),
            open: from_real(c["open"].as_f64().unwrap_or(0.0)),
            high: from_real(c["high"].as_f64().unwrap_or(0.0)),
            low: from_real(c["low"].as_f64().unwrap_or(0.0)),
            close: from_real(c["close"].as_f64().unwrap_or(0.0)),
            volume: 1000,
        })
        .collect()
}

fn convert_candles_to_events(candles: &[Candle]) -> Vec<MarketEvent> {
    let mut events = Vec::new();

    for c in candles {
        let prices = [c.open, c.high, c.low, c.close];

        for (i, &px) in prices.iter().enumerate() {
            events.push(MarketEvent {
                subtype: MarketEventType::Trade,
                price: px,
                quantity: 100,
                side: Some(Side::Buy),
                exchange_ts: c.timestamp + i as u64,
            });
        }
    }

    events
}

fn main() {
    println!("🚀 LIVE ENGINE (CONSENSUS + EXECUTION)");

    let symbols = vec![
        "HDFCBANK.NS",
        "RELIANCE.NS",
        "INFY.NS",
        "ICICIBANK.NS",
        "SBIN.NS",
        "IDEA.NS",
    ];

    loop {
        println!("\n================ NEW ITERATION ================\n");

        for symbol in &symbols {
            println!("🔍 {}", symbol);

            let raw = fetch_live_candles(symbol, "5m", 200);
            if raw.is_empty() {
                println!("❌ No data");
                continue;
            }

            let candles = json_to_candles(raw);
            let events = convert_candles_to_events(&candles);

            let scenario = ScenarioPair {
                name: "LIVE",
                signal_symbol: symbol,
                execution_symbol: symbol,
                signal: &events,
                execution: &events,
            };

            let config = GaConfig::default();

            // 🔥 YOUR ELITE STRATEGIES (replace later with GA output)
            let elites = vec![Strategy::default()];

            // 🔥 CONSENSUS ENGINE
            let report = compute_consensus_alpha(&elites, &scenario, &config);

            if report.top_signals.is_empty() {
                println!("⚠️ No signals");
                continue;
            }

            let top = &report.top_signals[0];

            let idx = top.signal_idx;
            let strength = top.alpha_score.clamp(0.0, 1.0);

            let is_long = top.conviction >= 0.0;

            // 🔥 FAKE minimal conviction (only required fields)
            let mut conviction = ConvictionOutcome::default();

            // minimal required overrides
            conviction.bullish_score = if is_long { 1.0 } else { 0.0 };
            conviction.bearish_score = if is_long { 0.0 } else { 1.0 };
            conviction.conviction_score = strength;
            conviction.edge_weight = strength;
            conviction.is_valid = true;

            // 🔥 EXECUTION SIMULATION (YOUR CORE ENGINE)
            if let Some(outcome) = ga_simulate_round_trip_at_cursor(
                &elites[0],
                &events,
                &events,
                &config,
                idx,
                0,
                &conviction,
                is_long,
                strength,
            ) {
                let entry = candles.last().unwrap().close as f64 / PRICE_SCALE as f64;

                println!(
                    "{} → {:?} | entry {:.2} | pnl {:.6} | e_score {:.3}",
                    symbol,
                    outcome.side,
                    entry,
                    outcome.pnl,
                    outcome.e_score
                );
            }
        }

        println!("\n⏳ Waiting 30 sec...\n");
        thread::sleep(Duration::from_secs(30));
    }
}