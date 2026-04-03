use chronosentiment_core::pipeline;
use chronosentiment_core::PRICE_SCALE;
use chronosentiment_core::ga::{GaConfig, evaluate_robustness, evaluate_ensemble_robustness, ga_top_k_pick_diverse, Strategy, StrategyEvaluation};
use chronosentiment_core::folder_source::FolderCandleSource;
use chronosentiment_core::data_source::CandleSource;
use std::path::PathBuf;
use std::cmp::Ordering;
use std::collections::HashMap;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Asset to process (filters sweep_assets and hardcoded assets)
    #[arg(short, long)]
    asset: Option<String>,

    /// Verify GA determinism by running twice and comparing results
    #[arg(short, long)]
    verify_determinism: bool,

    /// Institutional Deep Validation Mode (20 Pop, 15 Gen, Stabilized Mutation)
    #[arg(long)]
    deep_validation: bool,

    /// Fitness Evaluation Mode (sniper | scalable)
    #[arg(short, long)]
    mode: Option<String>,
}

struct LedgerEntry {
    asset: String,
    fitness: f64,
    act_cv: f64,
    coverage: f64,
    robustness: f64,
    determinism: String,
    classification: String,
    pressure: f64,
    path_drift: f64,
}

fn test_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("chronosentiment_core must live under workspace root")
        .join("test_assets")
}

fn main() {
    let args = Args::parse();
    let data_source = std::env::var("DATA_SOURCE").unwrap_or_else(|_| "folder".to_string()).to_lowercase();
    let folder_path = std::env::var("FOLDER_PATH").unwrap_or_else(|_| "/Users/nikhil/ChronoSentiment_MEGA_FINAL/data/nse/5m".to_string());

    let mut config = GaConfig::default();
    
    // Mode Resolution: CLI > ENV > Default
    if let Some(m) = &args.mode {
        config.fitness_mode = match m.to_lowercase().as_str() {
            "sniper" => chronosentiment_core::ga::FitnessMode::Sniper,
            _ => chronosentiment_core::ga::FitnessMode::Scalable,
        };
    }

    // Population/Generation Overrides (Environment Driven)
    if let Ok(pop) = std::env::var("GA_POPULATION_SIZE") {
        if let Ok(p) = pop.parse::<usize>() {
            config.population_size = p;
        }
    }
    if let Ok(gen) = std::env::var("GA_GENERATIONS") {
        if let Ok(g) = gen.parse::<usize>() {
            config.generations = g;
        }
    }

    println!("⚙️ FITNESS MODE ACTIVE → {:?}", config.fitness_mode);
    println!("⚙️ CONFIG: Pop={}, Gen={}", config.population_size, config.generations);

    if args.deep_validation {
        println!("🚨 DEEP_VALIDATION_MODE: Elevating search depth (Pop=20, Gen=15) and stabilizing mutation...");
        config.population_size = config.population_size.max(20);
        config.generations = config.generations.max(15);
        config.deep_validation = true;
    }

    let mut folder_candles = HashMap::new();
    let mut discovered_assets = Vec::new();

    if data_source == "folder" {
        println!("📂 Loading assets from folder: {}", folder_path);
        let source = FolderCandleSource { folder_path: folder_path.clone() };
        for (asset, candles) in source.load_all_flexible() {
            folder_candles.insert(asset.clone(), candles);
            discovered_assets.push(asset);
        }
        println!("DEBUG_LOADED_KEYS:");
for k in folder_candles.keys() {
    println!(" -> {}", k);
}
    }

    let sweep_assets: Vec<String> = if let Some(target) = &args.asset {
        vec![target.clone()]
    } else if !discovered_assets.is_empty() {
        discovered_assets
    } else {
        vec!["VODAFONEIDEA".to_string()]
    };

    let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "full".to_string()).to_lowercase();

    if run_mode == "train" {
        println!("RUN_MODE=train -> GA Discovery with Institutional Certification...");
        let mut ledger = Vec::new();
        let mut transfer_matrix = Vec::new(); // (From, To, InitialFit, TransferFit, Drop%)
        let mut last_best_strategy: Option<(String, Strategy)> = None;

        for asset_name in &sweep_assets {
            let normalized_asset = asset_name
                .replace("_5m_clean.csv", "")
                .replace("_5m_clean", "")
                .replace(".csv", "");

            println!(
                ">>> Processing Asset: raw={} normalized={}",
                asset_name, normalized_asset
            );

            let candle_ref = folder_candles
                .iter()
                .find(|(k, _)| k.contains(&normalized_asset))
                .map(|(_, v)| v);
            
            let signal_scenario_map = pipeline::scenario_map_for_signal_generation(
                &normalized_asset,
                &data_source,
                candle_ref,
                &folder_path,
            );

            if signal_scenario_map.is_empty() {
                println!("  [!] No signal scenarios found for asset: {}", asset_name);
                continue;
            }

            let exec_asset_raw = pipeline::resolve_execution_symbol(&normalized_asset);
            let fut_path = format!("{}/{}_5m_clean.csv", folder_path, exec_asset_raw);
            let exec_asset = if std::path::Path::new(&fut_path).exists() {
                exec_asset_raw
            } else {
                println!("  [!] No Futures data found for {}, falling back to Spot for execution.", asset_name);
                normalized_asset.clone()
            };

            let exec_candle_ref = folder_candles
                .iter()
                .find(|(k, _)| k.contains(&exec_asset))
                .map(|(_, v)| v);

            let exec_scenario_map = if exec_asset != normalized_asset {
                pipeline::scenario_map_for_signal_generation(
                    &exec_asset,
                    &data_source,
                    exec_candle_ref,
                    &folder_path,
                )
            } else {
                signal_scenario_map.clone()
            };

            if exec_scenario_map.is_empty() {
                println!("  [!] No execution scenarios found for asset: {}", exec_asset);
                continue;
            }

            let scenarios = pipeline::pair_scenarios_by_index(&normalized_asset, &exec_asset, &signal_scenario_map, &exec_scenario_map);
            if scenarios.is_empty() { continue; }

            // Determinism Twin Run
            let mut det_status = "N/A".to_string();
            if args.verify_determinism {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                fn compute_state_hash(res: &chronosentiment_core::ga::GaResult) -> u64 {
                    let mut s = DefaultHasher::new();
                    res.global_best.strategy.hash(&mut s);
                    res.global_best.fitness.to_bits().hash(&mut s);
                    s.finish()
                }

                let res1 = chronosentiment_core::ga::run_ga_evolution(config.clone(), &scenarios);
                let hash1 = compute_state_hash(&res1);
                let res2 = chronosentiment_core::ga::run_ga_evolution(config.clone(), &scenarios);
                let hash2 = compute_state_hash(&res2);
                
                det_status = if hash1 == hash2 { format!("{:X}", hash1) } else { "FAIL".to_string() };
            }

            let ga_res = chronosentiment_core::ga::run_ga_evolution(config.clone(), &scenarios);
            
            // --- PHASE 13: DIVERSIFIED ENSEMBLE SELECTION ---
            // 1. Gather all unique evaluations from the final population
            let mut final_evals = Vec::new();
            if let Some(evs) = chronosentiment_core::ga::evaluate_population_scoped(&ga_res.final_population, &config, &scenarios, 0) {
                for (i, e) in evs.into_iter().enumerate() {
                    let rank_score = chronosentiment_core::ga::ga_scenario_rank_score(&e);
                    final_evals.push((i, rank_score, e));
                }
            }

            // 2. Select Top-5 Diverse Elite Cluster (Repel mode)
            let ensemble_evals = if final_evals.len() >= 5 {
                ga_top_k_pick_diverse(final_evals, 5, 1.0, chronosentiment_core::selection_cap::GaDiversityMode::Repel)
            } else {
                // Fallback to whatever we have
                final_evals.into_iter().map(|(_, _, e)| e).collect()
            };
            
            let ensemble_strategies: Vec<Strategy> = ensemble_evals.iter().map(|e| e.strategy.clone()).collect();
            
            // 3. Final Institutional Ensemble Validation
            println!("🧠 ENSEMBLE_CONSENSUS: Evaluating 5-member diversified ensemble on {}...", asset_name);
            let robustness = evaluate_ensemble_robustness(&ensemble_strategies, &config, &scenarios);
            
            let baseline_fitness = robustness.regime_fitness[0]; // Regime C
            let jitter_fitness = robustness.regime_fitness[2];   // Regime D (index changed in evaluate_ensemble_robustness)
            let path_drift = ((jitter_fitness - baseline_fitness).abs() / baseline_fitness.max(1e-9)) * 100.0;
            let pressure = ga_res.global_best.fitness / (1e-9_f64).max(ga_res.final_generation_best.fitness);

            // Cross-Asset Transfer Test (If last asset available)
            if let Some((from_asset, strat)) = &last_best_strategy {
                println!("🧪 CROSS_ASSET_TRANSFER: Evaluating strategy from {} on {}...", from_asset, asset_name);
                let trans_robustness = evaluate_robustness(strat, &config, &scenarios);
                let trans_fit = trans_robustness.regime_fitness[0];
                let deg = ((ga_res.global_best.fitness - trans_fit) / ga_res.global_best.fitness.max(1e-9)) * 100.0;
                transfer_matrix.push((from_asset.clone(), asset_name.clone(), ga_res.global_best.fitness, trans_fit, deg));
            }
            last_best_strategy = Some((asset_name.clone(), ga_res.global_best.strategy.clone()));

            ledger.push(LedgerEntry {
                asset: asset_name.clone(),
                fitness: ga_res.global_best.fitness,
                act_cv: robustness.internal_cv * 100.0,
                coverage: robustness.participation_rate * 100.0,
                robustness: robustness.robustness_score,
                determinism: det_status,
                classification: robustness.classification,
                pressure,
                path_drift,
            });

            // Institutional Sharpness Guard (The 1.05 Logic)
            if pressure <= 1.05 && robustness.cv * 100.0 > 10.0 {
                println!("⚠️ SHARPNESS_ALERT: [{}] Weak differentiation detected (Pressure: {:.2}, CV: {:.2}%). System may be overfitting noise.", asset_name, pressure, robustness.cv * 100.0);
            } else if pressure <= 1.05 && robustness.cv * 100.0 < 8.0 && robustness.robustness_score > 0.8 {
                println!("✅ PLATEAU_CERTIFIED: [{}] Robust genetic convergence reached. No further sharpening required.", asset_name);
            }
        }

        // --- FINAL INSTITUTIONAL GENERALIZATION LEDGER ---
        println!("\n\n{}", "=".repeat(100));
        println!("📜 INSTITUTIONAL GENERALIZATION LEDGER (PHASE 8)");
        println!("{}", "-".repeat(100));
        println!("{:<12} | {:<7} | {:<7} | {:<8} | {:<10} | {:<12} | {:<12} | {:<8} | {:<8}", 
                 "Asset", "Fit", "ActCV%", "Cover%", "Robust", "Det (Hash)", "Class", "Press", "Drift%");
        println!("{}", "-".repeat(110));
        
        for e in &ledger {
            println!("{:<12} | {:<7.4} | {:<7.2} | {:<8.1} | {:<10.2} | {:<12} | {:<12} | {:<8.2} | {:<8.2}", 
                     e.asset, e.fitness, e.act_cv, e.coverage, e.robustness, e.determinism, e.classification, e.pressure, e.path_drift);
        }
        println!("{}", "=".repeat(100));

        if !transfer_matrix.is_empty() {
            println!("\n🎭 CROSS-ASSET TRANSFER MATRIX");
            println!("{}", "-".repeat(60));
            println!("{:<10} -> {:<10} | {:<7} | {:<7} | {:<6}", "From", "To", "OrigFit", "TransFit", "Drop%");
            println!("{}", "-".repeat(60));
            for (f, t, o, tr, d) in &transfer_matrix {
                println!("{:<10} -> {:<10} | {:<7.4} | {:<7.4} | {:<6.1}%", f, t, o, tr, d);
            }
            println!("{}", "=".repeat(60));
        }
        return;
    }

    println!("Pipeline completed. Run with RUN_MODE=train --deep-validation for Institutional Sweep.");
}
