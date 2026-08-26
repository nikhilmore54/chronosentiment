/// phase10b_repair_measure.rs — P10-B Repair-Scaling Characterization Sweep
///
/// Governance protocol: OBSERVATIONAL — measurement-only binary.
/// No changes to production path. Uses P10-B counters added to pipeline_impl.rs
/// and GenerationRecord in telemetry.rs.
///
/// Measures repair decomposition across the 7-instance P10-B ladder:
///   setA-04 (200d), setA-06 (500d), setA-10 (1000d), setA-13 (2000d),
///   setA-14 (600d), setA-16 (4800d), setA-19 (6000d)
///
/// Uses run_pipeline_evolution (v1) which contains the P10-B instrumentation.
/// A CapturingSink collects GenerationRecord per generation for post-run analysis.
///
/// Per-instance output:
///   - Total infeasible offspring entering repair (summed over all generations)
///   - Total feasible offspring entering improve path
///   - Repair success/failure counts
///   - Total repair_ms and improve_ms (P10-B measured)
///   - repair_ms / infeasible_individual (per-individual cost)
///   - repair_ms / demand (normalized by instance size)
///   - Infeasibility rate = infeasible / (infeasible + feasible)
///
/// Usage:
///   cargo run --release -p roadef --bin phase10b_repair_measure -- [--gens 5] [--seed 42]
///
/// Governance: P10-C remains locked until P10-B evidence is reviewed.
use std::sync::Arc;
use std::time::Instant;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::moga_impl::{
    ConstructionMode, EvolutionRunConfig, RoadefCrossover, RoadefFitnessEvaluator,
    RoadefGenomeFactory, RoadefMutator,
};
use roadef::pipeline_impl::run_pipeline_evolution;
use roadef::telemetry::{
    CandidateRecord, ComparatorMode, ConstructionRecord, GenerationRecord, MoveRecord,
    TelemetrySink,
};

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const POPULATION_SIZE: usize = 50;
const ELITE_COUNT: usize = 5;

// P10-B 7-instance ladder (same as P10-A sweep).
const INSTANCES: &[&str] = &[
    "setA-04", // 200 demands
    "setA-06", // 500 demands
    "setA-10", // 1000 demands
    "setA-13", // 2000 demands
    "setA-14", // 600 demands
    "setA-16", // 4800 demands
    "setA-19", // 6000 demands
];

// ---------------------------------------------------------------------------
// CapturingSink — collects GenerationRecord in memory for post-run analysis.
// ---------------------------------------------------------------------------

struct CapturingSink {
    pub generations: Vec<GenerationRecord>,
}

impl CapturingSink {
    fn new() -> Self {
        Self {
            generations: Vec::new(),
        }
    }
}

impl TelemetrySink for CapturingSink {
    fn emit_move(&mut self, _record: &MoveRecord) {}
    fn emit_generation(&mut self, record: &GenerationRecord) {
        self.generations.push(record.clone());
    }
    fn emit_construction(&mut self, _record: &ConstructionRecord) {}
    fn emit_candidate(&mut self, _record: &CandidateRecord) {}
    fn flush(&mut self) {}
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
    let mut generation_limit: usize = 5;
    let mut seed: u64 = 42;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--gens" => {
                if let Some(v) = args.next() {
                    generation_limit = v.parse().unwrap_or(5);
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

    eprintln!("=== P10-B Repair-Scaling Characterization Sweep ===");
    eprintln!("Generations: {}", generation_limit);
    eprintln!("Seed       : {}", seed);
    eprintln!("Instances  : {}", INSTANCES.join(", "));
    eprintln!("Path       : run_pipeline_evolution (v1, P10-B instrumented)");
    eprintln!();

    for instance_name in INSTANCES {
        let net_path = format!("{}/{}-net.json", INSTANCE_DIR, instance_name);
        let tm_path = format!("{}/{}-tm.json", INSTANCE_DIR, instance_name);
        let scenario_path = format!("{}/{}-scenario.json", INSTANCE_DIR, instance_name);

        eprintln!("=== {} ===", instance_name);

        let net = match load_network(&net_path) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("  ERROR loading network: {}", e);
                continue;
            }
        };
        let tm = match load_traffic_matrix(&tm_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("  ERROR loading traffic matrix: {}", e);
                continue;
            }
        };
        let scenario = match load_scenario(&scenario_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  ERROR loading scenario: {}", e);
                continue;
            }
        };

        let n_nodes = net.nodes.len();
        let n_time_slots = tm.num_time_slots;
        let n_demands = tm.demands.len();
        let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

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

        let mut log_sink: Vec<u8> = Vec::new();
        let mut telemetry = CapturingSink::new();

        let wall_start = Instant::now();
        let _result = run_pipeline_evolution(
            &factory,
            &fitness_eval,
            &mutator,
            &crossover,
            &pipeline,
            &config,
            instance_name,
            &mut log_sink,
            &mut telemetry,
        );
        let wall_ms = wall_start.elapsed().as_millis() as f64;

        // Aggregate P10-B counters from captured GenerationRecords.
        let n_gens = telemetry.generations.len();
        let total_infeasible: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10b_infeasible_entering_repair as u64)
            .sum();
        let total_feasible: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10b_feasible_entering_repair as u64)
            .sum();
        let total_repair_attempts: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10b_repair_attempts as u64)
            .sum();
        let total_repair_successes: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10b_repair_successes as u64)
            .sum();
        let total_repair_failures: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10b_repair_failures as u64)
            .sum();
        let total_repair_ms: f64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10b_repair_ms)
            .sum();
        let total_improve_ms: f64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10b_improve_ms)
            .sum();

        let total_offspring = total_infeasible + total_feasible;
        let infeasibility_rate_pct = pct(total_infeasible as f64, total_offspring as f64);

        // Key normalizations requested by governance:
        //   repair_ms / infeasible_individual
        //   repair_ms / demand
        let repair_ms_per_infeasible = if total_infeasible > 0 {
            total_repair_ms / total_infeasible as f64
        } else {
            f64::NAN
        };
        let repair_ms_per_demand = total_repair_ms / n_demands as f64;

        // Per-generation averages.
        let avg_infeasible_per_gen = if n_gens > 0 {
            total_infeasible as f64 / n_gens as f64
        } else {
            0.0
        };

        println!("=== {} ===", instance_name);
        println!(
            "  nodes={}, time_slots={}, demands={}",
            n_nodes, n_time_slots, n_demands
        );
        println!("  generations_run={}, seed={}", n_gens, seed);
        println!("  wall_ms={:.1}", wall_ms);
        println!();
        println!("--- P10-B: Offspring routing ---");
        println!(
            "  total_offspring                   : {}",
            total_offspring
        );
        println!(
            "  infeasible_entering_repair        : {}  ({:.1}%)",
            total_infeasible, infeasibility_rate_pct
        );
        println!(
            "  feasible_entering_improve         : {}  ({:.1}%)",
            total_feasible,
            pct(total_feasible as f64, total_offspring as f64)
        );
        println!(
            "  avg_infeasible_per_gen            : {:.1}",
            avg_infeasible_per_gen
        );
        println!();
        println!("--- P10-B: Repair outcomes ---");
        println!(
            "  repair_attempts                   : {}",
            total_repair_attempts
        );
        println!(
            "  repair_successes                  : {}  ({:.1}%)",
            total_repair_successes,
            pct(total_repair_successes as f64, total_repair_attempts as f64)
        );
        println!(
            "  repair_failures                   : {}  ({:.1}%)",
            total_repair_failures,
            pct(total_repair_failures as f64, total_repair_attempts as f64)
        );
        println!();
        println!("--- P10-B: Timing decomposition ---");
        println!(
            "  total_repair_ms                   : {:.1}",
            total_repair_ms
        );
        println!(
            "  total_improve_ms                  : {:.1}",
            total_improve_ms
        );
        println!(
            "  repair_ms / (repair+improve)      : {:.1}%",
            pct(total_repair_ms, total_repair_ms + total_improve_ms)
        );
        println!();
        println!("--- P10-B: Key normalizations ---");
        println!(
            "  repair_ms / infeasible_individual : {:.3} ms/individual",
            repair_ms_per_infeasible
        );
        println!(
            "  repair_ms / demand                : {:.3} ms/demand",
            repair_ms_per_demand
        );
        println!(
            "  improve_ms / feasible_individual  : {:.3} ms/individual",
            if total_feasible > 0 {
                total_improve_ms / total_feasible as f64
            } else {
                f64::NAN
            }
        );
        println!();
    }

    eprintln!("=== P10-B sweep complete ===");
}