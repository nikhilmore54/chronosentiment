use crate::types::{ConstraintViolation, CrewMember};

/// Check HC3 for a single crew member.
///
/// HC3-A (preferred reconstruction): W_n ≤ W^max_n.
///
/// Returns `Some(ConstraintViolation)` if `workload > member.max_workload`,
/// `None` if the constraint is satisfied.
///
/// The boundary condition W_n == W^max_n is **feasible** (≤, not <).
///
/// `workload` is the pre-computed W_n (from [`crate::workload::credited_workload`]).
/// `index` is the position of this crew member in the solution's crew slice
/// (used to populate `ConstraintViolation::crew_member_index`).
///
/// Mathematical basis: R3 (WP-M2.4), BENCHMARK-SEMANTICS-v1.0 §5.
pub fn hc3_check_member(
    member: &CrewMember,
    workload: f64,
    index: usize,
) -> Option<ConstraintViolation> {
    if workload > member.max_workload {
        Some(ConstraintViolation {
            constraint: "HC3",
            crew_member_index: index,
            crew_member_id: member.id,
            workload,
            threshold: member.max_workload,
        })
    } else {
        None
    }
}

/// Collect all HC3 violations across the crew roster.
///
/// Returns an empty `Vec` if all crew members satisfy W_n ≤ W^max_n.
/// Returns one [`ConstraintViolation`] per violating crew member, in crew order.
///
/// HC3 is a hard constraint: if any violation is present, the solution is
/// infeasible and the objective value is not meaningful.
///
/// `crew` is the slice of crew members (same order as `workloads`).
/// `workloads` is the pre-computed W_n slice (one entry per crew member).
///
/// # Panics
/// Panics if `crew.len() != workloads.len()`.
pub fn hc3_violations(crew: &[CrewMember], workloads: &[f64]) -> Vec<ConstraintViolation> {
    assert_eq!(
        crew.len(),
        workloads.len(),
        "crew and workloads slices must have equal length: {} vs {}",
        crew.len(),
        workloads.len()
    );
    crew.iter()
        .zip(workloads.iter())
        .enumerate()
        .filter_map(|(i, (m, &w))| hc3_check_member(m, w, i))
        .collect()
}

/// Convenience predicate: `true` iff no HC3 violations exist.
///
/// Equivalent to `hc3_violations(crew, workloads).is_empty()`.
/// Prefer [`hc3_violations`] when you need diagnostic information.
pub fn hc3_feasible(crew: &[CrewMember], workloads: &[f64]) -> bool {
    hc3_violations(crew, workloads).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CrewMember;

    fn make_member(id: u32, max_workload: f64) -> CrewMember {
        CrewMember {
            id,
            min_workload: 0.0,
            max_workload,
            target_workload: max_workload * 0.9,
            duties: vec![],
        }
    }

    /// O7: W_n == W^max_n (boundary) → feasible (None violation)
    #[test]
    fn o7_workload_at_cap_is_feasible() {
        let m = make_member(1, 600.0);
        assert!(hc3_check_member(&m, 600.0, 0).is_none());
    }

    /// O8: W_n > W^max_n by a representable delta → infeasible (Some violation)
    ///
    /// Note: f64::EPSILON is the machine epsilon relative to 1.0 (≈2.2e-16).
    /// At magnitude 600.0, the smallest representable increment is
    /// 600.0 * f64::EPSILON ≈ 1.3e-13. We use 1e-9 as a safe, clearly
    /// representable delta that is unambiguously above 600.0.
    #[test]
    fn o8_workload_above_cap_is_infeasible() {
        let m = make_member(1, 600.0);
        let v = hc3_check_member(&m, 600.0 + 1e-9, 0);
        assert!(v.is_some());
        let v = v.unwrap();
        assert_eq!(v.constraint, "HC3");
        assert_eq!(v.crew_member_id, 1);
        assert_eq!(v.crew_member_index, 0);
        assert!((v.threshold - 600.0).abs() < 1e-9);
    }

    #[test]
    fn workload_well_below_cap_is_feasible() {
        let m = make_member(1, 600.0);
        assert!(hc3_check_member(&m, 400.0, 0).is_none());
    }

    #[test]
    fn hc3_violations_empty_crew() {
        let violations = hc3_violations(&[], &[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn hc3_violations_all_within_cap() {
        let crew = vec![make_member(1, 600.0), make_member(2, 700.0)];
        let workloads = vec![500.0, 600.0];
        assert!(hc3_violations(&crew, &workloads).is_empty());
    }

    #[test]
    fn hc3_violations_one_violates() {
        let crew = vec![make_member(1, 600.0), make_member(2, 700.0)];
        // Member 2 (index 1) exceeds cap
        let workloads = vec![500.0, 750.0];
        let violations = hc3_violations(&crew, &workloads);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].crew_member_index, 1);
        assert_eq!(violations[0].crew_member_id, 2);
        assert!((violations[0].workload - 750.0).abs() < 1e-9);
        assert!((violations[0].threshold - 700.0).abs() < 1e-9);
    }

    #[test]
    fn hc3_violations_both_violate() {
        let crew = vec![make_member(1, 500.0), make_member(2, 600.0)];
        let workloads = vec![600.0, 700.0];
        let violations = hc3_violations(&crew, &workloads);
        assert_eq!(violations.len(), 2);
        assert_eq!(violations[0].crew_member_index, 0);
        assert_eq!(violations[1].crew_member_index, 1);
    }

    #[test]
    fn hc3_violations_all_at_boundary_feasible() {
        let crew = vec![make_member(1, 600.0), make_member(2, 700.0)];
        let workloads = vec![600.0, 700.0];
        assert!(hc3_violations(&crew, &workloads).is_empty());
    }

    #[test]
    fn hc3_feasible_convenience_predicate() {
        let crew = vec![make_member(1, 600.0)];
        assert!(hc3_feasible(&crew, &[500.0]));
        assert!(!hc3_feasible(&crew, &[700.0]));
    }

    #[test]
    #[should_panic(expected = "crew and workloads slices must have equal length")]
    fn hc3_violations_length_mismatch_panics() {
        let crew = vec![make_member(1, 600.0)];
        let workloads = vec![500.0, 600.0];
        hc3_violations(&crew, &workloads);
    }
}
