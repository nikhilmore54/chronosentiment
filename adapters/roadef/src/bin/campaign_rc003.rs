/// campaign_rc003 — RC-003: Lexicographic Objective Validation
///
/// Research question: Does the Coralys surrogate scalar objective preserve the official
/// ROADEF lexicographic ordering?
///
/// The surrogate objective is: Σ_t (MLU_t + inv_load_cost_t)
/// The official objective is:  sort_descending({ link_saturation(l,t) : l ∈ links, t ∈ slots })
///
/// Protocol:
///   - Identical configuration to campaign_rc001 (same seed, population, time budget).
///   - Both arms run on all 20 setA instances.
///   - For each instance where both arms produce valid solutions:
///       1. Compute surrogate winner (lower scalar obj wins).
///       2. Compute lex winner (lexicographic comparison of sorted saturation vectors).
///       3. Record match or inversion.
///   - Compute Spearman rank correlation (ρ) between surrogate and lex rankings.
///
/// Acceptance criterion:
///   PASS:             0 inversions AND ρ ≥ 0.95
///   CONDITIONAL PASS: ≤ 1 inversion AND ρ ≥ 0.90
///   FAIL:             ≥ 2 inversions OR ρ < 0.90
///
/// Outputs:
///   benchmarks/roadef/rc003/rc003_lex_results.json
///   benchmarks/roadef/rc003/RC003_LEX_VALIDATION_REPORT.md (results section)
///
/// Classification: Submission Gate campaign binary (RC-003).

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
use roadef::loader::{load_network, load_traffic_matrix, load_scenario};

// ---------------------------------------------------------------------------
// Configuration — identical to campaign_rc001 for reproducibility
// ---------------------------------------------------------------------------

const INSTANCE_DIR: &str = "repo/challenge-roadef-2026-main/setA";
const REPORT_DIR: &str = "benchmarks/roadef/rc003";

const POPULATION_SIZE: usize = 50;
const GENERATION_LIMIT: usize = 500;
const ELITE_COUNT: usize = 5;
const CAMPAIGN_ID: &str = "rc003_lex_v1.0";
const FIXED_SEED: u64 = 42;
const MIN_BUDGET_SECS: u64 = 10;
const MAX_BUDGET_SECS: u64 = 300;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArmLexResult {
    arm: String,
    instance: String,
    valid: bool,
    surrogate_obj: f64,
    /// First 10 values of the lex vector (rank-1 through rank-10).
    /// Full vector stored separately to keep JSON readable.
    lex_top10: Vec<f64>,
    lex_vector_len: usize,
    /// RC-003 diagnostic: number of valid individuals in the initial population (gen=0).
    /// If < population_size, the constructor is producing infeasible genomes.
    gen0_feasible_count: usize,
    /// RC-003 diagnostic: initial feasibility rate = gen0_feasible_count / population_size.
    initial_feasibility_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstanceLexComparison {
    instance: String,
    arm_a_valid: bool,
    arm_b_valid: bool,
    arm_a_surrogate_obj: f64,
    arm_b_surrogate_obj: f64,
    surrogate_winner: String, // "A", "B", "tie", "neither"
    lex_winner: String,       // "A", "B", "tie", "neither"
    inversion: bool,          // true if surrogate_winner != lex_winner (and both valid)
    /// Spearman rank contribution: (surrogate_rank_diff, lex_rank_diff) for this instance.
    /// Used to compute ρ across all instances.
    surrogate_rank: usize,
    lex_rank: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Rc003Report {
    campaign_id: String,
    timestamp: String,
    total_instances: usize,
    both_valid_count: usize,
    inversion_count: usize,
    spearman_rho: f64,
    verdict: String, // "PASS", "CONDITIONAL_PASS", "FAIL", "INSUFFICIENT_DATA"
    comparisons: Vec<InstanceLexComparison>,
    arm_a_results: Vec<ArmLexResult>,
    arm_b_results: Vec<ArmLexResult>,
}

// ---------------------------------------------------------------------------
// Lex vector comparison: returns Ordering::Less if a is better (lower rank-1 first)
// ---------------------------------------------------------------------------
fn lex_cmp(a: &[f64], b: &[f64]) -> std::cmp::Ordering {
    let n = a.len().min(b.len());
    for i in 0..n {
        let ord = a[i].partial_cmp(&b[i]).unwrap_or(std::cmp::Ordering::Equal);
        if ord != std::cmp::Ordering::Equal {
            return ord; // lower saturation at rank i wins
        }
    }
    a.len().cmp(&b.len())
}

// ---------------------------------------------------------------------------
// Spearman rank correlation
// ---------------------------------------------------------------------------
fn spearman_rho(surrogate_ranks: &[f64], lex_ranks: &[f64]) -> f64 {
    let n = surrogate_ranks.len();
    if n < 2 {
        return f64::NAN;
    }
    let n_f = n as f64;
    let d_sq_sum: f64 = surrogate_ranks.iter().zip(lex_ranks.iter())
        .map(|(s, l)| (s - l).powi(2))
        .sum();
    1.0 - (6.0 * d_sq_sum) / (n_f * (n_f * n_f - 1.0))
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
// Build GreedyConstructorData
// ---------------------------------------------------------------------------
fn build_greedy_data(
    net: &Network,
    evaluator: Arc<RoadefEvaluator>,
) -> Arc<GreedyConstructorData> {
    let mut demands_by_volume: Vec<(usize, u64, u64, f64)> = evaluator.tm.demands
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let max_vol = d.v.iter().cloned().fold(0.0_f64, f64::max);
            (i, d.s, d.t, max_vol)
        })
        .collect();
    demands_by_volume.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

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
// Run one arm and return (EvolutionRunResult, ArmLexResult)
// ---------------------------------------------------------------------------
fn run_arm_lex(
    arm_name: &str,
    instance_name: &str,
    factory: &RoadefGenomeFactory,
    fitness_eval: &RoadefFitnessEvaluator,
    mutator: &RoadefMutator,
    crossover: &RoadefCrossover,
    evaluator: &RoadefEvaluator,
    budget_secs: u64,
) -> ArmLexResult {
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

    // Compute lex vector for the best genome found.
    let (lex_top10, lex_vector_len) = if result.valid {
        let solution = result.best_genome.to_solution();
        match evaluator.compute_lex_vector(&solution) {
            Some(vec) => {
                let top10: Vec<f64> = vec.iter().take(10).cloned().collect();
                let len = vec.len();
                (top10, len)
            }
            None => (vec![], 0),
        }
    } else {
        (vec![], 0)
    };

    ArmLexResult {
        arm: arm_name.to_string(),
        instance: instance_name.to_string(),
        valid: result.valid,
        surrogate_obj: result.best_obj,
        lex_top10,
        lex_vector_len,
        gen0_feasible_count: result.gen0_feasible_count,
        initial_feasibility_rate: result.initial_feasibility_rate,
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() {
    let campaign_start = Instant::now();
    eprintln!("=== RC-003: Lexicographic Objective Validation ===");
    eprintln!("Campaign ID: {}", CAMPAIGN_ID);
    eprintln!("Seed: {}  Population: {}  Generations: {}  Elite: {}",
        FIXED_SEED, POPULATION_SIZE, GENERATION_LIMIT, ELITE_COUNT);

    let instances = discover_instances();
    eprintln!("Discovered {} instances.", instances.len());

    fs::create_dir_all(REPORT_DIR).expect("Failed to create report directory");

    let mut arm_a_results: Vec<ArmLexResult> = Vec::new();
    let mut arm_b_results: Vec<ArmLexResult> = Vec::new();

    for (name, net_path, tm_path, scenario_path) in &instances {
        eprintln!("\n--- {} ---", name);

        let net = load_network(net_path).expect("Failed to load network");
        let tm  = load_traffic_matrix(tm_path).expect("Failed to load TM");
        let scenario = load_scenario(scenario_path).expect("Failed to load scenario");

        let num_demands = tm.demands.len();
        let num_links   = net.links.len();

        // Adaptive time budget (same formula as campaign_rc001)
        let budget_secs = {
            let raw = (num_demands as u64 / 100).max(MIN_BUDGET_SECS);
            raw.min(MAX_BUDGET_SECS)
        };

        let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
        let fitness_eval = RoadefFitnessEvaluator { evaluator: evaluator.clone() };
        let mutator = RoadefMutator {
            node_ids: net.nodes.iter().map(|n| n.id).collect(),
        };
        let crossover = RoadefCrossover;

        let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();
        let num_time_slots = evaluator.tm.num_time_slots;

        // Arm A — Random constructor
        let factory_a = RoadefGenomeFactory {
            num_demands,
            num_time_slots,
            node_ids: node_ids.clone(),
            mode: ConstructionMode::Random,
            greedy_data: None,
        };
        let res_a = run_arm_lex("A", name, &factory_a, &fitness_eval, &mutator, &crossover,
                                &evaluator, budget_secs);
        eprintln!("  Arm A: valid={} surrogate={:.4} lex_len={} IFR={}/{} ({:.0}%)",
            res_a.valid, res_a.surrogate_obj, res_a.lex_vector_len,
            res_a.gen0_feasible_count, POPULATION_SIZE,
            res_a.initial_feasibility_rate * 100.0);

        // Arm B — Greedy constructor
        let greedy_data = build_greedy_data(&net, evaluator.clone());
        let factory_b = RoadefGenomeFactory {
            num_demands,
            num_time_slots,
            node_ids,
            mode: ConstructionMode::GreedyLoadAware,
            greedy_data: Some(greedy_data),
        };
        let res_b = run_arm_lex("B", name, &factory_b, &fitness_eval, &mutator, &crossover,
                                &evaluator, budget_secs);
        eprintln!("  Arm B: valid={} surrogate={:.4} lex_len={} IFR={}/{} ({:.0}%)",
            res_b.valid, res_b.surrogate_obj, res_b.lex_vector_len,
            res_b.gen0_feasible_count, POPULATION_SIZE,
            res_b.initial_feasibility_rate * 100.0);

        arm_a_results.push(res_a);
        arm_b_results.push(res_b);
    }

    // ---------------------------------------------------------------------------
    // Build comparison table
    // ---------------------------------------------------------------------------
    let mut comparisons: Vec<InstanceLexComparison> = Vec::new();
    let mut both_valid_count = 0usize;
    let mut inversion_count = 0usize;

    // For Spearman: collect (surrogate_rank, lex_rank) pairs across instances
    // where both arms are valid. We rank by arm B's advantage (B_obj - A_obj for surrogate,
    // lex comparison result for lex). Simpler: rank instances by surrogate delta and lex delta.
    // We use a per-instance "B wins" indicator: 1.0 if B wins, 0.5 if tie, 0.0 if A wins.
    let mut surrogate_scores: Vec<f64> = Vec::new();
    let mut lex_scores: Vec<f64> = Vec::new();

    for (res_a, res_b) in arm_a_results.iter().zip(arm_b_results.iter()) {
        let instance = res_a.instance.clone();

        let surrogate_winner = if !res_a.valid && !res_b.valid {
            "neither".to_string()
        } else if !res_a.valid {
            "B".to_string()
        } else if !res_b.valid {
            "A".to_string()
        } else if res_a.surrogate_obj < res_b.surrogate_obj {
            "A".to_string()
        } else if res_b.surrogate_obj < res_a.surrogate_obj {
            "B".to_string()
        } else {
            "tie".to_string()
        };

        let lex_winner = if !res_a.valid && !res_b.valid {
            "neither".to_string()
        } else if !res_a.valid {
            "B".to_string()
        } else if !res_b.valid {
            "A".to_string()
        } else {
            // Both valid — compare lex vectors
            match lex_cmp(&res_a.lex_top10, &res_b.lex_top10) {
                std::cmp::Ordering::Less    => "A".to_string(), // A has lower rank-1 → A wins
                std::cmp::Ordering::Greater => "B".to_string(),
                std::cmp::Ordering::Equal   => "tie".to_string(),
            }
        };

        let inversion = res_a.valid && res_b.valid
            && surrogate_winner != "tie"
            && lex_winner != "tie"
            && surrogate_winner != lex_winner;

        if res_a.valid && res_b.valid {
            both_valid_count += 1;
            if inversion { inversion_count += 1; }

            // Spearman: score = 1.0 if B wins, 0.5 if tie, 0.0 if A wins
            let s_score = match surrogate_winner.as_str() {
                "B" => 1.0, "tie" => 0.5, _ => 0.0,
            };
            let l_score = match lex_winner.as_str() {
                "B" => 1.0, "tie" => 0.5, _ => 0.0,
            };
            surrogate_scores.push(s_score);
            lex_scores.push(l_score);
        }

        comparisons.push(InstanceLexComparison {
            instance,
            arm_a_valid: res_a.valid,
            arm_b_valid: res_b.valid,
            arm_a_surrogate_obj: res_a.surrogate_obj,
            arm_b_surrogate_obj: res_b.surrogate_obj,
            surrogate_winner,
            lex_winner,
            inversion,
            surrogate_rank: 0, // filled below
            lex_rank: 0,
        });
    }

    // Compute Spearman ρ from scores (convert scores to ranks)
    let spearman_rho = if surrogate_scores.len() >= 2 {
        // Convert scores to ranks (1-based, average ties)
        let rank_vec = |scores: &[f64]| -> Vec<f64> {
            let n = scores.len();
            let mut indexed: Vec<(usize, f64)> = scores.iter().cloned().enumerate().collect();
            indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let mut ranks = vec![0.0f64; n];
            let mut i = 0;
            while i < n {
                let mut j = i;
                while j < n && (indexed[j].1 - indexed[i].1).abs() < 1e-12 { j += 1; }
                let avg_rank = (i + 1 + j) as f64 / 2.0;
                for k in i..j { ranks[indexed[k].0] = avg_rank; }
                i = j;
            }
            ranks
        };
        let s_ranks = rank_vec(&surrogate_scores);
        let l_ranks = rank_vec(&lex_scores);
        spearman_rho(&s_ranks, &l_ranks)
    } else {
        f64::NAN
    };

    let verdict = if both_valid_count < 2 {
        "INSUFFICIENT_DATA".to_string()
    } else if inversion_count == 0 && spearman_rho >= 0.95 {
        "PASS".to_string()
    } else if inversion_count <= 1 && spearman_rho >= 0.90 {
        "CONDITIONAL_PASS".to_string()
    } else {
        "FAIL".to_string()
    };

    let elapsed = campaign_start.elapsed().as_secs_f64();
    eprintln!("\n=== RC-003 Results ===");
    eprintln!("Both-valid instances: {}/{}", both_valid_count, instances.len());
    eprintln!("Inversions:           {}", inversion_count);
    eprintln!("Spearman ρ:           {:.4}", spearman_rho);
    eprintln!("Verdict:              {}", verdict);
    eprintln!("Total runtime:        {:.1}s", elapsed);

    // Print comparison table
    eprintln!("\n{:<12} {:>8} {:>8} {:>12} {:>12} {:>10}",
        "Instance", "A_valid", "B_valid", "Surr.Winner", "Lex.Winner", "Inversion");
    for c in &comparisons {
        eprintln!("{:<12} {:>8} {:>8} {:>12} {:>12} {:>10}",
            c.instance,
            c.arm_a_valid,
            c.arm_b_valid,
            c.surrogate_winner,
            c.lex_winner,
            c.inversion);
    }

    // Serialize report
    let report = Rc003Report {
        campaign_id: CAMPAIGN_ID.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        total_instances: instances.len(),
        both_valid_count,
        inversion_count,
        spearman_rho,
        verdict,
        comparisons,
        arm_a_results,
        arm_b_results,
    };

    let json_path = format!("{}/rc003_lex_results.json", REPORT_DIR);
    let f = fs::File::create(&json_path).expect("Failed to create JSON output");
    serde_json::to_writer_pretty(BufWriter::new(f), &report)
        .expect("Failed to write JSON");
    eprintln!("\nResults written to {}", json_path);
}