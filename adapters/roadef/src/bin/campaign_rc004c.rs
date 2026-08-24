use std::collections::hash_map::DefaultHasher;
/// campaign_rc004c — RC-004C: Constructor Forensics
///
/// Question: Does the greedy constructor generate a sufficiently diverse and
/// feasible initial population?
///
/// Protocol:
///   - Generate N_INDIVIDUALS individuals using only the constructor (no evolution).
///   - For each individual record: id, valid, objective, edge_hash, route_hash, construct_ms.
///   - Compute: validity rate, unique edge hashes, unique route hashes, mean pairwise
///     edge overlap, objective std deviation, mean construction time.
///   - Apply pass/fail thresholds to produce a binary go/no-go decision.
///
/// Acceptance thresholds:
///   Validity rate    >= 95%  -> PASS
///   Unique edge %    >= 90%  -> PASS
///   Unique route %   >= 85%  -> PASS
///   Obj std dev      >  0    -> PASS (non-trivial diversity)
///
/// Runs on a representative set of instances (small, medium, large).
/// Output: benchmarks/roadef/rc004c/rc004c_constructor_forensics.json
///
/// Classification: RC-004C constructor forensics binary.
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::BufWriter;
use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use coralys_moga::traits::evaluator::{Evaluated, FitnessEvaluator, GenomeFactory};
use rand::rngs::StdRng;
use rand::SeedableRng;
use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::Network;
use roadef::moga_impl::{
    ConstructionMode, GreedyConstructorData, RoadefFitnessEvaluator, RoadefGenomeFactory,
};

const INSTANCE_DIR: &str = "repo/challenge-roadef-2026-main/setA";
const REPORT_DIR: &str = "benchmarks/roadef/rc004c";
const N_INDIVIDUALS: usize = 100;
const FIXED_SEED: u64 = 42;

// Representative instances: small, medium, large
const TARGET_INSTANCES: &[&str] = &["setA-01", "setA-07", "setA-10", "setA-13", "setA-18"];

// ---------------------------------------------------------------------------
// Hashing helpers
// ---------------------------------------------------------------------------

fn hash_genome(genome: &roadef::moga_impl::RoadefGenome) -> (u64, u64) {
    // waypoints: Vec<Vec<u64>> — one Vec<u64> per demand (waypoint node IDs)
    let mut edge_hasher = DefaultHasher::new();
    let mut route_hasher = DefaultHasher::new();
    for (d_idx, wps) in genome.waypoints.iter().enumerate() {
        d_idx.hash(&mut edge_hasher);
        d_idx.hash(&mut route_hasher);
        // edge_hash: sort waypoints (order-independent edge set)
        let mut sorted_wps = wps.clone();
        sorted_wps.sort();
        sorted_wps.hash(&mut edge_hasher);
        // route_hash: preserve order (full route structure)
        wps.hash(&mut route_hasher);
    }
    (edge_hasher.finish(), route_hasher.finish())
}

fn pairwise_edge_overlap_rate(hashes: &[u64]) -> f64 {
    let n = hashes.len();
    if n < 2 {
        return 0.0;
    }
    let mut counts: HashMap<u64, usize> = HashMap::new();
    for &h in hashes {
        *counts.entry(h).or_insert(0) += 1;
    }
    let duplicate_count: usize = counts.values().filter(|&&c| c > 1).map(|&c| c - 1).sum();
    duplicate_count as f64 / n as f64
}

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
struct IndividualRecord {
    id: usize,
    valid: bool,
    surrogate_obj: f64,
    edge_hash: u64,
    route_hash: u64,
    construct_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct InstanceForensics {
    instance: String,
    n_individuals: usize,
    n_valid: usize,
    validity_rate: f64,
    n_unique_edge_hashes: usize,
    unique_edge_pct: f64,
    n_unique_route_hashes: usize,
    unique_route_pct: f64,
    mean_pairwise_edge_overlap: f64,
    obj_mean: f64,
    obj_std: f64,
    obj_min: f64,
    obj_max: f64,
    mean_construct_ms: f64,
    total_construct_ms: u64,
    verdict: String,
    individuals: Vec<IndividualRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Rc004cReport {
    campaign_id: String,
    timestamp: String,
    n_individuals: usize,
    seed: u64,
    instances: Vec<InstanceForensics>,
    overall_verdict: String,
}

// ---------------------------------------------------------------------------
// Build GreedyConstructorData
// ---------------------------------------------------------------------------
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

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() {
    eprintln!("=== RC-004C: Constructor Forensics ===");
    eprintln!("Individuals per instance: {}", N_INDIVIDUALS);
    eprintln!("Instances: {:?}", TARGET_INSTANCES);
    eprintln!("Seed: {}", FIXED_SEED);

    fs::create_dir_all(REPORT_DIR).expect("Failed to create report dir");

    let mut instance_results: Vec<InstanceForensics> = Vec::new();

    for instance_name in TARGET_INSTANCES {
        let net_path = format!("{}/{}-net.json", INSTANCE_DIR, instance_name);
        let tm_path = format!("{}/{}-tm.json", INSTANCE_DIR, instance_name);
        let scenario_path = format!("{}/{}-scenario.json", INSTANCE_DIR, instance_name);

        eprintln!("\n--- {} ---", instance_name);

        let net = load_network(&net_path).expect("load net");
        let tm = load_traffic_matrix(&tm_path).expect("load tm");
        let scenario = load_scenario(&scenario_path).expect("load scenario");

        let num_demands = tm.demands.len();
        let num_time_slots = tm.num_time_slots;
        let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

        let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
        let fitness_eval = RoadefFitnessEvaluator {
            evaluator: evaluator.clone(),
            l2_cache: None,
        };
        let greedy_data = build_greedy_data(&net, evaluator.clone());

        let factory = RoadefGenomeFactory {
            num_demands,
            num_time_slots,
            node_ids,
            mode: ConstructionMode::GreedyLoadAware,
            greedy_data: Some(greedy_data),
        };

        let mut rng = StdRng::seed_from_u64(FIXED_SEED);
        let mut records: Vec<IndividualRecord> = Vec::with_capacity(N_INDIVIDUALS);

        for id in 0..N_INDIVIDUALS {
            let t0 = std::time::Instant::now();
            let genome = factory.create(&mut rng);
            let construct_ms = t0.elapsed().as_millis() as u64;

            let ev = fitness_eval.evaluate(
                &genome,
                &coralys_moga::runtime::optimization::metric::MetricReport::default(),
            );
            let (edge_hash, route_hash) = hash_genome(&genome);

            records.push(IndividualRecord {
                id,
                valid: ev.is_valid(),
                surrogate_obj: if ev.is_valid() {
                    ev.fitness()
                } else {
                    f64::INFINITY
                },
                edge_hash,
                route_hash,
                construct_ms,
            });

            if id % 10 == 0 {
                eprintln!(
                    "  [{:3}/{}] valid={} construct_ms={}",
                    id + 1,
                    N_INDIVIDUALS,
                    ev.is_valid(),
                    construct_ms
                );
            }
        }

        // Compute statistics
        let n_valid = records.iter().filter(|r| r.valid).count();
        let validity_rate = n_valid as f64 / N_INDIVIDUALS as f64;

        let edge_hashes: Vec<u64> = records.iter().map(|r| r.edge_hash).collect();
        let route_hashes: Vec<u64> = records.iter().map(|r| r.route_hash).collect();

        let unique_edge_hashes: HashSet<u64> = edge_hashes.iter().cloned().collect();
        let unique_route_hashes: HashSet<u64> = route_hashes.iter().cloned().collect();

        let n_unique_edge = unique_edge_hashes.len();
        let n_unique_route = unique_route_hashes.len();
        let unique_edge_pct = n_unique_edge as f64 / N_INDIVIDUALS as f64;
        let unique_route_pct = n_unique_route as f64 / N_INDIVIDUALS as f64;
        let mean_pairwise_overlap = pairwise_edge_overlap_rate(&edge_hashes);

        let valid_objs: Vec<f64> = records
            .iter()
            .filter(|r| r.valid && r.surrogate_obj.is_finite())
            .map(|r| r.surrogate_obj)
            .collect();

        let (obj_mean, obj_std, obj_min, obj_max) = if valid_objs.is_empty() {
            (f64::NAN, f64::NAN, f64::NAN, f64::NAN)
        } else {
            let mean = valid_objs.iter().sum::<f64>() / valid_objs.len() as f64;
            let variance = valid_objs.iter().map(|x| (x - mean).powi(2)).sum::<f64>()
                / valid_objs.len() as f64;
            let std = variance.sqrt();
            let min = valid_objs.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = valid_objs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (mean, std, min, max)
        };

        let total_construct_ms: u64 = records.iter().map(|r| r.construct_ms).sum();
        let mean_construct_ms = total_construct_ms as f64 / N_INDIVIDUALS as f64;

        let verdict = if N_INDIVIDUALS < 10 {
            "INSUFFICIENT_DATA".to_string()
        } else {
            let pass_validity = validity_rate >= 0.95;
            let pass_edge = unique_edge_pct >= 0.90;
            let pass_route = unique_route_pct >= 0.85;
            let pass_std = obj_std > 0.0 || valid_objs.len() < 2;
            if pass_validity && pass_edge && pass_route && pass_std {
                "PASS".to_string()
            } else {
                "FAIL".to_string()
            }
        };

        eprintln!(
            "  Valid:          {}/{} ({:.1}%)",
            n_valid,
            N_INDIVIDUALS,
            validity_rate * 100.0
        );
        eprintln!(
            "  Unique edges:   {}/{} ({:.1}%)",
            n_unique_edge,
            N_INDIVIDUALS,
            unique_edge_pct * 100.0
        );
        eprintln!(
            "  Unique routes:  {}/{} ({:.1}%)",
            n_unique_route,
            N_INDIVIDUALS,
            unique_route_pct * 100.0
        );
        eprintln!("  Overlap:        {:.1}%", mean_pairwise_overlap * 100.0);
        eprintln!("  Obj mean/std:   {:.4} / {:.4}", obj_mean, obj_std);
        eprintln!(
            "  Construct ms:   mean={:.0} total={}ms",
            mean_construct_ms, total_construct_ms
        );
        eprintln!("  Verdict:        {}", verdict);

        instance_results.push(InstanceForensics {
            instance: instance_name.to_string(),
            n_individuals: N_INDIVIDUALS,
            n_valid,
            validity_rate,
            n_unique_edge_hashes: n_unique_edge,
            unique_edge_pct,
            n_unique_route_hashes: n_unique_route,
            unique_route_pct,
            mean_pairwise_edge_overlap: mean_pairwise_overlap,
            obj_mean,
            obj_std,
            obj_min,
            obj_max,
            mean_construct_ms,
            total_construct_ms,
            verdict,
            individuals: records,
        });
    }

    let overall_verdict = if instance_results.iter().all(|r| r.verdict == "PASS") {
        "PASS — constructor is healthy; stop investigating before submission".to_string()
    } else if instance_results.iter().any(|r| r.verdict == "FAIL") {
        "FAIL — constructor diversity or validity below threshold; investigate post-submission"
            .to_string()
    } else {
        "INSUFFICIENT_DATA".to_string()
    };

    eprintln!("\n=== RC-004C Overall Verdict ===");
    eprintln!("{}", overall_verdict);

    let report = Rc004cReport {
        campaign_id: "rc004c_v1.0".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        n_individuals: N_INDIVIDUALS,
        seed: FIXED_SEED,
        instances: instance_results,
        overall_verdict,
    };

    let json_path = format!("{}/rc004c_constructor_forensics.json", REPORT_DIR);
    let f = fs::File::create(&json_path).expect("create json");
    serde_json::to_writer_pretty(BufWriter::new(f), &report).expect("write json");
    eprintln!("Results written to {}", json_path);
}
