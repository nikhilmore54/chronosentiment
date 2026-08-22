use coralys_airline::evaluation::evaluate_roster;
use coralys_airline::legality::{
    LegalityChecker,
    base_return,
    coverage,
    duty_connectivity,
    duty_time,
    fdp,
    minimum_rest,
    qualification,
};

#[path = "fixtures/roster_fixtures.rs"]
mod roster_fixtures;
#[path = "fixtures/transformations.rs"]
mod transformations;

fn configured_checker() -> LegalityChecker {
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

#[test]
fn test_legal_roster_evaluation() {
    let roster = roster_fixtures::canonical_roster();
    let checker = configured_checker();

    let report = evaluate_roster(&roster, &checker);
    
    assert!(report.legal, "Canonical roster should be legal");
    assert_eq!(report.rules.len(), 7, "Should produce exactly 7 RuleSummary entries");
    
    // Check that every rule appears with zero violations
    for summary in &report.rules {
        assert_eq!(summary.violations.len(), 0, "Rule {} should have 0 violations", summary.rule_id);
    }
}

#[test]
fn test_reduce_rest_produces_violation() {
    use coralys_airline::domain::CrewId;
    let roster = roster_fixtures::canonical_roster();
    let roster = transformations::reduce_rest(&roster, &CrewId::new("C1")).expect("Transformation should succeed");
    let checker = configured_checker();

    let report = evaluate_roster(&roster, &checker);
    
    assert!(!report.legal, "Roster with reduced rest should be illegal");
    
    let min_rest_summary = report.rules.iter().find(|r| r.rule_id == "minimum_rest").unwrap();
    assert!(min_rest_summary.violations.len() > 0, "Should have a minimum_rest violation");
    assert!(min_rest_summary.violations[0].is_error(), "Should be an Error severity violation");
}

#[test]
fn test_rule_ordering_is_deterministic() {
    let roster = roster_fixtures::canonical_roster();
    let checker = configured_checker();
    let report = evaluate_roster(&roster, &checker);
    
    let expected_order = vec![
        "coverage",
        "duty_connectivity",
        "max_duty_time",
        "flight_duty_period",
        "minimum_rest",
        "qualification",
        "base_return"
    ];
    
    let actual_order: Vec<String> = report.rules.iter().map(|r| r.rule_id.clone()).collect();
    assert_eq!(actual_order, expected_order);
}

#[test]
fn test_repeated_evaluation_is_identical() {
    let roster = roster_fixtures::canonical_roster();
    let checker = configured_checker();
    
    let report1 = evaluate_roster(&roster, &checker);
    let report2 = evaluate_roster(&roster, &checker);
    
    assert_eq!(report1, report2, "Repeated evaluations should be identical");
}

#[test]
fn test_json_serialization_preserves_report() {
    let roster = roster_fixtures::canonical_roster();
    let checker = configured_checker();
    let report = evaluate_roster(&roster, &checker);
    
    let json = serde_json::to_string(&report).expect("Serialization failed");
    let deserialized: coralys_airline::evaluation::LegalityReport = serde_json::from_str(&json).expect("Deserialization failed");
    
    assert_eq!(report, deserialized, "JSON serialization should preserve report");
}
