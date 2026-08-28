/// phase10c2_penalty_sweep.rs — P10-C2 Saturation-Penalty Sweep Experiment
///
/// Governance protocol: EXPERIMENTAL — adapter-only intervention.
/// Coralys core is FROZEN. No changes to production defaults.
/// No changes outside adapters/roadef.
///
/// Authorization: 2026-08-27 (see docs/GERAD_PHASE10_P10C0_CHARACTERIZATION.md §H)
///
/// Research question:
///   Does increasing the saturation penalty coefficient in greedy_load_aware_dijkstra
///   reduce Arc 658 selection frequency and improve initial population feasibility?
///
/// Method:
///   Run the ROADEF greedy constructor for setA-13 with a parameterized saturation
///   penalty coefficient. The construction loop is hand-rolled (to inject the
///   parameterized penalty), then the resulting genome is evaluated via the
///   authoritative RoadefFitnessEvaluator — the same path used by C1-D.
///
///   The ECMP fallback (max_saturation_seen > 1.0 → return empty waypoints) is
///   preserved to match production behavior exactly.
///
/// Measurements per genome (CSV to stdout):
///   penalty, genome, arc658_selections, arc658_max_sat (evaluator),
///   genome_max_sat (evaluator), overloaded, ctor_ms
///
/// Experiment conditions (run separately, compare results):
///   penalty=100   ← control (matches production default)
///   penalty=200
///   penalty=400
///   penalty=800
///   penalty=1600
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c2_penalty_sweep -- \
///     --penalty 100 [--seed 42] [--genomes 50]
///
/// Causal discipline:
///   Even if penalty=400 proves dramatically better, the conclusion is:
///   "The ROADEF adapter's construction capability was improved."
///   NOT: "Coralys was modified to understand ROADEF capacity routing."
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
// ---------------------------------------------------------------------------

fn path_contains_arc(path: &[u64], arc_id: u64, evaluator: &RoadefEvaluator) -> bool {
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
// Compute Arc 658 saturation across all time slots using the authoritative
// evaluator path (same as C1-D arc_max_sat).
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
                if sat > max_sat {
                    max_sat = sat;
                }
            }
        }
    }
    max_sat
}

// ---------------------------------------------------------------------------
// Hand-rolled greedy construction with parameterized penalty.
// Replicates create_greedy() from moga_impl.rs exactly, except:
//   - penalty coefficient is a parameter (not hardcoded 100.0)
//   - returns (genome, arc658_selection_count)
//
// IMPORTANT: preserves the ECMP fallback (max_saturation_seen > 1.0 →
// return empty waypoints) to match production behavior exactly.
// ---------------------------------------------------------------------------

fn construct_genome_with_penalty(
    greedy_data: &GreedyConstructorData,
    n_demands: usize,
    n_time_slots: usize,
    penalty: f64,
    rng: &mut StdRng,
    evaluator: &RoadefEvaluator,
) -> (RoadefGenome, usize) {
    // Band-level shuffling (same as production)
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
            ordered[i..j].shuffle(rng);
            i = j;
        }
    }

    // Initialise saturation map
    let mut ecmp_saturation: HashMap<u64, f64> =
        greedy_data.link_capacity.keys().map(|&id| (id, 0.0_f64)).collect();

    let mut waypoints: Vec<Vec<u64>> = vec![vec![]; n_demands];

    // Disabled links: union across all time slots (same as production RC-001A2 fix)
    let disabled_links: HashSet<u64> = evaluator
        .scenario
        .interventions
        .iter()
        .flat_map(|iv| iv.links.iter().copied())
        .collect();

    // Worst slot: highest total demand volume (same as production RC-001A3 fix)
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

    // Disabled arcs per slot (same as production RC-001A4 fix)
    let disabled_arcs_per_slot: Vec<HashSet<u64>> = (0..n_time_slots)
        .map(|ts| {
            evaluator.scenario.interventions.iter()
                .filter(|iv| iv.t == ts)
                .flat_map(|iv| iv.links.iter().copied())
                .collect()
        })
        .collect();

    // Running arc flows per slot (same as production RC-001 scalability fix)
    let mut running_arc_flows_per_slot: Vec<HashMap<u64, f64>> = (0..n_time_slots)
        .map(|_| {
            evaluator.graph.arcs.iter().map(|a| (a.id, 0.0_f64)).collect()
        })
        .collect();

    let mut max_saturation_seen: f64 = 0.0;
    let mut arc658_selection_count: usize = 0;

    for (d_idx, src, dst, _vol) in &ordered {
        // Load-aware Dijkstra with PARAMETERIZED penalty (experiment variable)
        let load_aware_result = greedy_load_aware_dijkstra(
            &greedy_data.network,
            *src,
            *dst,
            &disabled_links,
            &ecmp_saturation,
            penalty,   // ← experiment parameter (production uses 100.0)
            0.20,
            rng,
        );

        let full_path = if load_aware_result.is_some() {
            load_aware_result
        } else {
            greedy_shortest_path(&greedy_data.network, *src, *dst, &disabled_links)
        };

        if let Some(fp) = full_path {
            // Count Arc 658 selections
            if path_contains_arc(&fp, TARGET_ARC, evaluator) {
                arc658_selection_count += 1;
            }

            let raw_wps: Vec<u64> = if fp.len() > 2 {
                fp[1..fp.len() - 1].to_vec()
            } else {
                vec![]
            };
            let wps = path_to_waypoints_rc001(&fp, greedy_data.max_segments);

            // Truncation = construction failure (same as production RC-001A5 fix)
            if raw_wps.len() > wps.len() {
                continue;
            }

            if !wps.is_empty() {
                // (no partial_srpaths needed — we build genome directly from waypoints)
            }
            waypoints[*d_idx] = wps;

            // Update running flows across all slots (same as production RC-001A4 fix)
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
                if ok {
                    for (arc_id, &flow) in &running_arc_flows_per_slot[ts] {
                        let cap = greedy_data.link_capacity.get(arc_id).copied().unwrap_or(1.0);
                        let sat = if cap > 0.0 { flow / cap } else { f64::INFINITY };
                        if sat > max_saturation_seen {
                            max_saturation_seen = sat;
                        }
                        // Update Dijkstra penalty only from worst_slot flows
                        if ts == worst_slot {
                            let entry = ecmp_saturation.entry(*arc_id).or_insert(0.0);
                            if sat > *entry {
                                *entry = sat;
                            }
                        }
                    }
                }
            }
        }
    }

    // RC-001A5 FIX: ECMP fallback when construction produces infeasible genome.
    // This matches production behavior exactly — must be preserved for comparability.
    if max_saturation_seen > 1.0 {
        let genome = RoadefGenome {
            waypoints: vec![vec![]; n_demands],
            num_time_slots: n_time_slots,
        };
        return (genome, arc658_selection_count);
    }

    let genome = RoadefGenome {
        waypoints,
        num_time_slots: n_time_slots,
    };
    (genome, arc658_selection_count)
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed: u64 = 42;
    let mut n_genomes: usize = 50;
    let mut penalty: f64 = 100.0;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                if let Some(v) = args.next() {
                    seed = v.parse().unwrap_or(42);
                }
            }
            "--genomes" => {
                if let Some(v) = args.next() {
                    n_genomes = v.parse().unwrap_or(50);
                }
            }
            "--penalty" => {
                if let Some(v) = args.next() {
                    penalty = v.parse().unwrap_or(100.0);
                }
            }
            _ => {}
        }
    }

    let stderr = io::stderr();
    let mut log = stderr.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(log, "=== P10-C2 Saturation-Penalty Sweep ===");
    let _ = writeln!(log, "Governance: EXPERIMENTAL — adapter-only, Coralys core FROZEN");
    let _ = writeln!(log, "Authorization: 2026-08-27");
    let _ = writeln!(log, "Instance  : {}", INSTANCE_NAME);
    let _ = writeln!(log, "Seed      : {}", seed);
    let _ = writeln!(log, "Genomes   : {}", n_genomes);
    let _ = writeln!(log, "Penalty   : {:.1}", penalty);
    let _ = writeln!(log, "Target arc: {}", TARGET_ARC);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Measurement path: hand-rolled construction (parameterized penalty)");
    let _ = writeln!(log, "  + RoadefFitnessEvaluator.evaluate() for authoritative overload check");
    let _ = writeln!(log, "  + arc_max_sat_evaluator() for Arc 658 per-slot saturation");
    let _ = writeln!(log, "  (same evaluator path as C1-D — ensures comparability)");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Experiment conditions (run separately):");
    let _ = writeln!(log, "  penalty=100   ← control (production default)");
    let _ = writeln!(log, "  penalty=200");
    let _ = writeln!(log, "  penalty=400");
    let _ = writeln!(log, "  penalty=800");
    let _ = writeln!(log, "  penalty=1600");
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

    let fitness_eval = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
        l2_cache: None,
    };
    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();

    // Arc 658 info
    let arc_658_cap = greedy_data.link_capacity.get(&TARGET_ARC).copied().unwrap_or(0.0);
    let arc_658_info = evaluator.graph.arcs.iter().find(|a| a.id == TARGET_ARC);
    let _ = writeln!(log, "Arc {} capacity: {:.6}", TARGET_ARC, arc_658_cap);
    if let Some(a) = arc_658_info {
        let _ = writeln!(log, "Arc {} from={} to={}", TARGET_ARC, a.from, a.to);
    }
    let _ = writeln!(log, "");

    // -----------------------------------------------------------------------
    // Aggregate accumulators
    // -----------------------------------------------------------------------
    let mut total_arc658_selections: usize = 0;
    let mut overloaded_genomes: usize = 0;
    let mut feasible_genomes: usize = 0;
    let mut arc658_max_sat_sum: f64 = 0.0;
    let mut arc658_max_sat_max: f64 = 0.0;
    let mut genome_max_sat_sum: f64 = 0.0;
    let mut genome_max_sat_max: f64 = 0.0;

    // Emit CSV header to stdout
    // Columns: penalty,genome,arc658_selections,arc658_max_sat,genome_max_sat,overloaded,ctor_ms
    let _ = writeln!(out,
        "penalty,genome,arc658_selections,arc658_max_sat,genome_max_sat,overloaded,ctor_ms"
    );

    // -----------------------------------------------------------------------
    // Run n_genomes constructions with the parameterized penalty
    // -----------------------------------------------------------------------
    let t_total = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);

    for genome_idx in 0..n_genomes {
        let t_genome = Instant::now();

        let (genome, arc658_count) = construct_genome_with_penalty(
            &greedy_data,
            n_demands,
            n_time_slots,
            penalty,
            &mut rng,
            &evaluator,
        );

        let ctor_ms = t_genome.elapsed().as_millis();

        // Authoritative evaluation (same path as C1-D)
        let ev = fitness_eval.evaluate(&genome, &metric_report);

        // Arc 658 saturation via evaluator (same as C1-D arc_max_sat)
        let arc658_sat = arc_max_sat_evaluator(&evaluator, &genome, TARGET_ARC, n_time_slots);

        let is_overloaded = !ev.is_valid();
        if is_overloaded {
            overloaded_genomes += 1;
        } else {
            feasible_genomes += 1;
        }

        total_arc658_selections += arc658_count;
        arc658_max_sat_sum += arc658_sat;
        if arc658_sat > arc658_max_sat_max {
            arc658_max_sat_max = arc658_sat;
        }
        genome_max_sat_sum += ev.max_sat;
        if ev.max_sat > genome_max_sat_max {
            genome_max_sat_max = ev.max_sat;
        }

        // Emit per-genome CSV row to stdout
        let _ = writeln!(out,
            "{:.1},{},{},{:.6},{:.6},{},{}",
            penalty, genome_idx,
            arc658_count,
            arc658_sat,
            ev.max_sat,
            is_overloaded as u8,
            ctor_ms,
        );

        let _ = writeln!(log,
            "  genome={} arc658_sel={} arc658_sat={:.4} max_sat={:.4} valid={} ctor={}ms",
            genome_idx, arc658_count,
            arc658_sat, ev.max_sat,
            ev.is_valid(), ctor_ms
        );
    }

    let wall_ms = t_total.elapsed().as_millis();

    // -----------------------------------------------------------------------
    // Summary
    // -----------------------------------------------------------------------
    let _ = writeln!(log, "");
    let _ = writeln!(log, "=== P10-C2 Summary (penalty={:.1}) ===", penalty);
    let _ = writeln!(log, "Genomes constructed : {}", n_genomes);
    let _ = writeln!(log, "Overloaded genomes  : {} / {} ({:.1}%)",
        overloaded_genomes, n_genomes,
        overloaded_genomes as f64 / n_genomes as f64 * 100.0);
    let _ = writeln!(log, "Feasible genomes    : {} / {} ({:.1}%)",
        feasible_genomes, n_genomes,
        feasible_genomes as f64 / n_genomes as f64 * 100.0);
    let _ = writeln!(log, "Arc {} total selections : {} ({:.2} per genome)",
        TARGET_ARC, total_arc658_selections,
        total_arc658_selections as f64 / n_genomes as f64);
    let _ = writeln!(log, "Arc {} mean final sat   : {:.6}",
        TARGET_ARC, arc658_max_sat_sum / n_genomes as f64);
    let _ = writeln!(log, "Arc {} max final sat    : {:.6}",
        TARGET_ARC, arc658_max_sat_max);
    let _ = writeln!(log, "Genome mean max_sat     : {:.6}",
        genome_max_sat_sum / n_genomes as f64);
    let _ = writeln!(log, "Genome max max_sat      : {:.6}", genome_max_sat_max);
    let _ = writeln!(log, "Total wall time     : {}ms ({:.1}s)",
        wall_ms, wall_ms as f64 / 1000.0);
    let _ = writeln!(log, "Mean ctor+eval/genome: {:.1}ms",
        wall_ms as f64 / n_genomes as f64);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Interpretation guide:");
    let _ = writeln!(log, "  Compare overloaded_genomes and arc658_selections across penalty conditions.");
    let _ = writeln!(log, "  Control (penalty=100): should reproduce C1-D baseline (49/50 overloaded).");
    let _ = writeln!(log, "  If penalty=X reduces arc658_selections significantly → heuristic bias reduced.");
    let _ = writeln!(log, "  If penalty=X reduces overloaded_genomes → initial feasibility improved.");
    let _ = writeln!(log, "  If penalty=X has no effect → saturation penalty is not the binding constraint.");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Gates before any production change:");
    let _ = writeln!(log, "  [GATE] 5/5 trajectory invariants bit-exact vs Phase 9 baseline (1919018aa)");
    let _ = writeln!(log, "  [GATE] T_net > 0 on setA-14 (medium) AND setA-16/setA-19 (large)");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "P10-C2 sweep complete. CSV data written to stdout.");
}