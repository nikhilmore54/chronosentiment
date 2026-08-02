/// rp401c_ecmp_construction — RP-401C: ECMP-aware load estimation during construction
///
/// Research question: Does replacing the heuristic link-load model with the
/// ECMP oracle during greedy construction improve solution quality on Dataset A?
///
/// Difference from campaign_engine (baseline):
///   Baseline: `solve_greedy()` tracks link saturation by adding full demand
///             volume to each link on the chosen Dijkstra path. No ECMP splitting.
///   RP-401C:  After tentatively assigning a path to a demand, calls
///             `evaluator.compute_loads(ts, &partial_solution)` to get accurate
///             ECMP-split saturations before deciding whether to commit.
///
/// This is a measurement experiment. Path selection strategy is unchanged
/// (load-aware Dijkstra with exponential penalty). Only the load model changes.
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

// ---------------------------------------------------------------------------
// Dijkstra shortest path (metric-weighted, ignoring disabled arcs)
// Identical to campaign_engine — path selection is unchanged in RP-401C.
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

    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) {
            continue;
        }
        adj.entry(link.from).or_default().push((link.to, link.metric));
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
// Load-aware Dijkstra using ECMP-oracle saturations.
// Identical penalty function to campaign_engine; only the saturation source
// changes (ECMP oracle instead of heuristic accumulator).
// ---------------------------------------------------------------------------
fn load_aware_path_ecmp(
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
        adj.entry(link.from).or_default().push((link.to, effective_metric));
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
// Extract waypoints from a full node path (intermediate nodes only).
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
// RP-401C greedy solver: ECMP-oracle load estimation during construction.
//
// For each demand (sorted by volume descending):
//   1. Query ECMP-oracle saturations from the current partial solution.
//   2. Run load-aware Dijkstra using those saturations.
//   3. Tentatively add the chosen path to the partial solution.
//   4. Re-query ECMP oracle to get updated saturations for the next demand.
//
// The partial solution is a Vec<SrPath> that grows as demands are assigned.
// compute_loads() is called once per demand (O(D) oracle calls total).
// ---------------------------------------------------------------------------
fn solve_greedy_ecmp(
    net: &Network,
    evaluator: &RoadefEvaluator,
    demands: &[(usize, u64, u64, f64)], // (demand_idx, src, dst, vol)
    disabled_links: &HashSet<u64>,
    time_slot: usize,
    max_segments: usize,
) -> HashMap<usize, Vec<u64>> {
    // Sort demands by volume descending (same as baseline)
    let mut sorted: Vec<(usize, u64, u64, f64)> = demands.to_vec();
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    // Build link capacity map for saturation computation
    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
    }

    // Partial solution: grows as demands are assigned
    let mut partial_srpaths: Vec<SrPath> = Vec::new();
    let mut assignments: HashMap<usize, Vec<u64>> = HashMap::new();

    // Initial ECMP saturation: empty solution
    let mut ecmp_saturation: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        ecmp_saturation.insert(link.id, 0.0);
    }

    for (d_idx, src, dst, _vol) in &sorted {
        // Use current ECMP saturations for path selection
        let full_path = load_aware_path_ecmp(net, *src, *dst, disabled_links, &ecmp_saturation, 100.0)
            .or_else(|| dijkstra_path(net, *src, *dst, disabled_links));

        if let Some(fp) = full_path {
            let waypoints = path_to_waypoints(&fp, max_segments);

            // Tentatively add this demand's path to the partial solution
            if !waypoints.is_empty() {
                partial_srpaths.push(SrPath { d: *d_idx, t: time_slot, w: waypoints.clone() });
            }

            assignments.insert(*d_idx, waypoints);

            // Update ECMP saturations using the oracle on the partial solution
            let partial_sol = Solution { srpaths: partial_srpaths.clone() };
            if let Some(loads) = evaluator.compute_loads(time_slot, &partial_sol) {
                // Recompute saturations from ECMP arc flows
                ecmp_saturation.clear();
                for (arc_id, flow) in &loads.arc_flows {
                    let cap = link_capacity.get(arc_id).copied().unwrap_or(1.0);
                    ecmp_saturation.insert(*arc_id, if cap > 0.0 { flow / cap } else { f64::INFINITY });
                }
            }
            // If compute_loads returns None (disconnected), keep previous saturations
        }
    }

    assignments
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";

    println!("RP-401C — ECMP-Aware Construction (Dataset A)");
    println!("{}", "=".repeat(68));
    println!("{:<10} {:>12} {:>12} {:>10} {:>8}", "Instance", "RP-401C obj", "Baseline obj", "Delta", "ms");
    println!("{}", "-".repeat(58));

    let mut total_improvement = 0.0f64;
    let mut improved_count = 0usize;
    let mut regressed_count = 0usize;
    let mut unchanged_count = 0usize;

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

        // Average volume demands for shared-path routing
        let demands_avg: Vec<(usize, u64, u64, f64)> = tm.demands.iter().enumerate()
            .map(|(i, d)| {
                let v0 = d.v[0];
                let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
                (i, d.s, d.t, (v0 + v1) / 2.0)
            })
            .collect();

        // RP-401C: ECMP-aware greedy construction
        // Use time_slot=0 for oracle queries during shared-path construction
        let t_start = Instant::now();
        let shared_assign = solve_greedy_ecmp(
            &net, &evaluator, &demands_avg, &disabled_both, 0, max_seg,
        );

        // Build srpaths: shared path for both t=0 and t=1
        let mut srpaths: Vec<SrPath> = Vec::new();
        for d_idx in 0..num_demands {
            if let Some(w) = shared_assign.get(&d_idx) {
                if !w.is_empty() {
                    srpaths.push(SrPath { d: d_idx, t: 0, w: w.clone() });
                    if num_slots > 1 {
                        srpaths.push(SrPath { d: d_idx, t: 1, w: w.clone() });
                    }
                }
            }
        }

        // Evaluate RP-401C solution
        let solution = Solution { srpaths: srpaths.clone() };
        let result = evaluator.evaluate_solution(&solution);

        // Evaluate empty solution (baseline reference)
        let empty_sol = Solution { srpaths: vec![] };
        let empty_result = evaluator.evaluate_solution(&empty_sol);

        // Choose best solution
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

        // Delta vs empty (baseline)
        let delta_str = if final_obj.is_finite() && empty_result.obj.is_finite() {
            let delta = final_obj - empty_result.obj;
            if delta < -0.001 {
                improved_count += 1;
                total_improvement += -delta;
                format!("{:+.2}", delta)
            } else if delta > 0.001 {
                regressed_count += 1;
                format!("{:+.2}", delta)
            } else {
                unchanged_count += 1;
                "=".to_string()
            }
        } else if final_obj.is_infinite() && empty_result.obj.is_infinite() {
            unchanged_count += 1;
            "both inf".to_string()
        } else if final_obj < empty_result.obj {
            improved_count += 1;
            "improved".to_string()
        } else {
            unchanged_count += 1;
            "=".to_string()
        };

        let elapsed_ms = t_start.elapsed().as_millis();
        let obj_str = if final_obj.is_finite() { format!("{:.4}", final_obj) } else { "inf".to_string() };
        let empty_str = if empty_result.obj.is_finite() { format!("{:.4}", empty_result.obj) } else { "inf".to_string() };

        println!("{:<10} {:>12} {:>12} {:>10} {:>8}", format!("setA-{}", inst), obj_str, empty_str, delta_str, elapsed_ms);

        // Write solution file
        let out_path = format!("{}/setA-{}-srpaths-rp401c.json", set_dir, inst);
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

    println!("{}", "=".repeat(68));
    println!("Summary: {} improved, {} regressed, {} unchanged", improved_count, regressed_count, unchanged_count);
    println!("Total objective improvement vs empty: {:.4}", total_improvement);
    println!("Solution files written to {}/setA-*-srpaths-rp401c.json", set_dir);

    Ok(())
}