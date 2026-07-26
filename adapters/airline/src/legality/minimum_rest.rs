//! Minimum rest rule.
//!
//! Checks that the rest period between consecutive [`Duty`]s within every
//! [`Pairing`] meets a configured minimum.
//!
//! The default minimum is **10 hours** (600 minutes), a common regulatory
//! floor for short-haul operations.  The limit is configurable at construction
//! time to support different fleet types and regulatory regimes.

use chrono::Duration;

use crate::domain::roster::Roster;
use crate::legality::{EntityRef, LegalityRule, LegalityViolation, ViolationSeverity};

/// Rule ID for [`MinimumRestRule`].
pub const RULE_ID: &str = "minimum_rest";

/// Default minimum rest between duties: 10 hours.
pub const DEFAULT_MIN_REST_MINUTES: i64 = 10 * 60;

/// Checks that the rest between consecutive duties within a pairing meets the
/// configured minimum.
pub struct MinimumRestRule {
    /// Minimum required rest duration.
    min_rest: Duration,
}

impl MinimumRestRule {
    /// Create a new rule with the default 10-hour minimum.
    pub fn new() -> Self {
        Self {
            min_rest: Duration::minutes(DEFAULT_MIN_REST_MINUTES),
        }
    }

    /// Create a new rule with a custom minimum rest duration.
    pub fn with_minimum(min_rest: Duration) -> Self {
        Self { min_rest }
    }

    /// The configured minimum rest duration.
    pub fn min_rest(&self) -> Duration {
        self.min_rest
    }
}

impl Default for MinimumRestRule {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalityRule for MinimumRestRule {
    fn rule_id(&self) -> &str {
        RULE_ID
    }

    fn rule_name(&self) -> &str {
        "Minimum Rest Between Duties"
    }

    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        let mut violations = Vec::new();
        let min_mins = self.min_rest.num_minutes() as f64;

        for rotation in roster.rotations() {
            for pairing in rotation.pairings() {
                for (duty_idx, rest) in pairing.rest_periods() {
                    let rest_mins = rest.num_minutes() as f64;
                    if rest_mins < min_mins {
                        let duties = pairing.duties();
                        let after_duty = &duties[duty_idx];
                        violations.push(LegalityViolation::new(
                            RULE_ID,
                            ViolationSeverity::Error,
                            EntityRef::Pairing(pairing.id.as_str().to_string()),
                            rest_mins,
                            min_mins,
                            format!(
                                "Pairing {}: rest before duty {} is {:.0} min, \
                                 minimum required is {:.0} min (shortfall: {:.0} min)",
                                pairing.id,
                                after_duty.id,
                                rest_mins,
                                min_mins,
                                min_mins - rest_mins,
                            ),
                        ));
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
    use crate::domain::flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};
    use crate::legality::test_helpers::*;

    fn rule_10h() -> MinimumRestRule {
        MinimumRestRule::new()
    }

    // ── Positive: rest exceeds minimum ────────────────────────────────────────

    #[test]
    fn adequate_rest_no_violation() {
        // D1 ends at 10h, D2 starts at 22h → 12h rest (> 10h minimum)
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_10h().check(&roster).is_empty());
    }

    // ── Boundary: exactly at minimum ──────────────────────────────────────────

    #[test]
    fn rest_exactly_at_minimum_no_violation() {
        // D1 ends at 10h, D2 starts at 20h → exactly 10h rest
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 20, 22)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_10h().check(&roster).is_empty());
    }

    // ── Single violation: one minute short ────────────────────────────────────

    #[test]
    fn rest_one_minute_short_produces_error() {
        // D1 ends at 10h, D2 starts at 10h + 599min = 19h59 → 599 min rest
        let base = base_time();
        let dep2 = base + Duration::hours(10) + Duration::minutes(599);
        let arr2 = base + Duration::hours(22);
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let l2 = FlightLeg::new(
            FlightLegId::new("L2"),
            FlightNumber::new("XX002"),
            AirportCode::new("CDG"),
            AirportCode::new("LHR"),
            dep2,
            arr2,
            AircraftType::new("B738"),
        );
        let d2 = make_duty("D2", vec![l2]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let violations = rule_10h().check(&roster);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, RULE_ID);
        assert!(violations[0].is_error());
        assert!((violations[0].observed - 599.0).abs() < 1.0);
        assert!((violations[0].threshold - 600.0).abs() < 1.0);
    }

    // ── Multiple violations: two short rests ──────────────────────────────────

    #[test]
    fn two_short_rests_produce_two_violations() {
        // Three duties with 5h rest between each (< 10h minimum)
        // D1: 0h–2h, D2: 7h–9h (5h rest), D3: 14h–16h (5h rest)
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 0, 2)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "FRA", 7, 9)]);
        let d3 = make_duty("D3", vec![make_leg("L3", "FRA", "LHR", 14, 16)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2, d3]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let violations = rule_10h().check(&roster);
        assert_eq!(violations.len(), 2);
        for v in &violations {
            assert_eq!(v.rule_id, RULE_ID);
            assert!(v.is_error());
            assert!((v.observed - 300.0).abs() < 1.0); // 5h = 300 min
            assert!((v.threshold - 600.0).abs() < 1.0);
        }
    }

    // ── Custom minimum ────────────────────────────────────────────────────────

    #[test]
    fn custom_minimum_respected() {
        // 8h rest, custom minimum 12h → violation
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 18, 20)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let rule = MinimumRestRule::with_minimum(Duration::hours(12));
        let violations = rule.check(&roster);
        assert_eq!(violations.len(), 1);
        assert!((violations[0].threshold - 720.0).abs() < 1.0);
        assert!((violations[0].observed - 480.0).abs() < 1.0);
    }

    #[test]
    fn single_duty_pairing_has_no_rest_periods() {
        // A pairing with one duty has no rest periods to check
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "LHR", 8, 10)]);
        let pairing = make_pairing("P1", "LHR", vec![d1]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_10h().check(&roster).is_empty());
    }

    #[test]
    fn rule_id_is_correct() {
        assert_eq!(rule_10h().rule_id(), RULE_ID);
        assert_eq!(rule_10h().rule_name(), "Minimum Rest Between Duties");
    }

    #[test]
    fn empty_roster_no_violations() {
        let roster = make_roster(vec![], vec![]);
        assert!(rule_10h().check(&roster).is_empty());
    }
}
