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
struct ArchetypeOutput {
    state_stream: Vec<i32>,
}

#[derive(Serialize)]
struct SimulationResult {
    portfolio_null_observer: ArchetypeOutput,
    signal_null_observer: ArchetypeOutput,
    twap: ArchetypeOutput,
    breakout: ArchetypeOutput,
    momentum: ArchetypeOutput,
    mean_reversion: ArchetypeOutput,
}

fn deterministic_rand(seed: u64, counter: u64) -> f64 {
    let mut state = seed.wrapping_add(counter).wrapping_add(0x9E3779B97F4A7C15);
    state ^= state >> 30;
    state = state.wrapping_mul(0xBF58476D1CE4E5B9);
    state ^= state >> 27;
    state = state.wrapping_mul(0x94D049BB133111EB);
    state ^= state >> 31;
    (state as f64) / (u64::MAX as f64)
}

struct ArchetypeState {
    pub out: ArchetypeOutput,
    pub pending_signal: Option<usize>,
    pub current_pos: i32,
    pub exit_tick: Option<usize>,
}

impl ArchetypeState {
    fn new() -> Self {
        Self {
            out: ArchetypeOutput {
                state_stream: vec![],
            },
            pending_signal: None,
            current_pos: 0,
            exit_tick: None,
        }
    }

    fn process_queue(&mut self, idx: usize, timestamps: &[u64], args: &Args, ticks_to_slip: usize) {
        // Handle Exits first
        if let Some(exit_t) = self.exit_tick {
            if idx >= exit_t {
                self.current_pos = 0;
                self.exit_tick = None;
            }
        }

        // Handle queue fills
        if let Some(signal_idx) = self.pending_signal {
            if idx >= signal_idx + ticks_to_slip {
                let rand_val = deterministic_rand(timestamps[idx], args.latency_ms + idx as u64);
                if rand_val >= args.missed_fill_prob {
                    // Enter Long position for 100 ticks upon fill
                    if self.current_pos == 0 {
                        self.current_pos = 1;
                        self.exit_tick = Some(idx + 100);
                    }
                }
                self.pending_signal = None;
            }
        }

        self.out.state_stream.push(self.current_pos);
    }
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

    let ticks_to_slip = (args.latency_ms / 10) as usize;

    let mut port_null_state = ArchetypeState::new();
    let mut sig_null_state = ArchetypeState::new();
    let mut twap_state = ArchetypeState::new();
    let mut brk_state = ArchetypeState::new();
    let mut mom_state = ArchetypeState::new();
    let mut mr_state = ArchetypeState::new();

    let twap_intervals = vec![120, 360, 600, 840, 1080, 1320];

    for i in 0..prices.len() {
        // Portfolio Null Observer doesn't even generate signals
        port_null_state.process_queue(i, &timestamps, &args, ticks_to_slip);

        // Signal Null Observer
        if i > 0 && i % 50 == 0 {
            // Generates signal but never executes
            // Wait, we just don't set pending_signal
        }
        sig_null_state.process_queue(i, &timestamps, &args, ticks_to_slip);

        twap_state.process_queue(i, &timestamps, &args, ticks_to_slip);
        brk_state.process_queue(i, &timestamps, &args, ticks_to_slip);
        mom_state.process_queue(i, &timestamps, &args, ticks_to_slip);
        mr_state.process_queue(i, &timestamps, &args, ticks_to_slip);

        if twap_intervals.contains(&i) {
            if twap_state.pending_signal.is_none() && twap_state.current_pos == 0 {
                twap_state.pending_signal = Some(i);
            }
        }

        if i >= 50 {
            let local_high = prices[i - 50..i].iter().cloned().fold(f64::NAN, f64::max);
            if prices[i] > local_high
                && brk_state.pending_signal.is_none()
                && brk_state.current_pos == 0
            {
                brk_state.pending_signal = Some(i);
            }
        }

        if i >= 3 {
            if prices[i] > prices[i - 1]
                && prices[i - 1] > prices[i - 2]
                && prices[i - 2] > prices[i - 3]
            {
                if mom_state.pending_signal.is_none() && mom_state.current_pos == 0 {
                    mom_state.pending_signal = Some(i);
                }
            }
        }

        if i >= 20 {
            let sma = prices[i - 20..=i].iter().sum::<f64>() / 21.0;
            if prices[i] < sma * 0.998 {
                if mr_state.pending_signal.is_none() && mr_state.current_pos == 0 {
                    mr_state.pending_signal = Some(i);
                }
            }
        }
    }

    let res = SimulationResult {
        portfolio_null_observer: port_null_state.out,
        signal_null_observer: sig_null_state.out,
        twap: twap_state.out,
        breakout: brk_state.out,
        momentum: mom_state.out,
        mean_reversion: mr_state.out,
    };

    println!("{}", serde_json::to_string(&res).unwrap());
}
