use crate::types::{Solution, EvaluationResult};
use crate::workload::credited_workload;
use crate::hc3::hc3_violations;
use crate::objective::objective;

/// Evaluate a solution against the CVD-001 benchmark.
///
/// # Execution flow
///
/// 1. Compute W_n for each crew member via [`credited_workload`].
/// 2. Collect HC3 violations: W_n > W^max_n for any n ∈ N.
///    If any violations exist: return infeasible result
///    (`violations` populated, `objective = f64::INFINITY`).
/// 3. Compute Z = Σ_n |W_n − t_n| via [`objective`].
/// 4. Return feasible result (`violations` empty, `objective = Z`).
///
/// # Returns
///
/// [`EvaluationResult`] with:
/// - `workloads`: W_n per crew member (always populated, even when infeasible)
/// - `violations`: structured HC3 violation records (empty iff feasible)
/// - `feasible`: `true` iff `violations` is empty
/// - `objective`: Z if feasible; `f64::INFINITY` if infeasible
///
/// # Empty solution
///
/// A solution with no crew members is feasible with Z = 0.0 and no violations.
///
/// Mathematical basis: BENCHMARK-REFERENCE-SPECIFICATION-v1.0 §5.5,
/// BENCHMARK-SEMANTICS-v1.0 §6.
pub fn evaluate(solution: &Solution) -> EvaluationResult {
    // Step 1: compute credited workload W_n for each crew member
    let workloads: Vec<f64> = solution
        .crew
        .iter()
        .map(|m| credited_workload(m))
        .collect();

    // Step 2: collect HC3 violations — W_n > W^max_n
    let violations = hc3_violations(&solution.crew, &workloads);

    if !violations.is_empty() {
        return EvaluationResult {
            workloads,
            violations,
            feasible: false,
            objective: f64::INFINITY,
        };
    }

    // Step 3: objective computation — Z = Σ_n |W_n − t_n|
    let z = objective(&solution.crew, &workloads);

    EvaluationResult {
        workloads,
        violations: vec![],
        feasible: true,
        objective: z,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Duty, CrewMember, Solution};

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

    /// OI1: All crew feasible — Z = Σ Δ_n
    ///
    /// Setup:
    ///   Member 1: duties [60, 60, 60] → W=180, t=200, Δ=20
    ///   Member 2: duties [100, 100]   → W=200, t=200, Δ=0
    ///   Member 3: duties [50, 60]     → W=110, t=100, Δ=10
    ///   All max_workload = 500 (no HC3 violation)
    ///   Expected Z = 20 + 0 + 10 = 30
    #[test]
    fn oi1_all_feasible_objective_sum() {
        let solution = Solution {
            crew: vec![
                make_member(1, 500.0, 200.0, vec![60.0, 60.0, 60.0]),
                make_member(2, 500.0, 200.0, vec![100.0, 100.0]),
                make_member(3, 500.0, 100.0, vec![50.0, 60.0]),
            ],
        };

        let result = evaluate(&solution);

        assert!(result.feasible, "solution should be feasible");
        assert!(result.violations.is_empty(), "no violations expected");
        assert_eq!(result.workloads.len(), 3);
        assert!((result.workloads[0] - 180.0).abs() < 1e-9, "W_1 should be 180");
        assert!((result.workloads[1] - 200.0).abs() < 1e-9, "W_2 should be 200");
        assert!((result.workloads[2] - 110.0).abs() < 1e-9, "W_3 should be 110");
        assert!((result.objective - 30.0).abs() < 1e-9, "Z should be 30");
    }

    /// OI2: One crew member violates HC3 → infeasible, objective = INFINITY
    ///
    /// Setup:
    ///   Member 1: duties [300, 300] → W=600, max=500 → HC3 VIOLATED
    ///   Member 2: duties [100]      → W=100, max=500 → OK
    #[test]
    fn oi2_hc3_violation_infeasible() {
        let solution = Solution {
            crew: vec![
                make_member(1, 500.0, 400.0, vec![300.0, 300.0]),
                make_member(2, 500.0, 100.0, vec![100.0]),
            ],
        };

        let result = evaluate(&solution);

        assert!(!result.feasible, "solution should be infeasible");
        assert!(result.objective.is_infinite(), "objective should be INFINITY");
        assert_eq!(result.violations.len(), 1, "exactly one HC3 violation");
        assert_eq!(result.violations[0].constraint, "HC3");
        assert_eq!(result.violations[0].crew_member_id, 1);
        assert_eq!(result.violations[0].crew_member_index, 0);
        assert!((result.violations[0].workload - 600.0).abs() < 1e-9);
        assert!((result.violations[0].threshold - 500.0).abs() < 1e-9);
        // workloads are still populated
        assert_eq!(result.workloads.len(), 2);
        assert!((result.workloads[0] - 600.0).abs() < 1e-9);
        assert!((result.workloads[1] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn empty_solution_is_feasible_with_zero_objective() {
        let solution = Solution { crew: vec![] };
        let result = evaluate(&solution);
        assert!(result.feasible);
        assert!(result.violations.is_empty());
        assert_eq!(result.objective, 0.0);
        assert!(result.workloads.is_empty());
    }

    #[test]
    fn single_member_at_cap_boundary_feasible() {
        // W_n == W^max_n exactly → feasible (boundary condition)
        let solution = Solution {
            crew: vec![make_member(1, 300.0, 300.0, vec![150.0, 150.0])],
        };
        let result = evaluate(&solution);
        assert!(result.feasible);
        assert!(result.violations.is_empty());
        assert!((result.workloads[0] - 300.0).abs() < 1e-9);
        assert_eq!(result.objective, 0.0); // W == t == max
    }

    #[test]
    fn single_member_no_duties_feasible() {
        // W_n = 0, max = 500, t = 200 → feasible, Δ = 200
        let solution = Solution {
            crew: vec![make_member(1, 500.0, 200.0, vec![])],
        };
        let result = evaluate(&solution);
        assert!(result.feasible);
        assert!(result.violations.is_empty());
        assert_eq!(result.workloads[0], 0.0);
        assert!((result.objective - 200.0).abs() < 1e-9);
    }

    #[test]
    fn multiple_hc3_violations_all_reported() {
        // Both members violate HC3
        let solution = Solution {
            crew: vec![
                make_member(10, 100.0, 80.0, vec![150.0]),  // W=150 > max=100
                make_member(20, 200.0, 150.0, vec![250.0]), // W=250 > max=200
            ],
        };
        let result = evaluate(&solution);
        assert!(!result.feasible);
        assert_eq!(result.violations.len(), 2);
        assert_eq!(result.violations[0].crew_member_id, 10);
        assert_eq!(result.violations[1].crew_member_id, 20);
        assert!(result.objective.is_infinite());
    }
}