use std::cmp::Reverse;
/// rp404d_ecmp_conflict — RP-404D: Problem-Specific Neighbourhood (ECMP-Conflict Destroy)
///
/// Research question: Does a destroy operator that targets demands in ECMP conflict
/// with the most-loaded demand outperform the bottleneck-link operator (RP-404C)
/// and the generic random operator (RP-404A)?
///
/// Hypothesis: ECMP routing splits traffic across multiple equal-cost paths. When
/// multiple demands share the same ECMP-expanded link set, they compete for the
/// same capacity. Destroying the K demands most in conflict with the highest-load
/// demand gives the repair operator a chance to find non-conflicting routes,
/// escaping the ECMP-induced local optima that the RP-401 model introduced.
///
/// This is a materially different hypothesis from all prior operators:
///   - Random (RP-404A): no structural bias.
///   - Congestion (RP-404B): targets demands near congested nodes.
///   - Highcost (RP-404B): targets demands with high routing cost.
///   - BottleneckLink (RP-404C): targets demands traversing the most saturated link.
///   - ECMPConflict (RP-404D): targets demands that share ECMP-expanded links with
///     the highest-load demand — directly addressing the ECMP routing interaction
///     introduced in RP-401.
///
/// Success criteria (any one sufficient):
///   1. Recover setA-17 (the single remaining infeasible instance), OR
///   2. Improve aggregate Dataset A objective beyond RP-404A (Δ > −5.3641), OR
///   3. Improve on instances where all prior operators were unchanged.
///
/// Algorithm (RP-404D — ECMP-conflict destroy):
///   1. Load RP-403 solution JSON as the deterministic baseline.
///   2. Evaluate baseline objective using the ECMP-accurate evaluator.
///   3. Destroy (ECMPConflict):
///      a. Compute ECMP arc flows for the current t=0 solution.
///      b. For each demand, compute its "conflict score" = sum of saturation
///         on all links it uses (weighted by its ECMP flow fraction).
///      c. Select the demand with the highest conflict score as the "pivot".
///      d. Find all demands that share at least one ECMP-expanded link with
///         the pivot demand, ranked by their overlap score (number of shared
///         saturated links, weighted by saturation).
///      e. Select the top K demands by overlap score (including the pivot).
///      f. Pad with random demands if fewer than K found.
///   4. Repair: re-route removed demands using RP-401C ECMP-aware Dijkstra.
///   5. Evaluate repaired solution.
///   6. Accept if objective improves (best-improving acceptance).
///   7. Repeat for `--iters` iterations.
///   8. Write improved solution JSON (suffix: rp404d-ecmp-conflict).
///   9. Print per-instance benchmark table vs RP-403 baseline.
///
/// Repair operator: RP-401C ECMP-aware greedy (corrected additive penalty).
/// Baseline: validated RP-403 construction portfolio (commit e9296dfa).
///
/// Classification: Research binary (RP-404D).
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};

// ---------------------------------------------------------------------------
// Command-line argument parsing (minimal, no external crate needed).
// ---------------------------------------------------------------------------
struct Config {
    k: usize,
    iters: usize,
    seed: u64,
}

impl Config {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut k = 10usize;
        let mut iters = 50usize;
        let mut seed = 42u64;
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--k" => {
                    i += 1;
                    if i < args.len() {
                        k = args[i].parse().unwrap_or(10);
                    }
                }
                "--iters" => {
                    i += 1;
                    if i < args.len() {
                        iters = args[i].parse().unwrap_or(50);
                    }
                }
                "--seed" => {
                    i += 1;
                    if i < args.len() {
                        seed = args[i].parse().unwrap_or(42);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        Config { k, iters, seed }
    }
}

// ---------------------------------------------------------------------------
// Minimal LCG pseudo-random number generator (no external crate).
// Deterministic given a fixed seed.
// ---------------------------------------------------------------------------
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg {
            state: seed.wrapping_add(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_usize(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }
}

// ---------------------------------------------------------------------------
// Dijkstra shortest path (metric-weighted, ignoring disabled arcs).
// ---------------------------------------------------------------------------
fn dijkstra_path(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
    metric_multipliers: &HashMap<u64, f64>,
) -> Option<Vec<u64>> {
    if src == dst {
        return Some(vec![src]);
    }

    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) {
            continue;
        }
        let mult = metric_multipliers.get(&link.id).copied().unwrap_or(1.0);
        adj.entry(link.from)
            .or_default()
            .push((link.to, link.metric * mult));
    }

    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut prev: HashMap<u64, u64> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>, u64)> = BinaryHeap::new();

    dist.insert(src, 0);
    heap.push((Reverse(0), src));

    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost {
            continue;
        }
        if node == dst {
            break;
        }
        if let Some(neighbors) = adj.get(&node) {
            for &(next, metric) in neighbors {
                let new_cost = cost + (metric * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost {
                    dist.insert(next, new_cost);
                    prev.insert(next, node);
                    heap.push((Reverse(new_cost), next));
                }
            }
        }
    }

    if !dist.contains_key(&dst) {
        return None;
    }

    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        if let Some(&p) = prev.get(&cur) {
            path.push(p);
            cur = p;
        } else {
            return None;
        }
    }
    path.reverse();
    Some(path)
}

// ---------------------------------------------------------------------------
// Extract waypoints from a full node path.
// ---------------------------------------------------------------------------
fn path_to_waypoints(full_path: &[u64], max_segments: usize) -> Vec<u64> {
    if full_path.len() <= 2 {
        return vec![];
    }
    let waypoints: Vec<u64> = full_path[1..full_path.len() - 1].to_vec();
    if max_segments > 0 && waypoints.len() + 1 > max_segments {
        waypoints[..max_segments - 1].to_vec()
    } else {
        waypoints
    }
}

// ---------------------------------------------------------------------------
// Load-aware Dijkstra with ADDITIVE penalty — validated RP-401C repair.
// ---------------------------------------------------------------------------
fn load_aware_path_ecmp_rp401c(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
    ecmp_saturation: &HashMap<u64, f64>,
    load_penalty: f64,
) -> Option<Vec<u64>> {
    if src == dst {
        return Some(vec![src]);
    }
    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) {
            continue;
        }
        let sat = ecmp_saturation.get(&link.id).copied().unwrap_or(0.0);
        let penalty = if sat >= 1.0 {
            1e9
        } else if sat > 0.8 {
            load_penalty * (1.0 / (1.0 - sat) - 1.0) * 10.0
        } else {
            load_penalty * sat
        };
        let effective_metric = link.metric + penalty;
        adj.entry(link.from)
            .or_default()
            .push((link.to, effective_metric));
    }
    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut prev: HashMap<u64, u64> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>, u64)> = BinaryHeap::new();
    dist.insert(src, 0);
    heap.push((Reverse(0), src));
    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost {
            continue;
        }
        if node == dst {
            break;
        }
        if let Some(neighbors) = adj.get(&node) {
            for &(next, em) in neighbors {
                let new_cost = cost + (em * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost {
                    dist.insert(next, new_cost);
                    prev.insert(next, node);
                    heap.push((Reverse(new_cost), next));
                }
            }
        }
    }
    if !dist.contains_key(&dst) {
        return None;
    }
    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        if let Some(&p) = prev.get(&cur) {
            path.push(p);
            cur = p;
        } else {
            return None;
        }
    }
    path.reverse();
    Some(path)
}

// ---------------------------------------------------------------------------
// Load RP-403 solution JSON.
// ---------------------------------------------------------------------------
fn load_rp403_solution(
    set_dir: &str,
    inst: &str,
    evaluator: &RoadefEvaluator,
    num_demands: usize,
) -> Option<(Vec<SrPath>, f64)> {
    let path = format!("{}/setA-{}-srpaths-rp403.json", set_dir, inst);
    if !std::path::Path::new(&path).exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let srpaths_json = json["srpaths"].as_array()?;

    let mut srpaths: Vec<SrPath> = Vec::new();
    for entry in srpaths_json {
        let d = entry["d"].as_u64()? as usize;
        let t = entry["t"].as_u64()? as usize;
        let w: Vec<u64> = entry["w"]
            .as_array()?
            .iter()
            .filter_map(|v| v.as_u64())
            .collect();
        if d < num_demands {
            srpaths.push(SrPath { d, t, w });
        }
    }

    let sol = Solution {
        srpaths: srpaths.clone(),
    };
    let result = evaluator.evaluate_solution(&sol);
    Some((srpaths, result.obj))
}

// ---------------------------------------------------------------------------
// Build a waypoint assignment map from srpaths (t=0 only).
// ---------------------------------------------------------------------------
fn srpaths_to_assignments(srpaths: &[SrPath]) -> HashMap<usize, Vec<u64>> {
    let mut map: HashMap<usize, Vec<u64>> = HashMap::new();
    for sp in srpaths {
        if sp.t == 0 {
            map.insert(sp.d, sp.w.clone());
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Build srpaths from t=0 and t=1 assignment maps.
// ---------------------------------------------------------------------------
fn build_srpaths(
    t0_assignments: &HashMap<usize, Vec<u64>>,
    t1_assignments: &HashMap<usize, Vec<u64>>,
    num_demands: usize,
    num_slots: usize,
) -> Vec<SrPath> {
    let mut srpaths: Vec<SrPath> = Vec::new();
    for d_idx in 0..num_demands {
        if let Some(w) = t0_assignments.get(&d_idx) {
            if !w.is_empty() {
                srpaths.push(SrPath {
                    d: d_idx,
                    t: 0,
                    w: w.clone(),
                });
            }
        }
        if num_slots > 1 {
            if let Some(w) = t1_assignments.get(&d_idx) {
                if !w.is_empty() {
                    srpaths.push(SrPath {
                        d: d_idx,
                        t: 1,
                        w: w.clone(),
                    });
                }
            }
        }
    }
    srpaths
}

// ---------------------------------------------------------------------------
// Compute ECMP saturation map from a solution at time slot t.
// ---------------------------------------------------------------------------
fn compute_saturation(
    evaluator: &RoadefEvaluator,
    srpaths: &[SrPath],
    t: usize,
    link_capacity: &HashMap<u64, f64>,
) -> HashMap<u64, f64> {
    let sol = Solution {
        srpaths: srpaths.to_vec(),
    };
    let mut sat: HashMap<u64, f64> = HashMap::new();
    if let Some(loads) = evaluator.compute_loads(t, &sol) {
        for (arc_id, flow) in &loads.arc_flows {
            let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
            sat.insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
        }
    }
    sat
}

// ---------------------------------------------------------------------------
// DESTROY OPERATOR: ECMPConflict (RP-404D — problem-specific)
//
// Algorithm:
//   1. Compute ECMP saturation for the current t=0 solution.
//   2. For each demand, build its set of ECMP-expanded links:
//      The links a demand uses are the directed edges (from→to) in its
//      waypoint path: src → w[0] → w[1] → ... → w[n-1] → dst.
//   3. For each demand, compute its "load score" = sum of saturation values
//      on all links it uses. This measures how much the demand contributes
//      to overall congestion.
//   4. Select the demand with the highest load score as the "pivot".
//   5. For each other demand, compute its "conflict score" with the pivot =
//      number of shared links, weighted by the saturation of those links.
//   6. Select the top K-1 demands by conflict score (plus the pivot = K total).
//   7. Pad with random demands if fewer than K found.
//
// Scientific distinction from RP-404C (bottleneck-link):
//   - RP-404C: destroys demands traversing the single most saturated link.
//     This is a link-centric view — it asks "which link is the bottleneck?"
//   - RP-404D: destroys demands that are in ECMP conflict with the most-loaded
//     demand. This is a demand-interaction view — it asks "which demands are
//     competing with each other for the same ECMP paths?"
//   The ECMP-conflict view directly targets the routing interaction introduced
//   by RP-401's ECMP model, which is the mechanism that creates the local optima
//   that prior operators have failed to escape.
// ---------------------------------------------------------------------------
fn destroy_ecmp_conflict(
    assignments: &HashMap<usize, Vec<u64>>,
    srpaths: &[SrPath],
    evaluator: &RoadefEvaluator,
    link_capacity: &HashMap<u64, f64>,
    net: &Network,
    demands: &[(usize, u64, u64, f64)], // (d_idx, src, dst, vol)
    k: usize,
    rng: &mut Lcg,
) -> Vec<usize> {
    let sat = compute_saturation(evaluator, srpaths, 0, link_capacity);

    // Build link endpoint map: link_id → (from_node, to_node)
    let mut link_endpoints: HashMap<u64, (u64, u64)> = HashMap::new();
    for link in &net.links {
        link_endpoints.insert(link.id, (link.from, link.to));
    }

    // Build demand source/dest map
    let mut demand_src_dst: HashMap<usize, (u64, u64)> = HashMap::new();
    for (d_idx, src, dst, _) in demands {
        demand_src_dst.insert(*d_idx, (*src, *dst));
    }

    // For each demand, build its set of directed link edges (from, to) used.
    // Full path: src → w[0] → ... → w[n-1] → dst
    // Edges: consecutive pairs in the full path.
    let mut demand_link_sets: HashMap<usize, HashSet<(u64, u64)>> = HashMap::new();
    for sp in srpaths {
        if sp.t != 0 {
            continue;
        }
        if !assignments.contains_key(&sp.d) {
            continue;
        }
        if let Some(&(src, dst)) = demand_src_dst.get(&sp.d) {
            let mut full = vec![src];
            full.extend_from_slice(&sp.w);
            full.push(dst);
            let edges: HashSet<(u64, u64)> = full.windows(2).map(|w| (w[0], w[1])).collect();
            demand_link_sets.insert(sp.d, edges);
        }
    }

    if demand_link_sets.is_empty() {
        // Fallback: random selection
        let mut all: Vec<usize> = assignments.keys().copied().collect();
        all.sort();
        let actual = k.min(all.len());
        for i in 0..actual {
            let j = i + rng.next_usize(all.len() - i);
            all.swap(i, j);
        }
        return all[..actual].to_vec();
    }

    // Build a reverse map: edge → set of demands using it
    // Also build link_id lookup: (from, to) → link_id for saturation lookup
    let mut edge_to_link_id: HashMap<(u64, u64), u64> = HashMap::new();
    for link in &net.links {
        edge_to_link_id.insert((link.from, link.to), link.id);
    }

    // Compute load score for each demand: sum of saturation on its links
    let mut demand_load_scores: Vec<(usize, f64)> = demand_link_sets
        .iter()
        .map(|(&d_idx, edges)| {
            let score: f64 = edges
                .iter()
                .map(|edge| {
                    let link_id = edge_to_link_id.get(edge).copied().unwrap_or(0);
                    sat.get(&link_id).copied().unwrap_or(0.0)
                })
                .sum();
            (d_idx, score)
        })
        .collect();
    demand_load_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Select pivot: demand with highest load score
    let pivot_d = demand_load_scores[0].0;
    let pivot_edges = match demand_link_sets.get(&pivot_d) {
        Some(e) => e.clone(),
        None => {
            // Fallback: random
            let mut all: Vec<usize> = assignments.keys().copied().collect();
            all.sort();
            let actual = k.min(all.len());
            for i in 0..actual {
                let j = i + rng.next_usize(all.len() - i);
                all.swap(i, j);
            }
            return all[..actual].to_vec();
        }
    };

    // Compute conflict score for each other demand with the pivot:
    // conflict_score = sum of saturation on shared links
    let mut conflict_scores: Vec<(usize, f64)> = demand_link_sets
        .iter()
        .filter(|(&d_idx, _)| d_idx != pivot_d)
        .map(|(&d_idx, edges)| {
            let shared_sat: f64 = edges
                .intersection(&pivot_edges)
                .map(|edge| {
                    let link_id = edge_to_link_id.get(edge).copied().unwrap_or(0);
                    sat.get(&link_id).copied().unwrap_or(0.0)
                })
                .sum();
            (d_idx, shared_sat)
        })
        .collect();
    conflict_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Build selected set: pivot + top K-1 by conflict score
    let mut selected: Vec<usize> = vec![pivot_d];
    let mut seen: HashSet<usize> = HashSet::from([pivot_d]);

    for (d_idx, score) in &conflict_scores {
        if selected.len() >= k {
            break;
        }
        // Only include demands with non-zero conflict (actually share a link with pivot)
        // If score == 0.0, they share no saturated links — fall through to random padding
        if *score > 0.0 {
            selected.push(*d_idx);
            seen.insert(*d_idx);
        }
    }

    // Pad with random demands if needed (demands with zero conflict score)
    if selected.len() < k {
        let mut remaining: Vec<usize> = assignments
            .keys()
            .copied()
            .filter(|d| !seen.contains(d))
            .collect();
        remaining.sort();
        let need = k - selected.len();
        let actual = need.min(remaining.len());
        for i in 0..actual {
            let j = i + rng.next_usize(remaining.len() - i);
            remaining.swap(i, j);
        }
        selected.extend_from_slice(&remaining[..actual]);
    }

    selected
}

// ---------------------------------------------------------------------------
// REPAIR OPERATOR: RP-401C ECMP-aware greedy
// ---------------------------------------------------------------------------
fn repair_rp401c(
    net: &Network,
    evaluator: &RoadefEvaluator,
    current_assignments: &HashMap<usize, Vec<u64>>,
    destroyed: &[usize],
    demands: &[(usize, u64, u64, f64)],
    disabled_links: &HashSet<u64>,
    max_segments: usize,
    link_capacity: &HashMap<u64, f64>,
    deadline: Instant,
) -> HashMap<usize, Vec<u64>> {
    let destroyed_set: HashSet<usize> = destroyed.iter().copied().collect();

    let mut assignments = current_assignments.clone();
    for d in &destroyed_set {
        assignments.remove(d);
    }

    let mut partial_srpaths: Vec<SrPath> = assignments
        .iter()
        .filter(|(_, w)| !w.is_empty())
        .map(|(&d_idx, w)| SrPath {
            d: d_idx,
            t: 0,
            w: w.clone(),
        })
        .collect();

    let mut ecmp_saturation = compute_saturation(evaluator, &partial_srpaths, 0, link_capacity);
    for link in &net.links {
        ecmp_saturation.entry(link.id).or_insert(0.0);
    }

    let mut to_repair: Vec<(usize, u64, u64, f64)> = demands
        .iter()
        .filter(|(d_idx, _, _, _)| destroyed_set.contains(d_idx))
        .copied()
        .collect();
    to_repair.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    for (d_idx, src, dst, _vol) in &to_repair {
        if Instant::now() >= deadline {
            break;
        }

        let full_path =
            load_aware_path_ecmp_rp401c(net, *src, *dst, disabled_links, &ecmp_saturation, 100.0)
                .or_else(|| dijkstra_path(net, *src, *dst, disabled_links, &HashMap::new()));

        if let Some(fp) = full_path {
            let waypoints = path_to_waypoints(&fp, max_segments);
            if !waypoints.is_empty() {
                partial_srpaths.push(SrPath {
                    d: *d_idx,
                    t: 0,
                    w: waypoints.clone(),
                });
            }
            assignments.insert(*d_idx, waypoints);

            let partial_sol = Solution {
                srpaths: partial_srpaths.clone(),
            };
            if let Some(loads) = evaluator.compute_loads(0, &partial_sol) {
                ecmp_saturation.clear();
                for (arc_id, flow) in &loads.arc_flows {
                    let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
                    ecmp_saturation
                        .insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
                }
            }
        }
    }

    assignments
}

// ---------------------------------------------------------------------------
// Evaluate a full solution (t=0 + t=1 shared paths).
// ---------------------------------------------------------------------------
fn evaluate_assignments(
    evaluator: &RoadefEvaluator,
    t0_assignments: &HashMap<usize, Vec<u64>>,
    t1_assignments: &HashMap<usize, Vec<u64>>,
    num_demands: usize,
    num_slots: usize,
) -> (Vec<SrPath>, f64) {
    let srpaths = build_srpaths(t0_assignments, t1_assignments, num_demands, num_slots);
    let sol = Solution {
        srpaths: srpaths.clone(),
    };
    let result = evaluator.evaluate_solution(&sol);
    (srpaths, result.obj)
}

// ---------------------------------------------------------------------------
// Selection helper: is objective A better than objective B?
// ---------------------------------------------------------------------------
fn is_better(obj_a: f64, obj_b: f64) -> bool {
    if obj_a.is_finite() && !obj_b.is_finite() {
        return true;
    }
    if !obj_a.is_finite() && obj_b.is_finite() {
        return false;
    }
    obj_a < obj_b
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    let cfg = Config::from_args();
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";

    println!(
        "RP-404D — ECMP-Conflict Destroy (k={}, iters={}, seed={})",
        cfg.k, cfg.iters, cfg.seed
    );
    println!("{}", "=".repeat(100));
    println!(
        "{:<10} {:>12} {:>12} {:>10} {:>8} {:>8} {:>8}",
        "Instance", "LNS obj", "RP-403 obj", "delta", "improved", "iters", "ms"
    );
    println!("{}", "-".repeat(100));

    let mut improved_count = 0usize;
    let mut regressed_count = 0usize;
    let mut unchanged_count = 0usize;
    let mut finite_count = 0usize;
    let mut total_delta = 0.0f64;

    for instance_id in 1..=20 {
        let inst = format!("{:02}", instance_id);
        let net_path = format!("{}/setA-{}-net.json", set_dir, inst);
        let tm_path = format!("{}/setA-{}-tm.json", set_dir, inst);
        let sc_path = format!("{}/setA-{}-scenario.json", set_dir, inst);

        let net = load_network(&net_path)?;
        let tm = load_traffic_matrix(&tm_path)?;
        let scenario = load_scenario(&sc_path)?;

        let num_demands = tm.demands.len();
        let num_slots = tm.num_time_slots;
        let max_seg = if scenario.max_segments >= 0 {
            scenario.max_segments as usize
        } else {
            100
        };

        let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario.clone());

        let disabled_t0: HashSet<u64> = scenario
            .interventions
            .iter()
            .filter(|i| i.t == 0)
            .flat_map(|i| i.links.iter().copied())
            .collect();
        let disabled_t1: HashSet<u64> = scenario
            .interventions
            .iter()
            .filter(|i| i.t == 1)
            .flat_map(|i| i.links.iter().copied())
            .collect();
        let disabled_both: HashSet<u64> = disabled_t0.union(&disabled_t1).copied().collect();

        let link_capacity: HashMap<u64, f64> =
            net.links.iter().map(|l| (l.id, l.capacity)).collect();

        let demands_avg: Vec<(usize, u64, u64, f64)> = tm
            .demands
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let v0 = d.v[0];
                let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
                (i, d.s, d.t, (v0 + v1) / 2.0)
            })
            .collect();

        let t_start = Instant::now();
        let deadline = t_start + std::time::Duration::from_secs(120);

        // Step 1: Load RP-403 baseline solution
        let (baseline_srpaths, baseline_obj) =
            match load_rp403_solution(set_dir, &inst, &evaluator, num_demands) {
                Some(x) => x,
                None => {
                    println!(
                        "{:<10} {:>12} {:>12} {:>10} {:>8} {:>8} {:>8}",
                        format!("setA-{}", inst),
                        "N/A",
                        "N/A",
                        "N/A",
                        "N/A",
                        0,
                        0
                    );
                    continue;
                }
            };

        let mut best_t0 = srpaths_to_assignments(&baseline_srpaths);
        let mut best_t1: HashMap<usize, Vec<u64>> = HashMap::new();
        for sp in &baseline_srpaths {
            if sp.t == 1 {
                best_t1.insert(sp.d, sp.w.clone());
            }
        }
        let mut best_obj = baseline_obj;
        let mut iters_improved = 0usize;

        let mut rng = Lcg::new(cfg.seed.wrapping_add(instance_id as u64));

        // Step 2: LNS loop
        for _iter in 0..cfg.iters {
            if Instant::now() >= deadline {
                break;
            }

            let current_srpaths = build_srpaths(&best_t0, &best_t1, num_demands, num_slots);

            // Step 2a: Destroy — ECMPConflict
            let destroyed = destroy_ecmp_conflict(
                &best_t0,
                &current_srpaths,
                &evaluator,
                &link_capacity,
                &net,
                &demands_avg,
                cfg.k,
                &mut rng,
            );

            if destroyed.is_empty() {
                continue;
            }

            // Step 2b: Repair using RP-401C
            let repaired_t0 = repair_rp401c(
                &net,
                &evaluator,
                &best_t0,
                &destroyed,
                &demands_avg,
                &disabled_both,
                max_seg,
                &link_capacity,
                deadline,
            );

            // Step 2c: Evaluate repaired solution (keep t=1 from best)
            let (_, repaired_obj) =
                evaluate_assignments(&evaluator, &repaired_t0, &best_t1, num_demands, num_slots);

            // Step 2d: Accept if improved (best-improving acceptance)
            if is_better(repaired_obj, best_obj) {
                best_t0 = repaired_t0;
                best_obj = repaired_obj;
                iters_improved += 1;
            }
        }

        // Step 3: Build final solution
        let final_srpaths = build_srpaths(&best_t0, &best_t1, num_demands, num_slots);

        // Step 4: Validate and select output
        let result = evaluator.evaluate_solution(&Solution {
            srpaths: final_srpaths.clone(),
        });
        let empty_result = evaluator.evaluate_solution(&Solution { srpaths: vec![] });

        let (output_srpaths, final_obj) = if !result.valid {
            (vec![], empty_result.obj)
        } else if result.obj.is_finite()
            && (empty_result.obj.is_infinite() || result.obj <= empty_result.obj)
        {
            (final_srpaths, result.obj)
        } else if result.obj.is_infinite() && empty_result.obj.is_finite() {
            (vec![], empty_result.obj)
        } else if result.obj.is_infinite() && empty_result.obj.is_infinite() {
            (final_srpaths, result.obj)
        } else {
            (vec![], empty_result.obj)
        };

        if final_obj.is_finite() {
            finite_count += 1;
        }

        // Step 5: Compute delta vs RP-403 baseline
        let delta_str = if baseline_obj.is_infinite() && final_obj.is_infinite() {
            "both inf".to_string()
        } else if baseline_obj.is_finite() && final_obj.is_finite() {
            let d = final_obj - baseline_obj;
            total_delta += d;
            if d < -1e-6 {
                improved_count += 1;
                format!("{:.4}", d)
            } else if d > 1e-6 {
                regressed_count += 1;
                format!("+{:.4}", d)
            } else {
                unchanged_count += 1;
                "=".to_string()
            }
        } else {
            unchanged_count += 1;
            "mixed".to_string()
        };

        let elapsed_ms = t_start.elapsed().as_millis();
        println!(
            "{:<10} {:>12.4} {:>12.4} {:>10} {:>8} {:>8} {:>8}",
            format!("setA-{}", inst),
            if final_obj.is_finite() {
                final_obj
            } else {
                f64::INFINITY
            },
            if baseline_obj.is_finite() {
                baseline_obj
            } else {
                f64::INFINITY
            },
            delta_str,
            if iters_improved > 0 { "yes" } else { "no" },
            iters_improved,
            elapsed_ms,
        );

        // Step 6: Write solution JSON
        let out_path = format!(
            "{}/setA-{}-srpaths-rp404d-ecmp-conflict.json",
            set_dir, inst
        );
        let srpaths_json: Vec<serde_json::Value> = output_srpaths
            .iter()
            .map(|sp| {
                serde_json::json!({
                    "d": sp.d,
                    "t": sp.t,
                    "w": sp.w,
                })
            })
            .collect();
        let json_out = serde_json::json!({ "srpaths": srpaths_json });
        let mut f = File::create(&out_path)?;
        writeln!(f, "{}", serde_json::to_string_pretty(&json_out)?)?;
    }

    println!("{}", "=".repeat(100));
    println!(
        "RP-404D vs RP-403: {} improved, {} regressed, {} unchanged",
        improved_count, regressed_count, unchanged_count
    );
    println!("Total improvement vs RP-403: {:.4}", total_delta);
    println!("Finite solutions: {}/20", finite_count);
    println!(
        "Solution files written to {}/setA-*-srpaths-rp404d-ecmp-conflict.json",
        set_dir
    );

    Ok(())
}
