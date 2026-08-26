/// phase10c0_repair_effectiveness.rs — P10-C0 Repair-Effectiveness Characterization Sweep
///
/// Governance protocol: OBSERVATIONAL — measurement-only binary.
/// No changes to production path. Uses P10-C0 counters added to pipeline_impl.rs
/// and GenerationRecord in telemetry.rs.
///
/// Answers the question: "What actually happens to an offspring during a failed repair attempt?"
///
/// Measures per-failed-repair:
///   - genome_changed: did repair modify the waypoints?
///   - violation_count_improved/unchanged/worsened: did violation count change?
///   - mean_max_sat_before/after: did capacity saturation change?
///
/// Uses run_pipeline_evolution (v1) which contains the P10-C0 instrumentation.
/// Same 7-instance ladder as P10-B: setA-04/06/10/13/14/16/19
/// Same seed=42, same 5 generations.
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c0_repair_effectiveness -- [--gens 5] [--seed 42]
///
/// Governance: P10-C hypothesis selection remains LOCKED until this evidence is reviewed.
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

// P10-C0 7-instance ladder (same as P10-B sweep).
const INSTANCES: &[&str] = &[
    "setA-04", // 200 demands,  50 nodes
    "setA-06", // 500 demands, 100 nodes
    "setA-10", // 1000 demands, 150 nodes
    "setA-13", // 2000 demands, 200 nodes
    "setA-14", // 600 demands,  250 nodes
    "setA-16", // 4800 demands, 250 nodes
    "setA-19", // 6000 demands, 300 nodes
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

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

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

    eprintln!("=== P10-C0 Repair-Effectiveness Characterization Sweep ===");
    eprintln!("Governance: OBSERVATIONAL — no behavioral changes");
    eprintln!("Generations: {}", generation_limit);
    eprintln!("Seed       : {}", seed);
    eprintln!("Instances  : {}", INSTANCES.join(", "));
    eprintln!("Question   : What actually happens inside a failed repair attempt?");
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

        // Aggregate P10-C0 counters.
        let total_genome_changed: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10c0_genome_changed_count as u64)
            .sum();
        let total_genome_unchanged: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10c0_genome_unchanged_count as u64)
            .sum();
        let total_viol_improved: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10c0_violation_count_improved as u64)
            .sum();
        let total_viol_unchanged: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10c0_violation_count_unchanged as u64)
            .sum();
        let total_viol_worsened: u64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10c0_violation_count_worsened as u64)
            .sum();
        let sum_max_sat_before: f64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10c0_sum_max_sat_before)
            .sum();
        let sum_max_sat_after: f64 = telemetry
            .generations
            .iter()
            .map(|g| g.p10c0_sum_max_sat_after)
            .sum();

        let total_offspring = total_infeasible + total_feasible;
        let infeasibility_rate = if total_offspring > 0 {
            total_infeasible as f64 / total_offspring as f64 * 100.0
        } else {
            0.0
        };
        let repair_share = if wall_ms > 0.0 {
            total_repair_ms / wall_ms * 100.0
        } else {
            0.0
        };
        let ms_per_repair = if total_repair_failures > 0 {
            total_repair_ms / total_repair_failures as f64
        } else {
            f64::NAN
        };

        // P10-C0 derived metrics.
        let genome_changed_rate = if total_repair_failures > 0 {
            total_genome_changed as f64 / total_repair_failures as f64 * 100.0
        } else {
            f64::NAN
        };
        let genome_unchanged_rate = if total_repair_failures > 0 {
            total_genome_unchanged as f64 / total_repair_failures as f64 * 100.0
        } else {
            f64::NAN
        };
        let viol_improved_rate = if total_repair_failures > 0 {
            total_viol_improved as f64 / total_repair_failures as f64 * 100.0
        } else {
            f64::NAN
        };
        let viol_unchanged_rate = if total_repair_failures > 0 {
            total_viol_unchanged as f64 / total_repair_failures as f64 * 100.0
        } else {
            f64::NAN
        };
        let viol_worsened_rate = if total_repair_failures > 0 {
            total_viol_worsened as f64 / total_repair_failures as f64 * 100.0
        } else {
            f64::NAN
        };
        // sat_n = number of failures that had at least one Capacity violation.
        let sat_n = total_genome_changed + total_genome_unchanged;
        let mean_max_sat_before = if sat_n > 0 {
            sum_max_sat_before / sat_n as f64
        } else {
            f64::NAN
        };
        let mean_max_sat_after = if sat_n > 0 {
            sum_max_sat_after / sat_n as f64
        } else {
            f64::NAN
        };

        println!("=== {} ===  nodes={}, demands={}, wall_ms={:.0}", instance_name, n_nodes, n_demands, wall_ms);
        println!("  P10-B (repair scaling):");
        println!("    infeasible={}/{} ({:.1}%), repair_ms={:.1}, ms/repair={:.3}, repair_share={:.1}%",
            total_infeasible, total_offspring, infeasibility_rate,
            total_repair_ms, ms_per_repair, repair_share);
        println!("    repair_attempts={}, successes={}, failures={}",
            total_repair_attempts, total_repair_successes, total_repair_failures);
        println!("    improve_ms={:.1}", total_improve_ms);
        println!("  P10-C0 (repair effectiveness — failed repairs only):");
        println!("    genome_changed={}/{} ({:.1}%)",
            total_genome_changed, total_repair_failures, genome_changed_rate);
        println!("    genome_unchanged={}/{} ({:.1}%)",
            total_genome_unchanged, total_repair_failures, genome_unchanged_rate);
        println!("    violation_count_improved={} ({:.1}%)",
            total_viol_improved, viol_improved_rate);
        println!("    violation_count_unchanged={} ({:.1}%)",
            total_viol_unchanged, viol_unchanged_rate);
        println!("    violation_count_worsened={} ({:.1}%)",
            total_viol_worsened, viol_worsened_rate);
        if mean_max_sat_before.is_finite() {
            println!("    mean_max_sat_before={:.4}  mean_max_sat_after={:.4}  delta={:.4}",
                mean_max_sat_before, mean_max_sat_after,
                mean_max_sat_after - mean_max_sat_before);
        } else {
            println!("    mean_max_sat: N/A (no Capacity violations observed)");
        }
        println!();
    }

    println!("=== P10-C0 sweep complete ===");
    println!("Governance: P10-C hypothesis selection LOCKED — review evidence before proceeding.");
}