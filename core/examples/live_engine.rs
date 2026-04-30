use chronosentiment_core::ga::{
    evaluate_current_status, load_elite_strategies, strategy_evaluation_for_live_reco_snapshot,
    update_paper_registry, DecisionReport, GaConfig, PaperRegistry, Strategy, SignalType,
    PercentileBuffer, DistributionStats, RankStats, TradeIntent, TradeRecommendation, finalize_paper_registry,
    close_active_trades_for_symbol,
};
use chronosentiment_core::reco::{RecommendationEngine, RecommendationResult, RecoConfig};
use chronosentiment_core::market_adapter::Candle;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};
use std::collections::{HashMap, VecDeque};
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

#[derive(Debug, Clone)]
struct RecommendationCandidate {
    rec_id: u64,
    symbol: String,
    score: f64,
    edge: f64,
    conf: f64,
    feas: f64,
    voters: usize,
    primary_id: usize,
    signal: SignalType,
    consistency: usize,
    recommendation: TradeRecommendation,
}

#[derive(Debug, Clone)]
struct RecMeta {
    rec_id: u64,
    symbol: String,
    score: f64,
    edge: f64,
    feas: f64,
    conf: f64,
    voters: usize,
    primary_id: usize,
}

#[derive(Debug, Clone)]
struct PendingConfirmation {
    candidate: RecommendationCandidate,
    created_symbol_updates: usize,
    base_price: f64,
    base_score: f64,
    base_vol: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecommendationMode {
    Coverage,
    Precision,
    Top1,
}

impl RecommendationMode {
    fn from_env() -> Self {
        match std::env::var("REC_MODE")
            .unwrap_or_else(|_| "coverage".to_string())
            .to_lowercase()
            .as_str()
        {
            "precision" => RecommendationMode::Precision,
            "top1" => RecommendationMode::Top1,
            _ => RecommendationMode::Coverage,
        }
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).map_or(false, |v| {
        !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
    })
}

fn percentile(mut values: Vec<f64>, p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((p.clamp(0.0, 100.0) / 100.0) * ((values.len() - 1) as f64)).round() as usize;
    values[rank.min(values.len() - 1)]
}

fn rolling_close_std(history: &[Candle], window: usize) -> f64 {
    if history.len() < window || window == 0 {
        return 0.0;
    }
    let values: Vec<f64> = history
        .iter()
        .rev()
        .take(window)
        .map(|c| c.close as f64)
        .collect();
    let n = values.len() as f64;
    if n <= 0.0 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / n;
    let var = values
        .iter()
        .map(|v| {
            let d = *v - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    var.sqrt()
}

/// Deterministic fitness proxy for the reco population layer (edge + feas + paper perf).
fn live_reco_fitness_proxy(report: &DecisionReport, paper_perf: f64) -> f64 {
    let edge_term = (report.raw_edge / (report.raw_edge + 0.002)).clamp(0.0, 1.0);
    let feas = report.execution_feasibility.clamp(0.0, 1.0);
    let perf = (paper_perf * 50.0).clamp(0.0, 1.0);
    (0.35 * edge_term + 0.35 * feas + 0.3 * perf).clamp(0.0, 1.0)
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

/// Trained elites from `ELITE_PATH` (JSON bundle), else orthogonal specialist seeds.
fn load_strategies_for_paper() -> Vec<Strategy> {
    let path = std::env::var("ELITE_PATH").unwrap_or_default();
    if !path.is_empty() {
        let p = std::path::Path::new(&path);
        if p.is_file() {
            let mut evals = load_elite_strategies(&path);
            if !evals.is_empty() {
                evals.sort_by(|a, b| {
                    b.fitness
                        .partial_cmp(&a.fitness)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let n = evals.len().min(8);
                println!(
                    "📂 Paper/live: loaded {} trained strateg(ies) from {} (using top {} by fitness)",
                    evals.len(),
                    path,
                    n
                );
                return evals
                    .into_iter()
                    .take(n)
                    .enumerate()
                    .map(|(i, mut e)| {
                        e.strategy.lineage = i;
                        e.strategy
                    })
                    .collect();
            }
            eprintln!(
                "⚠️ ELITE_PATH={} present but load_elite_strategies returned empty (invalid JSON?)",
                path
            );
        } else {
            eprintln!("⚠️ ELITE_PATH={} is not a file — using built-in specialists", path);
        }
    }
    create_specialist_strategies()
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
    let rec_mode = RecommendationMode::from_env();
    println!("📌 Recommendation mode: {:?}", rec_mode);
    let blocked_symbols: std::collections::HashSet<String> = std::env::var("REC_BLOCK_SYMBOLS")
        .unwrap_or_else(|_| "ADANIENT.NS,ADANIPORTS.NS".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    println!("🚫 Blocked recommendation symbols: {:?}", blocked_symbols);
    
    let mut rng = StdRng::seed_from_u64(42);
    
    // --- LOAD trained elites (ELITE_PATH) or built-in specialists ---
    let mut strategies = load_strategies_for_paper();
    println!("🧠 Running paper registry with {} strateg(ies).", strategies.len());
    
    let mut config = GaConfig::default();
    let mut paper = PaperRegistry::default();
    paper.max_concurrent = std::env::var("PAPER_MAX_CONCURRENT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10);
    paper.adaptation_threshold = 30; 
    
    let mut edge_buffer = PercentileBuffer::new(500);
    let mut current_stats = DistributionStats::default();
    
    let mut history_pipes: HashMap<String, Vec<Candle>> = HashMap::new();
    let mut score_history: HashMap<String, VecDeque<f64>> = HashMap::new();
    let mut symbol_update_counts: HashMap<String, usize> = HashMap::new();
    let mut pending_confirmations: HashMap<String, PendingConfirmation> = HashMap::new();
    let mut last_signals: HashMap<String, SignalType> = HashMap::new();
    let mut consistency_counts: HashMap<String, usize> = HashMap::new();
    
    let stdin = io::stdin();
    let mut total_processed = 0;
    let mut last_adaptation_count = 0;
    let mut next_rec_id: u64 = 1;
    let mut pending_meta: HashMap<String, VecDeque<RecMeta>> = HashMap::new();
    let mut active_meta: HashMap<String, VecDeque<RecMeta>> = HashMap::new();
    let mut next_rank_stats = RankStats::zeroed();
    let mut next_rank_obs = 0usize;
    let rankstats_commit_trades = std::env::var("RANKSTATS_COMMIT_TRADES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20)
        .max(1);
    let rec_min_move = std::env::var("REC_MIN_MOVE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.001);
    let signal_decay_exit = std::env::var("PAPER_SIGNAL_DECAY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let signal_edge_min = std::env::var("PAPER_SIGNAL_EDGE_MIN")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0005);
    let confirm_delta = std::env::var("REC_CONFIRM_DELTA")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let confirm_vol_mult = std::env::var("REC_CONFIRM_VOL_MULT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.5)
        .max(1.0);
    let candidate_voter_percentile = std::env::var("REC_CAND_VOTER_PCT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(60.0)
        .clamp(0.0, 100.0);
    let candidate_conf_percentile = std::env::var("REC_CAND_CONF_PCT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(75.0)
        .clamp(0.0, 100.0);
    let intent_max_age_base = std::env::var("INTENT_MAX_AGE_BASE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(14)
        .max(1);
    let intent_max_age_strong = std::env::var("INTENT_MAX_AGE_STRONG")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(14)
        .max(intent_max_age_base);
    let intent_high_voters_threshold = std::env::var("INTENT_HIGH_VOTERS_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(6)
        .max(1);
    let intent_high_conf_threshold = std::env::var("INTENT_HIGH_CONF_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.70)
        .clamp(0.0, 1.0);

    let live_gate_reco_stability = std::env::var("LIVE_GATE_RECO_STABILITY_MIN")
        .ok()
        .and_then(|s| s.parse::<f64>().ok());
    let live_gate_reco_ag_global = std::env::var("LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN")
        .ok()
        .and_then(|s| s.parse::<f64>().ok());
    let live_gate_reco_fitness = std::env::var("LIVE_GATE_RECO_FITNESS_MIN")
        .ok()
        .and_then(|s| s.parse::<f64>().ok());
    let run_reco_engine = live_gate_reco_stability.is_some()
        || live_gate_reco_ag_global.is_some()
        || live_gate_reco_fitness.is_some()
        || env_flag("POOL_DEBUG")
        || env_flag("RECO_DEBUG");
    let reco_path_probe = env_flag("RECO_PATH_PROBE");
    let strat_probe = env_flag("STRAT_PROBE");
    let reco_single_accept_diag = env_flag("REC_SINGLE_ACCEPT_DIAG");
    let candidate_probe = env_flag("CANDIDATE_PROBE");
    let mut cand_batches = 0usize;
    let mut cand_total_sum = 0usize;
    let mut cand_voters_pos_sum = 0usize;
    let mut cand_feas_pos_sum = 0usize;
    let mut cand_stage1_pass_sum = 0usize;
    let mut cand_admitted_sum = 0usize;

    println!("📡 Listening for candles...");
    println!(
        "   Optional gates: LIVE_GATE_EDGE_STABILITY_MIN=, LIVE_GATE_CONF_MIN=, LIVE_GATE_RECO_STABILITY_MIN= (reco S), LIVE_GATE_RECO_AGREEMENT_GLOBAL_MIN= (reco G), LIVE_GATE_RECO_FITNESS_MIN= (medoid fitness); POOL_DEBUG=1 / RECO_DEBUG=1"
    );
    println!(
        "   Live reco uses small proxy pools → S/G/F read weaker than train_nse; start S around 0.35–0.55 (not ~0.8). [DIAG] FINAL=1 = meta-gates only; emission still needs edge/feas/p90/voters/blocklist."
    );

    // One stdin line == one synchronized timestep across symbols (streamer batch) = one AWR window.
    let mut awr_windows_total: u64 = 0;
    let mut awr_windows_with_candidates: u64 = 0;
    let mut awr_windows_triggered: u64 = 0;

    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
        if line.trim().is_empty() { continue; }
        
        let incoming: Vec<SymbolicCandle> = match serde_json::from_str(&line) { Ok(c) => c, Err(_) => continue };
        awr_windows_total = awr_windows_total.saturating_add(1);
        let line_start_triggered = paper.intents_triggered;
        let mut recommendations: Vec<RecommendationCandidate> = Vec::new();

        for sym_candle in incoming {
            total_processed += 1;
            let symbol = &sym_candle.symbol;
            let candle = sym_candle.to_core_candle();
            let pre_closed = paper.closed_count;
            let pre_pnl_len = paper.pnl_history.len();
            let mut active_before: HashMap<String, usize> = HashMap::new();
            for t in &paper.active_trades {
                *active_before.entry(t.symbol.clone()).or_insert(0) += 1;
            }
            
            let history = history_pipes.entry(symbol.clone()).or_insert_with(Vec::new);
            history.push(candle.clone());
            if history.len() > 1000 { history.remove(0); }
            *symbol_update_counts.entry(symbol.clone()).or_insert(0) += 1;
            let sym_updates_now = *symbol_update_counts.get(symbol).unwrap_or(&0);
            let trigger_momentum_3 = if history.len() >= 4 {
                let last = history[history.len() - 1].close as f64;
                let lag3 = history[history.len() - 4].close as f64;
                last - lag3
            } else {
                0.0
            };
            let trigger_vol_5 = rolling_close_std(history, 5);

            update_paper_registry(
                &mut paper,
                &candle,
                symbol,
                sym_updates_now,
                trigger_momentum_3,
                trigger_vol_5,
            );
            if !paper.closed_observations.is_empty() {
                for obs in paper.closed_observations.drain(..) {
                    let r_bucket = (obs.rank * 10.0).floor().clamp(0.0, 9.0) as usize;
                    let v_bucket = obs.vol_bucket.min(4);
                    next_rank_stats.bucket_mfe_sum[r_bucket][v_bucket] += obs.mfe.clamp(0.0, 0.04);
                    next_rank_stats.bucket_mae_sum[r_bucket][v_bucket] += obs.mae_abs.clamp(0.0, 0.03);
                    next_rank_stats.bucket_time_sum[r_bucket][v_bucket] += obs.hold_bars as f64;
                    next_rank_stats.bucket_count[r_bucket][v_bucket] += 1;
                    next_rank_obs += 1;
                }
                if next_rank_obs >= rankstats_commit_trades {
                    config.rank_stats.blend_with_min_count(next_rank_stats, 0.2, 1);
                    next_rank_stats = RankStats::zeroed();
                    next_rank_obs = 0;
                    if std::env::var("RANKSTATS_PROBE").is_ok() {
                        println!("[RANKSTATS_COMMIT] committed_observations={}", rankstats_commit_trades);
                        let mut values: Vec<f64> = Vec::with_capacity(10 * 5);
                        let mut non_default = 0usize;
                        for r in 0..10 {
                            for v in 0..5 {
                                let c = config.rank_stats.bucket_count[r][v].max(1);
                                let mfe = config.rank_stats.bucket_mfe_sum[r][v] / c as f64;
                                values.push(mfe);
                                if (config.rank_stats.bucket_mfe_sum[r][v] - 0.0045).abs() > 1e-9
                                    || config.rank_stats.bucket_count[r][v] != 1
                                {
                                    non_default += 1;
                                }
                            }
                        }
                        let n = values.len().max(1) as f64;
                        let mean_mfe = values.iter().sum::<f64>() / n;
                        let var = values
                            .iter()
                            .map(|x| {
                                let d = x - mean_mfe;
                                d * d
                            })
                            .sum::<f64>()
                            / n;
                        let std_mfe = var.sqrt();
                        let min_mfe = values.iter().copied().fold(f64::INFINITY, f64::min);
                        let max_mfe = values
                            .iter()
                            .copied()
                            .fold(f64::NEG_INFINITY, f64::max);
                        println!(
                            "[RANKSTATS_LIVE] non_default={} mean_mfe={:.6} std_mfe={:.6} min_mfe={:.6} max_mfe={:.6}",
                            non_default,
                            mean_mfe,
                            std_mfe,
                            min_mfe,
                            max_mfe
                        );
                    }
                }
            }
            let mut active_after: HashMap<String, usize> = HashMap::new();
            for t in &paper.active_trades {
                *active_after.entry(t.symbol.clone()).or_insert(0) += 1;
            }
            for (sym, after_count) in &active_after {
                let before_count = *active_before.get(sym).unwrap_or(&0);
                if *after_count > before_count {
                    let delta = *after_count - before_count;
                    let entry = pending_meta.entry(sym.clone()).or_default();
                    let dst = active_meta.entry(sym.clone()).or_default();
                    for _ in 0..delta {
                        if let Some(meta) = entry.pop_front() {
                            dst.push_back(meta);
                        }
                    }
                }
            }
            if paper.closed_count > pre_closed && paper.pnl_history.len() > pre_pnl_len {
                let mut closed_metas: Vec<RecMeta> = Vec::new();
                for (sym, before_count) in &active_before {
                    let after_count = *active_after.get(sym).unwrap_or(&0);
                    if *before_count > after_count {
                        let delta = *before_count - after_count;
                        let src = active_meta.entry(sym.clone()).or_default();
                        for _ in 0..delta {
                            if let Some(meta) = src.pop_front() {
                                closed_metas.push(meta);
                            }
                        }
                    }
                }
                let new_pnls = &paper.pnl_history[pre_pnl_len..];
                for (meta, pnl) in closed_metas.into_iter().zip(new_pnls.iter().copied()) {
                    println!(
                        "[REC_OUTCOME] rec_id={} sym={} score={:.6} edge={:.6} feas={:.3} conf={:.3} voters={} S{} pnl={:.6}",
                        meta.rec_id,
                        meta.symbol,
                        meta.score,
                        meta.edge,
                        meta.feas,
                        meta.conf,
                        meta.voters,
                        meta.primary_id,
                        pnl
                    );
                }
            }

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
                let mock_edge_blend = std::env::var("MOCK_EDGE_BLEND").is_ok();
                let short_return = if history.len() >= 2 {
                    let prev = history[history.len() - 2].close as f64;
                    let curr = history[history.len() - 1].close as f64;
                    if prev > 0.0 {
                        ((curr / prev) - 1.0).clamp(-0.05, 0.05)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let mut buy_strength = 0.0;
                let mut sell_strength = 0.0;
                let mut buy_voters = 0;
                let mut sell_voters = 0;
                let mut shared_raw_edge = 0.0;
                let mut best_reco = None;
                let mut max_rank = -1.0;
                let mut primary_id = 0;
                let mut selected_edge = 0.0;
                let mut selected_feasibility = 0.0;
                let mut total_feasibility = 0.0;
                let mut voted_count = 0;
                let mut reject_no_reco = 0usize;
                let mut reject_nonpositive_edge = 0usize;
                let mut reject_low_feas = 0usize;
                // Raw edges from strategies that produced a recommendation (cross-strategy spread → structural stability).
                let mut edges_with_reco: Vec<f64> = Vec::new();
                let mut reco_population = Vec::new();

                let min_feas = if bootstrap { 0.05 } else { 0.40 };
                let edge_gate = if bootstrap { 0.0001 } else { 0.0012 };

                for (idx, strat) in strategies.iter().enumerate() {
                    let last_sig = last_signals.get(symbol).cloned().unwrap_or(SignalType::WAIT);
                    let cons = consistency_counts.get(symbol).cloned().unwrap_or(0);
                    let report = evaluate_current_status(strat, history, &config, symbol, last_sig, cons, &current_stats);
                    if strat_probe && total_processed % 100 == 0 {
                        println!(
                            "[STRAT_OUT] sym={} strat_id={} raw_edge={:.6} feas={:.3} active={} has_reco={}",
                            symbol,
                            idx,
                            report.raw_edge,
                            report.execution_feasibility,
                            (report.raw_edge >= edge_gate && report.execution_feasibility >= min_feas) as i32,
                            report.recommendation.is_some() as i32
                        );
                    }
                    if run_reco_engine {
                        let paper_perf = paper.get_strategy_performance(idx);
                        let fit = live_reco_fitness_proxy(&report, paper_perf);
                        let cap = report
                            .capture_efficiency
                            .unwrap_or_else(|| report.execution_feasibility.mul_add(2.0, -1.0))
                            .clamp(-1.0, 1.0);
                        reco_population.push(strategy_evaluation_for_live_reco_snapshot(
                            strat.clone(),
                            fit,
                            report.conviction_score,
                            report.execution_feasibility,
                            cap,
                        ));
                    }
                    
                    let effective_raw_edge = if mock_edge_blend {
                        (0.7 * report.raw_edge + 0.3 * short_return).max(0.0)
                    } else {
                        report.raw_edge
                    };
                    if effective_raw_edge > shared_raw_edge {
                        shared_raw_edge = effective_raw_edge;
                    }

                    // Mutually exclusive rejection accounting:
                    // low_edge -> low_feas -> no_reco -> voter.
                    if effective_raw_edge < edge_gate {
                        reject_nonpositive_edge += 1;
                        continue;
                    }
                    if report.execution_feasibility < min_feas {
                        reject_low_feas += 1;
                        continue;
                    }
                    if let Some(reco) = report.recommendation {
                        edges_with_reco.push(effective_raw_edge);
                        if signal_decay_exit
                            && effective_raw_edge < signal_edge_min
                            && paper.active_trades.iter().any(|t| t.symbol == *symbol)
                        {
                            let _closed = close_active_trades_for_symbol(
                                &mut paper,
                                symbol,
                                &candle,
                                "SIGNAL_DECAY",
                            );
                        }
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
                            selected_edge = effective_raw_edge;
                            selected_feasibility = report.execution_feasibility;
                        }

                        // 🔥 SURGICAL DEBUG: See what the Momentum Chaser (S1) sees
                        if idx == 1 && total_processed % 50 == 0 {
                             println!("[EDGE_DEBUG] S1 | edge={:.6} feas={:.3} rank={:.3} sig={:?}", 
                                effective_raw_edge, report.execution_feasibility, reco.rank, reco.signal);
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
                let decision_feasibility = if selected_feasibility > 0.0 {
                    selected_feasibility
                } else {
                    avg_feasibility
                };

                let min_conf = if bootstrap { 0.10 } else { 0.40 };
                let min_voters_required = if bootstrap { 1 } else { 2 };
                let is_high_conf = conf >= min_conf && (buy_voters + sell_voters) >= min_voters_required;
                // Bootstrap floors feasibility at 0.05; strict `>` would reject exactly 0.05 (dead zone).
                let is_capturable = if bootstrap {
                    decision_feasibility >= min_feas
                } else {
                    decision_feasibility > min_feas
                };
                let voters = buy_voters + sell_voters;
                let active_strats = voted_count;
                let voters = if reco_single_accept_diag {
                    active_strats
                } else {
                    voters
                };
                if strat_probe && total_processed % 100 == 0 {
                    println!(
                        "[STRAT_AGG] sym={} total_strats={} active_strats={} voters={} buy_voters={} sell_voters={} diag_single_accept={}",
                        symbol,
                        strategies.len(),
                        active_strats,
                        voters,
                        buy_voters,
                        sell_voters,
                        reco_single_accept_diag as i32
                    );
                }
                let edge_after_floor = if shared_raw_edge >= edge_gate {
                    shared_raw_edge
                } else {
                    0.0
                };
                let voters_pre_count = voters;
                let voters_post_count = if voters_pre_count >= min_voters_required {
                    voters_pre_count
                } else {
                    0
                };

                // Optional structural gates (unset = off). Uses cross-strategy edge dispersion as a
                // stability proxy: mean/(std+eps), capped like reco stability (higher = edges agree).
                let live_gate_edge_stab = std::env::var("LIVE_GATE_EDGE_STABILITY_MIN")
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok());
                let live_gate_conf_floor = std::env::var("LIVE_GATE_CONF_MIN")
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok());
                let edge_stability = if edges_with_reco.len() >= 2 {
                    let em = edges_with_reco.iter().sum::<f64>() / edges_with_reco.len() as f64;
                    let var = edges_with_reco
                        .iter()
                        .map(|e| (e - em).powi(2))
                        .sum::<f64>()
                        / edges_with_reco.len() as f64;
                    let es = var.sqrt();
                    (em / (es + 1e-9)).min(5.0)
                } else {
                    0.0
                };
                let pass_edge_stability = live_gate_edge_stab.map_or(true, |req| {
                    if edges_with_reco.len() < 2 {
                        return false;
                    }
                    edge_stability >= req
                });
                let pass_conf_floor = live_gate_conf_floor.map_or(true, |req| conf >= req);

                let (pass_reco_structure, reco_diag_s, reco_diag_g, reco_diag_f) =
                    if run_reco_engine && !reco_population.is_empty() {
                        let reco_cfg = RecoConfig::default();
                        let res = RecommendationEngine::process(
                            &reco_population,
                            &[],
                            &reco_cfg,
                            symbol,
                        );
                        let m = match &res {
                            RecommendationResult::Trade(r) => &r.ensemble_metrics,
                            RecommendationResult::WeakSignal(r) => &r.ensemble_metrics,
                            RecommendationResult::NoTrade { metrics, .. } => metrics,
                        };
                        let stab_ok =
                            live_gate_reco_stability.map_or(true, |t| m.stability >= t);
                        let agree_ok =
                            live_gate_reco_ag_global.map_or(true, |t| m.agreement_global >= t);
                        let fit_ok =
                            live_gate_reco_fitness.map_or(true, |t| m.medoid_fitness >= t);
                        let pass = stab_ok && agree_ok && fit_ok;
                        if reco_path_probe && total_processed % 100 == 0 {
                            println!(
                                "[RECO_POOL] sym={} run_reco_engine=1 pool_size={} feas={:.3} voters={} reco_S={:.3} reco_G={:.3} reco_F={:.3}",
                                symbol,
                                reco_population.len(),
                                avg_feasibility,
                                voters,
                                m.stability,
                                m.agreement_global,
                                m.medoid_fitness
                            );
                            println!(
                                "[RECO_BREAKDOWN] sym={} stab_ok={} agree_ok={} fit_ok={} stab_req={} agree_req={} fit_req={}",
                                symbol,
                                stab_ok as i32,
                                agree_ok as i32,
                                fit_ok as i32,
                                live_gate_reco_stability
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_ag_global
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_fitness
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string())
                            );
                            println!(
                                "[RECO_CHECK] sym={} feas={:.3} voters={} reco_S={:.3} reco_G={:.3} reco_F={:.3} pass_reco={}",
                                symbol,
                                avg_feasibility,
                                voters,
                                m.stability,
                                m.agreement_global,
                                m.medoid_fitness,
                                pass as i32
                            );
                        }
                        (pass, m.stability, m.agreement_global, m.medoid_fitness)
                    } else {
                        if reco_path_probe && total_processed % 100 == 0 {
                            println!(
                                "[RECO_POOL] sym={} run_reco_engine={} pool_size={} feas={:.3} voters={} reco_S=0.000 reco_G=0.000 reco_F=0.000",
                                symbol,
                                run_reco_engine as i32,
                                reco_population.len(),
                                avg_feasibility,
                                voters
                            );
                            println!(
                                "[RECO_BREAKDOWN] sym={} stab_ok=1 agree_ok=1 fit_ok=1 stab_req={} agree_req={} fit_req={}",
                                symbol,
                                live_gate_reco_stability
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_ag_global
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string()),
                                live_gate_reco_fitness
                                    .map(|v| format!("{:.3}", v))
                                    .unwrap_or_else(|| "off".to_string())
                            );
                            println!(
                                "[RECO_CHECK] sym={} feas={:.3} voters={} reco_S=0.000 reco_G=0.000 reco_F=0.000 pass_reco=1",
                                symbol,
                                avg_feasibility,
                                voters
                            );
                        }
                        (true, 0.0, 0.0, 0.0)
                    };

                // Deterministic recommendation gate: executable, consistent, and ranked.
                let (edge_min, feas_min, conf_min, reco_min_voters, score_min) = match rec_mode {
                    RecommendationMode::Coverage => {
                        if bootstrap {
                            // Align with `min_feas` / bootstrap feasibility floor (~0.05) so reco can emit.
                            (0.001, 0.05, 0.20, 1usize, 0.0002)
                        } else {
                            (0.0012, 0.40, 0.40, 2usize, 0.0010)
                        }
                    }
                    RecommendationMode::Precision => {
                        if bootstrap {
                            (0.001, 0.70, 0.20, 1usize, 0.0020)
                        } else {
                            (0.0012, 0.70, 0.40, 2usize, 0.0020)
                        }
                    }
                    RecommendationMode::Top1 => {
                        if bootstrap {
                            (0.001, 0.70, 0.20, 1usize, 0.0020)
                        } else {
                            (0.0012, 0.70, 0.40, 2usize, 0.0020)
                        }
                    }
                };

                if total_processed % 100 == 0 {
                    println!(
                        "[EDGE_TRACE] sym={} raw_edge={:.6} edge_after_floor={:.6} voters_pre={} voters_post={}",
                        symbol,
                        shared_raw_edge,
                        edge_after_floor,
                        voters_pre_count,
                        voters_post_count
                    );
                    let diag_edge = if selected_edge > 1e-12 {
                        selected_edge
                    } else {
                        shared_raw_edge
                    };
                    let pass_edge_i = u8::from(pass_edge_stability);
                    let pass_conf_i = u8::from(pass_conf_floor);
                    let pass_reco_i = u8::from(pass_reco_structure);
                    let final_meta_i = u8::from(
                        pass_edge_stability && pass_conf_floor && pass_reco_structure,
                    );
                    println!(
                        "[DIAG] sym={} edge={:.6} conf={:.2} edge_stab={:.3} reco_S={:.3} reco_G={:.3} reco_F={:.3} pass_edge={} pass_conf={} pass_reco={} FINAL={} feas={:.2} voters={} p90={:.6} rej:no_reco={} low_edge={} low_feas={}",
                        symbol,
                        diag_edge,
                        conf,
                        edge_stability,
                        reco_diag_s,
                        reco_diag_g,
                        reco_diag_f,
                        pass_edge_i,
                        pass_conf_i,
                        pass_reco_i,
                        final_meta_i,
                        avg_feasibility,
                        buy_voters + sell_voters,
                        current_stats.p90,
                        reject_no_reco,
                        reject_nonpositive_edge,
                        reject_low_feas
                    );
                }

                if std::env::var("EMIT_PROBE").is_ok() && symbol.as_str() == "AXISBANK.NS" {
                    let p90_ok = current_stats.p90 >= edge_gate;
                    let final_meta =
                        pass_edge_stability && pass_conf_floor && pass_reco_structure;
                    let blocked = blocked_symbols.contains(symbol);
                    let (has_best, aligned, rec_score, passes_gate) = match &best_reco {
                        Some((reco, sig, _)) => {
                            let al = *sig == final_sig;
                            let rs = if al {
                                let delta_ret_abs = reco.expected_edge_bps.abs() / 10000.0;
                                let move_factor = (delta_ret_abs / rec_min_move).clamp(1.0, 3.0);
                                (selected_edge * decision_feasibility * conf * move_factor).max(0.0)
                            } else {
                                0.0
                            };
                            let pg = al
                                && selected_edge >= edge_min
                                && decision_feasibility >= feas_min
                                && conf >= conf_min
                                && voters >= reco_min_voters
                                && rs >= score_min
                                && pass_edge_stability
                                && pass_conf_floor
                                && pass_reco_structure
                                && !blocked;
                            (true, al, rs, pg)
                        }
                        None => (false, false, 0.0, false),
                    };
                    let slot_ok = (paper.active_trades.len() + paper.pending_intents.len())
                        < paper.max_concurrent;
                    let no_dup = !paper.active_trades.iter().any(|t| t.symbol == *symbol)
                        && !paper
                            .pending_intents
                            .iter()
                            .any(|i| i.symbol == *symbol);
                    let would_push = p90_ok
                        && is_high_conf
                        && is_capturable
                        && has_best
                        && aligned
                        && slot_ok
                        && no_dup
                        && passes_gate;
                    println!(
                        "[EMIT_TRACE] sym={} p90_ok={} p90={:.6} edge_gate={:.6} hi_conf={} cap={} pass_pe={} pass_pc={} pass_pr={} final_meta={} edge={:.6} feas={:.3} conf={:.3} voters={} has_br={} sig_ok={} slot_ok={} no_dup={} blocked={} rec_score={:.6} pass_gate={} would_push={}",
                        symbol,
                        p90_ok as i32,
                        current_stats.p90,
                        edge_gate,
                        is_high_conf as i32,
                        is_capturable as i32,
                        pass_edge_stability as i32,
                        pass_conf_floor as i32,
                        pass_reco_structure as i32,
                        final_meta as i32,
                        selected_edge,
                        decision_feasibility,
                        conf,
                        voters,
                        has_best as i32,
                        aligned as i32,
                        slot_ok as i32,
                        no_dup as i32,
                        blocked as i32,
                        rec_score,
                        passes_gate as i32,
                        would_push as i32
                    );
                }

                if current_stats.p90 >= edge_gate && is_high_conf && is_capturable {
                    if let Some((mut reco, sig, cons)) = best_reco {
                        if sig == final_sig {
                            reco.position_size = BASE_POSITION_SIZE * (reco.rank * reco.rank) * (conf * 1.5).clamp(0.5, 2.0);

                            if (paper.active_trades.len() + paper.pending_intents.len()) < paper.max_concurrent {
                                if !paper.active_trades.iter().any(|t| t.symbol == *symbol) && !paper.pending_intents.iter().any(|i| i.symbol == *symbol) {
                                    let delta_ret_abs = reco.expected_edge_bps.abs() / 10000.0;
                                    // Movement factor boosts meaningful price travel but stays bounded.
                                    let move_factor = (delta_ret_abs / rec_min_move).clamp(1.0, 3.0);
                                    let rec_score = (selected_edge * decision_feasibility * conf * move_factor).max(0.0);
                                    let passes_reco_gate = selected_edge >= edge_min
                                        && decision_feasibility >= feas_min
                                        && conf >= conf_min
                                        && voters >= reco_min_voters
                                        && rec_score >= score_min
                                        && pass_edge_stability
                                        && pass_conf_floor
                                        && pass_reco_structure
                                        && !blocked_symbols.contains(symbol);
                                    if passes_reco_gate {
                                        recommendations.push(RecommendationCandidate {
                                            rec_id: next_rec_id,
                                            symbol: symbol.clone(),
                                            score: rec_score,
                                            edge: selected_edge,
                                            conf,
                                            feas: decision_feasibility,
                                            voters,
                                            primary_id,
                                            signal: sig,
                                            consistency: cons,
                                            recommendation: reco,
                                        });
                                        next_rec_id += 1;
                                    }
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

        // AWR: "candidates" = batches where the reco pool is non-empty *after* existing path gates
        // (history length, edge/conf/feas/reco structure, slot/dup/blocklist, `passes_reco_gate`, etc.).
        // This is not raw strategy hits; use C/W as "eligible-reco coverage", not pre-filter universe.
        if !recommendations.is_empty() {
            awr_windows_with_candidates = awr_windows_with_candidates.saturating_add(1);
        }

        if std::env::var("EMIT_PROBE").is_ok() {
            println!("[POOL_SIZE] n={}", recommendations.len());
        }
        if std::env::var("RANKSTATS_PROBE").is_ok() {
            let mut uniq: Vec<f64> = recommendations
                .iter()
                .map(|c| c.recommendation.expected_edge_bps)
                .collect();
            uniq.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            uniq.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
            println!("EDGE_VARIANCE_CHECK unique_edge_bps_values={}", uniq.len());
        }

        // Stage-1: candidate creation (t0) from recommendation pool.
        recommendations.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.symbol.cmp(&b.symbol))
        });
        let current_scores: HashMap<String, f64> = recommendations
            .iter()
            .map(|c| (c.symbol.clone(), c.score))
            .collect();
        let voter_threshold = percentile(
            recommendations
                .iter()
                .map(|c| c.voters as f64)
                .collect::<Vec<f64>>(),
            candidate_voter_percentile,
        );
        let conf_threshold = percentile(
            recommendations
                .iter()
                .map(|c| c.conf)
                .collect::<Vec<f64>>(),
            candidate_conf_percentile,
        );
        let batch_total = recommendations.len();
        let batch_voters_pos = recommendations.iter().filter(|c| c.voters > 0).count();
        let batch_feas_pos = recommendations.iter().filter(|c| c.feas > 0.0).count();
        let mut batch_stage1_pass = 0usize;
        let mut batch_admitted = 0usize;
        for cand in &recommendations {
            let duplicate_live = paper.active_trades.iter().any(|t| t.symbol == cand.symbol)
                || paper.pending_intents.iter().any(|i| i.symbol == cand.symbol);
            let candidate_gate =
                (cand.voters as f64) >= voter_threshold && cand.conf >= conf_threshold;
            if candidate_gate {
                batch_stage1_pass = batch_stage1_pass.saturating_add(1);
            }
            let blocked_by_pending = pending_confirmations.contains_key(&cand.symbol);
            if !candidate_gate || duplicate_live || blocked_by_pending {
                continue;
            }
            batch_admitted = batch_admitted.saturating_add(1);
            let update_count = *symbol_update_counts.get(&cand.symbol).unwrap_or(&0);
            let base_price = history_pipes
                .get(&cand.symbol)
                .and_then(|h| h.last())
                .map(|c| c.close as f64)
                .unwrap_or(cand.recommendation.entry_price);
            let base_vol = history_pipes
                .get(&cand.symbol)
                .map(|h| rolling_close_std(h, 5))
                .unwrap_or(0.0);
            pending_confirmations.insert(
                cand.symbol.clone(),
                PendingConfirmation {
                    candidate: cand.clone(),
                    created_symbol_updates: update_count,
                    base_price,
                    base_score: cand.score,
                    base_vol,
                },
            );
        }
        cand_batches = cand_batches.saturating_add(1);
        cand_total_sum = cand_total_sum.saturating_add(batch_total);
        cand_voters_pos_sum = cand_voters_pos_sum.saturating_add(batch_voters_pos);
        cand_feas_pos_sum = cand_feas_pos_sum.saturating_add(batch_feas_pos);
        cand_stage1_pass_sum = cand_stage1_pass_sum.saturating_add(batch_stage1_pass);
        cand_admitted_sum = cand_admitted_sum.saturating_add(batch_admitted);
        if candidate_probe && total_processed % 100 == 0 {
            println!(
                "[CANDIDATE_STATS] batch_total={} voters_pos={} feas_pos={} stage1_pass={} admitted={}",
                batch_total,
                batch_voters_pos,
                batch_feas_pos,
                batch_stage1_pass,
                batch_admitted
            );
        }

        // Stage-2: confirmation at t0 + Δ, then execute.
        let mut confirmed: Vec<(RecommendationCandidate, u32)> = Vec::new();
        let pending_symbols: Vec<String> = pending_confirmations.keys().cloned().collect();
        for sym in pending_symbols {
            let Some(pending) = pending_confirmations.get(&sym).cloned() else {
                continue;
            };
            let now_updates = *symbol_update_counts.get(&sym).unwrap_or(&0);
            if now_updates.saturating_sub(pending.created_symbol_updates) < confirm_delta {
                continue;
            }
            let Some(history) = history_pipes.get(&sym) else {
                pending_confirmations.remove(&sym);
                continue;
            };
            let current_price = history.last().map(|c| c.close as f64).unwrap_or(pending.base_price);
            let momentum_confirm = current_price - pending.base_price;
            let vol_confirm = rolling_close_std(history, confirm_delta.saturating_add(1));
            let score_now_opt = current_scores.get(&sym).copied();
            let score_trend = score_now_opt.unwrap_or(pending.base_score) - pending.base_score;
            let vol_limit = pending.base_vol.max(1e-9) * confirm_vol_mult;
            let vol_ok = if pending.base_vol <= 1e-9 {
                vol_confirm <= 1e-9
            } else {
                vol_confirm <= vol_limit
            };
            let confirmed_gate =
                momentum_confirm >= 0.0 && score_now_opt.is_some() && score_trend >= 0.0 && vol_ok;
            if std::env::var("EMIT_PROBE").is_ok() {
                println!(
                    "[CONFIRM_TRACE] sym={} upd_waited={} mom={:.6} vol={:.6} vol_lim={:.6} score_trend={:.6} score_seen={} pass={}",
                    sym,
                    now_updates.saturating_sub(pending.created_symbol_updates),
                    momentum_confirm,
                    vol_confirm,
                    vol_limit,
                    score_trend,
                    score_now_opt.is_some() as i32,
                    confirmed_gate as i32
                );
            }
            if confirmed_gate {
                let confirm_updates =
                    now_updates.saturating_sub(pending.created_symbol_updates) as u32;
                confirmed.push((pending.candidate.clone(), confirm_updates));
            }
            pending_confirmations.remove(&sym);
        }

        let top_n = match rec_mode {
            RecommendationMode::Coverage => 5usize,
            RecommendationMode::Precision => 3usize,
            RecommendationMode::Top1 => 1usize,
        };
        confirmed.sort_by(|a, b| {
            b.0.score
                .partial_cmp(&a.0.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.symbol.cmp(&b.0.symbol))
        });
        for (cand, confirm_delta_symbol_updates) in confirmed.into_iter().take(top_n) {
            last_signals.insert(cand.symbol.clone(), cand.signal);
            consistency_counts.insert(cand.symbol.clone(), cand.consistency);
            let momentum_3 = history_pipes
                .get(&cand.symbol)
                .and_then(|hist| {
                    if hist.len() >= 4 {
                        let last = hist.last()?.close as f64;
                        let lag3 = hist.get(hist.len() - 4)?.close as f64;
                        Some(last - lag3)
                    } else {
                        Some(0.0)
                    }
                })
                .unwrap_or(0.0);
            let vol_5 = history_pipes
                .get(&cand.symbol)
                .map(|hist| rolling_close_std(hist, 5))
                .unwrap_or(0.0);
            let score_std_5 = {
                let history = score_history.entry(cand.symbol.clone()).or_default();
                history.push_back(cand.score);
                while history.len() > 5 {
                    history.pop_front();
                }
                let n = history.len() as f64;
                if n <= 0.0 {
                    0.0
                } else {
                    let mean = history.iter().sum::<f64>() / n;
                    let var = history
                        .iter()
                        .map(|v| {
                            let d = *v - mean;
                            d * d
                        })
                        .sum::<f64>()
                        / n;
                    var.sqrt()
                }
            };
            if std::env::var("EMIT_PROBE").is_ok() && cand.symbol == "AXISBANK.NS" {
                println!(
                    "[EMIT_TRACE] emit sym={} rec_id={} score={:.6} edge={:.6} feas={:.3} conf={:.3} voters={} S{}",
                    cand.symbol,
                    cand.rec_id,
                    cand.score,
                    cand.edge,
                    cand.feas,
                    cand.conf,
                    cand.voters,
                    cand.primary_id
                );
            }
            println!(
                "[RECOMMENDATION] rec_id={} sym={} dir={:?} score={:.6} edge={:.6} feas={:.3} conf={:.3} voters={} S{}",
                cand.rec_id,
                cand.symbol,
                cand.signal,
                cand.score,
                cand.edge,
                cand.feas,
                cand.conf,
                cand.voters,
                cand.primary_id
            );
            println!(
                "\x1b[92m[ADAPTIVE_INTENT] {} conf={:.2} feas={:.2} voters={} size={:.1}% S{}\x1b[0m",
                cand.symbol,
                cand.conf,
                cand.feas,
                cand.voters,
                cand.recommendation.position_size * 100.0,
                cand.primary_id
            );
            pending_meta
                .entry(cand.symbol.clone())
                .or_default()
                .push_back(RecMeta {
                    rec_id: cand.rec_id,
                    symbol: cand.symbol.clone(),
                    score: cand.score,
                    edge: cand.edge,
                    feas: cand.feas,
                    conf: cand.conf,
                    voters: cand.voters,
                    primary_id: cand.primary_id,
                });
            let intent_created_symbol_updates =
                *symbol_update_counts.get(&cand.symbol).unwrap_or(&0);
            paper.pending_intents.push(TradeIntent {
                rec_id: cand.rec_id,
                symbol: cand.symbol.clone(),
                signal: cand.recommendation.signal,
                reference_price: cand.recommendation.entry_price,
                recommendation: cand.recommendation,
                strategy_id: cand.primary_id,
                rec_score: cand.score,
                rec_feas: cand.feas,
                rec_conf: cand.conf,
                rec_voters: cand.voters,
                momentum_3,
                vol_5,
                score_std_5,
                consensus: None,
                age: 0,
                max_age: if cand.voters >= intent_high_voters_threshold
                    && cand.conf >= intent_high_conf_threshold
                {
                    intent_max_age_strong
                } else {
                    intent_max_age_base
                },
                intent_created_symbol_updates,
                confirm_delta_symbol_updates,
            });
            paper.intents_created = paper.intents_created.saturating_add(1);
        }
        if paper.intents_triggered > line_start_triggered {
            awr_windows_triggered = awr_windows_triggered.saturating_add(1);
        }
    }

    // End-of-stream settlement: ensure paper execution completes deterministically.
    let latest_prices: HashMap<String, f64> = history_pipes
        .iter()
        .filter_map(|(sym, hist)| hist.last().map(|c| (sym.clone(), c.close as f64)))
        .collect();
    if !paper.active_trades.is_empty() || !paper.pending_intents.is_empty() {
        println!(
            "[FINALIZE] draining active={} pending={}",
            paper.active_trades.len(),
            paper.pending_intents.len()
        );
        finalize_paper_registry(&mut paper, &latest_prices);
    }
    if candidate_probe {
        println!(
            "[CANDIDATE_STATS_SUM] batches={} total_candidates={} voters_pos={} feas_pos={} stage1_pass={} admitted={}",
            cand_batches,
            cand_total_sum,
            cand_voters_pos_sum,
            cand_feas_pos_sum,
            cand_stage1_pass_sum,
            cand_admitted_sum
        );
    }
    paper.summary();

    let avg_pnl = if !paper.pnl_history.is_empty() {
        paper.pnl_history.iter().sum::<f64>() / paper.pnl_history.len() as f64
    } else {
        0.0
    };
    let total_pnl: f64 = paper.pnl_history.iter().sum();
    let pnl_per_trigger = if paper.intents_triggered > 0 {
        total_pnl / paper.intents_triggered as f64
    } else {
        0.0
    };
    let awr = if awr_windows_total > 0 {
        awr_windows_triggered as f64 / awr_windows_total as f64
    } else {
        0.0
    };
    let trigger_rate = if awr_windows_with_candidates > 0 {
        awr_windows_triggered as f64 / awr_windows_with_candidates as f64
    } else {
        0.0
    };
    eprintln!(
        "[AWR_SUMMARY] windows_total={} windows_with_candidates={} windows_triggered={} awr={:.4} trigger_rate={:.4} created={} triggered={} expired={} closed_trades={} avg_pnl={:.6} total_pnl={:.6} pnl_per_trigger={:.6}",
        awr_windows_total,
        awr_windows_with_candidates,
        awr_windows_triggered,
        awr,
        trigger_rate,
        paper.intents_created,
        paper.intents_triggered,
        paper.intents_expired,
        paper.closed_count,
        avg_pnl,
        total_pnl,
        pnl_per_trigger
    );
}
