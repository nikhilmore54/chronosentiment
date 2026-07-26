//! Hill-climbing local search.
//!
//! [`LocalSearch`] improves a roster by repeatedly applying neighborhood
//! moves (swap and relocate) and accepting any move that reduces the
//! weighted-sum cost **and** produces a legally feasible roster.
//!
//! The Layer 2 [`LegalityChecker`] is the sole feasibility oracle.
//!
//! # Algorithm
//!
//! 1. Evaluate the current roster's cost.
//! 2. Generate all swap and relocate moves.
//! 3. For each move:
//!    a. Apply the move to get a candidate roster.
//!    b. Check feasibility with the legality checker.
//!    c. Evaluate the candidate's cost.
//!    d. If the candidate is feasible and cheaper, accept it.
//! 4. If no improving move was found, stop (local optimum).
//! 5. Repeat up to `max_iterations` times.
//!
//! [`LegalityChecker`]: crate::legality::LegalityChecker

use crate::domain::roster::Roster;
use crate::legality::LegalityChecker;
use crate::optimization::cost::CostEvaluator;
use crate::optimization::metrics::OptimizationMetrics;
use crate::optimization::neighborhood::{relocate, swap};

/// Hill-climbing local search.
pub struct LocalSearch<'a> {
    evaluator: &'a CostEvaluator,
    checker: &'a LegalityChecker,
    weights: Vec<f64>,
    max_iterations: usize,
}

impl<'a> LocalSearch<'a> {
    /// Create a new [`LocalSearch`].
    ///
    /// - `evaluator` — cost evaluator with registered objectives.
    /// - `checker` — legality checker used as feasibility oracle.
    /// - `weights` — weights for the weighted-sum cost.
    /// - `max_iterations` — maximum number of improvement iterations.
    pub fn new(
        evaluator: &'a CostEvaluator,
        checker: &'a LegalityChecker,
        weights: Vec<f64>,
        max_iterations: usize,
    ) -> Self {
        Self { evaluator, checker, weights, max_iterations }
    }

    /// Run local search from `initial` and return the best roster found.
    pub fn run(&self, initial: &Roster, metrics: &mut OptimizationMetrics) -> Roster {
        let mut current = initial.clone();
        let mut current_cost = self.weighted_cost(&current);

        for _ in 0..self.max_iterations {
            let improved = self.best_improving_move(&current, current_cost, metrics);
            match improved {
                Some((better, cost)) => {
                    current = better;
                    current_cost = cost;
                    metrics.record_improvement();
                }
                None => break, // local optimum
            }
        }

        current
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn weighted_cost(&self, roster: &Roster) -> f64 {
        let cv = self.evaluator.evaluate(roster);
        if self.weights.is_empty() {
            cv.sum()
        } else {
            cv.weighted_sum(&self.weights)
        }
    }

    /// Find the best improving move from the current roster.
    fn best_improving_move(
        &self,
        current: &Roster,
        current_cost: f64,
        metrics: &mut OptimizationMetrics,
    ) -> Option<(Roster, f64)> {
        let rotations: Vec<_> = current.rotations().collect();
        let n = rotations.len();
        let mut best: Option<(Roster, f64)> = None;
        let mut best_cost = current_cost;

        // ── Swap moves ────────────────────────────────────────────────────────
        for i in 0..n {
            let pi = rotations[i].pairings().len();
            for j in (i + 1)..n {
                let pj = rotations[j].pairings().len();
                for pa in 0..pi {
                    for pb in 0..pj {
                        if let Some(candidate) = swap::swap_pairings(current, i, pa, j, pb) {
                            metrics.record_evaluation();
                            metrics.record_feasibility_check();
                            if self.checker.is_legal(&candidate) {
                                let cost = self.weighted_cost(&candidate);
                                if cost < best_cost {
                                    best_cost = cost;
                                    best = Some((candidate, cost));
                                }
                            }
                        }
                    }
                }
            }
        }

        // ── Relocate moves ────────────────────────────────────────────────────
        for src in 0..n {
            let p_count = rotations[src].pairings().len();
            for dst in 0..n {
                if src == dst {
                    continue;
                }
                for pi in 0..p_count {
                    if let Some(candidate) = relocate::relocate_pairing(current, src, pi, dst) {
                        metrics.record_evaluation();
                        metrics.record_feasibility_check();
                        if self.checker.is_legal(&candidate) {
                            let cost = self.weighted_cost(&candidate);
                            if cost < best_cost {
                                best_cost = cost;
                                best = Some((candidate, cost));
                            }
                        }
                    }
                }
            }
        }

        best
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::test_helpers::*;
    use crate::optimization::cost::CostEvaluator;
    use crate::optimization::metrics::OptimizationMetrics;
    use crate::optimization::objective::WorkloadBalanceObjective;

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

    #[test]
    fn local_search_runs_without_panic() {
        let mut evaluator = CostEvaluator::new();
        evaluator.add_objective(Box::new(WorkloadBalanceObjective));
        let checker = LegalityChecker::new();
        let ls = LocalSearch::new(&evaluator, &checker, vec![1.0], 10);
        let roster = make_two_rotation_roster();
        let mut metrics = OptimizationMetrics::new();
        let result = ls.run(&roster, &mut metrics);
        // Result should be a valid roster.
        assert!(result.rotations().count() > 0);
    }

    #[test]
    fn local_search_records_evaluations() {
        let mut evaluator = CostEvaluator::new();
        evaluator.add_objective(Box::new(WorkloadBalanceObjective));
        let checker = LegalityChecker::new();
        let ls = LocalSearch::new(&evaluator, &checker, vec![1.0], 5);
        let roster = make_two_rotation_roster();
        let mut metrics = OptimizationMetrics::new();
        ls.run(&roster, &mut metrics);
        // With 2 rotations and 1 pairing each, there is 1 swap move.
        // Swap is valid but cost is equal (balanced) → no improvement → stops.
        assert!(metrics.feasibility_checks() > 0);
    }

    #[test]
    fn local_search_empty_checker_accepts_all() {
        let mut evaluator = CostEvaluator::new();
        evaluator.add_objective(Box::new(WorkloadBalanceObjective));
        let checker = LegalityChecker::new(); // no rules → everything legal
        let ls = LocalSearch::new(&evaluator, &checker, vec![1.0], 3);
        let roster = make_two_rotation_roster();
        let mut metrics = OptimizationMetrics::new();
        let result = ls.run(&roster, &mut metrics);
        assert_eq!(result.rotations().count(), 2);
    }
}