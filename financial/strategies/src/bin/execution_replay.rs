use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use coralys_ecology::models::CognitionGeometry;
use coralys_ecology::models::MemoryState;

#[derive(Parser, Debug)]
#[command(author, version, about = "Phase 5B Execution Replay with Perturbations")]
struct Args {
    #[arg(short, long)]
    substrate_file: String,

    #[arg(long, default_value = "0")]
    latency_ms: u64,

    #[arg(long, default_value = "0.0")]
    missed_fill_prob: f64,
}

#[derive(Deserialize)]
struct ChronologyEvent {
    price: Option<f64>,
    close: Option<f64>,
}

#[derive(Serialize)]
struct ExecutionSummary {
    total_signals: usize,
    attempted_fills: usize,
    missed_fills: usize,
    successful_fills: usize,
    fill_rate: f64,
    effective_slippage_bps: f64,
}

fn deterministic_rand(tick: u64, salt: u64) -> f64 {
    let hash_int = tick.wrapping_mul(1103515245).wrapping_add(salt).wrapping_add(12345);
    (hash_int % 10000) as f64 / 10000.0
}

fn main() {
    let args = Args::parse();

    let file = File::open(&args.substrate_file).unwrap();
    let reader = BufReader::new(file);
    
    let mut prices = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        let event: ChronologyEvent = serde_json::from_str(&line).unwrap_or_else(|e| {
            panic!("Failed to parse line '{}': {}", line, e);
        });
        let p = event.price.unwrap_or_else(|| event.close.unwrap());
        prices.push(p);
    }

    // We use event_reset cognition as our "Strategy Signal Generator"
    // Every time occupancy resets, we generate a trade signal.
    let geometry = CognitionGeometry::EventReset { drop_threshold_pct: 0.005 };
    let mut state = MemoryState::new(geometry);
    
    let mut total_signals = 0;
    let mut missed_fills = 0;
    let mut successful_fills = 0;
    let mut total_slippage_bps = 0.0;
    
    // Estimate local volatility to scale latency impact
    let mut rolling_returns = Vec::new();
    for (i, &price) in prices.iter().enumerate() {
        if i > 0 {
            let ret = (price - prices[i-1]).abs() / prices[i-1];
            rolling_returns.push(ret);
            if rolling_returns.len() > 20 {
                rolling_returns.remove(0);
            }
        }
        
        let local_vol = if rolling_returns.is_empty() {
            0.001
        } else {
            rolling_returns.iter().sum::<f64>() / rolling_returns.len() as f64
        };

        let sma = if rolling_returns.len() >= 20 {
            prices[i.saturating_sub(20)..=i].iter().sum::<f64>() / 21.0
        } else {
            price
        };
        
        let signal = i >= 20 && ((price > sma && prices[i-1] <= sma) || (price < sma && prices[i-1] >= sma));
        
        if signal {
            total_signals += 1;
            
            // Perturbation Model
            // 1. Missed Fill Check
            let mut rand_val = deterministic_rand(i as u64, 1);
            
            // High volatility + high latency = higher chance of missing the fill
            let dynamic_miss_prob = args.missed_fill_prob + (args.latency_ms as f64 / 1000.0) * local_vol * 100.0;
            
            if rand_val < dynamic_miss_prob {
                missed_fills += 1;
            } else {
                successful_fills += 1;
                // 2. Slippage Calculation
                // Base slippage + latency induced slippage
                let latency_slip_bps = (args.latency_ms as f64) * local_vol * 10000.0; // scales with latency and vol
                total_slippage_bps += 0.5 + latency_slip_bps; // 0.5 bps base spread
            }
        }
    }
    
    let fill_rate = if total_signals > 0 {
        successful_fills as f64 / total_signals as f64
    } else {
        0.0
    };
    
    let avg_slippage = if successful_fills > 0 {
        total_slippage_bps / successful_fills as f64
    } else {
        0.0
    };

    let summary = ExecutionSummary {
        total_signals,
        attempted_fills: total_signals,
        missed_fills,
        successful_fills,
        fill_rate,
        effective_slippage_bps: avg_slippage,
    };

    let out_json = serde_json::to_string_pretty(&summary).unwrap();
    println!("{}", out_json);
}
