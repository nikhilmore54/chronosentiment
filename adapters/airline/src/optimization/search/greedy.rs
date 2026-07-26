//! Greedy constructive scheduler.
//!
//! [`GreedyScheduler`] assigns unassigned pairings to rotations one at a time,
//! always choosing the rotation that minimises the weighted-sum cost after
//! the assignment.
//!
//! This produces a feasible (or near-feasible) initial solution that can be
//! improved by [`LocalSearch`](super::local_search::LocalSearch).
//!
//! # Algorithm
//!
//! 1. Start from the provided roster (which may have empty rotations).
//! 2. For each unassigned pairing (in the order provided), try appending it
//!    to each rotation.
//! 3. Accept the assignment that produces the lowest weighted-sum cost.
//! 4. Repeat until all pairings are assigned.
//!
//! The greedy scheduler does **not** check legality — it optimises cost only.
//! Use the Layer 2 [`LegalityChecker`] after construction to verify the result.
//!
//! [`LegalityChecker`]: crate::legality::LegalityChecker

use crate::domain::pairing::Pairing;
use crate::domain::roster::Roster;
use crate::domain::rotation::Rotation;
use crate::optimization::cost::CostEvaluator;
use crate::optimization::metrics::OptimizationMetrics;

/// A greedy constructive scheduler.
pub struct GreedyScheduler<'a> {
    evaluator: &'a CostEvaluator,
    /// Weights for the weighted-sum cost.  Must have the same length as the
    /// number of objectives in `evaluator`.
    weights: Vec<f64>,
}

impl<'a> GreedyScheduler<'a> {
    /// Create a new [`GreedyScheduler`].
    ///
    /// `weights` must have the same length as the number of objectives in
    /// `evaluator`.  If `weights` is empty, equal weights are used.
    pub fn new(evaluator: &'a CostEvaluator, weights: Vec<f64>) -> Self {
        Self { evaluator, weights }
    }

    /// Assign `pairings` to rotations in the `roster` greedily.
    ///
    /// Returns the improved roster and the metrics collected during search.
    pub fn assign(
        &self,
        roster: &Roster,
        pairings: Vec<Pairing>,
        metrics: &mut OptimizationMetrics,
    ) -> Roster {
        let mut current = roster.clone();

        for pairing in pairings {
            let rotations: Vec<_> = current.rotations().collect();
            let n = rotations.len();
            if n == 0 {
                break;
            }

            let mut best_cost = f64::INFINITY;
            let mut best_roster: Option<Roster> = None;

            for (rot_idx, rotation) in rotations.iter().enumerate() {
                // Build a candidate roster with the pairing appended to this rotation.
                // pairings() returns &[Pairing]; use .to_vec() to clone into a Vec.
                let mut new_pairings: Vec<_> = rotation.pairings().to_vec();
                new_pairings.push(pairing.clone());

                let new_rotation = match Rotation::new(
                    rotation.id.clone(),
                    rotation.crew_id.clone(),
                    new_pairings,
                ) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let new_rotations: Vec<_> = current
                    .rotations()
                    .enumerate()
                    .map(|(i, r)| if i == rot_idx { new_rotation.clone() } else { r.clone() })
                    .collect();

                let candidate = match Roster::new(
                    current.id.clone(),
                    current.period.clone(),
                    current.legs().cloned().collect(),
                    new_rotations,
                ) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                metrics.record_evaluation();
                let cost = self.evaluator.evaluate(&candidate);
                let weighted = if self.weights.is_empty() {
                    cost.sum()
                } else {
                    cost.weighted_sum(&self.weights)
                };

                if weighted < best_cost {
                    best_cost = weighted;
                    best_roster = Some(candidate);
                    metrics.record_improvement();
                }
            }

            if let Some(better) = best_roster {
                current = better;
            }
        }

        current
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

    fn make_empty_two_rotation_roster() -> Roster {
        let d1a = make_duty("D1a", vec![make_leg("L1a", "LHR", "CDG", 8, 10)]);
        let d1b = make_duty("D1b", vec![make_leg("L1b", "CDG", "LHR", 22, 24)]);
        let p_seed = make_pairing("P_seed", "LHR", vec![d1a, d1b]);
        let d2a = make_duty("D2a", vec![make_leg("L2a", "LHR", "CDG", 8, 10)]);
        let d2b = make_duty("D2b", vec![make_leg("L2b", "CDG", "LHR", 22, 24)]);
        let p_seed2 = make_pairing("P_seed2", "LHR", vec![d2a, d2b]);
        let r1 = make_rotation("R1", "C1", vec![p_seed]);
        let r2 = make_rotation("R2", "C2", vec![p_seed2]);
        make_roster(vec![], vec![r1, r2])
    }

    #[test]
    fn greedy_assigns_pairing_to_best_rotation() {
        let mut evaluator = CostEvaluator::new();
        evaluator.add_objective(Box::new(WorkloadBalanceObjective));
        let scheduler = GreedyScheduler::new(&evaluator, vec![1.0]);
        let roster = make_empty_two_rotation_roster();

        // p_new must start after the seed pairings end (hour 24) and be a round-trip.
        let d_new_out = make_duty("D_new_out", vec![make_leg("L_new_out", "LHR", "CDG", 32, 34)]);
        let d_new_ret = make_duty("D_new_ret", vec![make_leg("L_new_ret", "CDG", "LHR", 46, 48)]);
        let p_new = make_pairing("P_new", "LHR", vec![d_new_out, d_new_ret]);

        let mut metrics = OptimizationMetrics::new();
        let result = scheduler.assign(&roster, vec![p_new], &mut metrics);

        // The pairing should have been assigned to one of the rotations.
        let total_pairings: usize = result.rotations().map(|r| r.pairings().len()).sum();
        assert_eq!(total_pairings, 3); // 2 seed + 1 new
        assert!(metrics.evaluations() > 0);
    }

    #[test]
    fn greedy_empty_pairings_returns_unchanged_roster() {
        let mut evaluator = CostEvaluator::new();
        evaluator.add_objective(Box::new(WorkloadBalanceObjective));
        let scheduler = GreedyScheduler::new(&evaluator, vec![1.0]);
        let roster = make_empty_two_rotation_roster();
        let mut metrics = OptimizationMetrics::new();
        let result = scheduler.assign(&roster, vec![], &mut metrics);
        let total: usize = result.rotations().map(|r| r.pairings().len()).sum();
        assert_eq!(total, 2); // unchanged
    }
}