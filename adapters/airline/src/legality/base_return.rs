//! Base return rule.
//!
//! Checks that every pairing in every rotation starts and ends at the crew
//! member's declared home base airport.
//!
//! Note: [`Pairing::new`] already enforces that a pairing starts and ends at
//! its declared `base` field.  This rule cross-checks that the pairing's
//! `base` matches the crew member's home base, catching mismatches that can
//! arise when pairings are assembled from external data.

use crate::domain::roster::Roster;
use crate::legality::{EntityRef, LegalityRule, LegalityViolation, ViolationSeverity};

/// Rule ID for [`BaseReturnRule`].
pub const RULE_ID: &str = "base_return";

/// Checks that every pairing starts and ends at the crew member's home base.
pub struct BaseReturnRule;

impl LegalityRule for BaseReturnRule {
    fn rule_id(&self) -> &str {
        RULE_ID
    }

    fn rule_name(&self) -> &str {
        "Base Return"
    }

    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        let mut violations = Vec::new();

        for rotation in roster.rotations() {
            let crew = roster.crew_member(&rotation.crew_id);

            for pairing in rotation.pairings() {
                match crew {
                    None => {
                        // Can't check base without crew record — emit warning.
                        violations.push(LegalityViolation::new(
                            RULE_ID,
                            ViolationSeverity::Warning,
                            EntityRef::Pairing(pairing.id.as_str().to_string()),
                            0.0,
                            0.0,
                            format!(
                                "Crew member {} not found; cannot verify base return \
                                 for pairing {}",
                                rotation.crew_id, pairing.id,
                            ),
                        ));
                    }
                    Some(member) => {
                        // Check that pairing base matches crew home base.
                        if pairing.base != member.base {
                            violations.push(LegalityViolation::new(
                                RULE_ID,
                                ViolationSeverity::Error,
                                EntityRef::Pairing(pairing.id.as_str().to_string()),
                                0.0,
                                0.0,
                                format!(
                                    "Pairing {} base is {} but crew member {} home base is {}",
                                    pairing.id, pairing.base, member.id, member.base,
                                ),
                            ));
                        }
                    }
                }
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

    fn rule() -> BaseReturnRule {
        BaseReturnRule
    }

    // ── Positive: pairing base matches crew base ───────────────────────────────

    #[test]
    fn matching_base_no_violation() {
        let crew = make_crew("C1", "LHR", &["B738"]);
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]); // base = LHR
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster_with_crew(vec![], vec![rotation], vec![crew]);
        assert!(rule().check(&roster).is_empty());
    }

    // ── Violation: pairing base differs from crew base ────────────────────────

    #[test]
    fn mismatched_base_produces_error() {
        let crew = make_crew("C1", "CDG", &["B738"]); // crew based at CDG
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]); // pairing base = LHR
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster_with_crew(vec![], vec![rotation], vec![crew]);
        let violations = rule().check(&roster);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, RULE_ID);
        assert!(violations[0].is_error());
    }

    // ── Boundary: multiple pairings, one mismatched ───────────────────────────

    #[test]
    fn one_of_two_pairings_mismatched() {
        let crew = make_crew("C1", "LHR", &["B738"]);
        // P1: LHR-based (ok)
        let p1 = make_pairing(
            "P1",
            "LHR",
            vec![
                make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]),
                make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]),
            ],
        );
        // P2: CDG-based (mismatch — crew is LHR)
        let p2 = make_pairing(
            "P2",
            "CDG",
            vec![
                make_duty("D3", vec![make_leg("L3", "CDG", "FRA", 30, 32)]),
                make_duty("D4", vec![make_leg("L4", "FRA", "CDG", 44, 46)]),
            ],
        );
        let rotation = make_rotation("R1", "C1", vec![p1, p2]);
        let roster = make_roster_with_crew(vec![], vec![rotation], vec![crew]);
        let violations = rule().check(&roster);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].is_error());
    }

    // ── Warning: crew record missing ──────────────────────────────────────────

    #[test]
    fn missing_crew_record_produces_warning() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster_with_crew(vec![], vec![rotation], vec![]); // no crew
        let violations = rule().check(&roster);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].is_warning());
    }

    #[test]
    fn rule_id_is_correct() {
        assert_eq!(rule().rule_id(), RULE_ID);
        assert_eq!(rule().rule_name(), "Base Return");
    }

    #[test]
    fn empty_roster_no_violations() {
        let roster = make_roster_with_crew(vec![], vec![], vec![]);
        assert!(rule().check(&roster).is_empty());
    }
}
