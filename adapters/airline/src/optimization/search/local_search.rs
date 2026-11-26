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
        Self {
            evaluator,
            checker,
            weights,
            max_iterations,
        }
    }

    /// Run local search from `initial` and return the best roster found.
    pub fn run(&self, initial: &Roster, metrics: &mut OptimizationMetrics) -> Roster {
        let mut current = initial.clone();
        let mut current_cost = self.weighted_cost(&current);

        // Sub-stage timing accumulators (cumulative across all iterations).
        let mut t_neighbour_us: u128 = 0;
        let mut t_legality_us: u128 = 0;
        let mut t_evaluate_us: u128 = 0;
        let mut move_count: u64 = 0;

        for _ in 0..self.max_iterations {
            let improved = self.best_improving_move(
                &current,
                current_cost,
                metrics,
                &mut t_neighbour_us,
                &mut t_legality_us,
                &mut t_evaluate_us,
                &mut move_count,
            );
            match improved {
                Some((better, cost)) => {
                    current = better;
                    current_cost = cost;
                    metrics.record_improvement();
                }
                None => break, // local optimum
            }
        }

        if move_count > 0 {
            eprintln!(
                "  [local_search_profile] moves={move_count} \
                 neighbour_gen={t_neighbour_us}µs legality={t_legality_us}µs \
                 evaluate={t_evaluate_us}µs"
            );
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

    /// Find the best improving move from the current roster using lazy descriptors.
    ///
    /// Phase 1: enumerate all (i, pa, j, pb) swap descriptors and compute the
    /// workload-balance variance delta analytically — O(1) per descriptor, no
    /// roster clone.  Track the best-delta descriptor.
    ///
    /// Phase 2: materialize the best descriptor (one `swap_pairings` call),
    /// verify legality, and accept if feasible and cheaper.
    ///
    /// This reduces cloning from O(moves × roster_size) to O(roster_size) per
    /// iteration — a factor of ~moves improvement over the eager approach.
    #[allow(clippy::too_many_arguments)]
    fn best_improving_move(
        &self,
        current: &Roster,
        current_cost: f64,
        metrics: &mut OptimizationMetrics,
        t_neighbour_us: &mut u128,
        t_legality_us: &mut u128,
        t_evaluate_us: &mut u128,
        move_count: &mut u64,
    ) -> Option<(Roster, f64)> {
        let rotations: Vec<_> = current.rotations().collect();
        let n = rotations.len();

        // Precompute per-rotation leg counts for O(1) delta evaluation.
        // leg_counts[k] = number of legs assigned to rotation k.
        let leg_counts: Vec<f64> = rotations
            .iter()
            .map(|r| {
                r.pairings()
                    .iter()
                    .flat_map(|p| p.duties().iter())
                    .flat_map(|d| d.legs().iter())
                    .count() as f64
            })
            .collect();
        let total_legs: f64 = leg_counts.iter().sum();
        let mean = if n > 0 { total_legs / n as f64 } else { 0.0 };

        // Precompute per-pairing leg counts for O(1) delta lookup.
        // pairing_legs[k][p] = leg count of pairing p in rotation k.
        let pairing_legs: Vec<Vec<f64>> = rotations
            .iter()
            .map(|r| {
                r.pairings()
                    .iter()
                    .map(|p| p.duties().iter().flat_map(|d| d.legs().iter()).count() as f64)
                    .collect()
            })
            .collect();

        // ── Phase 1: scan descriptors, compute variance delta analytically ────
        //
        // For a swap of pairing pa (with la legs) from rotation i with pairing
        // pb (with lb legs) from rotation j:
        //   c_i' = c_i - la + lb
        //   c_j' = c_j - lb + la
        //   Δvariance = [(c_i'-mean)² + (c_j'-mean)² - (c_i-mean)² - (c_j-mean)²] / n
        //
        // Mean is invariant under swap (total legs unchanged).
        //
        // We collect ALL improving descriptors (delta < -1e-9) into a ranked Vec
        // sorted by delta (most negative first).  Phase 2 then tries them in order
        // until one materializes successfully via swap_pairings().  This ensures we
        // never miss the second-best swap just because the best one is structurally
        // invalid (swap_pairings returns None for some descriptors).
        //
        // The -1e-9 threshold guards against floating-point noise: equal-leg swaps
        // (la == lb) produce delta ≈ -1e-17 due to cancellation errors.

        // Collect all improving descriptors: (delta, i, pa, j, pb).
        let mut improving_swaps: Vec<(f64, usize, usize, usize, usize)> = Vec::new();

        let t0 = std::time::Instant::now();
        for i in 0..n {
            let ci = leg_counts[i];
            let pi_count = pairing_legs[i].len();
            for j in (i + 1)..n {
                let cj = leg_counts[j];
                let pj_count = pairing_legs[j].len();
                for pa in 0..pi_count {
                    let la = pairing_legs[i][pa];
                    for pb in 0..pj_count {
                        let lb = pairing_legs[j][pb];
                        *move_count += 1;
                        metrics.record_evaluation();
                        metrics.record_feasibility_check();

                        // Analytical variance delta (no clone).
                        let ci_new = ci - la + lb;
                        let cj_new = cj - lb + la;
                        let delta = ((ci_new - mean).powi(2) + (cj_new - mean).powi(2)
                            - (ci - mean).powi(2)
                            - (cj - mean).powi(2))
                            / n as f64;

                        // Collect all descriptors with a meaningful negative delta.
                        if delta < -1e-9 {
                            improving_swaps.push((delta, i, pa, j, pb));
                        }
                    }
                }
            }
        }

        // Sort by delta ascending (most negative = best improvement first).
        improving_swaps.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        *t_neighbour_us += t0.elapsed().as_micros();

        // ── Phase 2: try ranked swap descriptors until one materializes ───────
        //
        // Iterate through improving_swaps (sorted best-delta first).  For each
        // descriptor, call swap_pairings().  If it returns None (structurally
        // invalid), skip and try the next.  Accept the first valid, legal,
        // cost-improving swap.  Only after exhausting all improving swap
        // descriptors do we fall back to relocate.
        for (_delta, i, pa, j, pb) in &improving_swaps {
            let t1 = std::time::Instant::now();
            let candidate_opt = swap::swap_pairings(current, *i, *pa, *j, *pb);
            *t_neighbour_us += t1.elapsed().as_micros();
            if let Some(candidate) = candidate_opt {
                metrics.record_feasibility_check();
                let t2 = std::time::Instant::now();
                let legal = self.checker.is_legal(&candidate);
                *t_legality_us += t2.elapsed().as_micros();
                if legal {
                    let t3 = std::time::Instant::now();
                    let cost = self.weighted_cost(&candidate);
                    *t_evaluate_us += t3.elapsed().as_micros();
                    if cost < current_cost {
                        return Some((candidate, cost));
                    }
                }
            }
            // swap_pairings returned None or legality/cost check failed:
            // continue to next-best descriptor.
        }

        // ── Relocate moves: fallback after exhausting all improving swaps ─────
        // Relocate changes the mean (total legs per rotation changes), so the
        // analytical delta is more complex.  We keep the eager approach for
        // relocate and only evaluate it when no improving swap was found.
        let mut best_relocate: Option<(Roster, f64)> = None;
        for src in 0..n {
            let p_count = rotations[src].pairings().len();
            for dst in 0..n {
                if src == dst {
                    continue;
                }
                for pi in 0..p_count {
                    let t1 = std::time::Instant::now();
                    let candidate_opt = relocate::relocate_pairing(current, src, pi, dst);
                    *t_neighbour_us += t1.elapsed().as_micros();
                    if let Some(candidate) = candidate_opt {
                        metrics.record_feasibility_check();
                        let t2 = std::time::Instant::now();
                        let legal = self.checker.is_legal(&candidate);
                        *t_legality_us += t2.elapsed().as_micros();
                        if legal {
                            let t3 = std::time::Instant::now();
                            let cost = self.weighted_cost(&candidate);
                            *t_evaluate_us += t3.elapsed().as_micros();
                            if cost < current_cost {
                                match &best_relocate {
                                    Some((_, bc)) if cost >= *bc => {}
                                    _ => {
                                        best_relocate = Some((candidate, cost));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        best_relocate
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
