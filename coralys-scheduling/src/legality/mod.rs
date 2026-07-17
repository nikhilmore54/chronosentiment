//! Legality layer — Layer 2 (stub).
//!
//! This module will contain operational correctness rules for the scheduling
//! domain: duty-time limits, minimum rest requirements, qualification checks,
//! and other regulatory or contractual constraints.
//!
//! # Status
//! **Stub only.**  The trait interface is defined here so that Layer 1 domain
//! entities can reference it by type, but no rules are implemented yet.
//! Implementation begins in Milestone 5 — Layer 2.
//!
//! # Planned contents
//! - `DutyTimeLimitRule` — maximum elapsed duty time per duty
//! - `MinimumRestRule` — minimum rest between consecutive duties
//! - `QualificationRule` — crew member must hold a type rating for each leg
//! - `CoverageRule` — every leg must be assigned to the required crew complement
//! - `LegalityChecker` — orchestrates all rules and returns `Vec<LegalityViolation>`

use crate::domain::roster::Roster;

/// A single legality violation found during schedule validation.
///
/// This is a placeholder type.  The full definition (severity, entity
/// references, observed vs. threshold values, etc.) will be added in Layer 2.
#[derive(Debug, Clone, PartialEq)]
pub struct LegalityViolation {
    /// Human-readable description of the violation.
    pub message: String,
}

/// Trait for a single legality rule.
///
/// Each rule inspects a [`Roster`] and returns any violations it finds.
/// Rules are composable: the legality checker runs all registered rules and
/// collects their violations.
///
/// # Layer 2 contract
/// - Rules must be **pure** (no side effects, no mutation of the roster).
/// - Rules must be **deterministic** (same input → same output).
/// - Rules must be **independent** (no rule depends on another rule's output).
pub trait LegalityRule {
    /// A short, stable identifier for this rule, e.g. `"duty_time_limit"`.
    fn rule_id(&self) -> &str;

    /// A human-readable name for this rule.
    fn rule_name(&self) -> &str;

    /// Check the roster and return any violations found.
    fn check(&self, roster: &Roster) -> Vec<LegalityViolation>;
}

/// Placeholder legality checker.
///
/// The full implementation will be added in Layer 2.  For now this type
/// exists so that downstream code can reference it without compilation errors.
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
        self.rules
            .iter()
            .flat_map(|r| r.check(roster))
            .collect()
    }

    /// Returns `true` if no rules are registered.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Number of registered rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for LegalityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::roster::{PlanningPeriod, Roster, RosterId};
    use chrono::{TimeZone, Utc};

    struct AlwaysViolates;

    impl LegalityRule for AlwaysViolates {
        fn rule_id(&self) -> &str {
            "always_violates"
        }
        fn rule_name(&self) -> &str {
            "Always Violates (test stub)"
        }
        fn check(&self, _roster: &Roster) -> Vec<LegalityViolation> {
            vec![LegalityViolation {
                message: "stub violation".to_string(),
            }]
        }
    }

    struct NeverViolates;

    impl LegalityRule for NeverViolates {
        fn rule_id(&self) -> &str {
            "never_violates"
        }
        fn rule_name(&self) -> &str {
            "Never Violates (test stub)"
        }
        fn check(&self, _roster: &Roster) -> Vec<LegalityViolation> {
            vec![]
        }
    }

    fn empty_roster() -> Roster {
        let period = PlanningPeriod::new(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 31, 23, 59, 59).unwrap(),
        );
        Roster::new(RosterId::new("R1"), period, vec![], vec![]).unwrap()
    }

    #[test]
    fn empty_checker_returns_no_violations() {
        let checker = LegalityChecker::new();
        let roster = empty_roster();
        assert!(checker.check(&roster).is_empty());
        assert!(checker.is_empty());
        assert_eq!(checker.rule_count(), 0);
    }

    #[test]
    fn always_violates_rule_fires() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(AlwaysViolates));
        let roster = empty_roster();
        let violations = checker.check(&roster);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].message, "stub violation");
    }

    #[test]
    fn never_violates_rule_is_silent() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(NeverViolates));
        let roster = empty_roster();
        assert!(checker.check(&roster).is_empty());
    }

    #[test]
    fn multiple_rules_aggregate_violations() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(AlwaysViolates));
        checker.add_rule(Box::new(AlwaysViolates));
        checker.add_rule(Box::new(NeverViolates));
        let roster = empty_roster();
        assert_eq!(checker.check(&roster).len(), 2);
        assert_eq!(checker.rule_count(), 3);
    }
}