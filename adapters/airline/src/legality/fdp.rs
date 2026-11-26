//! Flight Duty Period (FDP) rule.
//!
//! The Flight Duty Period is the time from when a crew member reports for duty
//! to the block-off of the last flight leg in that duty.  It is distinct from
//! elapsed duty time (which runs to block-on of the last leg) and is the
//! primary regulatory limit in many jurisdictions (e.g. EU-OPS, FAR Part 117).
//!
//! Default FDP limit: **13 hours** (780 minutes).

use chrono::Duration;

use crate::domain::roster::Roster;
use crate::legality::{EntityRef, LegalityRule, LegalityViolation, ViolationSeverity};

/// Rule ID for [`FlightDutyPeriodRule`].
pub const RULE_ID: &str = "flight_duty_period";

/// Default maximum FDP: 13 hours.
pub const DEFAULT_MAX_FDP_MINUTES: i64 = 13 * 60;

/// Checks that the Flight Duty Period of every duty does not exceed the limit.
///
/// FDP = time from first scheduled departure to last scheduled departure
/// (block-off of the last leg).  This approximates the regulatory definition
/// where the FDP ends at block-off of the last sector.
pub struct FlightDutyPeriodRule {
    /// Maximum allowed FDP.
    max_fdp: Duration,
}

impl FlightDutyPeriodRule {
    /// Create a new rule with the default 13-hour FDP limit.
    pub fn new() -> Self {
        Self {
            max_fdp: Duration::minutes(DEFAULT_MAX_FDP_MINUTES),
        }
    }

    /// Create a new rule with a custom FDP limit.
    pub fn with_limit(max_fdp: Duration) -> Self {
        Self { max_fdp }
    }

    /// The configured maximum FDP.
    pub fn max_fdp(&self) -> Duration {
        self.max_fdp
    }
}

impl Default for FlightDutyPeriodRule {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalityRule for FlightDutyPeriodRule {
    fn rule_id(&self) -> &str {
        RULE_ID
    }

    fn rule_name(&self) -> &str {
        "Flight Duty Period"
    }

    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        let mut violations = Vec::new();
        let limit_mins = self.max_fdp.num_minutes() as f64;

        for rotation in roster.rotations() {
            for pairing in rotation.pairings() {
                for duty in pairing.duties() {
                    let legs = duty.legs();
                    if legs.is_empty() {
                        continue;
                    }
                    // FDP: first departure → last departure (block-off of last leg)
                    let fdp_start = legs[0].scheduled_departure;
                    let fdp_end = legs[legs.len() - 1].scheduled_departure;
                    let fdp_mins = (fdp_end - fdp_start).num_minutes() as f64;

                    if fdp_mins > limit_mins {
                        violations.push(LegalityViolation::new(
                            RULE_ID,
                            ViolationSeverity::Error,
                            EntityRef::Duty(duty.id.as_str().to_string()),
                            fdp_mins,
                            limit_mins,
                            format!(
                                "Duty {}: FDP {:.0} min exceeds maximum {:.0} min \
                                 (excess: {:.0} min)",
                                duty.id,
                                fdp_mins,
                                limit_mins,
                                fdp_mins - limit_mins,
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

    fn rule_13h() -> FlightDutyPeriodRule {
        FlightDutyPeriodRule::new()
    }

    // ── Positive: FDP within limit ────────────────────────────────────────────

    #[test]
    fn fdp_within_limit_no_violation() {
        // Two legs: dep 08:00, dep 14:00 → FDP = 6h (< 13h)
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 14, 16);
        let duty = make_duty("D1", vec![l1, l2]);
        let pairing = make_pairing("P1", "LHR", vec![duty]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_13h().check(&roster).is_empty());
    }

    // ── Boundary: exactly at limit ────────────────────────────────────────────

    #[test]
    fn fdp_exactly_at_limit_no_violation() {
        // dep 08:00, dep 21:00 → FDP = 13h exactly
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 21, 23);
        let duty = make_duty("D1", vec![l1, l2]);
        let pairing = make_pairing("P1", "LHR", vec![duty]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_13h().check(&roster).is_empty());
    }

    // ── Single violation: one minute over ─────────────────────────────────────

    #[test]
    fn fdp_one_minute_over_produces_error() {
        // dep 08:00, dep 21:01 → FDP = 781 min
        let base = base_time();
        let dep1 = base + Duration::hours(8);
        let arr1 = base + Duration::hours(10);
        let dep2 = base + Duration::hours(21) + Duration::minutes(1);
        let arr2 = base + Duration::hours(23);
        let l1 = FlightLeg::new(
            FlightLegId::new("L1"),
            FlightNumber::new("XX001"),
            AirportCode::new("LHR"),
            AirportCode::new("CDG"),
            dep1,
            arr1,
            AircraftType::new("B738"),
        );
        let l2 = FlightLeg::new(
            FlightLegId::new("L2"),
            FlightNumber::new("XX002"),
            AirportCode::new("CDG"),
            AirportCode::new("LHR"),
            dep2,
            arr2,
            AircraftType::new("B738"),
        );
        let duty = make_duty("D1", vec![l1, l2]);
        let pairing = make_pairing("P1", "LHR", vec![duty]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let violations = rule_13h().check(&roster);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, RULE_ID);
        assert!(violations[0].is_error());
        assert!((violations[0].observed - 781.0).abs() < 1.0);
        assert!((violations[0].threshold - 780.0).abs() < 1.0);
        assert!((violations[0].excess() - 1.0).abs() < 1.0);
    }

    // ── Single-leg duty: FDP = 0 ──────────────────────────────────────────────

    #[test]
    fn single_leg_duty_fdp_is_zero() {
        // Single leg: dep == dep → FDP = 0
        let duty = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let pairing = make_pairing(
            "P1",
            "LHR",
            vec![
                duty,
                make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 22, 24)]),
            ],
        );
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_13h().check(&roster).is_empty());
    }

    // ── Multiple violations ───────────────────────────────────────────────────

    #[test]
    fn two_duties_over_fdp_produce_two_violations() {
        let base = base_time();
        // D1: dep 0h, dep 14h → FDP 14h (over 13h)
        let l1a = FlightLeg::new(
            FlightLegId::new("L1a"),
            FlightNumber::new("XX1a"),
            AirportCode::new("LHR"),
            AirportCode::new("CDG"),
            base,
            base + Duration::hours(2),
            AircraftType::new("B738"),
        );
        let l1b = FlightLeg::new(
            FlightLegId::new("L1b"),
            FlightNumber::new("XX1b"),
            AirportCode::new("CDG"),
            AirportCode::new("LHR"),
            base + Duration::hours(14),
            base + Duration::hours(16),
            AircraftType::new("B738"),
        );
        // D2: dep 30h, dep 44h → FDP 14h (over 13h)
        let l2a = FlightLeg::new(
            FlightLegId::new("L2a"),
            FlightNumber::new("XX2a"),
            AirportCode::new("LHR"),
            AirportCode::new("CDG"),
            base + Duration::hours(30),
            base + Duration::hours(32),
            AircraftType::new("B738"),
        );
        let l2b = FlightLeg::new(
            FlightLegId::new("L2b"),
            FlightNumber::new("XX2b"),
            AirportCode::new("CDG"),
            AirportCode::new("LHR"),
            base + Duration::hours(44),
            base + Duration::hours(46),
            AircraftType::new("B738"),
        );
        let d1 = make_duty("D1", vec![l1a, l1b]);
        let d2 = make_duty("D2", vec![l2a, l2b]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let violations = rule_13h().check(&roster);
        assert_eq!(violations.len(), 2);
    }

    #[test]
    fn rule_id_is_correct() {
        assert_eq!(rule_13h().rule_id(), RULE_ID);
        assert_eq!(rule_13h().rule_name(), "Flight Duty Period");
    }

    #[test]
    fn empty_roster_no_violations() {
        let roster = make_roster(vec![], vec![]);
        assert!(rule_13h().check(&roster).is_empty());
    }
}
