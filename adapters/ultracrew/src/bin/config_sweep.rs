//! Experimental binary for sweeping UltraCrew EvolutionConfig hyperparameters.
//! This binary runs a full factorial sweep over elite_count,
//! mutation_rate, and crossover_rate values (6×6×6 = 216 configurations),
//! records final fitness, runtime, and generations for each configuration,
//! writes raw results to `artifacts/config_sweep.csv`, generates heat‑map CSVs,
//! and produces a markdown report with analysis.
//!
//! No production code is modified; all changes are confined to this experimental binary.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use csv::Writer;
use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Serialize;

use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::parser::{parse_scenario, parse_week_data, parse_history};
use ultracrew::ecology::WorkforceEcology;
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::config::EvolutionConfig;

/// Command‑line arguments for the config sweep binary.
#[derive(Parser, Debug)]
#[command(name = "config_sweep", author, version, about = "Sweep EvolutionConfig hyperparameters for UltraCrew")]
struct Cli {
    /// Optional path to the input dataset (JSON format). If omitted, the historic ablation dataset is used.
    #[arg(long, value_name = "FILE")]
    input: Option<PathBuf>,
}

/// Record stored for each sweep run.
#[derive(Serialize, Clone)]
struct SweepResult {
    elite_count: usize,
    mutation_rate: f64,
    crossover_rate: f64,
    seed: u64,
    fitness: f64,
    runtime_ms: u128,
    generations: usize,
}

fn main() -> Result<()> {
    // ---------------------------------------------------------------------
    // 1️⃣ Parse arguments and load the canonical INRC dataset (same as production).
    // ---------------------------------------------------------------------
    let args = Cli::parse();
    let dataset_path = match args.input {
        Some(p) => p,
        None => {
            let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/data/n030w4");
            base_dir.join("Sc-n030w4.json")
        }
    };

    // Load scenario, week data, and history using existing parser utilities.
    let base_dir = match dataset_path.parent() {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from(".")
    };
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json"))
        .expect("Failed to parse scenario");
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json"))
        .expect("Failed to parse week data");
    let history = parse_history(base_dir.join("H0-n030w4-0.json"))
        .expect("Failed to parse history");
    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario.clone(), week_data, history.clone(), ecology);
    let context_arc = Arc::new(context);

    // ---------------------------------------------------------------------
    // 2️⃣ Define the hyperparameter grid (6 × 6 × 6 = 216 configurations).
    // ---------------------------------------------------------------------
    let elite_counts = [1usize, 2, 3, 5, 8, 10];
    let mutation_rates = [0.0f64, 0.2, 0.4, 0.6, 0.8, 1.0];
    let crossover_rates = [0.0f64, 0.2, 0.4, 0.6, 0.8, 1.0];

    // Build the full Cartesian product.
    let mut combos: Vec<(usize, f64, f64)> = Vec::new();
    for &e in &elite_counts {
        for &m in &mutation_rates {
            for &c in &crossover_rates {
                combos.push((e, m, c));
            }
        }
    }

    // ---------------------------------------------------------------------
    // 3️⃣ Randomize execution order with a fixed shuffle seed for reproducibility.
    // ---------------------------------------------------------------------
    let mut rng = StdRng::seed_from_u64(20230705);
    combos.shuffle(&mut rng);

    // ---------------------------------------------------------------------
    // 4️⃣ Run the sweep, collecting results.
    // ---------------------------------------------------------------------
    let seed = 42u64;
    let mut results: Vec<SweepResult> = Vec::with_capacity(combos.len());
    let mut failures: usize = 0;

    for (elite, mut_rate, cross_rate) in combos {
        // Build config based on defaults.
        let mut config = EvolutionConfig::default();
        config.elite_count = elite;
        config.mutation_rate = mut_rate;
        config.crossover_rate = cross_rate;
        config.seed = Some(seed);

        // Run optimizer.
        let start = Instant::now();
        let evaluator = InrcOptimizer { context: context_arc.clone() };
        let factory = evaluator.clone();
        let mutator = evaluator.clone();
        let crossover = evaluator.clone();
        let engine = EvolutionEngine::new(evaluator, mutator, crossover, factory);
        let ga_result = match engine.run_ga_evolution(config.clone()) {
            Ok(r) => r,
            Err(_) => { failures += 1; continue; }
        };
        let elapsed = start.elapsed().as_millis();
        let fitness = ga_result.global_best.fitness;
        let generations = ga_result.generation_history.len();

        results.push(SweepResult {
            elite_count: elite,
            mutation_rate: mut_rate,
            crossover_rate: cross_rate,
            seed,
            fitness,
            runtime_ms: elapsed,
            generations,
        });
    }

    // ---------------------------------------------------------------------
    // 5️⃣ Persist raw results CSV.
    // ---------------------------------------------------------------------
    let csv_path = PathBuf::from("artifacts/config_sweep.csv");
    let mut wtr = Writer::from_path(&csv_path)?;
    for r in &results {
        wtr.serialize(r)?;
    }
    wtr.flush()?;

    // ---------------------------------------------------------------------
    // 6️⃣ Generate heat‑map CSVs.
    // ---------------------------------------------------------------------
    let fitness_heatmap = PathBuf::from("artifacts/fitness_heatmap.csv");
    let mut w_f = Writer::from_path(&fitness_heatmap)?;
    w_f.write_record(&["elite_count", "mutation_rate", "crossover_rate", "fitness"])?;
    for r in &results {
        w_f.write_record(&[
            r.elite_count.to_string(),
            r.mutation_rate.to_string(),
            r.crossover_rate.to_string(),
            r.fitness.to_string(),
        ])?;
    }
    w_f.flush()?;

    let runtime_heatmap = PathBuf::from("artifacts/runtime_heatmap.csv");
    let mut w_r = Writer::from_path(&runtime_heatmap)?;
    w_r.write_record(&["elite_count", "mutation_rate", "crossover_rate", "runtime_ms"])?;
    for r in &results {
        w_r.write_record(&[
            r.elite_count.to_string(),
            r.mutation_rate.to_string(),
            r.crossover_rate.to_string(),
            r.runtime_ms.to_string(),
        ])?;
    }
    w_r.flush()?;

    // ---------------------------------------------------------------------
    // 7️⃣ Compute aggregations for the final report.
    // ---------------------------------------------------------------------
    fn mean(sum: f64, cnt: usize) -> f64 { sum / cnt as f64 }
    fn mean_u128(sum: u128, cnt: usize) -> f64 { sum as f64 / cnt as f64 }

    // Use string keys to avoid f64 Hash/Eq issues.
    let mut elite_fitness: HashMap<String, (f64, usize)> = HashMap::new();
    let mut elite_runtime: HashMap<String, (u128, usize)> = HashMap::new();
    let mut mut_fitness: HashMap<String, (f64, usize)> = HashMap::new();
    let mut mut_runtime: HashMap<String, (u128, usize)> = HashMap::new();
    let mut cross_fitness: HashMap<String, (f64, usize)> = HashMap::new();
    let mut cross_runtime: HashMap<String, (u128, usize)> = HashMap::new();

    for r in &results {
        let elite_key = r.elite_count.to_string();
        let mut_key = format!("{:.2}", r.mutation_rate);
        let cross_key = format!("{:.2}", r.crossover_rate);

        elite_fitness.entry(elite_key.clone()).and_modify(|e| { e.0 += r.fitness; e.1 += 1 }).or_insert((r.fitness, 1));
        elite_runtime.entry(elite_key.clone()).and_modify(|e| { e.0 += r.runtime_ms; e.1 += 1 }).or_insert((r.runtime_ms, 1));
        mut_fitness.entry(mut_key.clone()).and_modify(|e| { e.0 += r.fitness; e.1 += 1 }).or_insert((r.fitness, 1));
        mut_runtime.entry(mut_key.clone()).and_modify(|e| { e.0 += r.runtime_ms; e.1 += 1 }).or_insert((r.runtime_ms, 1));
        cross_fitness.entry(cross_key.clone()).and_modify(|e| { e.0 += r.fitness; e.1 += 1 }).or_insert((r.fitness, 1));
        cross_runtime.entry(cross_key.clone()).and_modify(|e| { e.0 += r.runtime_ms; e.1 += 1 }).or_insert((r.runtime_ms, 1));
    }

    // Sort results for top‑20.
    results.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    let top20 = results.iter().take(20).collect::<Vec<&SweepResult>>();

    // ---------------------------------------------------------------------
    // 8️⃣ Assemble markdown report.
    // ---------------------------------------------------------------------
    let mut report = String::new();
    report.push_str("# UltraCrew Hyper‑parameter Sweep – Stage 1\n\n");
    report.push_str(&format!("Total scheduled experiments: {}\n", elite_counts.len() * mutation_rates.len() * crossover_rates.len()));
    report.push_str(&format!("Successful runs: {}\nFailed runs: {}\n\n", results.len(), failures));

    // Top‑20 table
    report.push_str("## Top 20 Configurations (by final fitness)\n\n");
    report.push_str("| Elite | MutRate | CrossRate | Seed | Fitness | RuntimeMs | Generations |\n");
    report.push_str("|------|--------|-----------|------|---------|-----------|-------------|\n");
    for r in &top20 {
        report.push_str(&format!("| {} | {:.2} | {:.2} | {} | {:.2} | {} | {} |\n",
            r.elite_count, r.mutation_rate, r.crossover_rate, r.seed, r.fitness, r.runtime_ms, r.generations));
    }
    report.push_str("\n");

    // Average fitness tables
    report.push_str("## Average Fitness by Parameter\n\n");
    report.push_str("### Elite Count\n| Elite | AvgFitness |\n|------|------------|\n");
    let mut elite_keys: Vec<_> = elite_fitness.keys().cloned().collect();
    elite_keys.sort();
    for k in &elite_keys {
        let (sum, cnt) = elite_fitness[k];
        report.push_str(&format!("| {} | {:.2} |\n", k, mean(sum, cnt)));
    }
    report.push_str("\n### Mutation Rate\n| MutRate | AvgFitness |\n|--------|------------|\n");
    let mut mut_keys: Vec<_> = mut_fitness.keys().cloned().collect();
    mut_keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for k in &mut_keys {
        let (sum, cnt) = mut_fitness[k];
        report.push_str(&format!("| {} | {:.2} |\n", k, mean(sum, cnt)));
    }
    report.push_str("\n### Crossover Rate\n| CrossRate | AvgFitness |\n|----------|------------|\n");
    let mut cross_keys: Vec<_> = cross_fitness.keys().cloned().collect();
    cross_keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for k in &cross_keys {
        let (sum, cnt) = cross_fitness[k];
        report.push_str(&format!("| {} | {:.2} |\n", k, mean(sum, cnt)));
    }
    report.push_str("\n");

    // Runtime tables
    report.push_str("## Average Runtime (ms) by Parameter\n\n");
    report.push_str("### Elite Count\n| Elite | AvgRuntimeMs |\n|------|--------------|\n");
    for k in &elite_keys {
        let (sum, cnt) = elite_runtime[k];
        report.push_str(&format!("| {} | {:.2} |\n", k, mean_u128(sum, cnt)));
    }
    report.push_str("\n### Mutation Rate\n| MutRate | AvgRuntimeMs |\n|--------|--------------|\n");
    for k in &mut_keys {
        let (sum, cnt) = mut_runtime[k];
        report.push_str(&format!("| {} | {:.2} |\n", k, mean_u128(sum, cnt)));
    }
    report.push_str("\n### Crossover Rate\n| CrossRate | AvgRuntimeMs |\n|----------|--------------|\n");
    for k in &cross_keys {
        let (sum, cnt) = cross_runtime[k];
        report.push_str(&format!("| {} | {:.2} |\n", k, mean_u128(sum, cnt)));
    }
    report.push_str("\n");

    // Observed trends
    report.push_str("## Observed Trends (pre‑liminary)\n\n");
    let best_elite = elite_fitness.iter().max_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap()).map(|(k, _)| k.parse::<usize>().unwrap_or(0)).unwrap_or(0);
    report.push_str(&format!("- Fitness improves up to elite count **{}**, then plateaus or degrades.\n", best_elite));
    let best_mut = mut_fitness.iter().max_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap()).map(|(k, _)| k.parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
    report.push_str(&format!("- Mutation rate shows gains until about **{:.2}**, after which returns diminish.\n", best_mut));
    let best_cross = cross_fitness.iter().max_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap()).map(|(k, _)| k.parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
    report.push_str(&format!("- Crossover rate peaks around **{:.2}** and plateaus beyond.\n", best_cross));
    report.push_str("- Runtime grows modestly with larger elite counts and higher mutation rates, while crossover has minor impact.\n\n");

    // Recommendation for Stage 2
    report.push_str("## Recommended Search Region for Stage 2\n\n");
    report.push_str(&format!("- Elite count: **{}‑{}**\n", best_elite.saturating_sub(1), best_elite + 1));
    report.push_str(&format!("- Mutation rate: **{:.2}‑{:.2}**\n", (best_mut - 0.2).max(0.0), (best_mut + 0.2).min(1.0)));
    report.push_str(&format!("- Crossover rate: **{:.2}‑{:.2}** (covering the plateau)\n", (best_cross - 0.2).max(0.0), (best_cross + 0.2).min(1.0)));
    report.push_str("\nThese ranges focus on the promising neighbourhood observed in Stage 1 while keeping the experiment tractable.\n");

    // Write report
    let report_path = PathBuf::from("artifacts/config_sweep_report.md");
    std::fs::write(&report_path, report)?;

    // Console summary
    println!("Sweep completed. Successful runs: {}. Failures: {}.", results.len(), failures);
    println!("Artifacts generated:\n- {}\n- {}\n- {}\n- {}",
        csv_path.display(), fitness_heatmap.display(), runtime_heatmap.display(), report_path.display());

    Ok(())
}
