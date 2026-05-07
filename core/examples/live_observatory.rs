use chrono::Local;
use chronosentiment_core::ga::{
    evaluate_consensus_status, load_elite_strategies, update_paper_registry, GaConfig, PaperRegistry, SignalType, TradeIntent,
};
use chronosentiment_core::market_adapter::Candle;
use chronosentiment_core::TradeRecommendation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::fs::OpenOptions;

const PRICE_SCALE: f64 = 10000.0;

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
    println!("\n🚀 [LIVE_OBSERVATORY]");
    println!("mode=continuous");
    println!("feed=stdin_json");
    println!("symbols=BTC-USD,ETH-USD,SOL-USD");
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
    let mut symbol_update_counts: HashMap<String, usize> = HashMap::new();
    let mut next_rec_id: u64 = 1;

    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut total_processed = 0;

    println!("📡 Listening for continuous environmental telemetry via stdin...");
    println!("   Awaiting synchronized JSON batches...\n");

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

            history_pipes
                .entry(sym.clone())
                .or_insert_with(Vec::new)
                .push(core_c.clone());
            let count = symbol_update_counts.entry(sym.clone()).or_insert(0);
            *count += 1;

            symbols_updated.push(sym.clone());
        }

        // 2. Evaluate & Record (Synchronized Timestep)
        for sym in symbols_updated.clone() {
            let hist = history_pipes.get(&sym).unwrap();
            let latest_candle = hist.last().unwrap();
            let updates = *symbol_update_counts.get(&sym).unwrap();

            let trigger_momentum_3 = if hist.len() >= 3 {
                let last = hist[hist.len() - 1].close as f64;
                let lag3 = hist[hist.len() - 3].close as f64;
                last - lag3
            } else {
                0.0
            };

            let window_len = hist.len().min(5);
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

            // Log major telemetry events
            if report.signal != SignalType::WAIT && report.execution_feasible && report.threshold > 0.0 {
                let margin = (report.execution_score / report.execution_threshold).max(0.0);
                if margin > 1.2 && report.conviction_score > 0.2 {
                    let ts = Local::now().format("%H:%M:%S").to_string();
                    
                    // Priority 2: Execution-Window-Constrained Survivable Movement Modeling
                    let n_bars = config.max_hold_bars as usize;
                    let exec_window_slice = &hist[hist.len().saturating_sub(n_bars)..];
                    let mut micro_noise_sum = 0.0;
                    let mut min_low = f64::MAX;
                    let mut max_high = f64::MIN;
                    if exec_window_slice.len() > 1 {
                        for i in 1..exec_window_slice.len() {
                            let diff = (exec_window_slice[i].close as f64 - exec_window_slice[i-1].close as f64).abs();
                            micro_noise_sum += diff;
                        }
                        for c in exec_window_slice {
                            min_low = min_low.min(c.low as f64);
                            max_high = max_high.max(c.high as f64);
                        }
                    }
                    let micro_noise = if exec_window_slice.len() > 1 { micro_noise_sum / (exec_window_slice.len() - 1) as f64 } else { 0.0 };
                    let gross_move = if min_low < f64::MAX { max_high - min_low } else { 0.0 };
                    let spread_slippage = (latest_candle.close as f64) * 0.0002;
                    let survivable_move = (gross_move - micro_noise - spread_slippage).max(0.0);
                    let micro_exp_move_pct = survivable_move / (latest_candle.close as f64);
                    let legacy_exp_move_pct = report.expected_return;
                    
                    println!(
                        "[TELEMETRY] {} sym={} sig={:?} margin={:.2} conv={:.2} eq={:.4} | legacy_exp={:.6} micro_exp={:.6} gross={:.6} noise={:.6}",
                        ts, sym, report.signal, margin, report.conviction_score, report.execution_score, legacy_exp_move_pct, micro_exp_move_pct, gross_move / latest_candle.close as f64, micro_noise / latest_candle.close as f64
                    );

                    // Fire observatory intent for longitudinal tracking
                    if paper.active_trades.len() + paper.pending_intents.len() < paper.max_concurrent {
                        let price = latest_candle.close as f64;
                        let intent = TradeIntent {
                            rec_id: next_rec_id,
                            symbol: sym.clone(),
                            signal: report.signal,
                            reference_price: price,
                            birth_price: price,
                            recommendation: TradeRecommendation {
                                symbol: sym.clone(),
                                signal: report.signal,
                                rank: margin,
                                raw_edge: report.expected_return,
                                confidence: report.conviction_score,
                                quality_score: report.execution_score,
                                entry_price: price,
                                entry_low: price * 0.999,
                                entry_high: price * 1.001,
                                tp_target: price
                                    * if report.signal == SignalType::BUY {
                                        1.02
                                    } else {
                                        0.98
                                    },
                                sl_target: price
                                    * if report.signal == SignalType::BUY {
                                        0.98
                                    } else {
                                        1.02
                                    },
                                expected_rr: 1.0,
                                expected_edge_bps: report.expected_return * 10000.0,
                                risk_bps: 200.0,
                                holding_bars: config.max_hold_bars,
                                vol_bps: vol_5,
                                vol_bucket: 1,
                                is_execution: true,
                                position_size: 0.1,
                                directional_alpha: 0.0,
                                execution_alpha: 0.0,
                                structural_alpha: 0.0,
                            },
                            strategy_id: 0,
                            rec_score: report.execution_score,
                            rec_feas: 1.0,
                            rec_conf: report.conviction_score,
                            rec_voters: 1,
                            momentum_3: trigger_momentum_3,
                            vol_5,
                            score_std_5: 0.0,
                            consensus: None,
                            age: 0,
                            max_age: 15,
                            intent_created_symbol_updates: updates,
                            confirm_delta_symbol_updates: 0,
                            immediate_market_fill: false,
                            use_recommendation_tpsl: false,
                            sketch_risk_span: 0.0,
                            mode: "OBSERVATORY".to_string(),
                            entry_path: "CONSENSUS".to_string(),
                            regime: "LIVE".to_string(),
                            birth_timestamp: latest_candle.timestamp,
                            intensity: report.execution_score,
                            stability: report.threshold,
                            tier: "OBSERVATORY".to_string(),
                        };

                        if paper.submit_intent(intent) {
                            next_rec_id += 1;
                        }
                    }
                }
            }

            // Continuous Physics Archival (Every 10 batches)
            if total_processed % 10 == 0 {
                let n_bars = config.max_hold_bars as usize;
                let exec_window_slice = &hist[hist.len().saturating_sub(n_bars)..];
                if exec_window_slice.len() > 1 {
                    let mut micro_noise_sum = 0.0;
                    let mut min_low = f64::MAX;
                    let mut max_high = f64::MIN;
                    for i in 1..exec_window_slice.len() {
                        let diff = (exec_window_slice[i].close as f64 - exec_window_slice[i-1].close as f64).abs();
                        micro_noise_sum += diff;
                    }
                    for c in exec_window_slice {
                        min_low = min_low.min(c.low as f64);
                        max_high = max_high.max(c.high as f64);
                    }
                    let micro_noise = micro_noise_sum / (exec_window_slice.len() - 1) as f64;
                    let gross_move = max_high - min_low;
                    let spread_slippage = (latest_candle.close as f64) * 0.0002;
                    let survivable_move = (gross_move - micro_noise - spread_slippage).max(0.0);
                    
                    let micro_exp_move_pct = survivable_move / (latest_candle.close as f64);
                    let legacy_exp_move_pct = report.expected_return.max(0.000001); // Prevent div-by-zero
                    let gross_move_pct = gross_move / latest_candle.close as f64;
                    let noise_floor_pct = micro_noise / latest_candle.close as f64;
                    let divergence = legacy_exp_move_pct / micro_exp_move_pct.max(0.000001);

                    // Write to CSV archive
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("archive/physics_divergence.csv") {
                        let ts = Local::now().timestamp();
                        let regime_str = format!("{:?}", report.regime);
                        let vol_bucket = (vol_5 * 1000.0) as u32; // Primitive volatility bucket
                        let half_life_estimate = config.max_hold_bars / 2; // Crude proxy for signal half-life
                        
                        let _ = writeln!(
                            file, 
                            "{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.2}", 
                            ts, sym, regime_str, vol_bucket, half_life_estimate, legacy_exp_move_pct, gross_move_pct, noise_floor_pct, micro_exp_move_pct, divergence
                        );
                    }
                }
            }
        }

        total_processed += 1;
        if total_processed % 10 == 0 {
            let ts = Local::now().format("%H:%M:%S").to_string();
            println!("[HEARTBEAT] {} | Processed {} synchronized JSON batches. Archiving physics divergence.", ts, total_processed);
        }
    }

    println!("[OBSERVATORY] Input stream closed. Finalizing...");
    paper.summary();
}
