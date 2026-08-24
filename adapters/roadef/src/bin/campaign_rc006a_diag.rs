/// campaign_rc006a_diag — RC-006A Phase 1: Invariant Corruption Diagnostic
///
/// Runs only setA-18 and setA-20 (the two instances with IFR=1.00 but final valid=false)
/// with stderr [diag] output captured to identify which operator (crossover vs mutation)
/// is producing invalid offspring.
///
/// The [diag] lines are emitted by the evolution loop in moga_impl.rs whenever an
/// offspring with waypoints evaluates as invalid. Each line includes:
///   gen=N  origin=<crossover|mutation|crossover+mutation|elite>  overload=<class>  max_sat=...
///
/// RC-006A Hypotheses:
///   H1 (mutation):   [diag] lines with origin=mutation appear
///   H2 (crossover):  [diag] lines with origin=crossover appear
///   H3 (evaluator):  [diag] lines with origin=elite appear (re-evaluation inconsistency)
///
/// Usage:
///   cargo run --bin campaign_rc006a_diag --release 2>rc006a_diag.txt
///   grep '\[diag\]' rc006a_diag.txt | head -50
///
/// Classification: RC-006A Phase 1 diagnostic binary.
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::Network;
use roadef::moga_impl::{
    run_roadef_evolution, ConstructionMode, EvolutionRunConfig, GreedyConstructorData,
    RoadefCrossover, RoadefFitnessEvaluator, RoadefGenomeFactory, RoadefMutator,
};
use roadef::telemetry::{ComparatorMode, NullTelemetrySink};

const INSTANCE_DIR: &str = "repo/challenge-roadef-2026-main/setA";

// Diagnostic instances — the two with IFR=1.00 but final valid=false
const DIAG_INSTANCES: &[&str] = &["setA-18", "setA-20"];

// Short budget for diagnostic run — enough to trigger the invariant
const DIAG_BUDGET_SECS: u64 = 60;

const POPULATION_SIZE: usize = 50;
const GENERATION_LIMIT: usize = 500;
const ELITE_COUNT: usize = 5;
const FIXED_SEED: u64 = 42;

fn build_greedy_data(net: &Network, evaluator: Arc<RoadefEvaluator>) -> Arc<GreedyConstructorData> {
    let mut demands_by_volume: Vec<(usize, u64, u64, f64)> = evaluator
        .tm
        .demands
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let max_vol = d.v.iter().cloned().fold(0.0_f64, f64::max);
            (i, d.s, d.t, max_vol)
        })
        .collect();
    demands_by_volume.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let link_capacity: HashMap<u64, f64> = evaluator
        .graph
        .arcs
        .iter()
        .map(|a| (a.id, a.capacity))
        .collect();

    let max_segments = evaluator.scenario.max_segments.max(0) as usize;

    Arc::new(GreedyConstructorData {
        network: net.clone(),
        evaluator,
        demands_by_volume,
        max_segments,
        link_capacity,
    })
}

fn main() {
    eprintln!("=== RC-006A Phase 1: Invariant Corruption Diagnostic ===");
    eprintln!("Instances: {:?}", DIAG_INSTANCES);
    eprintln!("Budget: {}s per instance per arm", DIAG_BUDGET_SECS);
    eprintln!(
        "Seed: {}  Population: {}  Generations: {}",
        FIXED_SEED, POPULATION_SIZE, GENERATION_LIMIT
    );
    eprintln!("---");
    eprintln!("[diag] lines below identify invalid offspring by operator origin.");
    eprintln!("H1 confirmed if: origin=mutation lines appear");
    eprintln!("H2 confirmed if: origin=crossover lines appear");
    eprintln!("H3 confirmed if: origin=elite lines appear");
    eprintln!("---");

    for instance_name in DIAG_INSTANCES {
        let net_path = format!("{}/{}-net.json", INSTANCE_DIR, instance_name);
        let tm_path = format!("{}/{}-tm.json", INSTANCE_DIR, instance_name);
        let scenario_path = format!("{}/{}-scenario.json", INSTANCE_DIR, instance_name);

        if !Path::new(&net_path).exists() {
            eprintln!("SKIP {} — files not found", instance_name);
            continue;
        }

        eprintln!("\n=== {} ===", instance_name);

        let net = load_network(&net_path).expect("Failed to load network");
        let tm = load_traffic_matrix(&tm_path).expect("Failed to load TM");
        let scenario = load_scenario(&scenario_path).expect("Failed to load scenario");

        let num_demands = tm.demands.len();
        let num_time_slots = tm.num_time_slots;
        let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

        let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
        let fitness_eval = RoadefFitnessEvaluator {
            evaluator: evaluator.clone(),
            l2_cache: None,
        };
        let mutator = RoadefMutator {
            node_ids: node_ids.clone(),
        };
        let crossover = RoadefCrossover;

        // Run Arm B (Greedy) only — this is the arm that shows the invariant violation
        let greedy_data = build_greedy_data(&net, evaluator.clone());
        let factory = RoadefGenomeFactory {
            num_demands,
            num_time_slots,
            node_ids,
            mode: ConstructionMode::GreedyLoadAware,
            greedy_data: Some(greedy_data),
        };

        let evo_config = EvolutionRunConfig {
            population_size: POPULATION_SIZE,
            elite_count: ELITE_COUNT,
            generation_limit: GENERATION_LIMIT,
            mutation_rate: 0.3,
            crossover_rate: 0.7,
            no_improvement_limit: 20,
            seed: Some(FIXED_SEED),
            log_interval: 50,
            health_interval: 100,
            max_runtime: Some(std::time::Duration::from_secs(DIAG_BUDGET_SECS)),
            comparator_mode: ComparatorMode::Scalar,
            peak_demand_set: None,
        };

        // Log to stderr (the [diag] lines are already written to stderr by the evolution loop)
        let mut log_buf: Box<dyn std::io::Write> = Box::new(std::io::stderr());
        let result = run_roadef_evolution(
            &factory,
            &fitness_eval,
            &mutator,
            &crossover,
            &evo_config,
            instance_name,
            &mut *log_buf,
            &mut NullTelemetrySink,
        );

        eprintln!("--- {} Arm B result ---", instance_name);
        eprintln!("  IFR:          {:.4}", result.initial_feasibility_rate);
        eprintln!("  valid:        {}", result.valid);
        eprintln!("  best_obj:     {}", result.best_obj);
        eprintln!("  generations:  {}", result.generations_run);
        eprintln!("  termination:  {}", result.termination_reason);
        eprintln!(
            "  invariant:    IFR=1.0 AND valid=false = {}",
            result.initial_feasibility_rate >= 1.0 && !result.valid
        );
    }

    eprintln!("\n=== RC-006A Phase 1 complete ===");
    eprintln!("Filter [diag] lines: grep '\\[diag\\]' rc006a_diag.txt");
    eprintln!("Count by origin:     grep '\\[diag\\]' rc006a_diag.txt | grep -oP 'origin=\\S+' | sort | uniq -c");
}
