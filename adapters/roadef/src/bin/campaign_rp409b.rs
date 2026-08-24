/// RP-409B Campaign Runner — Mutation Strategy A/B Benchmark
///
/// Runs all 20 setA instances with a fixed seed under either the Uniform or
/// PeakTargeted mutation strategy.  Designed to be invoked twice (once per
/// strategy) with the same --seed value so that the two runs are directly
/// comparable.
///
/// Usage:
///   cargo run --bin campaign_rp409b -- --strategy uniform       --seed 42 --out /tmp/rp409b
///   cargo run --bin campaign_rp409b -- --strategy peak_targeted --seed 42 --out /tmp/rp409b
///
/// Outputs (under <out>/<strategy>/):
///   manifest.yaml                    — experiment metadata
///   rp409b_candidates_<inst>.jsonl   — CandidateRecord stream
///   rp409b_generations_<inst>.jsonl  — GenerationRecord stream
///   rp409b_moves_<inst>.jsonl        — MoveRecord stream
///   rp409b_construction_<inst>.jsonl — ConstructionRecord stream
///   logs/<inst>.log                  — per-instance evolution log
///   results.json                     — summary of all 20 instances
///
/// Primary outcome (RP-409B): Peak OSR ↑ AND final objective ↓.
/// Zone metrics (Peak ACR, Shoulder ACR) are diagnostic only.
use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::moga_impl::{
    new_peak_demand_set, run_roadef_evolution, EvolutionRunConfig, MutationStrategy,
    PeakTargetedMutator, RoadefCrossover, RoadefFitnessEvaluator, RoadefGenomeFactory,
    RoadefMutator,
};
use roadef::telemetry::{ComparatorMode, FourStreamTelemetrySink, NullTelemetrySink};

// ---------------------------------------------------------------------------
// Configuration constants
// ---------------------------------------------------------------------------

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const POPULATION_SIZE: usize = 50;
const GENERATION_LIMIT: usize = 500;
const ELITE_COUNT: usize = 5;
const MIN_BUDGET_SECS: u64 = 30;
const MAX_BUDGET_SECS: u64 = 300;
/// Probability of targeting a Peak-arc demand in PeakTargeted mode.
const PEAK_BIAS: f64 = 0.7;

// ---------------------------------------------------------------------------
// CLI argument parsing (no external crate — manual)
// ---------------------------------------------------------------------------

struct Args {
    strategy: MutationStrategy,
    seed: u64,
    out_dir: String,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut strategy = MutationStrategy::Uniform;
    let mut seed: u64 = 12345;
    let mut out_dir = "/tmp/rp409b".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--strategy" => {
                i += 1;
                strategy = match args.get(i).map(|s| s.as_str()) {
                    Some("uniform") => MutationStrategy::Uniform,
                    Some("peak_targeted") => MutationStrategy::PeakTargeted,
                    other => {
                        eprintln!(
                            "Unknown strategy: {:?}. Use 'uniform' or 'peak_targeted'.",
                            other
                        );
                        std::process::exit(1);
                    }
                };
            }
            "--seed" => {
                i += 1;
                seed = args.get(i).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--seed requires a u64 value");
                    std::process::exit(1);
                });
            }
            "--out" => {
                i += 1;
                out_dir = args.get(i).cloned().unwrap_or_else(|| {
                    eprintln!("--out requires a directory path");
                    std::process::exit(1);
                });
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    Args {
        strategy,
        seed,
        out_dir,
    }
}

fn strategy_str(s: MutationStrategy) -> &'static str {
    match s {
        MutationStrategy::Uniform => "uniform",
        MutationStrategy::PeakTargeted => "peak_targeted",
    }
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstanceResult {
    instance_id: usize,
    name: String,
    strategy: String,
    seed: u64,
    num_demands: usize,
    num_nodes: usize,
    num_links: usize,
    best_obj: f64,
    best_mlu: f64,
    valid: bool,
    runtime_ms: u128,
    generations: usize,
    termination_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CampaignResults {
    experiment: String,
    strategy: String,
    seed: u64,
    timestamp: String,
    total_instances: usize,
    valid_count: usize,
    results: Vec<InstanceResult>,
}

// ---------------------------------------------------------------------------
// Instance discovery
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Manifest emission
// ---------------------------------------------------------------------------

fn write_manifest(out_dir: &str, args: &Args, instances: &[(String, String, String, String)]) {
    let strat = strategy_str(args.strategy);
    let manifest = format!(
        "# RP-409B Experiment Manifest\n\
         # Auto-generated — do not edit\n\
         experiment: RP-409B\n\
         strategy: {strategy}\n\
         peak_bias: {peak_bias}\n\
         seed: {seed}\n\
         comparator: scalar\n\
         baseline_version: RP-406C\n\
         instance_set: setA\n\
         instance_count: {n}\n\
         population_size: {pop}\n\
         generation_limit: {gen}\n\
         elite_count: {elite}\n\
         min_budget_secs: {min_b}\n\
         max_budget_secs: {max_b}\n\
         date: {date}\n",
        strategy = strat,
        peak_bias = PEAK_BIAS,
        seed = args.seed,
        n = instances.len(),
        pop = POPULATION_SIZE,
        gen = GENERATION_LIMIT,
        elite = ELITE_COUNT,
        min_b = MIN_BUDGET_SECS,
        max_b = MAX_BUDGET_SECS,
        date = Utc::now().to_rfc3339(),
    );
    let manifest_path = format!("{}/manifest.yaml", out_dir);
    if let Err(e) = fs::write(&manifest_path, &manifest) {
        eprintln!("  Warning: could not write manifest: {}", e);
    } else {
        eprintln!("  Manifest written: {}", manifest_path);
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args = parse_args();
    let strat = strategy_str(args.strategy);

    // Output directory: <out>/<strategy>/
    let run_dir = format!("{}/{}", args.out_dir, strat);
    let log_dir = format!("{}/logs", run_dir);
    fs::create_dir_all(&log_dir).expect("could not create output directory");

    eprintln!("=== RP-409B Campaign Runner ===");
    eprintln!("Strategy   : {}", strat);
    eprintln!("Peak bias  : {} (PeakTargeted only)", PEAK_BIAS);
    eprintln!("Seed       : {}", args.seed);
    eprintln!("Output dir : {}", run_dir);
    eprintln!(
        "Population : {}  Generations: {}  Elite: {}",
        POPULATION_SIZE, GENERATION_LIMIT, ELITE_COUNT
    );
    eprintln!();

    let instances = discover_instances();
    let total = instances.len();
    eprintln!("Discovered {} instances", total);
    eprintln!();

    // Write manifest before any runs so it exists even if the campaign is interrupted.
    write_manifest(&run_dir, &args, &instances);

    let campaign_start = Instant::now();
    let mut results: Vec<InstanceResult> = Vec::new();

    for (idx, (name, net_path, tm_path, scenario_path)) in instances.iter().enumerate() {
        let instance_num = idx + 1;
        eprintln!("[{}/{}] {}", instance_num, total, name);

        // Load instance
        let net = match load_network(net_path) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("  ERROR loading network: {}", e);
                continue;
            }
        };
        let tm = match load_traffic_matrix(tm_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("  ERROR loading traffic matrix: {}", e);
                continue;
            }
        };
        let scenario = match load_scenario(scenario_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  ERROR loading scenario: {}", e);
                continue;
            }
        };

        let num_demands = tm.demands.len();
        let num_time_slots = tm.num_time_slots;
        let num_nodes = net.nodes.len();
        let num_links = net.links.len();
        let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

        eprintln!(
            "  nodes={} links={} demands={}",
            num_nodes, num_links, num_demands
        );

        // Adaptive time budget (same formula as RP-408B for comparability)
        let raw_budget_ms = (num_demands as u64) * (num_links as u64) / 2;
        let budget_secs =
            raw_budget_ms.clamp(MIN_BUDGET_SECS * 1000, MAX_BUDGET_SECS * 1000) / 1000;
        eprintln!("  budget={}s", budget_secs);

        // Build MOGA components
        let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
        let factory = RoadefGenomeFactory {
            num_demands,
            num_time_slots,
            node_ids: node_ids.clone(),
            mode: roadef::moga_impl::ConstructionMode::Random,
            greedy_data: None,
        };
        let fitness_eval = RoadefFitnessEvaluator {
            evaluator: Arc::clone(&evaluator),
            l2_cache: None,
        };
        let crossover = RoadefCrossover;

        // RP-409B: fixed seed per instance (same formula as RP-408B for comparability)
        let instance_seed = args.seed ^ (instance_num as u64 * 0x9e3779b97f4a7c15);

        // Build mutator and evo_config based on strategy.
        // For PeakTargeted: create a shared PeakDemandSet and pass it to both
        // the mutator and the EvolutionRunConfig so the loop updates it.
        let run_result = match args.strategy {
            MutationStrategy::Uniform => {
                let mutator = RoadefMutator {
                    node_ids: node_ids.clone(),
                };
                let evo_config = EvolutionRunConfig {
                    population_size: POPULATION_SIZE,
                    elite_count: ELITE_COUNT,
                    generation_limit: GENERATION_LIMIT,
                    mutation_rate: 0.3,
                    crossover_rate: 0.7,
                    no_improvement_limit: 20,
                    seed: Some(instance_seed),
                    log_interval: 10,
                    health_interval: 20,
                    max_runtime: Some(std::time::Duration::from_secs(budget_secs)),
                    comparator_mode: ComparatorMode::Scalar,
                    peak_demand_set: None,
                };

                let log_path = format!("{}/{}.log", log_dir, name);
                let log_file = fs::File::create(&log_path).ok();
                let mut log_buf: Box<dyn std::io::Write> = match log_file {
                    Some(f) => Box::new(BufWriter::new(f)),
                    None => Box::new(std::io::stderr()),
                };

                let cand_path = format!("{}/rp409b_candidates_{}.jsonl", run_dir, name);
                let gen_path = format!("{}/rp409b_generations_{}.jsonl", run_dir, name);
                let move_path = format!("{}/rp409b_moves_{}.jsonl", run_dir, name);
                let cons_path = format!("{}/rp409b_construction_{}.jsonl", run_dir, name);

                let cf = fs::File::create(&cand_path).map(|f| BufWriter::new(f));
                let gf = fs::File::create(&gen_path).map(|f| BufWriter::new(f));
                let mf = fs::File::create(&move_path).map(|f| BufWriter::new(f));
                let nf = fs::File::create(&cons_path).map(|f| BufWriter::new(f));

                match (cf, gf, mf, nf) {
                    (Ok(cf), Ok(gf), Ok(mf), Ok(nf)) => {
                        let mut sink = FourStreamTelemetrySink::new_full(cf, gf, mf, nf);
                        run_roadef_evolution(
                            &factory,
                            &fitness_eval,
                            &mutator,
                            &crossover,
                            &evo_config,
                            name,
                            &mut *log_buf,
                            &mut sink,
                        )
                    }
                    _ => {
                        eprintln!("  Warning: could not create telemetry files; running without telemetry");
                        run_roadef_evolution(
                            &factory,
                            &fitness_eval,
                            &mutator,
                            &crossover,
                            &evo_config,
                            name,
                            &mut *log_buf,
                            &mut NullTelemetrySink,
                        )
                    }
                }
            }

            MutationStrategy::PeakTargeted => {
                // Create shared peak-demand set — updated by the loop after each
                // global-best improvement, read by the mutator on each mutation call.
                let pds = new_peak_demand_set();
                let mutator = PeakTargetedMutator {
                    node_ids: node_ids.clone(),
                    peak_demand_set: Arc::clone(&pds),
                    peak_bias: PEAK_BIAS,
                };
                let evo_config = EvolutionRunConfig {
                    population_size: POPULATION_SIZE,
                    elite_count: ELITE_COUNT,
                    generation_limit: GENERATION_LIMIT,
                    mutation_rate: 0.3,
                    crossover_rate: 0.7,
                    no_improvement_limit: 20,
                    seed: Some(instance_seed),
                    log_interval: 10,
                    health_interval: 20,
                    max_runtime: Some(std::time::Duration::from_secs(budget_secs)),
                    comparator_mode: ComparatorMode::Scalar,
                    peak_demand_set: Some(pds),
                };

                let log_path = format!("{}/{}.log", log_dir, name);
                let log_file = fs::File::create(&log_path).ok();
                let mut log_buf: Box<dyn std::io::Write> = match log_file {
                    Some(f) => Box::new(BufWriter::new(f)),
                    None => Box::new(std::io::stderr()),
                };

                let cand_path = format!("{}/rp409b_candidates_{}.jsonl", run_dir, name);
                let gen_path = format!("{}/rp409b_generations_{}.jsonl", run_dir, name);
                let move_path = format!("{}/rp409b_moves_{}.jsonl", run_dir, name);
                let cons_path = format!("{}/rp409b_construction_{}.jsonl", run_dir, name);

                let cf = fs::File::create(&cand_path).map(|f| BufWriter::new(f));
                let gf = fs::File::create(&gen_path).map(|f| BufWriter::new(f));
                let mf = fs::File::create(&move_path).map(|f| BufWriter::new(f));
                let nf = fs::File::create(&cons_path).map(|f| BufWriter::new(f));

                match (cf, gf, mf, nf) {
                    (Ok(cf), Ok(gf), Ok(mf), Ok(nf)) => {
                        let mut sink = FourStreamTelemetrySink::new_full(cf, gf, mf, nf);
                        run_roadef_evolution(
                            &factory,
                            &fitness_eval,
                            &mutator,
                            &crossover,
                            &evo_config,
                            name,
                            &mut *log_buf,
                            &mut sink,
                        )
                    }
                    _ => {
                        eprintln!("  Warning: could not create telemetry files; running without telemetry");
                        run_roadef_evolution(
                            &factory,
                            &fitness_eval,
                            &mutator,
                            &crossover,
                            &evo_config,
                            name,
                            &mut *log_buf,
                            &mut NullTelemetrySink,
                        )
                    }
                }
            }
        };

        eprintln!(
            "  obj={:.4} mlu={:.4} valid={} gens={} reason={} runtime={}ms",
            run_result.best_obj,
            run_result.best_mlu,
            run_result.valid,
            run_result.generations_run,
            run_result.termination_reason,
            run_result.runtime_ms
        );

        results.push(InstanceResult {
            instance_id: instance_num,
            name: name.clone(),
            strategy: strat.to_string(),
            seed: instance_seed,
            num_demands,
            num_nodes,
            num_links,
            best_obj: run_result.best_obj,
            best_mlu: run_result.best_mlu,
            valid: run_result.valid,
            runtime_ms: run_result.runtime_ms,
            generations: run_result.generations_run,
            termination_reason: run_result.termination_reason.clone(),
        });
    }

    // Write results summary
    let valid_count = results.iter().filter(|r| r.valid).count();
    let campaign_results = CampaignResults {
        experiment: "RP-409B".to_string(),
        strategy: strat.to_string(),
        seed: args.seed,
        timestamp: Utc::now().to_rfc3339(),
        total_instances: results.len(),
        valid_count,
        results,
    };

    let results_path = format!("{}/results.json", run_dir);
    match serde_json::to_string_pretty(&campaign_results) {
        Ok(json) => {
            if let Err(e) = fs::write(&results_path, &json) {
                eprintln!("Warning: could not write results.json: {}", e);
            } else {
                eprintln!("Results written: {}", results_path);
            }
        }
        Err(e) => eprintln!("Warning: could not serialise results: {}", e),
    }

    let elapsed = campaign_start.elapsed();
    eprintln!();
    eprintln!("=== RP-409B Campaign Complete ===");
    eprintln!("Strategy  : {}", strat);
    eprintln!(
        "Instances : {}/{}",
        valid_count, campaign_results.total_instances
    );
    eprintln!("Elapsed   : {:.1}s", elapsed.as_secs_f64());
}
