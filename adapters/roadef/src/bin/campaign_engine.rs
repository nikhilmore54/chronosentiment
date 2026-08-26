use std::cmp::Reverse;
/// campaign_engine — ROADEF 2026 Dataset A Submission Generator
///
/// Runs on all 20 setA instances and writes srpaths.json solution files.
///
/// Key insight from path.rs:
///   Budget cost at t=1 = sum over demands of dist(t1_path, t0_path)
///   dist(uninitialized, explicit(len=N)) = N  (expensive!)
///   dist(explicit_A, explicit_A) = 0          (free — same path)
///   dist(uninitialized, uninitialized) = 0     (free — both default)
///
/// Strategy:
///   1. Find one good srpath per demand that works for BOTH time slots.
///   2. Emit it for both t=0 and t=1 → budget cost = 0 for all demands.
///   3. For demands affected by t=1 interventions, find an alternative path
///      and emit it only for t=1, within the budget limit.
///
/// The empty solution (srpaths=[]) is always valid. This solver attempts to
/// improve on it by steering traffic away from saturated links.
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::Write;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};

// ---------------------------------------------------------------------------
// Dijkstra shortest path (metric-weighted, ignoring disabled arcs)
// Returns the full path as a list of node IDs from src to dst (inclusive).
// Returns None if no path exists.
// ---------------------------------------------------------------------------
fn dijkstra_path(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
) -> Option<Vec<u64>> {
    if src == dst {
        return Some(vec![src]);
    }

    // Build adjacency: node_id -> Vec<(neighbor_id, metric)>
    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) {
            continue;
        }
        adj.entry(link.from)
            .or_default()
            .push((link.to, link.metric));
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
                let better = dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost;
                if better {
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
// Load-aware Dijkstra: weights links by (metric + load_penalty * saturation)
// ---------------------------------------------------------------------------
fn load_aware_path(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
    link_saturation: &HashMap<u64, f64>,
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
        let sat = link_saturation.get(&link.id).copied().unwrap_or(0.0);
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
            for &(next, effective_metric) in neighbors {
                let new_cost = cost + (effective_metric * 1000.0) as u64;
                let better = dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost;
                if better {
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
// Extract waypoints from a full node path (intermediate nodes only)
// and enforce max_segments constraint.
// ---------------------------------------------------------------------------
fn path_to_waypoints(full_path: &[u64], max_segments: usize) -> Vec<u64> {
    if full_path.len() <= 2 {
        return vec![];
    }
    let waypoints: Vec<u64> = full_path[1..full_path.len() - 1].to_vec();
    // max_segments constraint: waypoints.len() + 1 <= max_segments
    // i.e. waypoints.len() <= max_segments - 1
    if max_segments > 0 && waypoints.len() + 1 > max_segments {
        waypoints[..max_segments - 1].to_vec()
    } else {
        waypoints
    }
}

// ---------------------------------------------------------------------------
// Greedy load-balanced solver for one time slot.
// Returns: HashMap<demand_idx, waypoints>
// Only includes demands where we found a non-trivial path.
// ---------------------------------------------------------------------------
fn solve_greedy(
    net: &Network,
    demands: &[(usize, u64, u64, f64)], // (demand_idx, src, dst, vol)
    disabled_links: &HashSet<u64>,
    max_segments: usize,
) -> HashMap<usize, Vec<u64>> {
    // Sort demands by volume descending
    let mut sorted: Vec<(usize, u64, u64, f64)> = demands.to_vec();
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    // Build link lookup: link_id -> capacity
    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    let mut link_flow: HashMap<u64, f64> = HashMap::new();
    let mut link_saturation: HashMap<u64, f64> = HashMap::new();
    // Build (from, to) -> link_id for flow tracking
    let mut link_by_endpoints: HashMap<(u64, u64), u64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
        link_flow.insert(link.id, 0.0);
        link_saturation.insert(link.id, 0.0);
        link_by_endpoints.insert((link.from, link.to), link.id);
    }

    let mut assignments: HashMap<usize, Vec<u64>> = HashMap::new();

    for (d_idx, src, dst, vol) in &sorted {
        let full_path = load_aware_path(net, *src, *dst, disabled_links, &link_saturation, 100.0)
            .or_else(|| dijkstra_path(net, *src, *dst, disabled_links));

        if let Some(fp) = full_path {
            let waypoints = path_to_waypoints(&fp, max_segments);

            // Update link flows using the full path
            for j in 0..fp.len().saturating_sub(1) {
                if let Some(&link_id) = link_by_endpoints.get(&(fp[j], fp[j + 1])) {
                    let flow = link_flow.entry(link_id).or_insert(0.0);
                    *flow += vol;
                    let cap = link_capacity.get(&link_id).copied().unwrap_or(1.0);
                    link_saturation.insert(link_id, *flow / cap);
                }
            }

            assignments.insert(*d_idx, waypoints);
        }
        // If no path found, don't insert — demand will use ECMP default
    }

    assignments
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";

    println!("ROADEF 2026 — Dataset A Submission Generator");
    println!("{}", "=".repeat(60));

    for instance_id in 1..=20 {
        let inst = format!("{:02}", instance_id);
        let net_path = format!("{}/setA-{}-net.json", set_dir, inst);
        let tm_path = format!("{}/setA-{}-tm.json", set_dir, inst);
        let sc_path = format!("{}/setA-{}-scenario.json", set_dir, inst);
        let out_path = format!("{}/setA-{}-srpaths.json", set_dir, inst);

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

        print!(
            "setA-{}: {} nodes, {} links, {} demands, {} slots, budget_t1={} ... ",
            inst,
            net.nodes.len(),
            net.links.len(),
            num_demands,
            num_slots,
            budget_t1
        );

        // Disabled links at each time slot
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

        // Build demand lists for each time slot
        let demands_t0: Vec<(usize, u64, u64, f64)> = tm
            .demands
            .iter()
            .enumerate()
            .map(|(i, d)| (i, d.s, d.t, d.v[0]))
            .collect();
        let demands_t1: Vec<(usize, u64, u64, f64)> = tm
            .demands
            .iter()
            .enumerate()
            .map(|(i, d)| (i, d.s, d.t, if d.v.len() > 1 { d.v[1] } else { d.v[0] }))
            .collect();

        // --- Strategy: find paths that work for both time slots ---
        // Use the union of disabled links as the constraint for the "shared" path,
        // so the path is valid in both slots.
        let disabled_both: HashSet<u64> = disabled_t0.union(&disabled_t1).copied().collect();

        // Use average volume for routing priority
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

        let shared_assign = solve_greedy(&net, &demands_avg, &disabled_both, max_seg);

        // Build srpaths: emit the shared path for BOTH t=0 and t=1
        // This guarantees budget cost = 0 for all demands using shared paths.
        let mut srpaths: Vec<SrPath> = Vec::new();

        for d_idx in 0..num_demands {
            if let Some(w) = shared_assign.get(&d_idx) {
                if !w.is_empty() {
                    // Emit same waypoints for both t=0 and t=1
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

        // Validate and compare against empty solution
        let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario.clone());
        let solution = Solution {
            srpaths: srpaths.clone(),
        };
        let result = evaluator.evaluate_solution(&solution);

        let empty_sol = Solution { srpaths: vec![] };
        let empty_result = evaluator.evaluate_solution(&empty_sol);

        let final_srpaths = if !result.valid {
            // Our solution is structurally invalid (budget/segment violation) — use empty
            println!("INVALID → empty (obj={:.4})", empty_result.obj);
            vec![]
        } else if result.obj.is_finite()
            && (empty_result.obj.is_infinite() || result.obj <= empty_result.obj)
        {
            // Our solution is finite and better than or equal to empty
            println!("obj={:.4} (empty={:.4})", result.obj, empty_result.obj);
            srpaths
        } else if result.obj.is_infinite() && empty_result.obj.is_finite() {
            // Our solution is inf but empty is finite — use empty
            println!("inf → empty (obj={:.4})", empty_result.obj);
            vec![]
        } else if result.obj.is_infinite() && empty_result.obj.is_infinite() {
            // Both inf — keep our solution (it may still be better on some metric)
            println!("obj=inf (empty=inf, keeping ours)");
            srpaths
        } else {
            // Our solution is worse than empty — use empty
            println!(
                "obj={:.4} worse than empty={:.4} → using empty",
                result.obj, empty_result.obj
            );
            vec![]
        };

        // Write solution
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

    println!("{}", "=".repeat(60));
    println!("Done. Solution files written to {}", set_dir);
    Ok(())
}
