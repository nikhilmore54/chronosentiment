use std::cmp::Reverse;
/// rp403_construction_portfolio — RP-403: Construction Strategy Evaluation and Selection
///
/// Research question: Does selecting between two construction strategies (RP-401C
/// nearest-neighbour ECMP-aware greedy and RP-401D oracle-guided path selection)
/// eliminate the observed regressions for setA-12 and setA-08 without harming
/// the 18 currently finite instances?
///
/// Hypothesis: Construction strategy differences (including, but not yet proven
/// to be limited to, demand ordering) are the gating factor for the investigated
/// failures. Running both constructions and selecting the better result will
/// recover setA-12 and setA-08 without regressing the 18 currently finite instances.
///
/// Algorithm:
///   1. Run RP-401C ECMP-aware greedy construction (volume-sorted, ECMP oracle).
///   2. Run RP-401D oracle-guided path selection (volume-sorted, K=5 candidates).
///   3. Evaluate both constructions using the ECMP-accurate objective.
///   4. Select the better construction:
///      - Feasible (finite obj) beats infeasible (inf obj).
///      - Among same feasibility, lower objective wins.
///      - Tie: prefer RP-401C (historically stronger constructor).
///   5. Feed the selected construction into RP-402 budget-aware t=1 adaptation.
///   6. Benchmark the full portfolio against the RP-402 baseline (existing JSON files).
///
/// Success criteria:
///   - Recover setA-12 (RP-401C finite at 26.12, RP-402 inf).
///   - Recover setA-08 (RP-401D finite at 48.67, RP-402 inf).
///   - No regression on the 18 currently finite instances.
///
/// Classification: Research binary (not a Candidate or Competition Submission).
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};
use roadef::path::SrPathBit;

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
// Load-aware Dijkstra with ADDITIVE penalty — matches standalone binary exactly.
//
// Corrected in Validation Task V1 Commit C (2026-08-03).
// Original used multiplicative metric multiplier (metric * mult).
// Corrected to use additive penalty (metric + penalty), matching
// rp401c_ecmp_construction.rs load_aware_path_ecmp() exactly.
//
// Penalty formula (load_penalty = 100.0):
//   sat >= 1.0  → penalty = 1e9
//   sat >  0.8  → penalty = load_penalty * (1/(1-sat) - 1) * 10.0
//   else        → penalty = load_penalty * sat
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
// RP-401C: ECMP-aware greedy construction (volume-sorted).
//
// For each demand (sorted by volume descending):
//   1. Query ECMP-oracle saturations from the current partial solution.
//   2. Run load-aware Dijkstra with ADDITIVE penalty (corrected in V1 Commit C).
//   3. Commit the chosen path and update ECMP saturations.
// ---------------------------------------------------------------------------
fn solve_rp401c(
    net: &Network,
    evaluator: &RoadefEvaluator,
    demands: &[(usize, u64, u64, f64)],
    disabled_links: &HashSet<u64>,
    max_segments: usize,
    deadline: Instant,
) -> HashMap<usize, Vec<u64>> {
    let mut sorted: Vec<(usize, u64, u64, f64)> = demands.to_vec();
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
    }

    let mut partial_srpaths: Vec<SrPath> = Vec::new();
    let mut assignments: HashMap<usize, Vec<u64>> = HashMap::new();
    let mut ecmp_saturation: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        ecmp_saturation.insert(link.id, 0.0);
    }

    for (d_idx, src, dst, _vol) in &sorted {
        if Instant::now() >= deadline {
            break;
        }

        // Corrected: additive-penalty ECMP-aware Dijkstra (matches standalone binary)
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
// RP-401D: Oracle-guided path selection (volume-sorted, K=5 candidates).
//
// For each demand (sorted by volume descending):
//   1. Generate K candidate paths (unperturbed + load-aware + inflated variants).
//   2. Evaluate each candidate by calling compute_loads() on the partial solution.
//   3. Select the candidate with the lowest resulting MLU.
//   4. Commit and update ECMP saturations.
// ---------------------------------------------------------------------------
fn generate_candidates(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
    ecmp_saturation: &HashMap<u64, f64>,
    k: usize,
    max_segments: usize,
) -> Vec<Vec<u64>> {
    let mut candidates: Vec<Vec<u64>> = Vec::new();
    let mut seen_paths: HashSet<Vec<u64>> = HashSet::new();

    // Candidate 0: unperturbed shortest path
    let no_mult: HashMap<u64, f64> = HashMap::new();
    if let Some(path) = dijkstra_path(net, src, dst, disabled_links, &no_mult) {
        let wp = path_to_waypoints(&path, max_segments);
        if seen_paths.insert(wp.clone()) {
            candidates.push(wp);
        }
    }

    // Candidate 1: load-aware (penalise saturated links)
    let load_mult: HashMap<u64, f64> = net
        .links
        .iter()
        .filter(|l| !disabled_links.contains(&l.id))
        .map(|l| {
            let sat = ecmp_saturation.get(&l.id).copied().unwrap_or(0.0);
            let mult = if sat >= 1.0 {
                1e6
            } else if sat > 0.8 {
                1.0 + 100.0 * (1.0 / (1.0 - sat) - 1.0)
            } else {
                1.0 + sat
            };
            (l.id, mult)
        })
        .collect();
    if let Some(path) = dijkstra_path(net, src, dst, disabled_links, &load_mult) {
        let wp = path_to_waypoints(&path, max_segments);
        if seen_paths.insert(wp.clone()) {
            candidates.push(wp);
        }
    }

    // Candidates 2..K-1: inflate individual high-saturation links
    let mut sorted_links: Vec<(u64, f64)> = ecmp_saturation
        .iter()
        .filter(|(id, _)| !disabled_links.contains(id))
        .map(|(&id, &sat)| (id, sat))
        .collect();
    sorted_links.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (i, (link_id, _sat)) in sorted_links.iter().enumerate().take(k.saturating_sub(2)) {
        let mut mult: HashMap<u64, f64> = HashMap::new();
        for j in 0..=i {
            if j < sorted_links.len() {
                mult.insert(sorted_links[j].0, 1e4);
            }
        }
        if let Some(path) = dijkstra_path(net, src, dst, disabled_links, &mult) {
            let wp = path_to_waypoints(&path, max_segments);
            if seen_paths.insert(wp.clone()) {
                candidates.push(wp);
                if candidates.len() >= k {
                    break;
                }
            }
        }
        let _ = link_id;
    }

    candidates
}

fn solve_rp401d(
    net: &Network,
    evaluator: &RoadefEvaluator,
    demands: &[(usize, u64, u64, f64)],
    disabled_links: &HashSet<u64>,
    max_segments: usize,
    k_candidates: usize,
    deadline: Instant,
) -> HashMap<usize, Vec<u64>> {
    let mut sorted: Vec<(usize, u64, u64, f64)> = demands.to_vec();
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let mut partial_srpaths: Vec<SrPath> = Vec::new();
    let mut assignments: HashMap<usize, Vec<u64>> = HashMap::new();
    let mut ecmp_saturation: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        ecmp_saturation.insert(link.id, 0.0);
    }
    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
    }

    for (d_idx, src, dst, _vol) in &sorted {
        if Instant::now() >= deadline {
            break;
        }

        let candidates = generate_candidates(
            net,
            *src,
            *dst,
            disabled_links,
            &ecmp_saturation,
            k_candidates,
            max_segments,
        );

        if candidates.is_empty() {
            continue;
        }

        let mut best_waypoints: Option<Vec<u64>> = None;
        let mut best_mlu = f64::INFINITY;

        for waypoints in &candidates {
            if Instant::now() >= deadline {
                break;
            }
            let mut trial_srpaths = partial_srpaths.clone();
            if !waypoints.is_empty() {
                trial_srpaths.push(SrPath {
                    d: *d_idx,
                    t: 0,
                    w: waypoints.clone(),
                });
            }
            let trial_sol = Solution {
                srpaths: trial_srpaths,
            };
            if let Some(loads) = evaluator.compute_loads(0, &trial_sol) {
                if loads.mlu < best_mlu {
                    best_mlu = loads.mlu;
                    best_waypoints = Some(waypoints.clone());
                }
            }
        }

        if let Some(wp) = best_waypoints {
            if !wp.is_empty() {
                partial_srpaths.push(SrPath {
                    d: *d_idx,
                    t: 0,
                    w: wp.clone(),
                });
            }
            assignments.insert(*d_idx, wp);

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
// Build srpaths from a shared assignment map.
// ---------------------------------------------------------------------------
fn build_srpaths(
    assignments: &HashMap<usize, Vec<u64>>,
    num_demands: usize,
    num_slots: usize,
) -> Vec<SrPath> {
    let mut srpaths: Vec<SrPath> = Vec::new();
    for d_idx in 0..num_demands {
        if let Some(w) = assignments.get(&d_idx) {
            if !w.is_empty() {
                srpaths.push(SrPath {
                    d: d_idx,
                    t: 0,
                    w: w.clone(),
                });
                if num_slots > 1 {
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
// Selection logic: is construction A better than construction B?
//
// Rules (in priority order):
//   1. Feasible (finite obj) beats infeasible (inf obj).
//   2. Among same feasibility, lower objective wins.
//   3. Tie: prefer RP-401C (historically stronger constructor).
// ---------------------------------------------------------------------------
fn is_better(obj_a: f64, obj_b: f64) -> bool {
    if obj_a.is_finite() && !obj_b.is_finite() {
        return true; // A is feasible, B is not
    }
    if !obj_a.is_finite() && obj_b.is_finite() {
        return false; // B is feasible, A is not
    }
    // Both finite or both infinite: lower wins (tie → false = prefer A=401C)
    obj_a < obj_b
}

// ---------------------------------------------------------------------------
// RP-402: Budget-aware t=1 adaptation.
//
// Given shared path assignments, identify demands with the largest traffic
// change |v[1] - v[0]| and attempt to re-route them for t=1 only, within
// the budget constraint.
// ---------------------------------------------------------------------------
fn adapt_t1_within_budget(
    net: &Network,
    evaluator: &RoadefEvaluator,
    shared_assignments: &HashMap<usize, Vec<u64>>,
    demands_with_delta: &[(usize, u64, u64, f64, f64)],
    disabled_t1: &HashSet<u64>,
    max_segments: usize,
    budget_t1: usize,
    deadline: Instant,
) -> HashMap<usize, Vec<u64>> {
    let mut t1_assignments: HashMap<usize, Vec<u64>> = shared_assignments.clone();
    let mut budget_remaining = budget_t1;

    let mut sorted: Vec<(usize, u64, u64, f64, f64)> = demands_with_delta.to_vec();
    sorted.sort_by(|a, b| {
        let da = (a.4 - a.3).abs();
        let db = (b.4 - b.3).abs();
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut ecmp_saturation_t1: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        ecmp_saturation_t1.insert(link.id, 0.0);
    }
    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
    }

    // Compute initial t=1 saturation from shared paths
    let initial_t1_srpaths: Vec<SrPath> = shared_assignments
        .iter()
        .filter(|(_, wp)| !wp.is_empty())
        .map(|(&d_idx, wp)| SrPath {
            d: d_idx,
            t: 1,
            w: wp.clone(),
        })
        .collect();
    let initial_t1_sol = Solution {
        srpaths: initial_t1_srpaths,
    };
    if let Some(loads) = evaluator.compute_loads(1, &initial_t1_sol) {
        for (arc_id, flow) in &loads.arc_flows {
            let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
            ecmp_saturation_t1.insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
        }
    }

    for (d_idx, src, dst, _v0, _v1) in &sorted {
        if Instant::now() >= deadline {
            break;
        }
        if budget_remaining == 0 {
            break;
        }

        let shared_wp = shared_assignments.get(d_idx).cloned().unwrap_or_default();
        let shared_bit = SrPathBit::new_explicit(*src, *dst, &shared_wp);

        let load_mult: HashMap<u64, f64> = net
            .links
            .iter()
            .filter(|l| !disabled_t1.contains(&l.id))
            .map(|l| {
                let sat = ecmp_saturation_t1.get(&l.id).copied().unwrap_or(0.0);
                let mult = if sat >= 1.0 {
                    1e6
                } else if sat > 0.8 {
                    1.0 + 100.0 * (1.0 / (1.0 - sat) - 1.0)
                } else {
                    1.0 + sat
                };
                (l.id, mult)
            })
            .collect();

        let candidate_wp = match dijkstra_path(net, *src, *dst, disabled_t1, &load_mult) {
            Some(path) => path_to_waypoints(&path, max_segments),
            None => continue,
        };

        if candidate_wp == shared_wp {
            continue;
        }

        let candidate_bit = SrPathBit::new_explicit(*src, *dst, &candidate_wp);
        let switch_cost = candidate_bit.dist(&shared_bit);

        if switch_cost > budget_remaining {
            continue;
        }

        let mut trial_t1_srpaths: Vec<SrPath> = t1_assignments
            .iter()
            .filter(|(_, wp)| !wp.is_empty())
            .map(|(&di, wp)| SrPath {
                d: di,
                t: 1,
                w: wp.clone(),
            })
            .collect();

        trial_t1_srpaths.retain(|p| p.d != *d_idx);
        if !candidate_wp.is_empty() {
            trial_t1_srpaths.push(SrPath {
                d: *d_idx,
                t: 1,
                w: candidate_wp.clone(),
            });
        }

        let trial_sol = Solution {
            srpaths: trial_t1_srpaths.clone(),
        };
        if let Some(loads) = evaluator.compute_loads(1, &trial_sol) {
            let current_t1_sol = Solution {
                srpaths: t1_assignments
                    .iter()
                    .filter(|(_, wp)| !wp.is_empty())
                    .map(|(&di, wp)| SrPath {
                        d: di,
                        t: 1,
                        w: wp.clone(),
                    })
                    .collect(),
            };
            let current_mlu = evaluator
                .compute_loads(1, &current_t1_sol)
                .map(|l| l.mlu)
                .unwrap_or(f64::INFINITY);

            if loads.mlu < current_mlu {
                t1_assignments.insert(*d_idx, candidate_wp.clone());
                budget_remaining -= switch_cost;

                ecmp_saturation_t1.clear();
                for (arc_id, flow) in &loads.arc_flows {
                    let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
                    ecmp_saturation_t1
                        .insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
                }
            }
        }
    }

    t1_assignments
}

// ---------------------------------------------------------------------------
// Load RP-402 baseline objective from existing JSON solution file.
// Returns None if file does not exist or cannot be parsed.
// ---------------------------------------------------------------------------
fn load_rp402_obj(
    set_dir: &str,
    inst: &str,
    evaluator: &RoadefEvaluator,
    num_demands: usize,
) -> Option<f64> {
    let path = format!("{}/setA-{}-srpaths-rp402.json", set_dir, inst);
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

    let sol = Solution { srpaths };
    let result = evaluator.evaluate_solution(&sol);
    Some(result.obj)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
    let k_candidates = 5;

    println!("RP-403 — Construction Portfolio (RP-401C + RP-401D + RP-402 Adaptation)");
    println!("{}", "=".repeat(90));
    println!(
        "{:<10} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8} {:>8}",
        "Instance",
        "RP-403 obj",
        "RP-402 obj",
        "RP-401C obj",
        "RP-401D obj",
        "selected",
        "budget",
        "ms"
    );
    println!("{}", "-".repeat(90));

    let mut improved_vs_rp402 = 0usize;
    let mut regressed_vs_rp402 = 0usize;
    let mut unchanged_vs_rp402 = 0usize;
    let mut finite_count = 0usize;
    let mut total_improvement_vs_rp402 = 0.0f64;

    // Per-instance results for benchmark report
    let mut results: Vec<(String, f64, f64, f64, f64, &'static str)> = Vec::new();

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
        let budget_t1 = scenario
            .budget
            .iter()
            .find(|b| b.t == 1)
            .map(|b| b.value)
            .unwrap_or(0);

        let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario.clone());

        // Disabled links per time slot
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

        // Average volume demands for shared-path construction
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
        // Per-instance timeout: 300 seconds per construction, 600 total
        let deadline_401c = t_start + std::time::Duration::from_secs(300);
        let deadline_401d = t_start + std::time::Duration::from_secs(600);

        // Step 1: Run RP-401C construction
        let assign_401c = solve_rp401c(
            &net,
            &evaluator,
            &demands_avg,
            &disabled_both,
            max_seg,
            deadline_401c,
        );
        let srpaths_401c = build_srpaths(&assign_401c, num_demands, num_slots);
        let sol_401c = Solution {
            srpaths: srpaths_401c,
        };
        let result_401c = evaluator.evaluate_solution(&sol_401c);
        let obj_401c = result_401c.obj;

        // Step 2: Run RP-401D construction
        let assign_401d = solve_rp401d(
            &net,
            &evaluator,
            &demands_avg,
            &disabled_both,
            max_seg,
            k_candidates,
            deadline_401d,
        );
        let srpaths_401d = build_srpaths(&assign_401d, num_demands, num_slots);
        let sol_401d = Solution {
            srpaths: srpaths_401d,
        };
        let result_401d = evaluator.evaluate_solution(&sol_401d);
        let obj_401d = result_401d.obj;

        // Step 3: Select the better construction
        // is_better(401d, 401c) = true means 401D wins; false means 401C wins (tie → 401C)
        let (selected_assignments, selected_label) = if is_better(obj_401d, obj_401c) {
            (assign_401d, "rp401d")
        } else {
            (assign_401c, "rp401c")
        };

        // Step 4: RP-402 budget-aware t=1 adaptation on the selected construction
        let deadline_adapt = t_start + std::time::Duration::from_secs(900);
        let t1_assignments = if num_slots > 1 && budget_t1 > 0 {
            let demands_with_delta: Vec<(usize, u64, u64, f64, f64)> = tm
                .demands
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let v0 = d.v[0];
                    let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
                    (i, d.s, d.t, v0, v1)
                })
                .collect();

            adapt_t1_within_budget(
                &net,
                &evaluator,
                &selected_assignments,
                &demands_with_delta,
                &disabled_t1,
                max_seg,
                budget_t1,
                deadline_adapt,
            )
        } else {
            selected_assignments.clone()
        };

        // Step 5: Build final srpaths (t=0 from selected, t=1 from adaptation)
        let mut final_srpaths: Vec<SrPath> = Vec::new();
        for d_idx in 0..num_demands {
            if let Some(w) = selected_assignments.get(&d_idx) {
                if !w.is_empty() {
                    final_srpaths.push(SrPath {
                        d: d_idx,
                        t: 0,
                        w: w.clone(),
                    });
                }
            }
            if num_slots > 1 {
                if let Some(w) = t1_assignments.get(&d_idx) {
                    if !w.is_empty() {
                        final_srpaths.push(SrPath {
                            d: d_idx,
                            t: 1,
                            w: w.clone(),
                        });
                    }
                }
            }
        }

        // Step 6: Evaluate final solution
        let solution = Solution {
            srpaths: final_srpaths.clone(),
        };
        let result = evaluator.evaluate_solution(&solution);
        let empty_sol = Solution { srpaths: vec![] };
        let empty_result = evaluator.evaluate_solution(&empty_sol);

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

        // Load RP-402 baseline for comparison
        let rp402_obj =
            load_rp402_obj(set_dir, &inst, &evaluator, num_demands).unwrap_or(f64::INFINITY);

        // Compute delta vs RP-402
        let delta_vs_rp402 = if final_obj.is_finite() && rp402_obj.is_finite() {
            let delta = final_obj - rp402_obj;
            if delta < -0.001 {
                improved_vs_rp402 += 1;
                total_improvement_vs_rp402 += -delta;
                format!("{:+.2}", delta)
            } else if delta > 0.001 {
                regressed_vs_rp402 += 1;
                format!("{:+.2}", delta)
            } else {
                unchanged_vs_rp402 += 1;
                "=".to_string()
            }
        } else if final_obj.is_finite() && rp402_obj.is_infinite() {
            improved_vs_rp402 += 1;
            "inf→fin".to_string()
        } else if final_obj.is_infinite() && rp402_obj.is_finite() {
            regressed_vs_rp402 += 1;
            "fin→inf".to_string()
        } else {
            unchanged_vs_rp402 += 1;
            "both inf".to_string()
        };

        let elapsed_ms = t_start.elapsed().as_millis();
        let obj_str = if final_obj.is_finite() {
            format!("{:.4}", final_obj)
        } else {
            "inf".to_string()
        };
        let rp402_str = if rp402_obj.is_finite() {
            format!("{:.4}", rp402_obj)
        } else {
            "inf".to_string()
        };
        let obj_401c_str = if obj_401c.is_finite() {
            format!("{:.4}", obj_401c)
        } else {
            "inf".to_string()
        };
        let obj_401d_str = if obj_401d.is_finite() {
            format!("{:.4}", obj_401d)
        } else {
            "inf".to_string()
        };

        println!(
            "{:<10} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8} {:>8}",
            format!("setA-{}", inst),
            obj_str,
            rp402_str,
            obj_401c_str,
            obj_401d_str,
            selected_label,
            budget_t1,
            elapsed_ms
        );

        // Write solution file
        let out_path = format!("{}/setA-{}-srpaths-rp403.json", set_dir, inst);
        let sol_json = serde_json::json!({
            "srpaths": output_srpaths.iter().map(|p| serde_json::json!({
                "d": p.d,
                "t": p.t,
                "w": p.w
            })).collect::<Vec<_>>()
        });
        let mut f = File::create(&out_path)?;
        writeln!(f, "{}", serde_json::to_string_pretty(&sol_json)?)?;

        results.push((
            format!("setA-{}", inst),
            final_obj,
            rp402_obj,
            obj_401c,
            obj_401d,
            selected_label,
        ));
    }

    println!("{}", "=".repeat(90));
    println!(
        "RP-403 vs RP-402: {} improved, {} regressed, {} unchanged",
        improved_vs_rp402, regressed_vs_rp402, unchanged_vs_rp402
    );
    println!(
        "Total improvement vs RP-402: {:.4}",
        total_improvement_vs_rp402
    );
    println!("Finite solutions: {}/20", finite_count);
    println!("K candidates (RP-401D): {}", k_candidates);
    println!(
        "Solution files written to {}/setA-*-srpaths-rp403.json",
        set_dir
    );

    // Print benchmark summary table
    println!();
    println!("=== BENCHMARK REPORT: RP-402 vs RP-403 Construction Portfolio ===");
    println!(
        "{:<10} {:>12} {:>12} {:>10} {:>10} {:>8}",
        "Instance", "RP-403 obj", "RP-402 obj", "RP-401C", "RP-401D", "selected"
    );
    println!("{}", "-".repeat(70));
    for (inst, obj403, obj402, obj401c, obj401d, sel) in &results {
        let s403 = if obj403.is_finite() {
            format!("{:.4}", obj403)
        } else {
            "inf".to_string()
        };
        let s402 = if obj402.is_finite() {
            format!("{:.4}", obj402)
        } else {
            "inf".to_string()
        };
        let s401c = if obj401c.is_finite() {
            format!("{:.4}", obj401c)
        } else {
            "inf".to_string()
        };
        let s401d = if obj401d.is_finite() {
            format!("{:.4}", obj401d)
        } else {
            "inf".to_string()
        };
        println!(
            "{:<10} {:>12} {:>12} {:>10} {:>10} {:>8}",
            inst, s403, s402, s401c, s401d, sel
        );
    }
    println!("{}", "=".repeat(70));

    Ok(())
}
