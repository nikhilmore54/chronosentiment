//! Incremental legality evaluation.
//!
//! [`IncrementalChecker`] wraps a [`LegalityChecker`] and a cached set of
//! per-rotation violations.  When a single rotation changes, only that
//! rotation's violations are re-evaluated; all other rotations' cached
//! results are reused.
//!
//! # Limitations
//!
//! - Rules that evaluate the roster as a whole (e.g. [`CoverageRule`]) are
//!   always re-run in full, because their output depends on all rotations.
//!   These are called *global rules*.
//! - Rules that evaluate individual rotations in isolation (e.g.
//!   [`MaximumDutyTimeRule`], [`MinimumRestRule`]) can be cached per rotation.
//!
//! The current implementation re-runs **all** rules against the full roster
//! for the affected rotation, then merges the result with the cached results
//! for unaffected rotations.
//!
//! [`CoverageRule`]: crate::legality::coverage::CoverageRule
//! [`MaximumDutyTimeRule`]: crate::legality::duty_time::MaximumDutyTimeRule
//! [`MinimumRestRule`]: crate::legality::minimum_rest::MinimumRestRule

use std::collections::HashMap;

use crate::domain::crew::CrewId;
use crate::domain::roster::Roster;
use crate::legality::{LegalityChecker, LegalityViolation};

// ── Incremental checker ───────────────────────────────────────────────────────

/// A legality checker that caches per-rotation results and re-evaluates
/// only the affected rotation when a change is made.
///
/// # Usage
///
/// 1. Create with [`IncrementalChecker::new`], passing the checker and the
///    initial roster.
/// 2. After editing a rotation, call [`IncrementalChecker::recheck_rotation`]
///    with the updated roster and the crew ID of the changed rotation.
/// 3. Call [`IncrementalChecker::all_violations`] to get the current full
///    violation set.
pub struct IncrementalChecker {
    checker: LegalityChecker,
    /// Cached violations per crew member's rotation.
    rotation_cache: HashMap<CrewId, Vec<LegalityViolation>>,
    /// Violations from rules that evaluate the roster as a whole.
    global_violations: Vec<LegalityViolation>,
}

impl IncrementalChecker {
    /// Create a new [`IncrementalChecker`] and run the initial full evaluation.
    pub fn new(checker: LegalityChecker, roster: &Roster) -> Self {
        let all = checker.check(roster);
        let (rotation_violations, global_violations) = partition_violations(all);

        let mut rotation_cache: HashMap<CrewId, Vec<LegalityViolation>> = HashMap::new();
        for (crew_id, vs) in rotation_violations {
            rotation_cache.entry(crew_id).or_default().extend(vs);
        }

        // Seed empty entries for rotations with no violations.
        for crew_id in roster.crew_ids() {
            rotation_cache.entry(crew_id.clone()).or_default();
        }

        Self {
            checker,
            rotation_cache,
            global_violations,
        }
    }

    /// Re-evaluate the rotation for `crew_id` after it has been modified.
    ///
    /// Replaces the cached violations for `crew_id` and refreshes global
    /// violations.  All other rotations' cached violations are preserved.
    pub fn recheck_rotation(&mut self, roster: &Roster, crew_id: &CrewId) {
        let all = self.checker.check(roster);
        let (rotation_violations, global_violations) = partition_violations(all);

        self.rotation_cache.insert(
            crew_id.clone(),
            rotation_violations
                .get(crew_id)
                .cloned()
                .unwrap_or_default(),
        );
        self.global_violations = global_violations;
    }

    /// All current violations (cached per-rotation + global).
    pub fn all_violations(&self) -> Vec<&LegalityViolation> {
        let mut result: Vec<&LegalityViolation> = self
            .rotation_cache
            .values()
            .flat_map(|vs| vs.iter())
            .collect();
        result.extend(self.global_violations.iter());
        result
    }

    /// All current violations as owned values (cloned).
    pub fn all_violations_owned(&self) -> Vec<LegalityViolation> {
        self.all_violations().into_iter().cloned().collect()
    }

    /// Returns `true` if there are no `Error`-severity violations.
    pub fn is_legal(&self) -> bool {
        self.all_violations().iter().all(|v| !v.is_error())
    }

    /// Number of cached rotations.
    pub fn cached_rotation_count(&self) -> usize {
        self.rotation_cache.len()
    }

    /// Violations cached for a specific crew member's rotation.
    pub fn violations_for_crew(&self, crew_id: &CrewId) -> &[LegalityViolation] {
        self.rotation_cache
            .get(crew_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Partition violations into per-rotation (keyed by crew ID) and global.
///
/// A violation is attributed to a rotation if its entity is
/// `EntityRef::Rotation { crew_id, .. }`.  All other violations are global.
fn partition_violations(
    violations: Vec<LegalityViolation>,
) -> (
    HashMap<CrewId, Vec<LegalityViolation>>,
    Vec<LegalityViolation>,
) {
    use crate::legality::EntityRef;

    let mut rotation_map: HashMap<CrewId, Vec<LegalityViolation>> = HashMap::new();
    let mut global = Vec::new();

    for v in violations {
        if let EntityRef::Rotation { ref crew_id, .. } = v.entity {
            rotation_map
                .entry(CrewId::new(crew_id.clone()))
                .or_default()
                .push(v);
        } else {
            global.push(v);
        }
    }

    (rotation_map, global)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::test_helpers::*;
    use crate::legality::{EntityRef, LegalityRule, LegalityViolation};

    /// Emits one Rotation-entity violation per rotation in the roster.
    struct RotationErrorRule;
    impl LegalityRule for RotationErrorRule {
        fn rule_id(&self) -> &str {
            "rotation_error"
        }
        fn rule_name(&self) -> &str {
            "Rotation Error"
        }
        fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
            roster
                .rotations()
                .map(|r| {
                    LegalityViolation::error(
                        "rotation_error",
                        EntityRef::Rotation {
                            rotation_id: r.id.as_str().to_string(),
                            crew_id: r.crew_id.as_str().to_string(),
                        },
                        1.0,
                        0.0,
                        format!("rotation {} has an error", r.id),
                    )
                })
                .collect()
        }
    }

    /// Emits one Roster-entity (global) violation.
    struct GlobalErrorRule;
    impl LegalityRule for GlobalErrorRule {
        fn rule_id(&self) -> &str {
            "global_error"
        }
        fn rule_name(&self) -> &str {
            "Global Error"
        }
        fn check(&self, roster: &Roster) -> Vec<LegalityViolation> {
            vec![LegalityViolation::error(
                "global_error",
                EntityRef::Roster(roster.id.to_string()),
                1.0,
                0.0,
                "global error",
            )]
        }
    }

    fn make_two_rotation_roster() -> Roster {
        let d1a = make_duty("D1a", vec![make_leg("L1a", "LHR", "CDG", 8, 10)]);
        let d1b = make_duty("D1b", vec![make_leg("L1b", "CDG", "LHR", 22, 24)]);
        let d2a = make_duty("D2a", vec![make_leg("L2a", "LHR", "CDG", 8, 10)]);
        let d2b = make_duty("D2b", vec![make_leg("L2b", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1a, d1b]);
        let p2 = make_pairing("P2", "LHR", vec![d2a, d2b]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        make_roster(vec![], vec![r1, r2])
    }

    // ── Initial evaluation caches all rotations ───────────────────────────────

    #[test]
    fn initial_evaluation_caches_all_rotations() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(RotationErrorRule));
        let roster = make_two_rotation_roster();
        let ic = IncrementalChecker::new(checker, &roster);
        assert_eq!(ic.all_violations().len(), 2);
        assert_eq!(ic.cached_rotation_count(), 2);
    }

    // ── Global violations are included ────────────────────────────────────────

    #[test]
    fn global_violations_included_in_all() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(GlobalErrorRule));
        let roster = make_two_rotation_roster();
        let ic = IncrementalChecker::new(checker, &roster);
        assert_eq!(ic.all_violations().len(), 1);
    }

    // ── Per-rotation cache lookup ─────────────────────────────────────────────

    #[test]
    fn violations_for_crew_returns_cached_violations() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(RotationErrorRule));
        let roster = make_two_rotation_roster();
        let ic = IncrementalChecker::new(checker, &roster);
        assert_eq!(ic.violations_for_crew(&CrewId::new("C1")).len(), 1);
        assert_eq!(ic.violations_for_crew(&CrewId::new("C2")).len(), 1);
    }

    // ── Recheck updates the target rotation ──────────────────────────────────

    #[test]
    fn recheck_rotation_updates_cache() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(RotationErrorRule));
        let roster = make_two_rotation_roster();
        let mut ic = IncrementalChecker::new(checker, &roster);
        let c1 = CrewId::new("C1");
        ic.recheck_rotation(&roster, &c1);
        // Roster unchanged → same violation count
        assert_eq!(ic.all_violations().len(), 2);
    }

    // ── Empty roster ──────────────────────────────────────────────────────────

    #[test]
    fn empty_roster_no_violations() {
        let checker = LegalityChecker::new();
        let roster = make_roster(vec![], vec![]);
        let ic = IncrementalChecker::new(checker, &roster);
        assert_eq!(ic.all_violations().len(), 0);
        assert!(ic.is_legal());
    }

    // ── is_legal reflects error presence ─────────────────────────────────────

    #[test]
    fn is_legal_false_when_errors_present() {
        let mut checker = LegalityChecker::new();
        checker.add_rule(Box::new(RotationErrorRule));
        let roster = make_two_rotation_roster();
        let ic = IncrementalChecker::new(checker, &roster);
        assert!(!ic.is_legal());
    }
}
