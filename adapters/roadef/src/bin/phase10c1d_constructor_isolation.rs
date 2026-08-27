/// phase10c1d_constructor_isolation.rs — P10-C1 C1-D Constructor/Repair Isolation
///
/// Governance protocol: OBSERVATIONAL — measurement-only binary.
/// No changes to production path. No algorithmic modifications.
///
/// Answers the question:
///   "Is Arc 658 overload introduced by construction, introduced by repair,
///    or merely exposed/preserved by the baseline pipeline?"
///
/// Method: 3 controlled cases for setA-13, seed=42, 50 individuals.
///
///   Case A: constructor + explicit repair + evaluate
///     factory.create() → RoadefRepair::repair() → fitness_eval.evaluate()
///     Tests whether adding explicit repair to initial construction changes
///     the violation profile. H-SKIP means repair is a no-op for Capacity
///     violations, so this should be identical to Case B for Arc 658.
///
///   Case B: constructor + evaluate only (current initial population behavior)
///     factory.create() → fitness_eval.evaluate()
///     This is what the current pipeline does for initial population.
///     C1-C data is Case B — this re-runs it for a clean isolated measurement.
///
///   Case C: reference from C1-C evidence (full pipeline with MOGA).
///     Not re-run here — wall_ms=815218, valid=1/50, maj=46 from C1-C.
///
/// Measurements per case:
///   - n_individuals: population size
///   - n_valid: individuals with is_valid()=true
///   - n_major: individuals with major violations (max_sat > 1.01)
///   - n_minor: individuals with minor violations (1.0 < max_sat <= 1.01)
///   - arc_658_overloaded: count of individuals with arc 658 max_sat > 1.0
///   - max_sat statistics
///   - repair_calls / repair_noop (Case A only)
///   - wall_ms: wall clock time for the case
///
/// Usage:
///   cargo run --release -p roadef --bin phase10c1d_constructor_isolation -- [--seed 42] [--pop 50]
///
/// Governance: C1-D is observational only. No behavioral changes.
/// C1-E and C1-F remain locked until C1-D evidence is reviewed.
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::StdRng;

use roadef::constraints::RoadefConstraintModel;
use roadef::evaluator::RoadefEvaluator;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::moga_impl::{
    ConstructionMode, RoadefFitnessEvaluator, RoadefGenomeFactory, RoadefGenome,
};
use roadef::operators::RoadefRepair;
use coralys_core::operators::{OperatorBudget, RepairOperator};
use coralys_moga::traits::{Evaluated, FitnessEvaluator, GenomeFactory};

const INSTANCE_DIR: &str = "adapters/roadef/repo/challenge-roadef-2026-main/setA";
const INSTANCE_NAME: &str = "setA-13";
const TARGET_ARC: u64 = 658;

// ---------------------------------------------------------------------------
// Compute arc saturation for a specific arc across all time slots
// ---------------------------------------------------------------------------

fn arc_max_sat(
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
// main
// ---------------------------------------------------------------------------

fn main() {
    let mut args = std::env::args().skip(1);
    let mut seed: u64 = 42;
    let mut pop_size: usize = 50;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                if let Some(v) = args.next() {
                    seed = v.parse().unwrap_or(42);
                }
            }
            "--pop" => {
                if let Some(v) = args.next() {
                    pop_size = v.parse().unwrap_or(50);
                }
            }
            _ => {}
        }
    }

    let stderr = io::stderr();
    let mut log = stderr.lock();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let _ = writeln!(log, "=== C1-D Constructor/Repair Isolation ===");
    let _ = writeln!(log, "Governance: OBSERVATIONAL — no behavioral changes");
    let _ = writeln!(log, "Instance  : {}", INSTANCE_NAME);
    let _ = writeln!(log, "Seed      : {}", seed);
    let _ = writeln!(log, "Pop size  : {}", pop_size);
    let _ = writeln!(log, "Target arc: {}", TARGET_ARC);
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Cases:");
    let _ = writeln!(log, "  A: constructor + explicit repair + evaluate");
    let _ = writeln!(log, "  B: constructor + evaluate only (current initial population behavior)");
    let _ = writeln!(log, "  C: reference — full pipeline with MOGA (from C1-C: wall_ms=815218, valid=1/50, maj=46)");
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
    let node_ids: Vec<u64> = net.nodes.iter().map(|n| n.id).collect();

    let _ = writeln!(log, "Network: {} nodes, {} links", net.nodes.len(), net.links.len());
    let _ = writeln!(log, "Demands: {}, Time slots: {}", n_demands, n_time_slots);
    let _ = writeln!(log, "");

    let evaluator = Arc::new(RoadefEvaluator::new(&net, tm, scenario));

    let fitness_eval = RoadefFitnessEvaluator {
        evaluator: Arc::clone(&evaluator),
        l2_cache: None,
    };

    let constraint_model = RoadefConstraintModel {
        evaluator: Arc::clone(&evaluator),
    };

    let repair_op = RoadefRepair;
    let budget = OperatorBudget {
        max_iterations: 10,
        max_time_ms: 100,
    };

    let factory = RoadefGenomeFactory {
        num_demands: n_demands,
        num_time_slots: n_time_slots,
        node_ids: node_ids.clone(),
        mode: ConstructionMode::Random,
        greedy_data: None,
    };

    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();

    // -----------------------------------------------------------------------
    // Case B: constructor + evaluate only (current initial population behavior)
    // Run Case B first since it's the baseline and matches C1-C behavior.
    // -----------------------------------------------------------------------
    let _ = writeln!(log, "--- Running Case B (constructor + evaluate only) ---");
    let t_b = Instant::now();
    let mut rng_b = StdRng::seed_from_u64(seed);

    let mut b_n_valid = 0usize;
    let mut b_n_major = 0usize;
    let mut b_n_minor = 0usize;
    let mut b_n_arc658 = 0usize;
    let mut b_max_sat_sum = 0.0f64;
    let mut b_max_sat_max = f64::NEG_INFINITY;
    let mut b_max_sat_min = f64::INFINITY;

    for i in 0..pop_size {
        let g = factory.create(&mut rng_b);
        let ev = fitness_eval.evaluate(&g, &metric_report);
        let arc_sat = arc_max_sat(&evaluator, &g, TARGET_ARC, n_time_slots);

        if ev.is_valid() { b_n_valid += 1; }
        if ev.max_sat > 1.01 { b_n_major += 1; }
        if ev.max_sat > 1.0 && ev.max_sat <= 1.01 { b_n_minor += 1; }
        if arc_sat > 1.0 { b_n_arc658 += 1; }
        b_max_sat_sum += ev.max_sat;
        if ev.max_sat > b_max_sat_max { b_max_sat_max = ev.max_sat; }
        if ev.max_sat < b_max_sat_min { b_max_sat_min = ev.max_sat; }

        let _ = writeln!(out,
            "[c1d] case=B member={} is_valid={} max_sat={:.9} arc_{}_sat={:.9} arc_{}_overloaded={}",
            i, ev.is_valid(), ev.max_sat, TARGET_ARC, arc_sat, TARGET_ARC, arc_sat > 1.0
        );
    }
    let wall_b = t_b.elapsed().as_millis();

    let _ = writeln!(log, "[c1d] case=B");
    let _ = writeln!(log, "[c1d]   n_individuals={}", pop_size);
    let _ = writeln!(log, "[c1d]   n_valid={} ({:.1}%)", b_n_valid, b_n_valid as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   n_major={} ({:.1}%)", b_n_major, b_n_major as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   n_minor={} ({:.1}%)", b_n_minor, b_n_minor as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   arc_{}_overloaded={} ({:.1}%)", TARGET_ARC, b_n_arc658, b_n_arc658 as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   max_sat: mean={:.6} max={:.6} min={:.6}",
        b_max_sat_sum / pop_size as f64, b_max_sat_max, b_max_sat_min);
    let _ = writeln!(log, "[c1d]   wall_ms={}", wall_b);

    // -----------------------------------------------------------------------
    // Case A: constructor + explicit repair + evaluate
    // Same seed as Case B for direct comparison.
    // -----------------------------------------------------------------------
    let _ = writeln!(log, "");
    let _ = writeln!(log, "--- Running Case A (constructor + repair + evaluate) ---");
    let t_a = Instant::now();
    let mut rng_a = StdRng::seed_from_u64(seed);

    let mut a_n_valid = 0usize;
    let mut a_n_major = 0usize;
    let mut a_n_minor = 0usize;
    let mut a_n_arc658 = 0usize;
    let mut a_max_sat_sum = 0.0f64;
    let mut a_max_sat_max = f64::NEG_INFINITY;
    let mut a_max_sat_min = f64::INFINITY;
    let mut repair_calls = 0usize;
    let mut repair_noop = 0usize;

    for i in 0..pop_size {
        let mut g = factory.create(&mut rng_a);

        // Explicit repair call (H-SKIP: no-op for Capacity violations)
        repair_calls += 1;
        match repair_op.repair(&mut g, &constraint_model, &budget) {
            Ok(true) => {}  // feasible after repair
            Ok(false) => { repair_noop += 1; } // repair made no change
            Err(_) => {}
        }

        let ev = fitness_eval.evaluate(&g, &metric_report);
        let arc_sat = arc_max_sat(&evaluator, &g, TARGET_ARC, n_time_slots);

        if ev.is_valid() { a_n_valid += 1; }
        if ev.max_sat > 1.01 { a_n_major += 1; }
        if ev.max_sat > 1.0 && ev.max_sat <= 1.01 { a_n_minor += 1; }
        if arc_sat > 1.0 { a_n_arc658 += 1; }
        a_max_sat_sum += ev.max_sat;
        if ev.max_sat > a_max_sat_max { a_max_sat_max = ev.max_sat; }
        if ev.max_sat < a_max_sat_min { a_max_sat_min = ev.max_sat; }

        let _ = writeln!(out,
            "[c1d] case=A member={} is_valid={} max_sat={:.9} arc_{}_sat={:.9} arc_{}_overloaded={}",
            i, ev.is_valid(), ev.max_sat, TARGET_ARC, arc_sat, TARGET_ARC, arc_sat > 1.0
        );
    }
    let wall_a = t_a.elapsed().as_millis();

    let _ = writeln!(log, "[c1d] case=A");
    let _ = writeln!(log, "[c1d]   n_individuals={}", pop_size);
    let _ = writeln!(log, "[c1d]   n_valid={} ({:.1}%)", a_n_valid, a_n_valid as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   n_major={} ({:.1}%)", a_n_major, a_n_major as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   n_minor={} ({:.1}%)", a_n_minor, a_n_minor as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   arc_{}_overloaded={} ({:.1}%)", TARGET_ARC, a_n_arc658, a_n_arc658 as f64 / pop_size as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   max_sat: mean={:.6} max={:.6} min={:.6}",
        a_max_sat_sum / pop_size as f64, a_max_sat_max, a_max_sat_min);
    let _ = writeln!(log, "[c1d]   repair_calls={} repair_noop={} ({:.1}%)",
        repair_calls, repair_noop, repair_noop as f64 / repair_calls as f64 * 100.0);
    let _ = writeln!(log, "[c1d]   wall_ms={}", wall_a);

    // -----------------------------------------------------------------------
    // Case C: reference from C1-C (full pipeline with MOGA)
    // -----------------------------------------------------------------------
    let _ = writeln!(log, "");
    let _ = writeln!(log, "--- Case C reference (from C1-C evidence) ---");
    let _ = writeln!(log, "[c1d] case=C (reference — not re-run)");
    let _ = writeln!(log, "[c1d]   n_individuals=50");
    let _ = writeln!(log, "[c1d]   n_valid=1 (2.0%)");
    let _ = writeln!(log, "[c1d]   n_major=46 (92.0%)");
    let _ = writeln!(log, "[c1d]   arc_{}_overloaded=47 (94.0%)", TARGET_ARC);
    let _ = writeln!(log, "[c1d]   wall_ms=815218");
    let _ = writeln!(log, "[c1d]   source=evidence/phase10_p10c1c_initial_scan_corrected_raw.txt");

    // -----------------------------------------------------------------------
    // Cross-case comparison
    // -----------------------------------------------------------------------
    let _ = writeln!(log, "");
    let _ = writeln!(log, "=== C1-D Cross-Case Comparison ===");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "| Metric               | Case A (ctor+repair+eval) | Case B (ctor+eval) | Case C (full pipeline) |");
    let _ = writeln!(log, "|----------------------|--------------------------|--------------------|-----------------------|");
    let _ = writeln!(log, "| n_valid              | {:<24} | {:<18} | {:<21} |",
        format!("{}/{} ({:.1}%)", a_n_valid, pop_size, a_n_valid as f64 / pop_size as f64 * 100.0),
        format!("{}/{} ({:.1}%)", b_n_valid, pop_size, b_n_valid as f64 / pop_size as f64 * 100.0),
        "1/50 (2.0%)");
    let _ = writeln!(log, "| n_major              | {:<24} | {:<18} | {:<21} |",
        format!("{}/{} ({:.1}%)", a_n_major, pop_size, a_n_major as f64 / pop_size as f64 * 100.0),
        format!("{}/{} ({:.1}%)", b_n_major, pop_size, b_n_major as f64 / pop_size as f64 * 100.0),
        "46/50 (92.0%)");
    let _ = writeln!(log, "| arc_658_overloaded   | {:<24} | {:<18} | {:<21} |",
        format!("{}/{} ({:.1}%)", a_n_arc658, pop_size, a_n_arc658 as f64 / pop_size as f64 * 100.0),
        format!("{}/{} ({:.1}%)", b_n_arc658, pop_size, b_n_arc658 as f64 / pop_size as f64 * 100.0),
        "47/50 (94.0%)");
    let _ = writeln!(log, "| repair_noop          | {:<24} | {:<18} | {:<21} |",
        format!("{}/{}", repair_noop, repair_calls),
        "N/A",
        "N/A");
    let _ = writeln!(log, "| wall_ms              | {:<24} | {:<18} | {:<21} |",
        wall_a, wall_b, "815218");

    let _ = writeln!(log, "");
    let _ = writeln!(log, "C1-D complete. Evidence written to stdout ([c1d] lines).");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "Interpretation:");
    let _ = writeln!(log, "  If Case A ≈ Case B: repair does not change the violation profile.");
    let _ = writeln!(log, "    → Arc 658 overload is introduced by construction, not repair.");
    let _ = writeln!(log, "  If Case A < Case B: repair reduces violations.");
    let _ = writeln!(log, "    → Repair is partially effective for Arc 658.");
    let _ = writeln!(log, "  If Case A > Case B: repair worsens violations (unexpected).");
    let _ = writeln!(log, "    → Repair has adverse interaction with Arc 658.");
    let _ = writeln!(log, "");
    let _ = writeln!(log, "  H-SKIP prediction: repair_noop ≈ 100% (all Capacity violations).");
    let _ = writeln!(log, "  If confirmed: Arc 658 overload is a pure construction artifact.");
}