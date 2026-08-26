use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    substrate_file: String,

    #[arg(long, default_value_t = 0)]
    latency_ms: u64,

    #[arg(long, default_value_t = 0.0)]
    missed_fill_prob: f64,
}

#[derive(Deserialize, Debug)]
struct SubstrateTick {
    symbol: String,
    timestamp: u64,
    price: f64,
    volume: f64,
    is_buyer_maker: bool,
}

#[derive(Serialize)]
struct ExecutionResult {
    attempted_orders: usize,
    filled_orders: usize,
    fill_ratio: f64,
    total_entry_drift_bps: f64,
    total_opportunity_cost_bps: f64,
}

// Simple deterministic PRNG
fn deterministic_rand(seed: u64, counter: u64) -> f64 {
    let mut state = seed.wrapping_add(counter).wrapping_add(0x9E3779B97F4A7C15);
    state ^= state >> 30;
    state = state.wrapping_mul(0xBF58476D1CE4E5B9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94D049BB133111EB);
    state ^= state >> 31;
    (state as f64) / (u64::MAX as f64)
}

fn main() {
    let args = Args::parse();

    let file = File::open(&args.substrate_file).expect("Failed to open substrate file");
    let reader = BufReader::new(file);

    let mut prices = Vec::new();
    let mut timestamps = Vec::new();

    for line in reader.lines() {
        if let Ok(l) = line {
            if l.trim().is_empty() {
                continue;
            }
            if let Ok(tick) = serde_json::from_str::<SubstrateTick>(&l) {
                prices.push(tick.price);
                timestamps.push(tick.timestamp);
            }
        }
    }

    // We want deterministic orders at T+30m, T+90m, T+150m, T+210m, T+270m, T+330m.
    // Assuming ticks are separated by 15s (4 ticks per minute).
    // 30m = 120 ticks, 90m = 360 ticks, 150m = 600 ticks, 210m = 840 ticks, 270m = 1080 ticks, 330m = 1320 ticks.
    let order_tick_indices = vec![120, 360, 600, 840, 1080, 1320];

    let mut attempted_orders = 0;
    let mut filled_orders = 0;
    let mut total_entry_drift_bps = 0.0;
    let mut total_opportunity_cost_bps = 0.0;

    for &idx in &order_tick_indices {
        if idx < prices.len() {
            attempted_orders += 1;

            // Expected / Reference price is the price precisely at the tick the order is emitted.
            let reference_price = prices[idx];

            // Apply delay: 50ms latency doesn't strictly jump multiple 15s ticks unless there is massive queue delay.
            // Let's model queue delay deterministically: latency_ms / 10 is the number of ticks we slip.
            // Example: +5ms => 0 ticks slip. +50ms => 5 ticks slip.
            let ticks_to_slip = (args.latency_ms / 10) as usize;

            // Missed fill logic
            let rand_val = deterministic_rand(timestamps[idx], args.latency_ms);
            if rand_val < args.missed_fill_prob {
                // Missed fill!
                continue;
            }

            // It gets filled, but delayed by `ticks_to_slip`
            let mut fill_idx = idx + ticks_to_slip;
            if fill_idx >= prices.len() {
                fill_idx = prices.len() - 1;
            }

            let actual_fill_price = prices[fill_idx];
            filled_orders += 1;

            // Entry drift = absolute difference between expected and actual
            let drift = (actual_fill_price - reference_price).abs();
            let drift_bps = (drift / reference_price) * 10000.0;
            total_entry_drift_bps += drift_bps;

            // Opportunity loss: actual fill vs reference
            let opp_loss = (actual_fill_price - reference_price).abs();
            let opp_loss_bps = (opp_loss / reference_price) * 10000.0;
            total_opportunity_cost_bps += opp_loss_bps;
        }
    }

    let fill_ratio = if attempted_orders > 0 {
        filled_orders as f64 / attempted_orders as f64
    } else {
        0.0
    };

    let res = ExecutionResult {
        attempted_orders,
        filled_orders,
        fill_ratio,
        total_entry_drift_bps,
        total_opportunity_cost_bps,
    };

    let out = serde_json::to_string(&res).unwrap();
    println!("{}", out);
}
