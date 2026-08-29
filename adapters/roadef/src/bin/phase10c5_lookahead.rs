/// phase10c5_lookahead.rs — P10-C5 Sequence-Aware Look-Ahead Routing
///
/// Governance protocol: EXPERIMENTAL — adapter-only intervention.
/// Coralys core is FROZEN. Saturation coefficient 100.0 UNCHANGED.
/// No changes outside adapters/roadef.
///
/// Authorization: 2026-08-29 (see docs/GERAD_PHASE10_P10C0_CHARACTERIZATION.md §H)
///
/// P10-C1→C4 finding: the per-demand capacity check (at ANY threshold) cannot
/// detect cumulative overload because the overload is NOT caused by any single
/// demand exceeding capacity — it is caused by the aggregate of many individually
/// acceptable routing decisions.
///
/// P10-C5 hypothesis (H-C5):
///   A routing decision that is locally acceptable can nevertheless be globally
///   harmful because it consumes scarce future routing capacity. Evaluating the
///   projected downstream state before committing the decision should reduce
///   cumulative concentration on Arc 658 and improve final feasibility.
///
/// Look-ahead mechanism:
///   For each candidate route being considered for a demand:
///   1. Compute normal Dijkstra metric (unchanged).
///   2. Project resulting network state (simulate adding demand's flow).
///   3. Estimate remaining-demand pressure: count remaining demands whose
///      greedy shortest path traverses Arc 658 (proxy for future concentration).
///   4. Compute look-ahead score = projected_arc658_sat × remaining_arc658_pressure.
///   5. If look-ahead score exceeds a pressure threshold, try an alternative route
///      (block Arc 658 and re-run Dijkstra). Accept the alternative if it exists.
///
/// Key distinction from C3/C4:
///   C3/C4 asked: "Does THIS demand cause overload?"
///   C5 asks: "Does THIS demand + REMAINING demands cause overload?"
///   The relevant state variable is P(future overload | S_t, R_{1:t}, D_{t:n}).
///
/// Conditions:
///   Control (--lookahead false): standard construction, must reproduce C3/C4 baseline.
///   Treatment (--lookahead true): sequence-aware look-ahead routing.
///
/// Trajectory telemetry (per demand, per genome):
///   genome, step, arc658_in_path, sat_before_arc658, cumulative_arc658,
///   remaining_demands, remaining_arc658_pressure, lookahead_score,
///   decision_changed, projected_sat_after
///
/// Summary CSV (per genome):
///   condition, genome, arc658_selections, arc658_diversions, arc658_max_sat,
///   genome_max_sat, overloaded, first_arc658_selection, ctor_ms
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c5_lookahead -- \
///     --lookahead false [--seed 42] [--genomes 50]   # control
///   cargo run --release -p roadef --bin phase10c5_lookahead -- \
///     --lookahead true  [--seed 42] [--genomes 50]   # treatment
///
/// Gates (must pass before any production change):
///   - 5/5 trajectory invariants bit-exact vs Phase 9 baseline (commit 1919018aa)
///   - T_net > 0 on setA-14 (medium) AND setA-16/setA-19 (large)
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

// Look-ahead pressure threshold: if projected_arc658_sat × remaining_pressure
// exceeds this value, try an alternative route.
// Conservative initial value — will be characterized in follow-up if C5 is positive.
const LOOKAHEAD_PRESSURE_THRESHOLD: f64 = 0.5;

// ---------------------------------------------------------------------------
// build_greedy_data — local helper
// ---------------------------------------------------------------------------

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
        network: net.clone(),
        evaluator,
        demands_by_volume,
        max_segments,
        link_capacity,
    })
}

// ---------------------------------------------------------------------------
// Check if a full node path traverses a specific arc.
// ---------------------------------------------------------------------------

fn path_contains_arc(path: &[u64], arc_id: u64, evaluator: &RoadefEvaluator) -> bool {
    let arc_opt = evaluator.graph.arcs.iter().find(|a| a.id == arc_id);
    if let Some(arc) = arc_opt {
        for window in path.windows(2) {
            if window[0] == arc.from && window[1] == arc.to {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Simulate adding demand flow and return projected saturation of target arc.
// Returns None if expand_sr_path fails.
// ---------------------------------------------------------------------------

fn projected_arc_sat(
    src: u64,
    dst: u64,
    waypoints: &[u64],
    demand_vol: f64,
    running_flows: &HashMap<u64, f64>,
    link_capacity: &HashMap<u64, f64>,
    disabled_arcs: &HashSet<u64>,
    evaluator: &RoadefEvaluator,
    arc_id: u64,
) -> Option<f64> {
    let mut sim_flows = running_flows.clone();
    let ok = expand_sr_path(
        &evaluator.graph,
        src,
        dst,
        waypoints,
        disabled_arcs,
        demand_vol,
        &mut sim_flows,
    );
    if !ok { return None; }
    let flow = sim_flows.get(&arc_id).copied().unwrap_or(0.0);
    let cap = link_capacity.get(&arc_id).copied().unwrap_or(1.0);
    Some(if cap > 0.0 { flow / cap } else { f64::INFINITY })
}

// ---------------------------------------------------------------------------
// Estimate remaining-demand pressure on target arc.
//
// For each remaining demand (not yet routed), compute its greedy shortest path
// and check if it traverses the target arc. Return the fraction of remaining
// demands that would use the target arc under greedy routing.
//
// This is a bounded heuristic — it uses greedy_shortest_path (no load-awareness)
// to avoid circular dependency on the current saturation state.
// ---------------------------------------------------------------------------

fn remaining_arc_pressure(
    remaining_demands: &[(usize, u64, u64, f64)],
    arc_id: u64,
    disabled_links: &HashSet<u64>,
    net: &Network,
    evaluator: &RoadefEvaluator,
) -> f64 {
    if remaining_demands.is_empty() { return 0.0; }
    let mut count = 0usize;
    for (_, src, dst, _) in remaining_demands {
        if let Some(path) = greedy_shortest_path(net, *src, *dst, disabled_links) {
            if path_contains_arc(&path, arc_id, evaluator) {
                count += 1;
            }
        }
    }
    count as f64 / remaining_demands.len() as f64
}

// ---------------------------------------------------------------------------
// Arc 658 saturation via authoritative evaluator path.
// ---------------------------------------------------------------------------

fn arc_max_sat_evaluator(
    evaluator: &RoadefEvaluator,
    genome: &RoadefGenome,
    arc_id: u64,
    n_time_slots: usize,
) -> f64 {
    let solution = genome.to_solution();
    let mut max_sat: f64 = 0.0;
    for t in 0..n_time_slots {
        if let Some(loads) = evaluator.compute_loads(t, &solution) {
            if let Some(&sat) = loads.arc_saturations.get(&arc_id) {
                if sat > max_sat { max_sat = sat; }
            }
        }
    }
    max_sat
}

// ---------------------------------------------------------------------------
// Per-selection trajectory record
// ---------------------------------------------------------------------------

struct SelectionRecord {
    step: usize,
    arc658_in_path: bool,
    sat_before_arc658: f64,
    cumulative_arc658: usize,
    remaining_demands: usize,
    remaining_pressure: f64,
    lookahead_score: f64,
    decision_changed: bool,
    projected_sat_after: f64,
}

// ---------------------------------------------------------------------------
// Greedy construction with optional sequence-aware look-ahead.
//
// Returns (genome, arc658_selections, arc658_diversions, first_arc658_step,
//          trajectory)
// ---------------------------------------------------------------------------

fn construct_genome(
    greedy_data: &GreedyConstructorData,
    n_demands: usize,
    n_time_slots: usize,
    use_lookahead: bool,
    rng: &mut StdRng,
    evaluator: &RoadefEvaluator,
) -> (RoadefGenome, usize, usize, Option<usize>, Vec<SelectionRecord>) {
    // Band-level shuffling (same as production)
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

    let disabled_arcs_per_slot: Vec<HashSet<u64>> = (0..n_time_slots)
        .map(|ts| {
            evaluator.scenario.interventions.iter()
                .filter(|iv| iv.t == ts)
                .flat_map(|iv| iv.links.iter().copied())
                .collect()
        })
        .collect();

    let mut running_arc_flows_per_slot: Vec<HashMap<u64, f64>> = (0..n_time_slots)
        .map(|_| evaluator.graph.arcs.iter().map(|a| (a.id, 0.0_f64)).collect())
        .collect();

    let mut max_saturation_seen: f64 = 0.0;
    let mut arc658_selection_count: usize = 0;
    let mut arc658_diversion_count: usize = 0;
    let mut first_arc658_step: Option<usize> = None;
    let mut trajectory: Vec<SelectionRecord> = Vec::new();
    let mut step: usize = 0;

    for demand_pos in 0..ordered.len() {
        let (d_idx, src, dst, _vol) = ordered[demand_pos];

        let load_aware_result = greedy_load_aware_dijkstra(
            &greedy_data.network,
            src, dst,
            &disabled_links,
            &ecmp_saturation,
            100.0,  // saturation coefficient UNCHANGED
            0.20,
            rng,
        );

        let mut full_path = if load_aware_result.is_some() {
            load_aware_result
        } else {
            greedy_shortest_path(&greedy_data.network, src, dst, &disabled_links)
        };

        let mut decision_changed = false;
        let mut lookahead_score = 0.0_f64;
        let mut remaining_pressure = 0.0_f64;
        let mut projected_sat = 0.0_f64;

        // -----------------------------------------------------------------------
        // P10-C5 INTERVENTION: sequence-aware look-ahead
        // -----------------------------------------------------------------------
        if use_lookahead {
            if let Some(ref fp) = full_path {
                // Only apply look-ahead if the candidate route uses Arc 658
                if path_contains_arc(fp, TARGET_ARC, evaluator) {
                    let demand_vol_worst = evaluator.tm.demands[d_idx]
                        .v.get(worst_slot).copied().unwrap_or(0.0);

                    if demand_vol_worst > 0.0 {
                        let fp_wps = path_to_waypoints_rc001(fp, greedy_data.max_segments);

                        // Step B: project resulting Arc 658 saturation
                        let proj_sat = projected_arc_sat(
                            src, dst,
                            &fp_wps,
                            demand_vol_worst,
                            &running_arc_flows_per_slot[worst_slot],
                            &greedy_data.link_capacity,
                            &disabled_arcs_per_slot[worst_slot],
                            evaluator,
                            TARGET_ARC,
                        ).unwrap_or(0.0);
                        projected_sat = proj_sat;

                        // Step C: estimate remaining-demand pressure on Arc 658
                        let remaining = &ordered[demand_pos + 1..];
                        remaining_pressure = remaining_arc_pressure(
                            remaining,
                            TARGET_ARC,
                            &disabled_links,
                            &greedy_data.network,
                            evaluator,
                        );

                        // Step D: look-ahead score = projected_sat × remaining_pressure
                        lookahead_score = proj_sat * remaining_pressure;

                        // Step E: if score exceeds threshold, try alternative route
                        if lookahead_score > LOOKAHEAD_PRESSURE_THRESHOLD {
                            // Block Arc 658 and re-run Dijkstra
                            let arc658_node_pair: Option<(u64, u64)> = evaluator.graph.arcs
                                .iter()
                                .find(|a| a.id == TARGET_ARC)
                                .map(|a| (a.from, a.to));

                            if let Some((arc_from, arc_to)) = arc658_node_pair {
                                // Build a disabled set that blocks Arc 658
                                // We block by adding a pseudo-arc-id to disabled_links
                                // Actually we need to block the arc by its node pair.
                                // The cleanest approach: find all arc IDs with from=arc_from, to=arc_to
                                let arc658_ids: HashSet<u64> = evaluator.graph.arcs.iter()
                                    .filter(|a| a.from == arc_from && a.to == arc_to)
                                    .map(|a| a.id)
                                    .collect();

                                let mut alt_disabled = disabled_links.clone();
                                alt_disabled.extend(arc658_ids.iter().copied());

                                let alt_result = greedy_load_aware_dijkstra(
                                    &greedy_data.network,
                                    src, dst,
                                    &alt_disabled,
                                    &ecmp_saturation,
                                    100.0,
                                    0.20,
                                    rng,
                                );
                                let alt_path = if alt_result.is_some() {
                                    alt_result
                                } else {
                                    greedy_shortest_path(&greedy_data.network, src, dst, &alt_disabled)
                                };

                                if alt_path.is_some() {
                                    // Accept alternative — this is the key trajectory change
                                    full_path = alt_path;
                                    decision_changed = true;
                                    arc658_diversion_count += 1;
                                }
                                // If no alternative exists, accept original (avoid routing failure)
                            }
                        }
                    }
                }
            }
        }

        if let Some(fp) = full_path {
            let arc658_in_path = path_contains_arc(&fp, TARGET_ARC, evaluator);
            if arc658_in_path {
                arc658_selection_count += 1;
                if first_arc658_step.is_none() {
                    first_arc658_step = Some(step);
                }
            }

            let sat_before_arc658 = ecmp_saturation.get(&TARGET_ARC).copied().unwrap_or(0.0);

            trajectory.push(SelectionRecord {
                step,
                arc658_in_path,
                sat_before_arc658,
                cumulative_arc658: arc658_selection_count,
                remaining_demands: ordered.len().saturating_sub(demand_pos + 1),
                remaining_pressure,
                lookahead_score,
                decision_changed,
                projected_sat_after: projected_sat,
            });

            let raw_wps: Vec<u64> = if fp.len() > 2 { fp[1..fp.len()-1].to_vec() } else { vec![] };
            let wps = path_to_waypoints_rc001(&fp, greedy_data.max_segments);

            if raw_wps.len() > wps.len() {
                step += 1;
                continue;
            }

            waypoints[d_idx] = wps;

            for ts in 0..n_time_slots {
                let demand_vol = evaluator.tm.demands[d_idx]
                    .v.get(ts).copied().unwrap_or(0.0);
                if demand_vol == 0.0 { continue; }
                let ok = expand_sr_path(
                    &evaluator.graph,
                    src, dst,
                    &waypoints[d_idx],
                    &disabled_arcs_per_slot[ts],
                    demand_vol,
                    &mut running_arc_flows_per_slot[ts],
                );
                if ok {
                    for (arc_id, &flow) in &running_arc_flows_per_slot[ts] {
                        let cap = greedy_data.link_capacity.get(arc_id).copied().unwrap_or(1.0);
                        let sat = if cap > 0.0 { flow / cap } else { f64::INFINITY };
                        if sat > max_saturation_seen { max_saturation_seen = sat; }
                        if ts == worst_slot {
                            let entry = ecmp_saturation.entry(*arc_id).or_insert(0.0);
                            if sat > *entry { *entry = sat; }
                        }
                    }
                }
            }
        }

        step += 1;
    }

    // ECMP fallback
    if max_saturation_seen > 1.0 {
        let genome = RoadefGenome {
            waypoints: vec![vec![]; n_demands],
            num_time_slots: n_time_slots,
        };
        return (genome, arc658_selection_count, arc658_diversion_count,
                first_arc658_step, trajectory);
    }

    let genome = RoadefGenome { waypoints, num_time_slots: n_time_slots };
    (genome, arc658_selection_count, arc658_diversion_count,
     first_arc658_step, trajectory)
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed: u64 = 42;
    let mut n_genomes: usize = 50;
    let mut use_lookahead = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => { if let Some(v) = args.next() { seed = v.parse().unwrap_or(42); } }
            "--genomes" => { if let Some(v) = args.next() { n_genomes = v.parse().unwrap_or(50); } }
            "--lookahead" => {
                if let Some(v) = args.next() {
                    use_lookahead = v == "true" || v == "1" || v == "yes";
                }
            }
            _ => {}
        }
    }

    let condition = if use_lookahead { "lookahead" } else { "control" };

    let stderr = io::stderr();
    let mut log = stderr.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(log, "=== P10-C5 Sequence-Aware Look-Ahead Routing ===");
    let _ = writeln!(log, "Governance: EXPERIMENTAL — adapter-only, Coralys core FROZEN");
    let _ = writeln!(log, "Authorization: 2026-08-29");
    let _ = writeln!(log, "Instance  : {}", INSTANCE_NAME);
    let _ = writeln!(log, "Seed      : {}", seed);
    let _ = writeln!(log, "Genomes   : {}", n_genomes);
    let _ = writeln!(log, "Condition : {}", condition);
    let _ = writeln!(log, "Look-ahead: {}", use_lookahead);
    let _ = writeln!(log, "Pressure threshold: {:.2}", LOOKAHEAD_PRESSURE_THRESHOLD);
    let _ = writeln!(log, "Saturation coefficient: 100.0 (UNCHANGED)");
    let _ = writeln!(log, "C3/C4 baseline: 37/50 feasible, arc658_sel=2009, max_sat=1.0128");
    let _ = writeln!(log, "");

    let net_path = format!("{}/{}-net.json", INSTANCE_DIR, INSTANCE_NAME);
    let tm_path = format!("{}/{}-tm.json", INSTANCE_DIR, INSTANCE_NAME);
    let scenario_path = format!("{}/{}-scenario.json", INSTANCE_DIR, INSTANCE_NAME);

    let net = load_network(&net_path).expect("Failed to load network");
    let tm = load_traffic_matrix(&tm_path).expect("Failed to load traffic matrix");
    let scenario = load_scenario(&scenario_path).expect("Failed to load scenario");

    let n_demands = tm.demands.len();
    let n_time_slots = tm.num_time_slots;

    let _ = writeln!(log, "Network: {} nodes, {} links", net.nodes.len(), net.links.len());
    let _ = writeln!(log, "Demands: {}, Time slots: {}", n_demands, n_time_slots);

    let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));
    let greedy_data = build_greedy_data(&net, evaluator.clone());

    let fitness_eval = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
        l2_cache: None,
    };
    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();

    let arc_658_cap = greedy_data.link_capacity.get(&TARGET_ARC).copied().unwrap_or(0.0);
    let _ = writeln!(log, "Arc {} capacity: {:.6}", TARGET_ARC, arc_658_cap);
    let _ = writeln!(log, "");

    // Aggregate accumulators
    let mut total_arc658_selections: usize = 0;
    let mut total_arc658_diversions: usize = 0;
    let mut overloaded_genomes: usize = 0;
    let mut feasible_genomes: usize = 0;
    let mut arc658_max_sat_sum: f64 = 0.0;
    let mut arc658_max_sat_max: f64 = 0.0;
    let mut genome_max_sat_sum: f64 = 0.0;
    let mut genome_max_sat_max: f64 = 0.0;
    let mut first_arc658_steps: Vec<usize> = Vec::new();

    // Summary CSV header
    let _ = writeln!(out,
        "condition,genome,arc658_selections,arc658_diversions,first_arc658_step,arc658_max_sat,genome_max_sat,overloaded,ctor_ms"
    );

    let t_total = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);

    for genome_idx in 0..n_genomes {
        let t_genome = Instant::now();

        let (genome, arc658_sel, arc658_div, first_arc658_step, trajectory) =
            construct_genome(
                &greedy_data,
                n_demands,
                n_time_slots,
                use_lookahead,
                &mut rng,
                &evaluator,
            );

        let ctor_ms = t_genome.elapsed().as_millis();

        let ev = fitness_eval.evaluate(&genome, &metric_report);
        let arc658_sat = arc_max_sat_evaluator(&evaluator, &genome, TARGET_ARC, n_time_slots);

        let is_overloaded = !ev.is_valid();
        if is_overloaded { overloaded_genomes += 1; } else { feasible_genomes += 1; }

        total_arc658_selections += arc658_sel;
        total_arc658_diversions += arc658_div;
        arc658_max_sat_sum += arc658_sat;
        if arc658_sat > arc658_max_sat_max { arc658_max_sat_max = arc658_sat; }
        genome_max_sat_sum += ev.max_sat;
        if ev.max_sat > genome_max_sat_max { genome_max_sat_max = ev.max_sat; }
        if let Some(s) = first_arc658_step { first_arc658_steps.push(s); }

        let first_step_str = first_arc658_step
            .map(|s| s.to_string())
            .unwrap_or_else(|| "none".to_string());

        // Summary CSV row
        let _ = writeln!(out,
            "{},{},{},{},{},{:.6},{:.6},{},{}",
            condition, genome_idx,
            arc658_sel, arc658_div, first_step_str,
            arc658_sat, ev.max_sat,
            is_overloaded as u8, ctor_ms,
        );

        // Trajectory log (stderr, genome 0 only, first 50 steps)
        if genome_idx == 0 {
            let _ = writeln!(log, "  --- Genome 0 trajectory (first 50 steps) ---");
            for rec in trajectory.iter().take(50) {
                let _ = writeln!(log,
                    "  step={:4} arc658={} sat_before={:.4} cumul={:3} rem={:4} pressure={:.3} score={:.4} changed={} proj_sat={:.4}",
                    rec.step, rec.arc658_in_path as u8,
                    rec.sat_before_arc658, rec.cumulative_arc658,
                    rec.remaining_demands, rec.remaining_pressure,
                    rec.lookahead_score, rec.decision_changed as u8,
                    rec.projected_sat_after,
                );
            }
            let _ = writeln!(log, "  --- end genome 0 trajectory ---");
        }

        let _ = writeln!(log,
            "  genome={} arc658_sel={} arc658_div={} first_step={} arc658_sat={:.4} max_sat={:.4} valid={} ctor={}ms",
            genome_idx, arc658_sel, arc658_div, first_step_str,
            arc658_sat, ev.max_sat, ev.is_valid(), ctor_ms
        );
    }

    let wall_ms = t_total.elapsed().as_millis();

    let _ = writeln!(log, "");
    let _ = writeln!(log, "=== P10-C5 Summary (condition={}) ===", condition);
    let _ = writeln!(log, "Genomes constructed       : {}", n_genomes);
    let _ = writeln!(log, "Overloaded genomes        : {} / {} ({:.1}%)",
        overloaded_genomes, n_genomes,
        overloaded_genomes as f64 / n_genomes as f64 * 100.0);
    let _ = writeln!(log, "Feasible genomes          : {} / {} ({:.1}%)",
        feasible_genomes, n_genomes,
        feasible_genomes as f64 / n_genomes as f64 * 100.0);
    let _ = writeln!(log, "Arc {} total selections   : {} ({:.2}/genome)",
        TARGET_ARC, total_arc658_selections,
        total_arc658_selections as f64 / n_genomes as f64);
    let _ = writeln!(log, "Arc {} total diversions   : {} ({:.2}/genome)",
        TARGET_ARC, total_arc658_diversions,
        total_arc658_diversions as f64 / n_genomes as f64);
    if !first_arc658_steps.is_empty() {
        let mean_first = first_arc658_steps.iter().sum::<usize>() as f64
            / first_arc658_steps.len() as f64;
        let min_first = first_arc658_steps.iter().min().copied().unwrap_or(0);
        let max_first = first_arc658_steps.iter().max().copied().unwrap_or(0);
        let _ = writeln!(log, "First Arc658 selection    : mean={:.1} min={} max={}",
            mean_first, min_first, max_first);
    }
    let _ = writeln!(log, "Arc {} mean final sat     : {:.6}",
        TARGET_ARC, arc658_max_sat_sum / n_genomes as f64);
    let _ = writeln!(log, "Arc {} max final sat      : {:.6}",
        TARGET_ARC, arc658_max_sat_max);
    let _ = writeln!(log, "Genome mean max_sat       : {:.6}",
        genome_max_sat_sum / n_genomes as f64);
    let _ = writeln!(log, "Genome max max_sat        : {:.6}", genome_max_sat_max);
    let _ = writeln!(log, "Total wall time           : {}ms ({:.1}s)",
        wall_ms, wall_ms as f64 / 1000.0);
    let _ = writeln!(log, "Mean ctor+eval/genome     : {:.1}ms",
        wall_ms as f64 / n_genomes as f64);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "C3/C4 baseline (control reference):");
    let _ = writeln!(log, "  overloaded=13/50, arc658_sel=2009, arc658_div=0, max_sat=1.0128");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Causal decision rules:");
    let _ = writeln!(log, "  arc658_sel ↓ + trajectory changes → H-C5 supported");
    let _ = writeln!(log, "  arc658_sel ↓ + feasibility unchanged → causal routing influence, not sufficient");
    let _ = writeln!(log, "  max_sat ↓ + trajectory unchanged → insufficient causal evidence");
    let _ = writeln!(log, "  arc658_sel ≈ unchanged → H-C5 rejected for this formulation");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Gates before any production change:");
    let _ = writeln!(log, "  [GATE] 5/5 trajectory invariants bit-exact vs Phase 9 (1919018aa)");
    let _ = writeln!(log, "  [GATE] T_net > 0 on setA-14 (medium) AND setA-16/setA-19 (large)");
}