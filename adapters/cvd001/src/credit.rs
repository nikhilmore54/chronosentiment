use crate::types::Duty;

/// Return the credited minutes for a single duty.
///
/// The credited value is `duty.credit` (c_t), pre-computed by the instance
/// data loader. This function does not re-derive the credit formula; that
/// formula is encoded in the loader. See BENCHMARK-SEMANTICS-v1.0 §3 (R1)
/// and BENCHMARK-REFERENCE-SPECIFICATION-v1.0 §5.1.
///
/// # Panics
/// Panics in debug builds if `duty.credit < 0.0`.
pub fn duty_credit(duty: &Duty) -> f64 {
    debug_assert!(
        duty.credit >= 0.0,
        "duty {} has negative credit: {}",
        duty.id,
        duty.credit
    );
    duty.credit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Duty, FlightLeg};

    fn make_duty(id: u32, credit: f64) -> Duty {
        Duty {
            id,
            credit,
            legs: vec![],
        }
    }

    /// O1: Single duty, credit = 60.0
    #[test]
    fn o1_single_duty_credit() {
        let d = make_duty(1, 60.0);
        assert!((duty_credit(&d) - 60.0).abs() < 1e-9);
    }

    #[test]
    fn zero_credit_duty() {
        let d = make_duty(2, 0.0);
        assert_eq!(duty_credit(&d), 0.0);
    }

    #[test]
    fn fractional_credit() {
        let d = make_duty(3, 45.5);
        assert!((duty_credit(&d) - 45.5).abs() < 1e-9);
    }

    #[test]
    fn duty_with_legs_uses_duty_credit_not_leg_sum() {
        // Duty credit is authoritative; leg credits are not summed by the evaluator.
        // This test documents that the adapter boundary is at the duty level.
        let d = Duty {
            id: 4,
            credit: 90.0,
            legs: vec![
                FlightLeg {
                    id: 1,
                    credit: 40.0,
                    duration: 60.0,
                },
                FlightLeg {
                    id: 2,
                    credit: 40.0,
                    duration: 60.0,
                },
                // leg sum = 80.0, but duty credit = 90.0 (qualification rules)
            ],
        };
        assert!((duty_credit(&d) - 90.0).abs() < 1e-9);
    }
}
