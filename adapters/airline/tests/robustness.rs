//! M6.5 -- Robustness Qualification
//!
//! # Hypothesis
//!
//! The optimizer preserves solution legality and recovers measurable objective
//! improvement under controlled disruptions to the operating environment.
//!
//! # Disruption scenarios
//!
//! | ID    | Disruption                          | Operational analogue                              |
//! |-------|-------------------------------------|---------------------------------------------------|
//! | ROB-1 | Crew unavailability                 | Crew member grounded; pairings redistributed      |
//! | ROB-2 | Pairing cancellation                | Flight cancelled; reduced workload rebalanced     |
//! | ROB-3 | Operational policy change           | Fatigue policy limits pairings per rotation       |
//!
//! # CI gates (environment-independent correctness invariants)
//!
//! For every disruption:
//!   1. Optimized solution is legal under the disruption's checker.
//!   2. Objective improves relative to the post-disruption baseline.
//!   3. Expected workload is conserved (or reduced by exactly the cancelled amount).
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
use coralys_airline::legality::{
    EntityRef, LegalityChecker, LegalityRule, LegalityViolation, ViolationSeverity,
};
use coralys_airline::optimization::cost::CostEvaluator;
use coralys_airline::optimization::metrics::OptimizationMetrics;
use coralys_airline::optimization::objective::{SchedulingObjective, WorkloadBalanceObjective};
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

fn make_pairing(id: &str, dep_h: i64) -> (Vec<FlightLeg>, Pairing) {
    let out = make_leg(&format!("{id}_o"), "LHR", "CDG", dep_h, dep_h + 2);
    let ret = make_leg(&format!("{id}_r"), "CDG", "LHR", dep_h + 3, dep_h + 5);
    let duty = Duty::new(
        DutyId::new(&format!("{id}_D")),
        vec![out.clone(), ret.clone()],
    )
    .unwrap();
    let pairing = Pairing::new(PairingId::new(id), AirportCode::new("LHR"), vec![duty]).unwrap();
    (vec![out, ret], pairing)
}

fn make_rotation(rot_id: &str, crew_id: &str, pairings: Vec<Pairing>) -> Rotation {
    Rotation::new(RotationId::new(rot_id), CrewId::new(crew_id), pairings).unwrap()
}

fn make_roster(legs: Vec<FlightLeg>, rotations: Vec<Rotation>) -> Roster {
    let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(30));
    Roster::new(RosterId::new("R1"), period, legs, rotations).unwrap()
}

fn run_optimizer(baseline: &Roster, checker: LegalityChecker) -> (Roster, OptimizationMetrics) {
    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));
    let local_search = LocalSearch::new(&evaluator, &checker, vec![1.0], 500);
    let mut metrics = OptimizationMetrics::new();
    let optimized = local_search.run(baseline, &mut metrics);
    (optimized, metrics)
}

// ── ROB-3 policy rule ─────────────────────────────────────────────────────────

/// Operational policy rule: no rotation may exceed `max_pairings` pairings.
///
/// Models a fatigue management policy change (e.g. a revised industrial agreement
/// or a regulator-imposed temporary operational limit).
struct FatiguePolicyMaxPairings {
    max_pairings: usize,
}

impl LegalityRule for FatiguePolicyMaxPairings {
    fn rule_id(&self) -> &str {
        "FATIGUE_MAX_PAIRINGS_PER_ROTATION"
    }

    fn rule_name(&self) -> &str {
        "Fatigue policy: maximum pairings per rotation"
    }

    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        roster
            .rotations()
            .filter(|r| r.pairings().len() > self.max_pairings)
            .map(|r| LegalityViolation {
                rule_id: self.rule_id().to_string(),
                severity: ViolationSeverity::Error,
                entity: EntityRef::Rotation {
                    rotation_id: r.id.as_str().to_string(),
                    crew_id: r.crew_id.as_str().to_string(),
                },
                observed: r.pairings().len() as f64,
                threshold: self.max_pairings as f64,
                message: format!(
                    "Rotation {} (crew {}) has {} pairings, exceeding fatigue policy limit of {}",
                    r.id.as_str(),
                    r.crew_id.as_str(),
                    r.pairings().len(),
                    self.max_pairings
                ),
            })
            .collect()
    }
}

// ── ROB-1: Crew unavailability ────────────────────────────────────────────────

/// ROB-1: One crew member is grounded; their pairings are absorbed by another crew.
///
/// Operational analogue: crew member C4 becomes unavailable (illness, regulatory hold).
/// Their 2 pairings are absorbed by C1, creating an imbalanced post-disruption baseline.
///
/// Post-disruption baseline: C1=(8), C2=(2), C3=(2) -- 3 rotations, 12 pairings.
/// Optimal under disruption: C1=(4), C2=(4), C3=(4) -- variance = 0.0.
#[test]
fn rob_1__crew_unavailability() {
    let pairings: Vec<(Vec<FlightLeg>, Pairing)> = (0..12)
        .map(|i| make_pairing(&format!("R1P{i:02}"), (i * 6) as i64))
        .collect();

    let all_legs: Vec<FlightLeg> = pairings.iter().flat_map(|(legs, _)| legs.clone()).collect();

    // C4 unavailable: C1 absorbs all late pairings (P04..P11).
    let baseline = make_roster(
        all_legs,
        vec![
            make_rotation(
                "ROT2",
                "C2",
                vec![pairings[0].1.clone(), pairings[1].1.clone()],
            ),
            make_rotation(
                "ROT3",
                "C3",
                vec![pairings[2].1.clone(), pairings[3].1.clone()],
            ),
            make_rotation(
                "ROT1",
                "C1",
                vec![
                    pairings[4].1.clone(),
                    pairings[5].1.clone(),
                    pairings[6].1.clone(),
                    pairings[7].1.clone(),
                    pairings[8].1.clone(),
                    pairings[9].1.clone(),
                    pairings[10].1.clone(),
                    pairings[11].1.clone(),
                ],
            ),
        ],
    );

    let checker = LegalityChecker::new();
    let (optimized, metrics) = run_optimizer(&baseline, checker);

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let improvement_pct = if baseline_score > 0.0 {
        (baseline_score - optimized_score) / baseline_score * 100.0
    } else {
        0.0
    };

    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker2 = LegalityChecker::new();
    let optimized_legal = checker2.is_legal(&optimized);

    println!();
    println!("ROB-1: Crew Unavailability");
    println!("  Disruption:  C4 grounded; C1 absorbs 8 pairings (3 rotations remain)");
    println!("  Baseline:    (8,2,2) -- score={:.4}", baseline_score);
    println!(
        "  Optimized:   score={:.4}  improvement={:.1}%",
        optimized_score, improvement_pct
    );
    println!(
        "  Evaluations: {}  Improvements: {}",
        metrics.evaluations(),
        metrics.improvements()
    );
    println!(
        "  Legal: {}  Pairings conserved: {}",
        optimized_legal,
        optimized_pairings == baseline_pairings
    );

    assert!(
        optimized_score < baseline_score,
        "ROB-1 FAIL: optimized ({:.4}) must be < post-disruption baseline ({:.4})",
        optimized_score,
        baseline_score
    );
    assert!(
        optimized_pairings == baseline_pairings,
        "ROB-1 FAIL: pairings not conserved"
    );
    assert!(optimized_legal, "ROB-1 FAIL: optimized roster is not legal");
}

// ── ROB-2: Pairing cancellation ───────────────────────────────────────────────

/// ROB-2: One pairing is cancelled before optimization.
///
/// Operational analogue: a flight is cancelled due to weather or technical fault.
/// The remaining 11 pairings must be rebalanced across 4 rotations.
///
/// Post-disruption baseline: C1=(5), C2=(2), C3=(2), C4=(2) -- 11 pairings.
/// Optimal under disruption: approximately (3,3,3,2) -- reduced variance.
#[test]
fn rob_2__pairing_cancellation() {
    let pairings: Vec<(Vec<FlightLeg>, Pairing)> = (0..12)
        .map(|i| make_pairing(&format!("R2P{i:02}"), (i * 6) as i64))
        .collect();

    // P06 cancelled (first pairing of the heavy rotation).
    let all_legs: Vec<FlightLeg> = pairings
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 6)
        .flat_map(|(_, (legs, _))| legs.clone())
        .collect();

    let baseline = make_roster(
        all_legs,
        vec![
            make_rotation(
                "ROT2",
                "C2",
                vec![pairings[0].1.clone(), pairings[1].1.clone()],
            ),
            make_rotation(
                "ROT3",
                "C3",
                vec![pairings[2].1.clone(), pairings[3].1.clone()],
            ),
            make_rotation(
                "ROT4",
                "C4",
                vec![pairings[4].1.clone(), pairings[5].1.clone()],
            ),
            // P06 cancelled; heavy rotation has 5 pairings (P07..P11).
            make_rotation(
                "ROT1",
                "C1",
                vec![
                    pairings[7].1.clone(),
                    pairings[8].1.clone(),
                    pairings[9].1.clone(),
                    pairings[10].1.clone(),
                    pairings[11].1.clone(),
                ],
            ),
        ],
    );

    let checker = LegalityChecker::new();
    let (optimized, metrics) = run_optimizer(&baseline, checker);

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let improvement_pct = if baseline_score > 0.0 {
        (baseline_score - optimized_score) / baseline_score * 100.0
    } else {
        0.0
    };

    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let checker2 = LegalityChecker::new();
    let optimized_legal = checker2.is_legal(&optimized);

    println!();
    println!("ROB-2: Pairing Cancellation");
    println!("  Disruption:  P06 cancelled; 11 pairings remain across 4 rotations");
    println!("  Baseline:    (5,2,2,2) -- score={:.4}", baseline_score);
    println!(
        "  Optimized:   score={:.4}  improvement={:.1}%",
        optimized_score, improvement_pct
    );
    println!(
        "  Evaluations: {}  Improvements: {}",
        metrics.evaluations(),
        metrics.improvements()
    );
    println!(
        "  Legal: {}  Pairings conserved: {}",
        optimized_legal,
        optimized_pairings == baseline_pairings
    );

    assert!(
        optimized_score < baseline_score,
        "ROB-2 FAIL: optimized ({:.4}) must be < post-disruption baseline ({:.4})",
        optimized_score,
        baseline_score
    );
    assert!(
        optimized_pairings == baseline_pairings,
        "ROB-2 FAIL: pairings not conserved"
    );
    assert!(optimized_legal, "ROB-2 FAIL: optimized roster is not legal");
}

// ── ROB-3: Operational policy change ─────────────────────────────────────────

/// ROB-3: A revised fatigue management policy limits each rotation to 4 pairings.
///
/// Operational analogue: a regulator introduces a temporary fatigue restriction
/// (e.g. following an incident review) that caps pairings per rotation at 4.
///
/// The baseline is constructed to already satisfy the policy (all rotations ≤ 4)
/// but is unbalanced: (4,4,2,2). The optimizer must find a more balanced solution
/// while remaining compliant with the policy throughout.
///
/// Key property: every intermediate relocate move from a 4-pairing rotation to a
/// 2-pairing rotation produces (3,4,3,2) or similar — all ≤ 4, so all moves are
/// immediately legal. The optimizer can navigate to (3,3,3,3) without passing
/// through any illegal intermediate state.
///
/// Post-disruption baseline: (4,4,2,2) -- legal under policy, but unbalanced.
/// Optimal under policy: (3,3,3,3) -- legal and balanced.
#[test]
fn rob_3__operational_policy_change() {
    // 12 pairings in 6h slots.
    // ROT3/ROT4 (light): EARLY pairings (dep_h 0..18).
    // ROT1/ROT2 (medium-heavy): LATE pairings (dep_h 24..66).
    // All rotations ≤ 4 pairings -- baseline is legal under the policy.
    let pairings: Vec<(Vec<FlightLeg>, Pairing)> = (0..12)
        .map(|i| make_pairing(&format!("R3P{i:02}"), (i * 6) as i64))
        .collect();

    let all_legs: Vec<FlightLeg> = pairings.iter().flat_map(|(legs, _)| legs.clone()).collect();

    // Baseline (4,4,2,2): ROT1 and ROT2 each have 4 LATE pairings.
    // ROT3 and ROT4 each have 2 EARLY pairings.
    // Relocating from ROT1/ROT2 to ROT3/ROT4 appends LATE pairings after EARLY ones -- valid.
    let baseline = make_roster(
        all_legs,
        vec![
            make_rotation(
                "ROT3",
                "C3",
                vec![pairings[0].1.clone(), pairings[1].1.clone()],
            ),
            make_rotation(
                "ROT4",
                "C4",
                vec![pairings[2].1.clone(), pairings[3].1.clone()],
            ),
            make_rotation(
                "ROT1",
                "C1",
                vec![
                    pairings[4].1.clone(),
                    pairings[5].1.clone(),
                    pairings[6].1.clone(),
                    pairings[7].1.clone(),
                ],
            ),
            make_rotation(
                "ROT2",
                "C2",
                vec![
                    pairings[8].1.clone(),
                    pairings[9].1.clone(),
                    pairings[10].1.clone(),
                    pairings[11].1.clone(),
                ],
            ),
        ],
    );

    // Fatigue policy: no rotation may exceed 4 pairings.
    let mut policy_checker = LegalityChecker::new();
    policy_checker.add_rule(Box::new(FatiguePolicyMaxPairings { max_pairings: 4 }));

    // Baseline must be legal under the policy (all rotations ≤ 4).
    assert!(
        policy_checker.is_legal(&baseline),
        "ROB-3: baseline must be legal under the fatigue policy"
    );

    let (optimized, metrics) = run_optimizer(&baseline, policy_checker);

    // Re-check with a fresh policy checker.
    let mut checker2 = LegalityChecker::new();
    checker2.add_rule(Box::new(FatiguePolicyMaxPairings { max_pairings: 4 }));

    let obj = WorkloadBalanceObjective;
    let baseline_score = obj.evaluate(&baseline);
    let optimized_score = obj.evaluate(&optimized);
    let improvement_pct = if baseline_score > 0.0 {
        (baseline_score - optimized_score) / baseline_score * 100.0
    } else {
        0.0
    };

    let baseline_pairings: usize = baseline.rotations().map(|r| r.pairings().len()).sum();
    let optimized_pairings: usize = optimized.rotations().map(|r| r.pairings().len()).sum();
    let optimized_legal = checker2.is_legal(&optimized);

    println!();
    println!("ROB-3: Operational Policy Change");
    println!("  Disruption:  Fatigue policy: max 4 pairings per rotation");
    println!(
        "  Baseline:    (4,4,2,2) -- legal under policy -- score={:.4}",
        baseline_score
    );
    println!(
        "  Optimized:   score={:.4}  improvement={:.1}%",
        optimized_score, improvement_pct
    );
    println!(
        "  Evaluations: {}  Improvements: {}",
        metrics.evaluations(),
        metrics.improvements()
    );
    println!(
        "  Legal under policy: {}  Pairings conserved: {}",
        optimized_legal,
        optimized_pairings == baseline_pairings
    );

    assert!(
        optimized_score < baseline_score,
        "ROB-3 FAIL: optimized ({:.4}) must be < post-disruption baseline ({:.4})",
        optimized_score,
        baseline_score
    );
    assert!(
        optimized_pairings == baseline_pairings,
        "ROB-3 FAIL: pairings not conserved"
    );
    assert!(
        optimized_legal,
        "ROB-3 FAIL: optimized roster must comply with fatigue policy (max 4 pairings per rotation)"
    );
}
