/// rp402_budget_adapt — RP-402: Budget-Aware t=1 Adaptation
///
/// Research question: Can budget-aware transition planning recover additional
/// feasible solutions or improve objective values on the remaining infeasible
/// instances after RP-401?
///
/// Hypothesis: For instances with budget > 0, selectively re-routing the demands
/// with the largest traffic change between t=0 and t=1 (|v[1] - v[0]|) will
/// reduce t=1 objective without violating the budget constraint.
///
/// Target instances (remaining infeasible after RP-401):
///   setA-02, setA-07, setA-09, setA-12, setA-17
///
/// Approach:
///   1. Build a shared path for all demands using RP-401C ECMP-aware greedy
///      construction (same path for t=0 and t=1 → budget cost = 0).
///   2. Identify demands where |v[1] - v[0]| is largest (highest traffic change).
///   3. For each such demand (sorted by traffic change descending):
///      a. Generate K candidate t=1 paths using ECMP-aware Dijkstra.
///      b. For each candidate, compute the budget cost of switching from the
///         shared path to this candidate (SrPathBit::dist).
///      c. If budget_remaining >= dist_cost, evaluate the candidate and accept
///         if it improves t=1 MLU. Deduct dist_cost from budget_remaining.
///      d. Stop when budget is exhausted or all high-change demands are processed.
///   4. Evaluate the full solution (t=0 shared + t=1 adapted).
///
/// Discipline: One hypothesis, one capability, one evidence record.
/// This binary measures whether budget-aware t=1 adaptation is the capability
/// that recovers the remaining infeasible instances.
///
/// Classification: Research binary (not a Candidate or Competition Submission).

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;
use roadef::path::SrPathBit;

// ---------------------------------------------------------------------------
// Dijkstra shortest path with optional link metric perturbation.
// Reused from RP-401C/D.
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
        adj.entry(link.from).or_default().push((link.to, link.metric * mult));
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
// RP-401C ECMP-aware greedy construction (shared path for t=0 and t=1).
//
// Builds one path per demand that works for both time slots.
// Budget cost = 0 for all demands (same path at t=0 and t=1).
// ---------------------------------------------------------------------------
fn build_shared_paths(
    net: &Network,
    evaluator: &RoadefEvaluator,
    demands: &[(usize, u64, u64, f64)], // (d_idx, src, dst, avg_vol)
    disabled_links: &HashSet<u64>,
    max_segments: usize,
    deadline: std::time::Instant,
) -> HashMap<usize, Vec<u64>> {
    let mut sorted: Vec<(usize, u64, u64, f64)> = demands.to_vec();
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let mut partial_srpaths: Vec<SrPath> = Vec::new();
    let mut assignments: HashMap<usize, Vec<u64>> = HashMap::new();

    // Track ECMP saturation incrementally
    let mut ecmp_saturation: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        ecmp_saturation.insert(link.id, 0.0);
    }
    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
    }

    for (d_idx, src, dst, _vol) in &sorted {
        if std::time::Instant::now() >= deadline {
            break;
        }

        // Build load-aware metric multipliers from current ECMP saturation
        let load_mult: HashMap<u64, f64> = net.links.iter()
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

        if let Some(path) = dijkstra_path(net, *src, *dst, disabled_links, &load_mult) {
            let wp = path_to_waypoints(&path, max_segments);
            if !wp.is_empty() {
                partial_srpaths.push(SrPath { d: *d_idx, t: 0, w: wp.clone() });
            }
            assignments.insert(*d_idx, wp);

            // Update ECMP saturations
            let partial_sol = Solution { srpaths: partial_srpaths.clone() };
            if let Some(loads) = evaluator.compute_loads(0, &partial_sol) {
                ecmp_saturation.clear();
                for (arc_id, flow) in &loads.arc_flows {
                    let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
                    ecmp_saturation.insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
                }
            }
        }
    }

    assignments
}

// ---------------------------------------------------------------------------
// RP-402: Budget-aware t=1 adaptation.
//
// Given the shared path assignments (t=0 = t=1 = shared), identify demands
// with the largest traffic change |v[1] - v[0]| and attempt to re-route them
// for t=1 only, within the budget constraint.
//
// Returns: map from d_idx to t=1 waypoints (may differ from shared path).
// ---------------------------------------------------------------------------
fn adapt_t1_within_budget(
    net: &Network,
    evaluator: &RoadefEvaluator,
    shared_assignments: &HashMap<usize, Vec<u64>>,
    demands_with_delta: &[(usize, u64, u64, f64, f64)], // (d_idx, src, dst, v0, v1)
    disabled_t1: &HashSet<u64>,
    max_segments: usize,
    budget_t1: usize,
    deadline: std::time::Instant,
) -> HashMap<usize, Vec<u64>> {
    // Start with all demands using the shared path for t=1
    let mut t1_assignments: HashMap<usize, Vec<u64>> = shared_assignments.clone();
    let mut budget_remaining = budget_t1;

    // Sort demands by |v[1] - v[0]| descending — highest traffic change first
    let mut sorted: Vec<(usize, u64, u64, f64, f64)> = demands_with_delta.to_vec();
    sorted.sort_by(|a, b| {
        let da = (a.4 - a.3).abs();
        let db = (b.4 - b.3).abs();
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Build current t=1 ECMP saturation from shared paths
    let mut ecmp_saturation_t1: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        ecmp_saturation_t1.insert(link.id, 0.0);
    }
    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
    }

    // Compute initial t=1 saturation from shared paths
    let initial_t1_srpaths: Vec<SrPath> = shared_assignments.iter()
        .filter(|(_, wp)| !wp.is_empty())
        .map(|(&d_idx, wp)| SrPath { d: d_idx, t: 1, w: wp.clone() })
        .collect();
    let initial_t1_sol = Solution { srpaths: initial_t1_srpaths };
    if let Some(loads) = evaluator.compute_loads(1, &initial_t1_sol) {
        for (arc_id, flow) in &loads.arc_flows {
            let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
            ecmp_saturation_t1.insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
        }
    }

    for (d_idx, src, dst, _v0, _v1) in &sorted {
        if std::time::Instant::now() >= deadline {
            break;
        }
        if budget_remaining == 0 {
            break;
        }

        // Current shared path for this demand
        let shared_wp = shared_assignments.get(d_idx).cloned().unwrap_or_default();
        let shared_bit = SrPathBit::new_explicit(*src, *dst, &shared_wp);

        // Build load-aware metric multipliers for t=1
        let load_mult: HashMap<u64, f64> = net.links.iter()
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

        // Generate candidate t=1 path
        let candidate_wp = match dijkstra_path(net, *src, *dst, disabled_t1, &load_mult) {
            Some(path) => path_to_waypoints(&path, max_segments),
            None => continue,
        };

        // Skip if candidate is identical to shared path (no budget cost, no benefit)
        if candidate_wp == shared_wp {
            continue;
        }

        // Compute budget cost of switching to this candidate
        let candidate_bit = SrPathBit::new_explicit(*src, *dst, &candidate_wp);
        let switch_cost = candidate_bit.dist(&shared_bit);

        if switch_cost > budget_remaining {
            // Too expensive — skip this demand
            continue;
        }

        // Evaluate: build trial t=1 solution with this candidate
        let mut trial_t1_srpaths: Vec<SrPath> = t1_assignments.iter()
            .filter(|(_, wp)| !wp.is_empty())
            .map(|(&di, wp)| SrPath { d: di, t: 1, w: wp.clone() })
            .collect();

        // Replace this demand's t=1 path with the candidate
        trial_t1_srpaths.retain(|p| p.d != *d_idx);
        if !candidate_wp.is_empty() {
            trial_t1_srpaths.push(SrPath { d: *d_idx, t: 1, w: candidate_wp.clone() });
        }

        let trial_sol = Solution { srpaths: trial_t1_srpaths.clone() };
        if let Some(loads) = evaluator.compute_loads(1, &trial_sol) {
            // Accept if t=1 MLU improves
            let current_t1_sol = Solution {
                srpaths: t1_assignments.iter()
                    .filter(|(_, wp)| !wp.is_empty())
                    .map(|(&di, wp)| SrPath { d: di, t: 1, w: wp.clone() })
                    .collect()
            };
            let current_mlu = evaluator.compute_loads(1, &current_t1_sol)
                .map(|l| l.mlu)
                .unwrap_or(f64::INFINITY);

            if loads.mlu < current_mlu {
                // Accept the switch
                t1_assignments.insert(*d_idx, candidate_wp.clone());
                budget_remaining -= switch_cost;

                // Update t=1 ECMP saturation
                ecmp_saturation_t1.clear();
                for (arc_id, flow) in &loads.arc_flows {
                    let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
                    ecmp_saturation_t1.insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
                }
            }
        }
    }

    t1_assignments
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";

    println!("RP-402 — Budget-Aware t=1 Adaptation (Dataset A)");
    println!("{}", "=".repeat(80));
    println!("{:<10} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8}",
        "Instance", "RP-402 obj", "RP-401C obj", "Empty obj", "vs Empty", "budget", "ms");
    println!("{}", "-".repeat(80));

    let mut improved_vs_empty = 0usize;
    let mut total_improvement = 0.0f64;
    let mut new_finite = 0usize;

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
        let max_seg = if scenario.max_segments >= 0 { scenario.max_segments as usize } else { 100 };
        let budget_t1 = scenario.budget.iter().find(|b| b.t == 1).map(|b| b.value).unwrap_or(0);

        let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario.clone());

        // Disabled links per time slot
        let disabled_t0: HashSet<u64> = scenario.interventions.iter()
            .filter(|i| i.t == 0)
            .flat_map(|i| i.links.iter().copied())
            .collect();
        let disabled_t1: HashSet<u64> = scenario.interventions.iter()
            .filter(|i| i.t == 1)
            .flat_map(|i| i.links.iter().copied())
            .collect();
        let disabled_both: HashSet<u64> = disabled_t0.union(&disabled_t1).copied().collect();

        let t_start = Instant::now();
        let deadline = t_start + std::time::Duration::from_secs(300);

        // Step 1: Build shared paths (RP-401C ECMP-aware greedy, t=0 and t=1 identical)
        let demands_avg: Vec<(usize, u64, u64, f64)> = tm.demands.iter().enumerate()
            .map(|(i, d)| {
                let v0 = d.v[0];
                let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
                (i, d.s, d.t, (v0 + v1) / 2.0)
            })
            .collect();

        let shared_assignments = build_shared_paths(
            &net, &evaluator, &demands_avg, &disabled_both, max_seg, deadline,
        );

        // Step 2: Budget-aware t=1 adaptation (only if budget > 0 and 2 time slots)
        let t1_assignments = if num_slots > 1 && budget_t1 > 0 {
            let demands_with_delta: Vec<(usize, u64, u64, f64, f64)> = tm.demands.iter().enumerate()
                .map(|(i, d)| {
                    let v0 = d.v[0];
                    let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
                    (i, d.s, d.t, v0, v1)
                })
                .collect();

            adapt_t1_within_budget(
                &net, &evaluator, &shared_assignments, &demands_with_delta,
                &disabled_t1, max_seg, budget_t1, deadline,
            )
        } else {
            shared_assignments.clone()
        };

        // Step 3: Build final srpaths
        let mut srpaths: Vec<SrPath> = Vec::new();
        for d_idx in 0..num_demands {
            // t=0: always use shared path
            if let Some(w) = shared_assignments.get(&d_idx) {
                if !w.is_empty() {
                    srpaths.push(SrPath { d: d_idx, t: 0, w: w.clone() });
                }
            }
            // t=1: use adapted path (may differ from shared)
            if num_slots > 1 {
                if let Some(w) = t1_assignments.get(&d_idx) {
                    if !w.is_empty() {
                        srpaths.push(SrPath { d: d_idx, t: 1, w: w.clone() });
                    }
                }
            }
        }

        // Step 4: Evaluate
        let solution = Solution { srpaths: srpaths.clone() };
        let result = evaluator.evaluate_solution(&solution);
        let empty_sol = Solution { srpaths: vec![] };
        let empty_result = evaluator.evaluate_solution(&empty_sol);

        // Load RP-401C result for comparison
        let rp401c_path = format!("{}/setA-{}-srpaths-rp401c.json", set_dir, inst);
        let rp401c_obj_str = if std::path::Path::new(&rp401c_path).exists() {
            "see file".to_string()
        } else {
            "—".to_string()
        };

        let (final_srpaths, final_obj) = if !result.valid {
            (vec![], empty_result.obj)
        } else if result.obj.is_finite() && (empty_result.obj.is_infinite() || result.obj <= empty_result.obj) {
            (srpaths, result.obj)
        } else if result.obj.is_infinite() && empty_result.obj.is_finite() {
            (vec![], empty_result.obj)
        } else if result.obj.is_infinite() && empty_result.obj.is_infinite() {
            (srpaths, result.obj)
        } else {
            (vec![], empty_result.obj)
        };

        let delta_str = if final_obj.is_finite() && empty_result.obj.is_finite() {
            let delta = final_obj - empty_result.obj;
            if delta < -0.001 {
                improved_vs_empty += 1;
                total_improvement += -delta;
            }
            format!("{:+.2}", delta)
        } else {
            "n/a".to_string()
        };

        // Track new finite solutions (instances that were inf in RP-401)
        let target_instances = ["02", "07", "09", "12", "17"];
        if target_instances.contains(&inst.as_str()) && final_obj.is_finite() {
            new_finite += 1;
        }

        let elapsed_ms = t_start.elapsed().as_millis();
        let obj_str = if final_obj.is_finite() { format!("{:.4}", final_obj) } else { "inf".to_string() };
        let empty_str = if empty_result.obj.is_finite() { format!("{:.4}", empty_result.obj) } else { "inf".to_string() };

        println!("{:<10} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8}",
            format!("setA-{}", inst), obj_str, rp401c_obj_str, empty_str, delta_str, budget_t1, elapsed_ms);

        // Write solution file
        let out_path = format!("{}/setA-{}-srpaths-rp402.json", set_dir, inst);
        let sol_json = serde_json::json!({
            "srpaths": final_srpaths.iter().map(|p| serde_json::json!({
                "d": p.d,
                "t": p.t,
                "w": p.w
            })).collect::<Vec<_>>()
        });
        let mut f = File::create(&out_path)?;
        writeln!(f, "{}", serde_json::to_string_pretty(&sol_json)?)?;
    }

    println!("{}", "=".repeat(80));
    println!("Instances improved vs empty: {}", improved_vs_empty);
    println!("Total objective improvement vs empty: {:.4}", total_improvement);
    println!("New finite solutions on target instances (setA-02,07,09,12,17): {}/5", new_finite);
    println!("Solution files written to {}/setA-*-srpaths-rp402.json", set_dir);

    Ok(())
}