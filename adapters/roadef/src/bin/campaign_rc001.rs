/// campaign_rc001 — RC-001: Load-Aware Greedy Constructor A/B Campaign
///
/// Research question: Does replacing the random constructor (CB-000) with the
/// RP-401C load-aware greedy constructor raise the Initial Feasibility Rate (IFR)
/// of generation 0 and improve the final ROADEF objective?
///
/// Hypothesis: The greedy constructor, which routes demands in volume-descending
/// order using load-aware Dijkstra with additive saturation penalty, will produce
/// a higher fraction of feasible initial candidates than the random constructor,
/// thereby increasing EEB and improving the final objective.
///
/// EEB target: IFR ↑ (Construction subsystem)
/// CB-000 baseline: mean IFR = 10.6%, 6/20 instances with IFR = 0%
///
/// Protocol:
///   - Both arms use identical seeds, population size, generation limit, and
///     time budget per instance.
///   - Arm A: ConstructionMode::Random (CB-000 baseline)
///   - Arm B: ConstructionMode::GreedyLoadAware (RC-001 candidate)
///   - Primary metric: official ROADEF objective (lower is better)
///   - Explanatory metric: IFR (generation 0 feasibility rate)
///   - Secondary metrics: valid, runtime_ms, generations_run, n_eval,
///                        gen0_best_obj, gen0_mean_obj, gen0_feasible_count
///
/// Acceptance criterion (primary gate):
///   Arm B wins on official ROADEF objective on ≥ 2/3 of instances.
///   IFR improvement is supporting evidence, not a hard gate.
///   Regression check: arm B runtime must not exceed arm A by > 2×.
///
/// Statistical note:
///   This campaign uses a single fixed seed (42). Evolutionary algorithms have
///   high variance; a single seed is acceptable for an engineering gate but not
///   for publication-quality evidence. Multi-seed experiments (e.g. seeds 42–51)
///   are recommended before any paper submission.
///
/// Parallelism note:
///   Rayon parallelism is across instances (each instance runs on a separate
///   Rayon worker thread). Within each instance, Arm A and Arm B run sequentially
///   (A then B) on the same worker. This is correct for reproducibility: both
///   arms use the same fixed seed and the same evaluator state.
///
/// Outputs:
///   benchmarks/roadef/rc001/rc001_ab_report.json
///   benchmarks/roadef/rc001/RC001_AB_REPORT.md
///
/// Classification: Competition Engineering campaign binary (RC-001).

use std::collections::HashMap;
use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use chrono::Utc;

use roadef::evaluator::RoadefEvaluator;
use roadef::models::Network;
use roadef::moga_impl::{
    RoadefGenomeFactory, RoadefFitnessEvaluator, RoadefMutator, RoadefCrossover,
    EvolutionRunConfig, EvolutionRunResult, run_roadef_evolution,
    ConstructionMode, GreedyConstructorData,
};
use roadef::telemetry::{NullTelemetrySink, ComparatorMode};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const REPORT_DIR: &str = "benchmarks/roadef/rc001";

const POPULATION_SIZE: usize = 50;
const GENERATION_LIMIT: usize = 500;
const ELITE_COUNT: usize = 5;
const CAMPAIGN_ID: &str = "rc001_ab_v2.3";

// Fixed seed for reproducibility — both arms use the same seed.
const FIXED_SEED: u64 = 42;

// Adaptive time budget per instance (same formula as campaign.rs)
// v1.5: MIN lowered from 30s to 10s — small instances (setA-01..05) were hitting the
// 30s floor and spending the entire budget on no-improvement iterations. 10s is
// sufficient for the greedy constructor validation (IFR is measured at gen-0, before
// any evolution, so the time budget only affects the evolutionary phase).
const MIN_BUDGET_SECS: u64 = 10;
const MAX_BUDGET_SECS: u64 = 300;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArmResult {
    arm: String,
    instance: String,
    num_demands: usize,
    num_nodes: usize,
    num_links: usize,
    num_time_slots: usize,
    initial_feasibility_rate: f64,
    /// Number of feasible individuals in generation 0 (= IFR × pop_size).
    gen0_feasible_count: usize,
    /// Best objective value in generation 0 (before any evolution).
    /// f64::INFINITY serialised as null in JSON when no valid individual exists.
    gen0_best_obj: f64,
    /// Mean objective across all valid individuals in generation 0.
    /// f64::INFINITY serialised as null in JSON when no valid individual exists.
    gen0_mean_obj: f64,
    best_obj: f64,
    valid: bool,
    runtime_ms: u128,
    generations_run: usize,
    /// Derived: population_size + generations_run × (population_size − elite_count).
    /// Counts total fitness evaluations executed (initial population + per-generation offspring).
    n_eval: usize,
    termination_reason: String,
    /// Invariant violation flag: valid=false AND obj=inf AND IFR=1.0 is a potential
    /// correctness failure (constructor counts genomes as feasible but evaluator rejects them).
    /// Updated interpretation: may also indicate evolution operator corruption of a valid
    /// greedy genome (IFR=1.0 confirms constructor succeeded; valid=false confirms evolution broke it).
    invariant_violation_suspected: bool,
    /// Number of distinct objective values among valid gen-0 individuals (rounded to 4dp).
    /// Low value indicates population diversity collapse from the greedy constructor.
    gen0_unique_obj_count: usize,
    /// Number of gen-0 individuals with identical waypoint vectors to at least one other.
    /// High value confirms diversity collapse hypothesis.
    gen0_duplicate_genome_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstanceComparison {
    instance: String,
    arm_a_ifr: f64,
    arm_b_ifr: f64,
    ifr_delta: f64,
    arm_a_gen0_best_obj: f64,
    arm_b_gen0_best_obj: f64,
    arm_a_obj: f64,
    arm_b_obj: f64,
    obj_delta: f64,
    arm_a_valid: bool,
    arm_b_valid: bool,
    arm_b_better_obj: bool,
    arm_b_better_ifr: bool,
    arm_a_invariant_violation: bool,
    arm_b_invariant_violation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AbReport {
    campaign_id: String,
    timestamp: String,
    hypothesis: String,
    arm_a_description: String,
    arm_b_description: String,
    statistical_note: String,
    total_instances: usize,
    arm_a_mean_ifr: f64,
    arm_b_mean_ifr: f64,
    ifr_improvement: f64,
    arm_a_valid_count: usize,
    arm_b_valid_count: usize,
    arm_b_better_obj_count: usize,
    arm_b_better_ifr_count: usize,
    invariant_violation_count_a: usize,
    invariant_violation_count_b: usize,
    comparisons: Vec<InstanceComparison>,
    arm_a_results: Vec<ArmResult>,
    arm_b_results: Vec<ArmResult>,
}

// ---------------------------------------------------------------------------
// Instance discovery
// ---------------------------------------------------------------------------

fn discover_instances() -> Vec<(String, String, String, String)> {
    let mut instances = Vec::new();
    for i in 1..=20 {
        let name = format!("setA-{:02}", i);
        let net      = format!("{}/{}-net.json",      INSTANCE_DIR, name);
        let tm       = format!("{}/{}-tm.json",       INSTANCE_DIR, name);
        let scenario = format!("{}/{}-scenario.json", INSTANCE_DIR, name);
        if Path::new(&net).exists() && Path::new(&tm).exists() && Path::new(&scenario).exists() {
            instances.push((name, net, tm, scenario));
        }
    }
    instances
}

// ---------------------------------------------------------------------------
// Build GreedyConstructorData from loaded instance components.
// ---------------------------------------------------------------------------
fn build_greedy_data(
    net: &Network,
    evaluator: Arc<RoadefEvaluator>,
) -> Arc<GreedyConstructorData> {
    // Build demands_by_volume: (demand_index, src, dst, max_volume_across_slots)
    let mut demands_by_volume: Vec<(usize, u64, u64, f64)> = evaluator.tm.demands
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let max_vol = d.v.iter().cloned().fold(0.0_f64, f64::max);
            (i, d.s, d.t, max_vol)
        })
        .collect();
    // Sort descending by volume (greedy processes highest-volume demands first).
    demands_by_volume.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    // Build arc capacity map keyed by directed arc ID (same ID space as arc_flows from compute_loads).
    // CORRECTNESS FIX (RC-001A): Previously this was built from net.links (undirected link IDs),
    // but compute_loads returns arc_flows keyed by evaluator.graph.arcs (directed arc IDs).
    // On topologies where arc IDs ≠ link IDs, link_capacity.get(arc_id) returned None,
    // cap defaulted to 1.0, and sat = flow / 1.0 = flow — causing max_sat = 22.766 on setA-05.
    // Fix: use evaluator.graph.arcs to build the capacity map in the same ID space as arc_flows.
    let link_capacity: HashMap<u64, f64> = evaluator.graph.arcs.iter()
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

// ---------------------------------------------------------------------------
// Run one arm for one instance.
// ---------------------------------------------------------------------------
fn run_arm(
    arm_name: &str,
    instance_name: &str,
    factory: &RoadefGenomeFactory,
    fitness_eval: &RoadefFitnessEvaluator,
    mutator: &RoadefMutator,
    crossover: &RoadefCrossover,
    budget_secs: u64,
    num_demands: usize,
    num_nodes: usize,
    num_links: usize,
    num_time_slots: usize,
) -> ArmResult {
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
        max_runtime: Some(std::time::Duration::from_secs(budget_secs)),
        comparator_mode: ComparatorMode::Scalar,
        peak_demand_set: None,
    };

    let mut log_buf: Box<dyn std::io::Write> = Box::new(std::io::sink());
    let result: EvolutionRunResult = run_roadef_evolution(
        factory, fitness_eval, mutator, crossover,
        &evo_config, instance_name, &mut *log_buf, &mut NullTelemetrySink,
    );

    // n_eval: initial population (POPULATION_SIZE evals) + per-generation offspring
    // (POPULATION_SIZE - ELITE_COUNT evals per generation, since elites are carried over).
    let n_eval = POPULATION_SIZE
        + result.generations_run * (POPULATION_SIZE.saturating_sub(ELITE_COUNT));

    // Invariant violation detection (issue #9 from review):
    // IFR=1.0 AND valid=false AND obj=inf is a potential correctness failure.
    // The constructor counted all genomes as feasible but the evaluator rejected them all.
    // This may indicate a mismatch between constructor feasibility and evaluator feasibility,
    // a waypoint conversion bug, or a bug in the GreedyLoadAware path generation.
    let invariant_violation_suspected =
        result.initial_feasibility_rate >= 1.0
        && !result.valid
        && !result.best_obj.is_finite();

    ArmResult {
        arm: arm_name.to_string(),
        instance: instance_name.to_string(),
        num_demands,
        num_nodes,
        num_links,
        num_time_slots,
        initial_feasibility_rate: result.initial_feasibility_rate,
        gen0_feasible_count: result.gen0_feasible_count,
        gen0_best_obj: result.gen0_best_obj,
        gen0_mean_obj: result.gen0_mean_obj,
        best_obj: result.best_obj,
        valid: result.valid,
        runtime_ms: result.runtime_ms,
        generations_run: result.generations_run,
        n_eval,
        termination_reason: result.termination_reason,
        invariant_violation_suspected,
        gen0_unique_obj_count: result.gen0_unique_obj_count,
        gen0_duplicate_genome_count: result.gen0_duplicate_genome_count,
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let campaign_start = Instant::now();
    eprintln!("=== RC-001 A/B Campaign: Load-Aware Greedy Constructor ===");
    eprintln!("Campaign ID: {}", CAMPAIGN_ID);
    eprintln!("Arm A: ConstructionMode::Random (CB-000 baseline)");
    eprintln!("Arm B: ConstructionMode::GreedyLoadAware (RC-001 candidate)");
    eprintln!("Seed: {}  Population: {}  Generations: {}  Elite: {}",
        FIXED_SEED, POPULATION_SIZE, GENERATION_LIMIT, ELITE_COUNT);
    eprintln!("Note: single seed — engineering gate only, not publication-quality evidence.");
    eprintln!();

    let instances = discover_instances();
    let total = instances.len();
    eprintln!("Discovered {} instances", total);
    eprintln!("Execution: serial (one instance at a time) for clean logs and reproducible diagnostics.");
    eprintln!();

    // Create output directory at startup so incremental writes work even if interrupted.
    let out_dir = "benchmarks/roadef/rc001";
    fs::create_dir_all(out_dir).expect("failed to create output directory");
    let json_path = format!("{}/rc001_ab_report.json", out_dir);
    let md_path = format!("{}/RC001_AB_REPORT.md", out_dir);
    let _ = &md_path; // used in write_reports() at campaign end

    // ---------------------------------------------------------------------------
    // Serial execution: one instance at a time for clean logs and no shared-state risk.
    // After each instance completes, results are appended to the incremental JSON file
    // and the Markdown summary is rewritten. This ensures partial results are preserved
    // even if the campaign is interrupted before all instances complete.
    // ---------------------------------------------------------------------------
    let mut pairs: Vec<(ArmResult, ArmResult)> = Vec::new();

    for (name, net_path, tm_path, scenario_path) in &instances {
        let pair = (|| -> Option<(ArmResult, ArmResult)> {
            // Load instance.
            let net = match roadef::loader::load_network(net_path) {
                Ok(n) => n,
                Err(e) => { eprintln!("[{}] ERROR loading network: {}", name, e); return None; }
            };
            let tm = match roadef::loader::load_traffic_matrix(tm_path) {
                Ok(t) => t,
                Err(e) => { eprintln!("[{}] ERROR loading TM: {}", name, e); return None; }
            };
            let scenario = match roadef::loader::load_scenario(scenario_path) {
                Ok(s) => s,
                Err(e) => { eprintln!("[{}] ERROR loading scenario: {}", name, e); return None; }
            };

            let num_demands = tm.demands.len();
            let num_time_slots = tm.num_time_slots;
            let num_nodes = net.nodes.len();
            let num_links = net.links.len();
            let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

            // Adaptive time budget.
            let raw_budget_ms = (num_demands as u64) * (num_links as u64) / 2;
            let budget_secs = raw_budget_ms.clamp(MIN_BUDGET_SECS * 1000, MAX_BUDGET_SECS * 1000) / 1000;
            eprintln!("[{}] nodes={} links={} demands={} slots={} budget={}s — starting",
                name, num_nodes, num_links, num_demands, num_time_slots, budget_secs);

            // Build evaluator (shared between both arms for this instance).
            let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
            let fitness_eval = RoadefFitnessEvaluator { evaluator: Arc::clone(&evaluator) };
            let mutator = RoadefMutator { node_ids: node_ids.clone() };
            let crossover = RoadefCrossover;

            // Arm A: CB-000 Random constructor.
            let factory_a = RoadefGenomeFactory {
                num_demands,
                num_time_slots,
                node_ids: node_ids.clone(),
                mode: ConstructionMode::Random,
                greedy_data: None,
            };
            let result_a = run_arm(
                "A_Random", name, &factory_a, &fitness_eval, &mutator, &crossover,
                budget_secs, num_demands, num_nodes, num_links, num_time_slots,
            );

            // Arm B: RC-001 Greedy constructor.
            let greedy_data = build_greedy_data(&net, Arc::clone(&evaluator));
            let factory_b = RoadefGenomeFactory {
                num_demands,
                num_time_slots,
                node_ids: node_ids.clone(),
                mode: ConstructionMode::GreedyLoadAware,
                greedy_data: Some(greedy_data),
            };
            let result_b = run_arm(
                "B_GreedyLoadAware", name, &factory_b, &fitness_eval, &mutator, &crossover,
                budget_secs, num_demands, num_nodes, num_links, num_time_slots,
            );

            let inv_flag_a = if result_a.invariant_violation_suspected { " ⚠INVARIANT" } else { "" };
            let inv_flag_b = if result_b.invariant_violation_suspected { " ⚠INVARIANT" } else { "" };
            eprintln!(
                "[{}]\n  A: IFR={:.3} g0best={:.4} g0uniq={} g0dup={} obj={:.4} valid={} {}ms{}\n  \
                 B: IFR={:.3} g0best={:.4} g0uniq={} g0dup={} obj={:.4} valid={} {}ms{}\n  \
                 ΔIFR={:+.3} Δobj={:+.4}",
                name,
                result_a.initial_feasibility_rate, result_a.gen0_best_obj,
                result_a.gen0_unique_obj_count, result_a.gen0_duplicate_genome_count,
                result_a.best_obj, result_a.valid, result_a.runtime_ms, inv_flag_a,
                result_b.initial_feasibility_rate, result_b.gen0_best_obj,
                result_b.gen0_unique_obj_count, result_b.gen0_duplicate_genome_count,
                result_b.best_obj, result_b.valid, result_b.runtime_ms, inv_flag_b,
                result_b.initial_feasibility_rate - result_a.initial_feasibility_rate,
                result_b.best_obj - result_a.best_obj,
            );

            Some((result_a, result_b))
        })();

        if let Some(pair) = pair {
            pairs.push(pair);
            // Incremental write: persist all completed pairs after each instance.
            // This ensures partial results survive interruption.
            let partial_a: Vec<&ArmResult> = pairs.iter().map(|(a, _)| a).collect();
            let partial_b: Vec<&ArmResult> = pairs.iter().map(|(_, b)| b).collect();
            let partial_report = serde_json::json!({
                "campaign_id": CAMPAIGN_ID,
                "instances_completed": pairs.len(),
                "instances_total": total,
                "arm_a": partial_a,
                "arm_b": partial_b,
            });
            if let Ok(json_str) = serde_json::to_string_pretty(&partial_report) {
                if let Ok(mut f) = fs::File::create(&json_path) {
                    use std::io::Write;
                    let _ = f.write_all(json_str.as_bytes());
                }
            }
            eprintln!("[progress] {}/{} instances complete — partial results written to {}",
                pairs.len(), total, json_path);
        }
    }

    // pairs is already in insertion order (instances processed serially in discover_instances() order).
    // Sort by name for deterministic report ordering regardless of filesystem order.
    pairs.sort_by(|a, b| a.0.instance.cmp(&b.0.instance));

    let arm_a_results: Vec<ArmResult> = pairs.iter().map(|(a, _)| a.clone()).collect();
    let arm_b_results: Vec<ArmResult> = pairs.iter().map(|(_, b)| b.clone()).collect();

    // ---------------------------------------------------------------------------
    // Aggregate statistics
    // ---------------------------------------------------------------------------
    let n = arm_a_results.len();
    let arm_a_mean_ifr = if n > 0 {
        arm_a_results.iter().map(|r| r.initial_feasibility_rate).sum::<f64>() / n as f64
    } else { 0.0 };
    let arm_b_mean_ifr = if n > 0 {
        arm_b_results.iter().map(|r| r.initial_feasibility_rate).sum::<f64>() / n as f64
    } else { 0.0 };
    let ifr_improvement = arm_b_mean_ifr - arm_a_mean_ifr;

    let arm_a_valid_count = arm_a_results.iter().filter(|r| r.valid).count();
    let arm_b_valid_count = arm_b_results.iter().filter(|r| r.valid).count();

    let invariant_violation_count_a = arm_a_results.iter().filter(|r| r.invariant_violation_suspected).count();
    let invariant_violation_count_b = arm_b_results.iter().filter(|r| r.invariant_violation_suspected).count();

    let mut comparisons: Vec<InstanceComparison> = Vec::new();
    let mut arm_b_better_obj_count = 0usize;
    let mut arm_b_better_ifr_count = 0usize;

    for (a, b) in arm_a_results.iter().zip(arm_b_results.iter()) {
        let arm_b_better_obj = b.best_obj < a.best_obj;
        let arm_b_better_ifr = b.initial_feasibility_rate > a.initial_feasibility_rate;
        if arm_b_better_obj { arm_b_better_obj_count += 1; }
        if arm_b_better_ifr { arm_b_better_ifr_count += 1; }
        comparisons.push(InstanceComparison {
            instance: a.instance.clone(),
            arm_a_ifr: a.initial_feasibility_rate,
            arm_b_ifr: b.initial_feasibility_rate,
            ifr_delta: b.initial_feasibility_rate - a.initial_feasibility_rate,
            arm_a_gen0_best_obj: a.gen0_best_obj,
            arm_b_gen0_best_obj: b.gen0_best_obj,
            arm_a_obj: a.best_obj,
            arm_b_obj: b.best_obj,
            obj_delta: b.best_obj - a.best_obj,
            arm_a_valid: a.valid,
            arm_b_valid: b.valid,
            arm_b_better_obj,
            arm_b_better_ifr,
            arm_a_invariant_violation: a.invariant_violation_suspected,
            arm_b_invariant_violation: b.invariant_violation_suspected,
        });
    }

    // ---------------------------------------------------------------------------
    // Print summary
    // ---------------------------------------------------------------------------
    eprintln!("=== RC-001 Summary ===");
    eprintln!("Instances: {}", n);
    eprintln!("Arm A (Random)  mean IFR: {:.3}  valid: {}/{}", arm_a_mean_ifr, arm_a_valid_count, n);
    eprintln!("Arm B (Greedy)  mean IFR: {:.3}  valid: {}/{}", arm_b_mean_ifr, arm_b_valid_count, n);
    eprintln!("IFR improvement: {:+.3}", ifr_improvement);
    eprintln!("Arm B better obj: {}/{}", arm_b_better_obj_count, n);
    eprintln!("Arm B better IFR: {}/{}", arm_b_better_ifr_count, n);
    if invariant_violation_count_a > 0 || invariant_violation_count_b > 0 {
        eprintln!("⚠ INVARIANT VIOLATIONS: A={} B={} — investigate before trusting results",
            invariant_violation_count_a, invariant_violation_count_b);
    }
    eprintln!();

    // ---------------------------------------------------------------------------
    // Write JSON report
    // ---------------------------------------------------------------------------
    let report = AbReport {
        campaign_id: CAMPAIGN_ID.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        hypothesis: "GreedyLoadAware constructor raises IFR and improves final objective vs Random (CB-000)".to_string(),
        arm_a_description: "ConstructionMode::Random — CB-000 baseline (70% ECMP default, 30% random waypoint)".to_string(),
        arm_b_description: "ConstructionMode::GreedyLoadAware — RC-001 candidate (RP-401C volume-sorted load-aware Dijkstra)".to_string(),
        statistical_note: format!(
            "Single seed ({}). Acceptable for engineering gate; multi-seed experiments required for publication.",
            FIXED_SEED
        ),
        total_instances: n,
        arm_a_mean_ifr,
        arm_b_mean_ifr,
        ifr_improvement,
        arm_a_valid_count,
        arm_b_valid_count,
        arm_b_better_obj_count,
        arm_b_better_ifr_count,
        invariant_violation_count_a,
        invariant_violation_count_b,
        comparisons,
        arm_a_results: arm_a_results.clone(),
        arm_b_results: arm_b_results.clone(),
    };

    let _ = fs::create_dir_all(REPORT_DIR);
    let json_path = format!("{}/rc001_ab_report.json", REPORT_DIR);
    if let Ok(f) = fs::File::create(&json_path) {
        let _ = serde_json::to_writer_pretty(BufWriter::new(f), &report);
        eprintln!("JSON report: {}", json_path);
    }

    // ---------------------------------------------------------------------------
    // Write Markdown report
    // ---------------------------------------------------------------------------
    let md_path = format!("{}/RC001_AB_REPORT.md", REPORT_DIR);
    let mut md = String::new();
    md.push_str("# RC-001 A/B Report: Load-Aware Greedy Constructor\n\n");
    md.push_str(&format!("**Campaign:** {}  \n", CAMPAIGN_ID));
    md.push_str(&format!("**Timestamp:** {}  \n", Utc::now().to_rfc3339()));
    md.push_str(&format!("**Seed:** {}  Population: {}  Generations: {}  Elite: {}\n\n",
        FIXED_SEED, POPULATION_SIZE, GENERATION_LIMIT, ELITE_COUNT));
    md.push_str("> **Statistical note:** Single seed. Acceptable for engineering gate; ");
    md.push_str("multi-seed experiments (e.g. seeds 42–51) required before paper submission.\n\n");

    md.push_str("## Hypothesis\n\n");
    md.push_str("The RP-401C load-aware greedy constructor (volume-sorted, additive saturation penalty Dijkstra) ");
    md.push_str("raises the Initial Feasibility Rate (IFR) of generation 0 compared to the CB-000 random constructor, ");
    md.push_str("thereby increasing EEB and improving the final ROADEF objective.\n\n");
    md.push_str("**EEB target:** IFR ↑ (Construction subsystem)  \n");
    md.push_str("**CB-000 baseline:** mean IFR = 10.6%, 6/20 instances with IFR = 0%\n\n");

    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Arm A (Random / CB-000) | Arm B (Greedy / RC-001) | Delta |\n");
    md.push_str("|--------|------------------------|------------------------|-------|\n");
    md.push_str(&format!("| Mean IFR | {:.3} | {:.3} | {:+.3} |\n",
        arm_a_mean_ifr, arm_b_mean_ifr, ifr_improvement));
    md.push_str(&format!("| Valid instances | {}/{} | {}/{} | {:+} |\n",
        arm_a_valid_count, n, arm_b_valid_count, n,
        arm_b_valid_count as i64 - arm_a_valid_count as i64));
    md.push_str(&format!("| Arm B better obj | — | {}/{} | — |\n", arm_b_better_obj_count, n));
    md.push_str(&format!("| Arm B better IFR | — | {}/{} | — |\n", arm_b_better_ifr_count, n));
    if invariant_violation_count_a > 0 || invariant_violation_count_b > 0 {
        md.push_str(&format!("| ⚠ Invariant violations | {} | {} | — |\n\n",
            invariant_violation_count_a, invariant_violation_count_b));
    } else {
        md.push_str("\n");
    }

    // Invariant violation warning block
    if invariant_violation_count_a > 0 || invariant_violation_count_b > 0 {
        md.push_str("## ⚠ Invariant Violation Warning\n\n");
        md.push_str("One or more instances produced `IFR=1.0, valid=false, obj=inf`. ");
        md.push_str("This combination is a potential correctness failure: the constructor counted all genomes ");
        md.push_str("as feasible but the evaluator rejected them all. Possible causes:\n\n");
        md.push_str("- IFR measures something different from evaluator feasibility.\n");
        md.push_str("- The waypoint conversion produces an invalid representation.\n");
        md.push_str("- The evaluator rejects genomes that the constructor counts as feasible.\n");
        md.push_str("- A bug in the GreedyLoadAware path generation.\n\n");
        md.push_str("**Investigate before trusting performance conclusions from affected instances.**\n\n");
    }

    md.push_str("## Per-Instance Results\n\n");
    md.push_str("| Instance | A IFR | B IFR | ΔIFR | A g0best | B g0best | A obj | B obj | Δobj | B better? | Flags |\n");
    md.push_str("|----------|-------|-------|------|----------|----------|-------|-------|------|-----------|-------|\n");
    for c in &report.comparisons {
        let better = if c.arm_b_better_obj { "✓ obj" } else if c.arm_b_better_ifr { "✓ IFR" } else { "✗" };
        let flags = match (c.arm_a_invariant_violation, c.arm_b_invariant_violation) {
            (true, true)  => "⚠A ⚠B",
            (true, false) => "⚠A",
            (false, true) => "⚠B",
            (false, false) => "",
        };
        let g0a = if c.arm_a_gen0_best_obj.is_finite() { format!("{:.4}", c.arm_a_gen0_best_obj) } else { "∞".to_string() };
        let g0b = if c.arm_b_gen0_best_obj.is_finite() { format!("{:.4}", c.arm_b_gen0_best_obj) } else { "∞".to_string() };
        md.push_str(&format!("| {} | {:.3} | {:.3} | {:+.3} | {} | {} | {:.4} | {:.4} | {:+.4} | {} | {} |\n",
            c.instance, c.arm_a_ifr, c.arm_b_ifr, c.ifr_delta,
            g0a, g0b,
            c.arm_a_obj, c.arm_b_obj, c.obj_delta, better, flags));
    }

    // ---------------------------------------------------------------------------
    // Acceptance criterion (primary gate: official ROADEF objective)
    // Arm B wins on ≥ 2/3 of instances → ACCEPTED.
    // IFR improvement is supporting evidence, not a hard gate.
    // Regression check: arm B mean runtime must not exceed arm A by > 2×.
    // ---------------------------------------------------------------------------
    let arm_a_mean_runtime = if n > 0 {
        arm_a_results.iter().map(|r| r.runtime_ms as f64).sum::<f64>() / n as f64
    } else { 0.0 };
    let arm_b_mean_runtime = if n > 0 {
        arm_b_results.iter().map(|r| r.runtime_ms as f64).sum::<f64>() / n as f64
    } else { 0.0 };
    let runtime_regression = arm_a_mean_runtime > 0.0 && arm_b_mean_runtime > arm_a_mean_runtime * 2.0;

    let threshold_2_3 = (n * 2 + 2) / 3; // ceiling of 2n/3
    let obj_gate_passed = arm_b_better_obj_count >= threshold_2_3;
    let has_correctness_failures = invariant_violation_count_b > 0;

    md.push_str("\n## Verdict\n\n");
    md.push_str("**Acceptance criterion:** Arm B wins on official ROADEF objective on ≥ 2/3 of instances.  \n");
    md.push_str("**IFR** is explanatory evidence, not a hard gate.  \n");
    md.push_str("**Regression check:** arm B mean runtime ≤ 2× arm A mean runtime.\n\n");
    md.push_str(&format!("- Arm B better obj: {}/{} (threshold: {}/{})\n",
        arm_b_better_obj_count, n, threshold_2_3, n));
    md.push_str(&format!("- IFR improvement: {:+.3} (explanatory)\n", ifr_improvement));
    md.push_str(&format!("- Runtime: A={:.0}ms  B={:.0}ms  regression={}\n",
        arm_a_mean_runtime, arm_b_mean_runtime, runtime_regression));
    md.push_str(&format!("- Invariant violations: A={}  B={}\n\n",
        invariant_violation_count_a, invariant_violation_count_b));

    if has_correctness_failures {
        md.push_str("**CORRECTNESS FAILURE — RETURN TO IMPLEMENTATION**\n\n");
        md.push_str(&format!(
            "RC-001 produced {} instance(s) with IFR=1.0, valid=false, obj=inf. \
             This is a constructor defect, not an optimisation result. \
             The benchmark cannot produce a fair acceptance/rejection decision while \
             correctness failures are present. RC-001 must return to the Implementation \
             stage for a bug fix and be re-benchmarked under the same lifecycle.\n",
            invariant_violation_count_b
        ));
    } else if obj_gate_passed && !runtime_regression {
        md.push_str("**ACCEPTED**\n\n");
        md.push_str("RC-001 improves the official ROADEF objective on ≥ 2/3 of instances without runtime regression. ");
        md.push_str("Recommend integrating GreedyLoadAware as the default construction mode for the RC integration branch.\n");
    } else if obj_gate_passed && runtime_regression {
        md.push_str("**ACCEPTED WITH CAUTION**\n\n");
        md.push_str("RC-001 improves objective on ≥ 2/3 of instances but shows runtime regression (> 2×). ");
        md.push_str("Investigate constructor cost before integration.\n");
    } else if arm_b_better_obj_count > 0 {
        md.push_str("**REJECTED**\n\n");
        md.push_str(&format!("RC-001 improves objective on only {}/{} instances (threshold: {}/{}). ",
            arm_b_better_obj_count, n, threshold_2_3, n));
        md.push_str("Hypothesis not confirmed at the required threshold. Retain Random constructor.\n");
    } else {
        md.push_str("**REJECTED**\n\n");
        md.push_str("RC-001 does not improve the official ROADEF objective on any instance. ");
        md.push_str("Hypothesis falsified. Retain Random constructor.\n");
    }
    md.push_str(&format!("\n*Total campaign runtime: {}ms*\n", campaign_start.elapsed().as_millis()));

    if let Ok(mut f) = fs::File::create(&md_path) {
        use std::io::Write;
        let _ = f.write_all(md.as_bytes());
        eprintln!("Markdown report: {}", md_path);
    }

    eprintln!("Done. Total runtime: {}ms", campaign_start.elapsed().as_millis());
}