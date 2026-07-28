/// DGCA FDTL Rule: Maximum Flight Duty Period
///
/// Reference: CAR Section 7 Series J Part III §5.1
/// "The maximum FDP for a two-pilot crew shall not exceed 11 hours for
///  day operations and 10 hours for night operations."
///
/// Implementation note: we use `shift.duration_hours` as the FDP duration.
/// Night duty is detected when the shift spans 22:00–05:00 local (hours 22–29
/// in a 0-based weekly horizon, or any shift whose start_hour mod 24 >= 22
/// or whose end_hour mod 24 <= 5 and duration > 0).

use crate::compliance::traits::{ConstraintRule, RuleId, RuleOutcome, RuleContext, Severity};
use super::limits::DgcaLimits;

const RULE_ID: &str = "dgca.fdtl.maximum_fdp";
const REG_REF: &str = "CAR Section 7 Series J Part III §5.1";

pub struct MaximumFdpRule {
    limits: DgcaLimits,
}

impl MaximumFdpRule {
    pub fn new(limits: DgcaLimits) -> Self {
        MaximumFdpRule { limits }
    }

    /// Returns true if the shift qualifies as a night-duty FDP.
    /// Night duty: start hour (mod 24) is between 22:00 and 05:00.
    fn is_night_duty(start_hour: u64) -> bool {
        let hour_of_day = start_hour % 24;
        hour_of_day >= 22 || hour_of_day < 5
    }
}

impl ConstraintRule for MaximumFdpRule {
    fn id(&self) -> RuleId {
        RULE_ID
    }

    fn description(&self) -> &str {
        "CAR S7 J III §5.1 — Maximum FDP: 11 h day / 10 h night (2-pilot crew)"
    }

    fn severity(&self) -> Severity {
        Severity::Hard
    }

    fn evaluate(&self, ctx: &RuleContext<'_>) -> Vec<RuleOutcome> {
        let mut outcomes = Vec::new();

        for worker in ctx.workers {
            let shifts = match ctx.worker_shifts.get(&worker.id) {
                Some(s) => s,
                None => continue,
            };

            for shift in shifts {
                let night = Self::is_night_duty(shift.start_hour);
                // Night FDP limit is 10 h; day FDP limit is max_fdp_hours (11 h default).
                let limit = if night {
                    self.limits.max_fdp_hours.saturating_sub(1) // 10 h for night
                } else {
                    self.limits.max_fdp_hours
                };

                if shift.duration_hours > limit {
                    let duty_type = if night { "night" } else { "day" };
                    outcomes.push(RuleOutcome::violated(
                        RULE_ID,
                        REG_REF,
                        Severity::Hard,
                        format!(
                            "Worker {} shift {} is a {} FDP of {} h, exceeding the {} h limit",
                            worker.id, shift.id, duty_type,
                            shift.duration_hours, limit,
                        ),
                        format!(
                            "Reduce shift {}'s duration to at most {} h ({}). \
                             Options: split the duty into two shorter FDPs with a rest break, \
                             or reassign the later sectors to a standby crew member.",
                            shift.id, limit, duty_type,
                        ),
                    ));
                }
            }
        }

        outcomes
    }
}