/// DGCA FDTL Rule: Standby Duty Limits
///
/// References:
///   §8.2 — "Standby duty shall not exceed 12 hours."
///   §8.3 — "A crew member on standby shall be given at least 2 hours notice
///            before being required to report for a Flight Duty Period."
///
/// Implementation note: we identify standby shifts by checking whether the
/// shift's required_skill name contains "Standby" (case-insensitive).
/// In a production system a dedicated `ShiftType` enum would be cleaner;
/// this approach avoids changing the existing `Shift` model.

use crate::compliance::traits::{ConstraintRule, RuleId, RuleOutcome, RuleContext, Severity};
use super::limits::DgcaLimits;

const RULE_ID: &str = "dgca.fdtl.standby_limits";
const REG_REF: &str = "CAR Section 7 Series J Part III §8.2-8.3";

pub struct StandbyRule {
    limits: DgcaLimits,
}

impl StandbyRule {
    pub fn new(limits: DgcaLimits) -> Self {
        StandbyRule { limits }
    }

    fn is_standby_shift(shift: &crate::models::Shift) -> bool {
        shift.required_skill.0.to_lowercase().contains("standby")
    }
}

impl ConstraintRule for StandbyRule {
    fn id(&self) -> RuleId {
        RULE_ID
    }

    fn description(&self) -> &str {
        "CAR S7 J III §8.2-8.3 — Standby <= 12 h; callout notice >= 2 h"
    }

    fn severity(&self) -> Severity {
        Severity::Hard
    }

    fn evaluate(&self, ctx: &RuleContext<'_>) -> Vec<RuleOutcome> {
        let mut outcomes = Vec::new();
        let max_standby = self.limits.max_standby_hours;
        let min_notice = self.limits.min_callout_notice_hours;

        for worker in ctx.workers {
            let shifts = match ctx.worker_shifts.get(&worker.id) {
                Some(s) => s,
                None => continue,
            };

            // Collect standby shifts for this worker.
            let standby_shifts: Vec<_> = shifts
                .iter()
                .filter(|s| Self::is_standby_shift(s))
                .collect();

            for sb in &standby_shifts {
                // §8.2: Standby duration must not exceed max_standby_hours.
                if sb.duration_hours > max_standby {
                    outcomes.push(RuleOutcome::violated(
                        RULE_ID,
                        REG_REF,
                        Severity::Hard,
                        format!(
                            "Worker {} standby shift {} lasts {} h, exceeding the {} h limit (§8.2)",
                            worker.id, sb.id, sb.duration_hours, max_standby,
                        ),
                        format!(
                            "Shorten standby shift {} to at most {} h, or split it into two \
                             consecutive standby periods with a rest break between them.",
                            sb.id, max_standby,
                        ),
                    ));
                }

                // §8.3: After a standby shift, the next FDP must start at least
                // `min_callout_notice_hours` after the standby begins (i.e. the
                // crew member needs notice time before reporting).
                // We check: if the next assigned shift starts within `min_notice`
                // hours of the standby start, that is a callout-notice violation.
                let sb_start = sb.start_hour;
                for next in shifts.iter().filter(|s| !Self::is_standby_shift(s)) {
                    if next.start_hour > sb_start
                        && next.start_hour < sb_start + min_notice
                    {
                        let actual_notice = next.start_hour - sb_start;
                        outcomes.push(RuleOutcome::violated(
                            RULE_ID,
                            REG_REF,
                            Severity::Hard,
                            format!(
                                "Worker {} FDP shift {} starts only {} h after standby {} begins; \
                                 minimum callout notice is {} h (§8.3)",
                                worker.id, next.id, actual_notice, sb.id, min_notice,
                            ),
                            format!(
                                "Delay shift {}'s start by at least {} h (to h{}) to provide \
                                 the required {} h callout notice from standby {} start (h{}). \
                                 Alternatively, assign shift {} to a crew member not on standby.",
                                next.id,
                                min_notice - actual_notice,
                                sb_start + min_notice,
                                min_notice,
                                sb.id,
                                sb_start,
                                next.id,
                            ),
                        ));
                    }
                }
            }
        }

        outcomes
    }
}