//! Multi-objective cost representation.
//!
//! [`CostVector`] holds one score per objective.  It supports lexicographic
//! and weighted-sum comparison, making it suitable for both single-objective
//! and multi-objective optimisation.
//!
//! [`CostEvaluator`] evaluates a [`Roster`] against a set of
//! [`SchedulingObjective`]s and returns a [`CostVector`].

use crate::domain::roster::Roster;
use crate::optimization::objective::SchedulingObjective;

// ── CostVector ────────────────────────────────────────────────────────────────

/// A vector of objective scores, one per registered objective.
///
/// Lower is better for every component (minimisation convention).
#[derive(Debug, Clone, PartialEq)]
pub struct CostVector {
    /// Objective scores in registration order.
    scores: Vec<f64>,
    /// Objective IDs in registration order.
    objective_ids: Vec<String>,
}

impl CostVector {
    /// Construct a [`CostVector`] from parallel score and ID slices.
    pub fn new(scores: Vec<f64>, objective_ids: Vec<String>) -> Self {
        assert_eq!(
            scores.len(),
            objective_ids.len(),
            "scores and objective_ids must have the same length"
        );
        Self { scores, objective_ids }
    }

    /// The score for objective at position `index`.
    pub fn score(&self, index: usize) -> f64 {
        self.scores[index]
    }

    /// The score for the objective with the given ID, if present.
    pub fn score_for(&self, objective_id: &str) -> Option<f64> {
        self.objective_ids
            .iter()
            .position(|id| id == objective_id)
            .map(|i| self.scores[i])
    }

    /// All scores in registration order.
    pub fn scores(&self) -> &[f64] {
        &self.scores
    }

    /// All objective IDs in registration order.
    pub fn objective_ids(&self) -> &[String] {
        &self.objective_ids
    }

    /// Number of objectives.
    pub fn len(&self) -> usize {
        self.scores.len()
    }

    /// Returns `true` if there are no objectives.
    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }

    /// Weighted-sum scalar: Σ weight_i × score_i.
    ///
    /// `weights` must have the same length as `scores`.  If lengths differ,
    /// the shorter slice is used.
    pub fn weighted_sum(&self, weights: &[f64]) -> f64 {
        self.scores
            .iter()
            .zip(weights.iter())
            .map(|(s, w)| s * w)
            .sum()
    }

    /// Unweighted sum of all scores.
    pub fn sum(&self) -> f64 {
        self.scores.iter().sum()
    }

    /// Returns `true` if this vector dominates `other` in the Pareto sense:
    /// every component of `self` is ≤ the corresponding component of `other`,
    /// and at least one component is strictly less.
    pub fn dominates(&self, other: &CostVector) -> bool {
        if self.scores.len() != other.scores.len() {
            return false;
        }
        let all_le = self.scores.iter().zip(other.scores.iter()).all(|(a, b)| a <= b);
        let any_lt = self.scores.iter().zip(other.scores.iter()).any(|(a, b)| a < b);
        all_le && any_lt
    }
}

impl std::fmt::Display for CostVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self
            .objective_ids
            .iter()
            .zip(self.scores.iter())
            .map(|(id, s)| format!("{id}={s:.4}"))
            .collect();
        write!(f, "[{}]", parts.join(", "))
    }
}

// ── CostEvaluator ─────────────────────────────────────────────────────────────

/// Evaluates a [`Roster`] against a set of [`SchedulingObjective`]s.
///
/// Objectives are evaluated in registration order.  The resulting
/// [`CostVector`] has one component per objective.
pub struct CostEvaluator {
    objectives: Vec<Box<dyn SchedulingObjective>>,
}

impl CostEvaluator {
    /// Create an empty [`CostEvaluator`].
    pub fn new() -> Self {
        Self { objectives: Vec::new() }
    }

    /// Register an objective.
    pub fn add_objective(&mut self, objective: Box<dyn SchedulingObjective>) {
        self.objectives.push(objective);
    }

    /// Evaluate the roster and return a [`CostVector`].
    pub fn evaluate(&self, roster: &Roster) -> CostVector {
        let scores: Vec<f64> = self.objectives.iter().map(|o| o.evaluate(roster)).collect();
        let ids: Vec<String> = self.objectives.iter().map(|o| o.objective_id().to_string()).collect();
        CostVector::new(scores, ids)
    }

    /// Number of registered objectives.
    pub fn objective_count(&self) -> usize {
        self.objectives.len()
    }

    /// IDs of all registered objectives, in registration order.
    pub fn objective_ids(&self) -> Vec<&str> {
        self.objectives.iter().map(|o| o.objective_id()).collect()
    }
}

impl Default for CostEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::objective::{CoverageCostObjective, WorkloadBalanceObjective};
    use crate::legality::test_helpers::*;

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

    // ── CostVector ────────────────────────────────────────────────────────────

    #[test]
    fn cost_vector_score_by_index() {
        let cv = CostVector::new(vec![1.0, 2.0, 3.0], vec!["a".into(), "b".into(), "c".into()]);
        assert_eq!(cv.score(0), 1.0);
        assert_eq!(cv.score(1), 2.0);
        assert_eq!(cv.score(2), 3.0);
    }

    #[test]
    fn cost_vector_score_for_id() {
        let cv = CostVector::new(vec![1.0, 2.0], vec!["workload_balance".into(), "coverage_cost".into()]);
        assert_eq!(cv.score_for("workload_balance"), Some(1.0));
        assert_eq!(cv.score_for("coverage_cost"), Some(2.0));
        assert_eq!(cv.score_for("nonexistent"), None);
    }

    #[test]
    fn cost_vector_weighted_sum() {
        let cv = CostVector::new(vec![2.0, 3.0], vec!["a".into(), "b".into()]);
        assert!((cv.weighted_sum(&[1.0, 2.0]) - 8.0).abs() < 1e-9);
    }

    #[test]
    fn cost_vector_dominates() {
        let better = CostVector::new(vec![1.0, 2.0], vec!["a".into(), "b".into()]);
        let worse  = CostVector::new(vec![2.0, 3.0], vec!["a".into(), "b".into()]);
        assert!(better.dominates(&worse));
        assert!(!worse.dominates(&better));
    }

    #[test]
    fn cost_vector_no_dominance_when_mixed() {
        let a = CostVector::new(vec![1.0, 3.0], vec!["a".into(), "b".into()]);
        let b = CostVector::new(vec![2.0, 2.0], vec!["a".into(), "b".into()]);
        assert!(!a.dominates(&b));
        assert!(!b.dominates(&a));
    }

    // ── CostEvaluator ─────────────────────────────────────────────────────────

    #[test]
    fn evaluator_with_two_objectives() {
        let mut evaluator = CostEvaluator::new();
        evaluator.add_objective(Box::new(WorkloadBalanceObjective));
        evaluator.add_objective(Box::new(CoverageCostObjective::default()));
        let roster = make_two_rotation_roster();
        let cv = evaluator.evaluate(&roster);
        assert_eq!(cv.len(), 2);
        assert_eq!(cv.objective_ids(), &["workload_balance", "coverage_cost"]);
    }

    #[test]
    fn empty_evaluator_returns_empty_vector() {
        let evaluator = CostEvaluator::new();
        let roster = make_roster(vec![], vec![]);
        let cv = evaluator.evaluate(&roster);
        assert!(cv.is_empty());
    }

    #[test]
    fn evaluator_objective_ids() {
        let mut evaluator = CostEvaluator::new();
        evaluator.add_objective(Box::new(WorkloadBalanceObjective));
        assert_eq!(evaluator.objective_ids(), vec!["workload_balance"]);
        assert_eq!(evaluator.objective_count(), 1);
    }
}