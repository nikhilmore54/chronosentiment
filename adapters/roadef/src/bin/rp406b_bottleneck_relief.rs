/// rp406b_bottleneck_relief -- RP-406B: Bottleneck-Relief Micro-Repair
/// Classification: Research binary (RP-406B).
///
/// Strategy
/// --------
/// Phase 1 – Load best available prior solution (RP-405 → RP-403 → empty).
/// Phase 2 – Conditional bottleneck-relief micro-repair:
///   If any link has utilisation ≥ 1.0:
///     a. Identify the highest-utilisation link (bottleneck).
///     b. Find candidate demands whose ECMP-expanded route traverses it.
///     c. Rank candidates by flow contribution (volume × traversal count).
///     d. Reroute in batches of BATCH_SIZE using load-aware Dijkstra.
///     e. After each batch, evaluate objective and log relief curve.
///     f. Stop when objective is finite, no overloaded links remain,
///        or no improvement for MAX_STALL consecutive batches.
///     g. Rollback if final result is worse than starting point.
/// Phase 3 – Write solution JSON.
/// Phase 4 – Emit diagnostic table.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;

const BATCH_SIZE:   usize = 8;
const MAX_STALL:    usize = 3;
const MAX_SEG_CAP:  usize = 100;
const PENALTY:      f64   = 100.0;

// ── Config ────────────────────────────────────────────────────────────────────

struct Config {
    instance:     String,
    set_dir:      String,
    out_dir:      String,
    verbose:      bool,
    load_vector:  bool,
}

impl Config {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut instance = String::new();
        let mut set_dir  = "adapters/roadef/repo/challenge-roadef-2026-main/setA".to_string();
        let mut out_dir  = set_dir.clone();
        let mut verbose      = false;
        let mut load_vector  = false;
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--instance" => { i += 1; if i < args.len() { instance = args[i].clone(); } }
                "--set-dir"  => { i += 1; if i < args.len() { set_dir  = args[i].clone(); } }
                "--out-dir"  => { i += 1; if i < args.len() { out_dir  = args[i].clone(); } }
                "--verbose"      => { verbose = true; }
                "--load-vector"  => { load_vector = true; }
                _            => {}
            }
            i += 1;
        }
        if instance.is_empty() {
            eprintln!("Usage: rp406b_bottleneck_relief --instance <01..20> [--set-dir <path>] [--out-dir <path>] [--verbose] [--load-vector]");
            std::process::exit(1);
        }
        Config { instance, set_dir, out_dir, verbose, load_vector }
    }
}

// ── Dijkstra helpers ──────────────────────────────────────────────────────────

fn dijkstra_path(
    net: &Network,
    src: u64,
    dst: u64,
    disabled: &HashSet<u64>,
    sat: &HashMap<u64, f64>,
    penalty: f64,
) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let mut adj: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
    for l in &net.links {
        if disabled.contains(&l.id) { continue; }
        let s = sat.get(&l.id).copied().unwrap_or(0.0);
        let p = if s >= 1.0 {
            1e9
        } else if s > 0.8 {
            penalty * (1.0 / (1.0 - s) - 1.0) * 10.0
        } else {
            penalty * s
        };
        adj.entry(l.from).or_default().push((l.to, l.metric + p));
    }
    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut prev: HashMap<u64, u64> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>, u64)> = BinaryHeap::new();
    dist.insert(src, 0);
    heap.push((Reverse(0), src));
    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst { break; }
        if let Some(nbrs) = adj.get(&node) {
            for &(next, em) in nbrs {
                let nc = cost + (em * 1000.0) as u64;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > nc {
                    dist.insert(next, nc);
                    prev.insert(next, node);
                    heap.push((Reverse(nc), next));
                }
            }
        }
    }
    if !dist.contains_key(&dst) { return None; }
    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        if let Some(&p) = prev.get(&cur) { path.push(p); cur = p; } else { return None; }
    }
    path.reverse();
    Some(path)
}

fn path_to_waypoints(fp: &[u64], max_seg: usize) -> Vec<u64> {
    if fp.len() <= 2 { return vec![]; }
    let wp: Vec<u64> = fp[1..fp.len() - 1].to_vec();
    if max_seg > 0 && wp.len() + 1 > max_seg { wp[..max_seg - 1].to_vec() } else { wp }
}

// ── Solution loading helpers ──────────────────────────────────────────────────

fn load_srpaths_file(path: &str, nd: usize) -> Option<Vec<SrPath>> {
    if !std::path::Path::new(path).exists() { return None; }
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let arr = json["srpaths"].as_array()?;
    let mut srpaths: Vec<SrPath> = Vec::new();
    for e in arr {
        let d = e["d"].as_u64()? as usize;
        let t = e["t"].as_u64()? as usize;
        let w: Vec<u64> = e["w"].as_array()?.iter().filter_map(|v| v.as_u64()).collect();
        if d < nd { srpaths.push(SrPath { d, t, w }); }
    }
    Some(srpaths)
}

fn srpaths_to_maps(srpaths: &[SrPath]) -> (HashMap<usize, Vec<u64>>, HashMap<usize, Vec<u64>>) {
    let mut t0: HashMap<usize, Vec<u64>> = HashMap::new();
    let mut t1: HashMap<usize, Vec<u64>> = HashMap::new();
    for sp in srpaths {
        if sp.t == 0 { t0.insert(sp.d, sp.w.clone()); }
        else         { t1.insert(sp.d, sp.w.clone()); }
    }
    (t0, t1)
}

fn build_srpaths(
    t0: &HashMap<usize, Vec<u64>>,
    t1: &HashMap<usize, Vec<u64>>,
    nd: usize,
    ns: usize,
) -> Vec<SrPath> {
    let mut v = Vec::new();
    for d in 0..nd {
        if let Some(w) = t0.get(&d) { if !w.is_empty() { v.push(SrPath { d, t: 0, w: w.clone() }); } }
        if ns > 1 {
            if let Some(w) = t1.get(&d) { if !w.is_empty() { v.push(SrPath { d, t: 1, w: w.clone() }); } }
        }
    }
    v
}

// ── Utilisation helpers ───────────────────────────────────────────────────────

fn compute_sat(
    ev: &RoadefEvaluator,
    srpaths: &[SrPath],
    t: usize,
    cap: &HashMap<u64, f64>,
) -> HashMap<u64, f64> {
    let sol = Solution { srpaths: srpaths.to_vec() };
    let mut sat: HashMap<u64, f64> = HashMap::new();
    if let Some(loads) = ev.compute_loads(t, &sol) {
        for (id, flow) in &loads.arc_flows {
            let c = cap.get(id).copied().unwrap_or(1.0);
            sat.insert(*id, if c > 0.0 { flow / c } else { f64::INFINITY });
        }
    }
    sat
}

/// Compute per-link utilisation taking the maximum across all time slots.
/// This ensures the repair loop detects overloads at t=1 as well as t=0.
fn compute_combined_sat(
    ev: &RoadefEvaluator,
    srpaths: &[SrPath],
    ns: usize,
    cap: &HashMap<u64, f64>,
) -> HashMap<u64, f64> {
    let mut combined: HashMap<u64, f64> = HashMap::new();
    for t in 0..ns {
        let sat = compute_sat(ev, srpaths, t, cap);
        for (id, s) in sat {
            let e = combined.entry(id).or_insert(0.0);
            if s > *e { *e = s; }
        }
    }
    combined
}

/// Returns (link_id, from, to, utilisation) for the highest-utilisation link.
fn highest_util_link(
    net: &Network,
    sat: &HashMap<u64, f64>,
) -> Option<(u64, u64, u64, f64)> {
    net.links.iter()
        .map(|l| (l.id, l.from, l.to, sat.get(&l.id).copied().unwrap_or(0.0)))
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
}

fn overloaded_count(sat: &HashMap<u64, f64>) -> usize {
    sat.values().filter(|&&s| s >= 1.0).count()
}

fn max_util(sat: &HashMap<u64, f64>) -> f64 {
    sat.values().cloned().fold(0.0_f64, f64::max)
}

// ── Candidate demand identification ──────────────────────────────────────────
//
// A demand is a candidate if its ECMP-expanded route (via the evaluator's
// compute_loads arc_flows) traverses the bottleneck link.  We approximate
// this by checking whether the demand's waypoint path (src → w[0] → … → dst)
// contains the edge (from, to).  For demands with no explicit SR path (empty
// waypoints), the ECMP default path is approximated by the shortest-path
// Dijkstra on unweighted metric.

fn plain_dijkstra(net: &Network, src: u64, dst: u64, disabled: &HashSet<u64>) -> Option<Vec<u64>> {
    if src == dst { return Some(vec![src]); }
    let mut adj: HashMap<u64, Vec<(u64, u64)>> = HashMap::new();
    for l in &net.links {
        if disabled.contains(&l.id) { continue; }
        adj.entry(l.from).or_default().push((l.to, (l.metric * 1000.0) as u64));
    }
    let mut dist: HashMap<u64, u64> = HashMap::new();
    let mut prev: HashMap<u64, u64> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u64>, u64)> = BinaryHeap::new();
    dist.insert(src, 0);
    heap.push((Reverse(0), src));
    while let Some((Reverse(cost), node)) = heap.pop() {
        if dist.get(&node).copied().unwrap_or(u64::MAX) < cost { continue; }
        if node == dst { break; }
        if let Some(nbrs) = adj.get(&node) {
            for &(next, w) in nbrs {
                let nc = cost + w;
                if dist.get(&next).copied().unwrap_or(u64::MAX) > nc {
                    dist.insert(next, nc);
                    prev.insert(next, node);
                    heap.push((Reverse(nc), next));
                }
            }
        }
    }
    if !dist.contains_key(&dst) { return None; }
    let mut path = vec![dst];
    let mut cur = dst;
    while cur != src {
        if let Some(&p) = prev.get(&cur) { path.push(p); cur = p; } else { return None; }
    }
    path.reverse();
    Some(path)
}

/// Returns the set of demand indices whose approximate route traverses (from, to).
fn candidate_demands(
    demands: &[(usize, u64, u64, f64)],   // (idx, src, dst, vol)
    t0_map: &HashMap<usize, Vec<u64>>,
    net: &Network,
    disabled: &HashSet<u64>,
    bn_from: u64,
    bn_to: u64,
) -> Vec<(usize, f64)> {  // (demand_idx, flow_score)
    let mut result: Vec<(usize, f64)> = Vec::new();
    for &(d, src, dst, vol) in demands {
        let waypoints = t0_map.get(&d).cloned().unwrap_or_default();
        // Build full node sequence: src → wp[0] → … → wp[n-1] → dst
        let full: Vec<u64> = if waypoints.is_empty() {
            // Approximate ECMP path via plain Dijkstra
            plain_dijkstra(net, src, dst, disabled).unwrap_or_else(|| vec![src, dst])
        } else {
            let mut f = vec![src];
            f.extend_from_slice(&waypoints);
            f.push(dst);
            f
        };
        // Count how many times the bottleneck edge appears (usually 0 or 1)
        let traversals = full.windows(2).filter(|w| w[0] == bn_from && w[1] == bn_to).count();
        if traversals > 0 {
            result.push((d, vol * traversals as f64));
        }
    }
    // Sort descending by flow score (highest contributors first)
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

// ── Relief curve record ───────────────────────────────────────────────────────

struct ReliefPoint {
    batch:            usize,
    demands_rerouted: usize,
    bottleneck_link:  u64,
    bn_util:          f64,
    mlu:              f64,
    _overloaded:      usize,
    objective:        f64,
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let cfg = Config::from_args();
    let set_dir = &cfg.set_dir;
    let inst    = &cfg.instance;

    // ── Load instance ─────────────────────────────────────────────────────────
    let net = load_network(&format!("{}/setA-{}-net.json",      set_dir, inst))?;
    let tm  = load_traffic_matrix(&format!("{}/setA-{}-tm.json", set_dir, inst))?;
    let sc  = load_scenario(&format!("{}/setA-{}-scenario.json", set_dir, inst))?;

    let nd = tm.demands.len();
    let ns = tm.num_time_slots;
    let max_seg = if sc.max_segments >= 0 { sc.max_segments as usize } else { MAX_SEG_CAP };

    let ev  = RoadefEvaluator::new(&net, tm.clone(), sc.clone());
    let cap: HashMap<u64, f64> = net.links.iter().map(|l| (l.id, l.capacity)).collect();

    let dis_t0: HashSet<u64> = sc.interventions.iter()
        .filter(|i| i.t == 0).flat_map(|i| i.links.iter().copied()).collect();
    let dis_t1: HashSet<u64> = sc.interventions.iter()
        .filter(|i| i.t == 1).flat_map(|i| i.links.iter().copied()).collect();
    let dis_both: HashSet<u64> = dis_t0.union(&dis_t1).copied().collect();

    let demands: Vec<(usize, u64, u64, f64)> = tm.demands.iter().enumerate().map(|(i, d)| {
        let v0 = d.v[0];
        let v1 = if d.v.len() > 1 { d.v[1] } else { d.v[0] };
        (i, d.s, d.t, (v0 + v1) / 2.0)
    }).collect();

    // ── Phase 1: load best prior solution ────────────────────────────────────
    let rp405_path = format!("{}/setA-{}-srpaths-rp405-adaptive.json", set_dir, inst);
    let rp403_path = format!("{}/setA-{}-srpaths-rp403.json",          set_dir, inst);

    let prior_srpaths = load_srpaths_file(&rp405_path, nd)
        .or_else(|| load_srpaths_file(&rp403_path, nd))
        .unwrap_or_default();

    let prior_sol = Solution { srpaths: prior_srpaths.clone() };
    let prior_result = ev.evaluate_solution(&prior_sol);
    let prior_obj = prior_result.obj;

    let (mut best_t0, mut best_t1) = srpaths_to_maps(&prior_srpaths);

    eprintln!("RP-406B  setA-{}  prior_obj={:.6}  nd={}  ns={}",
        inst, prior_obj, nd, ns);

    // ── Phase 2: conditional bottleneck-relief micro-repair ───────────────────
    let t_repair_start = Instant::now();

    let mut relief_curve: Vec<ReliefPoint> = Vec::new();
    let mut total_rerouted = 0usize;
    let mut batches_executed = 0usize;
    let mut initial_bn_util = 0.0f64;
    let mut initial_bn_link = 0u64;
    let mut final_bn_util   = 0.0f64;
    let mut repair_activated = false;

    // Evaluate initial utilisation
    let init_srpaths = build_srpaths(&best_t0, &best_t1, nd, ns);
    let init_sat = compute_combined_sat(&ev, &init_srpaths, ns, &cap);
    let init_overloaded = overloaded_count(&init_sat);
    let init_mlu = max_util(&init_sat);

    if init_overloaded > 0 {
        repair_activated = true;

        // Record baseline point
        if let Some((lid, _from, _to, util)) = highest_util_link(&net, &init_sat) {
            initial_bn_link = lid;
            initial_bn_util = util;
            final_bn_util   = util;
        }

        relief_curve.push(ReliefPoint {
            batch: 0,
            demands_rerouted: 0,
            bottleneck_link: initial_bn_link,
            bn_util: initial_bn_util,
            mlu: init_mlu,
            _overloaded: init_overloaded,
            objective: prior_obj,
        });

        let mut current_t0 = best_t0.clone();
        let mut current_t1 = best_t1.clone();
        let mut current_obj = prior_obj;
        let mut current_mlu = init_mlu;
        let mut current_overloaded = init_overloaded;
        let mut stall = 0usize;

        loop {
            // Recompute utilisation on current solution
            let cur_srpaths = build_srpaths(&current_t0, &current_t1, nd, ns);
            let cur_sat = compute_combined_sat(&ev, &cur_srpaths, ns, &cap);
            let cur_overloaded = overloaded_count(&cur_sat);
            current_mlu = max_util(&cur_sat);

            if cur_overloaded == 0 { break; }

            // Find bottleneck link
            let (bn_lid, bn_from, bn_to, bn_util) = match highest_util_link(&net, &cur_sat) {
                Some(x) => x,
                None    => break,
            };
            final_bn_util = bn_util;

            // Identify candidate demands traversing bottleneck
            let candidates = candidate_demands(
                &demands, &current_t0, &net, &dis_both, bn_from, bn_to,
            );

            if candidates.is_empty() {
                eprintln!("  [batch {}] No candidates for link {} ({}->{}) util={:.4}; stopping.",
                    batches_executed, bn_lid, bn_from, bn_to, bn_util);
                break;
            }

            // Take next batch (skip already-rerouted in this pass by tracking
            // which demands we've already attempted this iteration)
            let batch: Vec<usize> = candidates.iter()
                .map(|(d, _)| *d)
                .take(BATCH_SIZE)
                .collect();

            if cfg.verbose {
                eprintln!("  [batch {}] bottleneck=link{} ({}->{}) util={:.4}  candidates={}  batch={:?}",
                    batches_executed + 1, bn_lid, bn_from, bn_to, bn_util,
                    candidates.len(), batch);
            }

            // Reroute batch using load-aware Dijkstra.
            // Scenario-consistent rerouting: install the same SR path in every
            // time slot so that SrPathBit::dist(t, t+1) = 0 for all t, keeping
            // the reconfiguration budget cost unchanged.
            let mut trial_t0 = current_t0.clone();
            let mut trial_t1 = best_t1.clone();
            // Build partial sat excluding the batch demands
            let partial_srpaths: Vec<SrPath> = cur_srpaths.iter()
                .filter(|sp| sp.t == 0 && !batch.contains(&sp.d))
                .cloned()
                .collect();
            let mut routing_sat = compute_sat(&ev, &partial_srpaths, 0, &cap);
            for l in &net.links { routing_sat.entry(l.id).or_insert(0.0); }

            let mut rerouted_this_batch = 0usize;
            for &d in &batch {
                let (_, src, dst, _) = demands[d];
                let fp = dijkstra_path(&net, src, dst, &dis_both, &routing_sat, PENALTY)
                    .or_else(|| plain_dijkstra(&net, src, dst, &dis_both));
                if let Some(fp) = fp {
                    let wp = path_to_waypoints(&fp, max_seg);
                    // Install in t=0
                    trial_t0.insert(d, wp.clone());
                    // Install the same path in all other time slots (scenario-consistent)
                    if ns > 1 {
                        trial_t1.insert(d, wp.clone());
                    }
                    // Update routing_sat incrementally
                    let trial_sp = build_srpaths(&trial_t0, &trial_t1, nd, ns);
                    let new_sat = compute_sat(&ev, &trial_sp, 0, &cap);
                    routing_sat = new_sat;
                    rerouted_this_batch += 1;
                }
            }

            // Evaluate trial solution
            let trial_srpaths = build_srpaths(&trial_t0, &trial_t1, nd, ns);
            let trial_sol = Solution { srpaths: trial_srpaths.clone() };
            let trial_result = ev.evaluate_solution(&trial_sol);
            let trial_obj = trial_result.obj;

            batches_executed += 1;
            total_rerouted += rerouted_this_batch;

            let trial_sat = compute_combined_sat(&ev, &trial_srpaths, ns, &cap);
            let trial_overloaded = overloaded_count(&trial_sat);
            let trial_mlu = max_util(&trial_sat);

            relief_curve.push(ReliefPoint {
                batch: batches_executed,
                demands_rerouted: total_rerouted,
                bottleneck_link: bn_lid,
                bn_util: trial_sat.get(&bn_lid).copied().unwrap_or(bn_util),
                mlu: trial_mlu,
                _overloaded: trial_overloaded,
                objective: trial_obj,
            });

            // Log all overloaded links after this batch (diagnostic instrumentation)
            if cfg.verbose {
                let overloaded_links: Vec<(u64, f64)> = net.links.iter()
                    .filter_map(|l| {
                        let s = trial_sat.get(&l.id).copied().unwrap_or(0.0);
                        if s >= 1.0 { Some((l.id, s)) } else { None }
                    })
                    .collect();
                if overloaded_links.is_empty() {
                    eprintln!("  [batch {}] Overloaded links: none", batches_executed);
                } else {
                    eprintln!("  [batch {}] Overloaded links ({}):", batches_executed, overloaded_links.len());
                    for (lid, s) in &overloaded_links {
                        eprintln!("    link {}  util={:.6}", lid, s);
                    }
                }
            }

            // Two-stage acceptance:
            //   Phase A (inf objective): accept if overloaded_count decreases OR MLU decreases
            //   Phase B (finite objective): accept only if objective strictly improves
            let improved = if trial_obj.is_finite() && !current_obj.is_finite() {
                // Feasibility restored — always accept
                true
            } else if trial_obj.is_finite() && current_obj.is_finite() {
                // Both finite: standard objective-based acceptance
                trial_obj < current_obj - 1e-9
            } else {
                // Both inf: feasibility-recovery mode — accept if overload state improves
                trial_overloaded < current_overloaded
                    || trial_mlu < current_mlu - 1e-9
            };

            if improved {
                current_t0 = trial_t0;
                current_t1 = trial_t1;
                current_obj = trial_obj;
                current_mlu = trial_mlu;
                current_overloaded = trial_overloaded;
                stall = 0;
            } else {
                stall += 1;
                if stall >= MAX_STALL {
                    eprintln!("  Stalled for {} batches; stopping micro-repair.", MAX_STALL);
                    break;
                }
            }

            // Stop if objective is now finite
            if current_obj.is_finite() { break; }
        }

        // Accept repaired solution only if it is at least as good as prior
        let repaired_srpaths = build_srpaths(&current_t0, &current_t1, nd, ns);
        let repaired_result = ev.evaluate_solution(&Solution { srpaths: repaired_srpaths.clone() });
        let repaired_obj = repaired_result.obj;
        let repaired_sat = compute_combined_sat(&ev, &repaired_srpaths, ns, &cap);
        let repaired_mlu = max_util(&repaired_sat);
        let prior_sat_check = compute_combined_sat(&ev, &init_srpaths, ns, &cap);
        let prior_mlu_check = max_util(&prior_sat_check);

        if cfg.verbose {
            eprintln!("[post-repair] obj={} mlu={:.6} valid={}",
                if repaired_obj.is_finite() { format!("{:.6}", repaired_obj) } else { "inf".to_string() },
                repaired_mlu, repaired_result.valid);
        }

        let accept = if repaired_obj.is_finite() && !prior_obj.is_finite() {
            true  // finite beats inf unconditionally
        } else if repaired_obj.is_finite() && prior_obj.is_finite() {
            repaired_obj <= prior_obj + 1e-9
        } else {
            // Both inf: accept if MLU strictly improved (feasibility-recovery progress)
            repaired_mlu < prior_mlu_check - 1e-9
        };

        if accept {
            best_t0 = current_t0;
            best_t1 = current_t1;
        } else {
            eprintln!("  Micro-repair did not improve; rolling back to prior solution.");
        }
    }

    let repair_ms = t_repair_start.elapsed().as_millis();

    // ── Phase 3: write solution ───────────────────────────────────────────────
    let final_srpaths = build_srpaths(&best_t0, &best_t1, nd, ns);
    let final_sol = Solution { srpaths: final_srpaths.clone() };
    let final_result = ev.evaluate_solution(&final_sol);
    let final_obj = final_result.obj;

    // Compute final utilisation for diagnostics
    let final_sat = compute_combined_sat(&ev, &final_srpaths, ns, &cap);
    if let Some((_, _, _, u)) = highest_util_link(&net, &final_sat) {
        final_bn_util = u;
    }

    std::fs::create_dir_all(&cfg.out_dir)?;
    let out_path = format!("{}/setA-{}-srpaths-rp406b.json", cfg.out_dir, inst);
    let srpaths_json: Vec<serde_json::Value> = final_srpaths.iter().map(|sp| {
        serde_json::json!({ "d": sp.d, "t": sp.t, "w": sp.w })
    }).collect();
    let json_out = serde_json::json!({ "srpaths": srpaths_json });
    let mut f = File::create(&out_path)?;
    writeln!(f, "{}", serde_json::to_string_pretty(&json_out)?)?;

    // ── Phase 4: emit diagnostics ─────────────────────────────────────────────
    let candidates_count = if repair_activated {
        let init_srpaths2 = build_srpaths(&best_t0, &best_t1, nd, ns);
        let init_sat2 = compute_combined_sat(&ev, &init_srpaths2, ns, &cap);
        if let Some((_, bn_from, bn_to, _)) = highest_util_link(&net, &init_sat2) {
            candidate_demands(&demands, &best_t0, &net, &dis_both, bn_from, bn_to).len()
        } else { 0 }
    } else { 0 };

    eprintln!();
    eprintln!("┌─────────────────────────────────────────────────────────────────┐");
    eprintln!("│  RP-406B Diagnostics  setA-{}                                   │", inst);
    eprintln!("├──────────────────────────────┬──────────────────────────────────┤");
    eprintln!("│ Repair activated             │ {:<32} │", if repair_activated { "yes" } else { "no (no overloaded links)" });
    eprintln!("│ Bottleneck link              │ {:<32} │", if repair_activated { initial_bn_link.to_string() } else { "—".to_string() });
    eprintln!("│ Candidate demands            │ {:<32} │", if repair_activated { candidates_count.to_string() } else { "—".to_string() });
    eprintln!("│ Rerouted demands             │ {:<32} │", total_rerouted);
    eprintln!("│ Batches executed             │ {:<32} │", batches_executed);
    eprintln!("│ Runtime (micro-repair)       │ {:<32} │", format!("{} ms", repair_ms));
    eprintln!("│ Initial utilisation (BN)     │ {:<32} │", if repair_activated { format!("{:.6}", initial_bn_util) } else { "—".to_string() });
    eprintln!("│ Final utilisation (BN)       │ {:<32} │", if repair_activated { format!("{:.6}", final_bn_util) } else { "—".to_string() });
    eprintln!("│ Prior objective              │ {:<32} │", if prior_obj.is_finite() { format!("{:.6}", prior_obj) } else { "inf".to_string() });
    eprintln!("│ Final objective              │ {:<32} │", if final_obj.is_finite() { format!("{:.6}", final_obj) } else { "inf".to_string() });
    eprintln!("└──────────────────────────────┴──────────────────────────────────┘");

    // Relief curve table (only when repair was activated)
    if repair_activated && !relief_curve.is_empty() {
        eprintln!();
        eprintln!("  Bottleneck Relief Curve");
        eprintln!("  {:>5}  {:>8}  {:>10}  {:>8}  {:>10}  {:>12}",
            "batch", "rerouted", "bn_link", "bn_util", "mlu", "objective");
        eprintln!("  {}", "-".repeat(62));
        for pt in &relief_curve {
            let obj_str = if pt.objective.is_finite() {
                format!("{:.6}", pt.objective)
            } else {
                "inf".to_string()
            };
            eprintln!("  {:>5}  {:>8}  {:>10}  {:>8.4}  {:>10.4}  {:>12}",
                pt.batch, pt.demands_rerouted, pt.bottleneck_link,
                pt.bn_util, pt.mlu, obj_str);
        }
    }

    // Summary line for batch benchmarking
    let delta_str = if prior_obj.is_finite() && final_obj.is_finite() {
        format!("{:+.6}", final_obj - prior_obj)
    } else if !prior_obj.is_finite() && final_obj.is_finite() {
        "inf->finite".to_string()
    } else if prior_obj.is_finite() && !final_obj.is_finite() {
        "finite->inf".to_string()
    } else {
        "both inf".to_string()
    };

    println!("{:<10}  prior={:<14}  final={:<14}  delta={:<14}  rerouted={:<6}  batches={:<4}  repair_ms={:<8}",
        format!("setA-{}", inst),
        if prior_obj.is_finite() { format!("{:.6}", prior_obj) } else { "inf".to_string() },
        if final_obj.is_finite() { format!("{:.6}", final_obj) } else { "inf".to_string() },
        delta_str,
        total_rerouted,
        batches_executed,
        repair_ms,
    );

    // ── Load vector output (competition criterion) ────────────────────────────
    {
        // Compute per-link utilisation across all time slots, sort descending
        let mut sorted_util: Vec<f64> = net.links.iter()
            .map(|l| final_sat.get(&l.id).copied().unwrap_or(0.0))
            .collect();
        sorted_util.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let top_n = 30.min(sorted_util.len());
        println!("========================================================");
        println!("RP-406B RESULT  setA-{}", inst);
        println!("========================================================");
        println!();
        println!("Objective : {}", if final_obj.is_finite() { format!("{:.6}", final_obj) } else { "inf".to_string() });
        println!("Valid     : {}", final_result.valid);
        println!();
        println!("MLU       : {:.6}", sorted_util.first().copied().unwrap_or(0.0));
        println!();
        println!("Top-{} Load Vector", top_n);
        println!("--------------------------------------------------------");
        for (i, &v) in sorted_util.iter().take(top_n).enumerate() {
            println!("{:3}  {:.6}", i + 1, v);
        }
        println!("--------------------------------------------------------");

        // Export full sorted load vector to CSV
        if cfg.load_vector {
            let csv_path = format!("{}/setA-{}-loadvec-rp406b.csv", cfg.out_dir, inst);
            let mut csv = File::create(&csv_path)?;
            writeln!(csv, "instance,rank,load")?;
            for (i, &v) in sorted_util.iter().enumerate() {
                writeln!(csv, "setA-{},{},{:.9}", inst, i + 1, v)?;
            }
            eprintln!("Load vector CSV written to {}", csv_path);
        }
    }


    Ok(())
}