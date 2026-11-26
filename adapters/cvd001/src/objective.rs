use crate::types::CrewMember;

/// Compute the workload balance deviation for one crew member.
///
/// Δ_n = |W_n − t_n|
///
/// `workload` is the pre-computed W_n (from [`crate::workload::credited_workload`]).
/// `member.target_workload` is t_n.
///
/// This is the per-member contribution to the benchmark objective Z.
/// Mathematical basis: R2 (WP-M2.3), BENCHMARK-SEMANTICS-v1.0 §4.
pub fn workload_deviation(member: &CrewMember, workload: f64) -> f64 {
    (workload - member.target_workload).abs()
}

/// Compute the benchmark objective Z = Σ_n Δ_n.
///
/// This implements R2 (Objective Function) from WP-M2.3 and
/// BENCHMARK-SEMANTICS-v1.0 §4, with α = 0 and β = 1 (benchmark adapter
/// defaults). The cost term α·cost_n is omitted (α = 0).
///
/// `crew` is the slice of crew members (same order as `workloads`).
/// `workloads` is the pre-computed W_n slice (one entry per crew member,
/// produced by calling [`crate::workload::credited_workload`] for each).
///
/// # Design
/// The objective takes pre-computed workloads as a separate slice rather than
/// recomputing them internally. This allows the caller (`evaluator.rs`) to
/// reuse the workload vector for both HC3 checking and objective computation
/// without redundant summation.
///
/// # Panics
/// Panics if `crew.len() != workloads.len()`.
pub fn objective(crew: &[CrewMember], workloads: &[f64]) -> f64 {
    assert_eq!(
        crew.len(),
        workloads.len(),
        "crew and workloads slices must have equal length: {} vs {}",
        crew.len(),
        workloads.len()
    );
    crew.iter()
        .zip(workloads.iter())
        .map(|(m, &w)| workload_deviation(m, w))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CrewMember;

    fn make_member(id: u32, target: f64) -> CrewMember {
        CrewMember {
            id,
            min_workload: 0.0,
            max_workload: 1000.0,
            target_workload: target,
            duties: vec![],
        }
    }

    /// O4: W_n == t_n exactly → Δ_n = 0.0
    #[test]
    fn o4_workload_equals_target() {
        let m = make_member(1, 500.0);
        assert_eq!(workload_deviation(&m, 500.0), 0.0);
    }

    /// O5: W_n > t_n by 10.0 → Δ_n = 10.0
    #[test]
    fn o5_workload_above_target() {
        let m = make_member(1, 500.0);
        assert!((workload_deviation(&m, 510.0) - 10.0).abs() < 1e-9);
    }

    /// O6: W_n < t_n by 10.0 → Δ_n = 10.0
    #[test]
    fn o6_workload_below_target() {
        let m = make_member(1, 500.0);
        assert!((workload_deviation(&m, 490.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn objective_empty_crew() {
        assert_eq!(objective(&[], &[]), 0.0);
    }

    #[test]
    fn objective_two_members() {
        // Member 1: W=510, t=500 → Δ=10
        // Member 2: W=480, t=500 → Δ=20
        // Z = 30
        let crew = vec![make_member(1, 500.0), make_member(2, 500.0)];
        let workloads = vec![510.0, 480.0];
        assert!((objective(&crew, &workloads) - 30.0).abs() < 1e-9);
    }

    #[test]
    fn objective_all_on_target() {
        let crew = vec![make_member(1, 400.0), make_member(2, 600.0)];
        let workloads = vec![400.0, 600.0];
        assert_eq!(objective(&crew, &workloads), 0.0);
    }

    #[test]
    #[should_panic(expected = "crew and workloads slices must have equal length")]
    fn objective_length_mismatch_panics() {
        let crew = vec![make_member(1, 500.0)];
        let workloads = vec![500.0, 500.0];
        objective(&crew, &workloads);
    }
}
