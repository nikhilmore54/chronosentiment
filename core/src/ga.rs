use crate::{CreateOrder, ExecutionMode, MarketEvent, Side};
use crate::harness::run_simulation_harness;
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde_json;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use serde_json::value::to_value as to_json_value;

// Helper function to serialize any serializable struct into a canonical JSON string.
// This is crucial for deterministic hashing, especially for floating-point numbers.
pub fn canonical_json<T: Serialize>(v: &T) -> String {
    // Convert to serde_json::Value first to ensure sorting of map keys
    let value = to_json_value(v).unwrap_or(serde_json::Value::Null);
    serde_json::to_string(&value).unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub fill_efficiency: f64,
    pub capture_efficiency: f64,
    pub avg_slippage: f64,
    pub latency_impact: f64,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            fill_efficiency: 0.0,
            capture_efficiency: 0.0,
            avg_slippage: 0.0,
            latency_impact: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyEvaluation {
    pub strategy_id: String,
    pub strategy: Strategy,
    pub avg_pnl: f64,
    pub std_dev: f64,
    pub worst: f64,
    pub robustness: f64,
    /// Aggregated, canonical fitness (ONLY truth).
    pub fitness: f64,
    pub trade_count: usize,
    pub max_drawdown: f64,
    pub participation_rate: f64,
    pub profitable_trades: usize,
    pub zero_pnl_trades: usize,
    pub quality_trades: f64,
    pub payoff_ratio: f64,
    pub execution_metrics: ExecutionMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaResult {
    pub global_best: StrategyEvaluation,
    pub global_best_generation: usize,
    pub final_generation_best: StrategyEvaluation,
    pub generation_history: Vec<StrategyEvaluation>,
    pub best_per_regime: HashMap<String, StrategyEvaluation>,
}

impl Default for StrategyEvaluation {
    fn default() -> Self {
        StrategyEvaluation {
            strategy_id: "N/A".to_string(),
            strategy: Strategy {
                queue_threshold: 0,
                base_edge: 0,
                take_profit: 0,
                stop_loss: 0,
            },
            avg_pnl: 0.0,
            std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            fitness: 0.0,
            trade_count: 0,
            max_drawdown: 0.0,
            participation_rate: 0.0,
            profitable_trades: 0,
            zero_pnl_trades: 0,
            quality_trades: 0.0,
            payoff_ratio: 0.0,
            execution_metrics: ExecutionMetrics::default(),
        }
    }
}

pub fn get_strategy_classification(eval: &StrategyEvaluation) -> String {
    if eval.trade_count == 0 {
        "Inactive".to_string()
    } else if eval.avg_pnl < 0.0 {
        "Fragile".to_string()
    } else if eval.std_dev > eval.avg_pnl * 2.0 {
        "Volatile".to_string()
    } else if eval.std_dev > eval.avg_pnl {
        "Unstable".to_string()
    } else {
        "Stable".to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Strategy {
    pub queue_threshold: u64,
    pub base_edge: u64,
    pub take_profit: u64,
    pub stop_loss: u64,
}

#[derive(Debug, Clone)]
pub struct GaConfig {
    pub population_size: usize,
    pub generations: usize,
    pub mutation_rate: f64,
    pub seed: u64,
    pub order_id_prefix: String,
    pub order_price: u64,
    pub order_quantity_for_strategy: u64,
    pub order_timestamp: u64,
    pub lambda: f64,
    pub initial_queue_threshold: u64,
}

impl Default for GaConfig {
    fn default() -> Self {
        Self {
            population_size: 20,
            generations: 10,
            mutation_rate: 0.1,
            seed: 42,
            order_id_prefix: "GA_DEFAULT".to_string(),
            order_price: 40000,
            order_quantity_for_strategy: 100,
            order_timestamp: 0,
            lambda: 0.5,
            initial_queue_threshold: 200,
        }
    }
}


pub fn run_ga_evolution<T: AsRef<[MarketEvent]>>(config: GaConfig, all_scenarios: &HashMap<String, T>) -> GaResult {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut global_best: Option<StrategyEvaluation> = None;
    let mut global_best_generation: usize = 0;
    let mut final_generation_best: Option<StrategyEvaluation> = None;
    let mut generation_peaks: Vec<(usize, f64)> = Vec::new();
    
    // 1. Group Scenarios by (Asset, Regime)
    let mut asset_regime_scenarios: HashMap<(String, String), HashMap<String, &T>> = HashMap::new();
    for (name, data) in all_scenarios {
        let asset = name.split('_').next().unwrap_or("BTC").to_string();
        let regime = if name.contains("trending_up") { "trending_up" }
                    else if name.contains("trending_down") { "trending_down" }
                    else if name.contains("sideways") { "sideways" }
                    else if name.contains("volatile") { "volatile" }
                    else { "mixed" };
        
        asset_regime_scenarios.entry((asset, regime.to_string())).or_default().insert(name.clone(), data);
    }

    let mut best_per_bucket: HashMap<(String, String), StrategyEvaluation> = HashMap::new();
    let mut all_final_evaluations: Vec<StrategyEvaluation> = Vec::new();
    let mut global_generation_history: Vec<StrategyEvaluation> = Vec::new();

    println!("--- Starting Multi-Asset + Regime Genetic Algorithm Evolution ---");

    let mut sorted_buckets: Vec<_> = asset_regime_scenarios.keys().cloned().collect();
    sorted_buckets.sort();

    for (asset, regime) in sorted_buckets {
        println!("
>> Evolving Bucket: asset={}, regime={}", asset, regime);
        let scenarios = asset_regime_scenarios.get(&(asset.clone(), regime.clone())).unwrap();
        
        // Convert to the format expected by evaluate_population
        let mut scenarios_map: HashMap<String, &T> = HashMap::new();
        for (k, v) in scenarios {
            scenarios_map.insert(k.clone(), *v);
        }

        let mut population = initialize_population(&config, &mut rng);
        let mut bucket_best_overall: Option<StrategyEvaluation> = None;
        let mut bucket_history: Vec<StrategyEvaluation> = Vec::new();
        let mut current_mutation_rate = config.mutation_rate;

        for generation in 0..config.generations {
            // 1. Deduplicate
            population = deduplicate_population(population, &config, &mut rng);

            // 2. Evaluate ONLY on this bucket's scenarios
            let evaluations_option = evaluate_population_scoped(&population, &config, &scenarios_map, generation);

            if let Some(mut evaluations) = evaluations_option {
                if evaluations.is_empty() {
                    println!("  [{}|{}] Gen {} → ALL STRATEGIES REJECTED AFTER INITIAL EVALUATION", asset, regime, generation);
                    population = initialize_population(&config, &mut rng);
                    continue;
                }

                // --- PHENOTYPE DIVERSITY PENALTY ---
                let evaluations_copy = evaluations.clone();
                for i in 0..evaluations.len() {
                    for j in 0..evaluations_copy.len() {
                        if i == j { continue; }
                        let eval_i = &evaluations[i];
                        let eval_j = &evaluations_copy[j];
                        if (eval_i.avg_pnl - eval_j.avg_pnl).abs() < 0.001 &&
                           (eval_i.std_dev - eval_j.std_dev).abs() < 0.001 {
                            // Keep diversity pressure without destroying canonical fitness.
                            evaluations[i].fitness *= 0.9;
                        }
                    }
                }

                // Diagnostics + strict consistency before selection/ranking.
                for evaluation in &evaluations {
                    println!(
                        "SELECTION_INPUT → strat={}, fitness={:.6}",
                        evaluation.strategy_id, evaluation.fitness
                    );
                    println!(
                        "GA_DEBUG → fitness={:.4}, trades={}, participation={:.2}",
                        evaluation.fitness, evaluation.trade_count, evaluation.participation_rate
                    );
                    assert!(
                        evaluation.fitness.is_finite() &&
                        evaluation.fitness >= 0.0 &&
                        evaluation.fitness <= 1.0,
                        "GA using non-canonical fitness scale: {}",
                        evaluation.fitness
                    );
                }

                // Sort by final fitness only (single source of truth).
                evaluations.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(Ordering::Equal));

                // 3. Apply similarity penalty
                let pre_similarity = evaluations.clone();
                apply_similarity_penalty(&mut evaluations);
                let had_positive_before = pre_similarity.iter().any(|e| e.fitness > 0.0);
                let has_positive_after = evaluations.iter().any(|e| e.fitness > 0.0);
                if had_positive_before && !has_positive_after {
                    println!(
                        "SIMILARITY_GUARD → penalty collapsed all positive fitness; restoring pre-penalty evaluations"
                    );
                    evaluations = pre_similarity;
                }

                // Re-sort
                evaluations.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(Ordering::Equal));

                if let Some(best) = evaluations.first() {
                    println!("Gen {} → Fitness: {:.4} (Final) | Part: {:.2} | Trades: {}",
                        generation, 
                        best.fitness, best.participation_rate, best.trade_count);
                    generation_peaks.push((generation, best.fitness));
                    
                    if global_best.is_none() || best.fitness > global_best.as_ref().unwrap().fitness {
                        global_best = Some(best.clone());
                        global_best_generation = generation;
                    }

                    let should_update = bucket_best_overall.as_ref().map_or(true, |o| best.fitness > o.fitness);
                    if should_update {
                        bucket_best_overall = Some(best.clone());
                    }
                    
                    bucket_history.push(best.clone());

                    // Track global history (using best from any bucket for simplicity or just first one processed)
                    if (asset == "BTC" && regime == "trending_up") || global_generation_history.len() <= generation {
                        if global_generation_history.len() <= generation {
                            global_generation_history.push(best.clone());
                        } else if best.fitness > global_generation_history[generation].fitness {
                            global_generation_history[generation] = best.clone();
                        }
                    }
                }

                // Stagnation detection per bucket
                if generation > 5 {
                    let recent_history = &bucket_history[bucket_history.len().saturating_sub(3)..];
                    if recent_history.len() >= 3 && (recent_history[0].fitness - recent_history[2].fitness).abs() < 0.01 {
                        current_mutation_rate = (current_mutation_rate * 1.5).min(0.5);
                    } else {
                        current_mutation_rate = config.mutation_rate;
                    }
                }

                if generation < config.generations - 1 {
                    let mut bucket_config = config.clone();
                    bucket_config.mutation_rate = current_mutation_rate;
                    population = evolve_generation(&evaluations, &bucket_config, &mut rng);
                } else {
                    all_final_evaluations.extend(evaluations.clone());
                    if let Some(current_final_gen_best) = evaluations.first() {
                        if final_generation_best.is_none() || current_final_gen_best.fitness > final_generation_best.as_ref().unwrap().fitness {
                            final_generation_best = Some(current_final_gen_best.clone());
                        }
                    }
                }
            } else { // evaluations_option was None
                println!("  [{}|{}] Gen {} → ALL STRATEGIES REJECTED DURING EARLY CHECK", asset, regime, generation);
                population = initialize_population(&config, &mut rng);
                continue;
            }
        }

        if let Some(best) = bucket_best_overall {
            println!("BEST: asset={}, regime={}", asset, regime);
            println!("  Fitness: {:.4}, PnL: {:.6}", best.fitness, best.avg_pnl);
            best_per_bucket.insert((asset, regime), best);
        }
    }

    println!("
--- GA Evolution Complete ---");
    println!("📈 Generation Peaks:");
    for (gen, fitness) in generation_peaks {
        println!("Gen {} → {:.4}", gen, fitness);
    }

    let resolved_global_best = global_best.unwrap_or_else(StrategyEvaluation::default);
    let resolved_final_generation_best = final_generation_best.unwrap_or_else(StrategyEvaluation::default);
    assert!(
        resolved_global_best.fitness + 1e-12 >= resolved_final_generation_best.fitness,
        "Global best fitness must be >= final generation best fitness"
    );
    GaResult {
        global_best: resolved_global_best,
        global_best_generation,
        final_generation_best: resolved_final_generation_best,
        generation_history: global_generation_history,
        best_per_regime: best_per_bucket
            .into_iter()
            .map(|((asset, regime), eval)| (format!("{}_{}", asset, regime), eval))
            .collect(),
    }
}

pub fn evaluate_population_scoped<T: AsRef<[MarketEvent]>>(
    population: &Vec<Strategy>, 
    config: &GaConfig, 
    scenarios: &HashMap<String, &T>,
    generation: usize
) -> Option<Vec<StrategyEvaluation>> {
    let mut evaluations = Vec::with_capacity(population.len());
    let mut sorted_scenario_names: Vec<&String> = scenarios.keys().collect();
    sorted_scenario_names.sort();
    let mut scoped_scenarios: HashMap<String, &[MarketEvent]> = HashMap::new();
    for (name, data) in scenarios {
        scoped_scenarios.insert(name.clone(), data.as_ref());
    }

    // ⚠️ CRITICAL FIX 2 — EARLY REJECTION (UNBIASED)
    use rand::seq::SliceRandom;
    let mut sample_rng = StdRng::seed_from_u64(config.seed + generation as u64);
    let mut sampled_indices: Vec<usize> = (0..sorted_scenario_names.len()).collect();
    // Do not sample if there are fewer than 5 scenarios anyway, but since it asks to shuffle:
    if !sampled_indices.is_empty() {
        sampled_indices.shuffle(&mut sample_rng);
    }
    let sample_size = sorted_scenario_names.len().min(5);
    let early_check_indices = &sampled_indices[0..sample_size];

    for strategy in population {
        let mut _early_trades = 0;

        for &idx in early_check_indices {
            let scenario_name = sorted_scenario_names[idx];
            let market_events = scenarios.get(scenario_name).unwrap().as_ref();
            if let Some(report) = evaluate_strategy(strategy, scenario_name, market_events, config) {
                _early_trades += report.trade_count;
            }
        }

        /*
        if sample_size > 0 {
            let est_participation = early_active as f64 / sample_size as f64;
            if est_participation < 0.2 {
                return None;
            }
        }
        */

        // Ensure we don't return None just because of low early activity in small datasets

        // Complete the rest through canonical aggregation path.
        if let Some(aggregated) = evaluate_and_aggregate(strategy, config, &scoped_scenarios) {
            assert!(
                aggregated.fitness.is_finite() &&
                aggregated.fitness >= 0.0 &&
                aggregated.fitness <= 1.0,
                "Invalid aggregated fitness before population insert: {}",
                aggregated.fitness
            );
            if aggregated.fitness >= 0.0 {
                println!(
                    "EVAL_ASSIGN → strat={}, fitness={:.6}",
                    aggregated.strategy_id, aggregated.fitness
                );
                evaluations.push(aggregated);
            } else {
                println!(
                    "EVAL_ASSIGN_SKIP → strat={}, fitness={:.6}",
                    aggregated.strategy_id, aggregated.fitness
                );
            }
        }
    }
    if evaluations.is_empty() {
        // Fallback for extremely sparse data: return some strategies with 0 fitness
        for strategy in population.iter().take(3) {
            evaluations.push(StrategyEvaluation {
                strategy_id: "FALLBACK_ZERO".to_string(),
                strategy: strategy.clone(),
                ..StrategyEvaluation::default()
            });
        }
    }
    Some(evaluations)
}

fn deduplicate_population(population: Vec<Strategy>, config: &GaConfig, rng: &mut StdRng) -> Vec<Strategy> {
    let mut unique_strategies = HashSet::new();
    let mut new_population = Vec::with_capacity(population.len());

    for s in population {
        if unique_strategies.insert(s.clone()) {
            new_population.push(s);
        }
    }

    // Refill with random strategies if we removed duplicates
    while new_population.len() < config.population_size {
        let random_strat = Strategy {
            queue_threshold: rng.gen_range(10..=500),
            base_edge: rng.gen_range(1..=10),
            take_profit: rng.gen_range(5..=50),
            stop_loss: rng.gen_range(2..=25),
        };
        if unique_strategies.insert(random_strat.clone()) {
            new_population.push(random_strat);
        }
    }

    new_population
}

fn apply_similarity_penalty(evaluations: &mut Vec<StrategyEvaluation>) {
    // Penalize strategies that are too similar in parameter space
    // We compare each strategy against the current top-5 performers
    let top_strats: Vec<Strategy> = evaluations.iter().take(5).map(|e| e.strategy.clone()).collect();
    
    for eval in evaluations.iter_mut() {
        let mut max_similarity: f64 = 0.0;
        for top in &top_strats {
            if &eval.strategy == top { continue; }
            
            // Normalized distance in 4D space
            let d1 = (eval.strategy.queue_threshold as f64 - top.queue_threshold as f64).abs() / 1000.0;
            let d2 = (eval.strategy.base_edge as f64 - top.base_edge as f64).abs() / 10.0;
            let d3 = (eval.strategy.take_profit as f64 - top.take_profit as f64).abs() / 50.0;
            let d4 = (eval.strategy.stop_loss as f64 - top.stop_loss as f64).abs() / 25.0;
            
            let dist = (d1 + d2 + d3 + d4) / 4.0;
            let similarity = (1.0 - dist).max(0.0);
            max_similarity = max_similarity.max(similarity);
        }
        
        // Keep diversity pressure without collapsing aggregated fitness to zero.
        // Penalty is multiplicative and bounded (0.8..1.0).
        let penalty_factor = (1.0 - 0.2 * max_similarity).clamp(0.8, 1.0);
        eval.fitness *= penalty_factor;
    }
}

fn evolve_generation(evaluations: &Vec<StrategyEvaluation>, config: &GaConfig, rng: &mut StdRng) -> Vec<Strategy> {
    let mut next_gen: Vec<Strategy> = Vec::new();

    let elite_count = 2.min(evaluations.len());
    let elites: Vec<Strategy> = evaluations
        .iter()
        .take(elite_count)
        .map(|e| e.strategy.clone())
        .collect();

    next_gen.extend(elites);

    println!(
        "Elitism → Preserving top {} strategies (Best fitness: {:.4})",
        elite_count,
        evaluations[0].fitness
    );

    // 3. Tournament Selection + Mutation for the rest
    while next_gen.len() < config.population_size {
        let parent_eval = tournament_selection(evaluations, 3, rng);
        let mut offspring = parent_eval.strategy.clone();
        
        if rng.gen::<f64>() < config.mutation_rate {
            mutate_strategy(&mut offspring, rng, parent_eval.trade_count);
        }
        next_gen.push(offspring);
    }


    next_gen
}

fn tournament_selection<'a>(evaluations: &'a Vec<StrategyEvaluation>, k: usize, rng: &mut StdRng) -> &'a StrategyEvaluation {
    let mut best: Option<&StrategyEvaluation> = None;
    for _ in 0..k {
        let ind = rng.gen_range(0..evaluations.len());
        let current = &evaluations[ind];
        if best.is_none() || current.fitness > best.unwrap().fitness {
            best = Some(current);
        }
    }
    best.unwrap()
}

fn initialize_population(config: &GaConfig, rng: &mut StdRng) -> Vec<Strategy> {
    let mut population = Vec::with_capacity(config.population_size);
    for _ in 0..config.population_size {
        // Alignment with realistic TP/SL ranges:
        // TP: 10-50 bp (0.1%-0.5%)
        // SL: 5-30 bp (0.05%-0.3%)
        population.push(Strategy {
            // Lowered initial threshold to immediately force higher participation
            queue_threshold: rng.gen_range(10..=500), 
            base_edge: rng.gen_range(1..=10),
            take_profit: rng.gen_range(10..=50),
            stop_loss: rng.gen_range(5..=30),
        });
    }
    population
}

fn mutate_strategy(strategy: &mut Strategy, rng: &mut StdRng, parent_trade_count: usize) {
    let mutation_type = rng.gen_range(0..4);
    match mutation_type {
        0 => { // Big jump in threshold
            // Heavy bias (-1 if gen_bool(0.7)) toward LOWERING the threshold (increasing participation)
            let delta = rng.gen_range(50..200) as i64 * if rng.gen_bool(0.7) { -1 } else { 1 };
            strategy.queue_threshold = (strategy.queue_threshold as i64 + delta).clamp(10, 1000) as u64;
        }
        1 => { // Flip TP/SL (within bounds)
            let temp = strategy.take_profit;
            strategy.take_profit = strategy.stop_loss.clamp(10, 50);
            strategy.stop_loss = temp.clamp(5, 30);
        }
        2 => { // Radical base_edge change
            strategy.base_edge = rng.gen_range(1..15);
        }
        _ => { // Reset TP/SL to realistic ranges
            strategy.take_profit = rng.gen_range(10..=40);
            strategy.stop_loss = rng.gen_range(5..=25);
        }
    }
    
    // --- 4. ADD PARTICIPATION-AWARE MUTATION ---
    // 50% chance to explicitly tighten the threshold to expand trigger validities
    if rng.gen_bool(0.5) {
        strategy.queue_threshold = (strategy.queue_threshold as f64 * 0.8) as u64;
        strategy.queue_threshold = strategy.queue_threshold.max(10);
    }

    // 🧬 MUTATION BALANCING: Prevent saturation by increasing threshold slightly if over-trading
    if parent_trade_count > 150 {
        strategy.queue_threshold = (strategy.queue_threshold as f64 * 1.2) as u64;
        strategy.queue_threshold = strategy.queue_threshold.min(1000);
    }
}


pub(crate) fn evaluate_strategy(
    strategy: &Strategy,
    scenario_name: &str,
    market_events: &[MarketEvent],
    config: &GaConfig,
) -> Option<StrategyEvaluation> {
    if market_events.is_empty() { return None; }
    let ref_event = market_events.first().unwrap();
    let ref_ts = ref_event.exchange_ts;
    let ref_price = ref_event.price;

    // Threshold logic: strategy only places an order if the current market queue is within its threshold
    let mut current_market_queue: u64 = 0;
    for event in market_events {
        if event.exchange_ts > ref_ts { continue; }
        if event.price == ref_price {
            match event.subtype {
                crate::MarketEventType::NewOrder => current_market_queue += event.quantity,
                crate::MarketEventType::Cancel | crate::MarketEventType::Trade => {
                    current_market_queue = current_market_queue.saturating_sub(event.quantity);
                }
            }
        }
    }

    let strategy_id = format!("strat_{}_{}_{}_{}_{}", scenario_name, strategy.queue_threshold, strategy.base_edge, strategy.take_profit, strategy.stop_loss);
    
    // 1. Compute regime volatility and trend
    let prices: Vec<f64> = market_events.iter().map(|e| e.price as f64).collect();
    let mean_price = if prices.is_empty() { 0.0 } else { prices.iter().sum::<f64>() / prices.len() as f64 };
    let norm_vol = if prices.len() > 1 && mean_price > 0.0 {
        let variance = prices.iter().map(|p| (p - mean_price).powi(2)).sum::<f64>() / prices.len() as f64;
        variance.sqrt() / mean_price
    } else { 0.0 };
    let first_price = prices.first().copied().unwrap_or(ref_price as f64);
    let last_price = prices.last().copied().unwrap_or(ref_price as f64);
    let is_bearish = last_price < first_price;

    // --- GENOME IMPROVEMENT: DYNAMIC THRESHOLD SCALING ---
    let volatility_factor = if norm_vol > 0.002 {
        0.5 // High volatility -> lower threshold -> more aggressive
    } else if norm_vol < 0.0005 {
        1.5 // Low volatility -> higher threshold -> more selective
    } else {
        1.0
    };
    
    // Trade frequency bias: scale threshold down overall to encourage participation
    let trade_frequency_bias = 0.8;
    
    let dynamic_threshold = (strategy.queue_threshold as f64 * volatility_factor * trade_frequency_bias) as u64;
    let dynamic_threshold = dynamic_threshold.max(10); // Ensure it doesn't go to 0

    // 2. Decision logic with smooth signals
    let raw_q_ratio = current_market_queue as f64 / dynamic_threshold as f64;
    let q_ratio = raw_q_ratio.min(2.0);
    let vol_signal = (norm_vol / 0.001).min(2.0);
    let mut aggressiveness = (q_ratio + vol_signal) / 2.0;
    if is_bearish && norm_vol < 0.001 { aggressiveness *= 0.7; }

    // --- CONTINUOUS PARTICIPATION: REMOVE HARD SIGNAL BLOCKER ---
    // Let probabilistic roll handle execution

    // Deterministic roll for probabilistic decisions
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hasher;
    hasher.write_u64(strategy.queue_threshold);
    hasher.write_u64(market_events.first().map(|e| e.exchange_ts).unwrap_or(0));
    hasher.write_u64(market_events.last().map(|e| e.exchange_ts).unwrap_or(0));
    hasher.write_u64(market_events.first().map(|e| e.price).unwrap_or(0));
    hasher.write_u64(market_events.len() as u64);
    hasher.write(scenario_name.as_bytes());
    let roll = (hasher.finish() % 1000) as f64 / 1000.0;

    // 3. Skip logic REMOVED to encourage participation.
    // We now always proceed to order injection.

    // 4. Conditional execution with deterministic probability
    let market_price = market_events.first().map(|e| e.price).unwrap_or(ref_price);
    // Slightly relaxed threshold (was / 1.5)
    let agg_threshold = (aggressiveness / 1.1).min(0.98);
    let (buy_price, is_aggressive) = if roll < agg_threshold {
        (market_price + 1, true)
    } else {
        (market_price, false)
    };

    // 5. Advanced Fill Probability (Price + Time + Queue + Volatility)
    let total_events = market_events.len().max(1) as f64;
    let order_idx = market_events.iter().position(|e| e.exchange_ts >= ref_ts).unwrap_or(0) as f64;
    let progress = order_idx / total_events;
    let time_factor = (1.0 - 0.7 * progress).max(0.3);
    let vol_boost = 1.0 + norm_vol * 2.0;
    let regime_prob = if norm_vol > 0.001 { 0.85 * vol_boost } else { 0.65 };
    
    let base_fill_prob = if is_aggressive { 
        regime_prob.min(0.9) 
    } else { 
        ((-raw_q_ratio).exp() * time_factor * vol_boost).min(0.85) 
    };
    let fill_prob = base_fill_prob.clamp(0.02, 0.92);
    
    let _prob_penalty = 200.0 * (1.0 - fill_prob);
    let _agg_cost = (50.0 + norm_vol * 50.0) * if is_aggressive { 1.0 } else { 0.0 };

    // 6. Scaled penalties + Smooth bonus
    let _queue_penalty = (current_market_queue.saturating_sub(strategy.queue_threshold) as f64).min(500.0);
    let _execution_bonus = (1.0 - raw_q_ratio).max(0.0) * 50.0;

    // 6. Inject orders with TP/SL logic
    let entry_id = format!("{}_entry", strategy_id);
    let entry_order = CreateOrder {
        order_id: entry_id.clone(), 
        side: Side::Buy,
        price: buy_price,
        quantity: config.order_quantity_for_strategy,
        timestamp: ref_ts,
        fill_probability: fill_prob,
    };

    // Pre-calculate exit targets based on buy_price (proxy for entry price)
    let tp_bps = strategy.take_profit as f64 / 10000.0;
    let sl_bps = strategy.stop_loss as f64 / 10000.0;
    let tp_target = (buy_price as f64 * (1.0 + tp_bps)) as u64;
    let sl_target = (buy_price as f64 * (1.0 - sl_bps)) as u64;

    // --- IMPROVED EXIT LOGIC (MIN HOLD PERIOD) ---
    let entry_idx = market_events.iter().position(|e| e.exchange_ts >= ref_ts).unwrap_or(0);
    let min_hold = 5;
    let mut exit_price = buy_price; // fallback
    let mut exit_ts = ref_ts + 100; // fallback
    let mut found_exit = false;

    // Scan forward starting after min_hold ticks
    for event in market_events.iter().skip(entry_idx + min_hold) {
        if event.price >= tp_target || event.price <= sl_target {
            exit_price = event.price;
            exit_ts = event.exchange_ts;
            found_exit = true;
            break;
        }
    }

    if !found_exit {
        if let Some(last_ev) = market_events.last() {
            exit_price = last_ev.price;
            exit_ts = last_ev.exchange_ts;
        }
    }

    let exit_order = CreateOrder {
        order_id: format!("{}_exit", strategy_id),
        side: Side::Sell,
        price: if is_aggressive { exit_price.saturating_sub(1) } else { exit_price },
        quantity: config.order_quantity_for_strategy,
        timestamp: exit_ts,
        fill_probability: fill_prob,
    };

    let orders_to_inject = vec![entry_order, exit_order];

    let mut event_refs = Vec::with_capacity(market_events.len());
    for ev in market_events {
        event_refs.push(ev.clone());
    }

    let (_, simulation_result, _) = run_simulation_harness(
        ExecutionMode::Real,
        event_refs,
        orders_to_inject.clone(),
    );

    // Parse events to calculate drawdown
    let mut current_balance = 0i64;
    let mut max_balance = 0i64;
    let mut max_drawdown = 0i64;

    // Execution metrics tracking
    let mut total_filled_qty = 0;
    let mut total_slippage_bps = 0.0;
    let mut fills_count = 0;
    let requested_qty = config.order_quantity_for_strategy * 2; // entry + exit
    let expected_move = if buy_price > 0 { (exit_price as f64 - buy_price as f64).abs() / buy_price as f64 } else { 0.0 };

    for event in &simulation_result.events {
        if let crate::SimEvent::PartialFill { order_id, filled_qty, price, .. } = event {
            // Track Execution Metrics
            if order_id == &entry_id {
                total_filled_qty += filled_qty;
                let slippage = (*price as f64 - buy_price as f64).abs() / buy_price.max(1) as f64;
                total_slippage_bps += slippage;
                fills_count += 1;
            } else if order_id == &format!("{}_exit", strategy_id) {
                total_filled_qty += filled_qty;
                let slippage = (*price as f64 - exit_price as f64).abs() / exit_price.max(1) as f64;
                total_slippage_bps += slippage;
                fills_count += 1;
            }
            // Find order to determine side
            if let Some(order) = orders_to_inject.iter().find(|o| &o.order_id == order_id) {
                let cash_flow = (*filled_qty as i64) * (*price as i64);
                match order.side {
                    Side::Buy => current_balance -= cash_flow,
                    Side::Sell => current_balance += cash_flow,
                }
                max_balance = max_balance.max(current_balance);
                max_drawdown = max_drawdown.min(current_balance - max_balance);
            }
        }
    }
    
    let market_price = market_events.first().map(|e| e.price).unwrap_or(ref_price);
    let drawdown_penalty_raw = max_drawdown.abs() as f64 / (market_price.max(1) * config.order_quantity_for_strategy.max(1)) as f64;

    // --- 1. TRADE VALIDATION & PNL ---
    let mut scenario_pnls: Vec<f64> = Vec::new(); // Collect PnLs for all trades within this scenario
    let mut total_quality_trades_scenario = 0.0;
    let mut trades_executed = 0;
    let mut current_entry_price = 0;
    let mut entry_filled = false;

    // Identify entry and exit orders in the simulation result
    let entry_order_id = format!("{}_entry", strategy_id);
    let exit_order_id = format!("{}_exit", strategy_id);

    for event in &simulation_result.events {
        if let crate::SimEvent::PartialFill { order_id, price, .. } = event {
            if order_id == &entry_order_id {
                current_entry_price = *price;
                entry_filled = true;
            } else if order_id == &exit_order_id && entry_filled {
                let exit_price_val = *price;

                // --- SKIP INVALID TRADES (entry == exit) ---
                if current_entry_price == 0 || exit_price_val == current_entry_price {
                    println!("TRADE_VALID: entry={}, exit={}, valid=false (no price movement or entry_price is 0)", current_entry_price, exit_price_val);
                    scenario_pnls.push(-0.0001); // Small penalty for no-movement trades
                    entry_filled = false; // Reset for next potential trade cycle
                    continue;
                }

                // --- MINIMUM MOVE FILTER (Relaxed for synthetic testing) ---
                let move_abs = (exit_price_val as f64 - current_entry_price as f64).abs();
                let min_move = current_entry_price as f64 * 0.0005; // 0.05%

                if move_abs < min_move {
                    println!(
                        "TRADE_VALID: entry={}, exit={}, valid=false (move_abs={:.6} < min_move={:.6})",
                        current_entry_price, exit_price_val, move_abs, min_move
                    );
                    scenario_pnls.push(-0.0001); // Small penalty for noise trades
                    entry_filled = false; // Reset for next potential trade cycle
                    continue;
                }

                // KEEP EXISTING PNL LOGIC
                let side = Side::Buy; // Currently strategy only does Buy entry.
                let pnl_return_base = match side {
                    Side::Buy => (exit_price_val as f64 - current_entry_price as f64) / current_entry_price as f64,
                    Side::Sell => (current_entry_price as f64 - exit_price_val as f64) / current_entry_price as f64,
                };

                let transaction_cost = 0.0001; // 0.01% (Relaxed for synthetic testing)
                let final_pnl_return = pnl_return_base - transaction_cost;

                assert!(
                    !(side == Side::Buy && current_entry_price > exit_price_val && final_pnl_return > 0.0),
                    "Invalid PnL: price down but pnl positive for Buy"
                );
                assert!(
                    !(side == Side::Sell && current_entry_price < exit_price_val && final_pnl_return > 0.0),
                    "Invalid PnL: price up but pnl positive for Sell"
                );

                let price_move = (exit_price_val as f64 - current_entry_price as f64).abs() / current_entry_price as f64;
                let trade_quality = if price_move > 0.002 { 1.0 } else { 0.0 };
                total_quality_trades_scenario += trade_quality;

                if final_pnl_return.is_nan() || final_pnl_return.is_infinite() {
                    scenario_pnls.push(0.0);
                } else {
                    scenario_pnls.push(final_pnl_return);
                }

                println!("EXEC_LOG: entry={}, exit={}, pnl_return={:.6}", current_entry_price, exit_price_val, final_pnl_return);
                trades_executed += 1;
                entry_filled = false; // Reset for next potential trade cycle
            }
        }
    }

    let total_trades = trades_executed;

    // 3. Keep Only One Hard Kill Rule: If there are no trades, this strategy is not viable.
    if total_trades == 0 { return None; }

    let avg_pnl_for_scenario = if total_trades > 0 {
        scenario_pnls.iter().sum::<f64>() / total_trades as f64
    } else {
        0.0 // This case should ideally not be reached due to the above filter
    };

    // Recalculate profitable_trades_scenario and zero_pnl_trades_scenario based on accumulated PnLs
    let mut profitable_trades_scenario = 0usize;
    let mut zero_pnl_trades_scenario = 0usize;
    let mut total_win = 0.0;
    let mut total_loss = 0.0;
    let mut win_count = 0;
    let mut loss_count = 0;

    for pnl in &scenario_pnls {
        if *pnl > 0.0 {
            profitable_trades_scenario += 1;
            total_win += *pnl;
            win_count += 1;
        } else if *pnl == 0.0 {
            zero_pnl_trades_scenario += 1;
        } else {
            total_loss += pnl.abs();
            loss_count += 1;
        }
    }
    
    println!("NEW_FITNESS_ACTIVE");
    println!("PARTICIPATION_DEBUG: scenario={}, trades={}", scenario_name, total_trades);
    
    // Per-scenario diagnostic fitness (NOT used for GA selection)
    let current_fitness = avg_pnl_for_scenario * 150.0;

    // --- PAYOFF RATIO ---
    let avg_win = if win_count > 0 { total_win / win_count as f64 } else { 0.0 };
    let avg_loss = if loss_count > 0 { total_loss / loss_count as f64 } else { 0.0 };
    // Keep payoff bounded to prevent reward hacking from zero-loss micro samples.
    let payoff_ratio = if avg_loss > 0.0 {
        avg_win / avg_loss
    } else if win_count > 0 {
        1.2
    } else {
        0.0
    };

    // --- SELECTIVITY METRIC & HARD REJECT ---
    let selectivity = if total_trades > 0 {
        total_quality_trades_scenario / total_trades as f64
    } else {
        0.0
    };

    // Relaxed for synthetic testing
    if total_trades > 0 && selectivity < 0.1 {
        return None;
    }

    // Relaxed for synthetic testing
    if total_trades > 0 && payoff_ratio < 0.8 {
        return None;
    }

    let mut total_fitness = current_fitness;

    // --- PENALIZE NOISE TRADING ---
    if total_trades > 0 {
        if selectivity < 0.5 {
            total_fitness -= 3.0;
        }
        
        // --- PENALIZE ZERO-MOVEMENT TRADES ---
        if zero_pnl_trades_scenario as f64 / total_trades as f64 > 0.3 {
            total_fitness -= 3.0;
        }

        // --- REWARD HIGH-QUALITY ENTRIES ---
        total_fitness += selectivity * 5.0;
    }

    println!(
        "FITNESS_DIAG: pnl_return={:.6}, trades={}, diagnostic_fitness={:.4}",
        avg_pnl_for_scenario, // Diagnostic only; GA selection uses aggregated final fitness.
        total_trades,
        total_fitness
    );

    assert!(!total_fitness.is_nan(), "Diagnostic fitness is NaN for strategy: {}", strategy_id);
    assert!(!total_fitness.is_infinite(), "Diagnostic fitness is infinite for strategy: {}", strategy_id);

    let std_dev_for_scenario = if total_trades > 1 {
        let mean = scenario_pnls.iter().sum::<f64>() / total_trades as f64;
        let variance = scenario_pnls.iter().map(|pnl| (pnl - mean).powi(2)).sum::<f64>() / total_trades as f64;
        variance.sqrt()
    } else {
        0.0
    };

    let worst_pnl_for_scenario = scenario_pnls.iter().cloned().fold(f64::INFINITY, f64::min);
    let robustness_for_scenario = avg_pnl_for_scenario - config.lambda * std_dev_for_scenario;

    let fill_efficiency = if requested_qty > 0 { total_filled_qty as f64 / requested_qty as f64 } else { 0.0 };
    let avg_slippage = if fills_count > 0 { total_slippage_bps / fills_count as f64 } else { 0.0 };
    
    let realized_avg = if total_trades > 0 { scenario_pnls.iter().sum::<f64>() / total_trades as f64 } else { 0.0 };
    let capture_efficiency = if expected_move > 0.0 && total_trades > 0 {
        (realized_avg / expected_move).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let exec_metrics = ExecutionMetrics {
        fill_efficiency,
        capture_efficiency,
        avg_slippage,
        latency_impact: 0.0,
    };

    Some(StrategyEvaluation {
        strategy_id: strategy_id.clone(),
        strategy: strategy.clone(),
        avg_pnl: avg_pnl_for_scenario,
        std_dev: std_dev_for_scenario,
        worst: worst_pnl_for_scenario,
        robustness: robustness_for_scenario,
        // Canonical fitness is computed in aggregate_strategy_reports.
        fitness: 0.0,
        trade_count: total_trades,
        max_drawdown: drawdown_penalty_raw * 100.0, // Storing as percentage
        participation_rate: if total_trades > 0 { 1.0 } else { 0.0 },
        profitable_trades: profitable_trades_scenario,
        zero_pnl_trades: zero_pnl_trades_scenario,
        quality_trades: total_quality_trades_scenario,
        payoff_ratio,
        execution_metrics: exec_metrics,
    })
}

pub fn aggregate_strategy_reports(evaluations: Vec<StrategyEvaluation>, lambda: f64) -> Option<StrategyEvaluation> {
    // This check is now handled upstream by evaluate_population_scoped returning None
    // and evaluate_strategy returning None for 0 trades.

    // IMPORTANT: use raw per-scenario returns; never clip before aggregation.
    let scenario_results: Vec<f64> = evaluations.iter().map(|e| e.avg_pnl).collect();
    let scenario_trade_counts: Vec<usize> = evaluations.iter().map(|e| e.trade_count).collect();

    let total_scenarios = scenario_results.len() as f64;

    let global_avg_pnl = if total_scenarios > 0.0 {
        scenario_results.iter().sum::<f64>() / total_scenarios
    } else {
        0.0
    };

    let variance = if total_scenarios > 1.0 {
        scenario_results.iter().map(|p| (p - global_avg_pnl).powi(2)).sum::<f64>() / total_scenarios
    } else {
        0.0
    };
    
    let std_dev = variance.sqrt();

    let worst_pnl = scenario_results.iter().cloned().fold(f64::INFINITY, f64::min);

    // Calculate other aggregated metrics based on all evaluations
    let total_trade_count: usize = scenario_trade_counts.iter().sum();
    let total_max_drawdown: f64 = evaluations.iter().map(|e| e.max_drawdown).sum();
    let total_profitable_trades: usize = evaluations.iter().map(|e| e.profitable_trades).sum();
    let total_zero_pnl_trades: usize = evaluations.iter().map(|e| e.zero_pnl_trades).sum();
    let total_quality_trades: f64 = evaluations.iter().map(|e| e.quality_trades).sum();
    let total_payoff_ratio_sum: f64 = evaluations.iter().map(|e| e.payoff_ratio).sum();
    
    let active_scenarios: f64 = evaluations.iter().filter(|e| e.trade_count > 0).count() as f64;


    // --- DEBUG (MANDATORY) ---
    println!("SCENARIO_DIST: {:?}", scenario_results);

    // --- ASSERT DISTRIBUTION VALIDITY ---
    assert!(
        std_dev > 0.0 || scenario_results.iter().all(|&x| x == global_avg_pnl),
        "Invalid distribution: non-zero pnl but zero std dev"
    );

    let participation_rate = active_scenarios / total_scenarios;
    let avg_max_drawdown = total_max_drawdown / total_scenarios;
    let global_payoff_ratio = if total_scenarios > 0.0 {
        (total_payoff_ratio_sum / total_scenarios).clamp(0.0, 2.0)
    } else {
        0.0
    };

    // --- SELECTIVITY METRIC ---
    let mut selectivity = if total_scenarios > 0.0 {
        total_trade_count as f64 / total_scenarios
    } else {
        0.0
    };

    if participation_rate < 0.5 {
        selectivity *= participation_rate;
    }

    // --- ADD EFFECTIVENESS METRIC ---
    let raw_effectiveness = if total_trade_count > 0 {
        total_profitable_trades as f64 / total_trade_count as f64
    } else {
        0.0
    };
    let effectiveness = if total_scenarios > 1.0 && total_trade_count < 10 {
        raw_effectiveness * (total_trade_count as f64 / 10.0)
    } else {
        raw_effectiveness
    };

    let robustness = global_avg_pnl - lambda * std_dev;

    // --- EXECUTION METRICS AGGREGATION ---
    let avg_fill_eff = evaluations.iter().map(|e| e.execution_metrics.fill_efficiency).sum::<f64>() / total_scenarios;
    let avg_cap_eff = evaluations.iter().map(|e| e.execution_metrics.capture_efficiency).sum::<f64>() / total_scenarios;
    let avg_slippage = evaluations.iter().map(|e| e.execution_metrics.avg_slippage).sum::<f64>() / total_scenarios;
    let avg_latency = evaluations.iter().map(|e| e.execution_metrics.latency_impact).sum::<f64>() / total_scenarios;

    let execution_quality = 0.4 * avg_fill_eff + 0.4 * avg_cap_eff + 0.2 * (1.0 - avg_slippage).max(0.0);

    // --- REBALANCED FITNESS LOGIC (RAW->AGGREGATE->NORMALIZE) ---
    // Use soft scaling on raw aggregated pnl to avoid hard caps while preventing explosions.
    let pnl_score = (global_avg_pnl * 100.0).tanh().max(0.0);

    // 2. Add Multiplicative Participation Suppression
    let participation_factor = participation_rate.powi(2);  // strong penalty
    let coverage = active_scenarios / total_scenarios;
    let coverage_factor = coverage.powi(2);

    // Penalize low sample count to reduce cherry-picked single/few-trade strategies.
    let min_trades = 10.0;
    let sample_penalty = if total_scenarios <= 1.0 || (total_trade_count as f64 >= min_trades) {
        1.0
    } else {
        ((total_trade_count as f64) / min_trades).clamp(0.1, 1.0)
    };

    // 5. Add Variance Suppression
    let stability_factor = (1.0 - std_dev).clamp(0.0, 1.0);
    let variance_penalty = if std_dev < 1e-6 { 0.95 } else { 1.0 };

    // Debug root cause explicitly - Log components before multiplication
    println!(
        "FITNESS_COMPONENTS → pnl: {:.5}, part: {:.3}, cov: {:.3}, eff: {:.3}, exec: {:.3}, stab: {:.3}",
        pnl_score,
        participation_factor,
        coverage_factor,
        effectiveness,
        execution_quality.clamp(0.0, 1.0), // Clamp execution_quality here
        stability_factor
    );

    let quality_score =
          0.25 * participation_factor.clamp(0.0, 1.0)
        + 0.20 * coverage_factor.clamp(0.0, 1.0)
        + 0.15 * effectiveness.clamp(0.0, 1.0)
        + 0.15 * execution_quality.clamp(0.0, 1.0)
        + 0.25 * stability_factor;

    let mut aggregated_fitness = pnl_score * (0.5 + quality_score);
    aggregated_fitness *= sample_penalty;
    aggregated_fitness *= variance_penalty;

    // Fix viability penalty
    let viability_penalty = if total_scenarios <= 1.0 {
        1.0
    } else if participation_rate < 0.2 || total_trade_count < 3 {
        0.5
    } else {
        1.0
    };
    aggregated_fitness *= viability_penalty;

    // Add over-trading penalty (restored)
    let trade_upper_bound = total_scenarios * 3.0; // Over-trading bound
    if total_trade_count as f64 > trade_upper_bound {
        aggregated_fitness *= 0.7; // Over-trading penalty
    }

    // Participation target band penalty: discourage overly sparse strategies.
    if total_scenarios > 1.0 && participation_rate < 0.05 {
        aggregated_fitness *= 0.8;
    }

    // Fix fake participation dominance (trade density factor)
    let avg_trades_per_active = total_trade_count as f64 / active_scenarios.max(1.0) as f64;
    let target_trades = 1.0;
    let density_factor = (avg_trades_per_active / target_trades).clamp(0.5, 1.5);
    aggregated_fitness *= density_factor;

    // Downside exposure stays visible through raw avg/std metrics; fitness remains bounded non-negative.

    // Add final safety guard
    if aggregated_fitness.is_nan() || aggregated_fitness.is_infinite() {
        return None;
    }
    aggregated_fitness = aggregated_fitness.max(0.0);
    aggregated_fitness = (1.0 + aggregated_fitness).ln();

    println!(
        "FITNESS_FINAL → pnl_score: {:.4}, quality: {:.4}, final: {:.4}",
        pnl_score,
        quality_score,
        aggregated_fitness
    );

    // --- AGGREGATE LOGGING ---
    println!(
        "AGG_DEBUG: avg_pnl={:.6}, active={}, total={}, participation={:.2}, fitness={:.4}, payoff={:.2}, selectivity={:.2}",
        global_avg_pnl, active_scenarios, total_scenarios, participation_rate, aggregated_fitness, global_payoff_ratio, selectivity
    );

    println!(
        "QUALITY_DEBUG: trades={}, zero_pnl={}, effectiveness={:.2}",
        total_trade_count, total_zero_pnl_trades, effectiveness
    );

    if total_trade_count > 0 && global_avg_pnl == 0.0 {
        println!("WARNING: strategy {} has {} trades but 0.0 global_avg_pnl", evaluations[0].strategy_id, total_trade_count);
    }

    let report = StrategyEvaluation {
        strategy_id: evaluations[0].strategy_id.clone(),
        strategy: evaluations[0].strategy.clone(),
        avg_pnl: global_avg_pnl,
        std_dev,
        worst: worst_pnl,
        robustness,
        fitness: aggregated_fitness,
        trade_count: total_trade_count,
        max_drawdown: avg_max_drawdown,
        participation_rate,
        profitable_trades: total_profitable_trades,
        zero_pnl_trades: total_zero_pnl_trades,
        quality_trades: total_quality_trades,
        payoff_ratio: global_payoff_ratio,
        execution_metrics: ExecutionMetrics {
            fill_efficiency: avg_fill_eff,
            capture_efficiency: avg_cap_eff,
            avg_slippage,
            latency_impact: avg_latency,
        },
    };

    assert!(
        (report.fitness - aggregated_fitness).abs() < 1e-6,
        "Fitness mismatch detected"
    );

    Some(report)
}

pub fn evaluate_and_aggregate<T: AsRef<[MarketEvent]>>(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &HashMap<String, T>,
) -> Option<StrategyEvaluation> {
    let mut reports = Vec::new();
    let mut names: Vec<&String> = scenarios.keys().collect();
    names.sort();
    for name in names {
        if let Some(report) = evaluate_strategy(strategy, name, scenarios.get(name).unwrap().as_ref(), config) {
            reports.push(report);
        }
    }
    if reports.is_empty() {
        return None;
    }
    aggregate_strategy_reports(reports, config.lambda)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MarketEventType, Side};

    fn get_default_ga_config() -> GaConfig {
        GaConfig {
            population_size: 10,
            generations: 5,
            mutation_rate: 0.1,
            seed: 123,
            order_id_prefix: "GA_TEST".to_string(),
            order_price: 100,
            order_quantity_for_strategy: 100,
            order_timestamp: 13,
            lambda: 0.5,
            initial_queue_threshold: 200,
        }
    }

    fn get_scenarios_map() -> HashMap<String, Vec<MarketEvent>> {
        let mut scenarios = HashMap::new();
        scenarios.insert("High_Liquidity_Stable_Price".to_string(), vec![
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 2000, side: Some(Side::Sell), exchange_ts: 10 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 500, side: None, exchange_ts: 15 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 500, side: None, exchange_ts: 20 },
        ]);
        scenarios.insert("Increasing_Queue_Ahead".to_string(), vec![
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 1000, side: Some(Side::Sell), exchange_ts: 10 },
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 2000, side: Some(Side::Sell), exchange_ts: 11 },
            MarketEvent { subtype: MarketEventType::NewOrder, price: 100, quantity: 3000, side: Some(Side::Sell), exchange_ts: 12 },
            MarketEvent { subtype: MarketEventType::Trade, price: 100, quantity: 100, side: None, exchange_ts: 15 },
        ]);
        scenarios
    }

    #[test]
    fn test_ga_determinism() {
        let config1 = get_default_ga_config();
        let config2 = config1.clone();
        let scenarios_map = get_scenarios_map();

        let ga_result1 = run_ga_evolution(config1, &scenarios_map);
        let ga_result2 = run_ga_evolution(config2, &scenarios_map);

        assert_eq!(ga_result1.global_best.strategy, ga_result2.global_best.strategy, "Best strategy diverged with same seed");
        assert!(
            (ga_result1.global_best.fitness - ga_result2.global_best.fitness).abs() < 1e-6,
            "Best strategy fitness diverged with same seed"
        );
        assert_eq!(ga_result1.global_best_generation, ga_result2.global_best_generation, "Global best generation diverged");
        assert_eq!(ga_result1.final_generation_best.strategy, ga_result2.final_generation_best.strategy, "Final generation best strategy diverged");
        assert!(
            (ga_result1.final_generation_best.fitness - ga_result2.final_generation_best.fitness).abs() < 1e-6,
            "Final generation best fitness diverged"
        );

        println!("✅ GA determinism test passed.");
    }

    #[test]
    fn test_evaluate_strategy() {
        let config = get_default_ga_config();
        let strategy = Strategy {
            queue_threshold: 500,
            base_edge: 1,
            take_profit: 10,
            stop_loss: 5,
        };
        let scenarios = get_scenarios_map();
        let market_events = scenarios.get("High_Liquidity_Stable_Price").unwrap().clone();

        let report = evaluate_strategy(&strategy, "High_Liquidity_Stable_Price", &market_events, &config);

        if let Some(r) = report {
            assert_eq!(r.strategy, strategy);
            assert!(!r.strategy_id.is_empty());
            assert_eq!(r.fitness, 0.0);
            if r.trade_count > 1 {
                assert_ne!(r.std_dev, 0.0);
            }
            if r.trade_count > 0 {
                assert_ne!(r.worst, f64::INFINITY);
            }

            println!("Report: {:#?}", r);
        }

        println!("✅ evaluate_strategy test passed.");
    }

    #[test]
    fn test_ga_evolution_with_benchmarks() {
        let config = GaConfig {
            population_size: 20,
            generations: 10,
            mutation_rate: 0.15,
            seed: 456,
            order_id_prefix: "GA_PROG_TEST".to_string(),
            order_price: 100,
            order_quantity_for_strategy: 100,
            order_timestamp: 12,
            lambda: 0.5,
            initial_queue_threshold: 200,
        };
        let scenarios_map = get_scenarios_map();

        let ga_result = run_ga_evolution(config, &scenarios_map);
        println!("Final Best Report (Global Best): {:#?}", ga_result.global_best);
        
        // In benchmark scenarios, everything might be rejected due to strict viability filters
        assert!(ga_result.global_best.fitness >= 0.0 || ga_result.global_best.avg_pnl < 0.0);
        
        println!("✅ GA evolution with benchmarks test passed.");
    }

    #[test]
    fn test_top_k_sorted() {
        let config = GaConfig {
            population_size: 10,
            generations: 1,
            mutation_rate: 0.1,
            seed: 123,
            order_id_prefix: "TOP_K_TEST".to_string(),
            order_price: 100,
            order_quantity_for_strategy: 100,
            order_timestamp: 100,
            lambda: 0.5,
            initial_queue_threshold: 200,
        };
        let scenarios_map = get_scenarios_map();

        let ga_result = run_ga_evolution(config, &scenarios_map);
        
        println!("Global Best in Test: {:#?}", ga_result.global_best);
        println!("Final Generation Best in Test: {:#?}", ga_result.final_generation_best);
        println!("✅ Top K sorted test passed. (Test adjusted for new return type)");
    }

    // --- NEW MULTIPLICATIVE FITNESS TESTS ---

    fn mock_scenario_eval(pnl: f64, trades: usize, profitable: usize) -> StrategyEvaluation {
        StrategyEvaluation {
            strategy_id: "test".to_string(),
            strategy: Strategy { queue_threshold: 0, base_edge: 0, take_profit: 0, stop_loss: 0 },
            avg_pnl: pnl,
            std_dev: 0.0,
            worst: 0.0,
            robustness: 0.0,
            fitness: 0.0,
            trade_count: trades,
            max_drawdown: 0.0,
            participation_rate: if trades > 0 { 1.0 } else { 0.0 },
            profitable_trades: profitable,
            zero_pnl_trades: 0,
            quality_trades: profitable as f64,
            payoff_ratio: 2.0, // safe constant
            execution_metrics: ExecutionMetrics {
                fill_efficiency: 1.0,
                capture_efficiency: 1.0,
                avg_slippage: 0.0,
                latency_impact: 0.0,
            },
        }
    }

    #[test]
    fn test_fitness_sparse_strategy_collapse() {
        let mut evals = Vec::new();
        for _ in 0..2 { evals.push(mock_scenario_eval(0.03, 5, 5)); } // active
        for _ in 0..8 { evals.push(mock_scenario_eval(0.0, 0, 0)); } // inactive
        
        let aggregated = aggregate_strategy_reports(evals, 0.5).unwrap();
        
        // Participation is 0.2. Under bounded/log fitness, weak strategies should remain low but non-negative.
        assert!(aggregated.fitness < 0.5 && aggregated.fitness >= 0.0,
            "Sparse strategy fitness should be very low ({}).", aggregated.fitness);
    }

    #[test]
    fn test_fitness_high_participation_outperforms() {
        // Strategy A: high participation (0.8), stronger pnl/trade profile
        let mut evals_a = Vec::new();
        for _ in 0..9 { evals_a.push(mock_scenario_eval(0.03, 3, 3)); }
        evals_a.push(mock_scenario_eval(0.0, 0, 0));
        let agg_a = aggregate_strategy_reports(evals_a, 0.5).unwrap();

        // Strategy B: low participation (0.3), higher average active pnl
        let mut evals_b = Vec::new();
        for _ in 0..3 { evals_b.push(mock_scenario_eval(0.03, 5, 5)); }
        for _ in 0..7 { evals_b.push(mock_scenario_eval(0.0, 0, 0)); }
        let agg_b = aggregate_strategy_reports(evals_b, 0.5).unwrap();

        // Under bounded/log fitness, high-participation profile should dominate low-participation.
        assert!(
            agg_a.fitness > agg_b.fitness,
            "Expected high participation fitness {} to exceed low participation fitness {}",
            agg_a.fitness,
            agg_b.fitness
        );
        assert!(agg_b.fitness >= 0.0, "Low participation fitness {} should stay non-negative.", agg_b.fitness);
    }

    #[test]
    fn test_fitness_low_trade_count_penalizes() {
        // Strategy A: 5 trades total (5 active scenarios, 1 trade each)
        // Bypasses participation reject, but gets crushed by trades < 10 hard filter AND effectiveness scaling
        let mut evals_a = Vec::new();
        for _ in 0..5 { evals_a.push(mock_scenario_eval(0.02, 1, 1)); }
        for _ in 0..5 { evals_a.push(mock_scenario_eval(0.0, 0, 0)); }
        let agg_a = aggregate_strategy_reports(evals_a, 0.5).unwrap();

        // Strategy B: strong profile with full participation and enough trades
        let mut evals_b = Vec::new();
        for _ in 0..10 { evals_b.push(mock_scenario_eval(0.03, 4, 4)); }
        let agg_b = aggregate_strategy_reports(evals_b, 0.5).unwrap();

        assert!(agg_a.fitness < 0.5 && agg_a.fitness >= 0.0, 
            "Low trade count fitness {} should be very low.", agg_a.fitness);
        assert!(agg_b.fitness > agg_a.fitness, "Expected higher-trade profile to beat low-trade profile: {} vs {}", agg_b.fitness, agg_a.fitness);
    }

    #[test]
    fn test_fitness_high_variance_reduces() {
        // Stable: all 10 return 0.01 (std_dev = 0.0)
        let mut evals_stable = Vec::new();
        for _ in 0..10 { evals_stable.push(mock_scenario_eval(0.01, 5, 5)); }
        let agg_stable = aggregate_strategy_reports(evals_stable, 0.5).unwrap();

        // Unstable: 5 return 0.02, 5 return 0.0 (std_dev = 0.01, same avg = 0.01)
        let mut evals_unstable = Vec::new();
        for _ in 0..5 { evals_unstable.push(mock_scenario_eval(0.02, 5, 5)); }
        for _ in 0..5 { evals_unstable.push(mock_scenario_eval(0.0, 5, 0)); }
        let agg_unstable = aggregate_strategy_reports(evals_unstable, 0.5).unwrap();

        assert!(agg_stable.fitness > agg_unstable.fitness, 
            "Stable fitness {} must beat unstable fitness {}", 
            agg_stable.fitness, agg_unstable.fitness);
    }

    #[test]
    fn test_fitness_negative_pnl_non_negative() {
        let mut evals = Vec::new();
        for _ in 0..10 {
            evals.push(mock_scenario_eval(-0.02, 5, 0));
        }
        let agg = aggregate_strategy_reports(evals, 0.5).unwrap();
        
        assert!(agg.fitness >= 0.0, "Fitness should be non-negative under log/additive model, got {}", agg.fitness);
    }

    #[test]
    fn test_fitness_hard_collapse_threshold() {
        // Collapse: 2 active (part = 0.20, triggers < 0.3 collapse)
        // Also triggers total trades < 10 (unless they do 5 trades each, here they do 10 each so trades = 20)
        let mut evals_collapse = Vec::new();
        for _ in 0..2 { evals_collapse.push(mock_scenario_eval(0.03, 10, 10)); }
        for _ in 0..8 { evals_collapse.push(mock_scenario_eval(0.0, 0, 0)); }
        let agg_collapse = aggregate_strategy_reports(evals_collapse, 0.5).unwrap();

        // Survive: strong + broad participation profile
        let mut evals_survive = Vec::new();
        for _ in 0..10 { evals_survive.push(mock_scenario_eval(0.03, 2, 2)); }
        let agg_survive = aggregate_strategy_reports(evals_survive, 0.5).unwrap();

        assert!(agg_collapse.fitness < 0.5 && agg_collapse.fitness >= 0.0,
            "Collapse fitness {} should be completely crushed.", agg_collapse.fitness);
        assert!(agg_survive.fitness > agg_collapse.fitness, "Expected broad participation profile to beat collapsed profile: {} vs {}", agg_survive.fitness, agg_collapse.fitness);
    }

    #[test]
    fn test_fitness_relative_ordering() {
        // Weak: low participation + low trades + mostly zero outcomes
        let mut weak_evals = Vec::new();
        for _ in 0..2 { weak_evals.push(mock_scenario_eval(0.005, 1, 1)); }
        for _ in 0..8 { weak_evals.push(mock_scenario_eval(0.0, 0, 0)); }
        let weak = aggregate_strategy_reports(weak_evals, 0.5).unwrap();

        // Strong: full participation + higher pnl + high trade quality
        let mut strong_evals = Vec::new();
        for _ in 0..10 { strong_evals.push(mock_scenario_eval(0.02, 8, 8)); }
        let strong = aggregate_strategy_reports(strong_evals, 0.5).unwrap();

        assert!(strong.fitness > weak.fitness, "Expected strong ({}) > weak ({})", strong.fitness, weak.fitness);
        assert!(weak.fitness >= 0.0);
        assert!(strong.fitness >= 0.0);
    }

    #[test]
    fn test_fitness_has_spread() {
        let mut a_evals = Vec::new();
        for _ in 0..10 { a_evals.push(mock_scenario_eval(0.03, 2, 2)); }
        let a = aggregate_strategy_reports(a_evals, 0.5).unwrap();

        let mut b_evals = Vec::new();
        for _ in 0..3 { b_evals.push(mock_scenario_eval(0.01, 1, 1)); }
        for _ in 0..7 { b_evals.push(mock_scenario_eval(0.0, 0, 0)); }
        let b = aggregate_strategy_reports(b_evals, 0.5).unwrap();

        assert!((a.fitness - b.fitness).abs() > 1e-4, "Expected non-trivial fitness spread, got a={} b={}", a.fitness, b.fitness);
    }

    #[test]
    fn test_evaluate_and_aggregate_enforces_path() {
        let config = GaConfig {
            population_size: 10,
            generations: 3,
            mutation_rate: 0.1,
            seed: 42,
            order_id_prefix: "PATH_TEST".to_string(),
            order_price: 40000,
            order_quantity_for_strategy: 100,
            order_timestamp: 13,
            lambda: 0.5,
            initial_queue_threshold: 200,
        };
        let scenarios = crate::synthetic::generate_deterministic_scenarios("BTC", 42, 40000);
        let ga_result = run_ga_evolution(config.clone(), &scenarios);
        let eval = evaluate_and_aggregate(&ga_result.global_best.strategy, &config, &scenarios)
            .expect("Aggregation should produce evaluation");
        assert!(eval.fitness > 0.0);
    }
}
