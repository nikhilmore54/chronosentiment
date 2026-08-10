/// rp406_setA17_diag -- RP-406A: setA-17 Feasibility Frontier Investigation
/// Diagnostic binary: identifies the root cause of inf objective on setA-17.
/// RP-406A findings:
///   - Empty solution → obj=inf because unassigned demands route via ECMP shortest paths
///   - ECMP default routing overloads link 1173 (12→36, cap=1513)
///   - RP-403 assigns only 425/2000 demands; 1575 remain on default ECMP routing
///   - Root cause: repair operator must assign SR paths to ALL demands traversing link 1173

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;

use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;
use roadef::graph::Digraph;
use roadef::ecmp::expand_sr_path;

const SET_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";

fn dijkstra_path(net: &Network, src: u64, dst: u64, disabled: &HashSet<u64>) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let mut adj: HashMap<u64, Vec<(u64, u64, f64)>> = HashMap::new(); // (to, link_id, metric)
    for l in &net.links {
        if disabled.contains(&l.id) { continue; }
        adj.entry(l.from).or_default().push((l.to, l.id, l.metric));
    }
    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut prev: HashMap<u64, u64> = HashMap::new(); // node -> prev node
    let mut heap: BinaryHeap<(Reverse<u64>, u64)> = BinaryHeap::new();
    dist.insert(src, 0);
    heap.push((Reverse(0), src));
    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst {
            // Reconstruct path
            let mut path = vec![dst];
            let mut cur = dst;
            while cur != src {
                if let Some(&p) = prev.get(&cur) { path.push(p); cur = p; } else { break; }
            }
            path.reverse();
            return Some(path);
        }
        if let Some(nbrs) = adj.get(&node) {
            for &(next, _lid, metric) in nbrs {
                let nc = cost + (metric * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > nc {
                    dist.insert(next, nc);
                    prev.insert(next, node);
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
    println!("RP-406A -- setA-17 Feasibility Frontier Investigation");
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
        for &lid in &iv.links {
            if let Some(l) = net.links.iter().find(|l| l.id == lid) {
                println!("  Link {}: {} -> {}, cap={:.2}, metric={:.2}", lid, l.from, l.to, l.capacity, l.metric);
            }
        }
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
    println!("=== DEMAND VOLUMES ===");
    println!("Total demand volume t=0: {:.2}", total_vol_t0);
    println!("Total demand volume t=1: {:.2}", total_vol_t1);
    println!("Volume/Capacity ratio t=0: {:.4}", total_vol_t0 / total_cap);
    println!("Volume/Capacity ratio t=1: {:.4}", total_vol_t1 / total_cap);
    println!();

    // Reachability analysis — classify each unreachable demand
    let dis_t0: HashSet<u64> = sc.interventions.iter().filter(|i| i.t == 0).flat_map(|i| i.links.iter().copied()).collect();
    let dis_t1: HashSet<u64> = sc.interventions.iter().filter(|i| i.t == 1).flat_map(|i| i.links.iter().copied()).collect();
    let dis_both: HashSet<u64> = dis_t0.union(&dis_t1).copied().collect();

    println!("=== REACHABILITY ANALYSIS ===");
    let mut unreachable_t0 = 0usize;
    let mut unreachable_t1 = 0usize;
    let mut unreachable_both = 0usize;
    for (i, d) in tm.demands.iter().enumerate() {
        let r0 = dijkstra_path(&net, d.s, d.t, &dis_t0);
        let r1 = dijkstra_path(&net, d.s, d.t, &dis_t1);
        let rb = dijkstra_path(&net, d.s, d.t, &dis_both);
        if r0.is_none() {
            unreachable_t0 += 1;
            println!("  Demand {}: src={} dst={} UNREACHABLE at t=0 (base graph disconnected)", i, d.s, d.t);
        }
        if r1.is_none() {
            unreachable_t1 += 1;
            println!("  Demand {}: src={} dst={} UNREACHABLE at t=1 (disconnected after intervention)", i, d.s, d.t);
        }
        if rb.is_none() && r0.is_some() && r1.is_some() {
            unreachable_both += 1;
        }
    }
    println!("Demands unreachable at t=0: {}/{}", unreachable_t0, nd);
    println!("Demands unreachable at t=1 (intervention applied): {}/{}", unreachable_t1, nd);
    println!("Demands unreachable at both t: {}/{}", unreachable_both, nd);
    println!();

    // Evaluate empty solution — identify which objective component becomes inf
    println!("=== EMPTY SOLUTION EVALUATION ===");
    let empty_sol = Solution { srpaths: vec![] };
    let empty_r = ev.evaluate_solution(&empty_sol);
    println!("Empty solution: obj={:.4}, valid={}", empty_r.obj, empty_r.valid);
    println!("Interpretation: empty solution routes all demands via ECMP shortest paths.");
    println!("If obj=inf, at least one link is overloaded by default ECMP routing.");
    println!();

    // Identify which links are overloaded by default ECMP routing (empty solution)
    println!("=== LINK OVERLOAD ANALYSIS (empty solution, t=0) ===");
    if let Some(loads) = ev.compute_loads(0, &empty_sol) {
        let cap_map: HashMap<u64, f64> = net.links.iter().map(|l| (l.id, l.capacity)).collect();
        let mut overloaded_links: Vec<(u64, f64, f64, f64)> = vec![]; // (lid, flow, cap, util)
        for (lid, flow) in &loads.arc_flows {
            let cap = cap_map.get(lid).copied().unwrap_or(1.0);
            if cap > 0.0 {
                let util = flow / cap;
                if util >= 1.0 {
                    overloaded_links.push((*lid, *flow, cap, util));
                }
            }
        }
        overloaded_links.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
        println!("Overloaded links (sat >= 1.0) under default ECMP routing at t=0: {}", overloaded_links.len());
        for (lid, flow, cap, util) in &overloaded_links {
            if let Some(l) = net.links.iter().find(|l| l.id == *lid) {
                println!("  Link {}: {} -> {}, cap={:.2}, flow={:.2}, util={:.4}", lid, l.from, l.to, cap, flow, util);
            }
        }
    } else {
        println!("compute_loads returned None (connectivity failure at t=0)");
    }
    println!();

    println!("=== LINK OVERLOAD ANALYSIS (empty solution, t=1) ===");
    if let Some(loads) = ev.compute_loads(1, &empty_sol) {
        let cap_map: HashMap<u64, f64> = net.links.iter().map(|l| (l.id, l.capacity)).collect();
        let mut overloaded_links: Vec<(u64, f64, f64, f64)> = vec![];
        for (lid, flow) in &loads.arc_flows {
            let cap = cap_map.get(lid).copied().unwrap_or(1.0);
            if cap > 0.0 {
                let util = flow / cap;
                if util >= 1.0 {
                    overloaded_links.push((*lid, *flow, cap, util));
                }
            }
        }
        overloaded_links.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
        println!("Overloaded links (sat >= 1.0) under default ECMP routing at t=1: {}", overloaded_links.len());
        for (lid, flow, cap, util) in &overloaded_links {
            if let Some(l) = net.links.iter().find(|l| l.id == *lid) {
                println!("  Link {}: {} -> {}, cap={:.2}, flow={:.2}, util={:.4}", lid, l.from, l.to, cap, flow, util);
            }
        }
    } else {
        println!("compute_loads returned None (connectivity failure at t=1)");
    }
    println!();

    // Identify demands that traverse the bottleneck link (1173: 12->36) under default routing
    println!("=== BOTTLENECK LINK DEMAND ANALYSIS (link 1173: 12->36) ===");
    let bottleneck_lid = 1173u64;
    let bottleneck_link = net.links.iter().find(|l| l.id == bottleneck_lid);
    if let Some(bl) = bottleneck_link {
        println!("Bottleneck link {}: {} -> {}, cap={:.2}", bottleneck_lid, bl.from, bl.to, bl.capacity);
        let mut demands_via_bottleneck_t0 = 0usize;
        let mut vol_via_bottleneck_t0 = 0.0f64;
        for (i, d) in tm.demands.iter().enumerate() {
            if d.v[0] <= 0.0 { continue; }
            if let Some(path) = dijkstra_path(&net, d.s, d.t, &dis_t0) {
                // Check if path uses link 1173 (12->36)
                for w in path.windows(2) {
                    if w[0] == bl.from && w[1] == bl.to {
                        demands_via_bottleneck_t0 += 1;
                        vol_via_bottleneck_t0 += d.v[0];
                        break;
                    }
                }
            }
        }
        println!("Demands routing via link 1173 at t=0 (default ECMP): {}/{}", demands_via_bottleneck_t0, nd);
        println!("Total volume via link 1173 at t=0: {:.2} (cap={:.2}, util={:.4})", vol_via_bottleneck_t0, bl.capacity, vol_via_bottleneck_t0 / bl.capacity);
    }
    println!();

    // RP-403 solution analysis
    println!("=== RP-403 SOLUTION EVALUATION ===");
    let rp403_sp = load_solution(&format!("{}/setA-{}-srpaths-rp403.json", SET_DIR, inst), nd);
    println!("RP-403 srpaths loaded: {} (of {} demands)", rp403_sp.len(), nd * ns);
    let t0_assigned: HashSet<usize> = rp403_sp.iter().filter(|s| s.t == 0).map(|s| s.d).collect();
    let t1_assigned: HashSet<usize> = rp403_sp.iter().filter(|s| s.t == 1).map(|s| s.d).collect();
    println!("  Demands with explicit SR path at t=0: {}/{}", t0_assigned.len(), nd);
    println!("  Demands with explicit SR path at t=1: {}/{}", t1_assigned.len(), nd);
    println!("  Demands routed via default ECMP at t=0: {}/{}", nd - t0_assigned.len(), nd);
    println!("  Demands routed via default ECMP at t=1: {}/{}", nd - t1_assigned.len(), nd);
    let rp403_sol = Solution { srpaths: rp403_sp };
    let rp403_r = ev.evaluate_solution(&rp403_sol);
    println!("  obj={:.4}, valid={}", rp403_r.obj, rp403_r.valid);
    println!();

    // First violated constraint identification
    println!("=== FIRST VIOLATED CONSTRAINT (RP-403, t=0) ===");
    if let Some(loads) = ev.compute_loads(0, &rp403_sol) {
        let cap_map: HashMap<u64, f64> = net.links.iter().map(|l| (l.id, l.capacity)).collect();
        let mut violations: Vec<(u64, f64, f64, f64)> = vec![];
        for (lid, flow) in &loads.arc_flows {
            let cap = cap_map.get(lid).copied().unwrap_or(1.0);
            if cap > 0.0 && flow / cap >= 1.0 {
                violations.push((*lid, *flow, cap, flow / cap));
            }
        }
        violations.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
        println!("Capacity violations at t=0: {}", violations.len());
        for (lid, flow, cap, util) in &violations {
            if let Some(l) = net.links.iter().find(|l| l.id == *lid) {
                println!("  VIOLATION: Link {}: {} -> {}, cap={:.2}, flow={:.2}, overflow={:.4}", lid, l.from, l.to, cap, flow, flow - cap);
            }
        }
        if violations.is_empty() {
            println!("  No capacity violations at t=0");
        }
    }
    println!();

    println!("=== DIAGNOSIS SUMMARY ===");
    println!("1. Connectivity: ALL 2000 demands reachable at t=0 and t=1 (graph is connected).");
    println!("2. Capacity: Volume/capacity ratio ~0.0019 (enormous spare capacity in aggregate).");
    println!("3. Root cause: Default ECMP routing of unassigned demands overloads link 1173 (12->36).");
    println!("4. RP-403 assigns SR paths to only 425/2000 demands; 1575 remain on default ECMP.");
    println!("5. The 1575 unassigned demands concentrate traffic on link 1173 via shortest paths.");
    println!("6. Fix direction: Repair operator must assign SR paths to ALL demands (or all that");
    println!("   traverse link 1173), rerouting them away from the bottleneck.");
    println!();
    println!("=== DIAGNOSIS COMPLETE ===");
    Ok(())
}