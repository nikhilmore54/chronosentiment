// tests/schedule_construction.rs
//! UC‑AIR‑002 evidence gate tests.
//! Constructs candidate rosters, applies immutable transformations, and verifies
//! evaluation with the existing `LegalityChecker` (all seven rules).

use coralys_airline::domain::crew::CrewId;
use coralys_airline::domain::flight::FlightLegId;
use coralys_airline::legality::{LegalityChecker, LegalityViolation};

mod fixtures {
    pub mod roster_fixtures;
    pub mod transformations;
}

use fixtures::roster_fixtures::{canonical_roster, roster_with_one_duty};
use fixtures::transformations::{
    assign_unqualified_crew,
    reduce_rest,
    reassign_leg,
    remove_required_leg,
    swap_duties,
    TransformationError,
};

use coralys_airline::legality::{coverage, duty_connectivity, duty_time, fdp, minimum_rest, qualification, base_return};

fn make_checker() -> LegalityChecker {
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(coverage::CoverageRule));
    checker.add_rule(Box::new(duty_connectivity::DutyConnectivityRule));
    checker.add_rule(Box::new(duty_time::MaximumDutyTimeRule::new()));
    checker.add_rule(Box::new(fdp::FlightDutyPeriodRule::new()));
    checker.add_rule(Box::new(minimum_rest::MinimumRestRule::new()));
    checker.add_rule(Box::new(qualification::QualificationRule));
    checker.add_rule(Box::new(base_return::BaseReturnRule));
    checker
}

/// Helper to run the full legality checker on a roster.
fn check_roster(roster: &coralys_airline::domain::roster::Roster) -> Vec<LegalityViolation> {
    let checker = make_checker();
    checker.check(roster)
}

#[test]
fn test_legal_transformations() {
    let base = canonical_roster();
    // Reassign a leg from C1 to C2 – should remain legal.
    let reassigned = reassign_leg(
        &base,
        &CrewId::new("C1"),
        &CrewId::new("C2"),
        &FlightLegId::new("L1"),
    )
    .expect("reassign_leg should succeed");
    let violations = check_roster(&reassigned);
    println!("Reassign violations: {:#?}", violations);
    assert!(violations.is_empty(), "reassign should be legal");

    // Swap duties between C1 and C2 – also legal.
    let swapped = swap_duties(&base, &CrewId::new("C1"), &CrewId::new("C2"))
        .expect("swap_duties should succeed");
    assert!(check_roster(&swapped).is_empty(), "swap should be legal");
}

#[test]
fn test_illegal_transformations() {
    let base = canonical_roster();

    // Remove a required leg – triggers CoverageRule.
    let missing = remove_required_leg(&base, &FlightLegId::new("L1"))
        .expect("remove_required_leg should succeed");
    let violations = check_roster(&missing);
    assert!(!violations.is_empty(), "coverage violation expected");
    assert!(violations.iter().any(|v| v.rule_id == "coverage"), "CoverageRule should be reported");



    // Assign unqualified crew – triggers QualificationRule.
    let unqualified = assign_unqualified_crew(&base, &CrewId::new("C1"), &FlightLegId::new("L1"))
        .expect("assign_unqualified_crew should succeed");
    let violations = check_roster(&unqualified);
    assert!(!violations.is_empty());
    assert!(violations.iter().any(|v| v.rule_id == "qualification"), "QualificationRule expected");

    // Reduce rest – triggers MinimumRestRule.
    let reduced = reduce_rest(&base, &CrewId::new("C1"))
        .expect("reduce_rest should succeed");
    let violations = check_roster(&reduced);
    assert!(!violations.is_empty());
    assert!(violations.iter().any(|v| v.rule_id == "minimum_rest"), "MinimumRestRule expected");
}

#[test]
fn test_determinism() {
    let base = canonical_roster();
    let transformed1 = reassign_leg(
        &base,
        &CrewId::new("C1"),
        &CrewId::new("C2"),
        &FlightLegId::new("L1"),
    )
    .expect("first transformation");
    let transformed2 = reassign_leg(
        &base,
        &CrewId::new("C1"),
        &CrewId::new("C2"),
        &FlightLegId::new("L1"),
    )
    .expect("second transformation");
    assert_eq!(transformed1, transformed2, "Transformations must be deterministic");
}
