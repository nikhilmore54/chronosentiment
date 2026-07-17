//! Core data types for the Coralys Evaluation Framework.
//!
//! These types are benchmark-independent. They represent the common vocabulary
//! that all Coralys adapters, scheduling engines, and evaluation pipelines share.
//!
//! Design principles:
//! - No airline-specific, benchmark-specific, or domain-specific fields.
//! - Extensible via `metadata` maps rather than closed enums.
//! - `EvaluationResult` is the single authoritative result type owned by Coralys.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constraint severity
// ---------------------------------------------------------------------------

/// Whether a constraint is hard (infeasibility) or soft (penalised).
///
/// Hard violations make a solution infeasible. Soft violations contribute to
/// the objective but do not disqualify the solution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    Hard,
    Soft,
}

impl std::fmt::Display for ConstraintSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintSeverity::Hard => write!(f, "Hard"),
            ConstraintSeverity::Soft => write!(f, "Soft"),
        }
    }
}

// ---------------------------------------------------------------------------
// ConstraintViolation
// ---------------------------------------------------------------------------

/// A structured record of a single constraint violation.
///
/// This is the normative violation type for the Coralys Evaluation Framework.
/// Adapters must map their internal violation representations to this type.
///
/// Fields are intentionally generic so that any domain (airline crew, nurse
/// rostering, vehicle routing, etc.) can populate them without loss of
/// information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstraintViolation {
    /// Stable identifier for the constraint (e.g. "HC3", "MAX_DUTY_TIME").
    pub constraint_id: String,

    /// Human-readable name for the constraint.
    pub constraint_name: String,

    /// Whether this is a hard or soft constraint.
    pub severity: ConstraintSeverity,

    /// The value that was observed (e.g. actual workload).
    pub observed: f64,

    /// The threshold that was exceeded or not met (e.g. maximum workload).
    pub threshold: f64,

    /// The amount by which the threshold was exceeded (observed - threshold).
    /// Always >= 0 for a genuine violation.
    pub excess: f64,

    /// Optional identifier for the entity that violated the constraint
    /// (e.g. crew member ID, vehicle ID, nurse ID).
    pub entity_id: Option<u64>,

    /// Optional index of the entity within the solution (for stable ordering).
    pub entity_index: Option<usize>,

    /// Free-form metadata for adapter-specific diagnostic information.
    /// Keys and values are strings to remain domain-agnostic.
    pub metadata: HashMap<String, String>,
}

impl ConstraintViolation {
    /// Construct a minimal hard violation with no entity or metadata.
    pub fn hard(
        constraint_id: impl Into<String>,
        constraint_name: impl Into<String>,
        observed: f64,
        threshold: f64,
    ) -> Self {
        let observed = observed;
        let threshold = threshold;
        let excess = (observed - threshold).max(0.0);
        Self {
            constraint_id: constraint_id.into(),
            constraint_name: constraint_name.into(),
            severity: ConstraintSeverity::Hard,
            observed,
            threshold,
            excess,
            entity_id: None,
            entity_index: None,
            metadata: HashMap::new(),
        }
    }

    /// Construct a minimal soft violation with no entity or metadata.
    pub fn soft(
        constraint_id: impl Into<String>,
        constraint_name: impl Into<String>,
        observed: f64,
        threshold: f64,
    ) -> Self {
        let observed = observed;
        let threshold = threshold;
        let excess = (observed - threshold).max(0.0);
        Self {
            constraint_id: constraint_id.into(),
            constraint_name: constraint_name.into(),
            severity: ConstraintSeverity::Soft,
            observed,
            threshold,
            excess,
            entity_id: None,
            entity_index: None,
            metadata: HashMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// ObjectiveValue
// ---------------------------------------------------------------------------

/// A named objective value produced by an adapter.
///
/// Supporting named objectives allows multi-objective evaluation without
/// requiring callers to know the positional index of each objective.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveValue {
    /// Stable identifier for this objective (e.g. "workload_balance", "cost").
    pub objective_id: String,

    /// Human-readable name.
    pub objective_name: String,

    /// The computed value. Lower is better by convention (minimisation).
    /// Adapters that maximise should negate their values.
    pub value: f64,

    /// Optional weight for multi-objective aggregation.
    pub weight: Option<f64>,
}

impl ObjectiveValue {
    /// Construct an unweighted objective value.
    pub fn new(id: impl Into<String>, name: impl Into<String>, value: f64) -> Self {
        Self {
            objective_id: id.into(),
            objective_name: name.into(),
            value,
            weight: None,
        }
    }

    /// Construct a weighted objective value.
    pub fn weighted(id: impl Into<String>, name: impl Into<String>, value: f64, weight: f64) -> Self {
        Self {
            objective_id: id.into(),
            objective_name: name.into(),
            value,
            weight: Some(weight),
        }
    }
}

// ---------------------------------------------------------------------------
// EvaluationResult
// ---------------------------------------------------------------------------

/// The authoritative evaluation result type for the Coralys Evaluation Framework.
///
/// Every adapter registered with the framework must produce an `EvaluationResult`.
/// This type is owned by Coralys, not by any individual adapter or benchmark.
///
/// Design notes:
/// - `feasible` is derived from `violations`: a result is feasible iff it has
///   no hard violations. Adapters must not set this field independently.
/// - `objectives` is a `Vec<ObjectiveValue>` to support multi-objective evaluation.
/// - `metrics` carries adapter-specific diagnostic scalars (e.g. runtime, iteration count).
/// - `adapter_id` identifies which adapter produced this result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Identifier of the adapter that produced this result.
    pub adapter_id: String,

    /// Whether the evaluated solution is feasible (no hard constraint violations).
    /// Derived from `violations`; set by [`EvaluationResult::new`].
    pub feasible: bool,

    /// Named objective values. Adapters may return one or more objectives.
    pub objectives: Vec<ObjectiveValue>,

    /// All constraint violations (hard and soft).
    pub violations: Vec<ConstraintViolation>,

    /// Scalar diagnostic metrics (e.g. "runtime_ms", "iterations").
    pub metrics: HashMap<String, f64>,

    /// Free-form string metadata (e.g. "adapter_version", "problem_id").
    pub metadata: HashMap<String, String>,
}

impl EvaluationResult {
    /// Construct an `EvaluationResult`, deriving `feasible` from `violations`.
    pub fn new(
        adapter_id: impl Into<String>,
        objectives: Vec<ObjectiveValue>,
        violations: Vec<ConstraintViolation>,
    ) -> Self {
        let feasible = violations.iter().all(|v| v.severity != ConstraintSeverity::Hard);
        Self {
            adapter_id: adapter_id.into(),
            feasible,
            objectives,
            violations,
            metrics: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Return only the hard constraint violations.
    pub fn hard_violations(&self) -> impl Iterator<Item = &ConstraintViolation> {
        self.violations.iter().filter(|v| v.severity == ConstraintSeverity::Hard)
    }

    /// Return only the soft constraint violations.
    pub fn soft_violations(&self) -> impl Iterator<Item = &ConstraintViolation> {
        self.violations.iter().filter(|v| v.severity == ConstraintSeverity::Soft)
    }

    /// Return the value of the first objective, or 0.0 if none.
    ///
    /// Convenience accessor for single-objective adapters.
    pub fn primary_objective(&self) -> f64 {
        self.objectives.first().map(|o| o.value).unwrap_or(0.0)
    }

    /// Return the value of the objective with the given id, if present.
    pub fn objective_by_id(&self, id: &str) -> Option<f64> {
        self.objectives.iter().find(|o| o.objective_id == id).map(|o| o.value)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feasible_result_has_no_hard_violations() {
        let result = EvaluationResult::new(
            "test-adapter",
            vec![ObjectiveValue::new("obj", "Objective", 42.0)],
            vec![],
        );
        assert!(result.feasible);
        assert_eq!(result.primary_objective(), 42.0);
    }

    #[test]
    fn hard_violation_makes_result_infeasible() {
        let v = ConstraintViolation::hard("HC1", "Hard Constraint 1", 110.0, 100.0);
        let result = EvaluationResult::new("test-adapter", vec![], vec![v]);
        assert!(!result.feasible);
        assert_eq!(result.hard_violations().count(), 1);
    }

    #[test]
    fn soft_violation_does_not_affect_feasibility() {
        let v = ConstraintViolation::soft("SC1", "Soft Constraint 1", 5.0, 0.0);
        let result = EvaluationResult::new("test-adapter", vec![], vec![v]);
        assert!(result.feasible);
        assert_eq!(result.soft_violations().count(), 1);
    }

    #[test]
    fn objective_by_id_returns_correct_value() {
        let result = EvaluationResult::new(
            "test-adapter",
            vec![
                ObjectiveValue::new("cost", "Cost", 100.0),
                ObjectiveValue::new("fairness", "Fairness", 5.0),
            ],
            vec![],
        );
        assert_eq!(result.objective_by_id("cost"), Some(100.0));
        assert_eq!(result.objective_by_id("fairness"), Some(5.0));
        assert_eq!(result.objective_by_id("missing"), None);
    }

    #[test]
    fn constraint_violation_excess_is_non_negative() {
        let v = ConstraintViolation::hard("HC1", "HC1", 95.0, 100.0);
        // observed < threshold: excess should be 0, not negative
        assert_eq!(v.excess, 0.0);

        let v2 = ConstraintViolation::hard("HC1", "HC1", 110.0, 100.0);
        assert!((v2.excess - 10.0).abs() < 1e-9);
    }
}