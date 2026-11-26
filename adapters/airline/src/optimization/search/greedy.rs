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

        // ── Sub-stage profiling accumulators ─────────────────────────────────
        let mut t_rotation_collect_us: u128 = 0; // collect rotations snapshot (once per pairing)
        let mut t_pairing_clone_us: u128 = 0; // clone pairings vec + Rotation::new()
        let mut t_rotation_clone_us: u128 = 0; // clone all rotations into new_rotations vec
        let mut t_leg_clone_us: u128 = 0; // current.legs().cloned().collect()
        let mut t_roster_new_us: u128 = 0; // Roster::new() construction
        let mut t_evaluate_us: u128 = 0; // CostEvaluator::evaluate()
        let mut t_commit_us: u128 = 0; // commit best_roster to current
        let mut total_candidates: u64 = 0;
        let mut pairing_count: u64 = 0;
        // Object-count accumulators (not timing — counts reveal asymptotic cost)
        let mut n_rotation_clones: u64 = 0;
        let mut n_leg_clones: u64 = 0;
        let mut n_pairing_vec_clones: u64 = 0;
        let mut n_roster_constructions: u64 = 0;

        // ── Feasibility / objective-distribution accumulators ─────────────────
        // Per-pairing: rotations_examined, rotations_legal (Rotation::new ok),
        // rotations_improving (weighted < best_cost so far).
        // We aggregate min/max/sum across pairings to avoid per-pairing log spam.
        let mut total_rot_examined: u64 = 0;
        let mut total_rot_legal: u64 = 0;
        let mut total_rot_improving: u64 = 0;
        let mut min_rot_legal: u64 = u64::MAX;
        let mut max_rot_legal: u64 = 0;
        let mut min_rot_improving: u64 = u64::MAX;
        let mut max_rot_improving: u64 = 0;
        // Objective value distribution: across all legal candidates, how many
        // are strictly improving vs tied with best vs strictly worse?
        let mut n_obj_improving: u64 = 0; // weighted < best_cost at time of evaluation
        let mut n_obj_tied: u64 = 0; // weighted == best_cost (f64 exact equality)
        let mut n_obj_worse: u64 = 0; // weighted > best_cost

        for pairing in pairings {
            pairing_count += 1;

            let t0 = std::time::Instant::now();
            let rotations: Vec<_> = current.rotations().collect();
            t_rotation_collect_us += t0.elapsed().as_micros();

            let n = rotations.len();
            if n == 0 {
                break;
            }

            let mut best_cost = f64::INFINITY;
            let mut best_roster: Option<Roster> = None;

            // Per-pairing feasibility counters
            let mut p_rot_examined: u64 = 0;
            let mut p_rot_legal: u64 = 0;
            let mut p_rot_improving: u64 = 0;

            for (rot_idx, rotation) in rotations.iter().enumerate() {
                total_candidates += 1;
                p_rot_examined += 1;

                // ── Phase A: clone pairings + Rotation::new() ─────────────
                let t1 = std::time::Instant::now();
                let mut new_pairings: Vec<_> = rotation.pairings().to_vec();
                n_pairing_vec_clones += new_pairings.len() as u64;
                new_pairings.push(pairing.clone());

                let new_rotation = match Rotation::new(
                    rotation.id.clone(),
                    rotation.crew_id.clone(),
                    new_pairings,
                ) {
                    Ok(r) => r,
                    Err(_) => {
                        t_pairing_clone_us += t1.elapsed().as_micros();
                        continue;
                    }
                };
                p_rot_legal += 1;
                t_pairing_clone_us += t1.elapsed().as_micros();

                // ── Phase B: clone all rotations into new_rotations ────────
                let t2a = std::time::Instant::now();
                let n_rot = current.rotations().count() as u64;
                let new_rotations: Vec<_> = current
                    .rotations()
                    .enumerate()
                    .map(|(i, r)| {
                        if i == rot_idx {
                            new_rotation.clone()
                        } else {
                            r.clone()
                        }
                    })
                    .collect();
                n_rotation_clones += n_rot;
                t_rotation_clone_us += t2a.elapsed().as_micros();

                // ── Phase C: clone all legs ────────────────────────────────
                let t2b = std::time::Instant::now();
                let n_legs = current.legs().count() as u64;
                let legs_cloned: Vec<_> = current.legs().cloned().collect();
                n_leg_clones += n_legs;
                t_leg_clone_us += t2b.elapsed().as_micros();

                // ── Phase D: Roster::new() ─────────────────────────────────
                let t2c = std::time::Instant::now();
                let candidate = match Roster::new(
                    current.id.clone(),
                    current.period.clone(),
                    legs_cloned,
                    new_rotations,
                ) {
                    Ok(r) => r,
                    Err(_) => {
                        t_roster_new_us += t2c.elapsed().as_micros();
                        continue;
                    }
                };
                n_roster_constructions += 1;
                t_roster_new_us += t2c.elapsed().as_micros();

                metrics.record_evaluation();
                let t3 = std::time::Instant::now();
                let cost = self.evaluator.evaluate(&candidate);
                t_evaluate_us += t3.elapsed().as_micros();

                let weighted = if self.weights.is_empty() {
                    cost.sum()
                } else {
                    cost.weighted_sum(&self.weights)
                };

                if weighted < best_cost {
                    best_cost = weighted;
                    best_roster = Some(candidate);
                    metrics.record_improvement();
                    p_rot_improving += 1;
                    n_obj_improving += 1;
                } else if weighted == best_cost {
                    n_obj_tied += 1;
                } else {
                    n_obj_worse += 1;
                }
            }

            // Aggregate per-pairing counters into cross-pairing totals
            total_rot_examined += p_rot_examined;
            total_rot_legal += p_rot_legal;
            total_rot_improving += p_rot_improving;
            if p_rot_legal < min_rot_legal {
                min_rot_legal = p_rot_legal;
            }
            if p_rot_legal > max_rot_legal {
                max_rot_legal = p_rot_legal;
            }
            if p_rot_improving < min_rot_improving {
                min_rot_improving = p_rot_improving;
            }
            if p_rot_improving > max_rot_improving {
                max_rot_improving = p_rot_improving;
            }

            let t4 = std::time::Instant::now();
            if let Some(better) = best_roster {
                current = better;
            }
            t_commit_us += t4.elapsed().as_micros();
        }

        eprintln!(
            "  [greedy_profile] pairings={pairing_count} candidates={total_candidates} \
             rotation_collect={t_rotation_collect_us}µs \
             pairing_clone={t_pairing_clone_us}µs \
             rotation_clone={t_rotation_clone_us}µs \
             leg_clone={t_leg_clone_us}µs \
             roster_new={t_roster_new_us}µs \
             evaluate={t_evaluate_us}µs \
             commit={t_commit_us}µs | \
             n_pairing_vec_clones={n_pairing_vec_clones} \
             n_rotation_clones={n_rotation_clones} \
             n_leg_clones={n_leg_clones} \
             n_roster_constructions={n_roster_constructions}"
        );

        // Normalise min values: if no pairing was processed, leave as 0 rather than u64::MAX
        let min_rot_legal_out = if min_rot_legal == u64::MAX {
            0
        } else {
            min_rot_legal
        };
        let min_rot_improving_out = if min_rot_improving == u64::MAX {
            0
        } else {
            min_rot_improving
        };
        eprintln!(
            "  [greedy_feasibility] \
             rot_examined={total_rot_examined} \
             rot_legal={total_rot_legal} \
             rot_improving={total_rot_improving} \
             legal_min={min_rot_legal_out} legal_max={max_rot_legal} \
             improving_min={min_rot_improving_out} improving_max={max_rot_improving} \
             obj_improving={n_obj_improving} obj_tied={n_obj_tied} obj_worse={n_obj_worse}"
        );

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
        let d_new_out = make_duty(
            "D_new_out",
            vec![make_leg("L_new_out", "LHR", "CDG", 32, 34)],
        );
        let d_new_ret = make_duty(
            "D_new_ret",
            vec![make_leg("L_new_ret", "CDG", "LHR", 46, 48)],
        );
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
