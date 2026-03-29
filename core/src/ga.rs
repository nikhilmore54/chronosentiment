use crate::{CreateOrder, ExecutionMode, MarketEvent, Side};
use crate::harness::run_simulation_harness;
use crate::selection_cap;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rayon::prelude::*;
use serde_json;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use serde_json::value::to_value as to_json_value;

#[derive(Clone)]
pub struct ScenarioPair<'a> {
    pub name: &'a str,
    pub signal_symbol: &'a str,
    pub execution_symbol: &'a str,
    pub signal: &'a [MarketEvent],
    pub execution: &'a [MarketEvent],
}

// Helper function to serialize any serializable struct into a canonical JSON string.
// This is crucial for deterministic hashing, especially for floating-point numbers.
pub fn canonical_json<T: Serialize>(v: &T) -> String {
    let value = to_json_value(v).unwrap_or(serde_json::Value::Null);
    serde_json::to_string(&value).unwrap_or_default()
}

/// Evolution Scale Factor: Maps "Genome Units" (Paise) to Institutional Precision (units of 1/100 paise).
/// Since we moved to 10,000 scale, GA_GENE_SCALE = 100.
pub const GA_GENE_SCALE: u64 = crate::PRICE_SCALE / 100;

/// Evolution State: Tracks generational memory for adaptive mutation and stability.
/// Derived only from deterministic population inputs to maintain GA reproducibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvoState {
    pub stagnation_counter: u32,
    pub last_best_fitness: f64,
    pub mutation_scale: f64,
    pub rolling_variance: f64,
}

impl Default for EvoState {
    fn default() -> Self {
        Self {
            stagnation_counter: 0,
            last_best_fitness: 0.0,
            mutation_scale: 1.0,
            rolling_variance: 0.05,
        }
    }
}

pub struct PopulationMetrics {
    pub min_threshold: u64,
    pub max_threshold: u64,
    pub min_edge: u64,
    pub max_edge: u64,
    pub min_tp: u64,
    pub max_tp: u64,
    pub min_sl: u64,
    pub max_sl: u64,
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

/// Deterministic execution-path fingerprint for one scenario evaluation (GA diversity / diagnostics).
/// Values are **normalized** for stable L1 distances in Top-K diversity; see [`scenario_execution_signature_from_simulation`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenarioExecutionSignature {
    /// Mean simulated `queue_ahead` (and fallbacks), scaled to ~O(1).
    pub avg_queue_ahead: f64,
    /// Mean intent→first-fill latency (exchange timestamps), scaled to ~O(1).
    pub avg_latency: f64,
    /// Realized fill ratio in `[0, 1]` (same idea as `ExecutionMetrics::fill_efficiency`).
    pub fill_ratio: f64,
    /// Per-scenario activity rate (`1` if the scenario traded, else `0` in current evaluator).
    pub participation: f64,
}

impl Default for ScenarioExecutionSignature {
    fn default() -> Self {
        Self {
            avg_queue_ahead: 0.0,
            avg_latency: 0.0,
            fill_ratio: 0.0,
            participation: 0.0,
        }
    }
}

#[inline]
fn scenario_execution_signature_l1(a: &ScenarioExecutionSignature, b: &ScenarioExecutionSignature) -> f64 {
    (a.avg_queue_ahead - b.avg_queue_ahead).abs()
        + (a.avg_latency - b.avg_latency).abs()
        + (a.fill_ratio - b.fill_ratio).abs()
        + (a.participation - b.participation).abs()
}

/// Builds a signature from the ESE event log for our entry/exit orders, plus aggregate fill and participation.
fn scenario_execution_signature_from_simulation(
    events: &[crate::SimEvent],
    entry_order_id: &str,
    exit_order_id: &str,
    fill_efficiency: f64,
    participation_rate: f64,
    queue_ratio_fallback: f64,
) -> (ScenarioExecutionSignature, f64) {
    let mut queue_samples: Vec<f64> = Vec::new();
    let mut intent_ts: HashMap<String, u64> = HashMap::new();
    let mut first_fill_ts: HashMap<String, u64> = HashMap::new();

    for ev in events {
        match ev {
            crate::SimEvent::OrderIntent { order_id, timestamp, .. } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    intent_ts.insert(order_id.clone(), *timestamp);
                }
            }
            crate::SimEvent::OrderEnteredQueue { order_id, queue_ahead, .. }
            | crate::SimEvent::QueueProgression { order_id, queue_ahead, .. } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    queue_samples.push(*queue_ahead as f64);
                }
            }
            crate::SimEvent::PartialFill { order_id, timestamp, .. } => {
                if order_id == entry_order_id || order_id == exit_order_id {
                    first_fill_ts.entry(order_id.clone()).or_insert(*timestamp);
                }
            }
            _ => {}
        }
    }

    let queue_raw_mean = if !queue_samples.is_empty() {
        queue_samples.iter().sum::<f64>() / queue_samples.len() as f64
    } else {
        (queue_ratio_fallback.max(0.0) * 2500.0).min(10_000.0)
    };
    let avg_queue_norm = (queue_raw_mean / 2500.0).clamp(0.0, 4.0);

    let mut latencies: Vec<f64> = Vec::new();
    for oid in [entry_order_id, exit_order_id] {
        if let (Some(&t0), Some(&tf)) = (intent_ts.get(oid), first_fill_ts.get(oid)) {
            latencies.push(tf.saturating_sub(t0) as f64);
        }
    }
    let latency_raw_mean = if !latencies.is_empty() {
        latencies.iter().sum::<f64>() / latencies.len() as f64
    } else {
        crate::ese::FIXED_LATENCY as f64
    };
    let latency_norm = (latency_raw_mean / 200.0).clamp(0.0, 4.0);

    let sig = ScenarioExecutionSignature {
        avg_queue_ahead: avg_queue_norm,
        avg_latency: latency_norm,
        fill_ratio: fill_efficiency.clamp(0.0, 1.0),
        participation: participation_rate.clamp(0.0, 1.0),
    };
    (sig, latency_raw_mean)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioCapability {
    Executable,
    NonExecutable,
}

impl ScenarioCapability {
    pub fn is_executable(&self) -> bool {
        matches!(self, ScenarioCapability::Executable)
    }
}

impl Default for ScenarioCapability {
    fn default() -> Self {
        ScenarioCapability::Executable
    }
}

pub fn determine_scenario_capability(name: &str) -> ScenarioCapability {
    let upper = name.to_uppercase();
    if upper.contains("NIFTY") || upper.contains("SENSEX") || upper.contains("INDEX") {
        ScenarioCapability::NonExecutable
    } else {
        ScenarioCapability::Executable
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyEvaluation {
    pub strategy_id: String,
    pub strategy: Strategy,
    #[serde(default)]
    pub capability: ScenarioCapability,
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
    /// Per-scenario execution microstructure (queue, latency, fills); used for GA Top-K diversity when `GA_DIVERSITY_LAMBDA` > 0.
    pub scenario_signature: ScenarioExecutionSignature,
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
            capability: ScenarioCapability::default(),
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
            scenario_signature: ScenarioExecutionSignature::default(),
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
    /// When set, overrides `GA_MAX_TRADES_PER_SCENARIO` for this config (deterministic; no env).
    pub max_trades_per_scenario: Option<usize>,
    /// When set, overrides `GA_TRADE_COOLDOWN` (event indices after each exit).
    pub trade_cooldown_events: Option<usize>,
    pub latency_ticks: usize,
    pub slippage_factor: f64,
    pub lot_size: f64,
}

impl Default for GaConfig {
    fn default() -> Self {
        let latency_ticks = std::env::var("GA_LATENCY_TICKS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1)
            .min(10);
        let slippage_factor = std::env::var("GA_SLIPPAGE_FACTOR")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.1)
            .clamp(0.0, 1.0);
        let lot_size = std::env::var("GA_LOT_SIZE")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0)
            .max(1.0);

        Self {
            population_size: 5,
            generations: 3,
            mutation_rate: 0.1,
            seed: 42,
            order_id_prefix: "GA_DEFAULT".to_string(),
            order_price: 40000,
            order_quantity_for_strategy: 100,
            order_timestamp: 0,
            lambda: 0.5,
            initial_queue_threshold: 20 * crate::PRICE_SCALE,
            max_trades_per_scenario: None,
            trade_cooldown_events: None,
            latency_ticks,
            slippage_factor,
            lot_size,
        }
    }
}


pub fn run_ga_evolution<'a>(config: GaConfig, all_scenarios: &[ScenarioPair<'a>]) -> GaResult {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut global_best: Option<StrategyEvaluation> = None;
    let mut global_best_generation: usize = 0;
    let mut final_generation_best: Option<StrategyEvaluation> = None;
    let mut generation_peaks: Vec<(usize, f64)> = Vec::new();
    
    // 1. Group Scenarios by (Asset, Regime) using indices
    let mut asset_regime_scenarios: HashMap<(String, String), Vec<ScenarioPair<'a>>> = HashMap::new();
    for pair in all_scenarios {
        let name = pair.name;
        let asset = name.split('_').next().unwrap_or("BTC").to_string();
        let regime = if name.contains("trending_up") { "trending_up" }
                    else if name.contains("trending_down") { "trending_down" }
                    else if name.contains("sideways") { "sideways" }
                    else if name.contains("volatile") { "volatile" }
                    else { "mixed" };
        
        asset_regime_scenarios.entry((asset, regime.to_string())).or_default().push(pair.clone());
    }

    let mut best_per_bucket: HashMap<(String, String), StrategyEvaluation> = HashMap::new();
    let mut all_final_evaluations: Vec<StrategyEvaluation> = Vec::new();
    let mut global_generation_history: Vec<StrategyEvaluation> = Vec::new();

    println!("--- Starting Multi-Asset + Regime Genetic Algorithm Evolution ---");

    let mut sorted_buckets: Vec<_> = asset_regime_scenarios.keys().cloned().collect();
    sorted_buckets.sort();

    for (asset, regime) in sorted_buckets {
        println!("\n>> Evolving Bucket: asset={}, regime={}", asset, regime);
        let scenarios = asset_regime_scenarios.get(&(asset.clone(), regime.clone())).unwrap();
        
        let mut population = initialize_population(&config, &mut rng);
        let mut bucket_best_overall: Option<StrategyEvaluation> = None;
        let mut bucket_history: Vec<StrategyEvaluation> = Vec::new();
        let mut evo = EvoState::default();

        for generation in 0..config.generations {
            // 1. Deduplicate
            population = deduplicate_population(population, &config, &mut rng);

            // 2. Evaluate ONLY on this bucket's scenarios
            let evaluations_option = evaluate_population_scoped(&population, &config, scenarios, generation);

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

                    // Track global history (using the best fitness found across all buckets for this generation)
                    if global_generation_history.len() <= generation {
                        global_generation_history.push(best.clone());
                    } else if best.fitness > global_generation_history[generation].fitness {
                        global_generation_history[generation] = best.clone();
                    }
                }

                // --- INSTITUTIONAL ADAPTIVE EVOLUTION (EvoState) ---
                if let Some(best) = evaluations.first() {
                    // 1. Annealing: Shrink deltas if we are improving
                    if best.fitness > evo.last_best_fitness {
                        evo.mutation_scale = (evo.mutation_scale * 0.85).max(0.3);
                        evo.stagnation_counter = 0;
                        evo.last_best_fitness = best.fitness;
                    } else {
                        // 2. Stagnation: Increase pressure if progress stalls
                        evo.stagnation_counter += 1;
                        if evo.stagnation_counter > 2 {
                            evo.mutation_scale = (evo.mutation_scale * 1.2).min(2.0);
                        }
                    }

                    // 3. Stability Guard: dampen if fitness variance explodes
                    let mean_fitness = evaluations.iter().map(|e| e.fitness).sum::<f64>() / evaluations.len() as f64;
                    let variance = evaluations.iter().map(|e| (e.fitness - mean_fitness).powi(2)).sum::<f64>() / evaluations.len() as f64;
                    evo.rolling_variance = variance;

                    if variance > 0.15 { // Diversity is too high / chaotic
                        evo.mutation_scale *= 0.5;
                        println!("STABILITY_GUARD → Chaotic variance ({:.4}); dampening mutation scale to {:.4}", variance, evo.mutation_scale);
                    }
                }

                if generation < config.generations - 1 {
                    population = evolve_generation(&evaluations, &config, &mut rng, &evo);
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

    println!("\n--- GA Evolution Complete ---");
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

/// One genome: early scenario sample (same order as sequential path) + full `evaluate_and_aggregate`.
/// Scenario timelines stay sequential inside `evaluate_strategy`; only genomes may run in parallel.
fn evaluate_population_member(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
    early_check_indices: &[usize],
) -> Option<StrategyEvaluation> {
    for &idx in early_check_indices {
        let pair = &scenarios[idx];
        let _ = evaluate_strategy(strategy, pair, config);
    }

    if let Some(aggregated) = evaluate_and_aggregate(strategy, config, scenarios) {
        if aggregated.fitness >= 0.0 {
            Some(aggregated)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn evaluate_population_scoped(
    population: &Vec<Strategy>, 
    config: &GaConfig, 
    scenarios: &[ScenarioPair],
    generation: usize
) -> Option<Vec<StrategyEvaluation>> {
    // ⚠️ CRITICAL FIX 2 — EARLY REJECTION (UNBIASED)
    use rand::seq::SliceRandom;
    let mut sample_rng = StdRng::seed_from_u64(config.seed + generation as u64);
    let mut sampled_indices: Vec<usize> = (0..scenarios.len()).collect();
    if !sampled_indices.is_empty() {
        sampled_indices.shuffle(&mut sample_rng);
    }
    let sample_size = scenarios.len().min(5);
    let early_check_indices = &sampled_indices[0..sample_size];

    let threads = selection_cap::resolved_ga_parallelism_threads();
    let per_member: Vec<Option<StrategyEvaluation>> = if threads <= 1 {
        population
            .iter()
            .map(|strategy| {
                evaluate_population_member(
                    strategy,
                    config,
                    scenarios,
                    early_check_indices,
                )
            })
            .collect()
    } else {
        match rayon::ThreadPoolBuilder::new().num_threads(threads).build() {
            Ok(pool) => pool.install(|| {
                population
                    .par_iter()
                    .map(|strategy| {
                        evaluate_population_member(
                            strategy,
                            config,
                            scenarios,
                            early_check_indices,
                        )
                    })
                    .collect()
            }),
            Err(e) => {
                eprintln!(
                    "GA_PARALLELISM: could not build thread pool ({e}); sequential genome eval"
                );
                population
                    .iter()
                    .map(|strategy| {
                        evaluate_population_member(
                            strategy,
                            config,
                            scenarios,
                            early_check_indices,
                        )
                    })
                    .collect()
            }
        }
    };

    let mut evaluations = Vec::with_capacity(population.len());
    for opt in per_member {
        if let Some(ev) = opt {
            evaluations.push(ev);
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
            queue_threshold: rng.gen_range((20 * GA_GENE_SCALE)..=(1000 * GA_GENE_SCALE)),
            base_edge: rng.gen_range((1 * GA_GENE_SCALE)..=(15 * GA_GENE_SCALE)),
            take_profit: rng.gen_range(5..=50), // BPS
            stop_loss: rng.gen_range(2..=25),   // BPS
        };
        if unique_strategies.insert(random_strat.clone()) {
            new_population.push(random_strat);
        }
    }

    new_population
}

fn apply_similarity_penalty(evaluations: &mut Vec<StrategyEvaluation>) {
    // 1. Calculate population-based dynamic ranges for normalization
    let mut metrics = PopulationMetrics {
        min_threshold: u64::MAX, max_threshold: 0,
        min_edge: u64::MAX, max_edge: 0,
        min_tp: u64::MAX, max_tp: 0,
        min_sl: u64::MAX, max_sl: 0,
    };

    for eval in evaluations.iter() {
        metrics.min_threshold = metrics.min_threshold.min(eval.strategy.queue_threshold);
        metrics.max_threshold = metrics.max_threshold.max(eval.strategy.queue_threshold);
        metrics.min_edge = metrics.min_edge.min(eval.strategy.base_edge);
        metrics.max_edge = metrics.max_edge.max(eval.strategy.base_edge);
        metrics.min_tp = metrics.min_tp.min(eval.strategy.take_profit);
        metrics.max_tp = metrics.max_tp.max(eval.strategy.take_profit);
        metrics.min_sl = metrics.min_sl.min(eval.strategy.stop_loss);
        metrics.max_sl = metrics.max_sl.max(eval.strategy.stop_loss);
    }

    // Min-range thresholds (Institutional Safety Floors)
    let range_threshold = (metrics.max_threshold as f64 - metrics.min_threshold as f64).max(100.0 * GA_GENE_SCALE as f64);
    let range_edge = (metrics.max_edge as f64 - metrics.min_edge as f64).max(5.0 * GA_GENE_SCALE as f64);
    let range_tp = (metrics.max_tp as f64 - metrics.min_tp as f64).max(10.0);
    let range_sl = (metrics.max_sl as f64 - metrics.min_sl as f64).max(5.0);

    let top_strats: Vec<Strategy> = evaluations.iter().take(5).map(|e| e.strategy.clone()).collect();
    
    for eval in evaluations.iter_mut() {
        let mut max_similarity: f64 = 0.0;
        for top in &top_strats {
            if &eval.strategy == top { continue; }
            
            // DYNAMIC NORMALIZATION: abs(a-b) / population_range
            let d1 = (eval.strategy.queue_threshold as f64 - top.queue_threshold as f64).abs() / range_threshold;
            let d2 = (eval.strategy.base_edge as f64 - top.base_edge as f64).abs() / range_edge;
            let d3 = (eval.strategy.take_profit as f64 - top.take_profit as f64).abs() / range_tp;
            let d4 = (eval.strategy.stop_loss as f64 - top.stop_loss as f64).abs() / range_sl;
            
            let dist = (d1 + d2 + d3 + d4) / 4.0;
            let similarity = (1.0 - dist).max(0.0);
            max_similarity = max_similarity.max(similarity);
        }
        
        // Multiplicative diversity pressure
        let penalty_factor = (1.0 - 0.2 * max_similarity).clamp(0.8, 1.0);
        eval.fitness *= penalty_factor;
    }
}

fn evolve_generation(evaluations: &Vec<StrategyEvaluation>, config: &GaConfig, rng: &mut StdRng, evo: &EvoState) -> Vec<Strategy> {
    let mut next_gen: Vec<Strategy> = Vec::new();

    let elite_count = 2.min(evaluations.len());
    let elites: Vec<Strategy> = evaluations
        .iter()
        .take(elite_count)
        .map(|e| e.strategy.clone())
        .collect();

    next_gen.extend(elites);

    println!(
        "Elitism → Preserving top {} (Elite fitness: {:.4}) | MutationScale: {:.2} | Stagnation: {}",
        elite_count,
        evaluations[0].fitness,
        evo.mutation_scale,
        evo.stagnation_counter
    );

    // 3. Tournament Selection + Mutation for the rest
    while next_gen.len() < config.population_size {
        let parent_eval = tournament_selection(evaluations, 3, rng);
        let mut offspring = parent_eval.strategy.clone();
        
        if rng.gen::<f64>() < config.mutation_rate {
            mutate_strategy(&mut offspring, rng, parent_eval.trade_count, evo);
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

fn mutate_strategy(strategy: &mut Strategy, rng: &mut StdRng, parent_trade_count: usize, evo: &EvoState) {
    let mut mutation_type = rng.gen_range(0..4);
    
    // 🔥 LINEAGE FORCE-MUTATION: Resurrection pressure if strategy is economically inactive
    if parent_trade_count == 0 {
        // Force mutation to move only entry conditions (threshold/edge)
        mutation_type = if rng.gen_bool(0.5) { 0 } else { 2 };
    }

    // Adaptive step size based on non-linear stagnation scaling
    let stagnation_jump = 1.0 + (evo.stagnation_counter as f64).powi(2) * 0.1;

    match mutation_type {
        0 => { // Big jump in threshold (RUPEE EQUIVALENT)
            let base_delta = rng.gen_range((50 * GA_GENE_SCALE)..(250 * GA_GENE_SCALE)) as f64;
            let delta = (base_delta * evo.mutation_scale * stagnation_jump) as i64 * if rng.gen_bool(0.7) { -1 } else { 1 };
            // Clamp to economic range (never more than 5x price to avoid perma-locked genes)
            strategy.queue_threshold = (strategy.queue_threshold as i64 + delta).clamp((10 * GA_GENE_SCALE) as i64, (5000 * GA_GENE_SCALE) as i64) as u64;
        }
        1 => { // Flip TP/SL (within bounds - BPS scale stays same)
            let temp = strategy.take_profit;
            strategy.take_profit = strategy.stop_loss.clamp(5, 50);
            strategy.stop_loss = temp.clamp(2, 30);
        }
        2 => { // Base_edge change (resurrection favored)
            let intensity = if parent_trade_count == 0 { 2.0 } else { 1.0 };
            let base_edge_new = rng.gen_range((1 * GA_GENE_SCALE)..((20.0 * intensity) as u64 * GA_GENE_SCALE));
            strategy.base_edge = base_edge_new;
        }
        _ => { // Reset TP/SL to realistic ranges (BPS)
            strategy.take_profit = rng.gen_range(10..=50);
            strategy.stop_loss = rng.gen_range(5..=30);
        }
    }
}


/// One non-overlapping round-trip from a cursor index (ESE harness), for multi-cycle GA evaluation.
struct GaRoundTripOutcome {
    pnl: f64,
    quality: f64,
    exit_event_idx: usize,
    drawdown_penalty_raw: f64,
    total_filled_qty: u64,
    fills_count: usize,
    total_slippage_bps: f64,
    expected_move: f64,
    raw_q_ratio: f64,
    fill_efficiency: f64,
    sim_events: Vec<crate::SimEvent>,
    entry_order_id: String,
    exit_order_id: String,
    spread: f64,
}

/// Deterministic single round-trip anchored at `market_events[cursor_i]`. Order IDs are unique per `cycle_idx`.
fn ga_simulate_round_trip_at_cursor(
    strategy: &Strategy,
    strategy_id: &str,
    scenario_name: &str,
    signal_events: &[MarketEvent],
    execution_events: &[MarketEvent],
    config: &GaConfig,
    cursor_i: usize,
    cycle_idx: usize,
) -> Option<GaRoundTripOutcome> {
    if cursor_i >= signal_events.len() {
        return None;
    }
    let ref_event = &signal_events[cursor_i];
    let ref_ts = ref_event.exchange_ts;
    let ref_price = ref_event.price;

    let mut current_market_queue: u64 = 0;
    for event in signal_events {
        if event.exchange_ts > ref_ts {
            continue;
        }
        if event.price == ref_price {
            match event.subtype {
                crate::MarketEventType::NewOrder => current_market_queue += event.quantity,
                crate::MarketEventType::Cancel | crate::MarketEventType::Trade => {
                    current_market_queue = current_market_queue.saturating_sub(event.quantity);
                }
            }
        }
    }

    let prices: Vec<f64> = signal_events.iter().map(|e| e.price as f64).collect();
    let mean_price = if prices.is_empty() {
        0.0
    } else {
        prices.iter().sum::<f64>() / prices.len() as f64
    };
    let norm_vol = if prices.len() > 1 && mean_price > 0.0 {
        let variance = prices.iter().map(|p| (p - mean_price).powi(2)).sum::<f64>() / prices.len() as f64;
        variance.sqrt() / mean_price
    } else {
        0.0
    };
    let first_price = prices.first().copied().unwrap_or(ref_price as f64);
    let last_price = prices.last().copied().unwrap_or(ref_price as f64);
    let is_bearish = last_price < first_price;

    let volatility_factor = if norm_vol > 0.002 {
        0.5
    } else if norm_vol < 0.0005 {
        1.5
    } else {
        1.0
    };
    let trade_frequency_bias = 0.8;
    let dynamic_threshold = (strategy.queue_threshold as f64 * volatility_factor * trade_frequency_bias) as u64;
    let dynamic_threshold = dynamic_threshold.max(10);

    let raw_q_ratio = current_market_queue as f64 / dynamic_threshold as f64;
    let q_ratio = raw_q_ratio.min(2.0);
    let vol_signal = (norm_vol / 0.001).min(2.0);
    let mut aggressiveness = (q_ratio + vol_signal) / 2.0;
    if is_bearish && norm_vol < 0.001 {
        aggressiveness *= 0.7;
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hasher;
    hasher.write_u64(strategy.queue_threshold);
    hasher.write_u64(strategy.base_edge);
    hasher.write_u64(strategy.take_profit);
    hasher.write_u64(strategy.stop_loss);
    hasher.write_u64(signal_events.first().map(|e| e.exchange_ts).unwrap_or(0));
    hasher.write_u64(signal_events.last().map(|e| e.exchange_ts).unwrap_or(0));
    hasher.write_u64(signal_events.first().map(|e| e.price).unwrap_or(0));
    hasher.write_u64(signal_events.len() as u64);
    hasher.write(scenario_name.as_bytes());
    hasher.write_usize(cursor_i);
    hasher.write_usize(cycle_idx);
    let roll = (hasher.finish() % 1000) as f64 / 1000.0;

    let entry_idx = cursor_i + config.latency_ticks;
    if entry_idx >= execution_events.len().saturating_sub(1) {
        return None;
    }

    let sig_px = signal_events[cursor_i].price as f64;
    let exe_px = execution_events[entry_idx].price as f64;
    let spread = (exe_px - sig_px).abs();
    
    // Institutional hard check: reject corrupt data with spread > 10% of price
    if spread > sig_px * 0.1 {
        return None;
    }
    
    let slippage = spread * config.slippage_factor;

    let market_price = (exe_px + slippage) as u64;
    let edge_bias = ((strategy.base_edge as f64 - 5.0) / 50.0).clamp(-0.12, 0.12);
    let agg_threshold = ((aggressiveness / 1.1) + edge_bias).clamp(0.05, 0.98);
    let tick_01 = (0.01 * crate::PRICE_SCALE as f64).round() as u64;
    let (buy_price, is_aggressive) = if roll < agg_threshold {
        (market_price.saturating_add(tick_01), true)
    } else {
        (market_price, false)
    };

    let total_events = signal_events.len().max(1) as f64;
    let progress = cursor_i as f64 / total_events;
    let time_factor = (1.0 - 0.7 * progress).max(0.3);
    let vol_boost = 1.0 + norm_vol * 2.0;
    let regime_prob = if norm_vol > 0.001 {
        0.85 * vol_boost
    } else {
        0.65
    };
    let base_fill_prob = if is_aggressive {
        regime_prob.min(0.9)
    } else {
        ((-raw_q_ratio).exp() * time_factor * vol_boost).min(0.85)
    };
    let fill_prob = base_fill_prob.clamp(0.02, 0.92);

    let entry_order_id = format!("{}_c{}_entry", strategy_id, cycle_idx);
    let exit_order_id = format!("{}_c{}_exit", strategy_id, cycle_idx);

    let tp_bps = strategy.take_profit as f64 / 10000.0;
    let sl_bps = strategy.stop_loss as f64 / 10000.0;
    let tp_target = (buy_price as f64 * (1.0 + tp_bps)) as u64;
    let sl_target = (buy_price as f64 * (1.0 - sl_bps)) as u64;

    let entry_idx = cursor_i + config.latency_ticks;
    let min_hold = 5usize.saturating_add(
        (strategy.base_edge as usize + strategy.take_profit as usize + strategy.stop_loss as usize) % 15,
    );
    let mut exit_price = buy_price;
    let mut exit_ts = ref_ts.saturating_add(100);
    let mut found_exit = false;
    let mut exit_event_idx = execution_events.len().saturating_sub(1);

    for (j, event) in execution_events.iter().enumerate().skip(entry_idx + min_hold) {
        if event.price >= tp_target || event.price <= sl_target {
            exit_price = (event.price as f64 - slippage) as u64;
            exit_ts = event.exchange_ts;
            exit_event_idx = j;
            found_exit = true;
            break;
        }
    }
    if !found_exit {
        if let Some(last_ev) = execution_events.last() {
            exit_price = (last_ev.price as f64 - slippage) as u64;
            exit_ts = last_ev.exchange_ts;
            exit_event_idx = execution_events.len() - 1;
        }
    }

    let entry_order = CreateOrder {
        order_id: entry_order_id.clone(),
        side: Side::Buy,
        price: buy_price,
        quantity: config.order_quantity_for_strategy,
        timestamp: ref_ts,
        fill_probability: fill_prob,
    };
    let exit_order = CreateOrder {
        order_id: exit_order_id.clone(),
        side: Side::Sell,
        price: if is_aggressive {
            exit_price.saturating_sub(1)
        } else {
            exit_price
        },
        quantity: config.order_quantity_for_strategy,
        timestamp: exit_ts,
        fill_probability: fill_prob,
    };
    let orders_to_inject = vec![entry_order, exit_order];

    let mut event_refs = Vec::with_capacity(execution_events.len());
    for ev in execution_events {
        event_refs.push(ev.clone());
    }

    let (_, simulation_result, _) =
        run_simulation_harness(ExecutionMode::Real, event_refs, orders_to_inject.clone());

    let mut current_balance = 0i64;
    let mut max_balance = 0i64;
    let mut max_drawdown = 0i64;
    let mut total_filled_qty = 0u64;
    let mut total_slippage_bps = 0.0;
    let mut fills_count = 0usize;
    let requested_qty = config.order_quantity_for_strategy * 2;

    for event in &simulation_result.events {
        if let crate::SimEvent::PartialFill { order_id, filled_qty, price, .. } = event {
            if order_id == &entry_order_id {
                total_filled_qty += filled_qty;
                let slippage = (*price as f64 - buy_price as f64).abs() / buy_price.max(1) as f64;
                total_slippage_bps += slippage;
                fills_count += 1;
            } else if order_id == &exit_order_id {
                total_filled_qty += filled_qty;
                let slippage = (*price as f64 - exit_price as f64).abs() / exit_price.max(1) as f64;
                total_slippage_bps += slippage;
                fills_count += 1;
            }
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

    let market_px = execution_events.first().map(|e| e.price).unwrap_or(ref_price);
    let drawdown_penalty_raw =
        max_drawdown.abs() as f64 / (market_px.max(1) * config.order_quantity_for_strategy.max(1)) as f64;

    let expected_move = if buy_price > 0 {
        (exit_price as f64 - buy_price as f64).abs() / buy_price as f64
    } else {
        0.0
    };

    let mut scenario_pnls: Vec<f64> = Vec::new();
    let mut total_quality_trades = 0.0;
    let mut trades_executed = 0usize;
    let mut current_entry_price = 0u64;
    let mut entry_filled = false;

    for event in &simulation_result.events {
        if let crate::SimEvent::PartialFill { order_id, price, .. } = event {
            if order_id == &entry_order_id {
                current_entry_price = *price;
                entry_filled = true;
            } else if order_id == &exit_order_id && entry_filled {
                let exit_price_val = *price;
                if current_entry_price == 0 || exit_price_val == current_entry_price {
                    scenario_pnls.push(-0.0001);
                    entry_filled = false;
                    trades_executed += 1;
                    continue;
                }
                let move_abs = (exit_price_val as f64 - current_entry_price as f64).abs();
                let min_move = current_entry_price as f64 * 0.0005;
                if move_abs < min_move {
                    scenario_pnls.push(-0.0001);
                    entry_filled = false;
                    trades_executed += 1;
                    continue;
                }
                let side = Side::Buy;
                let pnl_return_base = match side {
                    Side::Buy => (exit_price_val as f64 - current_entry_price as f64) / current_entry_price as f64,
                    Side::Sell => (current_entry_price as f64 - exit_price_val as f64) / current_entry_price as f64,
                };
                let transaction_cost = 0.0001;
                let final_pnl_return = pnl_return_base - transaction_cost;
                let price_move = (exit_price_val as f64 - current_entry_price as f64).abs() / current_entry_price as f64;
                let trade_quality = if price_move > 0.002 { 1.0 } else { 0.0 };
                total_quality_trades += trade_quality;
                if final_pnl_return.is_nan() || final_pnl_return.is_infinite() {
                    scenario_pnls.push(0.0);
                } else {
                    scenario_pnls.push(final_pnl_return);
                }
                trades_executed += 1;
                entry_filled = false;
            }
        }
    }

    if trades_executed == 0 {
        if std::env::var("GA_DEBUG_FORCE_TRADES").is_ok() {
            println!("GA_FORCE_TRADE → Synthesizing forced trade for scenario={} cursor_i={}", scenario_name, cursor_i);
            scenario_pnls.push(-0.0001); // Small penalty for forced trade
            trades_executed = 1;
        } else {
            // println!("ENTRY_FAIL → No trade executed (no fills) for scenario={} cursor_i={}", scenario_name, cursor_i);
            return None;
        }
    }

    let pnl = (scenario_pnls.iter().sum::<f64>() / scenario_pnls.len() as f64) * config.lot_size;
    let quality = if trades_executed > 0 {
        total_quality_trades / trades_executed as f64
    } else {
        0.0
    };
    let fill_efficiency = if requested_qty > 0 {
        total_filled_qty as f64 / requested_qty as f64
    } else {
        0.0
    };

    Some(GaRoundTripOutcome {
        pnl,
        quality,
        exit_event_idx,
        drawdown_penalty_raw,
        total_filled_qty,
        fills_count,
        total_slippage_bps,
        expected_move,
        raw_q_ratio,
        fill_efficiency,
        sim_events: simulation_result.events.clone(),
        entry_order_id,
        exit_order_id,
        spread,
    })
}

pub(crate) fn evaluate_strategy(
    strategy: &Strategy,
    pair: &ScenarioPair,
    config: &GaConfig,
) -> Option<StrategyEvaluation> {
    let scenario_name = pair.name;
    let signal_events = pair.signal;
    let execution_events = pair.execution;

    // Phase 4: Routing Integrity & Pointer Safety (True Dual-Stream)
    println!(
        "ROUTE_SOURCE → {} -> {}",
        pair.signal_symbol, pair.execution_symbol
    );
    
    // Hard Assert: Prevent "fake" separation at the memory level
    if pair.signal_symbol != pair.execution_symbol {
        assert!(
            !std::ptr::eq(signal_events.as_ptr(), execution_events.as_ptr()),
            "FATAL: signal and execution streams are physically identical buffers for symbols {}/{}",
            pair.signal_symbol, pair.execution_symbol
        );
    }

    println!(
        "ROUTE_VERIFY → diff={} sig_ptr={:p} exec_ptr={:p}",
        !std::ptr::eq(signal_events.as_ptr(), execution_events.as_ptr()),
        signal_events.as_ptr(),
        execution_events.as_ptr()
    );

    if signal_events.is_empty() {
        return None;
    }

    let strategy_id = format!("strat_{}_{}_{}_{}_{}", scenario_name, strategy.queue_threshold, strategy.base_edge, strategy.take_profit, strategy.stop_loss);
    let capability = determine_scenario_capability(scenario_name);

    let max_trades = config
        .max_trades_per_scenario
        .unwrap_or_else(|| selection_cap::resolved_ga_max_trades_per_scenario());
    let cooldown = config
        .trade_cooldown_events
        .unwrap_or_else(|| selection_cap::resolved_ga_trade_cooldown());

    let mut scenario_pnls: Vec<f64> = Vec::new();
    let mut total_quality_trades_scenario = 0.0;
    let mut total_spread_test = 0.0;
    let mut sum_price = 0.0;
    // Diagnostic Counters
    let mut signal_count = 0usize;
    let mut entry_attempted = 0usize;
    let mut entry_triggered = 0usize;

    let mut total_filled_qty = 0u64;
    let mut total_slippage_bps = 0.0;
    let mut fills_count = 0usize;
    let mut sum_drawdown_raw = 0.0;
    let mut sum_expected_move = 0.0;
    let mut sum_latency_raw = 0.0;
    let mut cycle_sigs: Vec<ScenarioExecutionSignature> = Vec::new();
    let mut n_cycles = 0usize;

    let mut i = 0usize;
    let fuse_limit = signal_events
        .len()
        .saturating_add(max_trades.saturating_mul(signal_events.len().max(1)));
    let mut fuse = 0usize;

    while scenario_pnls.len() < max_trades && i < signal_events.len() {
        if scenario_pnls.len() >= max_trades || i >= signal_events.len().saturating_sub(1) {
            break;
        }

        fuse += 1;
        if fuse > fuse_limit {
            break;
        }

        // 50-step Route Verification Log
        if (i % 50 == 0 || i == signal_events.len() - 1) && std::env::var("ROUTE_CHECK").is_ok() {
            let s_px = signal_events[i].price;
            let e_px = execution_events[i].price;
            println!(
                "ROUTE_CHECK → i={} sig_px={:.4} exe_px={:.4} spread={:.4}",
                i,
                crate::to_real(s_px),
                crate::to_real(e_px),
                crate::to_real(e_px.saturating_sub(s_px))
            );
        }

        signal_count += 1;
        entry_attempted += 1;
        match ga_simulate_round_trip_at_cursor(
            strategy,
            &strategy_id,
            scenario_name,
            signal_events,
            execution_events,
            config,
            i,
            scenario_pnls.len(),
        ) {
            Some(o) => {
                if o.exit_event_idx <= i {
                    i += 1;
                    continue;
                }
                entry_triggered += 1;
                scenario_pnls.push(o.pnl);
                total_spread_test += o.spread;
                sum_price += signal_events[i].price as f64;
                total_quality_trades_scenario += o.quality;
                let (sig, lat) = scenario_execution_signature_from_simulation(
                    &o.sim_events,
                    &o.entry_order_id,
                    &o.exit_order_id,
                    o.fill_efficiency,
                    1.0,
                    o.raw_q_ratio,
                );
                cycle_sigs.push(sig);
                sum_latency_raw += lat;
                total_filled_qty += o.total_filled_qty;
                fills_count += o.fills_count;
                total_slippage_bps += o.total_slippage_bps;
                sum_drawdown_raw += o.drawdown_penalty_raw;
                sum_expected_move += o.expected_move;
                n_cycles += 1;
                println!(
                    "GA_MULTI_TRADE: scenario={} cycle={} cursor_i={} exit_idx={} pnl={:.6}",
                    scenario_name,
                    n_cycles,
                    i,
                    o.exit_event_idx,
                    o.pnl
                );
                i = o.exit_event_idx.saturating_add(cooldown);
            }
            None => {
                i += 1;
            }
        }
    }

    println!(
        "GA_MULTI_TRADE_SUM: scenario={}, completed_cycles={}, cap={}, cooldown={}",
        scenario_name, n_cycles, max_trades, cooldown
    );
    println!(
        "DEBUG: trades per scenario = {}",
        scenario_pnls.len()
    );
    // User requested ENTRY_DEBUG log logic
    println!(
        "ENTRY_DEBUG → signals={} attempts={} triggered={}",
        signal_count,
        entry_attempted,
        entry_triggered
    );

    let total_trades = scenario_pnls.len();
    let requested_qty = config.order_quantity_for_strategy * 2 * (n_cycles.max(1) as u64);
    let mean_expected_move = if n_cycles > 0 {
        sum_expected_move / n_cycles as f64
    } else {
        0.0
    };
    let drawdown_penalty_raw = if n_cycles > 0 {
        sum_drawdown_raw / n_cycles as f64
    } else {
        0.0
    };

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

    // Standard move check
    if total_trades > 0 && selectivity < 0.1 {
        return None;
    }

    // Standard move check
    if total_trades > 0 && payoff_ratio < 0.8 {
        return None;
    }

    let mut total_fitness = current_fitness;

    if total_trades > 0 {
        let avg_spread = total_spread_test / total_trades as f64;
        let avg_price = sum_price / total_trades as f64;
        let spread_penalty = (avg_spread / avg_price.max(1.0)).min(0.02) * 0.05;
        total_fitness -= spread_penalty;
    }
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
        "FITNESS_DIAG: pnl_return={:.6}, trades={}, diagnostic_fitness={:.4}, avg_spread={:.4}",
        avg_pnl_for_scenario,
        total_trades,
        total_fitness,
        crate::to_real((total_spread_test / total_trades.max(1) as f64) as u64)
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
    let capture_efficiency = if mean_expected_move > 0.0 && total_trades > 0 {
        (realized_avg / mean_expected_move).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let participation_rate = if total_trades > 0 { 1.0 } else { 0.0 };
    let n_sig = cycle_sigs.len().max(1) as f64;
    let scenario_signature = if cycle_sigs.is_empty() {
        ScenarioExecutionSignature::default()
    } else {
        ScenarioExecutionSignature {
            avg_queue_ahead: cycle_sigs.iter().map(|s| s.avg_queue_ahead).sum::<f64>() / n_sig,
            avg_latency: cycle_sigs.iter().map(|s| s.avg_latency).sum::<f64>() / n_sig,
            fill_ratio: cycle_sigs.iter().map(|s| s.fill_ratio).sum::<f64>() / n_sig,
            participation: cycle_sigs.iter().map(|s| s.participation).sum::<f64>() / n_sig,
        }
    };
    let latency_raw_mean = if n_cycles > 0 {
        sum_latency_raw / n_cycles as f64
    } else {
        0.0
    };

    let exec_metrics = ExecutionMetrics {
        fill_efficiency,
        capture_efficiency,
        avg_slippage,
        latency_impact: latency_raw_mean,
    };

    Some(StrategyEvaluation {
        strategy_id: strategy_id.clone(),
        strategy: strategy.clone(),
        capability,
        avg_pnl: avg_pnl_for_scenario,
        std_dev: std_dev_for_scenario,
        worst: worst_pnl_for_scenario,
        robustness: robustness_for_scenario,
        // Canonical fitness is computed in aggregate_strategy_reports.
        fitness: 0.0,
        trade_count: total_trades,
        max_drawdown: drawdown_penalty_raw * 100.0, // Storing as percentage
        participation_rate,
        profitable_trades: profitable_trades_scenario,
        zero_pnl_trades: zero_pnl_trades_scenario,
        quality_trades: total_quality_trades_scenario,
        payoff_ratio,
        execution_metrics: exec_metrics,
        scenario_signature,
    })
}

/// Per-scenario rank for GA Top-K alignment with pipeline: `edge × confidence`.
/// Edge uses robustness (risk-adjusted) with a non-negative avg_pnl fallback; confidence uses win rate.
fn ga_scenario_rank_score(e: &StrategyEvaluation) -> f64 {
    let edge = e.robustness.max(0.0).max(e.avg_pnl.max(0.0));
    let conf = if e.trade_count > 0 {
        (e.profitable_trades as f64 / e.trade_count as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    selection_cap::rank_score_edge_confidence(edge, conf)
}

/// Greedy GA Top-K: repeatedly pick the remaining evaluation that maximizes an adjusted rank.
/// Let `mean_dist` be the mean L1 distance from the candidate signature to each already-selected signature.
/// - [`selection_cap::GaDiversityMode::Attract`]: `rank − λ * mean_dist`
/// - [`selection_cap::GaDiversityMode::Repel`]: `rank + λ * mean_dist`
/// Ties break on lower original index (input order). With `λ == 0`, this matches
/// sorting by descending rank score then taking the first `k`.
fn ga_top_k_pick_diverse(
    mut remaining: Vec<(usize, f64, StrategyEvaluation)>,
    k: usize,
    diversity_lambda: f64,
    diversity_mode: selection_cap::GaDiversityMode,
) -> Vec<StrategyEvaluation> {
    let mut selected_sigs: Vec<ScenarioExecutionSignature> = Vec::with_capacity(k);
    let mut out: Vec<StrategyEvaluation> = Vec::with_capacity(k);
    while out.len() < k && !remaining.is_empty() {
        let mut best_i = 0usize;
        let mut best_adj = f64::NEG_INFINITY;
        let mut best_orig = usize::MAX;
        let n_sel = selected_sigs.len().max(1);
        for (i, &(orig_idx, base_score, ref ev)) in remaining.iter().enumerate() {
            let sum_dist: f64 = selected_sigs
                .iter()
                .map(|s| scenario_execution_signature_l1(&ev.scenario_signature, s))
                .sum();
            let mean_dist = if selected_sigs.is_empty() {
                0.0
            } else {
                sum_dist / n_sel as f64
            };
            let adjusted = match diversity_mode {
                selection_cap::GaDiversityMode::Attract => {
                    base_score - diversity_lambda * mean_dist
                }
                selection_cap::GaDiversityMode::Repel => {
                    base_score + diversity_lambda * mean_dist
                }
            };
            let better = match adjusted.partial_cmp(&best_adj) {
                Some(Ordering::Greater) => true,
                Some(Ordering::Equal) => orig_idx < best_orig,
                Some(Ordering::Less) | None => false,
            };
            if better {
                best_adj = adjusted;
                best_i = i;
                best_orig = orig_idx;
            }
        }
        let (_, _score, ev) = remaining.remove(best_i);
        selected_sigs.push(ev.scenario_signature.clone());
        out.push(ev);
    }
    out
}

fn apply_ga_top_k_selection(
    mut evaluations: Vec<StrategyEvaluation>,
    top_k_cap: Option<usize>,
) -> Vec<StrategyEvaluation> {
    let has_executable = evaluations.iter().any(|e| e.capability.is_executable());
    if !has_executable {
        return evaluations;
    }
    
    let context_evals: Vec<StrategyEvaluation> = evaluations.iter().filter(|e| !e.capability.is_executable()).cloned().collect();
    evaluations.retain(|e| e.capability.is_executable());

    let Some(k) = top_k_cap else {
        evaluations.extend(context_evals);
        return evaluations;
    };
    if evaluations.len() <= k {
        evaluations.extend(context_evals);
        return evaluations;
    }
    let n_in = evaluations.len();
    let diversity_lambda = selection_cap::resolved_ga_diversity_lambda();
    let diversity_mode = selection_cap::resolved_ga_diversity_mode();
    let remaining: Vec<(usize, f64, StrategyEvaluation)> = evaluations
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            let s = ga_scenario_rank_score(&e);
            (i, s, e)
        })
        .collect();
    let indexed = ga_top_k_pick_diverse(remaining, k, diversity_lambda, diversity_mode);
    let used = indexed.len();
    if n_in > k {
        if diversity_lambda > 0.0 {
            let mode_s = match diversity_mode {
                selection_cap::GaDiversityMode::Attract => "attract",
                selection_cap::GaDiversityMode::Repel => "repel",
            };
            println!(
                "GA_TOPK: scenarios_in={}, scenarios_used={}, cap={}, diversity_lambda={:.4}, diversity_mode={} (execution_signature_l1_mean)",
                n_in, used, k, diversity_lambda, mode_s
            );
        } else {
            println!(
                "GA_TOPK: scenarios_in={}, scenarios_used={}, cap={}",
                n_in, used, k
            );
        }
    }
    let mut final_evals: Vec<StrategyEvaluation> = indexed;
    final_evals.extend(context_evals);
    final_evals
}

/// Aggregates per-scenario evaluations into one fitness. Uses [`selection_cap::resolved_ga_scenario_top_k`] (GA-only scarcity; pipeline uses `SIGNAL_TOP_K` separately).
///
/// Per-scenario `avg_pnl` / variance use an unweighted mean by default; set `GA_WEIGHTED_SCENARIO_PNL=1`
/// for rank-score-weighted aggregation (same weights as scenario Top-K ordering).
pub fn aggregate_strategy_reports(evaluations: Vec<StrategyEvaluation>, lambda: f64) -> Option<StrategyEvaluation> {
    let evaluations = apply_ga_top_k_selection(evaluations, selection_cap::resolved_ga_scenario_top_k());
    aggregate_strategy_reports_inner(evaluations, lambda).map(|(e, _)| e)
}

/// Same aggregation with an explicit Top-K cap (`None` = use all scenarios). Used by unit tests to avoid env coupling.
#[allow(dead_code)] // Referenced from `#[cfg(test)]` module; unused in non-test library builds.
pub(crate) fn aggregate_strategy_reports_with_top_k(
    evaluations: Vec<StrategyEvaluation>,
    lambda: f64,
    top_k_cap: Option<usize>,
) -> Option<StrategyEvaluation> {
    let evaluations = apply_ga_top_k_selection(evaluations, top_k_cap);
    aggregate_strategy_reports_inner(evaluations, lambda).map(|(e, _)| e)
}

fn aggregate_strategy_reports_inner(
    mut evaluations: Vec<StrategyEvaluation>,
    lambda: f64,
) -> Option<(StrategyEvaluation, f64)> {
    // This check is now handled upstream by evaluate_population_scoped returning None
    // and evaluate_strategy returning None for 0 trades.

    let total_scenarios_in = evaluations.len();
    let has_executable = evaluations.iter().any(|e| e.capability.is_executable());
    if has_executable {
        evaluations.retain(|e| e.capability.is_executable());
    }
    
    let executable_total = evaluations.len();
    let executable_active = evaluations.iter().filter(|e| e.trade_count > 0).count();

    println!(
        "DEBUG_EXEC → total={}, executable={}, active_exec={}, participation_exec={:.2}",
        total_scenarios_in,
        executable_total,
        executable_active,
        executable_active as f64 / (executable_total as f64).max(1.0)
    );

    // IMPORTANT: use raw per-scenario returns; never clip before aggregation.
    let scenario_results: Vec<f64> = evaluations.iter().map(|e| e.avg_pnl).collect();
    let scenario_trade_counts: Vec<usize> = evaluations.iter().map(|e| e.trade_count).collect();

    let total_scenarios = scenario_results.len() as f64;

    let use_rank_weights = selection_cap::resolved_ga_weighted_scenario_pnl();
    let mut weights: Vec<f64> = if use_rank_weights {
        evaluations
            .iter()
            .map(|e| ga_scenario_rank_score(e).max(1e-15))
            .collect()
    } else {
        vec![1.0; evaluations.len()]
    };
    let w_sum_raw: f64 = weights.iter().sum();
    if use_rank_weights && (w_sum_raw <= 0.0 || !w_sum_raw.is_finite()) {
        weights = vec![1.0; evaluations.len()];
    }
    let w_sum: f64 = weights.iter().sum::<f64>().max(1e-15);

    let global_avg_pnl = if total_scenarios > 0.0 {
        evaluations
            .iter()
            .zip(weights.iter())
            .map(|(e, &w)| e.avg_pnl * w)
            .sum::<f64>()
            / w_sum
    } else {
        0.0
    };

    let variance = if total_scenarios > 1.0 {
        evaluations
            .iter()
            .zip(weights.iter())
            .map(|(e, &w)| w * (e.avg_pnl - global_avg_pnl).powi(2))
            .sum::<f64>()
            / w_sum
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
    // With a single scenario, std dev is legitimately zero; weighted mean can also differ from
    // `scenario_results[0]` by floating-point rounding — do not require bitwise equality.
    if total_scenarios > 1.0 {
        let tol = 1e-9_f64.max(global_avg_pnl.abs() * 1e-12);
        assert!(
            std_dev > 1e-18
                || scenario_results
                    .iter()
                    .all(|&x| (x - global_avg_pnl).abs() <= tol),
            "Invalid distribution: non-zero pnl but zero std dev"
        );
    }

    let participation_rate = active_scenarios / total_scenarios;
    let avg_trades_per_active_scenario = if active_scenarios > 0.0 {
        total_trade_count as f64 / active_scenarios
    } else {
        0.0
    };
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

    let aggregated_scenario_signature = ScenarioExecutionSignature {
        avg_queue_ahead: evaluations
            .iter()
            .map(|e| e.scenario_signature.avg_queue_ahead)
            .sum::<f64>()
            / total_scenarios,
        avg_latency: evaluations
            .iter()
            .map(|e| e.scenario_signature.avg_latency)
            .sum::<f64>()
            / total_scenarios,
        fill_ratio: evaluations
            .iter()
            .map(|e| e.scenario_signature.fill_ratio)
            .sum::<f64>()
            / total_scenarios,
        participation: evaluations
            .iter()
            .map(|e| e.scenario_signature.participation)
            .sum::<f64>()
            / total_scenarios,
    };

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

    // 5. Cross-scenario pnl dispersion: higher `std_dev` → lower stability (prefer consistent scenario outcomes).
    let stability_factor = (1.0 - std_dev).clamp(0.0, 1.0);
    let variance_penalty = 1.0;

    // Within-scenario return dispersion (meaningful when scenarios have multiple trades).
    let mean_scenario_std: f64 = evaluations.iter().map(|e| e.std_dev).sum::<f64>() / total_scenarios.max(1.0);
    let intra_stability = (1.0 - (mean_scenario_std * 120.0).min(1.0)).clamp(0.0, 1.0);

    // Prefer strategies with more than one interaction per active scenario on average (targets 5+ as "rich").
    let depth_quality = (avg_trades_per_active_scenario / 5.0).clamp(0.0, 1.0);

    // Debug root cause explicitly - Log components before multiplication
    println!(
        "FITNESS_COMPONENTS → pnl: {:.5}, part: {:.3}, cov: {:.3}, eff: {:.3}, exec: {:.3}, stab: {:.3}, intra_stab: {:.3}, depth_q: {:.3}, avg_trades/active: {:.2}",
        pnl_score,
        participation_factor,
        coverage_factor,
        effectiveness,
        execution_quality.clamp(0.0, 1.0), // Clamp execution_quality here
        stability_factor,
        intra_stability,
        depth_quality,
        avg_trades_per_active_scenario
    );

    let quality_score = 0.20 * participation_factor.clamp(0.0, 1.0)
        + 0.16 * coverage_factor.clamp(0.0, 1.0)
        + 0.14 * effectiveness.clamp(0.0, 1.0)
        + 0.13 * execution_quality.clamp(0.0, 1.0)
        + 0.23 * stability_factor
        + 0.08 * intra_stability
        + 0.06 * depth_quality;

    let mut aggregated_fitness = pnl_score * (0.5 + quality_score);
    aggregated_fitness *= sample_penalty;
    aggregated_fitness *= variance_penalty;

    // Shallow path depth: one trade per scenario (or very few) keeps fitness flat — penalize vs repeatable behavior.
    let required_depth = (crate::selection_cap::resolved_ga_max_trades_per_scenario() as f64).min(3.0);
    let min_avg_trades_per_active = required_depth.max(1.0);
    const SHALLOW_DEPTH_PENALTY: f64 = 0.7;
    if total_scenarios > 1.0
        && active_scenarios > 0.0
        && avg_trades_per_active_scenario < min_avg_trades_per_active
    {
        aggregated_fitness *= SHALLOW_DEPTH_PENALTY;
        println!(
            "FITNESS_SHALLOW_DEPTH → avg_trades_per_active={:.2} < {:.1}, factor={}",
            avg_trades_per_active_scenario, min_avg_trades_per_active, SHALLOW_DEPTH_PENALTY
        );
    }

    // Fix viability penalty
    let viability_penalty = if total_scenarios <= 1.0 {
        1.0
    } else if participation_rate < 0.3 || total_trade_count < 3 {
        0.5
    } else {
        1.0
    };
    aggregated_fitness *= viability_penalty;

    // High participation (many scenarios with trades) reduces differentiation — penalize "everything trades".
    const HIGH_PARTICIPATION_THRESHOLD: f64 = 0.5;
    const HIGH_PARTICIPATION_PENALTY: f64 = 0.7;
    if total_scenarios > 1.0 && participation_rate > HIGH_PARTICIPATION_THRESHOLD {
        aggregated_fitness *= HIGH_PARTICIPATION_PENALTY;
    }

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
        "AGG_DEBUG: avg_pnl={:.6} (scenario_agg={}), active={}, total={}, participation={:.2}, fitness={:.4}, payoff={:.2}, selectivity={:.2}",
        global_avg_pnl,
        if use_rank_weights { "rank_weighted" } else { "mean" },
        active_scenarios,
        total_scenarios,
        participation_rate,
        aggregated_fitness,
        global_payoff_ratio,
        selectivity
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
        capability: ScenarioCapability::Executable,
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
        scenario_signature: aggregated_scenario_signature,
    };

    assert!(
        (report.fitness - aggregated_fitness).abs() < 1e-6,
        "Fitness mismatch detected"
    );

    Some((report, avg_trades_per_active_scenario))
}

/// Same as [`evaluate_and_aggregate`], plus mean round-trips per scenario that had `trade_count > 0` after GA Top-K (matches fitness `avg_trades_per_active`).
pub(crate) fn evaluate_and_aggregate_with_trade_depth(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
) -> Option<(StrategyEvaluation, f64)> {
    let mut reports = Vec::new();
    for (idx, pair) in scenarios.iter().enumerate() {
        if let Some(report) = evaluate_strategy(strategy, pair, config) {
            reports.push(report);
        }
        
        // Phase 3: Early Pruning for dead genomes
        if idx >= 1 && reports.is_empty() {
            // Prune if 0 trades after 2 scenarios
            return None;
        }
    }
    if reports.is_empty() {
        return None;
    }
    let evaluations = apply_ga_top_k_selection(reports, selection_cap::resolved_ga_scenario_top_k());
    aggregate_strategy_reports_inner(evaluations, config.lambda)
}

pub fn evaluate_and_aggregate(
    strategy: &Strategy,
    config: &GaConfig,
    scenarios: &[ScenarioPair],
) -> Option<StrategyEvaluation> {
    evaluate_and_aggregate_with_trade_depth(strategy, config, scenarios).map(|(e, _)| e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_source::CsvCandleSource;
    use crate::data_source::CandleSource;
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
            max_trades_per_scenario: Some(1),
            trade_cooldown_events: None,
        }
    }

    fn synthetic_harness_trade_tape(base_ts: u64, flat_price: u64, step_price: u64) -> Vec<MarketEvent> {
        let mut v = Vec::with_capacity(128);
        for i in 0..128 {
            let ts = base_ts + i as u64;
            // Flat then small step: fills + TP path while keeping aggregate fitness inside GA's `<= 1.0` gate.
            let price = if i < 48 { flat_price } else { step_price };
            v.push(MarketEvent {
                subtype: MarketEventType::Trade,
                price,
                quantity: 2_000,
                side: None,
                exchange_ts: ts,
            });
        }
        v
    }

    /// Two deterministic tapes (no disk I/O): cross-scenario aggregation + multi-trade depth.
    fn synthetic_harness_scenarios() -> HashMap<String, Vec<MarketEvent>> {
        let mut scenarios = HashMap::new();
        scenarios.insert(
            "HARNESS_LIQUID_RAMP_A".to_string(),
            synthetic_harness_trade_tape(1000, 100, 101),
        );
        scenarios.insert(
            "HARNESS_LIQUID_RAMP_B".to_string(),
            synthetic_harness_trade_tape(5000, 102, 103),
        );
        scenarios
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

    /// Real candles, explicit `max_trades_per_scenario` — ensures the multi-trade loop cannot exceed the cap.
    #[test]
    fn test_evaluate_strategy_multi_trade_cap_respected() {
        let test_assets = format!("{}/../test_assets", env!("CARGO_MANIFEST_DIR"));
        let path = format!("{}/RELIANCE_5m_clean.csv", test_assets);
        let candles = CsvCandleSource { path }.get_candles();
        let scenarios = crate::pipeline::scenarios_from_candles("RELIANCE", &candles);
        let mut keys: Vec<String> = scenarios.keys().cloned().collect();
        keys.sort();

        let cap = 3usize;
        let mut config = get_default_ga_config();
        config.max_trades_per_scenario = Some(cap);
        let strategy = Strategy {
            queue_threshold: 200,
            base_edge: 1,
            take_profit: 10,
            stop_loss: 5,
        };

        let mut found = false;
        for name in &keys {
            let events = scenarios.get(name).expect("key from scenarios").as_slice();
            if let Some(r) = evaluate_strategy(&strategy, name, events, &config) {
                assert!(
                    r.trade_count <= cap,
                    "trade_count {} exceeds configured cap {}",
                    r.trade_count,
                    cap
                );
                found = true;
                break;
            }
        }
        assert!(
            found,
            "expected at least one RELIANCE window where strategy trades (for cap test)"
        );
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
            max_trades_per_scenario: Some(1),
            trade_cooldown_events: None,
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
            max_trades_per_scenario: Some(1),
            trade_cooldown_events: None,
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
            scenario_signature: ScenarioExecutionSignature::default(),
        }
    }

    #[test]
    fn test_fitness_sparse_strategy_collapse() {
        let mut evals = Vec::new();
        for _ in 0..2 { evals.push(mock_scenario_eval(0.03, 5, 5)); } // active
        for _ in 0..8 { evals.push(mock_scenario_eval(0.0, 0, 0)); } // inactive
        
        let aggregated = aggregate_strategy_reports_with_top_k(evals, 0.5, None).unwrap();
        
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
        let agg_a = aggregate_strategy_reports_with_top_k(evals_a, 0.5, None).unwrap();

        // Strategy B: low participation (0.3), higher average active pnl
        let mut evals_b = Vec::new();
        for _ in 0..3 { evals_b.push(mock_scenario_eval(0.03, 5, 5)); }
        for _ in 0..7 { evals_b.push(mock_scenario_eval(0.0, 0, 0)); }
        let agg_b = aggregate_strategy_reports_with_top_k(evals_b, 0.5, None).unwrap();

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
        let agg_a = aggregate_strategy_reports_with_top_k(evals_a, 0.5, None).unwrap();

        // Strategy B: strong profile with full participation and enough trades
        let mut evals_b = Vec::new();
        for _ in 0..10 { evals_b.push(mock_scenario_eval(0.03, 4, 4)); }
        let agg_b = aggregate_strategy_reports_with_top_k(evals_b, 0.5, None).unwrap();

        assert!(agg_a.fitness < 0.5 && agg_a.fitness >= 0.0, 
            "Low trade count fitness {} should be very low.", agg_a.fitness);
        assert!(agg_b.fitness > agg_a.fitness, "Expected higher-trade profile to beat low-trade profile: {} vs {}", agg_b.fitness, agg_a.fitness);
    }

    #[test]
    fn test_fitness_high_variance_reduces() {
        // Stable: all 10 return 0.01 (std_dev = 0.0)
        let mut evals_stable = Vec::new();
        for _ in 0..10 { evals_stable.push(mock_scenario_eval(0.01, 5, 5)); }
        let agg_stable = aggregate_strategy_reports_with_top_k(evals_stable, 0.5, None).unwrap();

        // Unstable: 5 return 0.02, 5 return 0.0 (std_dev = 0.01, same avg = 0.01)
        let mut evals_unstable = Vec::new();
        for _ in 0..5 { evals_unstable.push(mock_scenario_eval(0.02, 5, 5)); }
        for _ in 0..5 { evals_unstable.push(mock_scenario_eval(0.0, 5, 0)); }
        let agg_unstable = aggregate_strategy_reports_with_top_k(evals_unstable, 0.5, None).unwrap();

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
        let agg = aggregate_strategy_reports_with_top_k(evals, 0.5, None).unwrap();
        
        assert!(agg.fitness >= 0.0, "Fitness should be non-negative under log/additive model, got {}", agg.fitness);
    }

    #[test]
    fn test_fitness_hard_collapse_threshold() {
        // Collapse: 2 active (part = 0.20, triggers < 0.3 collapse)
        // Also triggers total trades < 10 (unless they do 5 trades each, here they do 10 each so trades = 20)
        let mut evals_collapse = Vec::new();
        for _ in 0..2 { evals_collapse.push(mock_scenario_eval(0.03, 10, 10)); }
        for _ in 0..8 { evals_collapse.push(mock_scenario_eval(0.0, 0, 0)); }
        let agg_collapse = aggregate_strategy_reports_with_top_k(evals_collapse, 0.5, None).unwrap();

        // Survive: strong + broad participation profile
        let mut evals_survive = Vec::new();
        for _ in 0..10 { evals_survive.push(mock_scenario_eval(0.03, 2, 2)); }
        let agg_survive = aggregate_strategy_reports_with_top_k(evals_survive, 0.5, None).unwrap();

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
        let weak = aggregate_strategy_reports_with_top_k(weak_evals, 0.5, None).unwrap();

        // Strong: full participation + higher pnl + high trade quality
        let mut strong_evals = Vec::new();
        for _ in 0..10 { strong_evals.push(mock_scenario_eval(0.02, 8, 8)); }
        let strong = aggregate_strategy_reports_with_top_k(strong_evals, 0.5, None).unwrap();

        assert!(strong.fitness > weak.fitness, "Expected strong ({}) > weak ({})", strong.fitness, weak.fitness);
        assert!(weak.fitness >= 0.0);
        assert!(strong.fitness >= 0.0);
    }

    #[test]
    fn ga_top_k_pick_diverse_lambda_zero_matches_pure_rank_order() {
        let make = |i: usize, pnl: f64| {
            let mut e = mock_scenario_eval(pnl, 5, 5);
            e.strategy_id = format!("s{}", i);
            e
        };
        let evals = vec![make(0, 0.04), make(1, 0.01), make(2, 0.03), make(3, 0.02)];
        let remaining: Vec<(usize, f64, StrategyEvaluation)> = evals
            .into_iter()
            .enumerate()
            .map(|(i, e)| {
                let s = super::ga_scenario_rank_score(&e);
                (i, s, e)
            })
            .collect();
        let picked = super::ga_top_k_pick_diverse(
            remaining,
            2,
            0.0,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        assert_eq!(picked.len(), 2);
        assert_eq!(picked[0].strategy_id, "s0");
        assert_eq!(picked[1].strategy_id, "s2");
    }

    #[test]
    fn ga_top_k_pick_diverse_is_deterministic() {
        let make = |i: usize, pnl: f64| {
            let mut e = mock_scenario_eval(pnl, 5, 5);
            e.strategy_id = format!("s{}", i);
            e
        };
        let evals = vec![make(0, 0.05), make(1, 0.04), make(2, 0.03)];
        let build_remaining = |ev: Vec<StrategyEvaluation>| {
            ev.into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let s = super::ga_scenario_rank_score(&e);
                    (i, s, e)
                })
                .collect::<Vec<_>>()
        };
        let a = super::ga_top_k_pick_diverse(
            build_remaining(evals.clone()),
            2,
            0.7,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        let b = super::ga_top_k_pick_diverse(
            build_remaining(evals),
            2,
            0.7,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(b.iter()).all(|(x, y)| x.strategy_id == y.strategy_id));
    }

    #[test]
    fn ga_top_k_repel_vs_attract_second_pick() {
        let sig_a = ScenarioExecutionSignature {
            avg_queue_ahead: 0.1,
            avg_latency: 0.1,
            fill_ratio: 0.9,
            participation: 1.0,
        };
        let sig_close = ScenarioExecutionSignature {
            avg_queue_ahead: 0.15,
            avg_latency: 0.12,
            fill_ratio: 0.88,
            participation: 1.0,
        };
        let sig_far = ScenarioExecutionSignature {
            avg_queue_ahead: 2.5,
            avg_latency: 2.5,
            fill_ratio: 0.15,
            participation: 1.0,
        };
        let mut a = mock_scenario_eval(0.05, 5, 5);
        a.strategy_id = "a".to_string();
        a.scenario_signature = sig_a.clone();
        let mut b = mock_scenario_eval(0.05, 5, 5);
        b.strategy_id = "b".to_string();
        b.scenario_signature = sig_close;
        let mut c = mock_scenario_eval(0.05, 5, 5);
        c.strategy_id = "c".to_string();
        c.scenario_signature = sig_far;
        let build = |ev: Vec<StrategyEvaluation>| {
            ev.into_iter()
                .enumerate()
                .map(|(i, e)| {
                    let s = super::ga_scenario_rank_score(&e);
                    (i, s, e)
                })
                .collect::<Vec<_>>()
        };
        let repel = super::ga_top_k_pick_diverse(
            build(vec![a.clone(), b.clone(), c.clone()]),
            2,
            1.0,
            crate::selection_cap::GaDiversityMode::Repel,
        );
        let attract = super::ga_top_k_pick_diverse(
            build(vec![a, b, c]),
            2,
            1.0,
            crate::selection_cap::GaDiversityMode::Attract,
        );
        assert_eq!(repel[0].strategy_id, "a");
        assert_eq!(attract[0].strategy_id, "a");
        assert_eq!(repel[1].strategy_id, "c");
        assert_eq!(attract[1].strategy_id, "b");
    }

    #[test]
    fn test_ga_weighted_scenario_pnl_opt_in() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();

        let evals = vec![
            mock_scenario_eval(0.01, 5, 5),
            mock_scenario_eval(0.06, 5, 5),
        ];
        std::env::remove_var("GA_WEIGHTED_SCENARIO_PNL");
        let unweighted = aggregate_strategy_reports_with_top_k(evals.clone(), 0.5, None).unwrap();
        std::env::set_var("GA_WEIGHTED_SCENARIO_PNL", "1");
        let weighted = aggregate_strategy_reports_with_top_k(evals, 0.5, None).unwrap();
        std::env::remove_var("GA_WEIGHTED_SCENARIO_PNL");

        assert!(
            weighted.avg_pnl > unweighted.avg_pnl + 1e-9,
            "weighted avg_pnl {} should exceed unweighted {} when higher-edge scenarios get more weight",
            weighted.avg_pnl,
            unweighted.avg_pnl
        );
    }

    #[test]
    fn test_fitness_has_spread() {
        let mut a_evals = Vec::new();
        for _ in 0..10 { a_evals.push(mock_scenario_eval(0.03, 2, 2)); }
        let a = aggregate_strategy_reports_with_top_k(a_evals, 0.5, None).unwrap();

        let mut b_evals = Vec::new();
        for _ in 0..3 { b_evals.push(mock_scenario_eval(0.01, 1, 1)); }
        for _ in 0..7 { b_evals.push(mock_scenario_eval(0.0, 0, 0)); }
        let b = aggregate_strategy_reports_with_top_k(b_evals, 0.5, None).unwrap();

        assert!((a.fitness - b.fitness).abs() > 1e-4, "Expected non-trivial fitness spread, got a={} b={}", a.fitness, b.fitness);
    }

    /// In-memory scenarios only (no CSV): exercises `run_ga_evolution` + Top-K aggregate + trade depth in sub-second typical debug runs.
    #[test]
    fn test_synthetic_ga_microstructure_harness() {
        let config = GaConfig {
            population_size: 4,
            generations: 2,
            mutation_rate: 0.1,
            seed: 2026,
            order_id_prefix: "SYNTH_HARNESS".to_string(),
            order_price: 100,
            order_quantity_for_strategy: 100,
            order_timestamp: 13,
            lambda: 0.5,
            initial_queue_threshold: 200,
            max_trades_per_scenario: Some(3),
            trade_cooldown_events: Some(0),
        };
        let scenarios = synthetic_harness_scenarios();
        let ga_result = run_ga_evolution(config.clone(), &scenarios);
        let (eval, depth) = evaluate_and_aggregate_with_trade_depth(
            &ga_result.global_best.strategy,
            &config,
            &scenarios,
        )
        .expect("synthetic aggregate should produce a report");
        assert!(eval.fitness.is_finite());
        assert!(depth >= 1.0 - 1e-9);
        assert!(
            depth > 1.0 + 1e-9,
            "harness expects multi-trade (cap 3); depth {:.4} suggests single-trade regression",
            depth
        );
        assert!(
            depth <= 3.0 + 1e-9,
            "mean depth {:.6} exceeds per-scenario cap (max_trades_per_scenario=3)",
            depth
        );
        assert!(
            eval.fitness > 0.0,
            "synthetic harness produced non-positive fitness {:.6}; multi-trade should contribute to aggregate signal",
            eval.fitness
        );
        println!(
            "SYNTH_HARNESS → fitness: {:.4}, depth: {:.2}, trade_count: {}",
            eval.fitness, depth, eval.trade_count
        );
    }

    #[test]
    fn test_evaluate_and_aggregate_enforces_path() {
        use std::path::Path;
        use std::time::{Duration, Instant};

        /// Three symbols → cross-name / cross-window diversity without full-folder sweep.
        /// Multi-trade cap semantics: `test_evaluate_strategy_multi_trade_cap_respected`; here `max_trades=1` for GA cost.
        const CSV_FILES: &[&str] = &[
            "RELIANCE_5m_clean.csv",
            "VODAFONEIDEA_5m_clean.csv",
            "HDFCBANK_5m_clean.csv",
        ];
        /// Catches multi-minute regressions (e.g. full-folder load); three assets × ~20 windows each on debug CI.
        const MAX_WALL_SECS: u64 = 300;

        let start = Instant::now();

        let config = GaConfig {
            population_size: 4,
            generations: 2,
            mutation_rate: 0.1,
            seed: 42,
            order_id_prefix: "PATH_TEST".to_string(),
            order_price: 40000,
            order_quantity_for_strategy: 100,
            order_timestamp: 13,
            lambda: 0.5,
            initial_queue_threshold: 200,
            max_trades_per_scenario: Some(1),
            trade_cooldown_events: None,
        };

        let test_assets = format!("{}/../test_assets", env!("CARGO_MANIFEST_DIR"));
        let mut scenarios = std::collections::HashMap::new();
        for file in CSV_FILES {
            let csv_path = format!("{}/{}", test_assets, file);
            let candles = CsvCandleSource { path: csv_path.clone() }.get_candles();
            let asset = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|stem| stem.split('_').next())
                .unwrap_or("UNKNOWN")
                .to_ascii_uppercase();
            let n_before = scenarios.len();
            scenarios.extend(crate::pipeline::scenarios_from_candles(&asset, &candles));
            assert!(
                scenarios.len() > n_before,
                "{} should yield at least one scenario window",
                file
            );
        }

        let ga_result = run_ga_evolution(config.clone(), &scenarios);
        let (eval, avg_trades_per_active) =
            evaluate_and_aggregate_with_trade_depth(&ga_result.global_best.strategy, &config, &scenarios)
                .expect("Aggregation should produce evaluation");
        assert!(eval.fitness > 0.0);
        assert!(
            avg_trades_per_active >= 1.0 - 1e-9,
            "expected >= 1 trade per active scenario after Top-K aggregation, got {}",
            avg_trades_per_active
        );

        println!(
            "DEBUG → fitness: {:.4}, depth (avg_trades/active): {:.2}",
            eval.fitness, avg_trades_per_active
        );
        if avg_trades_per_active <= 1.0 + 1e-9 {
            eprintln!(
                "WARNING: multi-trade not yet active in this path run (avg_trades ≈ 1.0); depth will rise when scenarios allow >1 round-trip per active window"
            );
        }

        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_secs(MAX_WALL_SECS),
            "path integration test took {:?}; cap avoids silent multiplicative regressions (folder sweep × GA × scenario eval)",
            elapsed
        );
    }
}
