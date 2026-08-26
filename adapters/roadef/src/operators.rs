use crate::constraints::{RoadefConstraintModel, RoadefViolation};
use crate::moga_impl::RoadefGenome;
use coralys_core::operators::{
    ConstraintModel, ImprovementOperator, OperatorBudget, RepairOperator,
};
use std::fmt;

#[derive(Debug)]
pub struct OperatorError(pub String);

impl fmt::Display for OperatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for OperatorError {}

pub struct RoadefRepair;

impl RepairOperator<RoadefGenome, RoadefConstraintModel> for RoadefRepair {
    type Error = OperatorError;

    fn repair(
        &self,
        candidate: &mut RoadefGenome,
        model: &RoadefConstraintModel,
        _budget: &OperatorBudget,
    ) -> Result<bool, Self::Error> {
        let violations = model.evaluate_violations(candidate);
        if violations.is_empty() {
            return Ok(true); // Already feasible
        }

        // H-SKIP (P10-C0 authorized 2026-08-26):
        // P10-C0 established that the Capacity repair path is structurally inert:
        //   - 559 failed repairs across 7 instances, 0% genome change, 0% violation improvement.
        //   - The needs_ecmp_fallback flag was set but the clearing code was commented out.
        //   - Repair was paying the full evaluate_violations() cost to produce a violation list
        //     it then discarded without making any genome modification.
        //
        // Fast path: if all violations are Capacity type, return Ok(false) immediately.
        // This makes the no-op explicit and avoids the loop overhead for the dominant case.
        // SegmentLimit and Connectivity violations still proceed to the clearing path below.
        //
        // P10-C1 (bottleneck arc characterization) is authorized next to determine the correct
        // repair/construction intervention. Do not implement Dijkstra/ECMP here until P10-C1
        // evidence is reviewed.
        let all_capacity = violations
            .iter()
            .all(|v| matches!(v, RoadefViolation::Capacity { .. }));
        if all_capacity {
            return Ok(false);
        }

        // SegmentLimit / Connectivity violations: clear waypoints for affected demands.
        // These are the only violation types where the current repair makes a genome change.
        for v in violations {
            match v {
                RoadefViolation::SegmentLimit { demand_id, .. }
                | RoadefViolation::Connectivity { demand_id, .. } => {
                    candidate.waypoints[demand_id].clear();
                }
                _ => {}
            }
        }

        // Return false: clearing waypoints may not restore feasibility.
        // We rely on the feasibility gate in EvolutionaryPipeline.
        Ok(false)
    }
}

pub struct RoadefImprovement;

impl ImprovementOperator<RoadefGenome, RoadefConstraintModel> for RoadefImprovement {
    type Error = OperatorError;

    fn improve(
        &self,
        _candidate: &mut RoadefGenome,
        _model: &RoadefConstraintModel,
        _budget: &OperatorBudget,
    ) -> Result<bool, Self::Error> {
        // TODO: Implement bottleneck-relief local search or LNS here.
        // For now, this is a no-op that just returns true (preserves feasibility).
        Ok(true)
    }
}
