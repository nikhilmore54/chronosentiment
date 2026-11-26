// roadef_qualification.rs – Official ROADEF 2026 Qualification Solver
// Accepts: <net> <tm> <scenario> <output> [-t <time_limit>] [-s <seed>] [--threads <threads>]
// Supports both positional arguments and flag-based arguments.
// Integrates exact degree-1 topological floor detection, budget-gated surgical detours,
// shared greedy routing, and strict lexicographical acceptance gating.

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::env;

use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Network, Solution, SrPath};

// ---------------------------------------------------------------------------
// Lexicographical Comparison Helper
// ---------------------------------------------------------------------------
fn is_lex_strictly_better(v_cand: &[f64], v_ref: &[f64]) -> bool {
    let tol = 1e-6;
    let n = v_cand.len().min(v_ref.len());
    for i in 0..n {
        let diff = v_cand[i] - v_ref[i];
        if diff < -tol {
            return true;
        } else if diff > tol {
            return false;
        }
    }
    v_cand.len() < v_ref.len()
}

// ---------------------------------------------------------------------------
// Topological Invariant Lower Bound Computation
// ---------------------------------------------------------------------------
fn compute_topological_floor(
    net: &Network,
    tm: &roadef::models::TrafficMatrix,
    scenario: &roadef::models::Scenario,
    num_slots: usize,
) -> f64 {
    let mut link_caps: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    for link in &net.links {
        link_caps.insert(link.id, link.capacity);
    }
    let mut max_floor = 0.0f64;
    for t in 0..num_slots {
        let disabled_in_slot: HashSet<u64> = scenario
            .interventions
            .iter()
            .filter(|i| i.t == t)
            .flat_map(|i| i.links.iter().copied())
            .collect();
        let mut active_out: std::collections::HashMap<u64, Vec<u64>> =
            std::collections::HashMap::new();
        let mut active_in: std::collections::HashMap<u64, Vec<u64>> =
            std::collections::HashMap::new();
        for link in &net.links {
            if disabled_in_slot.contains(&link.id) || link.capacity <= 0.0 {
                continue;
            }
            active_out.entry(link.from).or_default().push(link.id);
            active_in.entry(link.to).or_default().push(link.id);
        }
        let mut forced_loads: std::collections::HashMap<u64, f64> =
            std::collections::HashMap::new();
        for d in &tm.demands {
            let vol = if t < d.v.len() { d.v[t] } else { 0.0 };
            if vol <= 0.0 {
                continue;
            }
            let s_out = active_out.get(&d.s);
            let t_in = active_in.get(&d.t);
            let out_lid = if let Some(arcs) = s_out {
                if arcs.len() == 1 {
                    Some(arcs[0])
                } else {
                    None
                }
            } else {
                None
            };
            let in_lid = if let Some(arcs) = t_in {
                if arcs.len() == 1 {
                    Some(arcs[0])
                } else {
                    None
                }
            } else {
                None
            };
            match (out_lid, in_lid) {
                (Some(o), Some(i)) => {
                    if o == i {
                        *forced_loads.entry(o).or_insert(0.0) += vol;
                    } else {
                        *forced_loads.entry(o).or_insert(0.0) += vol;
                        *forced_loads.entry(i).or_insert(0.0) += vol;
                    }
                }
                (Some(o), None) => {
                    *forced_loads.entry(o).or_insert(0.0) += vol;
                }
                (None, Some(i)) => {
                    *forced_loads.entry(i).or_insert(0.0) += vol;
                }
                (None, None) => {}
            }
        }
        for (lid, load) in forced_loads {
            if let Some(&cap) = link_caps.get(&lid) {
                if cap > 0.0 {
                    let sat = load / cap;
                    if sat > max_floor {
                        max_floor = sat;
                    }
                }
            }
        }
    }
    max_floor
}

// ---------------------------------------------------------------------------
// Per-instance solver pipeline
// ---------------------------------------------------------------------------
fn run_instance(net: &Network, tm: &roadef::models::TrafficMatrix, scenario: &roadef::models::Scenario, out_path: &str) -> anyhow::Result<()> {
    let num_demands = tm.demands.len();
    let num_slots = tm.num_time_slots;
    let max_seg = if scenario.max_segments >= 0 {
        scenario.max_segments as usize
    } else {
        100
    };

    let evaluator = RoadefEvaluator::new(net, tm.clone(), scenario.clone());

    // 1. Evaluate canonical empty ECMP baseline
    let empty_sol = Solution { srpaths: vec![] };
    let (empty_res, _) = evaluator.evaluate_solution_cached(&empty_sol);
    let mut best_srpaths: Vec<SrPath> = Vec::new();
    let mut best_res = roadef::evaluator::EvaluationResult { valid: empty_res.valid, obj: empty_res.obj };
    let mut best_lex = evaluator.compute_lex_vector(&empty_sol);
    let best_mlu = best_lex.as_ref().and_then(|v| v.first().copied()).unwrap_or(0.0);

    // Compute theoretical degree-1 leaf cut-edge saturation floor
    let topo_floor = compute_topological_floor(net, tm, scenario, num_slots);

    // 2. Budget-gated surgical 1-hop detour pass from empty baseline
    if best_mlu >= 0.9999 || best_res.obj >= 999_999.0 {
        let mut detour_paths = Vec::new();
        let mut detour_res = roadef::evaluator::EvaluationResult { valid: best_res.valid, obj: best_res.obj };
        run_surgical_detours(
            net,
            tm,
            scenario,
            &evaluator,
            &mut detour_paths,
            &mut detour_res,
            num_slots,
            max_seg,
        );
        if detour_res.valid && !detour_paths.is_empty() {
            if let Some(lex) = evaluator.compute_lex_vector(&Solution { srpaths: detour_paths.clone() }) {
                if best_lex.is_none() || is_lex_strictly_better(&lex, best_lex.as_ref().unwrap()) {
                    println!("Detour from empty improved: obj={:.4}", detour_res.obj);
                    best_srpaths = detour_paths;
                    best_res = detour_res;
                    best_lex = Some(lex);
                }
            }
        }
    }

    let cur_mlu = best_lex.as_ref().and_then(|v| v.first().copied()).unwrap_or(0.0);

    // 3. Skip heavy greedy search if already matching topological global optimum or high demand fleet with finite detour
    let skip_greedy = (cur_mlu <= topo_floor + 1e-6 && topo_floor >= 0.99)
        || (num_demands > 10_000 && cur_mlu <= 0.995 && best_res.valid)
        || (num_demands > 6_000 && cur_mlu < 0.98 && !best_srpaths.is_empty());

    if !skip_greedy {
        let disabled_all: HashSet<u64> = scenario.interventions.iter().flat_map(|i| i.links.iter().copied()).collect();
        let shared_assign = solve_greedy(net, &tm.demands, num_slots, scenario, &disabled_all, max_seg);

        let mut greedy_paths: Vec<SrPath> = Vec::new();
        for d_idx in 0..num_demands {
            if let Some(w) = shared_assign.get(&d_idx) {
                if !w.is_empty() {
                    for slot in 0..num_slots {
                        greedy_paths.push(SrPath { d: d_idx, t: slot, w: w.clone() });
                    }
                }
            }
        }

        let cand_sol = Solution { srpaths: greedy_paths.clone() };
        let (mut cand_res, _) = evaluator.evaluate_solution_cached(&cand_sol);
        let cand_mlu = evaluator.compute_lex_vector(&cand_sol).as_ref().and_then(|v| v.first().copied()).unwrap_or(0.0);

        if cand_res.valid && (cand_mlu >= 0.85 || cand_res.obj >= 999_999.0) {
            run_surgical_detours(
                net,
                tm,
                scenario,
                &evaluator,
                &mut greedy_paths,
                &mut cand_res,
                num_slots,
                max_seg,
            );
        }

        if cand_res.valid {
            if let Some(cand_lex) = evaluator.compute_lex_vector(&Solution { srpaths: greedy_paths.clone() }) {
                if best_lex.is_none() || is_lex_strictly_better(&cand_lex, best_lex.as_ref().unwrap()) {
                    println!("Greedy solution accepted: obj={:.4}", cand_res.obj);
                    best_srpaths = greedy_paths;
                    best_res = cand_res;
                    best_lex = Some(cand_lex);
                } else {
                    println!("Greedy solution rejected by lexicographical gate (regressed downstream ranks)");
                }
            }
        }
    }

    let final_mlu = best_lex.as_ref().and_then(|v| v.first().copied()).unwrap_or(0.0);
    println!("Final solution: valid={} mlu={:.6} srpaths={}", best_res.valid, final_mlu, best_srpaths.len());

    // Serialize JSON
    let sol_json = serde_json::json!({
        "srpaths": best_srpaths.iter().map(|p| serde_json::json!({
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
// Surgical 1-hop detour pass for solvable core-graph overloads
// ---------------------------------------------------------------------------
fn run_surgical_detours(
    net: &Network,
    tm: &roadef::models::TrafficMatrix,
    scenario: &roadef::models::Scenario,
    evaluator: &RoadefEvaluator,
    srpaths: &mut Vec<SrPath>,
    result: &mut roadef::evaluator::EvaluationResult,
    num_slots: usize,
    _max_segments: usize,
) {
    if result.valid && result.obj < 999_999.0 {
        // Already finite and sub-saturation
        return;
    }

    // Precompute forced saturation floors
    let mut forced_sat: Vec<std::collections::HashMap<u64, f64>> = vec![std::collections::HashMap::new(); num_slots];
    let mut link_caps: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
    let mut link_endpoints: std::collections::HashMap<u64, (u64, u64)> = std::collections::HashMap::new();
    let mut active_out_arcs: Vec<std::collections::HashMap<u64, Vec<u64>>> = vec![std::collections::HashMap::new(); num_slots];
    let mut active_in_arcs: Vec<std::collections::HashMap<u64, Vec<u64>>> = vec![std::collections::HashMap::new(); num_slots];
    let mut out_neighbors: Vec<std::collections::HashMap<u64, Vec<u64>>> = vec![std::collections::HashMap::new(); num_slots];
    let mut in_neighbors: Vec<std::collections::HashMap<u64, Vec<u64>>> = vec![std::collections::HashMap::new(); num_slots];

    for link in &net.links {
        link_caps.insert(link.id, link.capacity);
        link_endpoints.insert(link.id, (link.from, link.to));
    }

    for t in 0..num_slots {
        let disabled_in_slot: HashSet<u64> = scenario.interventions.iter()
            .filter(|i| i.t == t)
            .flat_map(|i| i.links.iter().copied())
            .collect();

        for link in &net.links {
            if disabled_in_slot.contains(&link.id) || link.capacity <= 0.0 {
                continue;
            }
            active_out_arcs[t].entry(link.from).or_default().push(link.id);
            active_in_arcs[t].entry(link.to).or_default().push(link.id);
            out_neighbors[t].entry(link.from).or_default().push(link.to);
            in_neighbors[t].entry(link.to).or_default().push(link.from);
        }

        let mut forced_load: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
        for d in &tm.demands {
            let vol = if t < d.v.len() { d.v[t] } else { 0.0 };
            if vol <= 0.0 {
                continue;
            }
            let out_arc = active_out_arcs[t].get(&d.s).and_then(|arcs| if arcs.len() == 1 { Some(arcs[0]) } else { None });
            let in_arc = active_in_arcs[t].get(&d.t).and_then(|arcs| if arcs.len() == 1 { Some(arcs[0]) } else { None });

            match (out_arc, in_arc) {
                (Some(e_out), Some(e_in)) => {
                    if e_out == e_in {
                        *forced_load.entry(e_out).or_insert(0.0) += vol;
                    } else {
                        *forced_load.entry(e_out).or_insert(0.0) += vol;
                        *forced_load.entry(e_in).or_insert(0.0) += vol;
                    }
                }
                (Some(e_out), None) => {
                    *forced_load.entry(e_out).or_insert(0.0) += vol;
                }
                (None, Some(e_in)) => {
                    *forced_load.entry(e_in).or_insert(0.0) += vol;
                }
                (None, None) => {}
            }
        }

        for (link_id, load) in forced_load {
            if let Some(&cap) = link_caps.get(&link_id) {
                if cap > 0.0 {
                    forced_sat[t].insert(link_id, load / cap);
                }
            }
        }
    }

    // Iterate through slots that have overloads
    for t in 0..num_slots {
        let cur_sol = Solution { srpaths: srpaths.clone() };
        let loads = match evaluator.compute_loads(t, &cur_sol) {
            Some(l) => l,
            None => continue,
        };
        if loads.mlu < 0.9999 {
            continue;
        }

        // Find worst core link in slot t
        let mut worst_core_link: Option<(u64, f64)> = None;
        for (&lid, &sat) in &loads.arc_saturations {
            let floor = forced_sat[t].get(&lid).copied().unwrap_or(0.0);
            if sat <= floor + 1e-6 {
                continue;
            }
            if sat >= 0.9999 {
                if worst_core_link.map(|(_, best_s)| sat > best_s).unwrap_or(true) {
                    worst_core_link = Some((lid, sat));
                }
            }
        }

        let (bottle_link, _) = match worst_core_link {
            Some(pair) => pair,
            None => continue, // only degree-1 leaf bottlenecks in this slot
        };

        let (u, v) = match link_endpoints.get(&bottle_link) {
            Some(&ends) => ends,
            None => continue,
        };

        let _disabled_in_slot: HashSet<u64> = scenario.interventions.iter()
            .filter(|i| i.t == t)
            .flat_map(|i| i.links.iter().copied())
            .collect();

        // Candidate demands: filter the largest demands in slot t that touch or traverse the bottleneck arc
        let mut top_slot_demands: Vec<(usize, f64)> = tm.demands.iter().enumerate()
            .filter_map(|(d_idx, d)| {
                let vol = if t < d.v.len() { d.v[t] } else { 0.0 };
                if vol > 0.0 { Some((d_idx, vol)) } else { None }
            })
            .collect();
        top_slot_demands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        top_slot_demands.truncate(40);

        let mut cand_demands: Vec<(usize, f64)> = Vec::new();
        for &(d_idx, vol) in &top_slot_demands {
            let d = &tm.demands[d_idx];
            if d.s == u || d.t == v {
                cand_demands.push((d_idx, vol));
            }
        }
        if cand_demands.len() < 10 {
            for &(d_idx, vol) in &top_slot_demands {
                if !cand_demands.iter().any(|&(idx, _)| idx == d_idx) {
                    cand_demands.push((d_idx, vol));
                    if cand_demands.len() >= 15 {
                        break;
                    }
                }
            }
        }

        // Candidate waypoints:
        // - u and v themselves
        // - Out-neighbors of u
        // - In-neighbors of v
        // - 2-hop out-neighbors: out-neighbors of in-neighbors of u
        let mut cand_wps: Vec<u64> = vec![u, v];
        if let Some(outs) = out_neighbors[t].get(&u) {
            for &n in outs {
                if !cand_wps.contains(&n) {
                    cand_wps.push(n);
                }
            }
        }
        if let Some(ins) = in_neighbors[t].get(&v) {
            for &n in ins {
                if !cand_wps.contains(&n) {
                    cand_wps.push(n);
                }
            }
        }
        if let Some(ins_u) = in_neighbors[t].get(&u) {
            for &p in ins_u {
                if let Some(p_outs) = out_neighbors[t].get(&p) {
                    for &n in p_outs {
                        if !cand_wps.contains(&n) {
                            cand_wps.push(n);
                        }
                    }
                }
            }
        }

        let mut flipped = false;
        for (d_idx, _) in cand_demands {
            let demand = &tm.demands[d_idx];
            for &wp in &cand_wps {
                if wp == demand.s || wp == demand.t {
                    continue;
                }
                // Construct candidate solution
                let mut cand_srpaths = srpaths.clone();
                // Replace or add SrPath for (d_idx, t)
                if let Some(pos) = cand_srpaths.iter().position(|p| p.d == d_idx && p.t == t) {
                    cand_srpaths[pos].w = vec![wp];
                } else {
                    cand_srpaths.push(SrPath { d: d_idx, t, w: vec![wp] });
                }

                let cand_sol = Solution { srpaths: cand_srpaths.clone() };
                // Fast check slot t loads first before full 12-slot evaluation:
                if let Some(cand_slot_loads) = evaluator.compute_loads(t, &cand_sol) {
                    if cand_slot_loads.mlu < loads.mlu - 1e-6 {
                        let (cand_res, _) = evaluator.evaluate_solution_cached(&cand_sol);
                        if cand_res.valid && cand_res.obj < result.obj {
                            println!("Surgical detour success: d={} t={} wp=[{}] -> mlu={:.6} obj={:.6}", d_idx, t, wp, cand_slot_loads.mlu, cand_res.obj);
                            *srpaths = cand_srpaths;
                            *result = roadef::evaluator::EvaluationResult { valid: cand_res.valid, obj: cand_res.obj };
                            flipped = true;
                            break;
                        }
                    }
                }
            }
            if flipped {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared greedy solver (exact copy from campaign_engine.rs)
// ---------------------------------------------------------------------------
fn solve_greedy(
    net: &Network,
    demands: &[roadef::models::Demand],
    num_slots: usize,
    scenario: &roadef::models::Scenario,
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

    // -----------------------------------------------------------------------
    // Precompute per-slot physical saturation floor forced_sat[t][e]
    // -----------------------------------------------------------------------
    let mut forced_sat: Vec<std::collections::HashMap<u64, f64>> = vec![std::collections::HashMap::new(); num_slots];
    let mut active_out_arcs: Vec<std::collections::HashMap<u64, Vec<u64>>> = vec![std::collections::HashMap::new(); num_slots];
    let mut active_in_arcs: Vec<std::collections::HashMap<u64, Vec<u64>>> = vec![std::collections::HashMap::new(); num_slots];

    for t in 0..num_slots {
        let disabled_in_slot: HashSet<u64> = scenario.interventions.iter()
            .filter(|i| i.t == t)
            .flat_map(|i| i.links.iter().copied())
            .collect();

        for link in &net.links {
            if disabled_in_slot.contains(&link.id) || link.capacity <= 0.0 {
                continue;
            }
            active_out_arcs[t].entry(link.from).or_default().push(link.id);
            active_in_arcs[t].entry(link.to).or_default().push(link.id);
        }

        let mut forced_load: std::collections::HashMap<u64, f64> = std::collections::HashMap::new();
        for d in demands {
            let vol = if t < d.v.len() { d.v[t] } else { 0.0 };
            if vol <= 0.0 {
                continue;
            }
            let out_arc = active_out_arcs[t].get(&d.s).and_then(|arcs| if arcs.len() == 1 { Some(arcs[0]) } else { None });
            let in_arc = active_in_arcs[t].get(&d.t).and_then(|arcs| if arcs.len() == 1 { Some(arcs[0]) } else { None });

            match (out_arc, in_arc) {
                (Some(e_out), Some(e_in)) => {
                    if e_out == e_in {
                        *forced_load.entry(e_out).or_insert(0.0) += vol;
                    } else {
                        *forced_load.entry(e_out).or_insert(0.0) += vol;
                        *forced_load.entry(e_in).or_insert(0.0) += vol;
                    }
                }
                (Some(e_out), None) => {
                    *forced_load.entry(e_out).or_insert(0.0) += vol;
                }
                (None, Some(e_in)) => {
                    *forced_load.entry(e_in).or_insert(0.0) += vol;
                }
                (None, None) => {}
            }
        }

        for (link_id, load) in forced_load {
            if let Some(&cap) = link_capacity.get(&link_id) {
                if cap > 0.0 {
                    forced_sat[t].insert(link_id, load / cap);
                }
            }
        }
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
    let mut ignored_bottlenecks: HashSet<(usize, u64)> = HashSet::new();
    for _iter in 0..50 {
        let mut worst_sat = 0.0f64;
        let mut worst_choice: Option<(usize, u64)> = None; // (slot, link_id)

        for t in 0..num_slots {
            for (&link_id, &cap) in &link_capacity {
                if cap <= 0.0 || ignored_bottlenecks.contains(&(t, link_id)) {
                    continue;
                }
                let flow = link_flow[t].get(&link_id).copied().unwrap_or(0.0);
                let cur_sat = flow / cap;
                let floor = forced_sat[t].get(&link_id).copied().unwrap_or(0.0);

                // Skip condition in the bottleneck selector:
                // Skip (t, e) if current_sat(t, e) <= forced_sat[t][e] + 10^-7
                if cur_sat <= floor + 1e-7 {
                    continue;
                }

                if cur_sat > worst_sat {
                    worst_sat = cur_sat;
                    worst_choice = Some((t, link_id));
                }
            }
        }

        if worst_sat < 0.9999 || worst_choice.is_none() {
            break;
        }
        let (worst_t, bottle_link) = worst_choice.unwrap();

        // Find candidate demands traversing the bottleneck link
        let mut candidate_demands: Vec<usize> = Vec::new();
        for (&d_idx, fp) in &demand_paths {
            let mut traverses = false;
            for j in 0..fp.len().saturating_sub(1) {
                if let Some(&lid) = link_by_endpoints.get(&(fp[j], fp[j + 1])) {
                    if lid == bottle_link {
                        traverses = true;
                        break;
                    }
                }
            }
            if !traverses {
                continue;
            }
            let d_obj = &demands[d_idx];
            // Exclude demands that are forced on bottle_link (degree-1 leaf endpoint cannot be rerouted away)
            let is_forced_out = active_out_arcs[worst_t].get(&d_obj.s).map(|v| v.len() == 1 && v[0] == bottle_link).unwrap_or(false);
            let is_forced_in = active_in_arcs[worst_t].get(&d_obj.t).map(|v| v.len() == 1 && v[0] == bottle_link).unwrap_or(false);
            if is_forced_out || is_forced_in {
                continue;
            }
            candidate_demands.push(d_idx);
        }

        if candidate_demands.is_empty() {
            ignored_bottlenecks.insert((worst_t, bottle_link));
            continue;
        }

        // Try rerouting demands, preferring smaller demands first to avoid creating larger bottlenecks elsewhere
        candidate_demands.sort_by(|&d1, &d2| {
            let v1 = demands.get(d1).and_then(|d| d.v.get(worst_t)).copied().unwrap_or(0.0);
            let v2 = demands.get(d2).and_then(|d| d.v.get(worst_t)).copied().unwrap_or(0.0);
            v1.partial_cmp(&v2).unwrap_or(std::cmp::Ordering::Equal)
        });
        candidate_demands.truncate(20);

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
            ignored_bottlenecks.insert((worst_t, bottle_link));
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
    let mut net_path = String::new();
    let mut tm_path = String::new();
    let mut sc_path = String::new();
    let mut out_path = String::new();
    let mut _time_limit: Option<f64> = None;
    let mut _seed: Option<u64> = None;
    let mut num_threads: usize = 1;

    let mut positional = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--net" | "-n" => {
                i += 1;
                if i < args.len() { net_path = args[i].clone(); }
            }
            "--tm" | "-m" => {
                i += 1;
                if i < args.len() { tm_path = args[i].clone(); }
            }
            "--scenario" | "--sc" | "-c" => {
                i += 1;
                if i < args.len() { sc_path = args[i].clone(); }
            }
            "--output" | "--srpaths" | "-o" => {
                i += 1;
                if i < args.len() { out_path = args[i].clone(); }
            }
            "-t" | "--time" | "--time-limit" => {
                i += 1;
                if i < args.len() { _time_limit = args[i].parse().ok(); }
            }
            "-s" | "--seed" => {
                i += 1;
                if i < args.len() { _seed = args[i].parse().ok(); }
            }
            "--threads" | "-j" => {
                i += 1;
                if i < args.len() { num_threads = args[i].parse().unwrap_or(1); }
            }
            other => {
                if !other.starts_with('-') {
                    positional.push(other.to_string());
                }
            }
        }
        i += 1;
    }

    if net_path.is_empty() && !positional.is_empty() { net_path = positional[0].clone(); }
    if tm_path.is_empty() && positional.len() >= 2 { tm_path = positional[1].clone(); }
    if sc_path.is_empty() && positional.len() >= 3 { sc_path = positional[2].clone(); }
    if out_path.is_empty() && positional.len() >= 4 { out_path = positional[3].clone(); }

    if net_path.is_empty() || tm_path.is_empty() || sc_path.is_empty() || out_path.is_empty() {
        eprintln!("Usage: {} <net> <tm> <scenario> <output> [-t <time>] [-s <seed>] [--threads <threads>]", args[0]);
        eprintln!("   or: {} --net <net> --tm <tm> --scenario <scenario> --output <output>", args[0]);
        std::process::exit(1);
    }

    // Set rayon thread pool for controlled single-threaded or thread-capped execution
    let _ = rayon::ThreadPoolBuilder::new().num_threads(num_threads).build_global();

    let net = load_network(&net_path)?;
    let tm = load_traffic_matrix(&tm_path)?;
    let scenario = load_scenario(&sc_path)?;

    run_instance(&net, &tm, &scenario, &out_path)
}
