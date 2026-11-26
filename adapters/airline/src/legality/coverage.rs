//! Coverage rule.
//!
//! Checks that every [`FlightLeg`] declared in the roster appears in at least
//! one rotation.  An uncovered leg means no crew has been assigned to operate
//! it, which is an operational error.
//!
//! The rule also detects over-coverage (a leg assigned to more than one
//! rotation) and emits a warning, since over-coverage may indicate a data
//! error or a deliberate augmented-crew arrangement that should be reviewed.

use std::collections::HashMap;

use crate::domain::flight::FlightLegId;
use crate::domain::roster::Roster;
use crate::legality::{EntityRef, LegalityRule, LegalityViolation, ViolationSeverity};

/// Rule ID for [`CoverageRule`].
pub const RULE_ID: &str = "coverage";

/// Checks that every leg in the roster is assigned to exactly one rotation.
pub struct CoverageRule;

impl LegalityRule for CoverageRule {
    fn rule_id(&self) -> &str {
        RULE_ID
    }

    fn rule_name(&self) -> &str {
        "Leg Coverage"
    }

    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        let mut violations = Vec::new();

        // Count how many rotations each leg appears in.
        let mut assignment_count: HashMap<FlightLegId, usize> = HashMap::new();

        for rotation in roster.rotations() {
            for pairing in rotation.pairings() {
                for duty in pairing.duties() {
                    for leg in duty.legs() {
                        *assignment_count.entry(leg.id.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Check every declared leg.
        for leg in roster.legs() {
            let count = assignment_count.get(&leg.id).copied().unwrap_or(0);

            if count == 0 {
                violations.push(LegalityViolation::new(
                    RULE_ID,
                    ViolationSeverity::Error,
                    EntityRef::Leg(leg.id.as_str().to_string()),
                    0.0,
                    1.0,
                    format!(
                        "Leg {} ({} → {}) is not assigned to any rotation",
                        leg.id, leg.origin, leg.destination,
                    ),
                ));
            } else if count > 1 {
                violations.push(LegalityViolation::new(
                    RULE_ID,
                    ViolationSeverity::Warning,
                    EntityRef::Leg(leg.id.as_str().to_string()),
                    count as f64,
                    1.0,
                    format!(
                        "Leg {} ({} → {}) is assigned to {} rotations (expected 1)",
                        leg.id, leg.origin, leg.destination, count,
                    ),
                ));
            }
        }

        violations
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::test_helpers::*;

    fn rule() -> CoverageRule {
        CoverageRule
    }

    // ── Positive: all legs covered exactly once ───────────────────────────────

    #[test]
    fn all_legs_covered_no_violation() {
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 22, 24);
        let d1 = make_duty("D1", vec![l1.clone()]);
        let d2 = make_duty("D2", vec![l2.clone()]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![l1, l2], vec![rotation]);
        assert!(rule().check(&roster).is_empty());
    }

    // ── Violation: uncovered leg ──────────────────────────────────────────────

    #[test]
    fn uncovered_leg_produces_error() {
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 22, 24);
        let l_uncovered = make_leg("L3", "LHR", "AMS", 12, 14); // not in any rotation
        let d1 = make_duty("D1", vec![l1.clone()]);
        let d2 = make_duty("D2", vec![l2.clone()]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![l1, l2, l_uncovered], vec![rotation]);
        let violations = rule().check(&roster);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, RULE_ID);
        assert!(violations[0].is_error());
        assert!(violations[0].message.contains("L3"));
    }

    // ── Warning: over-covered leg ─────────────────────────────────────────────

    #[test]
    fn over_covered_leg_produces_warning() {
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 22, 24);
        // Both rotations include L1 (over-coverage)
        let d1a = make_duty("D1a", vec![l1.clone()]);
        let d1b = make_duty("D1b", vec![l1.clone()]); // same leg, second rotation
        let d2a = make_duty("D2a", vec![l2.clone()]);
        let d2b = make_duty("D2b", vec![l2.clone()]);
        let p1 = make_pairing("P1", "LHR", vec![d1a, d2a]);
        let p2 = make_pairing("P2", "LHR", vec![d1b, d2b]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        let roster = make_roster(vec![l1, l2], vec![r1, r2]);
        let violations = rule().check(&roster);
        // Both L1 and L2 are over-covered → 2 warnings
        assert_eq!(violations.len(), 2);
        for v in &violations {
            assert!(v.is_warning());
            assert!((v.observed - 2.0).abs() < 1e-9);
        }
    }

    // ── Boundary: empty roster legs, rotations have legs ─────────────────────

    #[test]
    fn no_declared_legs_no_violations() {
        // Roster declares no legs — nothing to check for coverage
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]); // no declared legs
        assert!(rule().check(&roster).is_empty());
    }

    // ── Multiple uncovered legs ───────────────────────────────────────────────

    #[test]
    fn two_uncovered_legs_produce_two_errors() {
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 22, 24);
        // Roster declares both legs but no rotations
        let roster = make_roster(vec![l1, l2], vec![]);
        let violations = rule().check(&roster);
        assert_eq!(violations.len(), 2);
        for v in &violations {
            assert!(v.is_error());
        }
    }

    #[test]
    fn rule_id_is_correct() {
        assert_eq!(rule().rule_id(), RULE_ID);
        assert_eq!(rule().rule_name(), "Leg Coverage");
    }

    #[test]
    fn empty_roster_no_violations() {
        let roster = make_roster(vec![], vec![]);
        assert!(rule().check(&roster).is_empty());
    }
}
