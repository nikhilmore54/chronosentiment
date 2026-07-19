//! M6.4 -- Scalability Validation
//!
//! # Hypothesis
//!
//! LocalSearch demonstrates bounded execution time and consistently improves
//! objective quality across representative scheduling scales while preserving
//! solution validity.
//!
//! # Fixture sizes (qualification fixtures, not production-scale benchmarks)
//!
//! | Label  | Rotations | Pairings | Max iterations |
//! |--------|-----------|----------|----------------|
//! | Small  |         4 |       12 |            200 |
//! | Medium |         8 |       24 |            500 |
//! | Large  |        16 |       48 |           1000 |
//!
//! # CI gates (environment-independent correctness invariants)
//!
//! - `optimized_score < baseline_score` at each scale
//! - pairings conserved at each scale
//! - optimized solution is legal at each scale
//!
//! # Evidence (recorded, not gated)
//!
//! - runtime_ms
//! - evaluations
//! - improvements
//! - acceptance_ratio (improvements / evaluations)
//! - improvement_pct
//!
//! Runtime is recorded as evidence for M6.7 qualification reporting.
//! It is NOT a pass/fail gate because it is hardware- and environment-dependent.
//!
//! # Fixture design rules (from M6.3)
//!
//! 1. Heavy rotation holds LATEST pairings; light rotations hold EARLIEST.
//!    Ensures relocate moves satisfy `Rotation` chronological ordering.
//! 2. All rotations have >= 2 pairings (HashMap iteration order is non-deterministic).

#![allow(non_snake_case)]

use coralys_scheduling::domain::crew::CrewId;
use coralys_scheduling::domain::duty::{Duty, DutyId};
use coralys_scheduling::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_scheduling::domain::pairing::{Pairing, PairingId};
use coralys_scheduling::domain::roster::{PlanningPeriod, Roster, RosterId};
use coralys_scheduling::domain::rotation::{Rotation, RotationId};
use coralys_scheduling::legality::LegalityChecker;
use coralys_scheduling::optimization::cost::CostEvaluator;
use coralys_scheduling::optimization::metrics::OptimizationMetrics;
use coralys_scheduling::optimization::objective::{
    SchedulingObjective, WorkloadBalanceObjective,
};
use coralys_scheduling::optimization::search::local_search::LocalSearch;

use chrono::{Duration, TimeZone, Utc};
use std::time::Instant;

// ── Fixture helpers ───────────────────────────────────────────────────────────

fn base_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap()
}

fn make_leg(id: &str, origin: &str, dest: &str, dep_h: i64, arr_h: i64) -> FlightLeg {
    FlightLeg::new(
        FlightLegId::new(id),
        FlightNumber::new(format!("FL{id}")),
        AirportCode::new(origin),
        AirportCode::new(dest),
        base_time() + Duration::hours(dep_h),
        base_time() + Duration::hours(arr_h),
        AircraftType::new("B737"),
    )
}

fn make_pairing(id: &str, dep_h: i64) -> (Vec<FlightLeg>, Pairing) {
    let out = make_leg(&format!("{id}_o"), "LHR", "CDG", dep_h, dep_h + 2);
    let ret = make_leg(&format!("{id}_r"), "CDG", "LHR", dep_h + 3, dep_h + 5);
    let duty = Duty::new(
        DutyId::new(&format!("{id}_D")),
        vec![out.clone(), ret.clone()],
    )
    .unwrap();
    let pairing = Pairing::new(
        PairingId::new(id),
        AirportCode::new("LHR"),
        vec![duty],
    )
    .unwrap();
    (vec![out, ret], pairing)
}

fn make_rotation(rot_id: &str, crew_id: &str, pairings: Vec<Pairing>) -> Rotation {
    Rotation::new(RotationId::new(rot_id), CrewId::new(crew_id), pairings).unwrap()
}

fn make_roster(legs: Vec<FlightLeg>, rotations: Vec<Rotation>) -> Roster {
    let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(90));
    Roster::new(RosterId::new("R1"), period, legs, rotations).unwrap()
}

/// Build a scalability fixture with `n_light` light rotations (2 pairings each)
/// and 1 heavy rotation holding `heavy_count` pairings.
///
/// Light rotations receive the EARLIEST pairings (dep_h 0..n_light*2*6).
/// Heavy rotation receives the LATEST pairings (dep_h n_light*2*6..).
/// This guarantees relocate moves satisfy `Rotation` chronological ordering.
fn build_fixture(n_light: usize, heavy_count: usize) -> Roster {
    let n_pairings = n_light * 2 + heavy_count;
    let mut all_legs: Vec<FlightLeg> = Vec::new();
    let mut all_pairings: Vec<Pairing> = Vec::new();

    for i in 0..n_pairings {
        let dep_h = (i * 6) as i64;
        let (legs, pairing) = make_pairing(&format!("P{i:03}"), dep_h);
        all_legs.extend(legs);
        all_pairings.push(pairing);
    }

    // Light rotations: first n_light*2 pairings, 2 each (EARLY slots).
    let mut rotations: Vec<Rotation> = Vec::new();
    for i in 0..n_light {
        let p0 = all_pairings[i * 2].clone();
        let p1 = all_pairings[i * 2 + 1].clone();
        rotations.push(make_rotation(
            &format!("ROT_L{i:02}"),
            &format!("C_L{i:02}"),
            vec![p0, p1],
        ));
    }

    // Heavy rotation: last heavy_count pairings (LATE slots).
    let heavy_pairings: Vec<Pairing> = all_pairings[n_light * 2..].to_vec();
    rotations.push(make_rotation("ROT_H", "C_H", heavy_pairings));

    make_roster(all_legs, rotations)
}

// ── Result type ───────────────────────────────────────────────────────────────

/// Scalability measurement for one fixture size.
/// Reusable structure for M6.5 and M6.6 reporting.
struct ScaleResult {
    label: &'static str,
    n_rotations: usize,
    n_pairings: usize,
    max_iterations: usize,
    baseline_score: f64,
    optimized_score: f64,
    improvement_pct: f64,
    evaluations: usize,
    improvements: usize,
    acceptance_ratio: f64,
    runtime_ms: u128,
    pairings_conserved: bool,
    optimized_legal: bool,
}

fn run_scale(
    label: &'static str,
    n_light: usize,
    heavy_count: usize,
    max_iterations: usize,
) -> ScaleResult {
    let baseline = build_fixture(n_light, heavy_count);
    let n_rotations = n_light + 1;
    let n_pairings = n_light * 2 + heavy_count;

    let checker = LegalityChecker::new();
    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));
    let local_search = LocalSearch::new(&evaluator, &checker, vec![1.0], max_iterations);
    let mut metrics = OptimizationMetrics::new();

    let t0 = Instant::now();
    let optimized = local_search.run(&baseline, &mut metrics);
    let runtime_ms = t0.elapsed().as_millis();

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let improvement_pct = if baseline_score > 0.0 {
        (baseline_score - optimized_score) / baseline_score * 100.0
    } else {
        0.0
    };
    let acceptance_ratio = if metrics.evaluations() > 0 {
        metrics.improvements() as f64 / metrics.evaluations() as f64 * 100.0
    } else {
        0.0
    };

    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker2 = LegalityChecker::new();
    let optimized_legal = checker2.is_legal(&optimized);

    ScaleResult {
        label,
        n_rotations,
        n_pairings,
        max_iterations,
        baseline_score,
        optimized_score,
        improvement_pct,
        evaluations: metrics.evaluations(),
        improvements: metrics.improvements(),
        acceptance_ratio,
        runtime_ms,
        pairings_conserved: optimized_pairings == baseline_pairings,
        optimized_legal,
    }
}

// ── M6.4 test ─────────────────────────────────────────────────────────────────

/// M6.4: LocalSearch demonstrates bounded execution time and consistently
/// improves objective quality across representative scheduling scales while
/// preserving solution validity.
///
/// CI gates: correctness invariants only (environment-independent).
/// Evidence: runtime, evaluations, improvements, acceptance ratio (for M6.7).
#[test]
fn m6_4__scalability_small_medium_large() {
    // Iterations scale with problem size.
    let small  = run_scale("Small",   3,  6,  200);
    let medium = run_scale("Medium",  7, 10,  500);
    let large  = run_scale("Large",  15, 18, 1000);

    // Evidence report.
    println!();
    println!("M6.4 Scalability Report");
    println!("=======================");
    println!(
        "{:<8} {:>5} {:>8} {:>8} {:>10} {:>10} {:>8} {:>8} {:>8} {:>8} {:>12}",
        "Size", "Rots", "Pairs", "MaxIter",
        "Baseline", "Optimized", "Improv%",
        "Evals", "Imprv", "AccRatio%", "Runtime(ms)"
    );
    println!("{}", "-".repeat(100));
    for r in [&small, &medium, &large] {
        println!(
            "{:<8} {:>5} {:>8} {:>8} {:>10.4} {:>10.4} {:>7.1}% {:>8} {:>8} {:>8.2}% {:>12}",
            r.label, r.n_rotations, r.n_pairings, r.max_iterations,
            r.baseline_score, r.optimized_score, r.improvement_pct,
            r.evaluations, r.improvements, r.acceptance_ratio, r.runtime_ms
        );
    }
    println!();
    println!("Note: runtime_ms is evidence only — not a CI gate (hardware-dependent).");
    println!();

    // CI gates: correctness invariants only.
    for r in [&small, &medium, &large] {
        assert!(
            r.optimized_score < r.baseline_score,
            "M6.4 FAIL [{}]: optimized ({:.4}) must be < baseline ({:.4})",
            r.label, r.optimized_score, r.baseline_score
        );
        assert!(
            r.pairings_conserved,
            "M6.4 FAIL [{}]: pairings not conserved",
            r.label
        );
        assert!(
            r.optimized_legal,
            "M6.4 FAIL [{}]: optimized roster is not legal",
            r.label
        );
    }

    println!("CI gates:");
    println!("  optimized_score < baseline_score: PASS at all 3 scales");
    println!("  pairings conserved:               PASS at all 3 scales");
    println!("  optimized legal:                  PASS at all 3 scales");
}
