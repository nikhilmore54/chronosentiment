/// phase10c1c_initial_scan.rs — P10-C1 C1-C Initial Population Scan
///
/// Governance protocol: OBSERVATIONAL — measurement-only binary.
/// No changes to production path. Uses [c1c] instrumentation added to
/// pipeline_impl.rs (C1-C instrumentation block).
///
/// Answers the question:
///   "Were arcs 658/606/303/968 already overloaded in the initial constructed
///    genomes before any evolutionary operator executed, or did the gen-0
///    evolutionary transition create or propagate the overload?"
///
/// Method:
///   - Runs setA-13/16/19 with seed=42, 1 generation.
///   - [c1c] lines (emitted before the main loop) record each initial genome
///     member's overload status for the 4 target arcs.
///   - [c1b] lines (emitted during gen-0 operator pass) record the child-side
///     overload events, enabling parent→child correlation.
///
/// Causal taxonomy (from C1-C definition):
///   overloaded + overloaded → overloaded  = inherited
///   feasible   + feasible   → overloaded  = crossover-created
///   overloaded + feasible   → overloaded  = inherited/propagated
///   any        + any        → feasible    = no causal event
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c1c_initial_scan -- [--gens 1] [--seed 42] 2>/dev/null
///
/// Governance: C1-C is observational only. No behavioral changes.
/// C1-D through C1-F remain locked until C1-C evidence is reviewed.
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

// C1-C: same 3 high-coverage instances as C1-B.
const INSTANCES: &[&str] = &[
    "setA-13", // arc 658 dominant (96.7% of [diag] events)
    "setA-16", // arc 606 (74.0%), arc 303 (18.4%)
    "setA-19", // arc 968 dominant (91.1%)
];

// ---------------------------------------------------------------------------
// NullSink — discards telemetry (we only care about [c1c] and [c1b] log lines).
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
    // Default: 1 generation — enough to capture [c1c] initial scan + gen-0 [c1b] events.
    let mut generation_limit: usize = 1;
    let mut seed: u64 = 42;
    // Optional: run only specific instances (e.g. --only setA-16,setA-19)
    let mut only_instances: Option<Vec<String>> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--gens" => {
                if let Some(v) = args.next() {
                    generation_limit = v.parse().unwrap_or(1);
                }
            }
            "--seed" => {
                if let Some(v) = args.next() {
                    seed = v.parse().unwrap_or(42);
                }
            }
            "--only" => {
                if let Some(v) = args.next() {
                    only_instances = Some(v.split(',').map(|s| s.trim().to_string()).collect());
                }
            }
            _ => {}
        }
    }

    eprintln!("=== P10-C1 C1-C: Initial Population Scan ===");
    eprintln!("Governance: OBSERVATIONAL — no behavioral changes");
    eprintln!("Generations: {}", generation_limit);
    eprintln!("Seed       : {}", seed);
    eprintln!("Instances  : {}", INSTANCES.join(", "));
    eprintln!("Target arcs: 968 (setA-19), 658 (setA-13), 606/303 (setA-16)");
    eprintln!("Output     : [c1c] lines (initial scan) + [c1b] lines (gen-0 operators) on stdout");
    eprintln!();
    eprintln!("Causal taxonomy:");
    eprintln!("  overloaded + overloaded -> overloaded = inherited");
    eprintln!("  feasible   + feasible   -> overloaded = crossover-created");
    eprintln!("  overloaded + feasible   -> overloaded = inherited/propagated");
    eprintln!("  any        + any        -> feasible   = no causal event");
    eprintln!();

    // Use stdout as log_sink so [c1c] and [c1b] lines are captured by the caller.
    let stdout = io::stdout();
    let mut log_sink = stdout.lock();

    for instance_name in INSTANCES {
        // --only filter: skip instances not in the list.
        if let Some(ref only) = only_instances {
            if !only.iter().any(|o| o == instance_name) {
                eprintln!("  skipping {} (not in --only list)", instance_name);
                continue;
            }
        }
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
    eprintln!("=== C1-C sweep complete ===");
    eprintln!("Analysis commands:");
    eprintln!();
    eprintln!("# Count initial members with arc 658 overloaded (setA-13):");
    eprintln!("  grep '[c1c].*arc=658.*overloaded=true' output | wc -l");
    eprintln!("# Count initial members with arc 658 NOT overloaded (setA-13):");
    eprintln!("  grep '[c1c].*arc=658.*overloaded=false' output | wc -l");
    eprintln!();
    eprintln!("# Count initial members with arc 606 overloaded (setA-16):");
    eprintln!("  grep '[c1c].*arc=606.*overloaded=true' output | wc -l");
    eprintln!("# Count initial members with arc 303 overloaded (setA-16):");
    eprintln!("  grep '[c1c].*arc=303.*overloaded=true' output | wc -l");
    eprintln!();
    eprintln!("# Count initial members with arc 968 overloaded (setA-19):");
    eprintln!("  grep '[c1c].*arc=968.*overloaded=true' output | wc -l");
    eprintln!();
    eprintln!("# Cross-reference: gen-0 [c1b] events (child-side overloads):");
    eprintln!("  grep '[c1b].*gen=0' output");
    eprintln!();
    eprintln!("# Full causal classification:");
    eprintln!("  If ALL initial members show overloaded=true for an arc:");
    eprintln!("    -> inherited (constructor always produces overloaded genomes for this arc)");
    eprintln!("  If SOME initial members show overloaded=true:");
    eprintln!("    -> inherited/propagated (crossover selects from overloaded parents)");
    eprintln!("  If NO initial members show overloaded=true:");
    eprintln!("    -> crossover-created (operator introduces the overload)");
}
