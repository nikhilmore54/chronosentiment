/// DGCA FDTL Rule: Minimum Rest Between Consecutive FDPs
///
/// Reference: CAR Section 7 Series J Part III §6.1
/// "A crew member shall be given a rest period of not less than 12 hours
///  before commencing a Flight Duty Period."
///
/// Implementation note: we treat each assigned shift as one FDP.
/// The gap between the end of shift[i] and the start of shift[i+1]
/// for the same worker must be ≥ `limits.min_rest_hours`.

use crate::compliance::traits::{ConstraintRule, RuleId, RuleOutcome, RuleContext, Severity};
use super::limits::DgcaLimits;

const RULE_ID: &str = "dgca.fdtl.minimum_rest";
const REG_REF: &str = "CAR Section 7 Series J Part III §6.1";

pub struct MinimumRestRule {
    limits: DgcaLimits,
}

impl MinimumRestRule {
    pub fn new(limits: DgcaLimits) -> Self {
        MinimumRestRule { limits }
    }
}

impl ConstraintRule for MinimumRestRule {
    fn id(&self) -> RuleId {
        RULE_ID
    }

    fn description(&self) -> &str {
        "CAR S7 J III §6.1 — Minimum rest of 12 h between consecutive FDPs"
    }

    fn severity(&self) -> Severity {
        Severity::Hard
    }

    fn evaluate(&self, ctx: &RuleContext<'_>) -> Vec<RuleOutcome> {
        let mut outcomes = Vec::new();
        let min_rest = self.limits.min_rest_hours;

        for worker in ctx.workers {
            let shifts = match ctx.worker_shifts.get(&worker.id) {
                Some(s) if s.len() >= 2 => s,
                _ => continue,
            };

            for window in shifts.windows(2) {
                let prev = window[0];
                let next = window[1];
                // Gap = start of next FDP − end of previous FDP.
                // If next starts before prev ends (overlap), gap is 0.
                let gap = if next.start_hour >= prev.end_hour() {
                    next.start_hour - prev.end_hour()
                } else {
                    0
                };

                if gap < min_rest {
                    outcomes.push(RuleOutcome::violated(
                        RULE_ID,
                        REG_REF,
                        Severity::Hard,
                        format!(
                            "Worker {} has only {} h rest between shift {} (ends h{}) \
                             and shift {} (starts h{}); minimum is {} h",
                            worker.id, gap,
                            prev.id, prev.end_hour(),
                            next.id, next.start_hour,
                            min_rest,
                        ),
                        format!(
                            "Extend the gap between shift {} and shift {} to at least {} h. \
                             Options: delay shift {}'s start, shorten shift {}'s duration, \
                             or reassign one shift to a different crew member.",
                            prev.id, next.id, min_rest,
                            next.id, prev.id,
                        ),
                    ));
                }
            }
        }

        outcomes
    }
}