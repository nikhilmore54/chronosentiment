use chrono::Local;
use chronosentiment_core::ga::{
    evaluate_consensus_status, load_elite_strategies, update_paper_registry, GaConfig, PaperRegistry, SignalType, TradeIntent, MarketRegime,
};
use chronosentiment_core::market_adapter::Candle;
use chronosentiment_core::TradeRecommendation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::fs::OpenOptions;

const PRICE_SCALE: f64 = 10000.0;
/// Rolling coherence window cap per symbol (long-lived daemon sessions).
const MAX_HISTORY_BARS: usize = 512;

#[derive(Debug, Deserialize, Serialize)]
struct SymbolicCandle {
    pub symbol: String,
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl SymbolicCandle {
    fn to_core_candle(&self) -> Candle {
        let sc = if self.close > 1_000_000.0 { 1.0 } else { PRICE_SCALE };
        Candle {
            timestamp: self.timestamp,
            open: (self.open * sc) as u64,
            high: (self.high * sc) as u64,
            low: (self.low * sc) as u64,
            close: (self.close * sc) as u64,
            volume: self.volume as u64,
        }
    }
}

fn main() {
    let source_type = std::env::var("SOURCE_TYPE").unwrap_or_else(|_| "LIVE".to_string());
    let is_authentic = source_type == "LIVE";
    let generation = std::env::var("REPLAY_GENERATION").unwrap_or_else(|_| "0".to_string()).parse::<u32>().unwrap_or(0);

    println!("\n🚀 [LIVE_OBSERVATORY]");
    println!("mode=continuous");
    println!("feed=stdin_json");
    println!("symbols=BTC-USD,ETH-USD,SOL-USD");
    println!("provenance={}, authentic={}, gen={}", source_type, is_authentic, generation);
    println!("{}", "=".repeat(80));

    let elite_path = "core/elite/latest.json";
    let elite_strategies = load_elite_strategies(elite_path);
    if elite_strategies.is_empty() {
        panic!("❌ ERROR: No elite strategies found at {}.", elite_path);
    }
    println!(
        "✅ OBSERVATORY: Loaded {} environmental baseline genomes.",
        elite_strategies.len()
    );

    let config = GaConfig::default();
    let mut paper = PaperRegistry::default();
    paper.max_concurrent = 10;

    let mut history_pipes: HashMap<String, Vec<Candle>> = HashMap::new();
    let mut last_signals: HashMap<String, SignalType> = HashMap::new();
    let mut consistency_counts: HashMap<String, usize> = HashMap::new();
    let mut prev_micro_exp_moves: HashMap<String, f64> = HashMap::new();
    let mut symbol_update_counts: HashMap<String, usize> = HashMap::new();
    let mut next_rec_id: u64 = 1;

    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut total_processed = 0;

    println!("📡 Listening for continuous environmental telemetry via stdin...");
    println!("   Awaiting synchronized JSON batches...\n");
    println!("[OBSERVATORY_READY] long_lived=true max_history={}", MAX_HISTORY_BARS);
    io::stdout().flush().unwrap();

    for line_result in reader.lines() {
        let raw = match line_result {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[OBSERVATORY_ERROR] stdin read error: {}", e);
                break;
            }
        };

        if raw.trim().is_empty() {
            continue;
        }

        let batch: Vec<SymbolicCandle> = match serde_json::from_str(&raw) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "[OBSERVATORY_WARNING] JSON parse error: {} | raw: {}",
                    e, raw
                );
                continue;
            }
        };

        let mut symbols_updated = Vec::new();

        // 1. Ingest Batch
        for sym_cand in &batch {
            let core_c = sym_cand.to_core_candle();
            let sym = &sym_cand.symbol;

            let pipe = history_pipes
                .entry(sym.clone())
                .or_insert_with(Vec::new);
            pipe.push(core_c.clone());
            if pipe.len() > MAX_HISTORY_BARS {
                let drop_n = pipe.len() - MAX_HISTORY_BARS;
                pipe.drain(0..drop_n);
            }
            let count = symbol_update_counts.entry(sym.clone()).or_insert(0);
            *count += 1;

            symbols_updated.push(sym.clone());
        }

        // 2. Evaluate & Record (Synchronized Timestep)
        for sym in symbols_updated.clone() {
            let hist = history_pipes.get(&sym).unwrap();
            let latest_candle = hist.last().unwrap();
            let updates = *symbol_update_counts.get(&sym).unwrap();

            let feed_interval_seconds = if hist.len() >= 2 {
                hist[hist.len() - 1].timestamp - hist[hist.len() - 2].timestamp
            } else {
                300
            };
            let feed_interval_minutes = (feed_interval_seconds as f64 / 60.0).max(1.0);

            let mom_bars = ((15.0 / feed_interval_minutes).round() as usize).max(2);
            let vol_bars = ((25.0 / feed_interval_minutes).round() as usize).max(2);

            let trigger_momentum_3 = if hist.len() >= mom_bars {
                let last = hist[hist.len() - 1].close as f64;
                let lag = hist[hist.len() - mom_bars].close as f64;
                last - lag
            } else {
                0.0
            };

            let window_len = hist.len().min(vol_bars);
            let slice = &hist[hist.len() - window_len..];
            let vol_5 = if slice.len() > 1 {
                let sum: f64 = slice.iter().map(|c| c.close as f64).sum();
                let mean = sum / slice.len() as f64;
                let var: f64 = slice
                    .iter()
                    .map(|c| {
                        let d = c.close as f64 - mean;
                        d * d
                    })
                    .sum::<f64>()
                    / slice.len() as f64;
                var.sqrt()
            } else {
                0.0
            };

            // Process existing intents lifecycle in the environment
            update_paper_registry(
                &mut paper,
                latest_candle,
                &sym,
                updates,
                trigger_momentum_3,
                vol_5,
                false,
            );

            let last_sig = *last_signals.get(&sym).unwrap_or(&SignalType::WAIT);
            let cons = *consistency_counts.get(&sym).unwrap_or(&0);

            // Use consensus layer to gauge environmental reality
            let report =
                evaluate_consensus_status(&elite_strategies, hist, &config, &sym, last_sig, cons);

            last_signals.insert(sym.clone(), report.signal);
            consistency_counts.insert(sym.clone(), report.consistency);

            // Log major telemetry events (continuous phase-space scanning)
            let margin = if report.execution_threshold > 0.0 { (report.execution_score / report.execution_threshold).max(0.0) } else { 1.0 };
            let ts = latest_candle.timestamp;
                    
                    // Priority 2: Execution-Window-Constrained Survivable Movement Modeling
                    let n_bars = config.max_hold_bars as usize;
                    let exec_window_slice = &hist[hist.len().saturating_sub(n_bars)..];
                    let mut micro_noise_sum = 0.0;
                    let mut min_low = f64::MAX;
                    let mut max_high = f64::MIN;
                    
                    // Atlas Telemetry Accumulators
                    let mut bars_in_direction = 0;
                    let mut min_low_idx = 0;
                    let mut max_high_idx = 0;
                    
                    if exec_window_slice.len() > 1 {
                        for i in 1..exec_window_slice.len() {
                            let diff = exec_window_slice[i].close as f64 - exec_window_slice[i-1].close as f64;
                            micro_noise_sum += diff.abs();
                            
                            if report.signal == SignalType::BUY && diff > 0.0 {
                                bars_in_direction += 1;
                            } else if report.signal == SignalType::SELL && diff < 0.0 {
                                bars_in_direction += 1;
                            }
                        }
                        for (idx, c) in exec_window_slice.iter().enumerate() {
                            if (c.low as f64) < min_low {
                                min_low = c.low as f64;
                                min_low_idx = idx;
                            }
                            if (c.high as f64) > max_high {
                                max_high = c.high as f64;
                                max_high_idx = idx;
                            }
                        }
                    }
                    
                    let elasticity_age = if exec_window_slice.len() > 1 {
                        let current_idx = exec_window_slice.len() - 1;
                        if report.signal == SignalType::BUY {
                            current_idx.saturating_sub(min_low_idx)
                        } else {
                            current_idx.saturating_sub(max_high_idx)
                        }
                    } else {
                        0
                    };
                    
                    // === EDGE GENESIS OBSERVATORY ===
                    // Study what existed BEFORE the current topology — where does edge originate?
                    let pre_window_len = n_bars;
                    let pre_start = hist.len().saturating_sub(2 * n_bars);
                    let pre_end = hist.len().saturating_sub(n_bars);
                    let pre_window = if pre_end > pre_start && pre_start < hist.len() {
                        &hist[pre_start..pre_end]
                    } else {
                        &hist[0..0] // empty slice
                    };
                    
                    // 1. Pre-entry volatility (avg bar-to-bar absolute change)
                    let pre_vol = if pre_window.len() > 1 {
                        let mut sum = 0.0;
                        for i in 1..pre_window.len() {
                            sum += (pre_window[i].close as f64 - pre_window[i-1].close as f64).abs();
                        }
                        sum / (pre_window.len() - 1) as f64
                    } else {
                        0.0
                    };
                    
                    // 2. Exec-window volatility (for compression ratio)
                    let exec_vol = if exec_window_slice.len() > 1 {
                        micro_noise_sum / (exec_window_slice.len() - 1) as f64
                    } else {
                        0.0
                    };
                    
                    // 3. Compression Release Ratio: exec_vol / pre_vol
                    // > 1.0 = volatility expansion (compression releasing)
                    // < 1.0 = volatility contraction (compression building)
                    let compression_ratio = if pre_vol > 1e-9 { exec_vol / pre_vol } else { 1.0 };
                    
                    // 4. Pre-entry range (how tight was the market before?)
                    let pre_range = if pre_window.len() > 0 {
                        let ph: f64 = pre_window.iter().map(|c| c.high as f64).fold(f64::MIN, f64::max);
                        let pl: f64 = pre_window.iter().map(|c| c.low as f64).fold(f64::MAX, f64::min);
                        (ph - pl) / latest_candle.close as f64
                    } else {
                        0.0
                    };
                    
                    // 5. Pre-entry directional bias (did a trend exist before entry?)
                    let pre_bias = if pre_window.len() > 1 {
                        let net = pre_window.last().unwrap().close as f64 - pre_window.first().unwrap().close as f64;
                        let gross: f64 = (1..pre_window.len()).map(|i| (pre_window[i].close as f64 - pre_window[i-1].close as f64).abs()).sum();
                        if gross > 1e-9 { net / gross } else { 0.0 }
                    } else {
                        0.0
                    };
                    
                    let micro_noise = if exec_window_slice.len() > 1 { micro_noise_sum / (exec_window_slice.len() - 1) as f64 } else { 0.0 };
                    let gross_move = if min_low < f64::MAX { max_high - min_low } else { 0.0 };
                    let spread_slippage = (latest_candle.close as f64) * 0.0002;
                    let survivable_move = (gross_move - micro_noise - spread_slippage).max(0.0);
                    let micro_exp_move_pct = survivable_move / (latest_candle.close as f64);
                    let legacy_exp_move_pct = report.expected_return;
                    
                    // --- ATLAS TELEMETRY ---
                    let net_move = if exec_window_slice.len() > 1 {
                        (latest_candle.close as f64 - exec_window_slice[0].close as f64).abs()
                    } else {
                        0.0
                    };
                    
                    let directional_efficiency = if micro_noise_sum > 1e-9 { net_move / micro_noise_sum } else { 0.0 };
                    let continuation_density = if exec_window_slice.len() > 1 { bars_in_direction as f64 / (exec_window_slice.len() - 1) as f64 } else { 0.0 };
                    
                    let resilience_score = if gross_move > 1e-9 {
                        if report.signal == SignalType::BUY {
                            (latest_candle.close as f64 - min_low) / gross_move
                        } else {
                            (max_high - latest_candle.close as f64) / gross_move
                        }
                    } else {
                        0.0
                    };
                    
                    println!(
                        "[TELEMETRY] ts={} sym={} margin={:.2} conv={:.2} eq={:.4} eff={:.4} den={:.4} res={:.4} comp={:.4} range={:.6} bias={:.4}",
                        ts, sym, margin, report.conviction_score, report.execution_score, directional_efficiency, continuation_density, resilience_score, compression_ratio, pre_range, pre_bias
                    );
                    io::stdout().flush().unwrap();

                    // Fire observatory intent for longitudinal tracking with Consensus Purification Gates
                    if report.signal != SignalType::WAIT && report.execution_feasible && report.threshold > 0.0 {
                        let margin = (report.execution_score / report.execution_threshold).max(0.0);
                        if margin > 1.2 && report.conviction_score > 0.2 {
                            let prev_micro_exp = *prev_micro_exp_moves.get(&sym).unwrap_or(&0.0);
                            prev_micro_exp_moves.insert(sym.clone(), micro_exp_move_pct);
                        }
                    }
        }

        total_processed += 1;
        let telemetry_count = symbols_updated.len();
        println!(
            "[BATCH_COMPLETE] batches={} symbols={} telemetry={}",
            total_processed,
            batch.len(),
            telemetry_count
        );
        io::stdout().flush().unwrap();
        if total_processed % 10 == 0 {
            let ts = Local::now().format("%H:%M:%S").to_string();
            println!("[HEARTBEAT] {} | Processed {} synchronized JSON batches.", ts, total_processed);
        }
    }

    println!("[OBSERVATORY] Input stream closed. Finalizing...");
    paper.summary();
}
