/// phase3_rayon_ab.rs — Phase 3 Rayon A/B Invariant + Performance Harness
///
/// Governance protocol:
///   Arm A (L1 baseline):  run_pipeline_evolution_v2(..., use_rayon=false)
///   Arm B (Rayon):        run_pipeline_evolution_v2(..., use_rayon=true)
///
/// Both arms receive the IDENTICAL initial population (same seed, same factory).
/// The RNG sequence inside the evolution loop is also seeded identically.
///
/// Invariants checked:
///   1. best_obj identical between A and B (same search trajectory → same result)
///   2. n_actual_evals identical (Rayon only parallelises; does not skip evaluations)
///   3. generations_run identical (termination condition unchanged)
///
/// Performance metrics:
///   - Wall-clock runtime (ms) for each arm
///   - Speedup ratio: A_runtime / B_runtime
///   - Eval-phase time (ms) from GenerationSummary trajectory
///
/// Usage:
///   cargo run --release -p roadef --bin phase3_rayon_ab -- [--instance setA-14] [--gens 50] [--seed 42]
use std::io::BufWriter;
use std::sync::Arc;
use std::time::Instant;

use roadef::evaluator::RoadefEvaluator;
use roadef::moga_impl::{
    generate_gen0_population, ConstructionMode, EvolutionRunConfig, EvolutionRunResult,
    RoadefCrossover, RoadefFitnessEvaluator, RoadefGenomeFactory, RoadefMutator,
};
use roadef::pipeline_impl::run_pipeline_evolution_v2;
use roadef::telemetry::{ComparatorMode, NullTelemetrySink};

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const POPULATION_SIZE: usize = 50;
const ELITE_COUNT: usize = 5;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut instance_name = "setA-14".to_string();
    let mut generation_limit: usize = 50;
    let mut seed: u64 = 42;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--instance" => {
                if let Some(v) = args.next() {
                    instance_name = v;
                }
            }
            "--gens" => {
                if let Some(v) = args.next() {
                    generation_limit = v.parse().unwrap_or(50);
                }
            }
            "--seed" => {
                if let Some(v) = args.next() {
                    seed = v.parse().unwrap_or(42);
                }
            }
            _ => {}
        }
    }

    let net_path = format!("{}/{}-net.json", INSTANCE_DIR, instance_name);
    let tm_path = format!("{}/{}-tm.json", INSTANCE_DIR, instance_name);
    let scenario_path = format!("{}/{}-scenario.json", INSTANCE_DIR, instance_name);

    eprintln!("=== Phase 3 Rayon A/B Harness ===");
    eprintln!("Instance   : {}", instance_name);
    eprintln!("Generations: {}", generation_limit);
    eprintln!("Seed       : {}", seed);
    eprintln!("Pop size   : {}", POPULATION_SIZE);

    let net = roadef::loader::load_network(&net_path).expect("load network");
    let tm = roadef::loader::load_traffic_matrix(&tm_path).expect("load tm");
    let scenario = roadef::loader::load_scenario(&scenario_path).expect("load scenario");

    let num_demands = tm.demands.len();
    let num_time_slots = tm.num_time_slots;
    let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

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

    // Generate the shared initial population once.
    let init_pop = generate_gen0_population(&factory, &fitness_eval, Some(seed), POPULATION_SIZE);
    eprintln!(
        "Gen-0 IFR  : {:.4}",
        init_pop.genomes.len() as f64 / POPULATION_SIZE as f64
    );

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

    let config = EvolutionRunConfig {
        population_size: POPULATION_SIZE,
        elite_count: ELITE_COUNT,
        generation_limit,
        mutation_rate: 0.3,
        crossover_rate: 0.7,
        no_improvement_limit: generation_limit + 1, // disable stagnation termination for fair comparison
        seed: Some(seed),
        log_interval: generation_limit + 1, // suppress per-gen logs
        health_interval: generation_limit + 1,
        max_runtime: None,
        comparator_mode: ComparatorMode::Scalar,
        peak_demand_set: None,
    };

    // -----------------------------------------------------------------------
    // Arm A: L1 sequential baseline (use_rayon=false)
    // -----------------------------------------------------------------------
    eprintln!("\n--- Arm A: L1 Sequential (use_rayon=false) ---");
    let mut log_a: Vec<u8> = Vec::new();
    let t_a_start = Instant::now();
    let result_a = run_pipeline_evolution_v2(
        &factory,
        &fitness_eval,
        &mutator,
        &crossover,
        &pipeline,
        &config,
        init_pop.clone(),
        &instance_name,
        &mut log_a,
        &mut NullTelemetrySink,
        false, // L1 sequential baseline
    );
    let t_a_ms = t_a_start.elapsed().as_millis();

    let eval_ms_a: f64 = result_a
        .trajectory
        .iter()
        .map(|g| g.evaluation_runtime_ms)
        .sum();
    let n_eval_a: usize = result_a.trajectory.iter().map(|g| g.n_eval).sum();
    let cache_hits_a: usize = result_a.trajectory.iter().map(|g| g.cache_hits).sum();

    eprintln!("  best_obj         : {:.10}", result_a.best_obj);
    eprintln!("  valid            : {}", result_a.valid);
    eprintln!("  generations_run  : {}", result_a.generations_run);
    eprintln!("  n_actual_evals   : {}", n_eval_a);
    eprintln!("  cache_hits       : {}", cache_hits_a);
    eprintln!("  eval_time_ms     : {:.2}", eval_ms_a);
    eprintln!("  wall_clock_ms    : {}", t_a_ms);

    // -----------------------------------------------------------------------
    // Arm B: Rayon parallel (use_rayon=true)
    // -----------------------------------------------------------------------
    eprintln!("\n--- Arm B: Rayon Parallel (use_rayon=true) ---");
    let mut log_b: Vec<u8> = Vec::new();
    let t_b_start = Instant::now();
    let result_b = run_pipeline_evolution_v2(
        &factory,
        &fitness_eval,
        &mutator,
        &crossover,
        &pipeline,
        &config,
        init_pop.clone(),
        &instance_name,
        &mut log_b,
        &mut NullTelemetrySink,
        true, // Rayon parallel evaluation
    );
    let t_b_ms = t_b_start.elapsed().as_millis();

    let eval_ms_b: f64 = result_b
        .trajectory
        .iter()
        .map(|g| g.evaluation_runtime_ms)
        .sum();
    let n_eval_b: usize = result_b.trajectory.iter().map(|g| g.n_eval).sum();
    let cache_hits_b: usize = result_b.trajectory.iter().map(|g| g.cache_hits).sum();

    eprintln!("  best_obj         : {:.10}", result_b.best_obj);
    eprintln!("  valid            : {}", result_b.valid);
    eprintln!("  generations_run  : {}", result_b.generations_run);
    eprintln!("  n_actual_evals   : {}", n_eval_b);
    eprintln!("  cache_hits       : {}", cache_hits_b);
    eprintln!("  eval_time_ms     : {:.2}", eval_ms_b);
    eprintln!("  wall_clock_ms    : {}", t_b_ms);

    // -----------------------------------------------------------------------
    // Invariant checks
    // -----------------------------------------------------------------------
    eprintln!("\n=== Invariant Verification ===");

    let inv_best_obj = (result_a.best_obj - result_b.best_obj).abs() < 1e-9;
    let inv_n_evals = n_eval_a == n_eval_b;
    let inv_gens = result_a.generations_run == result_b.generations_run;
    let inv_valid = result_a.valid == result_b.valid;
    let inv_cache = cache_hits_a == cache_hits_b;

    eprintln!(
        "  [{}] best_obj identical: A={:.10}  B={:.10}",
        if inv_best_obj { "PASS" } else { "FAIL" },
        result_a.best_obj,
        result_b.best_obj
    );
    eprintln!(
        "  [{}] n_actual_evals identical: A={}  B={}",
        if inv_n_evals { "PASS" } else { "FAIL" },
        n_eval_a,
        n_eval_b
    );
    eprintln!(
        "  [{}] generations_run identical: A={}  B={}",
        if inv_gens { "PASS" } else { "FAIL" },
        result_a.generations_run,
        result_b.generations_run
    );
    eprintln!(
        "  [{}] valid identical: A={}  B={}",
        if inv_valid { "PASS" } else { "FAIL" },
        result_a.valid,
        result_b.valid
    );
    eprintln!(
        "  [{}] cache_hits identical: A={}  B={}",
        if inv_cache { "PASS" } else { "FAIL" },
        cache_hits_a,
        cache_hits_b
    );

    // -----------------------------------------------------------------------
    // Performance summary
    // -----------------------------------------------------------------------
    eprintln!("\n=== Performance Summary ===");
    let speedup_wall = if t_b_ms > 0 {
        t_a_ms as f64 / t_b_ms as f64
    } else {
        f64::INFINITY
    };
    let speedup_eval = if eval_ms_b > 0.0 {
        eval_ms_a / eval_ms_b
    } else {
        f64::INFINITY
    };
    let eval_pct_a = if t_a_ms > 0 {
        eval_ms_a / t_a_ms as f64 * 100.0
    } else {
        0.0
    };
    let eval_pct_b = if t_b_ms > 0 {
        eval_ms_b / t_b_ms as f64 * 100.0
    } else {
        0.0
    };

    eprintln!("  Wall-clock A (L1 seq) : {}ms", t_a_ms);
    eprintln!("  Wall-clock B (Rayon)  : {}ms", t_b_ms);
    eprintln!("  Wall-clock speedup    : {:.2}x", speedup_wall);
    eprintln!(
        "  Eval time A           : {:.2}ms  ({:.1}% of wall)",
        eval_ms_a, eval_pct_a
    );
    eprintln!(
        "  Eval time B           : {:.2}ms  ({:.1}% of wall)",
        eval_ms_b, eval_pct_b
    );
    eprintln!("  Eval speedup          : {:.2}x", speedup_eval);

    let all_pass = inv_best_obj && inv_n_evals && inv_gens && inv_valid && inv_cache;
    eprintln!(
        "\n=== Overall: {} ===",
        if all_pass {
            "ALL INVARIANTS PASS"
        } else {
            "INVARIANT VIOLATION DETECTED"
        }
    );

    if !all_pass {
        std::process::exit(1);
    }
}
