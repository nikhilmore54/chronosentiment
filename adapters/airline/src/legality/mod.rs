//! Legality layer — Layer 2: Operational Correctness.
//!
//! This module provides the infrastructure for checking whether a [`Roster`]
//! satisfies operational, regulatory, and contractual constraints.
//!
//! # Architecture
//!
//! ```text
//! LegalityRule (trait)
//!     ├── DutyConnectivityRule   — legs within a duty connect geographically
//!     ├── MaximumDutyTimeRule    — elapsed duty time ≤ configured limit
//!     ├── MinimumRestRule        — rest between duties ≥ configured minimum
//!     ├── FlightDutyPeriodRule   — FDP (report to block-off of last leg) ≤ limit
//!     ├── QualificationRule      — crew hold type ratings for every leg
//!     ├── BaseReturnRule         — every pairing starts and ends at crew base
//!     └── CoverageRule           — every leg in the roster is assigned
//! ```
//!
//! Rules are registered with a [`LegalityChecker`], which runs them all and
//! aggregates [`LegalityViolation`]s.  Each violation carries enough
//! structured information to drive both API responses and planner UIs.

pub mod base_return;
pub mod coverage;
pub mod duty_connectivity;
pub mod duty_time;
pub mod fdp;
pub mod minimum_rest;
pub mod qualification;

use crate::domain::roster::Roster;
use serde::{Deserialize, Serialize};

// ── Severity ──────────────────────────────────────────────────────────────────

/// Severity of a legality violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Advisory — informational; does not make the schedule illegal.
    Advisory,
    /// Warning — should be resolved but does not strictly prevent operation.
    Warning,
    /// Error — makes the schedule operationally or legally invalid.
    Error,
}

impl std::fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationSeverity::Advisory => write!(f, "Advisory"),
            ViolationSeverity::Warning => write!(f, "Warning"),
            ViolationSeverity::Error => write!(f, "Error"),
        }
    }
}

// ── Entity reference ──────────────────────────────────────────────────────────

/// A reference to the scheduling entity that caused a violation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityRef {
    /// A specific flight leg.
    Leg(String),
    /// A specific duty.
    Duty(String),
    /// A specific pairing.
    Pairing(String),
    /// A specific rotation (crew member).
    Rotation {
        rotation_id: String,
        crew_id: String,
    },
    /// The roster as a whole.
    Roster(String),
}

impl std::fmt::Display for EntityRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityRef::Leg(id) => write!(f, "Leg({id})"),
            EntityRef::Duty(id) => write!(f, "Duty({id})"),
            EntityRef::Pairing(id) => write!(f, "Pairing({id})"),
            EntityRef::Rotation {
                rotation_id,
                crew_id,
            } => {
                write!(f, "Rotation({rotation_id}, crew={crew_id})")
            }
            EntityRef::Roster(id) => write!(f, "Roster({id})"),
        }
    }
}

// ── Violation ─────────────────────────────────────────────────────────────────

/// A structured legality violation.
///
/// Each violation carries enough information to:
/// - identify which rule fired (`rule_id`)
/// - classify the severity (`severity`)
/// - pinpoint the offending entity (`entity`)
/// - explain what was found vs. what was required (`observed`, `threshold`)
/// - provide a human-readable explanation (`message`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegalityViolation {
    /// Stable identifier of the rule that produced this violation.
    pub rule_id: String,
    /// Severity of this violation.
    pub severity: ViolationSeverity,
    /// The scheduling entity that caused the violation.
    pub entity: EntityRef,
    /// The observed value (e.g. actual duty time in minutes).
    pub observed: f64,
    /// The threshold or expected value (e.g. maximum duty time in minutes).
    pub threshold: f64,
    /// Human-readable explanation for planners.
    pub message: String,
}

impl LegalityViolation {
    /// Construct a new [`LegalityViolation`].
    pub fn new(
        rule_id: impl Into<String>,
        severity: ViolationSeverity,
        entity: EntityRef,
        observed: f64,
        threshold: f64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            entity,
            observed,
            threshold,
            message: message.into(),
        }
    }

    /// Convenience constructor for `Error`-severity violations.
    pub fn error(
        rule_id: impl Into<String>,
        entity: EntityRef,
        observed: f64,
        threshold: f64,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            rule_id,
            ViolationSeverity::Error,
            entity,
            observed,
            threshold,
            message,
        )
    }

    /// Convenience constructor for `Warning`-severity violations.
    pub fn warning(
        rule_id: impl Into<String>,
        entity: EntityRef,
        observed: f64,
        threshold: f64,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            rule_id,
            ViolationSeverity::Warning,
            entity,
            observed,
            threshold,
            message,
        )
    }

    /// Returns `true` if this violation has `Error` severity.
    pub fn is_error(&self) -> bool {
        self.severity == ViolationSeverity::Error
    }

    /// Returns `true` if this violation has `Warning` severity.
    pub fn is_warning(&self) -> bool {
        self.severity == ViolationSeverity::Warning
    }

    /// Excess: how much the observed value exceeds the threshold.
    /// Returns 0.0 if observed ≤ threshold.
    pub fn excess(&self) -> f64 {
        (self.observed - self.threshold).max(0.0)
    }
}

// ── Rule trait ────────────────────────────────────────────────────────────────

/// Trait for a single, narrowly-scoped legality rule.
///
/// # Contract
/// - **Pure**: no side effects, no mutation of the roster.
/// - **Deterministic**: same input → same output.
/// - **Independent**: no rule depends on another rule's output.
pub trait LegalityRule: Send + Sync {
    /// A short, stable identifier for this rule, e.g. `"max_duty_time"`.
    fn rule_id(&self) -> &str;

    /// A human-readable name for this rule.
    fn rule_name(&self) -> &str;

    /// Check the roster and return any violations found.
    fn check(&self, roster: &Roster) -> Vec<LegalityViolation>;
}

// ── Checker ───────────────────────────────────────────────────────────────────

/// Orchestrates a collection of [`LegalityRule`]s against a [`Roster`].
///
/// Rules are run in registration order.  All violations from all rules are
/// collected and returned together.
pub struct LegalityChecker {
    rules: Vec<Box<dyn LegalityRule>>,
}

impl LegalityChecker {
    /// Create a new, empty [`LegalityChecker`].
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Register a legality rule.
    pub fn add_rule(&mut self, rule: Box<dyn LegalityRule>) {
        self.rules.push(rule);
    }

    /// Run all registered rules against `roster` and return all violations.
    pub fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
        self.rules.iter().flat_map(|r| r.check(roster)).collect()
    }

    /// Run all rules and return only `Error`-severity violations.
    pub fn errors(&self, roster: &Roster) -> Vec<LegalityViolation> {
        self.check(roster)
            .into_iter()
            .filter(|v| v.is_error())
            .collect()
    }

    /// Returns `true` if the roster has no `Error`-severity violations.
    pub fn is_legal(&self, roster: &Roster) -> bool {
        self.errors(roster).is_empty()
    }

    /// Returns `true` if no rules are registered.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Number of registered rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// IDs of all registered rules, in registration order.
    pub fn rule_ids(&self) -> Vec<&str> {
        self.rules.iter().map(|r| r.rule_id()).collect()
    }
}

impl Default for LegalityChecker {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
pub(crate) mod test_helpers {
    //! Shared test helpers for legality rule tests.

    use crate::domain::crew::{CrewId, CrewMember, CrewRole, Qualification};
    use crate::domain::duty::{Duty, DutyId};
    use crate::domain::flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};
    use crate::domain::pairing::{Pairing, PairingId};
    use crate::domain::roster::{PlanningPeriod, Roster, RosterId};
    use crate::domain::rotation::{Rotation, RotationId};
    use chrono::{DateTime, Duration, TimeZone, Utc};

    pub fn base_time() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap()
    }

    pub fn make_leg(id: &str, origin: &str, dest: &str, dep_h: i64, arr_h: i64) -> FlightLeg {
        FlightLeg::new(
            FlightLegId::new(id),
            FlightNumber::new(format!("XX{id}")),
            AirportCode::new(origin),
            AirportCode::new(dest),
            base_time() + Duration::hours(dep_h),
            base_time() + Duration::hours(arr_h),
            AircraftType::new("B738"),
        )
    }

    pub fn make_leg_typed(
        id: &str,
        origin: &str,
        dest: &str,
        dep_h: i64,
        arr_h: i64,
        aircraft: &str,
    ) -> FlightLeg {
        FlightLeg::new(
            FlightLegId::new(id),
            FlightNumber::new(format!("XX{id}")),
            AirportCode::new(origin),
            AirportCode::new(dest),
            base_time() + Duration::hours(dep_h),
            base_time() + Duration::hours(arr_h),
            AircraftType::new(aircraft),
        )
    }

    pub fn make_duty(id: &str, legs: Vec<FlightLeg>) -> Duty {
        Duty::new(DutyId::new(id), legs).unwrap()
    }

    pub fn make_pairing(id: &str, base: &str, duties: Vec<Duty>) -> Pairing {
        Pairing::new(PairingId::new(id), AirportCode::new(base), duties).unwrap()
    }

    pub fn make_rotation(rotation_id: &str, crew_id: &str, pairings: Vec<Pairing>) -> Rotation {
        Rotation::new(RotationId::new(rotation_id), CrewId::new(crew_id), pairings).unwrap()
    }

    pub fn make_crew(id: &str, base: &str, aircraft_types: &[&str]) -> CrewMember {
        CrewMember::new(
            CrewId::new(id),
            format!("Crew {id}"),
            CrewRole::Captain,
            aircraft_types
                .iter()
                .map(|t| Qualification::new(AircraftType::new(*t)))
                .collect(),
            AirportCode::new(base),
        )
    }

    pub fn make_roster(legs: Vec<FlightLeg>, rotations: Vec<Rotation>) -> Roster {
        let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(30));
        Roster::new(RosterId::new("R1"), period, legs, rotations).unwrap()
    }

    pub fn make_roster_with_crew(
        legs: Vec<FlightLeg>,
        rotations: Vec<Rotation>,
        crew_members: Vec<crate::domain::crew::CrewMember>,
    ) -> Roster {
        let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(30));
        Roster::with_crew(RosterId::new("R1"), period, legs, rotations, crew_members).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_helpers::*;

    struct AlwaysErrors;
    impl LegalityRule for AlwaysErrors {
        fn rule_id(&self) -> &str {
            "always_errors"
        }
        fn rule_name(&self) -> &str {
            "Always Errors"
        }
        fn check(&self, _: &Roster) -> Vec<LegalityViolation> {
            vec![LegalityViolation::error(
                "always_errors",
                EntityRef::Roster("R1".into()),
                1.0,
                0.0,
                "stub error",
            )]
        }
    }

    struct AlwaysWarns;
    impl LegalityRule for AlwaysWarns {
        fn rule_id(&self) -> &str {
            "always_warns"
        }
        fn rule_name(&self) -> &str {
            "Always Warns"
        }
        fn check(&self, _: &Roster) -> Vec<LegalityViolation> {
            vec![LegalityViolation::warning(
                "always_warns",
                EntityRef::Roster("R1".into()),
                1.0,
                0.0,
                "stub warning",
            )]
        }
    }

    fn empty_roster() -> Roster {
        make_roster(vec![], vec![])
    }

    #[test]
    fn empty_checker_is_legal() {
        let checker = LegalityChecker::new();
        assert!(checker.is_legal(&empty_roster()));
        assert!(checker.is_empty());
    }

    #[test]
    fn error_rule_makes_roster_illegal() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(AlwaysErrors));
        assert!(!checker.is_legal(&empty_roster()));
        assert_eq!(checker.errors(&empty_roster()).len(), 1);
    }

    #[test]
    fn warning_does_not_make_roster_illegal() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(AlwaysWarns));
        assert!(checker.is_legal(&empty_roster()));
        assert_eq!(checker.check(&empty_roster()).len(), 1);
    }

    #[test]
    fn rule_ids_in_registration_order() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(AlwaysErrors));
        checker.add_rule(Box::new(AlwaysWarns));
        assert_eq!(checker.rule_ids(), vec!["always_errors", "always_warns"]);
    }

    #[test]
    fn violation_excess() {
        let v = LegalityViolation::error("r", EntityRef::Duty("D1".into()), 14.0, 12.0, "");
        assert!((v.excess() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn violation_excess_zero_when_within_threshold() {
        let v = LegalityViolation::error("r", EntityRef::Duty("D1".into()), 10.0, 12.0, "");
        assert_eq!(v.excess(), 0.0);
    }
}
