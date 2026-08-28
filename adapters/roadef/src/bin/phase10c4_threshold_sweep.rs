/// phase10c4_threshold_sweep.rs — P10-C4 Threshold/Headroom Experiment
///
/// Governance protocol: EXPERIMENTAL — adapter-only intervention.
/// Coralys core is FROZEN. Saturation coefficient 100.0 UNCHANGED.
/// No changes outside adapters/roadef.
///
/// Authorization: 2026-08-28 (see docs/GERAD_PHASE10_P10C0_CHARACTERIZATION.md §H)
///
/// Research question:
///   Does lowering the capacity pre-filter threshold (from 1.0 to 0.5–0.8)
///   cause the filter to fire earlier in construction, changing the Arc 658
///   selection trajectory before cumulative overload develops?
///
/// P10-C3 finding: threshold=1.0 never fires (arc658_rej=0). Arc 658 is
/// selected at sat~0.12 — adding one demand's flow does not push any arc
/// above sat=1.0. The overload is cumulative.
///
/// P10-C4 hypothesis: a lower threshold (e.g. 0.5–0.8 of capacity) will
/// fire before cumulative overload develops, changing the routing trajectory.
///
/// Method:
///   Control (--threshold 1.0): same as P10-C3 filter=capacity (baseline).
///   Interventions: --threshold 0.8, 0.7, 0.6, 0.5
///
/// Key diagnostic metric: first divergence point from C3 trajectory.
///   If threshold=X produces same routing for first 30-35 selections and
///   only diverges near end → different conclusion than immediate divergence.
///
/// Telemetry (per genome, per selection step):
///   threshold, genome, step, arc658_in_path, sat_before_arc658,
///   cumulative_arc658_count, rejected_this_step
///
/// Summary CSV (per genome):
///   threshold, genome, arc658_selections, arc658_rejections,
///   total_rejections, first_rejection_step, arc658_max_sat,
///   genome_max_sat, overloaded, ctor_ms
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c4_threshold_sweep -- \
///     --threshold 1.0 [--seed 42] [--genomes 50]   # control
///   cargo run --release -p roadef --bin phase10c4_threshold_sweep -- \
///     --threshold 0.8 [--seed 42] [--genomes 50]   # intervention
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
const MAX_FILTER_RETRIES: usize = 3;

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
// Simulate adding demand flow and return arcs that would exceed cap_threshold.
// ---------------------------------------------------------------------------

fn would_exceed_capacity(
    src: u64,
    dst: u64,
    waypoints: &[u64],
    demand_vol: f64,
    running_flows: &HashMap<u64, f64>,
    link_capacity: &HashMap<u64, f64>,
    disabled_arcs: &HashSet<u64>,
    evaluator: &RoadefEvaluator,
    cap_threshold: f64,
) -> HashSet<u64> {
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
    if !ok {
        return HashSet::new();
    }
    sim_flows.iter()
        .filter_map(|(&arc_id, &flow)| {
            let cap = link_capacity.get(&arc_id).copied().unwrap_or(1.0);
            let sat = if cap > 0.0 { flow / cap } else { f64::INFINITY };
            if sat >= cap_threshold { Some(arc_id) } else { None }
        })
        .collect()
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
    sat_before_arc658: f64,   // current sat of arc 658 before this selection
    cumulative_arc658: usize, // arc658 selections so far (including this one if selected)
    rejected_this_step: bool,
}

// ---------------------------------------------------------------------------
// Greedy construction with configurable capacity threshold.
//
// Returns (genome, arc658_selections, arc658_rejections, total_rejections,
//          first_rejection_step, trajectory)
// ---------------------------------------------------------------------------

fn construct_genome(
    greedy_data: &GreedyConstructorData,
    n_demands: usize,
    n_time_slots: usize,
    cap_threshold: f64,
    rng: &mut StdRng,
    evaluator: &RoadefEvaluator,
) -> (RoadefGenome, usize, usize, usize, Option<usize>, Vec<SelectionRecord>) {
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
    let mut arc658_rejection_count: usize = 0;
    let mut total_rejection_count: usize = 0;
    let mut first_rejection_step: Option<usize> = None;
    let mut trajectory: Vec<SelectionRecord> = Vec::new();
    let mut step: usize = 0;

    for (d_idx, src, dst, _vol) in &ordered {
        let load_aware_result = greedy_load_aware_dijkstra(
            &greedy_data.network,
            *src, *dst,
            &disabled_links,
            &ecmp_saturation,
            100.0,  // saturation coefficient UNCHANGED
            0.20,
            rng,
        );

        let mut full_path = if load_aware_result.is_some() {
            load_aware_result
        } else {
            greedy_shortest_path(&greedy_data.network, *src, *dst, &disabled_links)
        };

        // -----------------------------------------------------------------------
        // P10-C4 INTERVENTION: capacity-aware pre-filter with configurable threshold
        // -----------------------------------------------------------------------
        let mut rejected_this_step = false;

        if let Some(ref fp) = full_path {
            let wps = path_to_waypoints_rc001(fp, greedy_data.max_segments);
            let demand_vol_worst = evaluator.tm.demands[*d_idx]
                .v.get(worst_slot).copied().unwrap_or(0.0);

            if demand_vol_worst > 0.0 {
                let mut extra_blocked: HashSet<u64> = HashSet::new();
                let mut retries = 0;
                let mut current_path = full_path.clone();

                loop {
                    if let Some(ref cp) = current_path {
                        let cp_wps = path_to_waypoints_rc001(cp, greedy_data.max_segments);
                        let overloaded_arcs = would_exceed_capacity(
                            *src, *dst,
                            &cp_wps,
                            demand_vol_worst,
                            &running_arc_flows_per_slot[worst_slot],
                            &greedy_data.link_capacity,
                            &disabled_arcs_per_slot[worst_slot],
                            evaluator,
                            cap_threshold,
                        );

                        if overloaded_arcs.is_empty() || retries >= MAX_FILTER_RETRIES {
                            full_path = current_path;
                            break;
                        }

                        // Track rejections
                        total_rejection_count += 1;
                        if path_contains_arc(cp, TARGET_ARC, evaluator) {
                            arc658_rejection_count += 1;
                            rejected_this_step = true;
                            if first_rejection_step.is_none() {
                                first_rejection_step = Some(step);
                            }
                        }

                        for arc_id in &overloaded_arcs {
                            extra_blocked.insert(*arc_id);
                        }
                        retries += 1;

                        let mut retry_disabled = disabled_links.clone();
                        retry_disabled.extend(extra_blocked.iter().copied());

                        let retry_result = greedy_load_aware_dijkstra(
                            &greedy_data.network,
                            *src, *dst,
                            &retry_disabled,
                            &ecmp_saturation,
                            100.0,
                            0.20,
                            rng,
                        );
                        current_path = if retry_result.is_some() {
                            retry_result
                        } else {
                            greedy_shortest_path(&greedy_data.network, *src, *dst, &retry_disabled)
                        };
                    } else {
                        break;
                    }
                }
            }
        }

        if let Some(fp) = full_path {
            let arc658_in_path = path_contains_arc(&fp, TARGET_ARC, evaluator);
            if arc658_in_path {
                arc658_selection_count += 1;
            }

            // Current sat of arc 658 before committing this selection
            let sat_before_arc658 = ecmp_saturation.get(&TARGET_ARC).copied().unwrap_or(0.0);

            trajectory.push(SelectionRecord {
                step,
                arc658_in_path,
                sat_before_arc658,
                cumulative_arc658: arc658_selection_count,
                rejected_this_step,
            });

            let raw_wps: Vec<u64> = if fp.len() > 2 { fp[1..fp.len()-1].to_vec() } else { vec![] };
            let wps = path_to_waypoints_rc001(&fp, greedy_data.max_segments);

            if raw_wps.len() > wps.len() {
                step += 1;
                continue;
            }

            waypoints[*d_idx] = wps;

            for ts in 0..n_time_slots {
                let demand_vol = evaluator.tm.demands[*d_idx]
                    .v.get(ts).copied().unwrap_or(0.0);
                if demand_vol == 0.0 { continue; }
                let ok = expand_sr_path(
                    &evaluator.graph,
                    *src, *dst,
                    &waypoints[*d_idx],
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
        return (genome, arc658_selection_count, arc658_rejection_count,
                total_rejection_count, first_rejection_step, trajectory);
    }

    let genome = RoadefGenome { waypoints, num_time_slots: n_time_slots };
    (genome, arc658_selection_count, arc658_rejection_count,
     total_rejection_count, first_rejection_step, trajectory)
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed: u64 = 42;
    let mut n_genomes: usize = 50;
    let mut cap_threshold: f64 = 1.0;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => { if let Some(v) = args.next() { seed = v.parse().unwrap_or(42); } }
            "--genomes" => { if let Some(v) = args.next() { n_genomes = v.parse().unwrap_or(50); } }
            "--threshold" => { if let Some(v) = args.next() { cap_threshold = v.parse().unwrap_or(1.0); } }
            _ => {}
        }
    }

    let stderr = io::stderr();
    let mut log = stderr.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(log, "=== P10-C4 Threshold/Headroom Sweep ===");
    let _ = writeln!(log, "Governance: EXPERIMENTAL — adapter-only, Coralys core FROZEN");
    let _ = writeln!(log, "Authorization: 2026-08-28");
    let _ = writeln!(log, "Instance  : {}", INSTANCE_NAME);
    let _ = writeln!(log, "Seed      : {}", seed);
    let _ = writeln!(log, "Genomes   : {}", n_genomes);
    let _ = writeln!(log, "Threshold : {:.2}", cap_threshold);
    let _ = writeln!(log, "Target arc: {}", TARGET_ARC);
    let _ = writeln!(log, "Max retries: {}", MAX_FILTER_RETRIES);
    let _ = writeln!(log, "Saturation coefficient: 100.0 (UNCHANGED)");
    let _ = writeln!(log, "C3 baseline: 37/50 feasible, arc658_sel=2009, max_sat=1.0128");
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
    let _ = writeln!(log, "Filter threshold: {:.2} × capacity = {:.6}",
        cap_threshold, cap_threshold * arc_658_cap);
    let _ = writeln!(log, "");

    // Aggregate accumulators
    let mut total_arc658_selections: usize = 0;
    let mut total_arc658_rejections: usize = 0;
    let mut total_rejections: usize = 0;
    let mut overloaded_genomes: usize = 0;
    let mut feasible_genomes: usize = 0;
    let mut arc658_max_sat_sum: f64 = 0.0;
    let mut arc658_max_sat_max: f64 = 0.0;
    let mut genome_max_sat_sum: f64 = 0.0;
    let mut genome_max_sat_max: f64 = 0.0;
    let mut first_rejection_steps: Vec<usize> = Vec::new();

    // Summary CSV header
    let _ = writeln!(out,
        "threshold,genome,arc658_selections,arc658_rejections,total_rejections,first_rejection_step,arc658_max_sat,genome_max_sat,overloaded,ctor_ms"
    );

    let t_total = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);

    for genome_idx in 0..n_genomes {
        let t_genome = Instant::now();

        let (genome, arc658_sel, arc658_rej, total_rej, first_rej_step, trajectory) =
            construct_genome(
                &greedy_data,
                n_demands,
                n_time_slots,
                cap_threshold,
                &mut rng,
                &evaluator,
            );

        let ctor_ms = t_genome.elapsed().as_millis();

        let ev = fitness_eval.evaluate(&genome, &metric_report);
        let arc658_sat = arc_max_sat_evaluator(&evaluator, &genome, TARGET_ARC, n_time_slots);

        let is_overloaded = !ev.is_valid();
        if is_overloaded { overloaded_genomes += 1; } else { feasible_genomes += 1; }

        total_arc658_selections += arc658_sel;
        total_arc658_rejections += arc658_rej;
        total_rejections += total_rej;
        arc658_max_sat_sum += arc658_sat;
        if arc658_sat > arc658_max_sat_max { arc658_max_sat_max = arc658_sat; }
        genome_max_sat_sum += ev.max_sat;
        if ev.max_sat > genome_max_sat_max { genome_max_sat_max = ev.max_sat; }
        if let Some(s) = first_rej_step { first_rejection_steps.push(s); }

        let first_rej_str = first_rej_step.map(|s| s.to_string()).unwrap_or_else(|| "none".to_string());

        // Summary CSV row
        let _ = writeln!(out,
            "{:.2},{},{},{},{},{},{:.6},{:.6},{},{}",
            cap_threshold, genome_idx,
            arc658_sel, arc658_rej, total_rej, first_rej_str,
            arc658_sat, ev.max_sat,
            is_overloaded as u8, ctor_ms,
        );

        // Trajectory log (stderr, first genome only to avoid log explosion)
        if genome_idx == 0 {
            let _ = writeln!(log, "  --- Genome 0 trajectory (first 50 steps) ---");
            for rec in trajectory.iter().take(50) {
                let _ = writeln!(log,
                    "  step={:4} arc658={} sat_before={:.4} cumul={:3} rej={}",
                    rec.step, rec.arc658_in_path as u8,
                    rec.sat_before_arc658, rec.cumulative_arc658,
                    rec.rejected_this_step as u8,
                );
            }
            let _ = writeln!(log, "  --- end genome 0 trajectory ---");
        }

        let _ = writeln!(log,
            "  genome={} arc658_sel={} arc658_rej={} total_rej={} first_rej={} arc658_sat={:.4} max_sat={:.4} valid={} ctor={}ms",
            genome_idx, arc658_sel, arc658_rej, total_rej, first_rej_str,
            arc658_sat, ev.max_sat, ev.is_valid(), ctor_ms
        );
    }

    let wall_ms = t_total.elapsed().as_millis();

    let _ = writeln!(log, "");
    let _ = writeln!(log, "=== P10-C4 Summary (threshold={:.2}) ===", cap_threshold);
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
    let _ = writeln!(log, "Arc {} total rejections   : {} ({:.2}/genome)",
        TARGET_ARC, total_arc658_rejections,
        total_arc658_rejections as f64 / n_genomes as f64);
    let _ = writeln!(log, "Total capacity rejections : {} ({:.2}/genome)",
        total_rejections, total_rejections as f64 / n_genomes as f64);
    if !first_rejection_steps.is_empty() {
        let mean_first = first_rejection_steps.iter().sum::<usize>() as f64
            / first_rejection_steps.len() as f64;
        let min_first = first_rejection_steps.iter().min().copied().unwrap_or(0);
        let max_first = first_rejection_steps.iter().max().copied().unwrap_or(0);
        let _ = writeln!(log, "First rejection step      : mean={:.1} min={} max={}",
            mean_first, min_first, max_first);
    } else {
        let _ = writeln!(log, "First rejection step      : none (filter never fired)");
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
    let _ = writeln!(log, "C3 baseline (threshold=1.0, filter=capacity):");
    let _ = writeln!(log, "  overloaded=13/50, arc658_sel=2009, arc658_rej=0, max_sat=1.0128");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Trajectory interpretation:");
    let _ = writeln!(log, "  arc658_rejections > 0 → filter is actively blocking Arc 658");
    let _ = writeln!(log, "  arc658_selections ↓ vs C3 → filter changes routing trajectory");
    let _ = writeln!(log, "  first_rejection_step early → filter fires before concentration");
    let _ = writeln!(log, "  first_rejection_step late → filter fires after concentration");
    let _ = writeln!(log, "  arc658_rejections ≈ 0 → threshold still too high");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Gates before any production change:");
    let _ = writeln!(log, "  [GATE] 5/5 trajectory invariants bit-exact vs Phase 9 (1919018aa)");
    let _ = writeln!(log, "  [GATE] T_net > 0 on setA-14 (medium) AND setA-16/setA-19 (large)");
}