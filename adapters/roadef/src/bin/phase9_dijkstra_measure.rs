/// phase9_dijkstra_measure.rs — H6-revised Dijkstra Cache Precondition Measurement
///
/// Governance protocol: OBSERVATIONAL — measurement-only binary.
/// No changes to production path. Uses thread-local counters added to ecmp.rs.
///
/// Measures four H6-revised preconditions:
///   1. Total backward_dijkstra() calls and timing share of improve_ms
///   2. Potential cache hit rate (total calls vs max unique (target,slot) entries)
///   3. DijkstraResult memory footprint estimate
///   4. (Clone cost assessed from DijkstraResult structure)
///
/// Usage:
///   cargo run --release -p roadef --bin phase9_dijkstra_measure -- [--instance setA-01] [--gens 50] [--seed 42]
use std::sync::Arc;
use std::time::Instant;

use roadef::ecmp::{dijkstra_counters_read, dijkstra_counters_reset};
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

fn pct(part: f64, total: f64) -> f64 {
    if total == 0.0 { 0.0 } else { part / total * 100.0 }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut instance_name = "setA-01".to_string();
    let mut generation_limit: usize = 50;
    let mut seed: u64 = 42;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--instance" => { if let Some(v) = args.next() { instance_name = v; } }
            "--gens"     => { if let Some(v) = args.next() { generation_limit = v.parse().unwrap_or(50); } }
            "--seed"     => { if let Some(v) = args.next() { seed = v.parse().unwrap_or(42); } }
            _ => {}
        }
    }

    let net_path      = format!("{}/{}-net.json",      INSTANCE_DIR, instance_name);
    let tm_path       = format!("{}/{}-tm.json",       INSTANCE_DIR, instance_name);
    let scenario_path = format!("{}/{}-scenario.json", INSTANCE_DIR, instance_name);

    eprintln!("=== Phase 9 H6-revised: Dijkstra Precondition Measurement ===");
    eprintln!("Instance   : {}", instance_name);
    eprintln!("Generations: {}", generation_limit);
    eprintln!("Seed       : {}", seed);

    let net      = roadef::loader::load_network(&net_path).expect("load network");
    let tm       = roadef::loader::load_traffic_matrix(&tm_path).expect("load tm");
    let scenario = roadef::loader::load_scenario(&scenario_path).expect("load scenario");

    let n_nodes      = net.nodes.len();
    let n_time_slots = tm.num_time_slots;
    let n_demands    = tm.demands.len();
    let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

    eprintln!("  nodes={}, time_slots={}, demands={}", n_nodes, n_time_slots, n_demands);
    eprintln!("  max_cache_entries (N×T) = {}", n_nodes * n_time_slots);

    let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
    let fitness_eval = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
        l2_cache: None,
    };

    let factory = RoadefGenomeFactory {
        num_demands: n_demands,
        num_time_slots: n_time_slots,
        node_ids: node_ids.clone(),
        mode: ConstructionMode::Random,
        greedy_data: None,
    };

    let init_pop = generate_gen0_population(&factory, &fitness_eval, Some(seed), POPULATION_SIZE);

    let mutator  = RoadefMutator { node_ids: node_ids.clone() };
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

    // Reset counters immediately before the run.
    dijkstra_counters_reset();

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
        true, // Rayon on (matches production configuration)
    );
    let wall_ms = wall_start.elapsed().as_millis() as f64;

    // Read aggregate Dijkstra counters.
    let (total_calls, total_us, unique_targets) = dijkstra_counters_read();
    let dijkstra_ms = total_us as f64 / 1000.0;

    // Aggregate trajectory timing.
    let total_improve_ms: f64 = result.trajectory.iter().map(|g| g.improve_ms).sum();
    let total_repair_ms:  f64 = result.trajectory.iter().map(|g| g.repair_ms).sum();
    let total_feas_ms:    f64 = result.trajectory.iter().map(|g| g.feasibility_ms).sum();
    let n_gens = result.trajectory.len();
    let last = result.trajectory.last();

    // Theoretical max unique (target, slot) entries per generation.
    let max_unique_per_gen = n_nodes * n_time_slots;
    // Upper bound on reuse: if all max_unique entries are queried once, remaining calls are reuse.
    let upper_bound_reuse_pct = pct(
        (total_calls as f64 - max_unique_per_gen as f64 * n_gens as f64).max(0.0),
        total_calls as f64,
    );

    // DijkstraResult memory estimate:
    //   dist:  HashMap<u64,f64>       — N entries × 16 bytes + ~48 overhead
    //   preds: HashMap<u64,Vec<usize>> — N entries × (8 + avg 2 arcs × 8) = N×24 + ~48 overhead
    //   Total per entry: N×40 + 96 bytes
    let bytes_per_result = n_nodes * 40 + 96;
    let cache_worst_bytes = max_unique_per_gen * bytes_per_result;

    println!("=== Phase 9 H6-revised: Dijkstra Precondition Measurements ===");
    println!("Instance: {}  gens={}  seed={}", instance_name, n_gens, seed);
    println!("  nodes={}, time_slots={}, demands={}", n_nodes, n_time_slots, n_demands);
    println!();
    println!("--- PC1: Dijkstra call volume ---");
    println!("  total backward_dijkstra() calls  : {}", total_calls);
    println!("  calls per generation (avg)        : {:.1}", total_calls as f64 / n_gens as f64);
    println!();
    println!("--- PC2: Dijkstra timing share ---");
    println!("  dijkstra_ms (instrumented)        : {:.1}", dijkstra_ms);
    println!("  improve_ms  (process_offspring)   : {:.1}", total_improve_ms);
    println!("  repair_ms   (process_offspring)   : {:.1}", total_repair_ms);
    println!("  feasibility_ms (pre-check)        : {:.1}  (should be 0 post-H3)", total_feas_ms);
    println!("  wall_ms (measured)                : {:.1}", wall_ms);
    println!("  dijkstra_ms / improve_ms          : {:.1}%", pct(dijkstra_ms, total_improve_ms));
    println!("  dijkstra_ms / wall_ms             : {:.1}%", pct(dijkstra_ms, wall_ms));
    println!();
    // Observed hit rate: unique_targets is the count of distinct target node IDs
    // seen across the entire run. Since T time slots share the same target nodes,
    // the actual unique (target,slot) pairs is at most unique_targets × T.
    // Observed reuse = (total_calls - unique_targets) / total_calls (lower bound,
    // since same target may be called in different slots).
    let observed_reuse_lower_pct = if total_calls > 0 {
        ((total_calls as f64 - unique_targets as f64) / total_calls as f64 * 100.0).max(0.0)
    } else { 0.0 };
    let calls_saved_per_entry = if unique_targets > 0 {
        (total_calls - unique_targets) / unique_targets
    } else { 0 };

    println!("--- PC3: Cache hit potential ---");
    println!("  max unique (target,slot)/gen      : {} (N={} × T={})", max_unique_per_gen, n_nodes, n_time_slots);
    println!("  observed unique target IDs (run)  : {}", unique_targets);
    println!("  total calls over {} gens           : {}", n_gens, total_calls);
    println!("  upper bound reuse (theoretical)   : {:.1}%  (if all unique entries queried each gen)", upper_bound_reuse_pct);
    println!("  observed reuse lower bound        : {:.1}%  ((calls - unique_targets) / calls)", observed_reuse_lower_pct);
    println!("  calls / (max_unique × gens)       : {:.2}x", total_calls as f64 / (max_unique_per_gen as f64 * n_gens as f64));
    println!("  calls saved per unique target     : {}", calls_saved_per_entry);
    println!();
    println!("--- PC4: Memory footprint ---");
    println!("  DijkstraResult size estimate      : {} bytes/entry", bytes_per_result);
    println!("  worst-case cache (N×T entries)    : {} bytes ({:.2} MB)",
        cache_worst_bytes, cache_worst_bytes as f64 / 1_048_576.0);
    println!("  NOTE: actual cache holds only queried targets, not all N×T");
    println!("  Clone cost: DijkstraResult contains 2 HashMaps — O(N) clone per hit");
    println!("  Arc<DijkstraResult> would reduce clone to O(1) pointer copy");
    println!();
    println!("--- Trajectory invariants ---");
    if let Some(g) = last {
        println!("  best_obj       : {}", g.best_obj);
        println!("  n_eval         : {}", g.n_eval);
        println!("  cache_hits     : {}", g.cache_hits);
    }
    println!("  valid          : {}", result.valid);
    println!("  generations    : {}", n_gens);
}