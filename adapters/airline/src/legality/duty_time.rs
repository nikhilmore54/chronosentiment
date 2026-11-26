//! Maximum duty time rule.
//!
//! Checks that the elapsed time of every [`Duty`] (from first departure to
//! last arrival) does not exceed a configured maximum.
//!
//! The default limit is **14 hours** (840 minutes), a common regulatory
//! threshold for short-haul operations.  The limit is configurable at
//! construction time to support different fleet types and regulatory regimes.

use chrono::Duration;

use crate::domain::roster::Roster;
use crate::legality::{EntityRef, LegalityRule, LegalityViolation, ViolationSeverity};

/// Rule ID for [`MaximumDutyTimeRule`].
pub const RULE_ID: &str = "max_duty_time";

/// Default maximum duty time: 14 hours.
pub const DEFAULT_MAX_DUTY_MINUTES: i64 = 14 * 60;

/// Checks that no duty exceeds the configured maximum elapsed time.
pub struct MaximumDutyTimeRule {
    /// Maximum allowed elapsed duty time.
    max_duty_time: Duration,
}

impl MaximumDutyTimeRule {
    /// Create a new rule with the default 14-hour limit.
    pub fn new() -> Self {
        Self {
            max_duty_time: Duration::minutes(DEFAULT_MAX_DUTY_MINUTES),
        }
    }

    /// Create a new rule with a custom limit.
    pub fn with_limit(max_duty_time: Duration) -> Self {
        Self { max_duty_time }
    }

    /// The configured maximum duty time.
    pub fn max_duty_time(&self) -> Duration {
        self.max_duty_time
    }
}

impl Default for MaximumDutyTimeRule {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalityRule for MaximumDutyTimeRule {
    fn rule_id(&self) -> &str {
        RULE_ID
    }

    fn rule_name(&self) -> &str {
        "Maximum Duty Time"
    }

    fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        let mut violations = Vec::new();
        let limit_mins = self.max_duty_time.num_minutes() as f64;

        for rotation in roster.rotations() {
            for pairing in rotation.pairings() {
                for duty in pairing.duties() {
                    let elapsed_mins = duty.elapsed_time().num_minutes() as f64;
                    if elapsed_mins > limit_mins {
                        violations.push(LegalityViolation::new(
                            RULE_ID,
                            ViolationSeverity::Error,
                            EntityRef::Duty(duty.id.as_str().to_string()),
                            elapsed_mins,
                            limit_mins,
                            format!(
                                "Duty {} elapsed time {:.0} min exceeds maximum {:.0} min \
                                 (excess: {:.0} min)",
                                duty.id,
                                elapsed_mins,
                                limit_mins,
                                elapsed_mins - limit_mins,
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
    use crate::legality::test_helpers::*;

    fn rule_14h() -> MaximumDutyTimeRule {
        MaximumDutyTimeRule::new()
    }

    // ── Positive: duty within limit ───────────────────────────────────────────

    #[test]
    fn duty_within_limit_no_violation() {
        // 8h duty, limit 14h
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 12, 16);
        let duty = make_duty("D1", vec![l1, l2]);
        let pairing = make_pairing("P1", "LHR", vec![duty]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_14h().check(&roster).is_empty());
    }

    // ── Boundary: exactly at limit ────────────────────────────────────────────

    #[test]
    fn duty_exactly_at_limit_no_violation() {
        // 14h duty: departs 08:00, arrives 22:00
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 12, 22);
        let duty = make_duty("D1", vec![l1, l2]);
        let pairing = make_pairing("P1", "LHR", vec![duty]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        assert!(rule_14h().check(&roster).is_empty());
    }

    // ── Single violation: one minute over ─────────────────────────────────────

    #[test]
    fn duty_one_minute_over_limit_produces_error() {
        // 14h 1min duty: departs 08:00, arrives 22:01 (841 min)
        let base = crate::legality::test_helpers::base_time();
        use chrono::Duration;
        let dep = base + Duration::hours(8);
        let arr = base + Duration::hours(22) + Duration::minutes(1);
        use crate::domain::flight::{
            AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
        };
        let leg = FlightLeg::new(
            FlightLegId::new("L1"),
            FlightNumber::new("XX001"),
            AirportCode::new("LHR"),
            AirportCode::new("LHR"),
            dep,
            arr,
            AircraftType::new("B738"),
        );
        let duty = make_duty("D1", vec![leg]);
        let pairing = make_pairing("P1", "LHR", vec![duty]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let violations = rule_14h().check(&roster);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id, RULE_ID);
        assert!(violations[0].is_error());
        assert!((violations[0].observed - 841.0).abs() < 1.0);
        assert!((violations[0].threshold - 840.0).abs() < 1.0);
        assert!((violations[0].excess() - 1.0).abs() < 1.0);
    }

    // ── Multiple violations: two duties over limit ────────────────────────────

    #[test]
    fn two_duties_over_limit_produce_two_violations() {
        let base = crate::legality::test_helpers::base_time();
        use crate::domain::flight::{
            AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
        };
        use chrono::Duration;

        let make_long_leg = |id: &str, dep_h: i64, arr_h: i64| {
            FlightLeg::new(
                FlightLegId::new(id),
                FlightNumber::new(format!("XX{id}")),
                AirportCode::new("LHR"),
                AirportCode::new("LHR"),
                base + Duration::hours(dep_h),
                base + Duration::hours(arr_h),
                AircraftType::new("B738"),
            )
        };

        // D1: 15h (over), D2: 15h (over), separated by 2h rest
        let d1 = make_duty("D1", vec![make_long_leg("L1", 0, 15)]);
        let d2 = make_duty("D2", vec![make_long_leg("L2", 17, 32)]);
        let pairing = make_pairing("P1", "LHR", vec![d1, d2]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let violations = rule_14h().check(&roster);
        assert_eq!(violations.len(), 2);
    }

    // ── Custom limit ──────────────────────────────────────────────────────────

    #[test]
    fn custom_limit_respected() {
        // 10h duty, custom limit 8h → violation
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 12, 18);
        let duty = make_duty("D1", vec![l1, l2]);
        let pairing = make_pairing("P1", "LHR", vec![duty]);
        let rotation = make_rotation("R1", "C1", vec![pairing]);
        let roster = make_roster(vec![], vec![rotation]);
        let rule = MaximumDutyTimeRule::with_limit(Duration::hours(8));
        let violations = rule.check(&roster);
        assert_eq!(violations.len(), 1);
        assert!((violations[0].threshold - 480.0).abs() < 1.0);
    }

    #[test]
    fn rule_id_is_correct() {
        assert_eq!(rule_14h().rule_id(), RULE_ID);
    }

    #[test]
    fn empty_roster_no_violations() {
        let roster = make_roster(vec![], vec![]);
        assert!(rule_14h().check(&roster).is_empty());
    }
}
