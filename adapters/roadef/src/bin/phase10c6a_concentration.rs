/// phase10c6a_concentration.rs — P10-C6-A Concentration Characterization
///
/// Governance: CHARACTERIZATION ONLY. No intervention. No routing change.
/// Coralys core FROZEN. Saturation coefficient 100.0 UNCHANGED.
/// Authorization: 2026-08-29 (commit 0accb1e44).
///
/// Objective: answer exactly one question —
///   "Is there a reproducible cumulative Arc-658 concentration threshold
///    associated with irreversible overload?"
///
/// Records per genome:
///   N_658(t), N_658(t)/N_routed(t), first irrecoverable overload step
///   (proxy: step at which cumulative count reaches 50% of final count),
///   alternative routes avoiding Arc 658 at that step, final feasibility.
///
/// Summary CSV (stdout, per genome):
///   genome, arc658_sel, overloaded, first_arc658_step, last_arc658_step,
///   median_arc658_step, irrecoverable_step_proxy, alt_avoiding_658_at_irr,
///   concentration_at_irr, arc658_max_sat, genome_max_sat, ctor_ms
///
/// Trajectory CSV (stderr, genome 0 only):
///   step, n_routed, arc658_in_path, n_658_cumul, conc_share,
///   sat_before, sat_after, alt_avoiding_658
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c6a_concentration -- \
///     [--seed 42] [--genomes 50]
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

/// Count alternative routes that avoid TARGET_ARC for a given src/dst.
/// Returns 1 if an alternative exists, 0 if Arc 658 is the only option.
fn alt_routes_avoiding_target(
    src: u64, dst: u64,
    disabled_links: &HashSet<u64>,
    net: &Network,
    evaluator: &RoadefEvaluator,
) -> usize {
    let arc658_ids: HashSet<u64> = evaluator.graph.arcs.iter()
        .find(|a| a.id == TARGET_ARC)
        .map(|arc| {
            evaluator.graph.arcs.iter()
                .filter(|a| a.from == arc.from && a.to == arc.to)
                .map(|a| a.id)
                .collect()
        })
        .unwrap_or_default();
    let mut alt_disabled = disabled_links.clone();
    alt_disabled.extend(arc658_ids.iter().copied());
    if greedy_shortest_path(net, src, dst, &alt_disabled).is_some() { 1 } else { 0 }
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

struct StepRecord {
    step: usize,
    n_routed: usize,
    arc658_in_path: bool,
    n_658_cumul: usize,
    conc_share: f64,
    sat_before: f64,
    sat_after: f64,
    alt_avoiding_658: usize,
}

fn construct_genome(
    greedy_data: &GreedyConstructorData,
    n_demands: usize,
    n_time_slots: usize,
    rng: &mut StdRng,
    evaluator: &RoadefEvaluator,
) -> (RoadefGenome, Vec<StepRecord>, Vec<usize>) {
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
        .map(|ts| evaluator.scenario.interventions.iter()
            .filter(|iv| iv.t == ts)
            .flat_map(|iv| iv.links.iter().copied()).collect())
        .collect();

    let mut running_arc_flows_per_slot: Vec<HashMap<u64, f64>> = (0..n_time_slots)
        .map(|_| evaluator.graph.arcs.iter().map(|a| (a.id, 0.0_f64)).collect())
        .collect();

    let mut max_saturation_seen = 0.0_f64;
    let mut n_658_cumul: usize = 0;
    let mut n_routed: usize = 0;
    let mut trajectory: Vec<StepRecord> = Vec::new();
    let mut arc658_steps: Vec<usize> = Vec::new();
    let mut step: usize = 0;

    for (d_idx, src, dst, _vol) in &ordered {
        let load_aware = greedy_load_aware_dijkstra(
            &greedy_data.network, *src, *dst, &disabled_links,
            &ecmp_saturation, 100.0, 0.20, rng,
        );
        let full_path = if load_aware.is_some() { load_aware }
            else { greedy_shortest_path(&greedy_data.network, *src, *dst, &disabled_links) };

        if let Some(fp) = full_path {
            let arc658_in_path = path_contains_arc(&fp, TARGET_ARC, evaluator);
            let sat_before = ecmp_saturation.get(&TARGET_ARC).copied().unwrap_or(0.0);

            // Observational: count alternatives avoiding Arc 658
            let alt_avoiding = if arc658_in_path {
                alt_routes_avoiding_target(*src, *dst, &disabled_links, &greedy_data.network, evaluator)
            } else { 0 };

            if arc658_in_path {
                n_658_cumul += 1;
                arc658_steps.push(step);
            }
            n_routed += 1;
            let conc_share = n_658_cumul as f64 / n_routed as f64;

            let wps = path_to_waypoints_rc001(&fp, greedy_data.max_segments);
            let raw_wps_len = if fp.len() > 2 { fp.len() - 2 } else { 0 };

            if raw_wps_len > wps.len() {
                trajectory.push(StepRecord {
                    step, n_routed, arc658_in_path, n_658_cumul, conc_share,
                    sat_before, sat_after: sat_before, alt_avoiding_658: alt_avoiding,
                });
                step += 1;
                continue;
            }

            waypoints[*d_idx] = wps;

            for ts in 0..n_time_slots {
                let demand_vol = evaluator.tm.demands[*d_idx].v.get(ts).copied().unwrap_or(0.0);
                if demand_vol == 0.0 { continue; }
                let ok = expand_sr_path(
                    &evaluator.graph, *src, *dst, &waypoints[*d_idx],
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

            let sat_after = ecmp_saturation.get(&TARGET_ARC).copied().unwrap_or(0.0);
            trajectory.push(StepRecord {
                step, n_routed, arc658_in_path, n_658_cumul, conc_share,
                sat_before, sat_after, alt_avoiding_658: alt_avoiding,
            });
        }
        step += 1;
    }

    if max_saturation_seen > 1.0 {
        return (RoadefGenome { waypoints: vec![vec![]; n_demands], num_time_slots: n_time_slots },
                trajectory, arc658_steps);
    }
    (RoadefGenome { waypoints, num_time_slots: n_time_slots }, trajectory, arc658_steps)
}

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed: u64 = 42;
    let mut n_genomes: usize = 50;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed"    => { if let Some(v) = args.next() { seed = v.parse().unwrap_or(42); } }
            "--genomes" => { if let Some(v) = args.next() { n_genomes = v.parse().unwrap_or(50); } }
            _ => {}
        }
    }

    let stderr = io::stderr();
    let mut log = stderr.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(log, "=== P10-C6-A Concentration Characterization ===");
    let _ = writeln!(log, "Governance: CHARACTERIZATION ONLY — no intervention, Coralys core FROZEN");
    let _ = writeln!(log, "Authorization: 2026-08-29 (commit 0accb1e44)");
    let _ = writeln!(log, "Instance: {} | Seed: {} | Genomes: {}", INSTANCE_NAME, seed, n_genomes);
    let _ = writeln!(log, "Baseline: 37/50 feasible, arc658_sel=2009, max_sat=1.0128");
    let _ = writeln!(log, "Key question: is there a reproducible N_658(t) threshold");
    let _ = writeln!(log, "  associated with irreversible overload?");
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

    // Summary CSV header
    let _ = writeln!(out,
        "genome,arc658_sel,overloaded,first_arc658_step,last_arc658_step,median_arc658_step,irrecoverable_step_proxy,alt_avoiding_658_at_irr,concentration_at_irr,arc658_max_sat,genome_max_sat,ctor_ms"
    );

    // Genome 0 trajectory header to stderr
    let _ = writeln!(log, "--- genome_0_trajectory ---");
    let _ = writeln!(log, "step,n_routed,arc658_in_path,n_658_cumul,conc_share,sat_before,sat_after,alt_avoiding_658");

    // Aggregates
    let mut total_sel: usize = 0;
    let mut overloaded: usize = 0;
    let mut feasible: usize = 0;
    let mut arc658_sat_sum = 0.0_f64;
    let mut arc658_sat_max = 0.0_f64;
    let mut genome_sat_sum = 0.0_f64;
    let mut genome_sat_max = 0.0_f64;
    let mut irr_steps: Vec<usize> = Vec::new();
    let mut irr_alts: Vec<usize> = Vec::new();
    let mut irr_concs: Vec<f64> = Vec::new();

    let t_total = Instant::now();
    let mut rng = StdRng::seed_from_u64(seed);

    for genome_idx in 0..n_genomes {
        let t0 = Instant::now();
        let (genome, trajectory, arc658_steps) =
            construct_genome(&greedy_data, n_demands, n_time_slots, &mut rng, &evaluator);
        let ctor_ms = t0.elapsed().as_millis();

        let ev = fitness_eval.evaluate(&genome, &metric_report);
        let arc658_sat = arc_max_sat_evaluator(&evaluator, &genome, TARGET_ARC, n_time_slots);
        let is_overloaded = !ev.is_valid();
        if is_overloaded { overloaded += 1; } else { feasible += 1; }

        let sel = arc658_steps.len();
        total_sel += sel;
        arc658_sat_sum += arc658_sat;
        if arc658_sat > arc658_sat_max { arc658_sat_max = arc658_sat; }
        genome_sat_sum += ev.max_sat;
        if ev.max_sat > genome_sat_max { genome_sat_max = ev.max_sat; }

        let first_step = arc658_steps.first().copied();
        let last_step  = arc658_steps.last().copied();
        let median_step = if !arc658_steps.is_empty() {
            Some(arc658_steps[arc658_steps.len() / 2]) } else { None };

        // Irrecoverable step proxy: step at which n_658_cumul first reaches 50% of final count
        let halfway = ((sel as f64) * 0.5).ceil() as usize;
        let irr_step = if is_overloaded && sel > 0 {
            trajectory.iter().find(|r| r.n_658_cumul >= halfway).map(|r| r.step)
        } else { None };

        let irr_alt = irr_step.and_then(|s|
            trajectory.iter().find(|r| r.step == s).map(|r| r.alt_avoiding_658));
        let irr_conc = irr_step.and_then(|s|
            trajectory.iter().find(|r| r.step == s).map(|r| r.conc_share));

        if let Some(s) = irr_step  { irr_steps.push(s); }
        if let Some(a) = irr_alt   { irr_alts.push(a); }
        if let Some(c) = irr_conc  { irr_concs.push(c); }

        let f = |o: Option<usize>| o.map(|v| v.to_string()).unwrap_or("none".to_string());
        let fc = |o: Option<f64>| o.map(|v| format!("{:.4}", v)).unwrap_or("none".to_string());

        let _ = writeln!(out,
            "{},{},{},{},{},{},{},{},{},{:.6},{:.6},{}",
            genome_idx, sel, is_overloaded as u8,
            f(first_step), f(last_step), f(median_step),
            f(irr_step), f(irr_alt), fc(irr_conc),
            arc658_sat, ev.max_sat, ctor_ms,
        );

        // Genome 0 trajectory to stderr
        if genome_idx == 0 {
            for rec in &trajectory {
                let _ = writeln!(log,
                    "{},{},{},{},{:.4},{:.4},{:.4},{}",
                    rec.step, rec.n_routed, rec.arc658_in_path as u8,
                    rec.n_658_cumul, rec.conc_share,
                    rec.sat_before, rec.sat_after, rec.alt_avoiding_658,
                );
            }
            let _ = writeln!(log, "--- end genome_0_trajectory ---");
        }

        let _ = writeln!(log,
            "  genome={} sel={} overloaded={} first={} irr={} irr_alt={} arc658_sat={:.4} max_sat={:.4} ctor={}ms",
            genome_idx, sel, is_overloaded as u8,
            f(first_step), f(irr_step), f(irr_alt),
            arc658_sat, ev.max_sat, ctor_ms,
        );
    }

    let wall_ms = t_total.elapsed().as_millis();

    let _ = writeln!(log, "");
    let _ = writeln!(log, "=== P10-C6-A Summary ===");
    let _ = writeln!(log, "Genomes: {} | Overloaded: {}/{} ({:.1}%) | Feasible: {}/{} ({:.1}%)",
        n_genomes, overloaded, n_genomes, overloaded as f64 / n_genomes as f64 * 100.0,
        feasible, n_genomes, feasible as f64 / n_genomes as f64 * 100.0);
    let _ = writeln!(log, "Arc {} total sel: {} ({:.2}/genome)",
        TARGET_ARC, total_sel, total_sel as f64 / n_genomes as f64);
    let _ = writeln!(log, "Arc {} mean/max sat: {:.6} / {:.6}",
        TARGET_ARC, arc658_sat_sum / n_genomes as f64, arc658_sat_max);
    let _ = writeln!(log, "Genome mean/max sat: {:.6} / {:.6}",
        genome_sat_sum / n_genomes as f64, genome_sat_max);

    if !irr_steps.is_empty() {
        let n = irr_steps.len() as f64;
        let _ = writeln!(log, "Irrecoverable step proxy (overloaded genomes, n={}):", irr_steps.len());
        let _ = writeln!(log, "  mean={:.1} min={} max={}",
            irr_steps.iter().sum::<usize>() as f64 / n,
            irr_steps.iter().min().unwrap(), irr_steps.iter().max().unwrap());
        if !irr_alts.is_empty() {
            let _ = writeln!(log, "  alt_avoiding_658 at irr: mean={:.2}",
                irr_alts.iter().sum::<usize>() as f64 / irr_alts.len() as f64);
        }
        if !irr_concs.is_empty() {
            let _ = writeln!(log, "  concentration_share at irr: mean={:.4}",
                irr_concs.iter().sum::<f64>() / irr_concs.len() as f64);
        }
    } else {
        let _ = writeln!(log, "Irrecoverable step proxy: none (no overloaded genomes or no Arc 658 selections)");
    }

    let _ = writeln!(log, "Wall time: {}ms ({:.1}s)", wall_ms, wall_ms as f64 / 1000.0);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Decision gate:");
    let _ = writeln!(log, "  YES (stable threshold) → C6-B diversity cap (N_max from data only)");
    let _ = writeln!(log, "  NO  (no stable threshold) → C6-alt aggregate pressure budget");
    let _ = writeln!(log, "  Either way: STOP after one intervention. No automatic C7.");
}