// roadef_qualification.rs – Qualification entry point for Set‑B instances
// Accepts: NET TM SCENARIO OUTPUT
// Uses the exact frozen per‑instance solver pipeline from campaign_engine.rs

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::env;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};

// ---------------------------------------------------------------------------
// Per‑instance solver (copied from campaign_engine.rs without modification)
// ---------------------------------------------------------------------------
fn run_instance(net: &Network, tm: &roadef::models::TrafficMatrix, scenario: &roadef::models::Scenario, out_path: &str) -> anyhow::Result<()> {
    // Derived parameters
    let num_demands = tm.demands.len();
    let num_slots = tm.num_time_slots;
    let max_seg = if scenario.max_segments >= 0 {
        scenario.max_segments as usize
    } else {
        100
    };
    let _budget_t1 = scenario.budget.iter().find(|b| b.t == 1).map(|b| b.value).unwrap_or(0);

    // Disabled links across all time slots
    let disabled_all: HashSet<u64> = scenario.interventions.iter().flat_map(|i| i.links.iter().copied()).collect();

    // Greedy shared assignment with per-slot flow tracking and bottleneck rip-up & reroute
    let shared_assign = solve_greedy(&net, &tm.demands, num_slots, &disabled_all, max_seg);

    // Build srpaths for all time slots (identical waypoints ensure 0 reconfiguration distance)
    let mut srpaths: Vec<SrPath> = Vec::new();
    for d_idx in 0..num_demands {
        if let Some(w) = shared_assign.get(&d_idx) {
            if !w.is_empty() {
                for slot in 0..num_slots {
                    srpaths.push(SrPath { d: d_idx, t: slot, w: w.clone() });
                }
            }
        }
    }

    // Evaluate – identical to the original per‑instance evaluation
    let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario.clone());
    let solution = Solution { srpaths: srpaths.clone() };
    let result = evaluator.evaluate_solution(&solution);
    let empty_sol = Solution { srpaths: vec![] };
    let empty_result = evaluator.evaluate_solution(&empty_sol);

    let final_srpaths = if !result.valid {
        println!("INVALID → empty (obj={:.4})", empty_result.obj);
        vec![]
    } else if result.obj.is_finite() && (empty_result.obj.is_infinite() || result.obj <= empty_result.obj) {
        println!("obj={:.4} (empty={:.4})", result.obj, empty_result.obj);
        srpaths
    } else if result.obj.is_infinite() && empty_result.obj.is_finite() {
        println!("inf → empty (obj={:.4})", empty_result.obj);
        vec![]
    } else if result.obj.is_infinite() && empty_result.obj.is_infinite() {
        println!("obj=inf (empty=inf, keeping ours)");
        srpaths
    } else {
        println!("obj={:.4} worse than empty={:.4} → using empty", result.obj, empty_result.obj);
        vec![]
    };

    // Serialize JSON
    let sol_json = serde_json::json!({
        "srpaths": final_srpaths.iter().map(|p| serde_json::json!({
            "d": p.d,
            "t": p.t,
            "w": p.w
        })).collect::<Vec<_>>()
    });
    let mut f = File::create(out_path)?;
    writeln!(f, "{}", serde_json::to_string_pretty(&sol_json)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared greedy solver (exact copy from campaign_engine.rs)
// ---------------------------------------------------------------------------
fn solve_greedy(
    net: &Network,
    demands: &[roadef::models::Demand],
    num_slots: usize,
    disabled_links: &HashSet<u64>,
    max_segments: usize,
) -> std::collections::HashMap<usize, Vec<u64>> {
    // Sort demands by peak volume descending
    let mut sorted: Vec<(usize, u64, u64, Vec<f64>, f64)> = demands.iter().enumerate()
        .map(|(i, d)| {
            let peak_v = d.v.iter().cloned().fold(0.0, f64::max);
            (i, d.s, d.t, d.v.clone(), peak_v)
        })
        .collect();
    sorted.sort_by(|a, b| b.4.partial_cmp(&a.4).unwrap_or(std::cmp::Ordering::Equal));

    // Build link lookup structures
    let mut link_capacity: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    let mut link_flow: Vec<std::collections::HashMap<u64, f64>> = vec![std::collections::HashMap::new(); num_slots];
    let mut link_saturation: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    let mut link_by_endpoints: std::collections::HashMap<(u64, u64), u64> = std::collections::HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
        for t in 0..num_slots {
            link_flow[t].insert(link.id, 0.0);
        }
        link_saturation.insert(link.id, 0.0);
        link_by_endpoints.insert((link.from, link.to), link.id);
    }

    let mut assignments: std::collections::HashMap<usize, Vec<u64>> = std::collections::HashMap::new();
    let mut demand_paths: std::collections::HashMap<usize, Vec<u64>> = std::collections::HashMap::new();

    for (d_idx, src, dst, v, _peak_v) in &sorted {
        // Load-aware path first, fall back to pure Dijkstra
        let full_path = load_aware_path(net, *src, *dst, disabled_links, &link_flow, v, &link_capacity, 100.0)
            .or_else(|| dijkstra_path(net, *src, *dst, disabled_links));
        if let Some(fp) = full_path {
            // Convert to waypoints respecting segment limit
            let waypoints = if fp.len() <= 2 || max_segments == 0 {
                vec![]
            } else {
                let raw_wp = fp[1..fp.len() - 1].to_vec();
                let max_wp = max_segments.saturating_sub(1);
                if raw_wp.len() > max_wp {
                    raw_wp[..max_wp].to_vec()
                } else {
                    raw_wp
                }
            };
            // Update link flows for future load-aware calculations
            for j in 0..fp.len().saturating_sub(1) {
                if let Some(link_id) = link_by_endpoints.get(&(fp[j], fp[j + 1])) {
                    let cap = link_capacity.get(link_id).copied().unwrap_or(1.0);
                    let mut max_sat_e = 0.0f64;
                    for t in 0..num_slots {
                        let vol_t = if t < v.len() { v[t] } else { 0.0 };
                        let flow = link_flow[t].entry(*link_id).or_insert(0.0);
                        *flow += vol_t;
                        let sat_t = *flow / cap;
                        if sat_t > max_sat_e {
                            max_sat_e = sat_t;
                        }
                    }
                    link_saturation.insert(*link_id, max_sat_e);
                }
            }
            demand_paths.insert(*d_idx, fp);
            assignments.insert(*d_idx, waypoints);
        }
    }

    // Rip-up and reroute bottleneck relief for boundary saturation (MLU >= 0.999)
    for _iter in 0..50 {
        let mut max_sat = 0.0f64;
        let mut max_link_id = None;
        for (&link_id, &sat) in &link_saturation {
            if sat > max_sat {
                max_sat = sat;
                max_link_id = Some(link_id);
            }
        }
        if max_sat < 0.9999 || max_link_id.is_none() {
            break;
        }
        let bottle_link = max_link_id.unwrap();

        // Find candidate demands traversing the bottleneck link
        let mut candidate_demands: Vec<usize> = Vec::new();
        for (&d_idx, fp) in &demand_paths {
            for j in 0..fp.len().saturating_sub(1) {
                if let Some(&lid) = link_by_endpoints.get(&(fp[j], fp[j + 1])) {
                    if lid == bottle_link {
                        candidate_demands.push(d_idx);
                        break;
                    }
                }
            }
        }
        if candidate_demands.is_empty() {
            break;
        }

        // Try rerouting demands, preferring smaller demands first to avoid creating larger bottlenecks elsewhere
        candidate_demands.sort_by(|&d1, &d2| {
            let v1 = demands.get(d1).map(|d| d.v.iter().cloned().fold(0.0, f64::max)).unwrap_or(0.0);
            let v2 = demands.get(d2).map(|d| d.v.iter().cloned().fold(0.0, f64::max)).unwrap_or(0.0);
            v1.partial_cmp(&v2).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut rerouted = false;
        for d_target in candidate_demands {
            let d_obj = &demands[d_target];
            let src = d_obj.s;
            let dst = d_obj.t;
            let v = &d_obj.v;
            let old_fp = demand_paths.get(&d_target).unwrap().clone();

            // Rip up: subtract flow
            for j in 0..old_fp.len().saturating_sub(1) {
                if let Some(lid) = link_by_endpoints.get(&(old_fp[j], old_fp[j + 1])) {
                    let cap = link_capacity.get(lid).copied().unwrap_or(1.0);
                    let mut max_sat_e = 0.0f64;
                    for t in 0..num_slots {
                        let vol_t = if t < v.len() { v[t] } else { 0.0 };
                        let flow = link_flow[t].entry(*lid).or_insert(0.0);
                        *flow = (*flow - vol_t).max(0.0);
                        let sat_t = *flow / cap;
                        if sat_t > max_sat_e {
                            max_sat_e = sat_t;
                        }
                    }
                    link_saturation.insert(*lid, max_sat_e);
                }
            }

            // Temporarily mask bottleneck link
            let mut temp_disabled = disabled_links.clone();
            temp_disabled.insert(bottle_link);

            let new_path = load_aware_path(net, src, dst, &temp_disabled, &link_flow, v, &link_capacity, 100.0)
                .or_else(|| dijkstra_path(net, src, dst, &temp_disabled));

            if let Some(nfp) = new_path {
                let raw_wp = if nfp.len() <= 2 || max_segments == 0 {
                    vec![]
                } else {
                    let wp = nfp[1..nfp.len() - 1].to_vec();
                    let max_wp = max_segments.saturating_sub(1);
                    if wp.len() > max_wp {
                        wp[..max_wp].to_vec()
                    } else {
                        wp
                    }
                };

                // Add flow for new path
                for j in 0..nfp.len().saturating_sub(1) {
                    if let Some(lid) = link_by_endpoints.get(&(nfp[j], nfp[j + 1])) {
                        let cap = link_capacity.get(lid).copied().unwrap_or(1.0);
                        let mut max_sat_e = 0.0f64;
                        for t in 0..num_slots {
                            let vol_t = if t < v.len() { v[t] } else { 0.0 };
                            let flow = link_flow[t].entry(*lid).or_insert(0.0);
                            *flow += vol_t;
                            let sat_t = *flow / cap;
                            if sat_t > max_sat_e {
                                max_sat_e = sat_t;
                            }
                        }
                        link_saturation.insert(*lid, max_sat_e);
                    }
                }

                demand_paths.insert(d_target, nfp);
                assignments.insert(d_target, raw_wp);
                rerouted = true;
                break;
            } else {
                // Restore old_fp
                for j in 0..old_fp.len().saturating_sub(1) {
                    if let Some(lid) = link_by_endpoints.get(&(old_fp[j], old_fp[j + 1])) {
                        let cap = link_capacity.get(lid).copied().unwrap_or(1.0);
                        let mut max_sat_e = 0.0f64;
                        for t in 0..num_slots {
                            let vol_t = if t < v.len() { v[t] } else { 0.0 };
                            let flow = link_flow[t].entry(*lid).or_insert(0.0);
                            *flow += vol_t;
                            let sat_t = *flow / cap;
                            if sat_t > max_sat_e {
                                max_sat_e = sat_t;
                            }
                        }
                        link_saturation.insert(*lid, max_sat_e);
                    }
                }
            }
        }
        if !rerouted {
            break;
        }
    }

    assignments
}

// ---------------------------------------------------------------------------
// Helper: Dijkstra and load‑aware variants (copied verbatim from campaign_engine.rs)
// ---------------------------------------------------------------------------
fn dijkstra_path(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let mut adj: std::collections::HashMap<u64, Vec<(u64, f64)>> = std::collections::HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) { continue; }
        adj.entry(link.from).or_default().push((link.to, link.metric));
    }
    let mut dist: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    let mut prev: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    let mut heap: std::collections::BinaryHeap<(std::cmp::Reverse<u64>, u64)> = std::collections::BinaryHeap::new();
    dist.insert(src, 0);
    heap.push((std::cmp::Reverse(0), src));
    while let Some((std::cmp::Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst { break; }
        if let Some(neighbors) = adj.get(&node) {
            for &(next, metric) in neighbors {
                let new_cost = cost + (metric * 1000.0) as u64;
                let better = dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost;
                if better {
                    dist.insert(next, new_cost);
                    prev.insert(next, node);
                    heap.push((std::cmp::Reverse(new_cost), next));
                }
            }
        }
    }
    if !dist.contains_key(&dst) { return None; }
    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        if let Some(p) = prev.get(&cur) { path.push(*p); cur = *p; } else { return None; }
    }
    path.reverse();
    Some(path)
}

fn load_aware_path(
    net: &Network,
    src: u64,
    dst: u64,
    disabled_links: &HashSet<u64>,
    link_flow: &[std::collections::HashMap<u64, f64>],
    v: &[f64],
    link_capacity: &std::collections::HashMap<u64, f64>,
    load_penalty: f64,
) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let num_slots = link_flow.len();
    let mut adj: std::collections::HashMap<u64, Vec<(u64, f64)>> = std::collections::HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) { continue; }
        let cap = link_capacity.get(&link.id).copied().unwrap_or(1.0);
        let mut proj_sat = 0.0f64;
        for t in 0..num_slots {
            let vol_t = if t < v.len() { v[t] } else { 0.0 };
            let cur_flow = link_flow[t].get(&link.id).copied().unwrap_or(0.0);
            let sat_t = (cur_flow + vol_t) / cap;
            if sat_t > proj_sat {
                proj_sat = sat_t;
            }
        }
        let penalty = if proj_sat >= 1.0 {
            1e9
        } else if proj_sat > 0.8 {
            load_penalty * (1.0 / (1.0 - proj_sat) - 1.0) * 10.0
        } else { load_penalty * proj_sat };
        let effective_metric = link.metric + penalty;
        adj.entry(link.from).or_default().push((link.to, effective_metric));
    }
    // Re‑use Dijkstra logic with adjusted adjacency
    let mut dist: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    let mut prev: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
    let mut heap: std::collections::BinaryHeap<(std::cmp::Reverse<u64>, u64)> = std::collections::BinaryHeap::new();
    dist.insert(src, 0);
    heap.push((std::cmp::Reverse(0), src));
    while let Some((std::cmp::Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst { break; }
        if let Some(neighbors) = adj.get(&node) {
            for &(next, metric) in neighbors {
                let new_cost = cost + (metric * 1000.0) as u64;
                let better = dist.get(&next).copied().unwrap_or(u64::MAX) > new_cost;
                if better {
                    dist.insert(next, new_cost);
                    prev.insert(next, node);
                    heap.push((std::cmp::Reverse(new_cost), next));
                }
            }
        }
    }
    if !dist.contains_key(&dst) { return None; }
    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        if let Some(p) = prev.get(&cur) { path.push(*p); cur = *p; } else { return None; }
    }
    path.reverse();
    Some(path)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <net> <tm> <scenario> <output>", args[0]);
        std::process::exit(1);
    }
    let net_path = &args[1];
    let tm_path = &args[2];
    let sc_path = &args[3];
    let out_path = &args[4];

    let net = load_network(net_path)?;
    let tm = load_traffic_matrix(tm_path)?;
    let scenario = load_scenario(sc_path)?;

    run_instance(&net, &tm, &scenario, out_path)
}
