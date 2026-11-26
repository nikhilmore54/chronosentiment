use crate::credit::duty_credit;
use crate::types::CrewMember;

/// Compute the total credited workload W_n for one crew member.
///
/// W_n = Σ_{d ∈ duties_n} duty_credit(d)
///
/// This implements R1 (Credited Workload Equation) from WP-M2.2 and
/// BENCHMARK-SEMANTICS-v1.0 §3. The sum covers all duties assigned to the
/// crew member in this solution. Each duty's credit is pre-computed by the
/// instance data loader and stored in `duty.credit`.
///
/// Returns 0.0 for a crew member with no duties.
/// Returns a non-negative f64 (guaranteed by the `duty_credit` invariant).
pub fn credited_workload(member: &CrewMember) -> f64 {
    member.duties.iter().map(|d| duty_credit(d)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CrewMember, Duty};

    fn make_member(duties: Vec<f64>) -> CrewMember {
        CrewMember {
            id: 1,
            min_workload: 0.0,
            max_workload: 1000.0,
            target_workload: 500.0,
            duties: duties
                .into_iter()
                .enumerate()
                .map(|(i, credit)| Duty {
                    id: i as u32 + 1,
                    credit,
                    legs: vec![],
                })
                .collect(),
        }
    }

    /// O2: Zero-duty crew member → W_n = 0.0
    #[test]
    fn o2_zero_duty_member() {
        let m = make_member(vec![]);
        assert_eq!(credited_workload(&m), 0.0);
    }

    /// O3: Three duties with credits 30/45/60 → W_n = 135.0
    #[test]
    fn o3_three_duties_sum() {
        let m = make_member(vec![30.0, 45.0, 60.0]);
        assert!((credited_workload(&m) - 135.0).abs() < 1e-9);
    }

    #[test]
    fn single_duty() {
        let m = make_member(vec![90.0]);
        assert!((credited_workload(&m) - 90.0).abs() < 1e-9);
    }

    #[test]
    fn fractional_credits_sum() {
        let m = make_member(vec![33.3, 33.3, 33.4]);
        assert!((credited_workload(&m) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn large_crew_member() {
        // 20 duties of 30 minutes each → W_n = 600
        let m = make_member(vec![30.0; 20]);
        assert!((credited_workload(&m) - 600.0).abs() < 1e-9);
    }
}
