//! CVD-001 framework adapter — M4.2
//!
//! Implements [`coralys_eval::BenchmarkAdapter`] for the CVD-001 reference evaluator,
//! registering it with the Coralys Evaluation Framework.
//!
//! # Design
//!
//! The CVD-001 benchmark encodes all problem parameters (W^min_n, W^max_n, t_n)
//! inside the `Solution` type itself (via `CrewMember` fields). There is no
//! separate problem file. Therefore:
//!
//! - `Problem = ()` (unit) — no separate problem input required.
//! - `Solution = crate::types::Solution` — carries all crew contract parameters.
//!
//! This is correct for the benchmark. Future production adapters may use richer
//! `Problem` types that separate instance data from solution data.
//!
//! # Mapping: cvd001::EvaluationResult → coralys_eval::EvaluationResult
//!
//! | cvd001 field          | coralys_eval field                          |
//! |-----------------------|---------------------------------------------|
//! | `feasible`            | `feasible` (derived from violations)        |
//! | `objective`           | `objectives[0]` ("workload_balance", Z)     |
//! | `violations[i]`       | `violations[i]` (ConstraintViolation)       |
//! | `workloads[i]`        | `metrics["workload_{i}"]`                   |
//!
//! Infeasible solutions set `objectives[0].value = f64::INFINITY` to preserve
//! the standalone adapter's sentinel convention.
//!
//! # Parity guarantee
//!
//! The parity tests in this module assert:
//!   `standalone::evaluate(sol).feasible == framework_adapter.evaluate(&(), sol).feasible`
//!   `standalone::evaluate(sol).objective == framework_adapter.evaluate(&(), sol).primary_objective()`
//!   `standalone::evaluate(sol).violations.len() == framework_adapter.evaluate(&(), sol).violations.len()`
//!
//! for all oracle inputs (OI1, OI2, O1–O8).

use coralys_eval::{
    BenchmarkAdapter,
    ConstraintViolation as EvalViolation,
    EvaluationResult as EvalResult,
    ObjectiveValue,
};
use crate::evaluator::evaluate as standalone_evaluate;
use crate::types::Solution;

// ---------------------------------------------------------------------------
// Cvd001FrameworkAdapter
// ---------------------------------------------------------------------------

/// The CVD-001 framework adapter.
///
/// Implements [`BenchmarkAdapter`] so that `adapters/cvd001` can be registered
/// with the Coralys Evaluation Framework and evaluated via
/// [`coralys_eval::EvaluationPipeline`].
pub struct Cvd001FrameworkAdapter;

impl BenchmarkAdapter for Cvd001FrameworkAdapter {
    /// No separate problem file — all parameters are embedded in `Solution`.
    type Problem = ();
    type Solution = Solution;

    fn adapter_id(&self) -> &'static str {
        "cvd001"
    }

    fn adapter_name(&self) -> &'static str {
        "CVD-001 Reference Adapter (GERAD G-2014-22)"
    }

    fn adapter_version(&self) -> &'static str {
        "1.0.0"
    }

    fn evaluate(
        &self,
        _problem: &Self::Problem,
        solution: &Self::Solution,
    ) -> EvalResult {
        // Delegate to the frozen standalone evaluator — no logic duplication.
        let r = standalone_evaluate(solution);

        // Map violations: cvd001::ConstraintViolation → coralys_eval::ConstraintViolation
        let violations: Vec<EvalViolation> = r.violations.iter().map(|v| {
            let mut ev = EvalViolation::hard(
                v.constraint,
                v.constraint,
                v.workload,
                v.threshold,
            );
            ev.entity_id = Some(v.crew_member_id as u64);
            ev.entity_index = Some(v.crew_member_index);
            ev
        }).collect();

        // Map objective: Z = Σ_n Δ_n (INFINITY when infeasible)
        let obj_value = if r.feasible { r.objective } else { f64::INFINITY };
        let objectives = vec![
            ObjectiveValue::new("workload_balance", "Workload Balance (Z = Σ|W_n − t_n|)", obj_value),
        ];

        // Build the framework result; feasible is derived from violations.
        let mut result = EvalResult::new(self.adapter_id(), objectives, violations);

        // Store per-crew workloads as named metrics for diagnostics.
        for (i, &w) in r.workloads.iter().enumerate() {
            result.metrics.insert(format!("workload_{i}"), w);
        }

        result
    }
}

// ---------------------------------------------------------------------------
// Parity tests — M4.2 / M4.3 oracle gate
//
// Each test asserts that the framework adapter produces identical results to
// the standalone evaluator for the same input. These are the permanent
// regression tests required by M4.3.
//
// Oracle inputs:
//   OI1, OI2 — from evaluator.rs (inline integration tests)
//   O1–O8    — representative cases covering boundary conditions
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use coralys_eval::ConstraintSeverity;
    use crate::types::{CrewMember, Duty, Solution};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn make_duty(id: u32, credit: f64) -> Duty {
        Duty { id, credit, legs: vec![] }
    }

    fn make_member(id: u32, max_workload: f64, target: f64, duty_credits: Vec<f64>) -> CrewMember {
        CrewMember {
            id,
            min_workload: 0.0,
            max_workload,
            target_workload: target,
            duties: duty_credits
                .into_iter()
                .enumerate()
                .map(|(i, c)| make_duty(i as u32 + 1, c))
                .collect(),
        }
    }

    /// Assert parity between standalone and framework adapter for a given solution.
    fn assert_parity(solution: &Solution, label: &str) {
        let standalone = standalone_evaluate(solution);
        let adapter = Cvd001FrameworkAdapter;
        let framework = adapter.evaluate(&(), solution);

        assert_eq!(
            standalone.feasible, framework.feasible,
            "{label}: feasible mismatch"
        );
        assert_eq!(
            standalone.violations.len(),
            framework.violations.len(),
            "{label}: violation count mismatch"
        );

        // Objective parity (both INFINITY when infeasible, or equal finite value)
        let standalone_obj = standalone.objective;
        let framework_obj = framework.primary_objective();
        if standalone_obj.is_infinite() {
            assert!(framework_obj.is_infinite(), "{label}: expected INFINITY objective");
        } else {
            assert!(
                (standalone_obj - framework_obj).abs() < 1e-9,
                "{label}: objective mismatch: standalone={standalone_obj}, framework={framework_obj}"
            );
        }

        // Violation field parity
        for (i, (sv, fv)) in standalone.violations.iter().zip(framework.violations.iter()).enumerate() {
            assert_eq!(
                sv.crew_member_id, fv.entity_id.unwrap() as u32,
                "{label}: violation[{i}] crew_member_id mismatch"
            );
            assert_eq!(
                sv.crew_member_index, fv.entity_index.unwrap(),
                "{label}: violation[{i}] crew_member_index mismatch"
            );
            assert!(
                (sv.workload - fv.observed).abs() < 1e-9,
                "{label}: violation[{i}] workload/observed mismatch"
            );
            assert!(
                (sv.threshold - fv.threshold).abs() < 1e-9,
                "{label}: violation[{i}] threshold mismatch"
            );
            assert_eq!(
                fv.severity, ConstraintSeverity::Hard,
                "{label}: violation[{i}] should be Hard"
            );
        }

        // Workload metrics parity
        for (i, &w) in standalone.workloads.iter().enumerate() {
            let key = format!("workload_{i}");
            let metric = framework.metrics.get(&key).copied().unwrap_or(f64::NAN);
            assert!(
                (w - metric).abs() < 1e-9,
                "{label}: workload metric[{i}] mismatch: standalone={w}, framework={metric}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // OI1 — All crew feasible (from evaluator.rs oracle)
    // -----------------------------------------------------------------------

    #[test]
    fn oi1_parity_all_feasible() {
        let solution = Solution {
            crew: vec![
                make_member(1, 500.0, 200.0, vec![60.0, 60.0, 60.0]),
                make_member(2, 500.0, 200.0, vec![100.0, 100.0]),
                make_member(3, 500.0, 100.0, vec![50.0, 60.0]),
            ],
        };
        assert_parity(&solution, "OI1");
    }

    // -----------------------------------------------------------------------
    // OI2 — HC3 violation (from evaluator.rs oracle)
    // -----------------------------------------------------------------------

    #[test]
    fn oi2_parity_hc3_violation() {
        let solution = Solution {
            crew: vec![
                make_member(1, 500.0, 400.0, vec![300.0, 300.0]),
                make_member(2, 500.0, 100.0, vec![100.0]),
            ],
        };
        assert_parity(&solution, "OI2");
    }

    // -----------------------------------------------------------------------
    // O1 — Empty solution
    // -----------------------------------------------------------------------

    #[test]
    fn o1_parity_empty_solution() {
        let solution = Solution { crew: vec![] };
        assert_parity(&solution, "O1");
    }

    // -----------------------------------------------------------------------
    // O2 — Single member, no duties
    // -----------------------------------------------------------------------

    #[test]
    fn o2_parity_single_member_no_duties() {
        let solution = Solution {
            crew: vec![make_member(1, 500.0, 200.0, vec![])],
        };
        assert_parity(&solution, "O2");
    }

    // -----------------------------------------------------------------------
    // O3 — Single member at cap boundary (W_n == W^max_n)
    // -----------------------------------------------------------------------

    #[test]
    fn o3_parity_at_cap_boundary() {
        let solution = Solution {
            crew: vec![make_member(1, 300.0, 300.0, vec![150.0, 150.0])],
        };
        assert_parity(&solution, "O3");
    }

    // -----------------------------------------------------------------------
    // O4 — Single member exceeds cap (W_n > W^max_n)
    // -----------------------------------------------------------------------

    #[test]
    fn o4_parity_exceeds_cap() {
        let solution = Solution {
            crew: vec![make_member(1, 100.0, 80.0, vec![150.0])],
        };
        assert_parity(&solution, "O4");
    }

    // -----------------------------------------------------------------------
    // O5 — Multiple members, all at target (Z = 0)
    // -----------------------------------------------------------------------

    #[test]
    fn o5_parity_all_at_target() {
        let solution = Solution {
            crew: vec![
                make_member(1, 500.0, 100.0, vec![100.0]),
                make_member(2, 500.0, 200.0, vec![100.0, 100.0]),
                make_member(3, 500.0, 300.0, vec![100.0, 100.0, 100.0]),
            ],
        };
        assert_parity(&solution, "O5");
    }

    // -----------------------------------------------------------------------
    // O6 — Multiple members, all violate HC3
    // -----------------------------------------------------------------------

    #[test]
    fn o6_parity_all_violate_hc3() {
        let solution = Solution {
            crew: vec![
                make_member(10, 100.0, 80.0, vec![150.0]),
                make_member(20, 200.0, 150.0, vec![250.0]),
                make_member(30, 50.0, 40.0, vec![60.0]),
            ],
        };
        assert_parity(&solution, "O6");
    }

    // -----------------------------------------------------------------------
    // O7 — Mixed: some feasible, one violates HC3
    // -----------------------------------------------------------------------

    #[test]
    fn o7_parity_mixed_feasibility() {
        let solution = Solution {
            crew: vec![
                make_member(1, 500.0, 200.0, vec![100.0, 100.0]),  // feasible
                make_member(2, 100.0, 80.0, vec![150.0]),           // HC3 violation
                make_member(3, 500.0, 300.0, vec![150.0, 150.0]),   // feasible
            ],
        };
        assert_parity(&solution, "O7");
    }

    // -----------------------------------------------------------------------
    // O8 — Large objective value (many members, large deviations)
    // -----------------------------------------------------------------------

    #[test]
    fn o8_parity_large_objective() {
        let solution = Solution {
            crew: (1..=10u32).map(|i| {
                // Each member has W_n = 100, t_n = 0 → Δ_n = 100
                make_member(i, 1000.0, 0.0, vec![100.0])
            }).collect(),
        };
        assert_parity(&solution, "O8");
    }

    // -----------------------------------------------------------------------
    // Framework-specific: adapter_id is set correctly in result
    // -----------------------------------------------------------------------

    #[test]
    fn framework_result_has_correct_adapter_id() {
        let solution = Solution { crew: vec![] };
        let adapter = Cvd001FrameworkAdapter;
        let result = adapter.evaluate(&(), &solution);
        assert_eq!(result.adapter_id, "cvd001");
    }

    // -----------------------------------------------------------------------
    // Framework-specific: objective_id is "workload_balance"
    // -----------------------------------------------------------------------

    #[test]
    fn framework_result_objective_id_is_workload_balance() {
        let solution = Solution {
            crew: vec![make_member(1, 500.0, 100.0, vec![100.0])],
        };
        let adapter = Cvd001FrameworkAdapter;
        let result = adapter.evaluate(&(), &solution);
        assert_eq!(result.objective_by_id("workload_balance"), Some(0.0));
    }
}