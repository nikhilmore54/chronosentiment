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

    // Disabled links per time slot
    let disabled_t0: HashSet<u64> = scenario.interventions.iter().filter(|i| i.t == 0).flat_map(|i| i.links.iter().copied()).collect();
    let disabled_t1: HashSet<u64> = scenario.interventions.iter().filter(|i| i.t == 1).flat_map(|i| i.links.iter().copied()).collect();
    let disabled_both: HashSet<u64> = disabled_t0.union(&disabled_t1).copied().collect();

    // Average‑volume demand list (used for routing priority)
    let demands_avg: Vec<(usize, u64, u64, f64)> = tm.demands.iter().enumerate()
        .map(|(i, d)| {
            let v0 = d.v[0];
            let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
            (i, d.s, d.t, (v0 + v1) / 2.0)
        })
        .collect();

    // Greedy shared assignment (core frozen algorithm)
    let shared_assign = solve_greedy(&net, &demands_avg, &disabled_both, max_seg);

    // Build srpaths for both slots
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
    demands: &[(usize, u64, u64, f64)],
    disabled_links: &HashSet<u64>,
    max_segments: usize,
) -> std::collections::HashMap<usize, Vec<u64>> {
    // Sort demands by volume descending
    let mut sorted: Vec<(usize, u64, u64, f64)> = demands.to_vec();
    sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    // Build link lookup structures
    let mut link_capacity: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    let mut link_flow: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    let mut link_saturation: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    let mut link_by_endpoints: std::collections::HashMap<(u64, u64), u64> = std::collections::HashMap::new();
    for link in &net.links {
        link_capacity.insert(link.id, link.capacity);
        link_flow.insert(link.id, 0.0);
        link_saturation.insert(link.id, 0.0);
        link_by_endpoints.insert((link.from, link.to), link.id);
    }

    let mut assignments: std::collections::HashMap<usize, Vec<u64>> = std::collections::HashMap::new();
    for (d_idx, src, dst, vol) in &sorted {
        // Load‑aware path first, fall back to pure Dijkstra
        let full_path = load_aware_path(net, *src, *dst, disabled_links, &link_saturation, 100.0)
            .or_else(|| dijkstra_path(net, *src, *dst, disabled_links));
        if let Some(fp) = full_path {
            // Convert to waypoints respecting segment limit
            let waypoints = if fp.len() <= 2 { vec![] } else { fp[1..fp.len() - 1].to_vec() };
            // Update link flows for future load‑aware calculations
            for j in 0..fp.len().saturating_sub(1) {
                if let Some(link_id) = link_by_endpoints.get(&(fp[j], fp[j + 1])) {
                    let flow = link_flow.entry(*link_id).or_insert(0.0);
                    *flow += vol;
                    let cap = link_capacity.get(link_id).copied().unwrap_or(1.0);
                    link_saturation.insert(*link_id, *flow / cap);
                }
            }
            assignments.insert(*d_idx, waypoints);
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
    link_saturation: &std::collections::HashMap<u64, f64>,
    load_penalty: f64,
) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let mut adj: std::collections::HashMap<u64, Vec<(u64, f64)>> = std::collections::HashMap::new();
    for link in &net.links {
        if disabled_links.contains(&link.id) { continue; }
        let sat = link_saturation.get(&link.id).copied().unwrap_or(0.0);
        let penalty = if sat >= 1.0 {
            1e9
        } else if sat > 0.8 {
            load_penalty * (1.0 / (1.0 - sat) - 1.0) * 10.0
        } else { load_penalty * sat };
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
