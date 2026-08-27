/// phase10c1e_constructor_trace.rs — P10-C1 C1-E Constructor Mechanism Investigation
///
/// Governance protocol: OBSERVATIONAL — measurement-only binary.
/// No changes to production path. No algorithmic modifications.
///
/// Answers the question:
///   "What decision in the constructor causes Arc 658 to become the
///    preferred/forced route, and at what point does that decision make
///    the eventual overload inevitable?"
///
/// Method: Instrument the greedy constructor's allocation loop for setA-13.
/// For each demand allocation, record:
///   - demand index and allocation step (position in ordered sequence)
///   - demand volume at worst_slot
///   - whether the selected path traverses Arc 658
///   - Arc 658 saturation BEFORE this demand is allocated (ecmp_saturation[658])
///   - Arc 658 saturation AFTER this demand is allocated
///   - whether load-aware Dijkstra or fallback shortest-path was used
///
/// Hypothesis discrimination:
///   H1 — Allocation ordering: Arc 658 fills early in the demand sequence
///         → Evidence: arc_658_in_path=true concentrated at low allocation steps
///   H2 — Capacity structure: Arc 658 sat_before is already high when selected
///         → Evidence: sat_before > threshold when arc_658_in_path=true
///   H3 — Heuristic bias: Arc 658 is selected even when sat_before is low
///         → Evidence: arc_658_in_path=true with sat_before ≈ 0.0
///   H4 — Network topology: Arc 658 appears in path for most demands regardless
///         → Evidence: arc_658_in_path=true for a large fraction of all demands
///
/// Critical guardrail: instrument ALL allocations (not only overloaded ones).
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c1e_constructor_trace -- [--seed 42] [--genomes 10]
///
/// Governance: C1-E is observational only. No behavioral changes.
/// C1-F remains locked until C1-E evidence is reviewed.
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
    GreedyConstructorData, greedy_load_aware_dijkstra, greedy_shortest_path,
    path_to_waypoints_rc001,
};
use roadef::models::Network;

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const INSTANCE_NAME: &str = "setA-13";
const TARGET_ARC: u64 = 658;

// ---------------------------------------------------------------------------
// build_greedy_data — same pattern as other binaries (local helper)
// ---------------------------------------------------------------------------

fn build_greedy_data(net: &Network, evaluator: Arc<RoadefEvaluator>) -> Arc<GreedyConstructorData> {
    let mut demands_by_volume: Vec<(usize, u64, u64, f64)> = evaluator
        .tm
        .demands
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let max_vol = d.v.iter().cloned().fold(0.0_f64, f64::max);
            (i, d.s, d.t, max_vol)
        })
        .collect();
    demands_by_volume.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let link_capacity: HashMap<u64, f64> = evaluator
        .graph
        .arcs
        .iter()
        .map(|a| (a.id, a.capacity))
        .collect();

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
// Check if a path (sequence of node IDs) traverses a specific arc.
// The path is a sequence of node IDs; Arc 658 connects two specific nodes.
// We look up Arc 658's src/dst in the graph and check if that edge appears.
// ---------------------------------------------------------------------------

fn path_contains_arc(path: &[u64], arc_id: u64, evaluator: &RoadefEvaluator) -> bool {
    // Find the arc's from/to nodes
    let arc_opt = evaluator.graph.arcs.iter().find(|a| a.id == arc_id);
    if let Some(arc) = arc_opt {
        let arc_from = arc.from;
        let arc_to = arc.to;
        for window in path.windows(2) {
            if window[0] == arc_from && window[1] == arc_to {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed: u64 = 42;
    let mut n_genomes: usize = 10;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                if let Some(v) = args.next() {
                    seed = v.parse().unwrap_or(42);
                }
            }
            "--genomes" => {
                if let Some(v) = args.next() {
                    n_genomes = v.parse().unwrap_or(10);
                }
            }
            _ => {}
        }
    }

    let stderr = io::stderr();
    let mut log = stderr.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(log, "=== C1-E Constructor Mechanism Investigation ===");
    let _ = writeln!(log, "Governance: OBSERVATIONAL — no behavioral changes");
    let _ = writeln!(log, "Instance  : {}", INSTANCE_NAME);
    let _ = writeln!(log, "Seed      : {}", seed);
    let _ = writeln!(log, "Genomes   : {}", n_genomes);
    let _ = writeln!(log, "Target arc: {}", TARGET_ARC);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Hypotheses:");
    let _ = writeln!(log, "  H1 — Allocation ordering: Arc 658 fills early in demand sequence");
    let _ = writeln!(log, "  H2 — Capacity structure: Arc 658 sat_before is high when selected");
    let _ = writeln!(log, "  H3 — Heuristic bias: Arc 658 selected even when sat_before is low");
    let _ = writeln!(log, "  H4 — Network topology: Arc 658 in path for most demands regardless");
    let _ = writeln!(log, "");

    // Load instance
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

    // Arc 658 capacity and src/dst
    let arc_658_cap = greedy_data.link_capacity.get(&TARGET_ARC).copied().unwrap_or(0.0);
    let arc_658_info = evaluator.graph.arcs.iter().find(|a| a.id == TARGET_ARC);
    let _ = writeln!(log, "Arc {} capacity: {:.6}", TARGET_ARC, arc_658_cap);
    if let Some(a) = arc_658_info {
        let _ = writeln!(log, "Arc {} from={} to={}", TARGET_ARC, a.from, a.to);
    }
    let _ = writeln!(log, "");

    // Precompute disabled links (union across all time slots)
    let disabled_links: HashSet<u64> = evaluator
        .scenario
        .interventions
        .iter()
        .flat_map(|iv| iv.links.iter().copied())
        .collect();

    // Precompute worst_slot
    let worst_slot: usize = {
        let mut best_ts = 0usize;
        let mut best_vol = 0.0f64;
        for ts in 0..n_time_slots {
            let total_vol: f64 = evaluator.tm.demands.iter()
                .map(|d| d.v.get(ts).copied().unwrap_or(0.0))
                .sum();
            if total_vol > best_vol {
                best_vol = total_vol;
                best_ts = ts;
            }
        }
        best_ts
    };
    let _ = writeln!(log, "Worst slot: {}", worst_slot);

    // Precompute disabled arcs per slot
    let disabled_arcs_per_slot: Vec<HashSet<u64>> = (0..n_time_slots)
        .map(|ts| {
            evaluator.scenario.interventions.iter()
                .filter(|iv| iv.t == ts)
                .flat_map(|iv| iv.links.iter().copied())
                .collect()
        })
        .collect();

    // -----------------------------------------------------------------------
    // Per-genome summary accumulators
    // -----------------------------------------------------------------------
    let mut total_allocs = 0usize;
    let mut total_arc658_allocs = 0usize;
    let mut total_arc658_load_aware = 0usize;
    let mut total_arc658_fallback = 0usize;
    let mut arc658_step_sum = 0usize;
    let mut arc658_sat_before_sum = 0.0f64;
    let mut arc658_sat_before_max = 0.0f64;
    let mut arc658_vol_sum = 0.0f64;

    // -----------------------------------------------------------------------
    // Run n_genomes instrumented constructions
    // -----------------------------------------------------------------------
    let t_total = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);

    for genome_idx in 0..n_genomes {
        let _ = writeln!(log, "--- Genome {} ---", genome_idx);

        // Build demand order with band-level shuffling (same as production)
        let mut ordered = greedy_data.demands_by_volume.clone();
        if ordered.len() > 1 {
            let mut i = 0;
            while i < ordered.len() {
                let band_vol = ordered[i].3;
                let threshold = band_vol * 0.90;
                let mut j = i + 1;
                while j < ordered.len() && ordered[j].3 >= threshold {
                    j += 1;
                }
                ordered[i..j].shuffle(&mut rng);
                i = j;
            }
        }

        // Initialise saturation map
        let mut ecmp_saturation: HashMap<u64, f64> =
            greedy_data.link_capacity.keys().map(|&id| (id, 0.0_f64)).collect();

        let mut waypoints: Vec<Vec<u64>> = vec![vec![]; n_demands];
        let mut running_arc_flows_per_slot: Vec<HashMap<u64, f64>> = (0..n_time_slots)
            .map(|_| {
                evaluator.graph.arcs.iter().map(|a| (a.id, 0.0_f64)).collect()
            })
            .collect();

        // Per-genome counters
        let mut genome_arc658_count = 0usize;
        let mut genome_total_allocs = 0usize;
        let mut genome_arc658_first_step: Option<usize> = None;
        let mut genome_arc658_sat_before_at_first: f64 = 0.0;

        for (step, (d_idx, src, dst, _vol)) in ordered.iter().enumerate() {
            let demand_vol_worst = evaluator.tm.demands[*d_idx]
                .v.get(worst_slot).copied().unwrap_or(0.0);

            // Capture Arc 658 saturation BEFORE this allocation
            let sat_before = ecmp_saturation.get(&TARGET_ARC).copied().unwrap_or(0.0);

            // Load-aware Dijkstra (same as production)
            let load_aware_result = greedy_load_aware_dijkstra(
                &greedy_data.network,
                *src,
                *dst,
                &disabled_links,
                &ecmp_saturation,
                100.0,
                0.20,
                &mut rng,
            );

            let (full_path, used_fallback) = if load_aware_result.is_some() {
                (load_aware_result, false)
            } else {
                let fallback = greedy_shortest_path(&greedy_data.network, *src, *dst, &disabled_links);
                (fallback, true)
            };

            genome_total_allocs += 1;

            if let Some(fp) = full_path {
                // Check if Arc 658 is in the selected path
                let arc_658_in_path = path_contains_arc(&fp, TARGET_ARC, &evaluator);

                let wps = path_to_waypoints_rc001(&fp, greedy_data.max_segments);
                let truncated = fp.len() > 2 && wps.len() < fp.len() - 2;

                if !truncated && !wps.is_empty() {
                    waypoints[*d_idx] = wps.clone();

                    // Update running flows (same as production)
                    for ts in 0..n_time_slots {
                        let demand_vol = evaluator.tm.demands[*d_idx]
                            .v.get(ts).copied().unwrap_or(0.0);
                        if demand_vol == 0.0 { continue; }
                        let ok = expand_sr_path(
                            &evaluator.graph,
                            *src,
                            *dst,
                            &waypoints[*d_idx],
                            &disabled_arcs_per_slot[ts],
                            demand_vol,
                            &mut running_arc_flows_per_slot[ts],
                        );
                        if ok && ts == worst_slot {
                            for (arc_id, &flow) in &running_arc_flows_per_slot[ts] {
                                let cap = greedy_data.link_capacity.get(arc_id).copied().unwrap_or(1.0);
                                let sat = if cap > 0.0 { flow / cap } else { f64::INFINITY };
                                let entry = ecmp_saturation.entry(*arc_id).or_insert(0.0);
                                *entry = sat;
                            }
                        }
                    }
                }

                // Capture Arc 658 saturation AFTER this allocation
                let sat_after = ecmp_saturation.get(&TARGET_ARC).copied().unwrap_or(0.0);

                // Emit per-allocation trace line
                let _ = writeln!(out,
                    "[c1e] genome={} step={} demand={} vol={:.4} arc_{}_in_path={} sat_before={:.6} sat_after={:.6} fallback={} truncated={}",
                    genome_idx, step, d_idx, demand_vol_worst,
                    TARGET_ARC, arc_658_in_path,
                    sat_before, sat_after,
                    used_fallback, truncated
                );

                if arc_658_in_path {
                    genome_arc658_count += 1;
                    if genome_arc658_first_step.is_none() {
                        genome_arc658_first_step = Some(step);
                        genome_arc658_sat_before_at_first = sat_before;
                    }
                    if used_fallback {
                        total_arc658_fallback += 1;
                    } else {
                        total_arc658_load_aware += 1;
                    }
                    arc658_step_sum += step;
                    arc658_sat_before_sum += sat_before;
                    if sat_before > arc658_sat_before_max {
                        arc658_sat_before_max = sat_before;
                    }
                    arc658_vol_sum += demand_vol_worst;
                }
            } else {
                // No path found — emit routing failure
                let _ = writeln!(out,
                    "[c1e] genome={} step={} demand={} vol={:.4} arc_{}_in_path=false sat_before={:.6} sat_after={:.6} fallback=true truncated=false routing_failure=true",
                    genome_idx, step, d_idx, demand_vol_worst,
                    TARGET_ARC, sat_before, sat_before
                );
            }
        }

        total_allocs += genome_total_allocs;
        total_arc658_allocs += genome_arc658_count;

        let _ = writeln!(log, "  genome={} total_allocs={} arc_{}_allocs={} ({:.1}%) first_step={:?} sat_before_at_first={:.6}",
            genome_idx, genome_total_allocs, TARGET_ARC,
            genome_arc658_count,
            genome_arc658_count as f64 / genome_total_allocs as f64 * 100.0,
            genome_arc658_first_step,
            genome_arc658_sat_before_at_first
        );
    }

    let wall_ms = t_total.elapsed().as_millis();

    // -----------------------------------------------------------------------
    // Summary
    // -----------------------------------------------------------------------
    let _ = writeln!(log, "");
    let _ = writeln!(log, "=== C1-E Summary ===");
    let _ = writeln!(log, "Genomes traced: {}", n_genomes);
    let _ = writeln!(log, "Total allocations: {}", total_allocs);
    let _ = writeln!(log, "Arc {} allocations: {} ({:.1}%)",
        TARGET_ARC, total_arc658_allocs,
        total_arc658_allocs as f64 / total_allocs as f64 * 100.0);
    if total_arc658_allocs > 0 {
        let _ = writeln!(log, "Arc {} mean allocation step: {:.1}",
            TARGET_ARC, arc658_step_sum as f64 / total_arc658_allocs as f64);
        let _ = writeln!(log, "Arc {} mean sat_before: {:.6}",
            TARGET_ARC, arc658_sat_before_sum / total_arc658_allocs as f64);
        let _ = writeln!(log, "Arc {} max sat_before: {:.6}", TARGET_ARC, arc658_sat_before_max);
        let _ = writeln!(log, "Arc {} mean demand vol: {:.4}",
            TARGET_ARC, arc658_vol_sum / total_arc658_allocs as f64);
        let _ = writeln!(log, "Arc {} load_aware selections: {} ({:.1}%)",
            TARGET_ARC, total_arc658_load_aware,
            total_arc658_load_aware as f64 / total_arc658_allocs as f64 * 100.0);
        let _ = writeln!(log, "Arc {} fallback selections: {} ({:.1}%)",
            TARGET_ARC, total_arc658_fallback,
            total_arc658_fallback as f64 / total_arc658_allocs as f64 * 100.0);
    }
    let _ = writeln!(log, "Wall time: {}ms", wall_ms);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Hypothesis indicators:");
    if total_arc658_allocs > 0 {
        let mean_step = arc658_step_sum as f64 / total_arc658_allocs as f64;
        let mean_sat_before = arc658_sat_before_sum / total_arc658_allocs as f64;
        let arc658_frac = total_arc658_allocs as f64 / total_allocs as f64;
        let _ = writeln!(log, "  H1 (ordering): mean Arc {} step = {:.1} / {} total demands",
            TARGET_ARC, mean_step, n_demands);
        let _ = writeln!(log, "    → If mean_step < {}, Arc {} fills early (H1 supported)",
            n_demands / 4, TARGET_ARC);
        let _ = writeln!(log, "  H2 (capacity): mean sat_before = {:.6}, max = {:.6}",
            mean_sat_before, arc658_sat_before_max);
        let _ = writeln!(log, "    → If sat_before > 0.5 when selected, capacity structure drives overload (H2 supported)");
        let _ = writeln!(log, "  H3 (heuristic): load_aware={} ({:.1}%), fallback={} ({:.1}%)",
            total_arc658_load_aware,
            total_arc658_load_aware as f64 / total_arc658_allocs as f64 * 100.0,
            total_arc658_fallback,
            total_arc658_fallback as f64 / total_arc658_allocs as f64 * 100.0);
        let _ = writeln!(log, "    → If load_aware dominates with low sat_before, heuristic prefers Arc {} (H3 supported)", TARGET_ARC);
        let _ = writeln!(log, "  H4 (topology): Arc {} in {:.1}% of all allocations",
            TARGET_ARC, arc658_frac * 100.0);
        let _ = writeln!(log, "    → If fraction > 50%, Arc {} is structurally dominant (H4 supported)", TARGET_ARC);
    }
    let _ = writeln!(log, "");
    let _ = writeln!(log, "C1-E complete. Evidence written to stdout ([c1e] lines).");
}