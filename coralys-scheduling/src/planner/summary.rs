//! Violation summarisation.
//!
//! [`ViolationSummary`] aggregates a flat list of [`LegalityViolation`]s into
//! structured views that planners can use to prioritise remediation:
//!
//! - by rule (`by_rule`)
//! - by severity (`errors`, `warnings`, `advisories`)
//! - by entity type (`by_entity_type`)
//! - by crew member (`by_crew`)
//!
//! # Example
//!
//! ```rust,ignore
//! let violations = checker.check(&roster);
//! let summary = ViolationSummary::from(violations);
//! println!("{} errors, {} warnings", summary.error_count(), summary.warning_count());
//! for (rule_id, vs) in summary.by_rule() {
//!     println!("  {}: {} violations", rule_id, vs.len());
//! }
//! ```

use std::collections::HashMap;

use crate::legality::{EntityRef, LegalityViolation, ViolationSeverity};

// ── Entity type discriminant ──────────────────────────────────────────────────

/// The type of scheduling entity referenced by a violation.
///
/// Used as a grouping key in [`ViolationSummary::by_entity_type`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Leg,
    Duty,
    Pairing,
    Rotation,
    Roster,
}

impl EntityType {
    fn from_ref(entity: &EntityRef) -> Self {
        match entity {
            EntityRef::Leg(_) => EntityType::Leg,
            EntityRef::Duty(_) => EntityType::Duty,
            EntityRef::Pairing(_) => EntityType::Pairing,
            EntityRef::Rotation { .. } => EntityType::Rotation,
            EntityRef::Roster(_) => EntityType::Roster,
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Leg => write!(f, "Leg"),
            EntityType::Duty => write!(f, "Duty"),
            EntityType::Pairing => write!(f, "Pairing"),
            EntityType::Rotation => write!(f, "Rotation"),
            EntityType::Roster => write!(f, "Roster"),
        }
    }
}

// ── Summary ───────────────────────────────────────────────────────────────────

/// An aggregated view of a set of [`LegalityViolation`]s.
///
/// Constructed from a `Vec<LegalityViolation>` via [`ViolationSummary::new`]
/// or the `From` impl.  All views are computed eagerly at construction time.
#[derive(Debug, Clone)]
pub struct ViolationSummary {
    /// All violations, in the order they were provided.
    all: Vec<LegalityViolation>,
    /// Violations grouped by rule ID.
    by_rule: HashMap<String, Vec<usize>>,
    /// Violations grouped by severity.
    by_severity: HashMap<String, Vec<usize>>,
    /// Violations grouped by entity type.
    by_entity_type: HashMap<EntityType, Vec<usize>>,
    /// Violations grouped by crew ID (for Rotation entities only).
    by_crew: HashMap<String, Vec<usize>>,
}

impl ViolationSummary {
    /// Construct a [`ViolationSummary`] from a list of violations.
    pub fn new(violations: Vec<LegalityViolation>) -> Self {
        let mut by_rule: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_severity: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_entity_type: HashMap<EntityType, Vec<usize>> = HashMap::new();
        let mut by_crew: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, v) in violations.iter().enumerate() {
            by_rule.entry(v.rule_id.clone()).or_default().push(idx);

            let sev_key = format!("{}", v.severity);
            by_severity.entry(sev_key).or_default().push(idx);

            let et = EntityType::from_ref(&v.entity);
            by_entity_type.entry(et).or_default().push(idx);

            if let EntityRef::Rotation { crew_id, .. } = &v.entity {
                by_crew.entry(crew_id.clone()).or_default().push(idx);
            }
        }

        Self {
            all: violations,
            by_rule,
            by_severity,
            by_entity_type,
            by_crew,
        }
    }

    // ── Counts ────────────────────────────────────────────────────────────────

    /// Total number of violations.
    pub fn total(&self) -> usize {
        self.all.len()
    }

    /// Number of `Error`-severity violations.
    pub fn error_count(&self) -> usize {
        self.by_severity
            .get("Error")
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Number of `Warning`-severity violations.
    pub fn warning_count(&self) -> usize {
        self.by_severity
            .get("Warning")
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Number of `Advisory`-severity violations.
    pub fn advisory_count(&self) -> usize {
        self.by_severity
            .get("Advisory")
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Returns `true` if there are no `Error`-severity violations.
    pub fn is_legal(&self) -> bool {
        self.error_count() == 0
    }

    // ── Filtered views ────────────────────────────────────────────────────────

    /// All `Error`-severity violations.
    pub fn errors(&self) -> Vec<&LegalityViolation> {
        self.all
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Error)
            .collect()
    }

    /// All `Warning`-severity violations.
    pub fn warnings(&self) -> Vec<&LegalityViolation> {
        self.all
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Warning)
            .collect()
    }

    /// All violations for a specific rule.
    pub fn for_rule(&self, rule_id: &str) -> Vec<&LegalityViolation> {
        self.by_rule
            .get(rule_id)
            .map(|idxs| idxs.iter().map(|&i| &self.all[i]).collect())
            .unwrap_or_default()
    }

    /// All violations for a specific entity type.
    pub fn for_entity_type(&self, entity_type: &EntityType) -> Vec<&LegalityViolation> {
        self.by_entity_type
            .get(entity_type)
            .map(|idxs| idxs.iter().map(|&i| &self.all[i]).collect())
            .unwrap_or_default()
    }

    /// All violations for a specific crew member (Rotation-entity violations only).
    pub fn for_crew(&self, crew_id: &str) -> Vec<&LegalityViolation> {
        self.by_crew
            .get(crew_id)
            .map(|idxs| idxs.iter().map(|&i| &self.all[i]).collect())
            .unwrap_or_default()
    }

    // ── Grouped views ─────────────────────────────────────────────────────────

    /// Violations grouped by rule ID.
    ///
    /// Returns a map from rule ID to the violations produced by that rule.
    pub fn by_rule(&self) -> HashMap<&str, Vec<&LegalityViolation>> {
        self.by_rule
            .iter()
            .map(|(rule_id, idxs)| {
                (
                    rule_id.as_str(),
                    idxs.iter().map(|&i| &self.all[i]).collect(),
                )
            })
            .collect()
    }

    /// Violations grouped by entity type.
    pub fn by_entity_type(&self) -> HashMap<&EntityType, Vec<&LegalityViolation>> {
        self.by_entity_type
            .iter()
            .map(|(et, idxs)| (et, idxs.iter().map(|&i| &self.all[i]).collect()))
            .collect()
    }

    /// Rule IDs that produced at least one violation, sorted alphabetically.
    pub fn violated_rule_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.by_rule.keys().map(|s| s.as_str()).collect();
        ids.sort_unstable();
        ids
    }

    /// All violations, in the order they were provided.
    pub fn all(&self) -> &[LegalityViolation] {
        &self.all
    }
}

impl From<Vec<LegalityViolation>> for ViolationSummary {
    fn from(violations: Vec<LegalityViolation>) -> Self {
        Self::new(violations)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::{EntityRef, LegalityViolation};

    fn make_error(rule: &str, entity: EntityRef) -> LegalityViolation {
        LegalityViolation::error(rule, entity, 10.0, 5.0, "test error")
    }

    fn make_warning(rule: &str, entity: EntityRef) -> LegalityViolation {
        LegalityViolation::warning(rule, entity, 2.0, 1.0, "test warning")
    }

    fn make_rotation_violation(rule: &str, crew_id: &str) -> LegalityViolation {
        LegalityViolation::error(
            rule,
            EntityRef::Rotation {
                rotation_id: format!("R-{crew_id}"),
                crew_id: crew_id.to_string(),
            },
            1.0, 0.0,
            "rotation violation",
        )
    }

    // ── Empty summary ─────────────────────────────────────────────────────────

    #[test]
    fn empty_summary_is_legal() {
        let s = ViolationSummary::new(vec![]);
        assert_eq!(s.total(), 0);
        assert_eq!(s.error_count(), 0);
        assert_eq!(s.warning_count(), 0);
        assert!(s.is_legal());
    }

    // ── Counts ────────────────────────────────────────────────────────────────

    #[test]
    fn counts_by_severity() {
        let violations = vec![
            make_error("rule_a", EntityRef::Duty("D1".into())),
            make_error("rule_b", EntityRef::Duty("D2".into())),
            make_warning("rule_c", EntityRef::Pairing("P1".into())),
        ];
        let s = ViolationSummary::new(violations);
        assert_eq!(s.total(), 3);
        assert_eq!(s.error_count(), 2);
        assert_eq!(s.warning_count(), 1);
        assert_eq!(s.advisory_count(), 0);
        assert!(!s.is_legal());
    }

    // ── By rule ───────────────────────────────────────────────────────────────

    #[test]
    fn group_by_rule() {
        let violations = vec![
            make_error("max_duty_time", EntityRef::Duty("D1".into())),
            make_error("max_duty_time", EntityRef::Duty("D2".into())),
            make_warning("coverage", EntityRef::Leg("L1".into())),
        ];
        let s = ViolationSummary::new(violations);
        assert_eq!(s.for_rule("max_duty_time").len(), 2);
        assert_eq!(s.for_rule("coverage").len(), 1);
        assert_eq!(s.for_rule("nonexistent").len(), 0);
        let mut ids = s.violated_rule_ids();
        ids.sort();
        assert_eq!(ids, vec!["coverage", "max_duty_time"]);
    }

    // ── By entity type ────────────────────────────────────────────────────────

    #[test]
    fn group_by_entity_type() {
        let violations = vec![
            make_error("r1", EntityRef::Duty("D1".into())),
            make_error("r2", EntityRef::Leg("L1".into())),
            make_warning("r3", EntityRef::Leg("L2".into())),
        ];
        let s = ViolationSummary::new(violations);
        assert_eq!(s.for_entity_type(&EntityType::Duty).len(), 1);
        assert_eq!(s.for_entity_type(&EntityType::Leg).len(), 2);
        assert_eq!(s.for_entity_type(&EntityType::Pairing).len(), 0);
    }

    // ── By crew ───────────────────────────────────────────────────────────────

    #[test]
    fn group_by_crew() {
        let violations = vec![
            make_rotation_violation("base_return", "C1"),
            make_rotation_violation("base_return", "C1"),
            make_rotation_violation("qualification", "C2"),
        ];
        let s = ViolationSummary::new(violations);
        assert_eq!(s.for_crew("C1").len(), 2);
        assert_eq!(s.for_crew("C2").len(), 1);
        assert_eq!(s.for_crew("C99").len(), 0);
    }

    // ── Filtered views ────────────────────────────────────────────────────────

    #[test]
    fn errors_and_warnings_filtered() {
        let violations = vec![
            make_error("r1", EntityRef::Duty("D1".into())),
            make_warning("r2", EntityRef::Pairing("P1".into())),
        ];
        let s = ViolationSummary::new(violations);
        assert_eq!(s.errors().len(), 1);
        assert_eq!(s.warnings().len(), 1);
    }

    // ── From impl ─────────────────────────────────────────────────────────────

    #[test]
    fn from_vec_works() {
        let v = vec![make_error("r", EntityRef::Roster("R1".into()))];
        let s: ViolationSummary = v.into();
        assert_eq!(s.total(), 1);
    }
}