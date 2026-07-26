//! M6.3 -- Solution Quality Validation
//!
//! Hypothesis:
//!   For a representative scheduling instance, Coralys Optimization produces
//!   a solution whose objective value is measurably lower than a deterministic
//!   constructive baseline.
//!
//! Fixture design:
//!   Heavy rotation (ROT1) holds the LATEST pairings (dep_h=36..66).
//!   Light rotations hold the EARLIEST pairings (dep_h=0..30).
//!   This ensures relocate moves satisfy Rotation chronological ordering.
//!
//! Baseline: (6,2,2,2) -- variance = 3.0
//! Optimal:  (3,3,3,3) -- variance = 0.0
//! Pipeline: LocalSearch (200 iterations, WorkloadBalanceObjective).
//! CI gate:  optimized_score < baseline_score (strict inequality).

#![allow(non_snake_case)]

use coralys_airline::domain::crew::CrewId;
use coralys_airline::domain::duty::{Duty, DutyId};
use coralys_airline::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_airline::domain::pairing::{Pairing, PairingId};
use coralys_airline::domain::roster::{PlanningPeriod, Roster, RosterId};
use coralys_airline::domain::rotation::{Rotation, RotationId};
use coralys_airline::legality::LegalityChecker;
use coralys_airline::optimization::cost::CostEvaluator;
use coralys_airline::optimization::metrics::OptimizationMetrics;
use coralys_airline::optimization::objective::{
    SchedulingObjective, WorkloadBalanceObjective,
};
use coralys_airline::optimization::search::local_search::LocalSearch;

use chrono::{Duration, TimeZone, Utc};

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

/// Closed pairing: LHR -> CDG -> LHR.
/// dep_h: outbound departure; arr_h = dep_h+5 (2h out + 1h ground + 2h return).
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
    let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(30));
    Roster::new(RosterId::new("R1"), period, legs, rotations).unwrap()
}

#[test]
fn m6_3__optimized_beats_deterministic_baseline() {
    // 12 pairings in 6h slots (each pairing: dep_h to dep_h+5).
    // Light rotations hold EARLY pairings (dep_h 0..30).
    // Heavy rotation holds LATE pairings (dep_h 36..66).
    // This ensures relocate moves (append to end) satisfy chronological ordering.
    let (l00, p00) = make_pairing("P00", 0);
    let (l01, p01) = make_pairing("P01", 6);
    let (l02, p02) = make_pairing("P02", 12);
    let (l03, p03) = make_pairing("P03", 18);
    let (l04, p04) = make_pairing("P04", 24);
    let (l05, p05) = make_pairing("P05", 30);
    let (l06, p06) = make_pairing("P06", 36);
    let (l07, p07) = make_pairing("P07", 42);
    let (l08, p08) = make_pairing("P08", 48);
    let (l09, p09) = make_pairing("P09", 54);
    let (l10, p10) = make_pairing("P10", 60);
    let (l11, p11) = make_pairing("P11", 66);

    let all_legs: Vec<FlightLeg> = [
        l00, l01, l02, l03, l04, l05, l06, l07, l08, l09, l10, l11,
    ]
    .into_iter().flatten().collect();

    // Baseline: (6,2,2,2) -- heavy rotation holds LATE pairings.
    // Mean = 3, variance = ((6-3)^2 + 3*(2-3)^2) / 4 = 3.0
    // WorkloadBalance measures leg counts: (12,4,4,4) legs, variance = 12.0
    let baseline = make_roster(
        all_legs.clone(),
        vec![
            // Light rotations: EARLY pairings (dep_h 0..30)
            make_rotation("ROT2", "C2", vec![p00, p01]),
            make_rotation("ROT3", "C3", vec![p02, p03]),
            make_rotation("ROT4", "C4", vec![p04, p05]),
            // Heavy rotation: LATE pairings (dep_h 36..66)
            make_rotation("ROT1", "C1", vec![p06, p07, p08, p09, p10, p11]),
        ],
    );

    // Optimized pipeline.
    let checker = LegalityChecker::new();
    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));
    let local_search = LocalSearch::new(&evaluator, &checker, vec![1.0], 200);
    let mut metrics = OptimizationMetrics::new();
    let optimized = local_search.run(&baseline, &mut metrics);

    // Scores.
    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);

    // Supporting metrics.
    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker2 = LegalityChecker::new();
    let baseline_legal = checker2.is_legal(&baseline);
    let optimized_legal = checker2.is_legal(&optimized);
    let improvement_pct = if baseline_score > 0.0 {
        (baseline_score - optimized_score) / baseline_score * 100.0
    } else {
        0.0
    };

    // Evidence report.
    println!();
    println!("M6.3 Solution Quality Report");
    println!("============================");
    println!("Fixture:    4 rotations, 12 pairings (LHR-CDG-LHR, non-overlapping)");
    println!("Baseline:   Unbalanced (6,2,2,2) -- heavy rotation holds late pairings");
    println!("Optimal:    Balanced   (3,3,3,3) -- variance = 0.0");
    println!("Pipeline:   LocalSearch (200 iterations, WorkloadBalanceObjective)");
    println!();
    println!("Metric               Baseline    Optimized");
    println!("-------------------------------------------");
    println!("Objective score      {:>10.4}  {:>10.4}", baseline_score, optimized_score);
    println!("Pairings assigned    {:>10}  {:>10}", baseline_pairings, optimized_pairings);
    println!("Legal                {:>10}  {:>10}", baseline_legal, optimized_legal);
    println!("Evaluations          {:>10}  {:>10}", 0, metrics.evaluations());
    println!("Improvements         {:>10}  {:>10}", 0, metrics.improvements());
    println!();
    println!("Observed improvement: {:.1}%", improvement_pct);
    println!();

    // CI gate.
    assert!(
        optimized_score < baseline_score,
        "M6.3 FAIL: optimized ({:.4}) must be < baseline ({:.4}); improvement={:.1}%",
        optimized_score, baseline_score, improvement_pct
    );
    assert_eq!(optimized_pairings, baseline_pairings, "M6.3 FAIL: pairings must be conserved");
    assert!(baseline_legal, "M6.3 FAIL: baseline must be legal");
    assert!(optimized_legal, "M6.3 FAIL: optimized must be legal");
}
