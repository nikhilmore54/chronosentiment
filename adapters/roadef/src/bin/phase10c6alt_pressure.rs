/// phase10c6alt_pressure.rs — P10-C6-alt Aggregate Future Pressure Budget
///
/// Governance: FINAL AUTHORIZED INTERVENTION. Adapter-only.
/// Coralys core FROZEN. Saturation coefficient 100.0 UNCHANGED.
/// Authorization: 2026-08-30 (C6-A decision gate: NO stable threshold → C6-alt).
/// After this experiment: STOP regardless of result.
///
/// C6-A finding: no stable N_max threshold separates feasible from overloaded.
/// Feasible sel range 32-47 (mean=40.1) vs overloaded 36-43 (mean=40.4).
/// Complete overlap. C6-B (diversity cap) ruled out.
///
/// C6-alt hypothesis:
///   The cumulative overload is caused by the aggregate of many individually
///   acceptable routing decisions. A per-demand check of the form
///   flow_a(t) + F_future_a(t) > Cap_a
///   where F_future_a(t) estimates the total future routing pressure on arc a
///   from remaining demands, should detect the impending overload before it
///   becomes irreversible and divert demands to alternative routes.
///
/// Mechanism:
///   For each candidate route being considered for a demand:
///   1. Compute current flow on Arc 658: flow_a(t).
///   2. Estimate future pressure: F_future_a(t) = sum over remaining demands
///      of (demand_vol × P(demand routes through Arc 658)).
///      P is estimated by greedy_shortest_path (no load-awareness) as a proxy.
///   3. If flow_a(t) + F_future_a(t) > Cap_a × PRESSURE_FACTOR, try alternative.
///   4. Accept alternative if it exists; otherwise accept original (no routing failure).
///
/// Key distinction from C5:
///   C5: projected_sat × remaining_pressure (product of two small numbers → never fires)
///   C6-alt: flow_a(t) + F_future_a(t) > Cap_a (absolute budget, not a product)
///   The future pressure F_future_a(t) accumulates across ALL remaining demands,
///   so it can be large even when individual demand contributions are small.
///
/// Conditions:
///   Control (--pressure false): standard construction, must reproduce C6-A baseline.
///   Treatment (--pressure true): aggregate future pressure budget intervention.
///
/// Summary CSV (stdout, per genome):
///   condition, genome, arc658_sel, arc658_div, overloaded,
///   first_arc658_step, arc658_max_sat, genome_max_sat, ctor_ms
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c6alt_pressure -- \
///     --pressure false [--seed 42] [--genomes 50]   # control
///   cargo run --release -p roadef --bin phase10c6alt_pressure -- \
///     --pressure true  [--seed 42] [--genomes 50]   # treatment
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use roadef::ecmp::expand_sr_path;
use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::moga_impl::{
    GreedyConstructorData, RoadefFitnessEvaluator, RoadefGenome,
    greedy_load_aware_dijkstra, greedy_shortest_path, path_to_waypoints_rc001,
};
use roadef::models::Network;
use coralys_moga::traits::{Evaluated, FitnessEvaluator};

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const INSTANCE_NAME: &str = "setA-13";
const TARGET_ARC: u64 = 658;

// Pressure factor: reject if flow_a + F_future_a > Cap_a * PRESSURE_FACTOR.
// 1.0 = reject if aggregate (current + future) would exceed capacity exactly.
// Conservative: start at 1.0 (the natural capacity boundary).
const PRESSURE_FACTOR: f64 = 1.0;

fn build_greedy_data(net: &Network, evaluator: Arc<RoadefEvaluator>) -> Arc<GreedyConstructorData> {
    let mut demands_by_volume: Vec<(usize, u64, u64, f64)> = evaluator
        .tm.demands.iter().enumerate()
        .map(|(i, d)| {
            let max_vol = d.v.iter().cloned().fold(0.0_f64, f64::max);
            (i, d.s, d.t, max_vol)
        })
        .collect();
    demands_by_volume.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
    let link_capacity: HashMap<u64, f64> = evaluator.graph.arcs.iter()
        .map(|a| (a.id, a.capacity)).collect();
    let max_segments = evaluator.scenario.max_segments.max(0) as usize;
    Arc::new(GreedyConstructorData {
        network: net.clone(), evaluator, demands_by_volume, max_segments, link_capacity,
    })
}

fn path_contains_arc(path: &[u64], arc_id: u64, evaluator: &RoadefEvaluator) -> bool {
    if let Some(arc) = evaluator.graph.arcs.iter().find(|a| a.id == arc_id) {
        for w in path.windows(2) {
            if w[0] == arc.from && w[1] == arc.to { return true; }
        }
    }
    false
}

/// Estimate future aggregate flow on TARGET_ARC from remaining demands.
/// For each remaining demand, check if its greedy shortest path uses Arc 658.
/// If yes, add its volume to the future pressure estimate.
/// Uses greedy_shortest_path (no load-awareness) as a bounded proxy.
fn estimate_future_pressure(
    remaining: &[(usize, u64, u64, f64)],
    disabled_links: &HashSet<u64>,
    net: &Network,
    evaluator: &RoadefEvaluator,
    worst_slot_vol_fraction: f64,  // scale volumes to worst-slot equivalent
) -> f64 {
    let mut future_pressure = 0.0_f64;
    for (_, src, dst, max_vol) in remaining {
        if let Some(path) = greedy_shortest_path(net, *src, *dst, disabled_links) {
            if path_contains_arc(&path, TARGET_ARC, evaluator) {
                future_pressure += max_vol * worst_slot_vol_fraction;
            }
        }
    }
    future_pressure
}

fn arc_max_sat_evaluator(
    evaluator: &RoadefEvaluator, genome: &RoadefGenome, arc_id: u64, n_time_slots: usize,
) -> f64 {
    let solution = genome.to_solution();
    let mut max_sat = 0.0_f64;
    for t in 0..n_time_slots {
        if let Some(loads) = evaluator.compute_loads(t, &solution) {
            if let Some(&sat) = loads.arc_saturations.get(&arc_id) {
                if sat > max_sat { max_sat = sat; }
            }
        }
    }
    max_sat
}

fn construct_genome(
    greedy_data: &GreedyConstructorData,
    n_demands: usize,
    n_time_slots: usize,
    use_pressure: bool,
    rng: &mut StdRng,
    evaluator: &RoadefEvaluator,
) -> (RoadefGenome, usize, usize, Option<usize>) {
    let mut ordered = greedy_data.demands_by_volume.clone();
    if ordered.len() > 1 {
        let mut i = 0;
        while i < ordered.len() {
            let band_vol = ordered[i].3;
            let threshold = band_vol * 0.90;
            let mut j = i + 1;
            while j < ordered.len() && ordered[j].3 >= threshold { j += 1; }
            ordered[i..j].shuffle(rng);
            i = j;
        }
    }

    let mut ecmp_saturation: HashMap<u64, f64> =
        greedy_data.link_capacity.keys().map(|&id| (id, 0.0_f64)).collect();
    let mut waypoints: Vec<Vec<u64>> = vec![vec![]; n_demands];

    let disabled_links: HashSet<u64> = evaluator.scenario.interventions.iter()
        .flat_map(|iv| iv.links.iter().copied()).collect();

    let worst_slot: usize = {
        let mut best_ts = 0usize;
        let mut best_vol = 0.0f64;
        for ts in 0..n_time_slots {
            let total_vol: f64 = evaluator.tm.demands.iter()
                .map(|d| d.v.get(ts).copied().unwrap_or(0.0)).sum();
            if total_vol > best_vol { best_vol = total_vol; best_ts = ts; }
        }
        best_ts
    };

    // Compute worst-slot volume fraction for pressure estimation
    let total_max_vol: f64 = evaluator.tm.demands.iter()
        .map(|d| d.v.iter().cloned().fold(0.0_f64, f64::max)).sum();
    let total_worst_vol: f64 = evaluator.tm.demands.iter()
        .map(|d| d.v.get(worst_slot).copied().unwrap_or(0.0)).sum();
    let worst_slot_vol_fraction = if total_max_vol > 0.0 {
        total_worst_vol / total_max_vol
    } else { 1.0 };

    let disabled_arcs_per_slot: Vec<HashSet<u64>> = (0..n_time_slots)
        .map(|ts| evaluator.scenario.interventions.iter()
            .filter(|iv| iv.t == ts)
            .flat_map(|iv| iv.links.iter().copied()).collect())
        .collect();

    let mut running_arc_flows_per_slot: Vec<HashMap<u64, f64>> = (0..n_time_slots)
        .map(|_| evaluator.graph.arcs.iter().map(|a| (a.id, 0.0_f64)).collect())
        .collect();

    let arc658_cap = greedy_data.link_capacity.get(&TARGET_ARC).copied().unwrap_or(1.0);

    let mut max_saturation_seen = 0.0_f64;
    let mut arc658_sel: usize = 0;
    let mut arc658_div: usize = 0;
    let mut first_arc658_step: Option<usize> = None;
    let mut step: usize = 0;

    for demand_pos in 0..ordered.len() {
        let (d_idx, src, dst, _vol) = ordered[demand_pos];

        let load_aware = greedy_load_aware_dijkstra(
            &greedy_data.network, src, dst, &disabled_links,
            &ecmp_saturation, 100.0, 0.20, rng,
        );
        let mut full_path = if load_aware.is_some() { load_aware }
            else { greedy_shortest_path(&greedy_data.network, src, dst, &disabled_links) };

        // -----------------------------------------------------------------------
        // C6-alt INTERVENTION: aggregate future pressure budget
        // -----------------------------------------------------------------------
        if use_pressure {
            if let Some(ref fp) = full_path {
                if path_contains_arc(fp, TARGET_ARC, evaluator) {
                    // Current flow on Arc 658 at worst slot
                    let current_flow = running_arc_flows_per_slot[worst_slot]
                        .get(&TARGET_ARC).copied().unwrap_or(0.0);

                    // Estimate future pressure from remaining demands
                    let remaining = &ordered[demand_pos + 1..];
                    let future_pressure = estimate_future_pressure(
                        remaining, &disabled_links, &greedy_data.network,
                        evaluator, worst_slot_vol_fraction,
                    );

                    // Reject if current + future > Cap * PRESSURE_FACTOR
                    if current_flow + future_pressure > arc658_cap * PRESSURE_FACTOR {
                        // Try alternative route (block Arc 658)
                        let arc658_ids: HashSet<u64> = evaluator.graph.arcs.iter()
                            .find(|a| a.id == TARGET_ARC)
                            .map(|arc| evaluator.graph.arcs.iter()
                                .filter(|a| a.from == arc.from && a.to == arc.to)
                                .map(|a| a.id).collect())
                            .unwrap_or_default();
                        let mut alt_disabled = disabled_links.clone();
                        alt_disabled.extend(arc658_ids.iter().copied());

                        let alt = greedy_load_aware_dijkstra(
                            &greedy_data.network, src, dst, &alt_disabled,
                            &ecmp_saturation, 100.0, 0.20, rng,
                        ).or_else(|| greedy_shortest_path(&greedy_data.network, src, dst, &alt_disabled));

                        if alt.is_some() {
                            full_path = alt;
                            arc658_div += 1;
                        }
                        // If no alternative, accept original (avoid routing failure)
                    }
                }
            }
        }

        if let Some(fp) = full_path {
            let arc658_in_path = path_contains_arc(&fp, TARGET_ARC, evaluator);
            if arc658_in_path {
                arc658_sel += 1;
                if first_arc658_step.is_none() { first_arc658_step = Some(step); }
            }

            let wps = path_to_waypoints_rc001(&fp, greedy_data.max_segments);
            let raw_wps_len = if fp.len() > 2 { fp.len() - 2 } else { 0 };
            if raw_wps_len > wps.len() { step += 1; continue; }

            waypoints[d_idx] = wps;

            for ts in 0..n_time_slots {
                let demand_vol = evaluator.tm.demands[d_idx].v.get(ts).copied().unwrap_or(0.0);
                if demand_vol == 0.0 { continue; }
                let ok = expand_sr_path(
                    &evaluator.graph, src, dst, &waypoints[d_idx],
                    &disabled_arcs_per_slot[ts], demand_vol,
                    &mut running_arc_flows_per_slot[ts],
                );
                if ok {
                    for (arc_id, &flow) in &running_arc_flows_per_slot[ts] {
                        let cap = greedy_data.link_capacity.get(arc_id).copied().unwrap_or(1.0);
                        let sat = if cap > 0.0 { flow / cap } else { f64::INFINITY };
                        if sat > max_saturation_seen { max_saturation_seen = sat; }
                        if ts == worst_slot {
                            let e = ecmp_saturation.entry(*arc_id).or_insert(0.0);
                            if sat > *e { *e = sat; }
                        }
                    }
                }
            }
        }
        step += 1;
    }

    if max_saturation_seen > 1.0 {
        return (RoadefGenome { waypoints: vec![vec![]; n_demands], num_time_slots: n_time_slots },
                arc658_sel, arc658_div, first_arc658_step);
    }
    (RoadefGenome { waypoints, num_time_slots: n_time_slots }, arc658_sel, arc658_div, first_arc658_step)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed: u64 = 42;
    let mut n_genomes: usize = 50;
    let mut use_pressure = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed"     => { if let Some(v) = args.next() { seed = v.parse().unwrap_or(42); } }
            "--genomes"  => { if let Some(v) = args.next() { n_genomes = v.parse().unwrap_or(50); } }
            "--pressure" => {
                if let Some(v) = args.next() {
                    use_pressure = v == "true" || v == "1" || v == "yes";
                }
            }
            _ => {}
        }
    }

    let condition = if use_pressure { "pressure" } else { "control" };

    let stderr = io::stderr();
    let mut log = stderr.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(log, "=== P10-C6-alt Aggregate Future Pressure Budget ===");
    let _ = writeln!(log, "Governance: FINAL AUTHORIZED INTERVENTION — adapter-only, Coralys core FROZEN");
    let _ = writeln!(log, "Authorization: 2026-08-30 (C6-A decision gate: NO stable threshold)");
    let _ = writeln!(log, "Instance: {} | Seed: {} | Genomes: {}", INSTANCE_NAME, seed, n_genomes);
    let _ = writeln!(log, "Condition: {} | Pressure factor: {:.1}", condition, PRESSURE_FACTOR);
    let _ = writeln!(log, "Baseline: 37/50 feasible, arc658_sel=2009, max_sat=1.0128");
    let _ = writeln!(log, "Mechanism: reject Arc 658 if flow_a(t) + F_future_a(t) > Cap_a * {:.1}", PRESSURE_FACTOR);
    let _ = writeln!(log, "After this experiment: STOP regardless of result.");
    let _ = writeln!(log, "");

    let net = load_network(&format!("{}/{}-net.json", INSTANCE_DIR, INSTANCE_NAME)).unwrap();
    let tm = load_traffic_matrix(&format!("{}/{}-tm.json", INSTANCE_DIR, INSTANCE_NAME)).unwrap();
    let scenario = load_scenario(&format!("{}/{}-scenario.json", INSTANCE_DIR, INSTANCE_NAME)).unwrap();

    let n_demands = tm.demands.len();
    let n_time_slots = tm.num_time_slots;
    let _ = writeln!(log, "Network: {} nodes, {} links | Demands: {} | Slots: {}",
        net.nodes.len(), net.links.len(), n_demands, n_time_slots);

    let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
    let greedy_data = build_greedy_data(&net, evaluator.clone());
    let fitness_eval = RoadefFitnessEvaluator { evaluator: Arc::clone(&evaluator), l2_cache: None };
    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();

    let _ = writeln!(out,
        "condition,genome,arc658_sel,arc658_div,overloaded,first_arc658_step,arc658_max_sat,genome_max_sat,ctor_ms"
    );

    let mut total_sel: usize = 0;
    let mut total_div: usize = 0;
    let mut overloaded: usize = 0;
    let mut feasible: usize = 0;
    let mut arc658_sat_sum = 0.0_f64;
    let mut arc658_sat_max = 0.0_f64;
    let mut genome_sat_sum = 0.0_f64;
    let mut genome_sat_max = 0.0_f64;

    let t_total = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);

    for genome_idx in 0..n_genomes {
        let t0 = Instant::now();
        let (genome, sel, div, first_step) = construct_genome(
            &greedy_data, n_demands, n_time_slots, use_pressure, &mut rng, &evaluator,
        );
        let ctor_ms = t0.elapsed().as_millis();

        let ev = fitness_eval.evaluate(&genome, &metric_report);
        let arc658_sat = arc_max_sat_evaluator(&evaluator, &genome, TARGET_ARC, n_time_slots);
        let is_overloaded = !ev.is_valid();
        if is_overloaded { overloaded += 1; } else { feasible += 1; }

        total_sel += sel;
        total_div += div;
        arc658_sat_sum += arc658_sat;
        if arc658_sat > arc658_sat_max { arc658_sat_max = arc658_sat; }
        genome_sat_sum += ev.max_sat;
        if ev.max_sat > genome_sat_max { genome_sat_max = ev.max_sat; }

        let f = |o: Option<usize>| o.map(|v| v.to_string()).unwrap_or("none".to_string());

        let _ = writeln!(out,
            "{},{},{},{},{},{},{:.6},{:.6},{}",
            condition, genome_idx, sel, div, is_overloaded as u8,
            f(first_step), arc658_sat, ev.max_sat, ctor_ms,
        );

        let _ = writeln!(log,
            "  genome={} sel={} div={} overloaded={} first={} arc658_sat={:.4} max_sat={:.4} ctor={}ms",
            genome_idx, sel, div, is_overloaded as u8, f(first_step),
            arc658_sat, ev.max_sat, ctor_ms,
        );
    }

    let wall_ms = t_total.elapsed().as_millis();

    let _ = writeln!(log, "");
    let _ = writeln!(log, "=== P10-C6-alt Summary (condition={}) ===", condition);
    let _ = writeln!(log, "Genomes: {} | Overloaded: {}/{} ({:.1}%) | Feasible: {}/{} ({:.1}%)",
        n_genomes, overloaded, n_genomes, overloaded as f64 / n_genomes as f64 * 100.0,
        feasible, n_genomes, feasible as f64 / n_genomes as f64 * 100.0);
    let _ = writeln!(log, "Arc {} total sel: {} ({:.2}/genome) | diversions: {} ({:.2}/genome)",
        TARGET_ARC, total_sel, total_sel as f64 / n_genomes as f64,
        total_div, total_div as f64 / n_genomes as f64);
    let _ = writeln!(log, "Arc {} mean/max sat: {:.6} / {:.6}",
        TARGET_ARC, arc658_sat_sum / n_genomes as f64, arc658_sat_max);
    let _ = writeln!(log, "Genome mean/max sat: {:.6} / {:.6}",
        genome_sat_sum / n_genomes as f64, genome_sat_max);
    let _ = writeln!(log, "Wall time: {}ms ({:.1}s)", wall_ms, wall_ms as f64 / 1000.0);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Baseline (C6-A control): overloaded=13/50, arc658_sel=2009, arc658_div=0, max_sat=1.0128");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Causal decision rules:");
    let _ = writeln!(log, "  arc658_sel ↓ + feasibility ↑ → H-C6-alt supported");
    let _ = writeln!(log, "  arc658_div > 0 + feasibility unchanged → causal routing influence, not sufficient");
    let _ = writeln!(log, "  arc658_div = 0 → pressure budget never fires (same root cause as C3/C4/C5)");
    let _ = writeln!(log, "  feasibility unchanged → H-C6-alt rejected");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "STOP: this is the final authorized intervention. No C7 authorized.");
}