/// DGCA FDTL Rules: Maximum Block Hours (28-day and 365-day windows)
///
/// References:
///   §7.1 — "A crew member shall not exceed 100 block hours in any 28 consecutive days."
///   §7.2 — "A crew member shall not exceed 1 000 block hours in any 365 consecutive days."
///
/// Implementation note: the scheduler horizon is typically one week (168 h) or
/// two weeks (336 h).  We cannot observe a full 28-day or 365-day window from
/// the schedule alone, so we sum the block hours *within the current schedule
/// window* and flag a violation if that partial sum already exceeds the limit.
/// In a production system the `RuleContext` would carry historical block hours;
/// for now we conservatively enforce the limit against the scheduled window only.

use crate::compliance::traits::{ConstraintRule, RuleId, RuleOutcome, RuleContext, Severity};
use super::limits::DgcaLimits;

// ── 28-day rule ───────────────────────────────────────────────────────────────

const RULE_ID_28D: &str = "dgca.fdtl.max_block_hours_28d";
const REG_REF_28D: &str = "CAR Section 7 Series J Part III §7.1";

pub struct MaxFlightHours28DaysRule {
    limits: DgcaLimits,
}

impl MaxFlightHours28DaysRule {
    pub fn new(limits: DgcaLimits) -> Self {
        MaxFlightHours28DaysRule { limits }
    }
}

impl ConstraintRule for MaxFlightHours28DaysRule {
    fn id(&self) -> RuleId {
        RULE_ID_28D
    }

    fn description(&self) -> &str {
        "CAR S7 J III §7.1 — Maximum 100 block hours in any 28 consecutive days"
    }

    fn severity(&self) -> Severity {
        Severity::Hard
    }

    fn evaluate(&self, ctx: &RuleContext<'_>) -> Vec<RuleOutcome> {
        let mut outcomes = Vec::new();
        let limit = self.limits.max_block_hours_28d;

        for worker in ctx.workers {
            let shifts = match ctx.worker_shifts.get(&worker.id) {
                Some(s) => s,
                None => continue,
            };

            let total_hours: u64 = shifts.iter().map(|s| s.duration_hours).sum();

            if total_hours > limit {
                let excess = total_hours - limit;
                outcomes.push(RuleOutcome::violated(
                    RULE_ID_28D,
                    REG_REF_28D,
                    Severity::Hard,
                    format!(
                        "Worker {} has {} block hours in the scheduled window, \
                         exceeding the 28-day cap of {} h by {} h",
                        worker.id, total_hours, limit, excess,
                    ),
                    format!(
                        "Remove or shorten {} h of shifts assigned to worker {} \
                         within the current 28-day window. Consider redistributing \
                         {} h of block time to a crew member with remaining capacity.",
                        excess, worker.id, excess,
                    ),
                ));
            }
        }

        outcomes
    }
}

// ── 365-day rule ──────────────────────────────────────────────────────────────

const RULE_ID_365D: &str = "dgca.fdtl.max_block_hours_365d";
const REG_REF_365D: &str = "CAR Section 7 Series J Part III §7.2";

pub struct MaxFlightHours365DaysRule {
    limits: DgcaLimits,
}

impl MaxFlightHours365DaysRule {
    pub fn new(limits: DgcaLimits) -> Self {
        MaxFlightHours365DaysRule { limits }
    }
}

impl ConstraintRule for MaxFlightHours365DaysRule {
    fn id(&self) -> RuleId {
        RULE_ID_365D
    }

    fn description(&self) -> &str {
        "CAR S7 J III §7.2 — Maximum 1 000 block hours in any 365 consecutive days"
    }

    fn severity(&self) -> Severity {
        Severity::Hard
    }

    fn evaluate(&self, ctx: &RuleContext<'_>) -> Vec<RuleOutcome> {
        let mut outcomes = Vec::new();
        let limit = self.limits.max_block_hours_365d;

        for worker in ctx.workers {
            let shifts = match ctx.worker_shifts.get(&worker.id) {
                Some(s) => s,
                None => continue,
            };

            let total_hours: u64 = shifts.iter().map(|s| s.duration_hours).sum();

            if total_hours > limit {
                let excess = total_hours - limit;
                outcomes.push(RuleOutcome::violated(
                    RULE_ID_365D,
                    REG_REF_365D,
                    Severity::Hard,
                    format!(
                        "Worker {} has {} block hours in the scheduled window, \
                         exceeding the 365-day cap of {} h by {} h",
                        worker.id, total_hours, limit, excess,
                    ),
                    format!(
                        "Worker {} has reached the annual block-hour ceiling. \
                         No further flight duties may be assigned in this calendar year. \
                         Reassign all remaining {} h of excess block time to other crew members.",
                        worker.id, excess,
                    ),
                ));
            }
        }

        outcomes
    }
}