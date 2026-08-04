/// rp406_setA17_diag -- RP-406: setA-17 Feasibility Frontier Investigation
/// Diagnostic binary: analyses why setA-17 produces inf objective across all operators.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;

use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;

const SET_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";

fn dijkstra(net: &Network, src: u64, dst: u64, disabled: &HashSet<u64>) -> Option<f64> {
    if src == dst { return Some(0.0); }
    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for l in &net.links {
        if disabled.contains(&l.id) { continue; }
        adj.entry(l.from).or_default().push((l.to, l.metric));
    }
    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>, u64)> = BinaryHeap::new();
    dist.insert(src, 0);
    heap.push((Reverse(0), src));
    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst { return Some(cost as f64 / 1000.0); }
        if let Some(nbrs) = adj.get(&node) {
            for &(next, metric) in nbrs {
                let nc = cost + (metric * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > nc {
                    dist.insert(next, nc);
                    heap.push((Reverse(nc), next));
                }
            }
        }
    }
    None
}

fn load_solution(path: &str, nd: usize) -> Vec<SrPath> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
    let arr = match json["srpaths"].as_array() { Some(a) => a.clone(), None => return vec![] };
    let mut srpaths = Vec::new();
    for e in &arr {
        let d = e["d"].as_u64().unwrap_or(0) as usize;
        let t = e["t"].as_u64().unwrap_or(0) as usize;
        let w: Vec<u64> = e["w"].as_array().unwrap_or(&vec![]).iter().filter_map(|v| v.as_u64()).collect();
        if d < nd { srpaths.push(SrPath { d, t, w }); }
    }
    srpaths
}

fn main() -> anyhow::Result<()> {
    let inst = "17";
    println!("RP-406 -- setA-17 Feasibility Frontier Investigation");
    println!("{}", "=".repeat(80));

    let net  = load_network(&format!("{}/setA-{}-net.json", SET_DIR, inst))?;
    let tm   = load_traffic_matrix(&format!("{}/setA-{}-tm.json", SET_DIR, inst))?;
    let sc   = load_scenario(&format!("{}/setA-{}-scenario.json", SET_DIR, inst))?;
    let nd = tm.demands.len();
    let ns = tm.num_time_slots;
    let max_seg = if sc.max_segments >= 0 { sc.max_segments as usize } else { 100 };
    let ev = RoadefEvaluator::new(&net, tm.clone(), sc.clone());

    println!("Instance: setA-{}", inst);
    println!("Nodes: {}", net.nodes.len());
    println!("Links: {}", net.links.len());
    println!("Demands: {}", nd);
    println!("Time slots: {}", ns);
    println!("Max segments: {}", max_seg);
    println!();

    // Intervention details
    println!("=== INTERVENTIONS ===");
    for (i, iv) in sc.interventions.iter().enumerate() {
        println!("Intervention {}: t={}, {} links disabled: {:?}", i, iv.t, iv.links.len(), iv.links);
    }
    println!();

    // Network capacity stats
    let caps: Vec<f64> = net.links.iter().map(|l| l.capacity).collect();
    let total_cap: f64 = caps.iter().sum();
    let min_cap = caps.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_cap = caps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean_cap = total_cap / caps.len() as f64;
    let zero_cap = caps.iter().filter(|&&c| c == 0.0).count();
    println!("=== NETWORK CAPACITY ===");
    println!("Total link capacity: {:.2}", total_cap);
    println!("Min/Mean/Max cap: {:.2} / {:.2} / {:.2}", min_cap, mean_cap, max_cap);
    println!("Zero-capacity links: {}", zero_cap);
    println!();

    // Demand volume stats
    let vols_t0: Vec<f64> = tm.demands.iter().map(|d| d.v[0]).collect();
    let vols_t1: Vec<f64> = tm.demands.iter().map(|d| if d.v.len() > 1 { d.v[1] } else { d.v[0] }).collect();
    let total_vol_t0: f64 = vols_t0.iter().sum();
    let total_vol_t1: f64 = vols_t1.iter().sum();
    let max_vol_t0 = vols_t0.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let max_vol_t1 = vols_t1.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    println!("=== DEMAND VOLUMES ===");
    println!("Total demand volume t=0: {:.2}", total_vol_t0);
    println!("Total demand volume t=1: {:.2}", total_vol_t1);
    println!("Max single demand t=0: {:.2}", max_vol_t0);
    println!("Max single demand t=1: {:.2}", max_vol_t1);
    println!("Volume/Capacity ratio t=0: {:.4}", total_vol_t0 / total_cap);
    println!("Volume/Capacity ratio t=1: {:.4}", total_vol_t1 / total_cap);
    println!();

    // Reachability analysis
    let dis_t0: HashSet<u64> = sc.interventions.iter().filter(|i| i.t == 0).flat_map(|i| i.links.iter().copied()).collect();
    let dis_t1: HashSet<u64> = sc.interventions.iter().filter(|i| i.t == 1).flat_map(|i| i.links.iter().copied()).collect();
    let dis_both: HashSet<u64> = dis_t0.union(&dis_t1).copied().collect();

    println!("=== REACHABILITY ANALYSIS ===");
    let mut unreachable_t0 = 0usize;
    let mut unreachable_t1 = 0usize;
    let mut unreachable_both = 0usize;
    for d in &tm.demands {
        let r0 = dijkstra(&net, d.s, d.t, &dis_t0);
        let r1 = dijkstra(&net, d.s, d.t, &dis_t1);
        let rb = dijkstra(&net, d.s, d.t, &dis_both);
        if r0.is_none() { unreachable_t0 += 1; }
        if r1.is_none() { unreachable_t1 += 1; }
        if rb.is_none() { unreachable_both += 1; }
    }
    println!("Demands unreachable at t=0 (dis_t0 links disabled): {}/{}", unreachable_t0, nd);
    println!("Demands unreachable at t=1 (dis_t1 links disabled): {}/{}", unreachable_t1, nd);
    println!("Demands unreachable at both t (all intervention links disabled): {}/{}", unreachable_both, nd);
    println!();

    // Evaluate empty solution
    println!("=== EMPTY SOLUTION EVALUATION ===");
    let empty_sol = Solution { srpaths: vec![] };
    let empty_r = ev.evaluate_solution(&empty_sol);
    println!("Empty solution: obj={:.4}, valid={}", empty_r.obj, empty_r.valid);
    println!();

    // Evaluate RP-403 solution
    println!("=== RP-403 SOLUTION EVALUATION ===");
    let rp403_sp = load_solution(&format!("{}/setA-{}-srpaths-rp403.json", SET_DIR, inst), nd);
    println!("RP-403 srpaths loaded: {}", rp403_sp.len());
    let t0_count = rp403_sp.iter().filter(|s| s.t == 0).count();
    let t1_count = rp403_sp.iter().filter(|s| s.t == 1).count();
    let empty_w = rp403_sp.iter().filter(|s| s.w.is_empty()).count();
    println!("  t=0 paths: {}, t=1 paths: {}, empty-waypoint (direct): {}", t0_count, t1_count, empty_w);
    let rp403_sol = Solution { srpaths: rp403_sp.clone() };
    let rp403_r = ev.evaluate_solution(&rp403_sol);
    println!("  obj={:.4}, valid={}", rp403_r.obj, rp403_r.valid);
    println!();

    // Evaluate RP-405 solution
    println!("=== RP-405 SOLUTION EVALUATION ===");
    let rp405_sp = load_solution(&format!("{}/setA-{}-srpaths-rp405-adaptive.json", SET_DIR, inst), nd);
    println!("RP-405 srpaths loaded: {}", rp405_sp.len());
    let rp405_sol = Solution { srpaths: rp405_sp };
    let rp405_r = ev.evaluate_solution(&rp405_sol);
    println!("  obj={:.4}, valid={}", rp405_r.obj, rp405_r.valid);
    println!();

    // Load RP-403 and compute per-link utilisation at t=0
    println!("=== LINK UTILISATION (RP-403, t=0) ===");
    if let Some(loads) = ev.compute_loads(0, &rp403_sol) {
        let mut overloaded = 0usize;
        let mut near_cap = 0usize;
        let cap_map: HashMap<u64, f64> = net.links.iter().map(|l| (l.id, l.capacity)).collect();
        let mut max_util = 0.0f64;
        let mut max_util_lid = 0u64;
        for (lid, flow) in &loads.arc_flows {
            let cap = cap_map.get(lid).copied().unwrap_or(1.0);
            let util = if cap > 0.0 { flow / cap } else { f64::INFINITY };
            if util > max_util { max_util = util; max_util_lid = *lid; }
            if util > 1.0 { overloaded += 1; }
            else if util > 0.9 { near_cap += 1; }
        }
        println!("Links with flow > capacity (overloaded): {}", overloaded);
        println!("Links with flow > 90% capacity: {}", near_cap);
        println!("Max utilisation: {:.4} on link {}", max_util, max_util_lid);
        if let Some(l) = net.links.iter().find(|l| l.id == max_util_lid) {
            println!("  Link {}: {} -> {}, cap={:.2}, flow={:.2}", max_util_lid, l.from, l.to, l.capacity, loads.arc_flows.get(&max_util_lid).copied().unwrap_or(0.0));
        }
    } else {
        println!("Could not compute loads for RP-403 solution at t=0");
    }
    println!();

    // Check t=1 utilisation
    println!("=== LINK UTILISATION (RP-403, t=1) ===");
    if let Some(loads) = ev.compute_loads(1, &rp403_sol) {
        let mut overloaded = 0usize;
        let cap_map: HashMap<u64, f64> = net.links.iter().map(|l| (l.id, l.capacity)).collect();
        let mut max_util = 0.0f64;
        let mut max_util_lid = 0u64;
        for (lid, flow) in &loads.arc_flows {
            let cap = cap_map.get(lid).copied().unwrap_or(1.0);
            let util = if cap > 0.0 { flow / cap } else { f64::INFINITY };
            if util > max_util { max_util = util; max_util_lid = *lid; }
            if util > 1.0 { overloaded += 1; }
        }
        println!("Links with flow > capacity (overloaded): {}", overloaded);
        println!("Max utilisation: {:.4} on link {}", max_util, max_util_lid);
        if let Some(l) = net.links.iter().find(|l| l.id == max_util_lid) {
            println!("  Link {}: {} -> {}, cap={:.2}, flow={:.2}", max_util_lid, l.from, l.to, l.capacity, loads.arc_flows.get(&max_util_lid).copied().unwrap_or(0.0));
        }
    } else {
        println!("Could not compute loads for RP-403 solution at t=1");
    }
    println!();

    println!("=== DIAGNOSIS COMPLETE ===");
    Ok(())
}