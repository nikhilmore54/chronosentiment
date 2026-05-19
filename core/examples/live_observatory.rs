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
                    
                    // --- SHADOW FERTILITY GOVERNOR ---
                    // 1. Elasticity Zone Bonus (Target: 0.30 - 0.40)
                    let elasticity_bonus: f64 = if directional_efficiency >= 0.30 && directional_efficiency <= 0.40 {
                        1.20 // +20% optimal elastic grinding
                    } else if directional_efficiency < 0.30 {
                        1.10 // +10% chaotic convexity
                    } else {
                        0.80 // -20% penalty for terminal linear persistence (smoothness trap)
                    };
                    
                    // 2. Resilience Bonus (Target: > 0.85)
                    let resilience_bonus: f64 = if resilience_score > 0.85 {
                        1.15 // +15% for strong recovery capability
                    } else {
                        0.90 // -10% for weak bounce
                    };
                    
                    // 3. Shadow Multiplier (Logged, but NOT executed)
                    let shadow_fertility_multiplier = (elasticity_bonus * resilience_bonus).clamp(0.5, 1.5);
                    
                    println!(
                        "[TELEMETRY] {} sym={} sig={:?} margin={:.2} conv={:.2} eq={:.4} | legacy_exp={:.6} micro_exp={:.6} gross={:.6} noise={:.6} | atlas_eff={:.4} atlas_den={:.4} atlas_res={:.4} shadow_fert={:.4} atlas_age={} | genesis_comp={:.4} genesis_range={:.6} genesis_bias={:.4}",
                        ts, sym, report.signal, margin, report.conviction_score, report.execution_score, legacy_exp_move_pct, micro_exp_move_pct, gross_move / latest_candle.close as f64, micro_noise / latest_candle.close as f64, directional_efficiency, continuation_density, resilience_score, shadow_fertility_multiplier, elasticity_age, compression_ratio, pre_range, pre_bias
                    );

                    // Fire observatory intent for longitudinal tracking with Consensus Purification Gates
                    let prev_micro_exp = *prev_micro_exp_moves.get(&sym).unwrap_or(&0.0);
                    let has_propagation_acceleration = prev_micro_exp == 0.0 || micro_exp_move_pct >= prev_micro_exp * 0.90;
                    prev_micro_exp_moves.insert(sym.clone(), micro_exp_move_pct);

                    // 1. Reversal Compression & Adverse Excursion Penalty
                    let excursion_retrace = if gross_move > 1e-9 {
                        if report.signal == SignalType::BUY {
                            (max_high - latest_candle.close as f64) / gross_move
                        } else {
                            (latest_candle.close as f64 - min_low) / gross_move
                        }
                    } else {
                        0.0
                    };

                    // 2. Acceleration Persistence Factor
                    let acc_factor = if prev_micro_exp > 0.0 {
                        (micro_exp_move_pct / prev_micro_exp).clamp(0.5, 1.5)
                    } else {
                        1.0
                    };

                    // 3. Persistence Governor (G_persistence)
                    // Punish severe retracement (> 40% of swing) or severe deceleration
                    let retrace_penalty = (1.5 - excursion_retrace * 2.0).clamp(0.4, 1.1);
                    let persistence_governor = (retrace_penalty * acc_factor).clamp(0.4, 1.25);

                    // 4. Asset Microstructure Hostility Profile (Tighter baseline amplitude)
                    let asset_baseline_hostility = match sym.as_str() {
                        "BTC-USD" => 0.08, // Slower, deeper liquidity, tighter tolerance
                        "ETH-USD" => 0.10, // Intermediate
                        "SOL-USD" => 0.13, // Explosive, more chaotic, wider tolerance
                        _ => 0.10,
                    };

                    // 5. Throttled Regime & Conviction Multipliers
                    let regime_elasticity = match report.regime {
                        MarketRegime::BullTrend | MarketRegime::BearTrend => 1.15, // Trend-supported, tolerate 15% more noise
                        MarketRegime::HighVolatilityNoise => 0.75,                // High noise, contract envelope by 25%
                        MarketRegime::MeanReversion => 0.85,                      // Mean-reverting, reduce tolerance by 15%
                    };

                    let conviction_elasticity = 0.85 + 0.3 * report.conviction_score; // Throttled amplitude

                    // 6. Dynamic Survivable Hostility Envelope with Persistence Governor + Freshness Decay
                    // Soft logistic decay: full fertility when fresh (<10 bars), rapid exhaustion after
                    // Centered at 10 bars, steepness 2.5 — models the phase-transition observed in Toxicity Atlas
                    let freshness_decay = 1.0 / (1.0 + ((elasticity_age as f64 - 10.0) / 2.5_f64).exp());
                    let freshness_decay = freshness_decay.clamp(0.25, 1.0);
                    let live_fertility_multiplier = (shadow_fertility_multiplier * freshness_decay).clamp(0.85, 1.15);
                    let max_allowed_hostility = (asset_baseline_hostility * regime_elasticity * conviction_elasticity * persistence_governor * live_fertility_multiplier).clamp(0.04, 0.22);

                    let noise_to_signal = micro_noise / gross_move.max(1e-6);
                    let is_orderly_propagation = noise_to_signal <= max_allowed_hostility;
                    let has_consensus_stability = report.consistency >= 2;

                    if has_consensus_stability && has_propagation_acceleration && is_orderly_propagation {
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
                                    tp_target: {
                                        let norm_vol = if price > 0.0 { vol_5 / price } else { 0.0020 };
                                        let prop_quality = (micro_exp_move_pct / norm_vol.max(1e-6)).powf(1.35).clamp(0.5, 2.75);
                                        let conviction_factor = 0.8 + 0.4 * report.conviction_score;
                                        let regime_factor = match report.regime {
                                            MarketRegime::BullTrend | MarketRegime::BearTrend => 1.25,
                                            MarketRegime::HighVolatilityNoise => 0.65,
                                            MarketRegime::MeanReversion => 0.85,
                                        };
                                        let tp_dist_pct = (norm_vol * 1.5 * prop_quality * conviction_factor * regime_factor).clamp(0.0025, 0.0120);
                                        price
                                            * if report.signal == SignalType::BUY {
                                                1.0 + tp_dist_pct
                                            } else {
                                                1.0 - tp_dist_pct
                                            }
                                    },
                                    sl_target: {
                                        let norm_vol = if price > 0.0 { vol_5 / price } else { 0.0020 };
                                        let regime_factor = match report.regime {
                                            MarketRegime::BullTrend | MarketRegime::BearTrend => 1.1,
                                            MarketRegime::HighVolatilityNoise => 0.8,
                                            MarketRegime::MeanReversion => 0.9,
                                        };
                                        let sl_dist_pct = (norm_vol * 2.2 * regime_factor).clamp(0.0050, 0.0100);
                                        price
                                            * if report.signal == SignalType::BUY {
                                                1.0 - sl_dist_pct
                                            } else {
                                                1.0 + sl_dist_pct
                                            }
                                    },
                                    expected_rr: {
                                        let norm_vol = if price > 0.0 { vol_5 / price } else { 0.0020 };
                                        let prop_quality = (micro_exp_move_pct / norm_vol.max(1e-6)).powf(1.35).clamp(0.5, 2.75);
                                        let conviction_factor = 0.8 + 0.4 * report.conviction_score;
                                        let regime_factor_tp = match report.regime {
                                            MarketRegime::BullTrend | MarketRegime::BearTrend => 1.25,
                                            MarketRegime::HighVolatilityNoise => 0.65,
                                            MarketRegime::MeanReversion => 0.85,
                                        };
                                        let tp_dist_pct = (norm_vol * 1.5 * prop_quality * conviction_factor * regime_factor_tp).clamp(0.0025, 0.0120);
                                        
                                        let regime_factor_sl = match report.regime {
                                            MarketRegime::BullTrend | MarketRegime::BearTrend => 1.1,
                                            MarketRegime::HighVolatilityNoise => 0.8,
                                            MarketRegime::MeanReversion => 0.9,
                                        };
                                        let sl_dist_pct = (norm_vol * 2.2 * regime_factor_sl).clamp(0.0050, 0.0100);
                                        tp_dist_pct / sl_dist_pct.max(1e-6)
                                    },
                                    expected_edge_bps: report.expected_return * 10000.0,
                                    risk_bps: {
                                        let norm_vol = if price > 0.0 { vol_5 / price } else { 0.0020 };
                                        let regime_factor = match report.regime {
                                            MarketRegime::BullTrend | MarketRegime::BearTrend => 1.1,
                                            MarketRegime::HighVolatilityNoise => 0.8,
                                            MarketRegime::MeanReversion => 0.9,
                                        };
                                        let sl_dist_pct = (norm_vol * 2.2 * regime_factor).clamp(0.0050, 0.0100);
                                        sl_dist_pct * 10000.0
                                    },
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
            }

            // Continuous Physics Archival (Requires 100-bar warmup for structural coherence)
            if updates > 100 {
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
                        let ts = latest_candle.timestamp; // Use Market Timestamp for alignment and backfilling
                        let regime_str = format!("{:?}", report.regime);
                        let vol_bucket = (vol_5 * 1000.0) as u32; 
                        let half_life_estimate = config.max_hold_bars / 2; 
                        
                        let _ = writeln!(
                            file, 
                            "{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.2},{},{},{},{}", 
                            ts, sym, regime_str, vol_bucket, half_life_estimate, legacy_exp_move_pct, gross_move_pct, noise_floor_pct, micro_exp_move_pct, divergence, updates, source_type, is_authentic, generation
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
