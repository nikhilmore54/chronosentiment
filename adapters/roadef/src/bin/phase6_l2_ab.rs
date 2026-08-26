/// phase6_l2_ab.rs — Phase 6 L2 Cross-Evaluation Dijkstra Cache A/B Harness
///
/// Governance protocol:
///   Arm A (Phase 4 baseline): run_pipeline_evolution_v2(..., use_rayon=true, l2_cache=None)
///   Arm B (Phase 4 + L2):     run_pipeline_evolution_v2(..., use_rayon=true, l2_cache=Some(...))
///
/// Both arms receive the IDENTICAL initial population (same seed, same factory).
/// The RNG sequence inside the evolution loop is also seeded identically.
///
/// Invariants checked (all 5 must be identical):
///   1. best_obj
///   2. n_actual_evals
///   3. generations_run
///   4. valid
///   5. cache_hits (L1 genome cache)
///
/// Performance metrics:
///   - Wall-clock runtime (ms) for each arm
///   - Eval-phase time (ms) from GenerationSummary trajectory
///   - T_net = eval_time_A - eval_time_B (must be > 0 for promotion)
///   - L2 cache entries (unique (target_node, time_slot) pairs computed)
///
/// Phase 4 baseline (authoritative):
///   instance      : setA-14
///   seed          : 42
///   best_obj      : 86.1250850504
///   n_actual_evals: 2006
///   generations   : 50
///   valid         : true
///   cache_hits    : 181
///   eval_time_ms  : 185,159
///   wall_clock_ms : 1,739,745
///   commit        : 4a691cdd2
///
/// Usage:
///   cargo run --release -p roadef --bin phase6_l2_ab -- [--instance setA-01] [--gens 50] [--seed 42]
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use roadef::evaluator::{L2DijkstraCache, RoadefEvaluator};
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
    let mut instance_name = "setA-01".to_string();
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

    eprintln!("=== Phase 6 L2 Dijkstra Cache A/B Harness ===");
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

    // Arm A: Phase 4 baseline — no L2 cache.
    let fitness_eval_a = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
        l2_cache: None,
    };

    let factory = RoadefGenomeFactory {
        num_demands,
        num_time_slots,
        node_ids: node_ids.clone(),
        mode: ConstructionMode::Random,
        greedy_data: None,
    };

    // Generate the shared initial population once (same seed → same genomes for both arms).
    let init_pop = generate_gen0_population(&factory, &fitness_eval_a, Some(seed), POPULATION_SIZE);
    eprintln!("Gen-0 pop hash : {}", init_pop.hash);

    let mutator = RoadefMutator {
        node_ids: node_ids.clone(),
    };
    let crossover = RoadefCrossover;

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
        no_improvement_limit: generation_limit + 1, // disable stagnation for fair comparison
        seed: Some(seed),
        log_interval: generation_limit + 1, // suppress per-gen logs
        health_interval: generation_limit + 1,
        max_runtime: None,
        comparator_mode: ComparatorMode::Scalar,
        peak_demand_set: None,
    };

    // -----------------------------------------------------------------------
    // Arm A: Phase 4 baseline (L1 + Rayon, no L2)
    // -----------------------------------------------------------------------
    eprintln!("\n--- Arm A: Phase 4 Baseline (use_rayon=true, l2_cache=None) ---");
    let mut log_a: Vec<u8> = Vec::new();
    let t_a_start = Instant::now();
    let result_a: EvolutionRunResult = run_pipeline_evolution_v2(
        &factory,
        &fitness_eval_a,
        &mutator,
        &crossover,
        &pipeline,
        &config,
        init_pop.clone(),
        &instance_name,
        &mut log_a,
        &mut NullTelemetrySink,
        true, // Rayon parallel evaluation (Phase 4 baseline)
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
    // Arm B: Phase 4 + L2 cross-evaluation Dijkstra cache
    // -----------------------------------------------------------------------
    eprintln!("\n--- Arm B: Phase 4 + L2 Dijkstra Cache (use_rayon=true, l2_cache=Some) ---");

    let l2_cache_b: L2DijkstraCache = Arc::new(RwLock::new(HashMap::new()));
    let fitness_eval_b = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
        l2_cache: Some(Arc::clone(&l2_cache_b)),
    };

    let mut log_b: Vec<u8> = Vec::new();
    let t_b_start = Instant::now();
    let result_b: EvolutionRunResult = run_pipeline_evolution_v2(
        &factory,
        &fitness_eval_b,
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
    let l2_entries = l2_cache_b.read().unwrap().len();

    eprintln!("  best_obj         : {:.10}", result_b.best_obj);
    eprintln!("  valid            : {}", result_b.valid);
    eprintln!("  generations_run  : {}", result_b.generations_run);
    eprintln!("  n_actual_evals   : {}", n_eval_b);
    eprintln!("  cache_hits       : {}", cache_hits_b);
    eprintln!("  eval_time_ms     : {:.2}", eval_ms_b);
    eprintln!("  wall_clock_ms    : {}", t_b_ms);
    eprintln!("  l2_cache_entries : {}", l2_entries);

    // -----------------------------------------------------------------------
    // Invariant verification
    // -----------------------------------------------------------------------
    eprintln!("\n=== Invariant Verification ===");
    let mut all_pass = true;

    macro_rules! check {
        ($label:expr, $a:expr, $b:expr) => {
            if $a == $b {
                eprintln!("  [PASS] {} identical: A={}  B={}", $label, $a, $b);
            } else {
                eprintln!("  [FAIL] {} MISMATCH: A={}  B={}", $label, $a, $b);
                all_pass = false;
            }
        };
    }

    // best_obj: compare bit-exact (f64 must be identical, not just close)
    check!(
        "best_obj bits",
        result_a.best_obj.to_bits(),
        result_b.best_obj.to_bits()
    );
    check!("n_actual_evals", n_eval_a, n_eval_b);
    check!(
        "generations_run",
        result_a.generations_run,
        result_b.generations_run
    );
    check!("valid", result_a.valid, result_b.valid);
    check!("cache_hits", cache_hits_a, cache_hits_b);

    eprintln!("\n=== Performance Summary ===");
    eprintln!("  Wall-clock A (Phase 4)    : {}ms", t_a_ms);
    eprintln!("  Wall-clock B (Phase 4+L2) : {}ms", t_b_ms);
    if t_a_ms > 0 {
        eprintln!(
            "  Wall-clock speedup        : {:.2}x",
            t_a_ms as f64 / t_b_ms as f64
        );
    }
    eprintln!(
        "  Eval time A               : {:.2}ms  ({:.1}% of wall)",
        eval_ms_a,
        eval_ms_a / t_a_ms as f64 * 100.0
    );
    eprintln!(
        "  Eval time B               : {:.2}ms  ({:.1}% of wall)",
        eval_ms_b,
        eval_ms_b / t_b_ms as f64 * 100.0
    );
    if eval_ms_a > 0.0 {
        eprintln!(
            "  Eval speedup              : {:.2}x",
            eval_ms_a / eval_ms_b
        );
    }
    let t_net = eval_ms_a - eval_ms_b;
    eprintln!("  T_net (A_eval - B_eval)   : {:.2}ms", t_net);
    eprintln!("  L2 cache entries          : {}", l2_entries);

    eprintln!();
    if all_pass {
        eprintln!("=== Overall: ALL INVARIANTS PASS ===");
        if t_net > 0.0 {
            eprintln!(
                "=== Phase 6 PROMOTION CRITERION: MET (T_net={:.2}ms > 0) ===",
                t_net
            );
        } else {
            eprintln!(
                "=== Phase 6 PROMOTION CRITERION: NOT MET (T_net={:.2}ms <= 0) ===",
                t_net
            );
        }
    } else {
        eprintln!("=== Overall: INVARIANT FAILURE — DO NOT PROMOTE ===");
        std::process::exit(1);
    }
}
