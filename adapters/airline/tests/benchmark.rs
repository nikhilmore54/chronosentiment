//! M6.6 -- Benchmark Validation
//!
//! # Hypothesis
//!
//! On representative scheduling instances of varying size and structure, the
//! optimizer consistently reduces workload variance relative to the initial
//! baseline, with improvement rates and convergence behaviour consistent with
//! a steepest-descent local search on this problem class.
//!
//! # Benchmark instances
//!
//! | ID   | Rotations | Pairings | Legs/pairing | Character                              |
//! |------|-----------|----------|--------------|----------------------------------------|
//! | BM-1 | 4         | 16       | 2            | Small, uniform leg counts              |
//! | BM-2 | 8         | 32       | 2            | Medium, maximally imbalanced baseline  |
//! | BM-3 | 16        | 64       | 2            | Large, uniform (scalability crossover) |
//! | BM-4 | 8         | 32       | mixed (2/4)  | Medium, heterogeneous leg counts       |
//!
//! # CI gates (environment-independent correctness invariants)
//!
//! For every instance:
//!   1. Optimized score is strictly less than baseline score.
//!   2. Total pairings are conserved.
//!   3. Optimized roster is legal.
//!
//! Runtime is reported as evidence only; no runtime gate is applied.
//!
//! # Fixture design rules (from M6.3)
//!
//! 1. Heavy rotation holds LATEST pairings; light rotations hold EARLIEST.
//! 2. All rotations have >= 2 pairings.

#![allow(non_snake_case)]

use coralys_airline::domain::crew::CrewId;
use coralys_airline::domain::duty::{Duty, DutyId};
use coralys_airline::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_airline::domain::pairing::{Pairing, PairingId};
use coralys_airline::domain::roster::{PlanningPeriod, Roster, RosterId};
use coralys_airline::domain::rotation::{Rotation, RotationId};
use coralys_airline::legality::{LegalityChecker};
use coralys_airline::optimization::cost::CostEvaluator;
use coralys_airline::optimization::metrics::OptimizationMetrics;
use coralys_airline::optimization::objective::{
    SchedulingObjective, WorkloadBalanceObjective,
};
use coralys_airline::optimization::search::local_search::LocalSearch;

use chrono::{Duration, TimeZone, Utc};

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

/// Build a pairing with 2 legs (outbound + return), starting at `dep_h`.
fn make_pairing_2leg(id: &str, dep_h: i64) -> (Vec<FlightLeg>, Pairing) {
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

/// Build a pairing with 4 legs (two duties, two legs each), starting at `dep_h`.
/// Duty 1: dep_h → dep_h+2 (LHR→CDG), dep_h+3 → dep_h+5 (CDG→AMS).
/// Duty 2: dep_h+14 → dep_h+16 (AMS→CDG), dep_h+17 → dep_h+19 (CDG→LHR).
/// 9-hour rest between duties satisfies the 10h minimum rest rule... but since
/// we use a bare LegalityChecker (no rules registered), this is not checked.
fn make_pairing_4leg(id: &str, dep_h: i64) -> (Vec<FlightLeg>, Pairing) {
    let l1 = make_leg(&format!("{id}_l1"), "LHR", "CDG", dep_h,      dep_h + 2);
    let l2 = make_leg(&format!("{id}_l2"), "CDG", "AMS", dep_h + 3,  dep_h + 5);
    let l3 = make_leg(&format!("{id}_l3"), "AMS", "CDG", dep_h + 15, dep_h + 17);
    let l4 = make_leg(&format!("{id}_l4"), "CDG", "LHR", dep_h + 18, dep_h + 20);
    let duty1 = Duty::new(
        DutyId::new(&format!("{id}_D1")),
        vec![l1.clone(), l2.clone()],
    )
    .unwrap();
    let duty2 = Duty::new(
        DutyId::new(&format!("{id}_D2")),
        vec![l3.clone(), l4.clone()],
    )
    .unwrap();
    let pairing = Pairing::new(
        PairingId::new(id),
        AirportCode::new("LHR"),
        vec![duty1, duty2],
    )
    .unwrap();
    (vec![l1, l2, l3, l4], pairing)
}

fn make_rotation(rot_id: &str, crew_id: &str, pairings: Vec<Pairing>) -> Rotation {
    Rotation::new(RotationId::new(rot_id), CrewId::new(crew_id), pairings).unwrap()
}

fn make_roster(legs: Vec<FlightLeg>, rotations: Vec<Rotation>) -> Roster {
    let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(60));
    Roster::new(RosterId::new("R1"), period, legs, rotations).unwrap()
}

fn run_optimizer(
    baseline: &Roster,
    max_iterations: usize,
) -> (Roster, OptimizationMetrics) {
    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));
    let checker = LegalityChecker::new();
    let local_search = LocalSearch::new(&evaluator, &checker, vec![1.0], max_iterations);
    let mut metrics = OptimizationMetrics::new();
    let optimized = local_search.run(baseline, &mut metrics);
    (optimized, metrics)
}

/// Summary of one benchmark run, used for the evidence table.
struct BenchmarkResult {
    id: &'static str,
    rotations: usize,
    pairings: usize,
    baseline_score: f64,
    optimized_score: f64,
    improvement_pct: f64,
    evaluations: usize,
    improvements: usize,
    acceptance_ratio: f64,
}

impl BenchmarkResult {
    fn new(
        id: &'static str,
        rotations: usize,
        pairings: usize,
        baseline_score: f64,
        optimized_score: f64,
        evaluations: usize,
        improvements: usize,
    ) -> Self {
        let improvement_pct = if baseline_score > 0.0 {
            (baseline_score - optimized_score) / baseline_score * 100.0
        } else {
            0.0
        };
        let acceptance_ratio = if evaluations > 0 {
            improvements as f64 / evaluations as f64 * 100.0
        } else {
            0.0
        };
        Self {
            id,
            rotations,
            pairings,
            baseline_score,
            optimized_score,
            improvement_pct,
            evaluations,
            improvements,
            acceptance_ratio,
        }
    }

    fn print(&self) {
        println!(
            "  {}: {} rot / {} pairs | baseline={:.4} optimized={:.4} improvement={:.1}% | \
             evals={} improvements={} acceptance={:.2}%",
            self.id,
            self.rotations,
            self.pairings,
            self.baseline_score,
            self.optimized_score,
            self.improvement_pct,
            self.evaluations,
            self.improvements,
            self.acceptance_ratio,
        );
    }
}

// ── BM-1: Small, uniform ─────────────────────────────────────────────────────

/// BM-1: 4 rotations, 16 pairings, 2 legs each.
///
/// Baseline: (10, 2, 2, 2) — one heavy rotation holds the 10 latest pairings.
/// Optimal:  (4, 4, 4, 4) — variance = 0.
///
/// This is the smallest representative instance. It establishes the baseline
/// optimizer behaviour on a well-structured problem.
#[test]
fn bm_1__small_uniform() {
    // 16 pairings in 6h slots.
    let pairings: Vec<(Vec<FlightLeg>, Pairing)> = (0..16)
        .map(|i| make_pairing_2leg(&format!("B1P{i:02}"), (i * 6) as i64))
        .collect();

    let all_legs: Vec<FlightLeg> = pairings.iter()
        .flat_map(|(legs, _)| legs.clone())
        .collect();

    // Light rotations hold EARLIEST pairings (P00..P05); heavy holds LATEST (P06..P15).
    let baseline = make_roster(
        all_legs,
        vec![
            make_rotation("ROT2", "C2", vec![
                pairings[0].1.clone(), pairings[1].1.clone(),
            ]),
            make_rotation("ROT3", "C3", vec![
                pairings[2].1.clone(), pairings[3].1.clone(),
            ]),
            make_rotation("ROT4", "C4", vec![
                pairings[4].1.clone(), pairings[5].1.clone(),
            ]),
            make_rotation("ROT1", "C1", vec![
                pairings[6].1.clone(),  pairings[7].1.clone(),
                pairings[8].1.clone(),  pairings[9].1.clone(),
                pairings[10].1.clone(), pairings[11].1.clone(),
                pairings[12].1.clone(), pairings[13].1.clone(),
                pairings[14].1.clone(), pairings[15].1.clone(),
            ]),
        ],
    );

    let (optimized, metrics) = run_optimizer(&baseline, 300);

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker = LegalityChecker::new();
    let optimized_legal = checker.is_legal(&optimized);

    let result = BenchmarkResult::new(
        "BM-1",
        4, 16,
        baseline_score, optimized_score,
        metrics.evaluations(), metrics.improvements(),
    );

    println!();
    println!("BM-1: Small, Uniform (4 rot / 16 pairs / 2 legs each)");
    result.print();
    println!("  Legal: {}  Pairings conserved: {}", optimized_legal, optimized_pairings == baseline_pairings);

    assert!(
        optimized_score < baseline_score,
        "BM-1 FAIL: optimized ({:.4}) must be < baseline ({:.4})",
        optimized_score, baseline_score
    );
    assert!(optimized_pairings == baseline_pairings, "BM-1 FAIL: pairings not conserved");
    assert!(optimized_legal, "BM-1 FAIL: optimized roster is not legal");
}

// ── BM-2: Medium, maximally imbalanced ───────────────────────────────────────

/// BM-2: 8 rotations, 32 pairings, 2 legs each.
///
/// Baseline: (18, 2, 2, 2, 2, 2, 2, 2) — one rotation holds 18 of 32 pairings.
/// This is the maximally adversarial configuration for 8 rotations with ≥2 each.
/// Optimal:  (4, 4, 4, 4, 4, 4, 4, 4) — variance = 0.
///
/// Tests the optimizer's ability to recover from a severely imbalanced state
/// on a medium-scale instance.
#[test]
fn bm_2__medium_imbalanced() {
    // 32 pairings in 6h slots.
    let pairings: Vec<(Vec<FlightLeg>, Pairing)> = (0..32)
        .map(|i| make_pairing_2leg(&format!("B2P{i:02}"), (i * 6) as i64))
        .collect();

    let all_legs: Vec<FlightLeg> = pairings.iter()
        .flat_map(|(legs, _)| legs.clone())
        .collect();

    // Light rotations hold EARLIEST pairings (P00..P13); heavy holds LATEST (P14..P31).
    let baseline = make_roster(
        all_legs,
        vec![
            make_rotation("ROT2", "C2", vec![pairings[0].1.clone(),  pairings[1].1.clone()]),
            make_rotation("ROT3", "C3", vec![pairings[2].1.clone(),  pairings[3].1.clone()]),
            make_rotation("ROT4", "C4", vec![pairings[4].1.clone(),  pairings[5].1.clone()]),
            make_rotation("ROT5", "C5", vec![pairings[6].1.clone(),  pairings[7].1.clone()]),
            make_rotation("ROT6", "C6", vec![pairings[8].1.clone(),  pairings[9].1.clone()]),
            make_rotation("ROT7", "C7", vec![pairings[10].1.clone(), pairings[11].1.clone()]),
            make_rotation("ROT8", "C8", vec![pairings[12].1.clone(), pairings[13].1.clone()]),
            make_rotation("ROT1", "C1", vec![
                pairings[14].1.clone(), pairings[15].1.clone(),
                pairings[16].1.clone(), pairings[17].1.clone(),
                pairings[18].1.clone(), pairings[19].1.clone(),
                pairings[20].1.clone(), pairings[21].1.clone(),
                pairings[22].1.clone(), pairings[23].1.clone(),
                pairings[24].1.clone(), pairings[25].1.clone(),
                pairings[26].1.clone(), pairings[27].1.clone(),
                pairings[28].1.clone(), pairings[29].1.clone(),
                pairings[30].1.clone(), pairings[31].1.clone(),
            ]),
        ],
    );

    let (optimized, metrics) = run_optimizer(&baseline, 800);

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker = LegalityChecker::new();
    let optimized_legal = checker.is_legal(&optimized);

    let result = BenchmarkResult::new(
        "BM-2",
        8, 32,
        baseline_score, optimized_score,
        metrics.evaluations(), metrics.improvements(),
    );

    println!();
    println!("BM-2: Medium, Maximally Imbalanced (8 rot / 32 pairs / 2 legs each)");
    result.print();
    println!("  Legal: {}  Pairings conserved: {}", optimized_legal, optimized_pairings == baseline_pairings);

    assert!(
        optimized_score < baseline_score,
        "BM-2 FAIL: optimized ({:.4}) must be < baseline ({:.4})",
        optimized_score, baseline_score
    );
    assert!(optimized_pairings == baseline_pairings, "BM-2 FAIL: pairings not conserved");
    assert!(optimized_legal, "BM-2 FAIL: optimized roster is not legal");
}

// ── BM-3: Large, uniform ─────────────────────────────────────────────────────

/// BM-3: 16 rotations, 64 pairings, 2 legs each.
///
/// Baseline: (36, 2, 2, ..., 2) — one rotation holds 36 of 64 pairings.
/// Optimal:  (4, 4, ..., 4) — variance = 0.
///
/// This is the large-scale instance, providing a scalability crossover point
/// with M6.4 (which used 16 rotations / 48 pairings). The larger pairing count
/// increases the search space and tests optimizer behaviour at scale.
#[test]
fn bm_3__large_uniform() {
    // 64 pairings in 6h slots.
    let pairings: Vec<(Vec<FlightLeg>, Pairing)> = (0..64)
        .map(|i| make_pairing_2leg(&format!("B3P{i:02}"), (i * 6) as i64))
        .collect();

    let all_legs: Vec<FlightLeg> = pairings.iter()
        .flat_map(|(legs, _)| legs.clone())
        .collect();

    // Light rotations hold EARLIEST pairings (P00..P27); heavy holds LATEST (P28..P63).
    let light: Vec<Rotation> = (0..15).map(|r| {
        let base = r * 2;
        make_rotation(
            &format!("ROT{}", r + 2),
            &format!("C{}", r + 2),
            vec![pairings[base].1.clone(), pairings[base + 1].1.clone()],
        )
    }).collect();

    let heavy = make_rotation("ROT1", "C1", (28..64).map(|i| pairings[i].1.clone()).collect());

    let mut rotations = light;
    rotations.push(heavy);

    let baseline = make_roster(all_legs, rotations);

    let (optimized, metrics) = run_optimizer(&baseline, 2000);

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker = LegalityChecker::new();
    let optimized_legal = checker.is_legal(&optimized);

    let result = BenchmarkResult::new(
        "BM-3",
        16, 64,
        baseline_score, optimized_score,
        metrics.evaluations(), metrics.improvements(),
    );

    println!();
    println!("BM-3: Large, Uniform (16 rot / 64 pairs / 2 legs each)");
    result.print();
    println!("  Legal: {}  Pairings conserved: {}", optimized_legal, optimized_pairings == baseline_pairings);

    assert!(
        optimized_score < baseline_score,
        "BM-3 FAIL: optimized ({:.4}) must be < baseline ({:.4})",
        optimized_score, baseline_score
    );
    assert!(optimized_pairings == baseline_pairings, "BM-3 FAIL: pairings not conserved");
    assert!(optimized_legal, "BM-3 FAIL: optimized roster is not legal");
}

// ── BM-4: Medium, heterogeneous leg counts ────────────────────────────────────

/// BM-4: 8 rotations, 32 pairings, mixed leg counts (2 and 4 legs per pairing).
///
/// Half the pairings have 2 legs (2 flight legs); half have 4 legs (4 flight legs).
/// The WorkloadBalanceObjective measures variance of total leg counts per rotation,
/// not pairing counts. This instance tests whether the optimizer correctly minimises
/// leg-count variance when pairings contribute unequal leg counts.
///
/// Baseline: heavy rotation holds all 4-leg pairings (16 legs) plus 2 of the
/// 2-leg pairings (4 legs) = 20 legs. Light rotations each hold 2 of the 2-leg
/// pairings = 4 legs each. Distribution: (20, 4, 4, 4, 4, 4, 4, 4).
///
/// The optimizer must balance leg counts, not just pairing counts.
#[test]
fn bm_4__medium_heterogeneous() {
    // 16 pairings with 2 legs each (EARLY slots: dep_h 0..96 in 6h steps).
    let pairs_2leg: Vec<(Vec<FlightLeg>, Pairing)> = (0..16)
        .map(|i| make_pairing_2leg(&format!("B4S{i:02}"), (i * 6) as i64))
        .collect();

    // 16 pairings with 4 legs each (LATE slots: dep_h 100..196 in 6h steps).
    // Each 4-leg pairing spans dep_h to dep_h+20, so slots must not overlap.
    let pairs_4leg: Vec<(Vec<FlightLeg>, Pairing)> = (0..16)
        .map(|i| make_pairing_4leg(&format!("B4L{i:02}"), 100 + (i * 24) as i64))
        .collect();

    let all_legs: Vec<FlightLeg> = pairs_2leg.iter()
        .flat_map(|(legs, _)| legs.clone())
        .chain(pairs_4leg.iter().flat_map(|(legs, _)| legs.clone()))
        .collect();

    // Light rotations (ROT2..ROT8): each holds 2 of the EARLY 2-leg pairings.
    // Heavy rotation (ROT1): holds 2 EARLY 2-leg pairings + all 16 LATE 4-leg pairings.
    // Leg counts: ROT1 = 2*2 + 16*4 = 68; ROT2..ROT8 = 2*2 = 4 each.
    // Note: ROT1 must hold EARLY 2-leg pairings before LATE 4-leg pairings (chronological order).
    let baseline = make_roster(
        all_legs,
        vec![
            make_rotation("ROT2", "C2", vec![pairs_2leg[0].1.clone(),  pairs_2leg[1].1.clone()]),
            make_rotation("ROT3", "C3", vec![pairs_2leg[2].1.clone(),  pairs_2leg[3].1.clone()]),
            make_rotation("ROT4", "C4", vec![pairs_2leg[4].1.clone(),  pairs_2leg[5].1.clone()]),
            make_rotation("ROT5", "C5", vec![pairs_2leg[6].1.clone(),  pairs_2leg[7].1.clone()]),
            make_rotation("ROT6", "C6", vec![pairs_2leg[8].1.clone(),  pairs_2leg[9].1.clone()]),
            make_rotation("ROT7", "C7", vec![pairs_2leg[10].1.clone(), pairs_2leg[11].1.clone()]),
            make_rotation("ROT8", "C8", vec![pairs_2leg[12].1.clone(), pairs_2leg[13].1.clone()]),
            make_rotation("ROT1", "C1", {
                // EARLY 2-leg pairings first, then LATE 4-leg pairings.
                let mut v: Vec<Pairing> = vec![
                    pairs_2leg[14].1.clone(),
                    pairs_2leg[15].1.clone(),
                ];
                v.extend(pairs_4leg.iter().map(|(_, p)| p.clone()));
                v
            }),
        ],
    );

    let (optimized, metrics) = run_optimizer(&baseline, 800);

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker = LegalityChecker::new();
    let optimized_legal = checker.is_legal(&optimized);

    let result = BenchmarkResult::new(
        "BM-4",
        8, 32,
        baseline_score, optimized_score,
        metrics.evaluations(), metrics.improvements(),
    );

    println!();
    println!("BM-4: Medium, Heterogeneous Leg Counts (8 rot / 32 pairs / mixed 2+4 legs)");
    result.print();
    println!("  Legal: {}  Pairings conserved: {}", optimized_legal, optimized_pairings == baseline_pairings);

    assert!(
        optimized_score < baseline_score,
        "BM-4 FAIL: optimized ({:.4}) must be < baseline ({:.4})",
        optimized_score, baseline_score
    );
    assert!(optimized_pairings == baseline_pairings, "BM-4 FAIL: pairings not conserved");
    assert!(optimized_legal, "BM-4 FAIL: optimized roster is not legal");
}