/// phase7_loop_profile.rs — Phase 7 Evolution Loop Overhead Profile
///
/// Governance protocol: OBSERVATIONAL — no code changes to production path.
///
/// Objective: decompose the non-evaluator wall-clock overhead (88.8% of total
/// on setA-14 Phase 6 baseline) into measurable components using the existing
/// GenerationSummary trajectory fields.
///
/// Available per-generation fields (from GenerationSummary):
///   generation_runtime_ms      — total generation wall-clock
///   evaluation_runtime_ms      — Phase B parallel eval time only
///   cache_lookup_ms            — Phase A L1 cache lookup time
///   cache_hit_materialize_ms   — Phase A L1 cache hit clone time
///   cache_insert_ms            — Phase C L1 cache insert time
///
/// Derived:
///   non_eval_ms = generation_runtime_ms - evaluation_runtime_ms
///   l1_cache_total_ms = cache_lookup_ms + cache_hit_materialize_ms + cache_insert_ms
///   sequential_loop_ms = non_eval_ms - l1_cache_total_ms
///     (covers: selection, crossover, mutation, repair, sort, merge, Rayon spawn/join)
///
/// Output: per-generation CSV + summary statistics to stdout.
///
/// Usage:
///   cargo run --release -p roadef --bin phase7_loop_profile -- [--instance setA-14] [--gens 50] [--seed 42]
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

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

fn stddev(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = mean(v);
    let variance = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64;
    variance.sqrt()
}

fn pct(part: f64, total: f64) -> f64 {
    if total == 0.0 {
        0.0
    } else {
        part / total * 100.0
    }
}

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

    eprintln!("=== Phase 7 Evolution Loop Overhead Profile ===");
    eprintln!("Instance   : {}", instance_name);
    eprintln!("Generations: {}", generation_limit);
    eprintln!("Seed       : {}", seed);
    eprintln!("Pop size   : {}", POPULATION_SIZE);
    eprintln!();

    let net = roadef::loader::load_network(&net_path).expect("load network");
    let tm = roadef::loader::load_traffic_matrix(&tm_path).expect("load tm");
    let scenario = roadef::loader::load_scenario(&scenario_path).expect("load scenario");

    let num_demands = tm.demands.len();
    let num_time_slots = tm.num_time_slots;
    let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

    let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
    let fitness_eval = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
        l2_cache: None, // Phase 6 baseline: L2 off for clean measurement
    };

    let factory = RoadefGenomeFactory {
        num_demands,
        num_time_slots,
        node_ids: node_ids.clone(),
        mode: ConstructionMode::Random,
        greedy_data: None,
    };

    let init_pop = generate_gen0_population(&factory, &fitness_eval, Some(seed), POPULATION_SIZE);
    eprintln!(
        "Gen-0 IFR  : {:.4}",
        init_pop.genomes.len() as f64 / POPULATION_SIZE as f64
    );

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
        no_improvement_limit: generation_limit + 1,
        seed: Some(seed),
        log_interval: generation_limit + 1,
        health_interval: generation_limit + 1,
        max_runtime: None,
        comparator_mode: ComparatorMode::Scalar,
        peak_demand_set: None,
    };

    let wall_start = Instant::now();
    let mut log_sink: Vec<u8> = Vec::new();
    let result: EvolutionRunResult = run_pipeline_evolution_v2(
        &factory,
        &fitness_eval,
        &mutator,
        &crossover,
        &pipeline,
        &config,
        init_pop,
        &instance_name,
        &mut log_sink,
        &mut NullTelemetrySink,
        true, // Rayon on (Phase 6 baseline configuration)
    );
    let wall_ms = wall_start.elapsed().as_millis();

    // -----------------------------------------------------------------------
    // Per-generation breakdown
    // -----------------------------------------------------------------------
    println!("gen,gen_ms,eval_ms,non_eval_ms,l1_lookup_ms,l1_materialize_ms,l1_insert_ms,l1_total_ms,unattributed_ms,n_eval,cache_hits,crossover_ms,mutation_ms,repair_ms,improve_ms,sort_ms,selection_ms,feasibility_ms,staging_ms,attributed_ms,rayon_residual_ms");
    for g in &result.trajectory {
        let non_eval_ms = g.generation_runtime_ms - g.evaluation_runtime_ms;
        let l1_total_ms = g.cache_lookup_ms + g.cache_hit_materialize_ms + g.cache_insert_ms;
        let unattributed_ms = non_eval_ms - l1_total_ms;
        let attributed_ms = g.crossover_ms
            + g.mutation_ms
            + g.repair_ms
            + g.improve_ms
            + g.sort_ms
            + g.selection_ms
            + g.feasibility_ms
            + g.staging_ms;
        let rayon_residual_ms = unattributed_ms - attributed_ms;
        println!(
            "{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            g.generation,
            g.generation_runtime_ms,
            g.evaluation_runtime_ms,
            non_eval_ms,
            g.cache_lookup_ms,
            g.cache_hit_materialize_ms,
            g.cache_insert_ms,
            l1_total_ms,
            unattributed_ms,
            g.n_eval,
            g.cache_hits,
            g.crossover_ms,
            g.mutation_ms,
            g.repair_ms,
            g.improve_ms,
            g.sort_ms,
            g.selection_ms,
            g.feasibility_ms,
            g.staging_ms,
            attributed_ms,
            rayon_residual_ms,
        );
    }

    // -----------------------------------------------------------------------
    // Summary statistics
    // -----------------------------------------------------------------------
    let traj = &result.trajectory;
    let n = traj.len() as f64;

    let gen_ms_vec: Vec<f64> = traj.iter().map(|g| g.generation_runtime_ms).collect();
    let eval_ms_vec: Vec<f64> = traj.iter().map(|g| g.evaluation_runtime_ms).collect();
    let non_eval_ms_vec: Vec<f64> = traj
        .iter()
        .map(|g| g.generation_runtime_ms - g.evaluation_runtime_ms)
        .collect();
    let l1_lookup_vec: Vec<f64> = traj.iter().map(|g| g.cache_lookup_ms).collect();
    let l1_mat_vec: Vec<f64> = traj.iter().map(|g| g.cache_hit_materialize_ms).collect();
    let l1_ins_vec: Vec<f64> = traj.iter().map(|g| g.cache_insert_ms).collect();
    let l1_total_vec: Vec<f64> = traj
        .iter()
        .map(|g| g.cache_lookup_ms + g.cache_hit_materialize_ms + g.cache_insert_ms)
        .collect();
    let unattributed_vec: Vec<f64> = non_eval_ms_vec
        .iter()
        .zip(l1_total_vec.iter())
        .map(|(ne, l1)| ne - l1)
        .collect();

    // Phase 8: operator timing vectors.
    let crossover_vec: Vec<f64> = traj.iter().map(|g| g.crossover_ms).collect();
    let mutation_vec: Vec<f64> = traj.iter().map(|g| g.mutation_ms).collect();
    let repair_vec: Vec<f64> = traj.iter().map(|g| g.repair_ms).collect();
    let improve_vec: Vec<f64> = traj.iter().map(|g| g.improve_ms).collect();
    let sort_vec: Vec<f64> = traj.iter().map(|g| g.sort_ms).collect();
    let selection_vec: Vec<f64> = traj.iter().map(|g| g.selection_ms).collect();
    let feasibility_vec: Vec<f64> = traj.iter().map(|g| g.feasibility_ms).collect();
    let staging_vec: Vec<f64> = traj.iter().map(|g| g.staging_ms).collect();
    let attributed_vec: Vec<f64> = traj
        .iter()
        .map(|g| {
            g.crossover_ms
                + g.mutation_ms
                + g.repair_ms
                + g.improve_ms
                + g.sort_ms
                + g.selection_ms
                + g.feasibility_ms
                + g.staging_ms
        })
        .collect();
    let rayon_residual_vec: Vec<f64> = unattributed_vec
        .iter()
        .zip(attributed_vec.iter())
        .map(|(u, a)| u - a)
        .collect();

    let _total_gen_ms: f64 = gen_ms_vec.iter().sum();
    let total_eval_ms: f64 = eval_ms_vec.iter().sum();
    let total_non_eval_ms: f64 = non_eval_ms_vec.iter().sum();
    let total_l1_lookup_ms: f64 = l1_lookup_vec.iter().sum();
    let total_l1_mat_ms: f64 = l1_mat_vec.iter().sum();
    let total_l1_ins_ms: f64 = l1_ins_vec.iter().sum();
    let total_l1_total_ms: f64 = l1_total_vec.iter().sum();
    let total_unattributed_ms: f64 = unattributed_vec.iter().sum();
    let total_crossover_ms: f64 = crossover_vec.iter().sum();
    let total_mutation_ms: f64 = mutation_vec.iter().sum();
    let total_repair_ms: f64 = repair_vec.iter().sum();
    let total_improve_ms: f64 = improve_vec.iter().sum();
    let total_sort_ms: f64 = sort_vec.iter().sum();
    let total_selection_ms: f64 = selection_vec.iter().sum();
    let total_feasibility_ms: f64 = feasibility_vec.iter().sum();
    let total_staging_ms: f64 = staging_vec.iter().sum();
    let total_attributed_ms: f64 = attributed_vec.iter().sum();
    let total_rayon_residual_ms: f64 = rayon_residual_vec.iter().sum();

    eprintln!();
    eprintln!(
        "=== Phase 8 Operator Attribution Summary: {} generations on {} ===",
        generation_limit, instance_name
    );
    eprintln!();
    eprintln!("Run result:");
    eprintln!("  best_obj        : {:.10}", result.best_obj);
    eprintln!("  valid           : {}", result.valid);
    eprintln!("  generations_run : {}", result.generations_run);
    eprintln!("  wall_clock_ms   : {}", wall_ms);
    eprintln!();
    eprintln!(
        "Component breakdown (totals across {} generations):",
        generation_limit
    );
    eprintln!();
    eprintln!(
        "  {:35} {:>12}  {:>7}  {:>10}  {:>10}",
        "Component", "Total (ms)", "% wall", "Mean/gen", "Stddev"
    );
    eprintln!("  {}", "-".repeat(80));

    let row = |label: &str, total: f64, mean_v: f64, sd: f64| {
        eprintln!(
            "  {:35} {:>12.1}  {:>6.1}%  {:>10.1}  {:>10.1}",
            label,
            total,
            pct(total, wall_ms as f64),
            mean_v,
            sd
        );
    };

    row(
        "Wall-clock (total)",
        wall_ms as f64,
        wall_ms as f64 / n,
        0.0,
    );
    row(
        "  Eval (Phase B parallel)",
        total_eval_ms,
        mean(&eval_ms_vec),
        stddev(&eval_ms_vec),
    );
    row(
        "  Non-eval overhead",
        total_non_eval_ms,
        mean(&non_eval_ms_vec),
        stddev(&non_eval_ms_vec),
    );
    row(
        "    L1 cache lookup",
        total_l1_lookup_ms,
        mean(&l1_lookup_vec),
        stddev(&l1_lookup_vec),
    );
    row(
        "    L1 cache materialize",
        total_l1_mat_ms,
        mean(&l1_mat_vec),
        stddev(&l1_mat_vec),
    );
    row(
        "    L1 cache insert",
        total_l1_ins_ms,
        mean(&l1_ins_vec),
        stddev(&l1_ins_vec),
    );
    row(
        "    L1 cache total",
        total_l1_total_ms,
        mean(&l1_total_vec),
        stddev(&l1_total_vec),
    );
    row(
        "    Unattributed overhead",
        total_unattributed_ms,
        mean(&unattributed_vec),
        stddev(&unattributed_vec),
    );
    eprintln!("  {}", "-".repeat(80));
    eprintln!("  Phase 8 operator breakdown (subset of unattributed):");
    row(
        "    Crossover",
        total_crossover_ms,
        mean(&crossover_vec),
        stddev(&crossover_vec),
    );
    row(
        "    Mutation",
        total_mutation_ms,
        mean(&mutation_vec),
        stddev(&mutation_vec),
    );
    row(
        "    Repair (process_offspring)",
        total_repair_ms,
        mean(&repair_vec),
        stddev(&repair_vec),
    );
    row(
        "    Improve (process_offspring)",
        total_improve_ms,
        mean(&improve_vec),
        stddev(&improve_vec),
    );
    row(
        "    Sort",
        total_sort_ms,
        mean(&sort_vec),
        stddev(&sort_vec),
    );
    row(
        "    Selection",
        total_selection_ms,
        mean(&selection_vec),
        stddev(&selection_vec),
    );
    row(
        "    Feasibility check",
        total_feasibility_ms,
        mean(&feasibility_vec),
        stddev(&feasibility_vec),
    );
    row(
        "    Staging overhead",
        total_staging_ms,
        mean(&staging_vec),
        stddev(&staging_vec),
    );
    row(
        "    Attributed total",
        total_attributed_ms,
        mean(&attributed_vec),
        stddev(&attributed_vec),
    );
    row(
        "    Rayon residual",
        total_rayon_residual_ms,
        mean(&rayon_residual_vec),
        stddev(&rayon_residual_vec),
    );

    eprintln!();
    let attribution_pct = if total_unattributed_ms > 0.0 {
        total_attributed_ms / total_unattributed_ms * 100.0
    } else {
        0.0
    };
    eprintln!(
        "Phase 8 attribution gate: attributed_ms / unattributed_ms = {:.1}%",
        attribution_pct
    );
    if attribution_pct >= 80.0 {
        eprintln!("  GATE PASS: >= 80% attribution achieved.");
    } else {
        eprintln!("  GATE FAIL: < 80% attribution. Rayon coordination must be instrumented before Phase 9.");
    }
    eprintln!();
    eprintln!("Note: 'Rayon residual' = unattributed_ms - attributed_ms.");
    eprintln!("      This is an accounting residual, NOT proof of Rayon causality.");
    eprintln!("      It includes Rayon spawn/join + any other unmeasured overhead.");
    eprintln!();
    eprintln!("Phase 7 baseline invariants:");
    eprintln!("  best_obj       : {:.10}", result.best_obj);
    eprintln!(
        "  n_actual_evals : {}",
        traj.iter().map(|g| g.n_eval).sum::<usize>()
    );
    eprintln!("  generations    : {}", result.generations_run);
    eprintln!("  valid          : {}", result.valid);
    eprintln!(
        "  cache_hits     : {}",
        traj.iter().map(|g| g.cache_hits).sum::<usize>()
    );
}
