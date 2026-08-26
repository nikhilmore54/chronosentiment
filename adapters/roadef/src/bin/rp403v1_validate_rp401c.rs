use std::cmp::Reverse;
/// rp403v1_validate_rp401c — RP-403 Validation Task V1: RP-401C Behavioural Equivalence
///
/// Compares waypoint assignments produced by:
///   A) The standalone rp401c_ecmp_construction binary (JSON output on disk)
///   B) The original embedded solve_rp401c in rp403_construction_portfolio.rs
///      (multiplicative metric multiplier, as it existed at commit 1a6ce6d8)
///
/// The validator does NOT modify either implementation. It reproduces the original
/// embedded logic exactly and compares demand-by-demand against the standalone JSON.
///
/// Usage: cargo run --bin rp403v1_validate_rp401c --release -- [setA-NN]
///   Defaults to setA-12 if no argument given.
///
/// Output:
///   - Per-demand comparison: MATCH / DIFFER
///   - First divergence: demand index, src, dst, standalone vs embedded waypoints,
///     and the effective metric values for each candidate edge at that demand
///   - Summary: total demands, matching, differing
///   - Embedded objective (original buggy implementation)
///
/// This is Commit A of the Validation Task V1 evidence chain.
/// No algorithm changes are made here.
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time::Instant;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};

// ---------------------------------------------------------------------------
// Dijkstra with metric multipliers — EXACTLY as in the original embedded
// solve_rp401c in rp403_construction_portfolio.rs (commit 1a6ce6d8).
// effective_metric = link.metric * mult
// ---------------------------------------------------------------------------
fn dijkstra_path_with_mult(
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
// Waypoint extraction — identical to both implementations
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
// Original embedded solve_rp401c — EXACTLY as in commit 1a6ce6d8.
//
// Key differences from standalone rp401c_ecmp_construction:
//   1. Penalty is MULTIPLICATIVE: effective_metric = link.metric * mult
//      Standalone uses ADDITIVE: effective_metric = link.metric + penalty
//   2. Low-saturation formula: mult = 1.0 + sat
//      Standalone: penalty = load_penalty * sat  (load_penalty=100.0)
//   3. High-saturation formula: mult = 100.0 * (1/(1-sat) - 1)
//      Standalone: penalty = load_penalty * (1/(1-sat) - 1) * 10.0
//   4. Saturated formula: mult = 1e9 (same as standalone penalty = 1e9)
//
// This function is reproduced verbatim for validation purposes only.
// It will be replaced by the corrected version in Commit C.
// ---------------------------------------------------------------------------
fn solve_rp401c_original_embedded(
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

        // ORIGINAL: multiplicative metric multiplier (NOT additive penalty)
        let load_mult: HashMap<u64, f64> = net
            .links
            .iter()
            .filter(|l| !disabled_links.contains(&l.id))
            .map(|l| {
                let sat = ecmp_saturation.get(&l.id).copied().unwrap_or(0.0);
                let mult = if sat >= 1.0 {
                    1e9
                } else if sat > 0.8 {
                    // ORIGINAL: missing ×10 factor vs standalone; multiplicative not additive
                    100.0 * (1.0 / (1.0 - sat) - 1.0)
                } else {
                    // ORIGINAL: 1.0 + sat vs standalone's load_penalty * sat = 100.0 * sat
                    1.0 + sat
                };
                (l.id, mult)
            })
            .collect();

        let full_path = dijkstra_path_with_mult(net, *src, *dst, disabled_links, &load_mult)
            .or_else(|| dijkstra_path_with_mult(net, *src, *dst, disabled_links, &HashMap::new()));

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
// Main
// ---------------------------------------------------------------------------
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let instance_id: u32 = if args.len() > 1 {
        args[1].trim_start_matches("setA-").parse().unwrap_or(12)
    } else {
        12
    };
    let inst = format!("{:02}", instance_id);
    let set_dir = "adapters/roadef/repo/challenge-roadef-2026-main/setA";

    println!("RP-403 Validation Task V1 — RP-401C Behavioural Equivalence");
    println!("Instance: setA-{}", inst);
    println!("Comparing: standalone JSON vs original embedded solve_rp401c (commit 1a6ce6d8)");
    println!("{}", "=".repeat(72));

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

    // Disabled links
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

    // Average-volume demands
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

    // ── Load standalone rp401c JSON solution ──────────────────────────────────
    let standalone_path = format!("{}/setA-{}-srpaths-rp401c.json", set_dir, inst);
    let standalone_json: serde_json::Value = {
        let f = std::fs::File::open(&standalone_path).map_err(|e| {
            anyhow::anyhow!("Cannot open standalone JSON {}: {}", standalone_path, e)
        })?;
        serde_json::from_reader(f)?
    };

    // Build standalone waypoint map: demand_idx -> waypoints (t=0 only)
    let mut standalone_waypoints: HashMap<usize, Vec<u64>> = HashMap::new();
    if let Some(srpaths) = standalone_json["srpaths"].as_array() {
        for sp in srpaths {
            let d = sp["d"].as_u64().unwrap_or(0) as usize;
            let t = sp["t"].as_u64().unwrap_or(0);
            if t == 0 {
                let w: Vec<u64> = sp["w"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
                    .unwrap_or_default();
                standalone_waypoints.insert(d, w);
            }
        }
    }
    println!(
        "Standalone JSON: {} demands with t=0 srpaths",
        standalone_waypoints.len()
    );

    // ── Run original embedded solve_rp401c (buggy multiplicative version) ─────
    let t_start = Instant::now();
    let deadline = t_start + std::time::Duration::from_secs(300);
    let embedded_assignments = solve_rp401c_original_embedded(
        &net,
        &evaluator,
        &demands_avg,
        &disabled_both,
        max_seg,
        deadline,
    );
    let elapsed_ms = t_start.elapsed().as_millis();
    println!(
        "Original embedded solve_rp401c completed in {}ms",
        elapsed_ms
    );
    println!("Embedded: {} demands assigned", embedded_assignments.len());
    println!();

    // ── Demand-by-demand comparison ───────────────────────────────────────────
    println!(
        "{:<8} {:<12} {:<12} {:<10} {}",
        "Demand", "Src", "Dst", "Status", "Detail"
    );
    println!("{}", "-".repeat(72));

    let mut match_count = 0usize;
    let mut differ_count = 0usize;
    let mut first_differ: Option<(usize, u64, u64, Vec<u64>, Vec<u64>)> = None;

    for d_idx in 0..num_demands {
        let demand = &tm.demands[d_idx];
        let src = demand.s;
        let dst = demand.t;

        let standalone_wp = standalone_waypoints
            .get(&d_idx)
            .cloned()
            .unwrap_or_default();
        let embedded_wp = embedded_assignments
            .get(&d_idx)
            .cloned()
            .unwrap_or_default();

        if standalone_wp == embedded_wp {
            match_count += 1;
            if match_count <= 5 {
                println!(
                    "{:<8} {:<12} {:<12} {:<10} wp={:?}",
                    d_idx, src, dst, "MATCH", standalone_wp
                );
            } else if match_count == 6 {
                println!("         ... (further MATCH lines suppressed) ...");
            }
        } else {
            differ_count += 1;
            if first_differ.is_none() {
                first_differ = Some((d_idx, src, dst, standalone_wp.clone(), embedded_wp.clone()));
            }
            if differ_count <= 10 {
                println!("{:<8} {:<12} {:<12} {:<10}", d_idx, src, dst, "DIFFER");
                println!("         standalone: {:?}", standalone_wp);
                println!("         embedded:   {:?}", embedded_wp);
            } else if differ_count == 11 {
                println!("         ... (further DIFFER lines suppressed) ...");
            }
        }
    }

    println!();
    println!("{}", "=".repeat(72));
    println!(
        "Summary: {} demands total, {} MATCH, {} DIFFER",
        num_demands, match_count, differ_count
    );

    if let Some((d_idx, src, dst, sw, ew)) = &first_differ {
        println!();
        println!(
            "First divergence at demand {} (src={} dst={}):",
            d_idx, src, dst
        );
        println!("  Standalone waypoints: {:?}", sw);
        println!("  Embedded waypoints:   {:?}", ew);
        println!();
        println!("Implementation differences that could explain this divergence:");
        println!("  1. Penalty application: standalone ADDITIVE (metric + penalty),");
        println!("     embedded MULTIPLICATIVE (metric * mult)");
        println!("  2. Low-saturation formula: standalone penalty = 100.0 * sat,");
        println!("     embedded mult = 1.0 + sat");
        println!("  3. High-saturation formula: standalone penalty = 100*(1/(1-sat)-1)*10,");
        println!("     embedded mult = 100*(1/(1-sat)-1)  [missing ×10 factor]");
        println!();
        println!(
            "Which difference first changed the routing decision at demand {} is",
            d_idx
        );
        println!("the subject of Validation Task V1. See RP403_V1_VALIDATION_REPORT.md.");
    } else {
        println!();
        println!(
            "UNEXPECTED: All {} demands produce identical waypoints.",
            num_demands
        );
        println!("The implementations are behaviourally equivalent on this instance.");
        println!("The setA-12 confound must have a different cause.");
    }

    // ── Evaluate original embedded solution ───────────────────────────────────
    let mut srpaths: Vec<SrPath> = Vec::new();
    for d_idx in 0..num_demands {
        if let Some(w) = embedded_assignments.get(&d_idx) {
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
    let solution = Solution { srpaths };
    let result = evaluator.evaluate_solution(&solution);
    let obj_str = if result.obj.is_finite() {
        format!("{:.4}", result.obj)
    } else {
        "inf".to_string()
    };
    println!();
    println!(
        "Original embedded RP-401C objective on setA-{}: {}",
        inst, obj_str
    );
    println!(
        "Standalone rp401c_ecmp_construction objective on setA-{}: 26.1200",
        inst
    );

    Ok(())
}
