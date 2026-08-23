/// campaign_parallel_ab.rs — V2 Deterministic Parallel A/B Harness
///
/// Research question: Does replacing the random constructor (CB-000) with the
/// PIPELINE candidate improve objective scores?
///
/// This harness uses deterministic generation of Gen-0 populations and parallel execution
/// to ensure absolute fairness between Arm A and Arm B on matched (instance, seed) pairs.
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::Utc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use roadef::evaluator::RoadefEvaluator;
use roadef::models::Network;
use roadef::moga_impl::{
    generate_gen0_population, run_roadef_evolution_v2, ConstructionMode, EvolutionRunConfig,
    EvolutionRunResult, RoadefCrossover, RoadefFitnessEvaluator, RoadefGenomeFactory,
    RoadefMutator,
};
use roadef::pipeline_impl::run_pipeline_evolution_v2;
use roadef::telemetry::{ComparatorMode, NullTelemetrySink};

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const REPORT_DIR: &str = "benchmarks/roadef/pipeline";

const POPULATION_SIZE: usize = 50;
const GENERATION_LIMIT: usize = 500;
const ELITE_COUNT: usize = 5;
const CAMPAIGN_ID: &str = "SET-A-COVERAGE-001";

const SEEDS: [u64; 10] = [42, 43, 44, 45, 46, 47, 48, 49, 50, 51];

const MIN_BUDGET_SECS: u64 = 10;
const MAX_BUDGET_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArmResult {
    seed: u64,
    arm: String,
    instance: String,
    num_demands: usize,
    num_nodes: usize,
    num_links: usize,
    num_time_slots: usize,
    initial_feasibility_rate: f64,
    gen0_feasible_count: usize,
    gen0_best_obj: f64,
    gen0_mean_obj: f64,
    best_obj: f64,
    valid: bool,
    runtime_ms: u128,
    generations_run: usize,
    n_eval: usize,
    termination_reason: String,
    invariant_violation_suspected: bool,
    gen0_unique_obj_count: usize,
    gen0_duplicate_genome_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SeedComparison {
    seed: u64,
    instance: String,
    initial_population_hash: String,
    arm_a_ifr: f64,
    arm_b_ifr: f64,
    ifr_delta: f64,
    arm_a_gen0_best_obj: f64,
    arm_b_gen0_best_obj: f64,
    arm_a_obj: f64,
    arm_b_obj: f64,
    obj_delta: f64,
    arm_a_valid: bool,
    arm_b_valid: bool,
    arm_b_better_obj: bool,
    arm_b_better_ifr: bool,
    arm_a_invariant_violation: bool,
    arm_b_invariant_violation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AbReport {
    campaign_id: String,
    timestamp: String,
    budget_mode: String,
    hypothesis: String,
    arm_a_description: String,
    arm_b_description: String,
    statistical_note: String,
    total_instances: usize,
    arm_a_mean_ifr: f64,
    arm_b_mean_ifr: f64,
    ifr_improvement: f64,
    arm_a_valid_count: usize,
    arm_b_valid_count: usize,
    arm_b_better_obj_count: usize,
    arm_b_better_ifr_count: usize,
    invariant_violation_count_a: usize,
    invariant_violation_count_b: usize,
    comparisons: Vec<SeedComparison>,
    arm_a_results: Vec<ArmResult>,
    arm_b_results: Vec<ArmResult>,
}

fn discover_instances() -> Vec<(String, String, String, String)> {
    let mut instances = Vec::new();
    for i in 1..=20 {
        let name = format!("setA-{:02}", i);
        let net = format!("{}/{}-net.json", INSTANCE_DIR, name);
        let tm = format!("{}/{}-tm.json", INSTANCE_DIR, name);
        let scenario = format!("{}/{}-scenario.json", INSTANCE_DIR, name);
        if Path::new(&net).exists() && Path::new(&tm).exists() && Path::new(&scenario).exists() {
            instances.push((name, net, tm, scenario));
        }
    }
    instances
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BudgetMode {
    Evaluations,
    WallClock,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArmType {
    Legacy,
    Pipeline,
}

struct RunContext {
    instance_name: String,
    seed: u64,
    net_path: String,
    tm_path: String,
    scenario_path: String,
    budget_mode: BudgetMode,
    arm_type: ArmType,
}

fn run_single_arm(ctx: &RunContext) -> Option<(ArmResult, u64)> {
    let net = roadef::loader::load_network(&ctx.net_path).ok()?;
    let tm = roadef::loader::load_traffic_matrix(&ctx.tm_path).ok()?;
    let scenario = roadef::loader::load_scenario(&ctx.scenario_path).ok()?;

    let num_demands = tm.demands.len();
    let num_time_slots = tm.num_time_slots;
    let num_nodes = net.nodes.len();
    let num_links = net.links.len();
    let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

    let raw_budget_ms = (num_demands as u64) * (num_links as u64) / 2;
    let budget_secs = raw_budget_ms.clamp(MIN_BUDGET_SECS * 1000, MAX_BUDGET_SECS * 1000) / 1000;

    let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
    let fitness_eval = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
    };
    let mutator = RoadefMutator {
        node_ids: node_ids.clone(),
    };
    let crossover = RoadefCrossover;

    let factory = RoadefGenomeFactory {
        num_demands,
        num_time_slots,
        node_ids: node_ids.clone(),
        mode: ConstructionMode::Random,
        greedy_data: None,
    };

    // 1. Generate Deterministic Initial Population exactly once
    let init_pop =
        generate_gen0_population(&factory, &fitness_eval, Some(ctx.seed), POPULATION_SIZE);
    let pop_hash = init_pop.hash;

    let max_runtime = if ctx.budget_mode == BudgetMode::WallClock {
        Some(std::time::Duration::from_secs(budget_secs))
    } else {
        None
    };

    let mut evo_config = EvolutionRunConfig {
        population_size: POPULATION_SIZE,
        elite_count: ELITE_COUNT,
        generation_limit: GENERATION_LIMIT,
        mutation_rate: 0.3,
        crossover_rate: 0.7,
        no_improvement_limit: 20,
        seed: Some(ctx.seed),
        log_interval: 100, // Reduced logging for parallel
        health_interval: 200,
        max_runtime,
        comparator_mode: ComparatorMode::Scalar,
        peak_demand_set: None,
    };

    let arm_result = match ctx.arm_type {
        ArmType::Legacy => {
            let mut log_buf_a = Vec::new();
            let result_a = run_roadef_evolution_v2(
                &factory,
                &fitness_eval,
                &mutator,
                &crossover,
                &evo_config,
                init_pop.clone(),
                &ctx.instance_name,
                &mut log_buf_a,
                &mut NullTelemetrySink,
            );
            let n_eval_a = POPULATION_SIZE
                + result_a.generations_run * (POPULATION_SIZE.saturating_sub(ELITE_COUNT));
            ArmResult {
                arm: "A_Legacy".to_string(),
                seed: ctx.seed,
                instance: ctx.instance_name.clone(),
                num_demands,
                num_nodes,
                num_links,
                num_time_slots,
                initial_feasibility_rate: result_a.initial_feasibility_rate,
                gen0_feasible_count: result_a.gen0_feasible_count,
                gen0_best_obj: result_a.gen0_best_obj,
                gen0_mean_obj: result_a.gen0_mean_obj,
                best_obj: result_a.best_obj,
                valid: result_a.valid,
                runtime_ms: result_a.runtime_ms,
                generations_run: result_a.generations_run,
                n_eval: n_eval_a,
                termination_reason: result_a.termination_reason,
                invariant_violation_suspected: result_a.initial_feasibility_rate >= 1.0
                    && !result_a.valid
                    && !result_a.best_obj.is_finite(),
                gen0_unique_obj_count: result_a.gen0_unique_obj_count,
                gen0_duplicate_genome_count: result_a.gen0_duplicate_genome_count,
            }
        }
        ArmType::Pipeline => {
            let mut log_buf_b = Vec::new();
            let pipeline = coralys_core::pipeline::EvolutionaryPipeline {
                constraint_model: roadef::constraints::RoadefConstraintModel {
                    evaluator: evaluator.clone(),
                },
                repair_operators: vec![Box::new(roadef::operators::RoadefRepair)],
                improvement_operators: vec![Box::new(roadef::operators::RoadefImprovement)],
                repair_budget: coralys_core::operators::OperatorBudget {
                    max_iterations: 10,
                    max_time_ms: 100,
                },
                improve_budget: coralys_core::operators::OperatorBudget {
                    max_iterations: 10,
                    max_time_ms: 100,
                },
            };
            let result_b = run_pipeline_evolution_v2(
                &factory,
                &fitness_eval,
                &mutator,
                &crossover,
                &pipeline,
                &evo_config,
                init_pop.clone(),
                &ctx.instance_name,
                &mut log_buf_b,
                &mut NullTelemetrySink,
                true, // Phase 3: Rayon parallel evaluation enabled
            );
            let n_eval_b = POPULATION_SIZE
                + result_b.generations_run * (POPULATION_SIZE.saturating_sub(ELITE_COUNT));
            ArmResult {
                arm: "B_Pipeline".to_string(),
                seed: ctx.seed,
                instance: ctx.instance_name.clone(),
                num_demands,
                num_nodes,
                num_links,
                num_time_slots,
                initial_feasibility_rate: result_b.initial_feasibility_rate,
                gen0_feasible_count: result_b.gen0_feasible_count,
                gen0_best_obj: result_b.gen0_best_obj,
                gen0_mean_obj: result_b.gen0_mean_obj,
                best_obj: result_b.best_obj,
                valid: result_b.valid,
                runtime_ms: result_b.runtime_ms,
                generations_run: result_b.generations_run,
                n_eval: n_eval_b,
                termination_reason: result_b.termination_reason,
                invariant_violation_suspected: result_b.initial_feasibility_rate >= 1.0
                    && !result_b.valid
                    && !result_b.best_obj.is_finite(),
                gen0_unique_obj_count: result_b.gen0_unique_obj_count,
                gen0_duplicate_genome_count: result_b.gen0_duplicate_genome_count,
            }
        }
    };

    Some((arm_result, pop_hash))
}

fn main() {
    let mut args = env::args().skip(1);
    let mut workers = 4;
    let mut budget_mode = BudgetMode::Evaluations;
    let mut instance_limit = 20;
    let mut seed_limit = 1;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workers" => {
                if let Some(w) = args.next() {
                    workers = w.parse().unwrap_or(4);
                }
            }
            "--budget" => {
                if let Some(b) = args.next() {
                    if b == "wall-clock" {
                        budget_mode = BudgetMode::WallClock;
                    }
                }
            }
            "--quick" => {
                instance_limit = 2;
                seed_limit = 1;
            }
            _ => {}
        }
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build_global()
        .unwrap();

    let b_str = if budget_mode == BudgetMode::WallClock {
        "wall-clock"
    } else {
        "evaluations"
    };
    eprintln!("=== ROADEF Parallel Deterministic Harness ===");
    eprintln!("Workers: {}", workers);
    eprintln!("Budget Mode: {}", b_str);

    let all_instances = discover_instances();
    let instances: Vec<_> = all_instances.into_iter().take(instance_limit).collect();
    let seeds = &SEEDS[..seed_limit];

    let mut tasks = Vec::new();
    for (name, net, tm, scenario) in &instances {
        for &seed in seeds {
            tasks.push(RunContext {
                instance_name: name.clone(),
                seed,
                net_path: net.clone(),
                tm_path: tm.clone(),
                scenario_path: scenario.clone(),
                budget_mode,
                arm_type: ArmType::Legacy,
            });
            tasks.push(RunContext {
                instance_name: name.clone(),
                seed,
                net_path: net.clone(),
                tm_path: tm.clone(),
                scenario_path: scenario.clone(),
                budget_mode,
                arm_type: ArmType::Pipeline,
            });
        }
    }

    // Deterministically shuffle the unified queue by hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    tasks.sort_by_key(|ctx| {
        let mut hasher = DefaultHasher::new();
        ctx.instance_name.hash(&mut hasher);
        ctx.seed.hash(&mut hasher);
        (ctx.arm_type == ArmType::Legacy).hash(&mut hasher);
        hasher.finish()
    });

    let total_tasks = tasks.len();
    eprintln!(
        "Running {} jobs ({} instances x {} seeds) ...",
        total_tasks,
        instances.len(),
        seeds.len()
    );

    let completed_tasks = Arc::new(std::sync::Mutex::new(0));
    let _ = std::fs::create_dir_all(REPORT_DIR);
    let jsonl_path = format!(
        "{}/pipeline_ab_report_parallel_{}_partial.jsonl",
        REPORT_DIR, b_str
    );
    let jsonl_file = Arc::new(std::sync::Mutex::new(
        std::fs::File::create(&jsonl_path).unwrap(),
    ));
    eprintln!("Streaming partial results to: {}", jsonl_path);

    let results: Vec<(ArmResult, u64)> = tasks
        .into_par_iter()
        .filter_map(|ctx| {
            let arm_str = if ctx.arm_type == ArmType::Legacy {
                "Legacy"
            } else {
                "Pipeline"
            };
            let res = run_single_arm(&ctx);
            if let Some((ref arm_res, _hash)) = res {
                if let Ok(json_line) = serde_json::to_string(arm_res) {
                    let mut file_lock = jsonl_file.lock().unwrap();
                    use std::io::Write;
                    let _ = writeln!(file_lock, "{}", json_line);
                }
            }
            let mut count = completed_tasks.lock().unwrap();
            *count += 1;
            eprintln!("[{}] {}/{} jobs complete", arm_str, *count, total_tasks);
            res
        })
        .collect();

    let mut grouped_results: HashMap<(String, u64), (Option<ArmResult>, Option<ArmResult>, u64)> =
        HashMap::new();
    for (arm_res, hash) in results {
        let key = (arm_res.instance.clone(), arm_res.seed);
        let entry = grouped_results.entry(key).or_insert((None, None, hash));
        if arm_res.arm == "A_Legacy" {
            entry.0 = Some(arm_res);
        } else {
            entry.1 = Some(arm_res);
        }
    }

    let mut paired_results: Vec<(ArmResult, ArmResult, u64)> = grouped_results
        .into_values()
        .filter_map(|(a_opt, b_opt, hash)| match (a_opt, b_opt) {
            (Some(a), Some(b)) => Some((a, b, hash)),
            _ => None,
        })
        .collect();

    paired_results.sort_by(|a, b| {
        a.0.instance
            .cmp(&b.0.instance)
            .then(a.0.seed.cmp(&b.0.seed))
    });

    let mut comparisons = Vec::new();
    let mut arm_a_results = Vec::new();
    let mut arm_b_results = Vec::new();
    let mut arm_b_better_obj_count = 0;
    let mut arm_b_better_ifr_count = 0;

    let mut invalid_hashes = 0;

    for (a, b, hash) in paired_results {
        if format!("{:.4}", a.initial_feasibility_rate)
            != format!("{:.4}", b.initial_feasibility_rate)
        {
            eprintln!(
                "ERROR: Mismatched IFR on {} seed {}: A={} B={}",
                a.instance, a.seed, a.initial_feasibility_rate, b.initial_feasibility_rate
            );
            invalid_hashes += 1;
        }

        let arm_b_better_obj = b.best_obj < a.best_obj;
        let arm_b_better_ifr = b.initial_feasibility_rate > a.initial_feasibility_rate;
        if arm_b_better_obj {
            arm_b_better_obj_count += 1;
        }
        if arm_b_better_ifr {
            arm_b_better_ifr_count += 1;
        }

        comparisons.push(SeedComparison {
            seed: a.seed,
            instance: a.instance.clone(),
            initial_population_hash: format!("{:016x}", hash),
            arm_a_ifr: a.initial_feasibility_rate,
            arm_b_ifr: b.initial_feasibility_rate,
            ifr_delta: b.initial_feasibility_rate - a.initial_feasibility_rate,
            arm_a_gen0_best_obj: a.gen0_best_obj,
            arm_b_gen0_best_obj: b.gen0_best_obj,
            arm_a_obj: a.best_obj,
            arm_b_obj: b.best_obj,
            obj_delta: b.best_obj - a.best_obj,
            arm_a_valid: a.valid,
            arm_b_valid: b.valid,
            arm_b_better_obj,
            arm_b_better_ifr,
            arm_a_invariant_violation: a.invariant_violation_suspected,
            arm_b_invariant_violation: b.invariant_violation_suspected,
        });

        arm_a_results.push(a);
        arm_b_results.push(b);
    }

    if invalid_hashes > 0 {
        eprintln!(
            "FATAL: {} runs had mismatched Gen-0 metrics despite shared population!",
            invalid_hashes
        );
        std::process::exit(1);
    }

    let n = comparisons.len();
    let arm_a_mean_ifr = if n > 0 {
        arm_a_results
            .iter()
            .map(|r| r.initial_feasibility_rate)
            .sum::<f64>()
            / n as f64
    } else {
        0.0
    };
    let arm_b_mean_ifr = if n > 0 {
        arm_b_results
            .iter()
            .map(|r| r.initial_feasibility_rate)
            .sum::<f64>()
            / n as f64
    } else {
        0.0
    };
    let ifr_improvement = arm_b_mean_ifr - arm_a_mean_ifr;
    let arm_a_valid_count = arm_a_results.iter().filter(|r| r.valid).count();
    let arm_b_valid_count = arm_b_results.iter().filter(|r| r.valid).count();
    let invariant_violation_count_a = arm_a_results
        .iter()
        .filter(|r| r.invariant_violation_suspected)
        .count();
    let invariant_violation_count_b = arm_b_results
        .iter()
        .filter(|r| r.invariant_violation_suspected)
        .count();

    eprintln!("=== PIPELINE Summary ===");
    eprintln!("Runs: {}", n);
    eprintln!("Arm B better obj: {}/{}", arm_b_better_obj_count, n);

    let report = AbReport {
        campaign_id: CAMPAIGN_ID.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        budget_mode: b_str.to_string(),
        hypothesis: "V2 Harness".to_string(),
        arm_a_description: "Legacy".to_string(),
        arm_b_description: "Pipeline".to_string(),
        statistical_note: "Parallel Deterministic".to_string(),
        total_instances: n,
        arm_a_mean_ifr,
        arm_b_mean_ifr,
        ifr_improvement,
        arm_a_valid_count,
        arm_b_valid_count,
        arm_b_better_obj_count,
        arm_b_better_ifr_count,
        invariant_violation_count_a,
        invariant_violation_count_b,
        comparisons,
        arm_a_results,
        arm_b_results,
    };

    let _ = fs::create_dir_all(REPORT_DIR);
    let json_path = format!("{}/pipeline_ab_report_parallel_{}.json", REPORT_DIR, b_str);
    if let Ok(f) = fs::File::create(&json_path) {
        let _ = serde_json::to_writer_pretty(BufWriter::new(f), &report);
        eprintln!("JSON report: {}", json_path);
    }
}
