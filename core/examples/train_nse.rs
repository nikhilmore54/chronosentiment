use chronosentiment_core::folder_source::FolderCandleSource;
/// ChronoSentiment — Multi-Asset NSE GA Training Pipeline
///
/// Trains a Genetic Algorithm on a diversified universe of NSE stocks,
/// producing strategies that generalize across large-cap, cyclical, and
/// mid-cap regimes.
///
/// Usage (train mode):
///   DATA_FOLDER=data/nse/5m RUN_MODE=train \
///   GA_POPULATION_SIZE=60 GA_GENERATIONS=30 GA_MAX_HOLD_BARS=30 \
///   cargo run --example train_nse
///
/// Usage (validate mode):
///   RUN_MODE=validate cargo run --example train_nse -- --validate-on IRCTC.NS
use rand::SeedableRng;
use rand::rngs::StdRng;
use chronosentiment_core::ga::{
    evaluate_and_aggregate, load_elite_strategies, save_elite_population,
    GaConfig, Strategy, StrategyEvaluation, AssetEvoState, GlobalEvoState, AssetSnapshot,
};
use chronosentiment_core::reco::{RecommendationEngine, RecoConfig, RecommendationResult};
use chronosentiment_core::market_adapter::Candle;
use chronosentiment_core::pipeline;
use chronosentiment_core::MarketEvent;
use std::collections::HashMap;

// --- UTILS -------------------------------------------------------------------

/// Normalizes symbol strings (e.g., RELIANCE_5M_CLEAN -> RELIANCE)
fn get_base_symbol(s: &str) -> String {
    s.split(|c| c == '_' || c == '.')
        .next()
        .unwrap_or(s)
        .to_uppercase()
}

// ─── Tier Classification ─────────────────────────────────────────────────────

/// Large-cap symbols (Tier 1) — used for weighted fitness (70%).
const LARGE_CAP_SYMBOLS: &[&str] = &[
    "RELIANCE.NS",
    "HDFCBANK.NS",
    "ICICIBANK.NS",
    "INFY.NS",
    "TCS.NS",
    "LT.NS",
    "BHARTIARTL.NS",
    "ITC.NS",
    "HINDUNILVR.NS",
    "SBIN.NS",
    "KOTAKBANK.NS",
    "AXISBANK.NS",
];

// ─── NEW: AssetDataset ────────────────────────────────────────────────────────

/// NEW: Holds per-symbol candle data and tier classification.
pub struct AssetDataset {
    pub symbol: String,
    pub candles: Vec<Candle>,
    pub is_large_cap: bool,
}

/// [NEW V3.6.1] Session state for synchronous distributed evolution.
struct DistributedAssetSession<'a> {
    symbol: String,
    scenarios: Vec<chronosentiment_core::ga::ScenarioPair<'a>>,
    population: Vec<Strategy>,
    evo_state: AssetEvoState,
    best_evaluations: Vec<StrategyEvaluation>,
    global_best: Option<StrategyEvaluation>,
    current_evaluations: Vec<StrategyEvaluation>,
}

// ─── NEW: Multi-asset math helpers ───────────────────────────────────────────

fn mean_f64(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn variance_f64(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean_f64(values);
    values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / values.len() as f64
}

// ─── NEW: Dataset Loader ─────────────────────────────────────────────────────

/// NEW: Load all CSV files from DATA_FOLDER, classify tiers, filter insufficient data.
fn load_nse_datasets(folder: &str, min_candles: usize) -> Vec<AssetDataset> {
    let source = FolderCandleSource {
        folder_path: folder.to_string(),
    };
    let raw = source.load_all_flexible();

    println!("📂 Found {} CSV files in '{}'", raw.len(), folder);

    let mut datasets = Vec::new();
    for (symbol, candles) in raw {
        if candles.len() < min_candles {
            println!(
                "  ⏭️  SKIP  {} — {} candles < {} minimum",
                symbol,
                candles.len(),
                min_candles
            );
            continue;
        }
        let base = get_base_symbol(&symbol);
        let is_large_cap = LARGE_CAP_SYMBOLS
            .iter()
            .any(|lc| get_base_symbol(lc).eq_ignore_ascii_case(&base));

        let tier = if is_large_cap { "LARGE" } else { "MID/SMALL" };
        println!(
            "  ✅ LOAD  {:<20} — {} candles [{}]",
            symbol,
            candles.len(),
            tier
        );
        datasets.push(AssetDataset {
            symbol,
            candles,
            is_large_cap,
        });
    }

    println!(
        "\n📊 Loaded {} assets — {} large-cap, {} mid/small\n",
        datasets.len(),
        datasets.iter().filter(|a| a.is_large_cap).count(),
        datasets.iter().filter(|a| !a.is_large_cap).count(),
    );
    datasets
}

// ─── NEW: Per-asset scenario helpers ─────────────────────────────────────────

/// Build scenario pairs for a single asset. Returns empty vec if insufficient.
fn build_scenarios<'a>(
    symbol: &'a str,
    signal_map: &'a HashMap<String, Vec<MarketEvent>>,
) -> Vec<chronosentiment_core::ga::ScenarioPair<'a>> {
    pipeline::pair_scenarios_by_index(symbol, symbol, signal_map, signal_map)
}

// ─── NEW: Multi-asset fitness ─────────────────────────────────────────────────

struct CrossAssetMetrics {
    fitness: f64,
    avg_pnl_large: f64,
    avg_pnl_small: f64,
    pnl_variance: f64,
    assets_evaluated: usize,
    avg_win_rate: f64,
    avg_capture_eff: f64,
    avg_selectivity: f64,
    avg_entropy: f64,
    avg_conviction: f64,
    avg_aqg_health: f64,
    aqg_skip_ratio: f64,
    avg_payoff: f64,
    avg_participation: f64,

    avg_hold_time: f64,
    total_trades: usize,
    profitable_trades: usize,
    avg_edge_spread: f64,
    avg_dominance: f64,
}

fn evaluate_cross_asset(
    strategy: &Strategy,
    config: &GaConfig,
    per_asset_maps: &[(String, bool, HashMap<String, Vec<MarketEvent>>)],
) -> CrossAssetMetrics {
    let mut pnl_large = Vec::new();
    let mut pnl_small = Vec::new();
    let mut pnl_all = Vec::new();
    let mut entropies = Vec::new();
    let mut selectivities = Vec::new();
    let mut convictions = Vec::new();
    let mut total_trades = 0;
    let mut profitable_trades = 0;
    let mut win_rates = Vec::new();
    let mut capture_effs = Vec::new();
    let mut aqg_healths = Vec::new();
    let mut aqg_skips = Vec::new();
    let mut payoffs = Vec::new();
    let mut participations = Vec::new();
    let mut hold_times = Vec::new();
    let mut assets_evaluated = 0;
    let mut _active_scenarios = 0;
    let mut edge_spreads = Vec::new();
    let mut dominances = Vec::new();

    for (symbol, is_large_cap, signal_map) in per_asset_maps {
        let scenarios = build_scenarios(symbol, signal_map);
        if scenarios.is_empty() {
            continue;
        }

        assets_evaluated += 1;
        if let Some(eval) = evaluate_and_aggregate(strategy, config, &scenarios, 0, 1.0, 0, 1.0, 0) {
            if eval.trade_count > 0 {
                pnl_all.push(eval.avg_pnl);
                total_trades += eval.trade_count;
                profitable_trades += eval.profitable_trades;
                entropies.push(eval.avg_entropy);
                selectivities.push(eval.selectivity);
                convictions.push(eval.avg_conviction);
                win_rates.push(eval.win_rate);
                capture_effs.push(eval.avg_efficiency);
                aqg_healths.push(eval.avg_aqg_health);
                aqg_skips.push(eval.aqg_skip_ratio);
                payoffs.push(eval.payoff);
                participations.push(eval.participation_rate);
                hold_times.push(eval.avg_hold_time);
                edge_spreads.push(eval.avg_edge_spread);
                dominances.push(eval.avg_dominance);
                _active_scenarios += 1;

                if *is_large_cap {
                    pnl_large.push(eval.avg_pnl);
                } else {
                    pnl_small.push(eval.avg_pnl);
                }
            }
        }
    }

    let avg_pnl_large = mean_f64(&pnl_large);
    let avg_pnl_small = mean_f64(&pnl_small);
    let avg_win_rate = mean_f64(&win_rates);
    let avg_capture_eff = mean_f64(&capture_effs);
    let pnl_variance = variance_f64(&pnl_all);
    let eps = 1e-9_f64;

    // MODIFIED: Multi-asset weighted fitness with overfitting penalty
    let raw_fitness =
        0.7 * avg_pnl_large + 0.3 * avg_pnl_small + 0.3 * avg_win_rate + 0.2 * avg_capture_eff;

    let fitness = (raw_fitness - 0.2 * pnl_variance).max(-1.0);

    CrossAssetMetrics {
        fitness: fitness.max(eps - 1.0),
        avg_pnl_large,
        avg_pnl_small,
        pnl_variance,
        assets_evaluated,
        avg_win_rate,
        avg_capture_eff,
        avg_selectivity: mean_f64(&selectivities),
        avg_entropy: mean_f64(&entropies),
        avg_conviction: mean_f64(&convictions),
        avg_aqg_health: mean_f64(&aqg_healths),
        aqg_skip_ratio: mean_f64(&aqg_skips),
        avg_payoff: mean_f64(&payoffs),
        avg_participation: mean_f64(&participations),

        avg_hold_time: mean_f64(&hold_times),
        total_trades,
        profitable_trades,
        avg_edge_spread: mean_f64(&edge_spreads),
        avg_dominance: mean_f64(&dominances),
    }
}

// ─── TRAIN MODE ───────────────────────────────────────────────────────────────

fn run_train(config: &GaConfig, datasets: &[AssetDataset], elite_path: &str) {
    println!("🧬 MULTI-ASSET GA TRAINING");
    println!("   Population : {}", config.population_size);
    println!("   Generations: {}", config.generations);
    println!("   Max hold   : {} bars", config.max_hold_bars);
    println!("   Assets     : {}", datasets.len());
    println!("{}", "=".repeat(90));

    // Step 1: Pre-compute scenario maps for every asset (HashMap<String, Vec<MarketEvent>>)
    let mut per_asset_maps: Vec<(String, bool, HashMap<String, Vec<MarketEvent>>)> = Vec::new();
    for asset in datasets {
        let signal_map = pipeline::scenario_map_for_signal_generation(
            &asset.symbol,
            "folder",
            Some(&asset.candles),
            "",
        );
        let n_scenarios = build_scenarios(&asset.symbol, &signal_map).len();
        if n_scenarios == 0 {
            println!("  ⚠️  {} — 0 scenarios generated, skipping.", asset.symbol);
            continue;
        }
        println!("  📐 {} — {} scenarios", asset.symbol, n_scenarios);
        per_asset_maps.push((asset.symbol.clone(), asset.is_large_cap, signal_map));
    }

    if per_asset_maps.is_empty() {
        eprintln!("❌ No valid assets with scenarios. Aborting training.");
        return;
    }

    println!(
        "\n🔬 PASS 1/{} — Per-asset GA evolution",
        per_asset_maps.len()
    );
    println!("{}", "-".repeat(70));

    // Step 2: Initialize Distributed Organism
    let mut candidate_strategies: Vec<Strategy> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    let global_state_path = "global_state.json";
    let mut global_evo = if std::path::Path::new(global_state_path).exists() {
        let content = std::fs::read_to_string(global_state_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        GlobalEvoState::default()
    };

    println!("🌐 INITIAL_GLOBAL_STATE | bias: {:.2} | agreement: {:.2} | progress: {:.4}", 
        global_evo.expansion_bias, global_evo.agreement_ema, global_evo.progress_ema);

    let mut sessions: Vec<DistributedAssetSession> = Vec::new();
    let mut rng = StdRng::seed_from_u64(config.seed);

    for (symbol, _is_lc, signal_map) in &per_asset_maps {
        let scenarios = build_scenarios(symbol, signal_map);
        if scenarios.is_empty() { continue; }
        
        sessions.push(DistributedAssetSession {
            symbol: symbol.clone(),
            scenarios,
            population: chronosentiment_core::ga::initialize_population(config, &mut rng),
            evo_state: AssetEvoState { symbol: symbol.clone(), ..AssetEvoState::default() },
            best_evaluations: Vec::new(),
            global_best: None,
            current_evaluations: Vec::new(),
        });
    }

    println!("\n🧬 STARTING SYNCHRONOUS DISTRIBUTED EVOLUTION ({} Assets, {} Gens)", sessions.len(), config.generations);
    println!("{}", "-".repeat(90));

    // MAIN GENERATIONAL LOOP (V3.6.1 Strict 5-Phase)
    for gen in 0..config.generations {
        let mut snapshots: std::collections::HashMap<String, AssetSnapshot> = std::collections::HashMap::new();

        // PHASE 1: EVALUATE & PHASE 2: CAPTURE
        for session in &mut sessions {
            let diversity = chronosentiment_core::ga::calculate_population_diversity(&session.population);
            let unique_count = session.population.iter().collect::<std::collections::HashSet<_>>().len();

            let (evals_opt, _) = chronosentiment_core::ga::evaluate_population_scoped(
                &session.population,
                config,
                &session.scenarios,
                gen,
                diversity,
                unique_count,
                global_evo.expansion_bias,
            );

            if let Some(mut evaluations) = evals_opt {
                evaluations.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
                
                if let Some(best) = evaluations.first() {
                    // Update session bests
                    if session.global_best.is_none() || best.fitness > session.global_best.as_ref().unwrap().fitness {
                        session.global_best = Some(best.clone());
                    }
                    if gen == config.generations - 1 {
                        session.best_evaluations.extend(evaluations.clone());
                    }
                    
                    // Diversity Guard Log (V3.6.7+ Hardening)
                    println!("🧬 DIVERSITY  | {:<12} | unique_genomes: {}", session.symbol, unique_count);

                    session.current_evaluations = evaluations.clone();

                    // Update local evolution state (primitive fields)
                    let log_queues: Vec<f64> = evaluations.iter()
                        .map(|e| (1.0 + e.scenario_signature.avg_queue_ahead).ln())
                        .collect();
                    session.evo_state.prev_max_log_queue = session.evo_state.max_log_queue;
                    session.evo_state.max_log_queue = log_queues.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    session.evo_state.delta_log_q = session.evo_state.max_log_queue - session.evo_state.prev_max_log_queue;
                    session.evo_state.trade_density = evaluations.iter().map(|e| e.trade_count).sum::<usize>() as f64 / evaluations.len() as f64;
                    session.evo_state.fill_rate = evaluations.iter().map(|e| e.execution_metrics.fill_rate as f64).sum::<f64>() / evaluations.len() as f64;
                    
                    // [V3.6.2] Relative Stability: Ignore early noise phase
                    // [V3.6.3 Hardened] Lagged Stability Perception & Relative Maturity
                    let maturity = session.evo_state.max_log_queue / global_evo.global_max_log_q.max(1e-6);
                    let is_stable = maturity > 0.3 && session.evo_state.delta_log_q.abs() < (0.5 * global_evo.energy_ema_prev);
                    
                    if is_stable {
                        session.evo_state.stability_streak += 1;
                    } else {
                        session.evo_state.stability_streak = 0;
                    }
                    
                    // HARDENING 1: Snapshot Integrity Check
                    let snap = AssetSnapshot {
                        symbol: session.symbol.clone(),
                        max_log_queue: session.evo_state.max_log_queue,
                        delta_log_q: session.evo_state.delta_log_q,
                        trade_density: session.evo_state.trade_density,
                        fill_rate: session.evo_state.fill_rate,
                        stability_streak: session.evo_state.stability_streak,
                    };
                    
                    #[cfg(debug_assertions)]
                    {
                        debug_assert!(snap.fill_rate >= 0.0 && snap.fill_rate <= 2.0); // fill_rate can occasionally be > 1 due to slippage artifacts
                        debug_assert!(!snap.delta_log_q.is_nan());
                    }

                    snapshots.insert(session.symbol.clone(), snap);
                }
            }
        }

        // PHASE 3: AGGREGATE
        global_evo.aggregate(&snapshots, gen);

        // [V3.6.6] Alignment Anchor Capture
        if global_evo.post_strike_cooldown == 3 {
            let mut converged_evals = Vec::new();
            for session in &sessions {
                if session.evo_state.stability_streak >= 3 {
                    if let Some(best) = &session.global_best {
                        converged_evals.push(best);
                    }
                }
            }
            if !converged_evals.is_empty() {
                global_evo.alignment_anchor = Some(chronosentiment_core::ga::calculate_alignment_centroid(converged_evals));
                println!("🧠 ANCHOR_CAPTURED | assets_aligned: {} | Genes Anchored", global_evo.prev_converged_assets);
            }
        }
        if global_evo.post_strike_cooldown == 0 {
            global_evo.alignment_anchor = None;
        }

        // [V3.6.7] Global Mean Capture (Maturity Gated)
        if global_evo.global_max_log_q > 0.15 {
            let mut best_evals = Vec::new();
            for session in &sessions {
                if let Some(best) = &session.global_best {
                    best_evals.push(best);
                }
            }
            if !best_evals.is_empty() {
                global_evo.global_mean = Some(chronosentiment_core::ga::calculate_alignment_centroid(best_evals));
            }
        } else {
            global_evo.global_mean = None;
        }

        // Diagnostic Reporting (V3.6.7)
        if global_evo.pull_strength > 0.0 && global_evo.global_mean.is_some() {
            println!("🧲 MEAN_PULL_ACTIVE | strength: {:.4} | maturity: {:.4}", 
                global_evo.pull_strength, global_evo.global_max_log_q);
        }

        // [V3.6.1] LOG SAMPLES (2-3 assets per gen as requested)
        let sample_count = if sessions.len() > 3 { 3 } else { sessions.len() };
        for i in 0..sample_count {
            let s = &sessions[i];
            println!("  🧬 ASSET_SAMPLE | {:<12} | gen: {:<2} | best={:.4} | delta_q={:+.4} | trades={} | anchor={}", 
                s.symbol, gen, s.global_best.as_ref().map(|b| b.fitness).unwrap_or(0.0), 
                s.evo_state.delta_log_q, s.evo_state.trade_density as usize,
                if global_evo.alignment_anchor.is_some() { "ACTIVE" } else { "OFF" });
        }

        // PHASE 4: APPLY BIAS & PHASE 5: EVOLVE
        for session in &mut sessions {
            // [V3.6.8] DETERMINISTIC DIVERSITY CONTROL
            let effective_diversity = chronosentiment_core::ga::calculate_effective_diversity(&session.current_evaluations);
            let progress = gen as f64 / config.generations as f64;
            let threshold = 0.3f64.max(1.0 - progress);

            if effective_diversity < threshold {
                let severity = (threshold - effective_diversity) / threshold;
                let inject_rate = (0.05 + 0.1 * severity).clamp(0.05, 0.15);
                let inject_count = (session.population.len() as f64 * inject_rate).ceil() as usize;
                
                // Elite Protection: Never replace the Top 10% (min 2)
                let elite_k = ((session.population.len() as f64 * 0.1).ceil() as usize).max(2);
                
                // Deterministic Injection from bottom-up
                let start_idx = session.population.len().saturating_sub(inject_count).max(elite_k);
                for i in start_idx..session.population.len() {
                    let seed = chronosentiment_core::ga::stable_deterministic_hash((gen as u64, i as u64, (effective_diversity * 1000.0) as u64));
                    if let Some(best) = &session.global_best {
                        // Orthogonal Mutant of the best individual
                        session.population[i] = best.strategy.orthogonal_mutant(seed);
                    } else {
                        // Total Reset if no alpha found yet
                        session.population[i] = chronosentiment_core::ga::Strategy::from_seed(seed);
                    }
                }
                println!("  🧬 DIVERSITY_INJECTION | asset={} | div={:.2} | threshold={:.2} | inject={}", 
                    session.symbol, effective_diversity, threshold, inject_count);
            }

            // [V3.6.3 Hardened] Sigmoid Exploration Cooling
            let phase = gen as f64 / config.generations as f64;
            let cooling_factor = 1.0 - 0.5 * (1.0 / (1.0 + (-10.0 * (phase - 0.6)).exp()));
            
            // [V3.6.5] Survival Dynamics: Dampening & Stabilization
            let effective_bias = 1.0 + (global_evo.expansion_bias - 1.0) * 0.5;
            let mut survival_multiplier = 1.0;
            
            // Post-Strike Stabilization Window (3 gens)
            if global_evo.post_strike_cooldown > 0 {
                survival_multiplier *= 0.7; // V3.6.6: Stronger Elitism
            }

            // [V3.6.7] Pre-alignment damping: Catch the regime before it dissipates
            if global_evo.agreement_ema > 0.25 {
                survival_multiplier *= 0.92;
            }

            // [V3.6.7+] Behavioral Stability Detection
            let is_asset_stable = session.evo_state.delta_log_q.abs() < (0.5 * global_evo.energy_ema_prev);
            let eval_stability = vec![is_asset_stable; session.population.len()];
            
            // Energy Rebound Guard: Prevent chaos spikes
            let energy_jump = global_evo.energy_ema / global_evo.energy_ema_prev.max(1e-6);
            if energy_jump > 1.5 {
                survival_multiplier *= 0.8;
            }

            // [V3.6.8] Anti-Correlated Mutation Floor: Scale mutation up as diversity drops
            let mutation_boost = (1.0 - effective_diversity).max(0.0) * 0.3;
            let mut local_evo = session.evo_state.clone();
            local_evo.mutation_scale = (session.evo_state.mutation_scale * cooling_factor + mutation_boost) * effective_bias * survival_multiplier;

            session.population = chronosentiment_core::ga::evolve_generation(
                &session.current_evaluations, // Restored population-based evolution pool
                config,
                &mut rng,
                &local_evo,
                global_evo.post_strike_cooldown,
                global_evo.alignment_anchor.as_ref(),
                global_evo.global_mean.as_ref(),
                global_evo.pull_strength,
                gen,
                &eval_stability,
            );
        }
    }

    // Wrap up results
    for session in &sessions {
        if let Some(best) = &session.global_best {
            candidate_strategies.push(best.strategy.clone());
        }
        for eval in &session.best_evaluations {
            let id = format!("{}-{}-{}-{}", eval.strategy.queue_threshold, eval.strategy.base_edge, eval.strategy.take_profit, eval.strategy.stop_loss);
            if seen_ids.insert(id) {
                candidate_strategies.push(eval.strategy.clone());
            }
        }
    }

    // Persist finalized global state for the next session
    let global_json = serde_json::to_string_pretty(&global_evo).unwrap_or_default();
    let _ = std::fs::write(global_state_path, global_json);

    println!(
        "\n📦 Candidate pool: {} unique strategies from {} assets",
        candidate_strategies.len(),
        per_asset_maps.len()
    );
    println!("{}", "=".repeat(90));

    // Step 3: Cross-asset re-evaluation of ALL candidates
    println!("\n🔁 CROSS-ASSET EVALUATION — Scoring all candidates against all assets");
    println!("{}", "-".repeat(90));
    println!(
        "{:<6} | {:<7} | {:<9} | {:<9} | {:<9} | {:<10} | {:<9} | {:<6}",
        "Rank", "Fitness", "PnL-Lrg", "PnL-Sml", "WinRate", "Capture", "Variance", "Evald"
    );
    println!("{}", "-".repeat(90));

    let mut scored: Vec<(Strategy, CrossAssetMetrics)> = Vec::new();
    for strategy in &candidate_strategies {
        let metrics = evaluate_cross_asset(strategy, config, &per_asset_maps);
        scored.push((strategy.clone(), metrics));
    }

    // Sort descending by multi-asset fitness
    scored.sort_by(|a, b| {
        b.1.fitness
            .partial_cmp(&a.1.fitness)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Log top 20
    for (rank, (_, m)) in scored.iter().take(20).enumerate() {
        println!(
            "{:<6} | {:<7.4} | {:<9.4} | {:<9.4} | {:<9.4} | {:<10.4} | {:<9.4} | {:<6}",
            rank + 1,
            m.fitness,
            m.avg_pnl_large,
            m.avg_pnl_small,
            m.avg_win_rate,
            m.avg_capture_eff,
            m.pnl_variance,
            m.assets_evaluated,
        );
    }

    println!("{}", "=".repeat(90));

    // Step 4: Build elite StrategyEvaluations from top candidates
    let top_n = (config.population_size / 4).max(5).min(scored.len());
    let mut elite_evals: Vec<StrategyEvaluation> = Vec::new();

    for (strategy, metrics) in scored.iter().take(top_n) {
        let eval = StrategyEvaluation {
            strategy_id: format!("NSE-MT-{}-{}", strategy.queue_threshold, strategy.base_edge),
            strategy: strategy.clone(),
            capability: chronosentiment_core::ga::ScenarioCapability::Executable,
            fitness: metrics.fitness,
            avg_pnl: metrics.avg_pnl_large * 0.7 + metrics.avg_pnl_small * 0.3,
            std_dev: metrics.pnl_variance.sqrt(),
            downside_std_dev: 0.0, // Aggregated proxy
            worst: 0.0,
            robustness: 0.0,
            trade_count: metrics.total_trades,
            max_drawdown: 0.0,
            participation_rate: metrics.avg_participation,
            profitable_trades: metrics.profitable_trades,
            zero_pnl_trades: 0,
            quality_trades: 0.0,
            win_rate: metrics.avg_win_rate,
            payoff: metrics.avg_payoff,
            payoff_ratio: 0.0,
            execution_metrics: chronosentiment_core::ga::ExecutionMetrics {
                fill_efficiency: metrics.avg_capture_eff, // Using capture_eff as proxy
                capture_efficiency: metrics.avg_capture_eff,
                fill_rate: metrics.avg_capture_eff as f32,
                avg_slippage: 0.0,
                latency_impact: 0.0,
                queue_blocked_count: 0,
                liquidity_starved_count: 0,
                total_attempts: metrics.total_trades,
            },
            scenario_signature: chronosentiment_core::ga::ScenarioExecutionSignature {
                avg_queue_ahead: 0.0,
                avg_latency: 0.0,
                fill_ratio: metrics.avg_capture_eff,
                participation: metrics.avg_participation,
                execution_variance: 0.0,
            },
            avg_conviction: metrics.avg_conviction,
            avg_efficiency: metrics.avg_capture_eff,
            avg_edge_quality: metrics.avg_capture_eff,
            directional_accuracy: metrics.avg_win_rate,
            decisiveness: metrics.avg_selectivity,
            execution_friction: 1.0 - metrics.avg_capture_eff,
            short_term_capture_eff: 1.0,
            long_term_capture_eff: 1.0,
            realized_pnl_rolling: metrics.avg_pnl_large,
            predicted_pnl_rolling: metrics.avg_pnl_large,
            exit_tp_count: 0,
            exit_sl_count: 0,
            exit_ts_count: 0,
            avg_hold_time: metrics.avg_hold_time,
            consistency_score: 1.0 - metrics.pnl_variance.clamp(0.0, 1.0),
            recent_performance: metrics.fitness,
            pnl_from_tp: 0.0,
            pnl_from_sl: 0.0,
            max_trade_pnl: 0.0,
            pnl_fingerprint: Vec::new(),
            selectivity: metrics.avg_selectivity,
            avg_entropy: metrics.avg_entropy,
            avg_aqg_health: metrics.avg_aqg_health,
            aqg_skip_ratio: metrics.aqg_skip_ratio,
            avg_edge_spread: metrics.avg_edge_spread,
            avg_dominance: metrics.avg_dominance,
            ..Default::default()
        };
        elite_evals.push(eval);
    }

    // Step 5: Final summary
    let avg_cross_fitness = mean_f64(
        &scored
            .iter()
            .take(top_n)
            .map(|(_, m)| m.fitness)
            .collect::<Vec<_>>(),
    );
    let avg_large_pnl = mean_f64(
        &scored
            .iter()
            .take(top_n)
            .map(|(_, m)| m.avg_pnl_large)
            .collect::<Vec<_>>(),
    );
    let avg_small_pnl = mean_f64(
        &scored
            .iter()
            .take(top_n)
            .map(|(_, m)| m.avg_pnl_small)
            .collect::<Vec<_>>(),
    );
    let avg_variance = mean_f64(
        &scored
            .iter()
            .take(top_n)
            .map(|(_, m)| m.pnl_variance)
            .collect::<Vec<_>>(),
    );

    println!("\n📊 TRAINING COMPLETE");
    println!("   Elites selected  : {}", elite_evals.len());
    println!("   Avg cross-fitness: {:.4}", avg_cross_fitness);
    println!("   Avg PnL (large)  : {:.4}", avg_large_pnl);
    println!("   Avg PnL (small)  : {:.4}", avg_small_pnl);
    println!(
        "   Avg PnL variance : {:.4} (overfitting indicator)",
        avg_variance
    );

    // Step 6: Save elites
    let elite_dir = std::path::Path::new(elite_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("core/elite"))
        .to_str()
        .unwrap_or("core/elite");

    std::fs::create_dir_all(elite_dir).unwrap_or(());

    match save_elite_population(&elite_evals, config, elite_dir) {
        Ok(path) => {
            // Also write to fixed intraday_nse.json (timestamped version already saved)
            let json =
                serde_json::to_string_pretty(&chronosentiment_core::ga::ElitePopulationBundle {
                    metadata: chronosentiment_core::ga::PersistenceMetadata {
                        timestamp: chrono::Utc::now().format("%Y-%m-%d_%H-%M").to_string(),
                        avg_fitness: avg_cross_fitness,
                        avg_pnl: avg_large_pnl,
                        cv: avg_variance.sqrt().max(0.0),
                        regime_profile: chronosentiment_core::ga::RegimeProfile {
                            volatility: avg_variance,
                            liquidity: 0.5,
                            participation: avg_cross_fitness.clamp(0.0, 1.0),
                            label: "multi_asset_nse".to_string(),
                            timestamp: chrono::Utc::now().timestamp() as u64,
                        },
                        strategies_count: elite_evals.len(),
                        fitness_mode: config.fitness_mode,
                    },
                    strategies: elite_evals.clone(),
                })
                .unwrap_or_default();

            let nse_path = format!("{}/intraday_nse.json", elite_dir);
            // Do NOT overwrite if file exists without explicit consent
            if std::path::Path::new(&nse_path).exists() {
                let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
                let backup_path = format!("{}/intraday_nse_{}.json", elite_dir, ts);
                std::fs::copy(&nse_path, &backup_path).ok();
                println!(
                    "  📦 Existing intraday_nse.json backed up → {}",
                    backup_path
                );
            }
            std::fs::write(&nse_path, json).ok();

            println!("  💾 Elites saved → {}", path);
            println!("  💾 NSE elites  → {}", nse_path);
        }
        Err(e) => eprintln!("❌ Failed to save elites: {}", e),
    }

    // Step 7: Final Recommendation Dashboard (V4.1.0)
    println!("\n🚀 FINAL RECOMMENDATION DASHBOARD — Identifying high-confidence opportunities");
    println!("{}", "=".repeat(90));
    println!(
        "{:<12} | {:<4} | {:<12} | {:<7} | {:<15} | {:<5}",
        "Asset", "Act", "Decision", "Conf", "Execution", "Cons"
    );
    println!("{}", "-".repeat(90));

    let reco_config = RecoConfig::default();
    
    for (symbol, _is_lc, signal_map) in &per_asset_maps {
        // Find the latest evaluations for this session
        if let Some(session) = sessions.iter().find(|s| &s.symbol == symbol) {
            // Pick a representative market snapshot (last available window)
            let latest_window_id = signal_map.keys().last().cloned().unwrap_or_default();
            let market_snapshot = signal_map.get(&latest_window_id).cloned().unwrap_or_default();

            let result = RecommendationEngine::process(
                &session.current_evaluations,
                &market_snapshot,
                &reco_config,
                symbol
            );

            match result {
                RecommendationResult::Trade(r) => {
                    println!("\x1b[92m{:<12} | {:<4} | {:<12} | {:.2}    | {:<15} | {:<5}\x1b[0m",
                        symbol, format!("{:?}", r.action), "TRADE", r.confidence.total, 
                        format!("Fill:{:.2}", r.execution.fill_probability),
                        format!("{:.2}", r.consensus.agreement_score)
                    );
                },
                RecommendationResult::WeakSignal(r) => {
                    println!("\x1b[93m{:<12} | {:<4} | {:<12} | {:.2}    | {:<15} | {:<5}\x1b[0m",
                        symbol, format!("{:?}", r.action), "WEAK_SIGNAL", r.confidence.total,
                        format!("Fill:{:.2}", r.execution.fill_probability),
                        format!("{:.2}", r.consensus.agreement_score)
                    );
                },
                RecommendationResult::NoTrade { reason, metrics } => {
                    println!("{:<12} | {:<4} | {:<12} | {:.2}    | {:<15} | {:<5}",
                        symbol, "---", format!("{:?}", reason), 0.0,
                        format!("F:{:.2} S:{:.2}", metrics.execution_score, metrics.stability),
                        format!("{:.2}", metrics.agreement)
                    );
                }
            }
        }
    }
    println!("{}", "=".repeat(90));
}

// ─── VALIDATE MODE ────────────────────────────────────────────────────────────

fn run_validate(config: &GaConfig, datasets: &[AssetDataset], elite_path: &str, validate_on: &str) {
    println!("🔍 VALIDATION MODE — Testing elites on: {}", validate_on);
    println!("{}", "=".repeat(70));

    // Load elites
    let elites = load_elite_strategies(elite_path);
    if elites.is_empty() {
        eprintln!(
            "❌ No elites found at '{}'. Run train mode first.",
            elite_path
        );
        return;
    }
    println!("📥 Loaded {} elite strategies", elites.len());

    // Find validation asset
    let val_asset = datasets.iter().find(|a| {
        a.symbol.eq_ignore_ascii_case(validate_on)
            || a.symbol
                .to_uppercase()
                .contains(&validate_on.to_uppercase())
    });

    let (symbol, candles) = match val_asset {
        Some(a) => (&a.symbol, &a.candles),
        None => {
            eprintln!(
                "❌ Asset '{}' not found in dataset. Available:",
                validate_on
            );
            for a in datasets {
                eprintln!("   {}", a.symbol);
            }
            return;
        }
    };

    println!(
        "📊 Validation asset: {} ({} candles)",
        symbol,
        candles.len()
    );

    let signal_map =
        pipeline::scenario_map_for_signal_generation(symbol, "folder", Some(candles), "");
    let scenarios = build_scenarios(symbol, &signal_map);
    if scenarios.is_empty() {
        eprintln!("❌ No scenarios generated for {}.", symbol);
        return;
    }
    println!("📐 {} scenarios generated", scenarios.len());
    println!("{}", "-".repeat(70));
    println!(
        "{:<6} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8} | {:<6}",
        "Elite", "Fitness", "AvgPnL", "WinRate", "Capture", "Trades", "EXEC%"
    );
    println!("{}", "-".repeat(70));

    let mut total_pnl = 0.0_f64;
    let mut total_exec = 0.0_f64;
    let mut total_cap = 0.0_f64;
    let mut eval_count = 0_usize;

    for (i, elite) in elites.iter().enumerate() {
        if let Some(eval) = evaluate_and_aggregate(&elite.strategy, config, &scenarios, 0, 1.0, 0, 1.0, 0) {
            let exec_ratio = if eval.trade_count > 0 {
                (eval.profitable_trades + eval.zero_pnl_trades) as f64 / eval.trade_count as f64
            } else {
                0.0
            };
            let _wait_ratio = 1.0 - eval.participation_rate;

            println!(
                "{:<6} | {:<8.4} | {:<8.4} | {:<8.4} | {:<8.4} | {:<8} | {:<6.1}%",
                i + 1,
                eval.fitness,
                eval.avg_pnl,
                eval.participation_rate,
                eval.avg_efficiency,
                eval.trade_count,
                exec_ratio * 100.0,
            );

            total_pnl += eval.avg_pnl;
            total_exec += exec_ratio;
            total_cap += eval.avg_efficiency;
            eval_count += 1;
        }
    }

    println!("{}", "=".repeat(70));
    if eval_count > 0 {
        println!("VALIDATION SUMMARY — {}", symbol);
        println!(
            "  Avg PnL              : {:.4}",
            total_pnl / eval_count as f64
        );
        println!(
            "  Avg capture effcty   : {:.4}",
            total_cap / eval_count as f64
        );
        println!(
            "  Avg EXEC ratio       : {:.1}%",
            (total_exec / eval_count as f64) * 100.0
        );
        println!(
            "  WAIT ratio           : {:.1}%",
            (1.0 - total_exec / eval_count as f64) * 100.0
        );
    }
}

// ─── MAIN ─────────────────────────────────────────────────────────────────────

fn main() {
    // ── Config from ENV ──────────────────────────────────────────────────────
    let data_folder = std::env::var("DATA_FOLDER").unwrap_or_else(|_| "data/nse/5m".to_string());
    let run_mode = std::env::var("RUN_MODE")
        .unwrap_or_else(|_| "train".to_string())
        .to_lowercase();
    let elite_path =
        std::env::var("ELITE_PATH").unwrap_or_else(|_| "core/elite/intraday_nse.json".to_string());
    let min_candles: usize = std::env::var("MIN_CANDLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);

    // ── CLI args ─────────────────────────────────────────────────────────────
    let cli_args: Vec<String> = std::env::args().collect();
    let validate_on = cli_args
        .windows(2)
        .find(|w| w[0] == "--validate-on")
        .map(|w| w[1].as_str())
        .unwrap_or("IRCTC.NS");

    // ── GaConfig ─────────────────────────────────────────────────────────────
    let config = GaConfig::default(); // reads GA_POPULATION_SIZE, GA_GENERATIONS, GA_MAX_HOLD_BARS from ENV

    println!("\n🚀 CHRONOSENTIMENT — NSE MULTI-ASSET TRAINING PIPELINE");
    println!("{}", "=".repeat(70));
    println!("   RUN_MODE   : {}", run_mode);
    println!("   DATA_FOLDER: {}", data_folder);
    println!("   ELITE_PATH : {}", elite_path);
    println!("   Pop size   : {}", config.population_size);
    println!("   Generations: {}", config.generations);
    println!("   Max hold   : {} bars", config.max_hold_bars);

    // ── Load datasets ─────────────────────────────────────────────────────────
    let datasets = load_nse_datasets(&data_folder, min_candles);
    if datasets.is_empty() {
        eprintln!(
            "❌ No valid assets loaded from '{}'. \
             Run: python3 scripts/download_nse_data.py --interval 5m --period 60d",
            data_folder
        );
        return;
    }

    // ── Dispatch ──────────────────────────────────────────────────────────────
    match run_mode.as_str() {
        "train" => run_train(&config, &datasets, &elite_path),
        "validate" => run_validate(&config, &datasets, &elite_path, validate_on),
        other => eprintln!(
            "❌ Unknown RUN_MODE='{}'. Use 'train' or 'validate'.",
            other
        ),
    }
}
