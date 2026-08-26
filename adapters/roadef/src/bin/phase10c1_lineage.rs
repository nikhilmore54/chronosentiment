/// phase10c1_lineage.rs — P10-C1 C1-B Bottleneck Lineage Sweep
///
/// Governance protocol: OBSERVATIONAL — measurement-only binary.
/// No changes to production path. Uses [c1b] instrumentation added to
/// pipeline_impl.rs (commit 0f1896fa4).
///
/// Answers the question: "When does each dominant bottleneck arc first appear,
/// and through which operator (crossover_ca / crossover_cb / mutation)?"
///
/// Target arcs: 968 (setA-19), 658 (setA-13), 606 (setA-16), 303 (setA-16)
///
/// Uses io::stdout() as log_sink so [c1b] lines stream to stdout and can be
/// captured. Runs only the 3 high-coverage instances: setA-13, setA-16, setA-19.
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c1_lineage -- [--gens 5] [--seed 42] 2>/dev/null
///
/// Governance: C1-B is observational only. No behavioral changes.
/// C1-C (parent comparison) requires C1-B evidence first.
use std::io::{self, Write};
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

// C1-B: only the 3 high-coverage instances (setA-04/06/10/14 have <13% diag coverage).
const INSTANCES: &[&str] = &[
    "setA-13", // 2000 demands, 200 nodes — arc 658 dominant (96.7%)
    "setA-16", // 4800 demands, 250 nodes — arc 606 (74.0%), arc 303 (18.4%)
    "setA-19", // 6000 demands, 300 nodes — arc 968 dominant (91.1%)
];

// ---------------------------------------------------------------------------
// NullSink — discards telemetry (we only care about [c1b] log lines).
// ---------------------------------------------------------------------------

struct NullSink;

impl TelemetrySink for NullSink {
    fn emit_move(&mut self, _record: &MoveRecord) {}
    fn emit_generation(&mut self, _record: &GenerationRecord) {}
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

    eprintln!("=== P10-C1 C1-B: Bottleneck Lineage Sweep ===");
    eprintln!("Governance: OBSERVATIONAL — no behavioral changes");
    eprintln!("Generations: {}", generation_limit);
    eprintln!("Seed       : {}", seed);
    eprintln!("Instances  : {}", INSTANCES.join(", "));
    eprintln!("Target arcs: 968 (setA-19), 658 (setA-13), 606/303 (setA-16)");
    eprintln!("Output     : [c1b] lines on stdout, instance headers on stderr");
    eprintln!();

    // Use stdout as log_sink so [c1b] lines are captured by the caller.
    let stdout = io::stdout();
    let mut log_sink = stdout.lock();

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

        // Emit instance separator to stdout so it appears in the captured output.
        let _ = writeln!(log_sink, "=== {} ===", instance_name);

        let mut telemetry = NullSink;
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

        eprintln!("  done: wall_ms={:.0}", wall_ms);
        let _ = log_sink.flush();
    }

    eprintln!();
    eprintln!("=== C1-B sweep complete ===");
    eprintln!("Extract first [c1b] line per target arc to establish first-appearance generation.");
    eprintln!("  arc 658: grep '[c1b].*arc=658' output | head -1");
    eprintln!("  arc 606: grep '[c1b].*arc=606' output | head -1");
    eprintln!("  arc 303: grep '[c1b].*arc=303' output | head -1");
    eprintln!("  arc 968: grep '[c1b].*arc=968' output | head -1");
}