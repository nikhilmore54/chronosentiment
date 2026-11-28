//! Production INRC-II Nurse Rostering Benchmark Runner and Telemetry Generator.
//!
//! Evaluates Coralys (`InrcDecisionPlugin` + `CoralysInrcEngine`) across the canonical
//! Second International Nurse Rostering Competition (INRC-II) benchmark suite
//! against published Best Known Solutions (BKS) from Ceschia et al. (2019 / Annals of OR)
//! and competition finalists (Schedule Nurse III, Römer & Mellouli).
//!
//! Emits full structured telemetry into:
//! - `data/benchmarks/inrc/canonical_inrc_bks_benchmarks.json`
//! - `data/benchmarks/inrc/canonical_inrc_bks_benchmarks.csv`

use clap::Parser;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use coralys_core::{DecisionPlugin, ReasoningEngine};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::decision_plugin::{
    CANONICAL_INRC_BKS_TABLE, CoralysInrcEngine, InrcBksEntry, InrcDecisionPlugin,
    InrcOptimizationConfig,
};
use ultracrew::inrc::history::extract_next_history;
use ultracrew::inrc::optimization::{InrcContext, InrcGenome};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[derive(Parser, Debug)]
#[command(
    name = "inrc_bks_runner",
    about = "Coralys INRC-II DecisionPlugin Benchmark Runner"
)]
struct Cli {
    /// Filter to run a specific instance code (e.g. 'n030w4_1_6-2-9-1' or 'n030w4')
    #[arg(short, long)]
    instance: Option<String>,

    /// Run quick 4-week benchmark subset (one per nurse fleet size)
    #[arg(short, long)]
    quick: bool,

    /// Run only 4-week instances (14 instances)
    #[arg(long)]
    w4: bool,

    /// Run only 8-week instances (14 instances)
    #[arg(long)]
    w8: bool,

    /// Run all 28 canonical instances
    #[arg(short, long)]
    all: bool,

    /// Population size for MOGA evolution engine
    #[arg(short, long, default_value_t = 300)]
    pop: usize,

    /// Generation limit per week
    #[arg(short, long, default_value_t = 150)]
    gen: usize,

    /// Random seed
    #[arg(short, long, default_value_t = 42)]
    seed: u64,

    /// Output directory for telemetry JSON/CSV
    #[arg(short, long)]
    out_dir: Option<PathBuf>,

    /// Run instances in parallel across available CPU cores
    #[arg(short = 'P', long)]
    parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyBenchmarkTelemetry {
    pub week_idx: usize,
    pub week_data_idx: usize,
    pub hard_violations: usize,
    pub soft_penalty: i32,
    pub solve_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceBenchmarkTelemetry {
    pub instance_code: String,
    pub scenario: String,
    pub num_nurses: usize,
    pub num_weeks: usize,
    pub history_idx: usize,
    pub week_indices: Vec<usize>,
    pub best_known_lb: i32,
    pub best_known_ub: i32,
    pub schedule_nurse_iii: Option<i32>,
    pub romer_mellouli_bks: Option<i32>,
    pub legrain_et_al_bks: Option<i32>,
    pub lit_source: String,
    pub coralys_penalty: i32,
    pub optimality_gap_pct: f64,
    pub is_feasible: bool,
    pub total_hard_violations: usize,
    pub hard_violations_by_type: std::collections::HashMap<String, usize>,
    pub weekly_breakdown: Vec<WeeklyBenchmarkTelemetry>,
    pub total_solve_time_ms: f64,
}

fn get_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

fn get_output_dir(override_path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = override_path {
        return p;
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
        .join("benchmarks")
        .join("inrc")
}

fn run_single_instance(
    entry: &InrcBksEntry,
    data_dir: &Path,
    pop: usize,
    gen: usize,
    seed: u64,
) -> Result<InstanceBenchmarkTelemetry, String> {
    let scenario_dir = data_dir.join(entry.scenario);
    let scenario_path = scenario_dir.join(format!("Sc-{}.json", entry.scenario));
    let init_history_path = scenario_dir.join(format!("H0-{}-{}.json", entry.scenario, entry.history_idx));

    let scenario = parse_scenario(scenario_path)
        .map_err(|e| format!("Failed to parse scenario for {}: {:?}", entry.instance_code, e))?;
    let mut current_history = parse_history(init_history_path)
        .map_err(|e| format!("Failed to parse initial history for {}: {:?}", entry.instance_code, e))?;

    let ecology = WorkforceEcology::new();
    let engine = CoralysInrcEngine;
    let config = InrcOptimizationConfig {
        population_size: pop,
        generation_limit: gen,
        seed: Some(seed),
    };

    let start_total = Instant::now();
    let mut weekly_telemetry = Vec::new();
    let mut total_penalty = 0i32;
    let mut all_feasible = true;
    let mut total_hard_violations = 0usize;
    let mut hard_violations_by_type: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (week_num, &wd_idx) in entry.week_indices.iter().enumerate() {
        let wd_path = scenario_dir.join(format!("WD-{}-{}.json", entry.scenario, wd_idx));
        let week_data = parse_week_data(wd_path)
            .map_err(|e| format!("Failed to parse WD-{} for {}: {:?}", wd_idx, entry.instance_code, e))?;

        let context = Arc::new(InrcContext::new(
            scenario.clone(),
            week_data,
            current_history.clone(),
            ecology.clone(),
        ));

        let mut plugin = InrcDecisionPlugin::from_context(context.clone(), week_num);

        let t0 = Instant::now();
        let proposals = engine
            .solve(&mut plugin, &config)
            .map_err(|e| format!("Solve error week {}: {}", week_num, e))?;
        let solve_ms = t0.elapsed().as_secs_f64() * 1000.0;

        if let Some(proposal) = proposals.first() {
            let next_state = plugin
                .simulate(&plugin.current_state(), proposal)
                .map_err(|e| format!("Simulate error week {}: {}", week_num, e))?;

            let eval = plugin.evaluate(&next_state);
            let hard_viols = eval.hard_constraint_violations.len();
            let soft_pen = next_state.total_soft_penalty;

            for v in &eval.hard_constraint_violations {
                eprintln!("  [HC-VIOLATION] week {}: {} - {}", week_num, v.constraint_id, v.description);
                *hard_violations_by_type.entry(v.constraint_id.clone()).or_insert(0) += 1;
            }

            if hard_viols > 0 {
                all_feasible = false;
                total_hard_violations += hard_viols;
            }
            total_penalty += soft_pen;
            println!(
                "    [METRICS] week {}: assign={:.0} streak={:.0} day_off={:.0} wend={:.0} pref={:.0} opt_cov={:.0}",
                week_num,
                eval.metrics.get("assignment_penalty").unwrap_or(&0.0),
                eval.metrics.get("work_streak_penalty").unwrap_or(&0.0),
                eval.metrics.get("day_off_penalty").unwrap_or(&0.0),
                eval.metrics.get("weekend_penalty").unwrap_or(&0.0),
                eval.metrics.get("preferences_penalty").unwrap_or(&0.0),
                eval.metrics.get("optimal_coverage_penalty").unwrap_or(&0.0),
            );

            // Extract next history for rolling horizon
            let genome = InrcGenome {
                bits: next_state.genome_bits.clone(),
            };
            current_history = extract_next_history(&context, &genome);

            weekly_telemetry.push(WeeklyBenchmarkTelemetry {
                week_idx: week_num,
                week_data_idx: wd_idx,
                hard_violations: hard_viols,
                soft_penalty: soft_pen,
                solve_time_ms: solve_ms,
            });
        } else {
            return Err(format!("Engine returned no proposal for week {}", week_num));
        }
    }

    let total_time_ms = start_total.elapsed().as_secs_f64() * 1000.0;
    let gap_pct = if entry.best_known_ub > 0 {
        ((total_penalty - entry.best_known_ub) as f64 / entry.best_known_ub as f64) * 100.0
    } else {
        0.0
    };

    Ok(InstanceBenchmarkTelemetry {
        instance_code: entry.instance_code.to_string(),
        scenario: entry.scenario.to_string(),
        num_nurses: entry.num_nurses,
        num_weeks: entry.num_weeks,
        history_idx: entry.history_idx,
        week_indices: entry.week_indices.to_vec(),
        best_known_lb: entry.best_known_lb,
        best_known_ub: entry.best_known_ub,
        schedule_nurse_iii: entry.schedule_nurse_iii(),
        romer_mellouli_bks: entry.romer_mellouli(),
        legrain_et_al_bks: entry.legrain_et_al(),
        lit_source: entry.lit_source().to_string(),
        coralys_penalty: total_penalty,
        optimality_gap_pct: gap_pct,
        is_feasible: all_feasible,
        total_hard_violations,
        hard_violations_by_type,
        weekly_breakdown: weekly_telemetry,
        total_solve_time_ms: total_time_ms,
    })
}

fn main() {
    let args = Cli::parse();
    let data_dir = get_data_dir();
    let out_dir = get_output_dir(args.out_dir);

    fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    println!("===============================================================================");
    println!("  Coralys INRC-II Nurse Rostering Production Benchmark Runner");
    println!("  Contract: coralys_core::DecisionPlugin + ReasoningEngine");
    println!("  Reference Baseline: Ceschia et al. (2019 / Annals of OR) & SN-III");
    println!("===============================================================================\n");

    let instances_to_run: Vec<&InrcBksEntry> = if let Some(ref filter) = args.instance {
        CANONICAL_INRC_BKS_TABLE
            .iter()
            .filter(|e| e.instance_code.contains(filter) || e.scenario.contains(filter))
            .collect()
    } else if args.w4 {
        CANONICAL_INRC_BKS_TABLE
            .iter()
            .filter(|e| e.num_weeks == 4)
            .collect()
    } else if args.w8 {
        CANONICAL_INRC_BKS_TABLE
            .iter()
            .filter(|e| e.num_weeks == 8)
            .collect()
    } else if args.quick {
        // One 4-week instance for each fleet size (30, 40, 50, 60, 80, 100, 120)
        vec![
            &CANONICAL_INRC_BKS_TABLE[0],  // n030w4
            &CANONICAL_INRC_BKS_TABLE[2],  // n040w4
            &CANONICAL_INRC_BKS_TABLE[4],  // n050w4
            &CANONICAL_INRC_BKS_TABLE[6],  // n060w4
            &CANONICAL_INRC_BKS_TABLE[8],  // n080w4
            &CANONICAL_INRC_BKS_TABLE[10], // n100w4
            &CANONICAL_INRC_BKS_TABLE[12], // n120w4
        ]
    } else if args.all {
        CANONICAL_INRC_BKS_TABLE.iter().collect()
    } else {
        // Default: 4-week suite (first 14 instances)
        CANONICAL_INRC_BKS_TABLE.iter().take(14).collect()
    };

    println!("Selected {} instances for evaluation (pop={}, gen={}, seed={})\n",
        instances_to_run.len(), args.pop, args.gen, args.seed);

    let results: Vec<InstanceBenchmarkTelemetry> = if args.parallel {
        println!("Running {} instances in parallel with Rayon...", instances_to_run.len());
        instances_to_run
            .par_iter()
            .filter_map(|entry| {
                println!(">> Starting instance: {} (N={}, W={})", entry.instance_code, entry.num_nurses, entry.num_weeks);
                match run_single_instance(entry, &data_dir, args.pop, args.gen, args.seed) {
                    Ok(telemetry) => {
                        let status_str = if telemetry.is_feasible {
                            "FEASIBLE (0 HC)"
                        } else {
                            "VIOLATIONS"
                        };
                        println!(
                            "<< Finished: {} | Coralys: {} | BKS: {} | Gap: {:+.2}% | {} in {:.2}s",
                            entry.instance_code,
                            telemetry.coralys_penalty,
                            entry.best_known_ub,
                            telemetry.optimality_gap_pct,
                            status_str,
                            telemetry.total_solve_time_ms / 1000.0
                        );
                        Some(telemetry)
                    }
                    Err(err) => {
                        eprintln!("Error on {}: {}", entry.instance_code, err);
                        None
                    }
                }
            })
            .collect()
    } else {
        let mut res = Vec::new();
        for (idx, entry) in instances_to_run.iter().enumerate() {
            print!("[{:02}/{:02}] Evaluating {} (N={}, W={}) ... ",
                idx + 1, instances_to_run.len(), entry.instance_code, entry.num_nurses, entry.num_weeks);
            std::io::stdout().flush().unwrap();

            match run_single_instance(entry, &data_dir, args.pop, args.gen, args.seed) {
                Ok(telemetry) => {
                    let status_str = if telemetry.is_feasible {
                        "FEASIBLE (0 HC)"
                    } else {
                        "VIOLATIONS"
                    };
                    let gap_str = if telemetry.optimality_gap_pct <= 0.0 {
                        format!("\x1b[32m{:.2}%\x1b[0m (<= BKS)", telemetry.optimality_gap_pct)
                    } else {
                        format!("{:.2}%", telemetry.optimality_gap_pct)
                    };
                    println!(
                        "Coralys: {} | BKS: {} | Gap: {} | {} in {:.2}s",
                        telemetry.coralys_penalty,
                        entry.best_known_ub,
                        gap_str,
                        status_str,
                        telemetry.total_solve_time_ms / 1000.0
                    );
                    res.push(telemetry);
                }
                Err(err) => {
                    println!("\x1b[31mFAILED: {}\x1b[0m", err);
                }
            }
        }
        res
    };

    if results.is_empty() {
        eprintln!("No successful evaluations.");
        return;
    }

    // Print Markdown Summary Table
    println!("\n===============================================================================");
    println!("  CANONICAL INRC-II BENCHMARK RESULTS (Ceschia et al. 2019 / BKS)");
    println!("===============================================================================\n");

    println!("| Instance Code | Nurses | Wks | BKS (LB) | BKS (UB) | SN-III | Coralys | Gap % | Feasible | Time (s) |");
    println!("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |");

    let mut total_gap = 0.0;
    let mut feasible_count = 0;
    let mut beat_bks_count = 0;

    for r in &results {
        let sn_str = r.schedule_nurse_iii
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string());
        let feas_str = if r.is_feasible { "Yes (0 HC)" } else { "No" };
        println!(
            "| {:<24} | {:>6} | {:>3} | {:>8} | {:>8} | {:>6} | {:>7} | {:>+6.2}% | {:>10} | {:>8.2} |",
            r.instance_code,
            r.num_nurses,
            r.num_weeks,
            r.best_known_lb,
            r.best_known_ub,
            sn_str,
            r.coralys_penalty,
            r.optimality_gap_pct,
            feas_str,
            r.total_solve_time_ms / 1000.0
        );

        total_gap += r.optimality_gap_pct;
        if r.is_feasible {
            feasible_count += 1;
        }
        if r.coralys_penalty <= r.best_known_ub {
            beat_bks_count += 1;
        }
    }

    let n = results.len() as f64;
    let mean_gap = total_gap / n;
    let feas_rate = (feasible_count as f64 / n) * 100.0;

    println!("\n-------------------------------------------------------------------------------");
    println!("  Summary: Evaluated {} instances", results.len());
    println!("  Zero-Violation Feasibility Rate : {:.1}% ({}/{})", feas_rate, feasible_count, results.len());
    println!("  Matching or Beating BKS Bound   : {} / {}", beat_bks_count, results.len());
    println!("  Mean Optimality Gap to BKS      : {:+.2}%", mean_gap);
    println!("-------------------------------------------------------------------------------\n");

    // Serialize JSON Telemetry
    let json_path = out_dir.join("canonical_inrc_bks_benchmarks.json");
    let json_file = File::create(&json_path).expect("Failed to create JSON output file");
    serde_json::to_writer_pretty(json_file, &results).expect("Failed to write JSON telemetry");
    println!("Wrote JSON telemetry: {}", json_path.display());

    let bench_results_path = out_dir.join("benchmark_results.json");
    let bench_file = File::create(&bench_results_path).expect("Failed to create benchmark_results.json");
    serde_json::to_writer_pretty(bench_file, &results).expect("Failed to write benchmark_results.json");
    println!("Wrote Benchmark results: {}", bench_results_path.display());

    // Serialize CSV Telemetry
    let csv_path = out_dir.join("canonical_inrc_bks_benchmarks.csv");
    let mut csv_file = File::create(&csv_path).expect("Failed to create CSV output file");
    writeln!(
        csv_file,
        "instance_code,scenario,num_nurses,num_weeks,history_idx,best_known_lb,best_known_ub,schedule_nurse_iii,romer_mellouli,legrain_et_al,coralys_penalty,optimality_gap_pct,is_feasible,total_hard_violations,total_solve_time_ms"
    ).unwrap();

    for r in &results {
        writeln!(
            csv_file,
            "{},{},{},{},{},{},{},{},{},{},{},{:.4},{},{},{:.2}",
            r.instance_code,
            r.scenario,
            r.num_nurses,
            r.num_weeks,
            r.history_idx,
            r.best_known_lb,
            r.best_known_ub,
            r.schedule_nurse_iii.map(|v| v.to_string()).unwrap_or_default(),
            r.romer_mellouli_bks.map(|v| v.to_string()).unwrap_or_default(),
            r.legrain_et_al_bks.map(|v| v.to_string()).unwrap_or_default(),
            r.coralys_penalty,
            r.optimality_gap_pct,
            r.is_feasible,
            r.total_hard_violations,
            r.total_solve_time_ms
        ).unwrap();
    }
    println!("Wrote CSV telemetry:  {}", csv_path.display());
}
