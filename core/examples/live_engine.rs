use chronosentiment_core::ga::{
    evaluate_current_status, update_paper_registry, GaConfig, PaperRegistry, Strategy, 
    SignalType, PercentileBuffer, DistributionStats, TradeIntent,
};
use chronosentiment_core::market_adapter::Candle;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::collections::HashMap;
use rand::prelude::*;

const PRICE_SCALE: f64 = 10000.0;
const BASE_POSITION_SIZE: f64 = 0.05; 

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
        Candle {
            timestamp: self.timestamp,
            open: (self.open * PRICE_SCALE) as u64,
            high: (self.high * PRICE_SCALE) as u64,
            low: (self.low * PRICE_SCALE) as u64,
            close: (self.close * PRICE_SCALE) as u64,
            volume: self.volume as u64,
        }
    }
}

fn create_specialist_strategies() -> Vec<Strategy> {
    let mut strats = Vec::new();
    
    // 1. Mean Reversion Specialist (L0)
    let mut s0 = Strategy::from_seed(100);
    s0.archetype = 2; // Reversion
    s0.take_profit = 15;
    s0.stop_loss = 20;
    s0.lineage = 0;
    strats.push(s0);
    
    // 2. Momentum Chaser (L1)
    let mut s1 = Strategy::from_seed(200);
    s1.archetype = 1; // Momentum
    s1.take_profit = 40;
    s1.stop_loss = 15;
    s1.lineage = 1;
    strats.push(s1);
    
    // 3. Liquidity Provider (L2) - High Selectivity
    let mut s2 = Strategy::from_seed(300);
    s2.archetype = 3; // Volatility
    s2.selectivity = 95; 
    s2.lineage = 2;
    strats.push(s2);
    
    // 4. Low-Latency Sniper (L3) - Low Entry Offset
    let mut s3 = Strategy::from_seed(400);
    s3.entry_offset = -2;
    s3.lineage = 3;
    strats.push(s3);
    
    // 5. Anchor Strategy (L4) - Balanced
    let mut s4 = Strategy::from_seed(500);
    s4.lineage = 4;
    strats.push(s4);
    
    strats
}

fn main() {
    println!("⚡ CHRONOSENTIMENT LIVE ENGINE | Mode: Orthogonal Specialists");
    
    let mut rng = StdRng::seed_from_u64(42);
    
    // --- LOAD OR GENERATE SPECIALISTS ---
    let mut strategies = create_specialist_strategies();
    println!("🧠 Loaded {} orthogonal specialists.", strategies.len());
    
    let config = GaConfig::default();
    let mut paper = PaperRegistry::default();
    paper.max_concurrent = 10; 
    paper.adaptation_threshold = 30; 
    
    let mut edge_buffer = PercentileBuffer::new(500);
    let mut current_stats = DistributionStats::default();
    
    let mut history_pipes: HashMap<String, Vec<Candle>> = HashMap::new();
    let mut last_signals: HashMap<String, SignalType> = HashMap::new();
    let mut consistency_counts: HashMap<String, usize> = HashMap::new();
    
    let stdin = io::stdin();
    let mut total_processed = 0;
    let mut last_adaptation_count = 0;

    println!("📡 Listening for candles...");

    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        if line.trim().is_empty() { continue; }
        
        let incoming: Vec<SymbolicCandle> = match serde_json::from_str(&line) { Ok(c) => c, Err(_) => continue };

        for sym_candle in incoming {
            total_processed += 1;
            let symbol = &sym_candle.symbol;
            let candle = sym_candle.to_core_candle();
            
            let history = history_pipes.entry(symbol.clone()).or_insert_with(Vec::new);
            history.push(candle.clone());
            if history.len() > 1000 { history.remove(0); }

            update_paper_registry(&mut paper, &candle);

            // 🔥 STABILIZED WALK-FORWARD ADAPTATION (Active)
            if paper.closed_count > 0 && paper.closed_count % paper.adaptation_threshold == 0 && paper.closed_count != last_adaptation_count {
                last_adaptation_count = paper.closed_count;
                if rng.gen_bool(0.2) {
                    let mut perfs: Vec<(usize, f64)> = (0..strategies.len()).map(|i| (i, paper.get_strategy_performance(i))).collect();
                    perfs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                    let best_id = perfs[0].0;
                    let worst_id = perfs.last().unwrap().0;
                    
                    if worst_id != 4 && perfs.last().unwrap().1 < -0.0010 { // Only replace if truly failing
                        println!("\x1b[93m[STABLE_EVO] Pruning S{} (perf={:.6}) -> Mutant of S{}\x1b[0m", 
                            worst_id, perfs.last().unwrap().1, best_id);
                        
                        let parent_lineage = strategies[best_id].lineage;
                        let siblings = strategies.iter().filter(|s| s.lineage == parent_lineage).count();
                        
                        if siblings < 2 {
                            let mut new_strat = strategies[best_id].mutate(42 + paper.closed_count as u64);
                            new_strat.lineage = parent_lineage;
                            strategies[worst_id] = new_strat;
                            paper.strategy_pnl.insert(worst_id, 0.0);
                            paper.strategy_counts.insert(worst_id, 0);
                        }
                    }
                }
            }

            if history.len() >= 300 {
                let bootstrap = std::env::var("GA_BOOTSTRAP").is_ok();
                let mut buy_strength = 0.0;
                let mut sell_strength = 0.0;
                let mut buy_voters = 0;
                let mut sell_voters = 0;
                let mut shared_raw_edge = 0.0;
                let mut best_reco = None;
                let mut max_rank = -1.0;
                let mut primary_id = 0;
                let mut total_feasibility = 0.0;
                let mut voted_count = 0;
                let mut reject_no_reco = 0usize;
                let mut reject_nonpositive_edge = 0usize;
                let mut reject_low_feas = 0usize;

                let min_feas = if bootstrap { 0.05 } else { 0.40 };
                let edge_gate = if bootstrap { 0.0001 } else { 0.0012 };

                for (idx, strat) in strategies.iter().enumerate() {
                    let last_sig = last_signals.get(symbol).cloned().unwrap_or(SignalType::WAIT);
                    let cons = consistency_counts.get(symbol).cloned().unwrap_or(0);
                    let report = evaluate_current_status(strat, history, &config, symbol, last_sig, cons, &current_stats);
                    
                    if report.raw_edge > shared_raw_edge { shared_raw_edge = report.raw_edge; }

                    // Mutually exclusive rejection accounting:
                    // low_edge -> low_feas -> no_reco -> voter.
                    if report.raw_edge < edge_gate {
                        reject_nonpositive_edge += 1;
                        continue;
                    }
                    if report.execution_feasibility < min_feas {
                        reject_low_feas += 1;
                        continue;
                    }
                    if let Some(reco) = report.recommendation {
                        total_feasibility += report.execution_feasibility;
                        voted_count += 1;

                        let perf = paper.get_strategy_performance(idx);
                        let weight = (1.0 + perf * 50.0).clamp(0.1, 2.0);
                        let w_rank_sq = (reco.rank * reco.rank) * weight;

                        if reco.signal == SignalType::BUY { buy_strength += w_rank_sq; buy_voters += 1; }
                        else if reco.signal == SignalType::SELL { sell_strength += w_rank_sq; sell_voters += 1; }

                        if reco.rank > max_rank {
                            max_rank = reco.rank; primary_id = idx;
                            best_reco = Some((reco.clone(), report.signal, report.consistency));
                        }

                        // 🔥 SURGICAL DEBUG: See what the Momentum Chaser (S1) sees
                        if idx == 1 && total_processed % 50 == 0 {
                             println!("[EDGE_DEBUG] S1 | edge={:.6} feas={:.3} rank={:.3} sig={:?}", 
                                report.raw_edge, report.execution_feasibility, reco.rank, reco.signal);
                        }
                    } else {
                        reject_no_reco += 1;
                    }
                }

                if shared_raw_edge > 0.0 {
                    edge_buffer.push(shared_raw_edge);
                    current_stats = edge_buffer.get_stats();
                }

                let total_strength = buy_strength + sell_strength + 0.001;
                let conf = (buy_strength - sell_strength).abs() / total_strength;
                let final_sig = if buy_strength > sell_strength { SignalType::BUY } else { SignalType::SELL };
                
                let avg_feasibility = if voted_count > 0 { total_feasibility / voted_count as f64 } else { 0.0 };

                let min_conf = if bootstrap { 0.10 } else { 0.40 };
                let min_voters = if bootstrap { 1 } else { 2 };
                let is_high_conf = conf >= min_conf && (buy_voters + sell_voters) >= min_voters;
                let is_capturable = avg_feasibility > min_feas;

                if total_processed % 100 == 0 {
                    println!(
                        "[DIAG] {} conf={:.2} feas={:.2} voters={} p90={:.6} rej:no_reco={} low_edge={} low_feas={}",
                        symbol,
                        conf,
                        avg_feasibility,
                        buy_voters + sell_voters,
                        current_stats.p90,
                        reject_no_reco,
                        reject_nonpositive_edge,
                        reject_low_feas
                    );
                }

                if current_stats.p90 >= edge_gate && is_high_conf && is_capturable {
                    if let Some((mut reco, sig, cons)) = best_reco {
                        if sig == final_sig {
                            last_signals.insert(symbol.clone(), sig);
                            consistency_counts.insert(symbol.clone(), cons);
                            reco.position_size = BASE_POSITION_SIZE * (reco.rank * reco.rank) * (conf * 1.5).clamp(0.5, 2.0);

                            if (paper.active_trades.len() + paper.pending_intents.len()) < paper.max_concurrent {
                                if !paper.active_trades.iter().any(|t| t.symbol == *symbol) && !paper.pending_intents.iter().any(|i| i.symbol == *symbol) {
                                    println!("\x1b[92m[ADAPTIVE_INTENT] {} conf={:.2} feas={:.2} voters={} size={:.1}% S{}\x1b[0m", 
                                        symbol, conf, avg_feasibility, if final_sig == SignalType::BUY { buy_voters } else { sell_voters }, reco.position_size * 100.0, primary_id);
                                    
                                    paper.pending_intents.push(TradeIntent {
                                        symbol: symbol.clone(), signal: reco.signal, reference_price: reco.entry_price,
                                        recommendation: reco, strategy_id: primary_id, consensus: None,
                                        age: 0, max_age: 10,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if total_processed % 500 == 0 {
                let mut lineages = HashMap::new();
                for s in &strategies { *lineages.entry(s.lineage).or_insert(0) += 1; }
                print!("\x1b[95m[HEARTBEAT] count={} p50={:.6} | Diversity:", total_processed, current_stats.p50);
                for (lin, cnt) in lineages { print!(" L{}:{}", lin, cnt); }
                println!("\x1b[0m");
            }
        }
    }
    paper.summary();
}
