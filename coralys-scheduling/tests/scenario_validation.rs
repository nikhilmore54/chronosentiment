//! M6.2 — Functional Scenario Validation
//!
//! Eleven deterministic end-to-end scenarios that exercise every layer of the
//! Coralys scheduling engine.  Each scenario is self-contained, uses only
//! public API, and produces a binary PASS/FAIL assertion.
//!
//! Scenario IDs are **stable** — they are never renumbered.  New scenarios
//! continue from SCN-012 onwards.
//!
//! # Scenario index
//!
//! | ID      | Layer(s) | Description                                              |
//! |---------|----------|----------------------------------------------------------|
//! | SCN-001 | L2       | Empty roster is legal with no rules                      |
//! | SCN-002 | L1+L2    | Single crew, single pairing — legal assignment           |
//! | SCN-003 | L2       | Error rule propagates correctly                          |
//! | SCN-004 | L4       | Greedy scheduler assigns additional pairings             |
//! | SCN-005 | L4       | Cost evaluator ranks balanced roster below unbalanced    |
//! | SCN-006 | L4+L2    | Local search never produces an illegal roster            |
//! | SCN-007 | L5       | Disruption recovery removes a cancelled pairing          |
//! | SCN-008 | L5       | Crew-unavailable disruption orphans all pairings         |
//! | SCN-009 | L5       | Robustness evaluator scores generous-rest roster higher  |
//! | SCN-010 | L2       | Multi-rule checker accumulates violations from all rules |
//! | SCN-011 | L3       | ViolationSummary faithfully reflects legality            |

#![allow(non_snake_case)]

// ── Imports ───────────────────────────────────────────────────────────────────

use coralys_scheduling::domain::crew::{CrewId, CrewMember, CrewRole, Qualification};
use coralys_scheduling::domain::duty::{Duty, DutyId};
use coralys_scheduling::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_scheduling::domain::pairing::{Pairing, PairingId};
use coralys_scheduling::domain::roster::{PlanningPeriod, Roster, RosterId};
use coralys_scheduling::domain::rotation::{Rotation, RotationId};
use coralys_scheduling::legality::{
    EntityRef, LegalityChecker, LegalityRule, LegalityViolation, ViolationSeverity,
};
use coralys_scheduling::optimization::cost::CostEvaluator;
use coralys_scheduling::optimization::metrics::OptimizationMetrics;
use coralys_scheduling::optimization::objective::{
    SchedulingObjective, WorkloadBalanceObjective,
};
use coralys_scheduling::optimization::search::greedy::GreedyScheduler;
use coralys_scheduling::optimization::search::local_search::LocalSearch;
use coralys_scheduling::planner::summary::ViolationSummary;
use coralys_scheduling::resilience::disruption::{
    Disruption, DisruptionKind, DisruptionRecovery,
};
use coralys_scheduling::resilience::robustness::RobustnessEvaluator;

use chrono::{Duration, TimeZone, Utc};

// ── Inline fixture helpers ────────────────────────────────────────────────────
//
// `test_helpers` in legality/mod.rs is `#[cfg(test)] pub(crate)` and is
// therefore inaccessible from integration tests.  We duplicate the minimal
// subset needed here using only public API.

/// Fixed base time: 2026-07-01T00:00:00Z — matches the crate's test_helpers.
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

fn make_duty(id: &str, legs: Vec<FlightLeg>) -> Duty {
    Duty::new(DutyId::new(id), legs).unwrap()
}

/// Build a round-trip pairing: outbound `base→dest` at `dep_h..arr_h`,
/// then return `dest→base` at `(arr_h+2)..(arr_h+4)`.
///
/// Returns `(outbound_leg, return_leg, pairing)`.
fn make_roundtrip(
    id: &str,
    base: &str,
    dest: &str,
    dep_h: i64,
    arr_h: i64,
) -> (FlightLeg, FlightLeg, Pairing) {
    let out_leg = make_leg(&format!("{id}_o"), base, dest, dep_h, arr_h);
    let ret_leg = make_leg(&format!("{id}_r"), dest, base, arr_h + 2, arr_h + 4);
    let duty = make_duty(
        &format!("{id}_d"),
        vec![out_leg.clone(), ret_leg.clone()],
    );
    let pairing = Pairing::new(
        PairingId::new(id),
        AirportCode::new(base),
        vec![duty],
    )
    .unwrap();
    (out_leg, ret_leg, pairing)
}

fn make_rotation(rotation_id: &str, crew_id: &str, pairings: Vec<Pairing>) -> Rotation {
    Rotation::new(
        RotationId::new(rotation_id),
        CrewId::new(crew_id),
        pairings,
    )
    .unwrap()
}

fn make_crew(id: &str, base: &str) -> CrewMember {
    CrewMember::new(
        CrewId::new(id),
        format!("Crew {id}"),
        CrewRole::Captain,
        vec![Qualification::new(AircraftType::new("B737"))],
        AirportCode::new(base),
    )
}

fn make_roster(legs: Vec<FlightLeg>, rotations: Vec<Rotation>) -> Roster {
    let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(30));
    Roster::new(RosterId::new("R1"), period, legs, rotations).unwrap()
}

fn make_roster_with_crew(
    legs: Vec<FlightLeg>,
    rotations: Vec<Rotation>,
    crew_members: Vec<CrewMember>,
) -> Roster {
    let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(30));
    Roster::with_crew(RosterId::new("R1"), period, legs, rotations, crew_members).unwrap()
}

// ── Stub rules ────────────────────────────────────────────────────────────────

struct AlwaysErrorsA;
impl LegalityRule for AlwaysErrorsA {
    fn rule_id(&self) -> &str { "stub_error_a" }
    fn rule_name(&self) -> &str { "Stub Error A" }
    fn check(&self, _: &Roster) -> Vec<LegalityViolation> {
        vec![LegalityViolation::error(
            "stub_error_a",
            EntityRef::Roster("R1".into()),
            1.0, 0.0,
            "always fires (A)",
        )]
    }
}

struct AlwaysErrorsB;
impl LegalityRule for AlwaysErrorsB {
    fn rule_id(&self) -> &str { "stub_error_b" }
    fn rule_name(&self) -> &str { "Stub Error B" }
    fn check(&self, _: &Roster) -> Vec<LegalityViolation> {
        vec![LegalityViolation::error(
            "stub_error_b",
            EntityRef::Roster("R1".into()),
            1.0, 0.0,
            "always fires (B)",
        )]
    }
}

struct AlwaysWarns;
impl LegalityRule for AlwaysWarns {
    fn rule_id(&self) -> &str { "stub_warn" }
    fn rule_name(&self) -> &str { "Stub Warning" }
    fn check(&self, _: &Roster) -> Vec<LegalityViolation> {
        vec![LegalityViolation::warning(
            "stub_warn",
            EntityRef::Roster("R1".into()),
            1.0, 0.0,
            "always warns",
        )]
    }
}

/// Rejects any rotation that contains more than one pairing.
struct MaxOnePairingPerRotation;
impl LegalityRule for MaxOnePairingPerRotation {
    fn rule_id(&self) -> &str { "max_one_pairing" }
    fn rule_name(&self) -> &str { "Max One Pairing Per Rotation" }
    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        roster
            .rotations()
            .filter(|rot| rot.pairings().len() > 1)
            .map(|rot| LegalityViolation::error(
                "max_one_pairing",
                EntityRef::Rotation {
                    rotation_id: rot.id.to_string(),
                    crew_id: rot.crew_id.to_string(),
                },
                rot.pairings().len() as f64,
                1.0,
                "rotation has more than one pairing",
            ))
            .collect()
    }
}

// ── SCN-001 ───────────────────────────────────────────────────────────────────

/// SCN-001: An empty roster with no rules registered is legal.
///
/// Validates: `LegalityChecker::new()` + `is_legal()` baseline.
#[test]
fn scn_001__empty_roster_no_rules_is_legal() {
    let checker = LegalityChecker::new();
    let roster = make_roster(vec![], vec![]);

    assert!(
        checker.is_legal(&roster),
        "SCN-001 FAIL: empty roster with no rules must be legal"
    );
    assert_eq!(
        checker.errors(&roster).len(), 0,
        "SCN-001 FAIL: no errors expected"
    );
}

// ── SCN-002 ───────────────────────────────────────────────────────────────────

/// SCN-002: A roster with one crew member and one round-trip pairing is legal
/// when no rules are registered.
///
/// Validates: domain construction (L1) + legality oracle (L2) round-trip.
#[test]
fn scn_002__single_crew_single_pairing_legal() {
    let (out_leg, ret_leg, pairing) = make_roundtrip("P1", "LHR", "CDG", 8, 10);
    let rotation = make_rotation("ROT1", "C1", vec![pairing]);
    let roster = make_roster_with_crew(
        vec![out_leg, ret_leg],
        vec![rotation],
        vec![make_crew("C1", "LHR")],
    );

    let checker = LegalityChecker::new();
    assert!(
        checker.is_legal(&roster),
        "SCN-002 FAIL: single-crew single-pairing roster must be legal"
    );
}

// ── SCN-003 ───────────────────────────────────────────────────────────────────

/// SCN-003: A rule that always fires an error makes any roster illegal, and
/// the violation is faithfully propagated through `errors()`.
///
/// Validates: `add_rule()` + error propagation through `is_legal()` and `errors()`.
#[test]
fn scn_003__error_rule_propagates_correctly() {
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(AlwaysErrorsA));

    let roster = make_roster(vec![], vec![]);

    assert!(
        !checker.is_legal(&roster),
        "SCN-003 FAIL: roster must be illegal when an error rule fires"
    );

    let errors = checker.errors(&roster);
    assert_eq!(
        errors.len(), 1,
        "SCN-003 FAIL: exactly one error expected, got {}",
        errors.len()
    );
    assert_eq!(
        errors[0].rule_id, "stub_error_a",
        "SCN-003 FAIL: error must carry the correct rule_id"
    );
    assert_eq!(
        errors[0].severity, ViolationSeverity::Error,
        "SCN-003 FAIL: severity must be Error"
    );
}

// ── SCN-004 ───────────────────────────────────────────────────────────────────

/// SCN-004: The greedy scheduler assigns additional pairings to existing
/// rotations, increasing the total pairing count.
///
/// Validates: `GreedyScheduler::assign()` — pairings are appended, none dropped.
///
/// Initial roster: two rotations, one round-trip pairing each (2 total).
/// Assigned: two more round-trip pairings (departing after the first ones land).
/// Expected: total pairings > 2.
#[test]
fn scn_004__greedy_assigns_additional_pairings() {
    // Initial pairings (hours 8–14 for outbound+return).
    let (o1, r1, p1) = make_roundtrip("P1", "LHR", "CDG", 8, 10);
    let (o2, r2, p2) = make_roundtrip("P2", "LHR", "CDG", 8, 10);

    let roster = make_roster_with_crew(
        vec![o1, r1, o2, r2],
        vec![
            make_rotation("ROT1", "C1", vec![p1]),
            make_rotation("ROT2", "C2", vec![p2]),
        ],
        vec![make_crew("C1", "LHR"), make_crew("C2", "LHR")],
    );

    // Additional pairings to assign (depart after hour 14 so no overlap).
    let (o3, r3, p3) = make_roundtrip("P3", "LHR", "CDG", 20, 22);
    let (o4, r4, p4) = make_roundtrip("P4", "LHR", "CDG", 20, 22);
    let _ = (o3, r3, o4, r4); // legs not added to roster; greedy only needs pairings

    let initial_total: usize = roster.rotations().map(|r| r.pairings().len()).sum();
    assert_eq!(initial_total, 2, "SCN-004 setup: expected 2 initial pairings");

    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));
    let scheduler = GreedyScheduler::new(&evaluator, vec![]);
    let mut metrics = OptimizationMetrics::new();

    let result = scheduler.assign(&roster, vec![p3, p4], &mut metrics);

    let final_total: usize = result.rotations().map(|r| r.pairings().len()).sum();
    assert!(
        final_total > initial_total,
        "SCN-004 FAIL: greedy must increase total pairings; initial={initial_total}, final={final_total}"
    );
}

// ── SCN-005 ───────────────────────────────────────────────────────────────────

/// SCN-005: The workload-balance objective scores a balanced roster lower than
/// an unbalanced one (minimisation convention).
///
/// Validates: `WorkloadBalanceObjective::evaluate()`.
///
/// Balanced:   C1 has 1 pairing (2 legs), C2 has 1 pairing (2 legs) → variance = 0.
/// Unbalanced: C1 has 3 pairings (6 legs), C2 has 1 pairing (2 legs) → variance > 0.
#[test]
fn scn_005__balanced_roster_scores_lower_than_unbalanced() {
    // Balanced: one round-trip pairing per crew member.
    let (o1, r1, p1) = make_roundtrip("P1", "LHR", "CDG", 8, 10);
    let (o2, r2, p2) = make_roundtrip("P2", "LHR", "CDG", 8, 10);
    let balanced = make_roster_with_crew(
        vec![o1, r1, o2, r2],
        vec![
            make_rotation("ROT1", "C1", vec![p1]),
            make_rotation("ROT2", "C2", vec![p2]),
        ],
        vec![make_crew("C1", "LHR"), make_crew("C2", "LHR")],
    );

    // Unbalanced: C1 has three round-trip pairings, C2 has one.
    let (oa, ra, pa) = make_roundtrip("PA", "LHR", "CDG", 8, 10);
    let (ob, rb, pb) = make_roundtrip("PB", "LHR", "CDG", 20, 22);
    let (oc, rc, pc) = make_roundtrip("PC", "LHR", "CDG", 32, 34);
    let (od, rd, pd) = make_roundtrip("PD", "LHR", "CDG", 8, 10);
    let unbalanced = make_roster_with_crew(
        vec![oa, ra, ob, rb, oc, rc, od, rd],
        vec![
            make_rotation("ROT1", "C1", vec![pa, pb, pc]),
            make_rotation("ROT2", "C2", vec![pd]),
        ],
        vec![make_crew("C1", "LHR"), make_crew("C2", "LHR")],
    );

    let obj = WorkloadBalanceObjective;
    let balanced_score = obj.evaluate(&balanced);
    let unbalanced_score = obj.evaluate(&unbalanced);

    assert!(
        balanced_score < unbalanced_score,
        "SCN-005 FAIL: balanced score ({balanced_score:.4}) must be < unbalanced score ({unbalanced_score:.4})"
    );
}

// ── SCN-006 ───────────────────────────────────────────────────────────────────

/// SCN-006: Local search never produces a roster that violates a registered
/// error rule.
///
/// This is the most architecturally important scenario.  It validates that
/// the optimizer uses `LegalityChecker` as a feasibility oracle — any move
/// that would create an illegal roster is rejected, not accepted.
///
/// Validates: `LocalSearch` + `LegalityChecker` oracle integration (L4 + L2).
#[test]
fn scn_006__local_search_never_produces_illegal_roster() {
    let (o1, r1, p1) = make_roundtrip("P1", "LHR", "CDG", 8, 10);
    let (o2, r2, p2) = make_roundtrip("P2", "LHR", "CDG", 8, 10);
    let roster = make_roster_with_crew(
        vec![o1, r1, o2, r2],
        vec![
            make_rotation("ROT1", "C1", vec![p1]),
            make_rotation("ROT2", "C2", vec![p2]),
        ],
        vec![make_crew("C1", "LHR"), make_crew("C2", "LHR")],
    );

    // Rule: any rotation with more than one pairing is illegal.
    // Any relocate move that stacks two pairings on one rotation must be rejected.
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(MaxOnePairingPerRotation));

    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));

    let local_search = LocalSearch::new(&evaluator, &checker, vec![], 50);
    let mut metrics = OptimizationMetrics::new();
    let result = local_search.run(&roster, &mut metrics);

    assert!(
        checker.is_legal(&result),
        "SCN-006 FAIL: local search produced an illegal roster"
    );
}

// ── SCN-007 ───────────────────────────────────────────────────────────────────

/// SCN-007: Disruption recovery removes a cancelled pairing from its rotation.
///
/// Validates: `DisruptionRecovery::recover()` with `PairingCancelled`.
#[test]
fn scn_007__disruption_recovery_removes_cancelled_pairing() {
    // Rotation with two round-trip pairings.
    let (o1, r1, p1) = make_roundtrip("P1", "LHR", "CDG", 8, 10);
    let (o2, r2, p2) = make_roundtrip("P2", "LHR", "CDG", 20, 22);
    let roster = make_roster_with_crew(
        vec![o1, r1, o2, r2],
        vec![make_rotation("ROT1", "C1", vec![p1, p2])],
        vec![make_crew("C1", "LHR")],
    );

    // MaxOnePairingPerRotation prevents the orphaned pairing from being
    // re-assigned back to the rotation that already holds one pairing.
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(MaxOnePairingPerRotation));
    let recovery = DisruptionRecovery::new(&checker);

    let disruption = Disruption::new(
        DisruptionKind::PairingCancelled { rotation_index: 0, pairing_index: 0 },
        "P1 cancelled due to aircraft swap",
    );

    let result = recovery.recover(&roster, &[disruption]);

    // After cancellation the rotation holds 1 pairing; the orphaned pairing
    // cannot be re-assigned (MaxOnePairingPerRotation blocks it).
    let total_pairings: usize = result.roster.rotations().map(|r| r.pairings().len()).sum();
    assert!(
        total_pairings < 2,
        "SCN-007 FAIL: cancelled pairing must be removed; total pairings = {total_pairings}"
    );
}

// ── SCN-008 ───────────────────────────────────────────────────────────────────

/// SCN-008: A crew-unavailable disruption orphans all pairings in that rotation.
///
/// The conservation invariant must hold: unrecovered + recovered_count equals
/// the original pairing count in the disrupted rotation.
///
/// Validates: `DisruptionKind::CrewUnavailable` accounting (L5).
#[test]
fn scn_008__crew_unavailable_orphans_all_pairings() {
    let (o1, r1, p1) = make_roundtrip("P1", "LHR", "CDG", 8, 10);
    let (o2, r2, p2) = make_roundtrip("P2", "LHR", "CDG", 20, 22);
    let roster = make_roster_with_crew(
        vec![o1, r1, o2, r2],
        vec![make_rotation("ROT1", "C1", vec![p1, p2])],
        vec![make_crew("C1", "LHR")],
    );

    // Checker rejects any rotation with more than one pairing, so recovery
    // cannot re-assign both orphaned pairings to the same rotation.
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(MaxOnePairingPerRotation));
    let recovery = DisruptionRecovery::new(&checker);

    let disruption = Disruption::new(
        DisruptionKind::CrewUnavailable { rotation_index: 0 },
        "C1 sick leave",
    );

    let result = recovery.recover(&roster, &[disruption]);

    // Conservation invariant: unrecovered + recovered_count == original pairing count.
    let original_count = 2;
    assert_eq!(
        result.unrecovered.len() + result.recovered_count,
        original_count,
        "SCN-008 FAIL: unrecovered ({}) + recovered ({}) must equal original count ({})",
        result.unrecovered.len(), result.recovered_count, original_count
    );
}

// ── SCN-009 ───────────────────────────────────────────────────────────────────

/// SCN-009: The robustness evaluator scores a roster with spare crew capacity
/// higher than a fully-loaded roster.
///
/// Validates: `RobustnessEvaluator::evaluate()` — higher `overall` score = more robust.
///
/// Generous roster: two rotations, one pairing each → 100% crew slack (max=1).
/// Dense roster:    one rotation, two pairings      → 0% crew slack (max=1).
#[test]
fn scn_009__generous_rest_roster_scores_higher_robustness() {
    // Generous: two rotations, one pairing each.
    let (o1, r1, p1) = make_roundtrip("P1", "LHR", "CDG", 8, 10);
    let (o2, r2, p2) = make_roundtrip("P2", "LHR", "CDG", 8, 10);
    let generous = make_roster_with_crew(
        vec![o1, r1, o2, r2],
        vec![
            make_rotation("ROT1", "C1", vec![p1]),
            make_rotation("ROT2", "C2", vec![p2]),
        ],
        vec![make_crew("C1", "LHR"), make_crew("C2", "LHR")],
    );

    // Dense: one rotation, two pairings.
    let (o3, r3, p3) = make_roundtrip("P3", "LHR", "CDG", 8, 10);
    let (o4, r4, p4) = make_roundtrip("P4", "LHR", "CDG", 20, 22);
    let dense = make_roster_with_crew(
        vec![o3, r3, o4, r4],
        vec![make_rotation("ROT1", "C1", vec![p3, p4])],
        vec![make_crew("C1", "LHR")],
    );

    // max_pairings_per_rotation = 2: generous (1/2 used = 50% slack) > dense (2/2 = 0% slack).
    let evaluator = RobustnessEvaluator::new(480.0, 60.0, 2, 0.5);
    let generous_score = evaluator.evaluate(&generous);
    let dense_score = evaluator.evaluate(&dense);

    assert!(
        generous_score.overall > dense_score.overall,
        "SCN-009 FAIL: generous overall score ({:.4}) must exceed dense ({:.4})",
        generous_score.overall, dense_score.overall
    );
    assert!(
        generous_score.crew_slack_ratio > dense_score.crew_slack_ratio,
        "SCN-009 FAIL: generous crew_slack_ratio ({:.4}) must exceed dense ({:.4})",
        generous_score.crew_slack_ratio, dense_score.crew_slack_ratio
    );
}

// ── SCN-010 ───────────────────────────────────────────────────────────────────

/// SCN-010: A checker with multiple distinct rules accumulates violations from
/// all rules; warnings do not affect `is_legal()`.
///
/// Validates: `LegalityChecker` composition with distinct rule IDs (L2).
///
/// Rules: AlwaysErrorsA (rule_id="stub_error_a") + AlwaysErrorsB (rule_id="stub_error_b")
///        + AlwaysWarns (rule_id="stub_warn")
/// Expected: 3 total violations, 2 errors, 1 warning, is_legal() = false.
#[test]
fn scn_010__multi_rule_checker_accumulates_all_violations() {
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(AlwaysErrorsA)); // rule_id = "stub_error_a"
    checker.add_rule(Box::new(AlwaysErrorsB)); // rule_id = "stub_error_b"
    checker.add_rule(Box::new(AlwaysWarns));   // rule_id = "stub_warn"

    let roster = make_roster(vec![], vec![]);

    // Total violations = 3 (2 errors + 1 warning).
    let all_violations = checker.check(&roster);
    assert_eq!(
        all_violations.len(), 3,
        "SCN-010 FAIL: expected 3 total violations, got {}",
        all_violations.len()
    );

    // Errors = 2 (from two distinct rules).
    let errors = checker.errors(&roster);
    assert_eq!(
        errors.len(), 2,
        "SCN-010 FAIL: expected 2 errors, got {}",
        errors.len()
    );

    // Distinct rule IDs in the errors.
    let error_rule_ids: std::collections::HashSet<&str> =
        errors.iter().map(|v| v.rule_id.as_str()).collect();
    assert!(
        error_rule_ids.contains("stub_error_a"),
        "SCN-010 FAIL: expected stub_error_a in error rule IDs"
    );
    assert!(
        error_rule_ids.contains("stub_error_b"),
        "SCN-010 FAIL: expected stub_error_b in error rule IDs"
    );

    // Roster is illegal because there are errors.
    assert!(
        !checker.is_legal(&roster),
        "SCN-010 FAIL: roster with error violations must be illegal"
    );

    // Rule count matches registrations.
    assert_eq!(
        checker.rule_count(), 3,
        "SCN-010 FAIL: expected 3 registered rules, got {}",
        checker.rule_count()
    );
}

// ── SCN-011 ───────────────────────────────────────────────────────────────────

/// SCN-011: `ViolationSummary` (Layer 3) faithfully reflects
/// verdict produced by LegalityChecker (Layer 2).
#[test]
fn scn_011__violation_summary_faithfully_reflects_legality() {
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(AlwaysErrorsA));
    checker.add_rule(Box::new(AlwaysWarns));
    let roster = make_roster(vec![], vec![]);
    let violations = checker.check(&roster);
    let summary = ViolationSummary::new(violations);
    assert_eq!(summary.total(), 2, "SCN-011 FAIL: expected 2 violations");
    assert_eq!(summary.error_count(), 1, "SCN-011 FAIL: expected 1 error");
    assert_eq!(summary.warning_count(), 1, "SCN-011 FAIL: expected 1 warning");
    assert!(!summary.is_legal(), "SCN-011 FAIL: summary must report illegal");
    assert!(!checker.is_legal(&roster), "SCN-011 FAIL: checker must report illegal");
    let by_rule = summary.by_rule();
    assert!(by_rule.contains_key("stub_error_a"), "SCN-011 FAIL: missing stub_error_a");
    assert!(by_rule.contains_key("stub_warn"), "SCN-011 FAIL: missing stub_warn");
    assert_eq!(by_rule["stub_error_a"].len(), 1, "SCN-011 FAIL: stub_error_a count");
    assert_eq!(by_rule["stub_warn"].len(), 1, "SCN-011 FAIL: stub_warn count");
}
