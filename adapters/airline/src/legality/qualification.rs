//! Qualification rule.
//!
//! Checks that every crew member holds a type rating for the aircraft type
//! required by each flight leg in their rotation.
//!
//! This rule requires the roster to carry [`CrewMember`] records alongside
//! the rotations.  If a crew member record is not found in the roster, the
//! rule emits a warning (missing data is not treated as a hard error, since
//! the roster may be partially populated during planning).

use crate::domain::roster::Roster;
use crate::legality::{EntityRef, LegalityRule, LegalityViolation, ViolationSeverity};

/// Rule ID for [`QualificationRule`].
pub const RULE_ID: &str = "qualification";

/// Checks that crew members hold type ratings for every leg they operate.
pub struct QualificationRule;

impl LegalityRule for QualificationRule {
    fn rule_id(&self) -> &str {
        RULE_ID
    }

    fn rule_name(&self) -> &str {
        "Crew Qualification"
    }

    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        let mut violations = Vec::new();

        for rotation in roster.rotations() {
            // Look up the crew member record.
            let crew = roster.crew_member(&rotation.crew_id);

            for pairing in rotation.pairings() {
                for duty in pairing.duties() {
                    for leg in duty.legs() {
                        match crew {
                            None => {
                                // Crew record missing — emit a warning.
                                violations.push(LegalityViolation::new(
                                    RULE_ID,
                                    ViolationSeverity::Warning,
                                    EntityRef::Leg(leg.id.as_str().to_string()),
                                    0.0,
                                    0.0,
                                    format!(
                                        "Crew member {} not found in roster; \
                                         cannot verify qualification for leg {} ({})",
                                        rotation.crew_id, leg.id, leg.aircraft_type,
                                    ),
                                ));
                            }
                            Some(member) => {
                                if !member.is_qualified_for(&leg.aircraft_type) {
                                    violations.push(LegalityViolation::new(
                                        RULE_ID,
                                        ViolationSeverity::Error,
                                        EntityRef::Leg(leg.id.as_str().to_string()),
                                        0.0,
                                        1.0,
                                        format!(
                                            "Crew member {} ({}) is not qualified for \
                                             aircraft type {} required by leg {}",
                                            member.id, member.name, leg.aircraft_type, leg.id,
                                        ),
                                    ));
                                }
                            }
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

    fn rule() -> QualificationRule {
        QualificationRule
    }

    // ── Positive: crew qualified for all legs ─────────────────────────────────

    #[test]
    fn qualified_crew_no_violation() {
        let crew = make_crew("C1", "LHR", &["B738"]);
        let leg = make_leg("L1", "LHR", "CDG", 8, 10);
        let duty = make_duty("D1", vec![leg.clone()]);
        let pairing = make_pairing(
            "P1",
            "LHR",
            vec![
                duty,
                make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]),
            ],
        );
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster_with_crew(
            vec![leg, make_leg("L2", "CDG", "LHR", 22, 24)],
            vec![rotation],
            vec![crew],
        );
        assert!(rule().check(&roster).is_empty());
    }

    // ── Violation: crew not qualified ─────────────────────────────────────────

    #[test]
    fn unqualified_crew_produces_error() {
        // Crew qualified for A320 only, but leg requires B738
        let crew = make_crew("C1", "LHR", &["A320"]);
        let leg = make_leg("L1", "LHR", "CDG", 8, 10); // B738 (default)
        let duty = make_duty("D1", vec![leg.clone()]);
        let pairing = make_pairing(
            "P1",
            "LHR",
            vec![
                duty,
                make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]),
            ],
        );
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster_with_crew(
            vec![leg, make_leg("L2", "CDG", "LHR", 22, 24)],
            vec![rotation],
            vec![crew],
        );
        let violations = rule().check(&roster);
        assert_eq!(violations.len(), 2); // both legs are B738
        for v in &violations {
            assert_eq!(v.rule_id, RULE_ID);
            assert!(v.is_error());
        }
    }

    // ── Boundary: crew qualified for one type, leg uses another ───────────────

    #[test]
    fn mixed_qualification_partial_violation() {
        // Crew qualified for B738 only; one leg B738 (ok), one leg A320 (violation)
        let crew = make_crew("C1", "LHR", &["B738"]);
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10); // B738 — ok
        let l2 = make_leg_typed("L2", "CDG", "LHR", 22, 24, "A320"); // A320 — violation
        let d1 = make_duty("D1", vec![l1.clone()]);
        let d2 = make_duty("D2", vec![l2.clone()]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster_with_crew(vec![l1, l2], vec![rotation], vec![crew]);
        let violations = rule().check(&roster);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].is_error());
    }

    // ── Warning: crew member not in roster ────────────────────────────────────

    #[test]
    fn missing_crew_record_produces_warning() {
        // Rotation references C1 but no CrewMember record for C1 in roster
        let leg = make_leg("L1", "LHR", "CDG", 8, 10);
        let duty = make_duty("D1", vec![leg.clone()]);
        let pairing = make_pairing(
            "P1",
            "LHR",
            vec![
                duty,
                make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]),
            ],
        );
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        // No crew members passed — roster has no CrewMember records
        let roster = make_roster_with_crew(
            vec![leg, make_leg("L2", "CDG", "LHR", 22, 24)],
            vec![rotation],
            vec![],
        );
        let violations = rule().check(&roster);
        assert!(!violations.is_empty());
        for v in &violations {
            assert!(v.is_warning());
        }
    }

    // ── Multiple crew, mixed qualifications ───────────────────────────────────

    #[test]
    fn two_crew_one_qualified_one_not() {
        let c1 = make_crew("C1", "LHR", &["B738"]);
        let c2 = make_crew("C2", "LHR", &["A320"]); // wrong type

        let make_b738_pairing = |pid: &str, rid: &str, cid: &str| {
            let d1 = make_duty(
                &format!("{rid}D1"),
                vec![make_leg(&format!("{rid}L1"), "LHR", "CDG", 8, 10)],
            );
            let d2 = make_duty(
                &format!("{rid}D2"),
                vec![make_leg(&format!("{rid}L2"), "CDG", "LHR", 22, 24)],
            );
            let p = make_pairing(pid, "LHR", vec![d1, d2]);
            make_rotation(rid, cid, vec![p])
        };

        let r1 = make_b738_pairing("P1", "R1", "C1");
        let r2 = make_b738_pairing("P2", "R2", "C2");
        let roster = make_roster_with_crew(vec![], vec![r1, r2], vec![c1, c2]);
        let violations = rule().check(&roster);
        // C2 has 2 legs, both B738 → 2 violations
        assert_eq!(violations.len(), 2);
        for v in &violations {
            assert!(v.is_error());
        }
    }

    #[test]
    fn rule_id_is_correct() {
        assert_eq!(rule().rule_id(), RULE_ID);
        assert_eq!(rule().rule_name(), "Crew Qualification");
    }

    #[test]
    fn empty_roster_no_violations() {
        let roster = make_roster_with_crew(vec![], vec![], vec![]);
        assert!(rule().check(&roster).is_empty());
    }
}
