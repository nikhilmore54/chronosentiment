/// rp401d_ecmp_path_selection — RP-401D: ECMP-aware path selection
///
/// Research question: Does selecting paths by minimising the ECMP-oracle MLU
/// increase (rather than using a penalty-weighted Dijkstra) further improve
/// solution quality on Dataset A?
///
/// Difference from RP-401C:
///   RP-401C: Uses ECMP-oracle saturations as input to load-aware Dijkstra.
///            Path selection is still driven by a penalty-weighted metric.
///   RP-401D: Generates K candidate paths per demand (K-shortest paths via
///            Yen's algorithm approximation: perturbed metrics), evaluates
///            each candidate by calling compute_loads() on the partial
///            solution with that candidate inserted, and selects the
///            candidate that minimises the resulting MLU.
///
/// This is a measurement experiment. The key question is whether oracle-guided
/// path selection outperforms penalty-guided selection, and by how much.
///
/// Classification: Research binary (not a Candidate or Competition Submission).

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
use std::fs::File;
use std::io::Write;

use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;

// ---------------------------------------------------------------------------
// Dijkstra shortest path with optional link metric perturbation.
// Used to generate diverse candidate paths.
// ---------------------------------------------------------------------------
fn dijkstra_path_perturbed(
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
// Generate K diverse candidate paths for a demand.
//
// Strategy: run Dijkstra K times with different metric perturbations.
//   - Perturbation 0: unperturbed (shortest path)
//   - Perturbation 1: multiply metrics by ECMP saturation (load-aware)
//   - Perturbations 2..K-1: randomly inflate individual links to force
//     the solver to explore alternative routes
//
// This is a lightweight approximation of K-shortest paths. It does not
// guarantee K distinct paths but produces diverse candidates in practice.
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
    if let Some(path) = dijkstra_path_perturbed(net, src, dst, disabled_links, &no_mult) {
        let wp = path_to_waypoints(&path, max_segments);
        if seen_paths.insert(wp.clone()) {
            candidates.push(wp);
        }
    }

    // Candidate 1: load-aware (penalise saturated links)
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
    if let Some(path) = dijkstra_path_perturbed(net, src, dst, disabled_links, &load_mult) {
        let wp = path_to_waypoints(&path, max_segments);
        if seen_paths.insert(wp.clone()) {
            candidates.push(wp);
        }
    }

    // Candidates 2..K-1: inflate individual high-saturation links
    // Sort links by saturation descending; inflate top links one at a time
    let mut sorted_links: Vec<(u64, f64)> = ecmp_saturation.iter()
        .filter(|(id, _)| !disabled_links.contains(id))
        .map(|(&id, &sat)| (id, sat))
        .collect();
    sorted_links.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (i, (link_id, _sat)) in sorted_links.iter().enumerate().take(k.saturating_sub(2)) {
        let mut mult: HashMap<u64, f64> = HashMap::new();
        // Inflate this link and the previous i links
        for j in 0..=i {
            if j < sorted_links.len() {
                mult.insert(sorted_links[j].0, 1e4);
            }
        }
        if let Some(path) = dijkstra_path_perturbed(net, src, dst, disabled_links, &mult) {
            let wp = path_to_waypoints(&path, max_segments);
            if seen_paths.insert(wp.clone()) {
                candidates.push(wp);
                if candidates.len() >= k {
                    break;
                }
            }
        }
        let _ = link_id; // suppress unused warning
    }

    candidates
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
// RP-401D greedy solver: oracle-guided path selection.
//
// For each demand (sorted by volume descending):
//   1. Generate K candidate waypoint sets.
//   2. For each candidate, build a trial partial solution and call
//      compute_loads() to get the resulting MLU.
//   3. Select the candidate with the lowest post-assignment MLU.
//   4. Commit that candidate to the partial solution.
//   5. Update ECMP saturations for the next demand.
//
// K=5 candidates per demand. Total oracle calls: O(D × K).
// ---------------------------------------------------------------------------
fn solve_greedy_oracle(
    net: &Network,
    evaluator: &RoadefEvaluator,
    demands: &[(usize, u64, u64, f64)],
    disabled_links: &HashSet<u64>,
    time_slot: usize,
    max_segments: usize,
    k_candidates: usize,
) -> HashMap<usize, Vec<u64>> {
    let mut sorted: Vec<(usize, u64, u64, f64)> = demands.to_vec();
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let mut partial_srpaths: Vec<SrPath> = Vec::new();
    let mut assignments: HashMap<usize, Vec<u64>> = HashMap::new();

    // Initial ECMP saturation: empty solution
    let mut ecmp_saturation: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        ecmp_saturation.insert(link.id, 0.0);
    }
    let mut link_capacity: HashMap<u64, f64> = HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
    }

    for (d_idx, src, dst, _vol) in &sorted {
        // Generate K candidate paths
        let candidates = generate_candidates(
            net, *src, *dst, disabled_links, &ecmp_saturation, k_candidates, max_segments,
        );

        if candidates.is_empty() {
            continue;
        }

        // Evaluate each candidate: pick the one with lowest resulting MLU
        let mut best_waypoints: Option<Vec<u64>> = None;
        let mut best_mlu = f64::INFINITY;

        for waypoints in &candidates {
            // Build trial partial solution with this candidate
            let mut trial_srpaths = partial_srpaths.clone();
            if !waypoints.is_empty() {
                trial_srpaths.push(SrPath { d: *d_idx, t: time_slot, w: waypoints.clone() });
            }
            let trial_sol = Solution { srpaths: trial_srpaths };

            if let Some(loads) = evaluator.compute_loads(time_slot, &trial_sol) {
                if loads.mlu < best_mlu {
                    best_mlu = loads.mlu;
                    best_waypoints = Some(waypoints.clone());
                }
            }
        }

        // Commit the best candidate
        if let Some(wp) = best_waypoints {
            if !wp.is_empty() {
                partial_srpaths.push(SrPath { d: *d_idx, t: time_slot, w: wp.clone() });
            }
            assignments.insert(*d_idx, wp);

            // Update ECMP saturations
            let partial_sol = Solution { srpaths: partial_srpaths.clone() };
            if let Some(loads) = evaluator.compute_loads(time_slot, &partial_sol) {
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
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
    let k_candidates = 5;

    println!("RP-401D — ECMP Oracle-Guided Path Selection (Dataset A)");
    println!("{}", "=".repeat(65));
    println!("{:<10} {:>12} {:>12} {:>12} {:>10}",
        "Instance", "RP-401D obj", "RP-401C obj", "Empty obj", "vs Empty");
    println!("{}", "-".repeat(60));

    let mut improved_vs_empty = 0usize;
    let mut total_improvement = 0.0f64;

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

        // Disabled links
        let disabled_t0: HashSet<u64> = scenario.interventions.iter()
            .filter(|i| i.t == 0)
            .flat_map(|i| i.links.iter().copied())
            .collect();
        let disabled_t1: HashSet<u64> = scenario.interventions.iter()
            .filter(|i| i.t == 1)
            .flat_map(|i| i.links.iter().copied())
            .collect();
        let disabled_both: HashSet<u64> = disabled_t0.union(&disabled_t1).copied().collect();

        // Average volume demands
        let demands_avg: Vec<(usize, u64, u64, f64)> = tm.demands.iter().enumerate()
            .map(|(i, d)| {
                let v0 = d.v[0];
                let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
                (i, d.s, d.t, (v0 + v1) / 2.0)
            })
            .collect();

        // RP-401D: oracle-guided path selection
        let shared_assign = solve_greedy_oracle(
            &net, &evaluator, &demands_avg, &disabled_both, 0, max_seg, k_candidates,
        );

        // Build srpaths
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

        // Evaluate
        let solution = Solution { srpaths: srpaths.clone() };
        let result = evaluator.evaluate_solution(&solution);
        let empty_sol = Solution { srpaths: vec![] };
        let empty_result = evaluator.evaluate_solution(&empty_sol);

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

        let obj_str = if final_obj.is_finite() { format!("{:.4}", final_obj) } else { "inf".to_string() };
        let empty_str = if empty_result.obj.is_finite() { format!("{:.4}", empty_result.obj) } else { "inf".to_string() };

        // Load RP-401C result for comparison if available
        let rp401c_path = format!("{}/setA-{}-srpaths-rp401c.json", set_dir, inst);
        let rp401c_str = if std::path::Path::new(&rp401c_path).exists() {
            // We can't easily re-evaluate here without re-running; show placeholder
            "see rp401c".to_string()
        } else {
            "—".to_string()
        };

        println!("{:<10} {:>12} {:>12} {:>12} {:>10}",
            format!("setA-{}", inst), obj_str, rp401c_str, empty_str, delta_str);

        // Write solution file
        let out_path = format!("{}/setA-{}-srpaths-rp401d.json", set_dir, inst);
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

    println!("{}", "=".repeat(65));
    println!("Instances improved vs empty: {}", improved_vs_empty);
    println!("Total objective improvement vs empty: {:.4}", total_improvement);
    println!("K candidates per demand: {}", k_candidates);
    println!("Solution files written to {}/setA-*-srpaths-rp401d.json", set_dir);

    Ok(())
}