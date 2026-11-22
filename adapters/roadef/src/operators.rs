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

        let mut changed = false;

        // 1. Repair SegmentLimit / Connectivity violations by clearing explicit waypoints
        for v in &violations {
            match v {
                RoadefViolation::SegmentLimit { demand_id, .. }
                | RoadefViolation::Connectivity { demand_id, .. } => {
                    if !candidate.waypoints[*demand_id].is_empty() {
                        candidate.waypoints[*demand_id].clear();
                        changed = true;
                    }
                }
                _ => {}
            }
        }

        // 2. Active Capacity Repair: relieve bottleneck arcs by clearing explicit waypoints
        // for demands routed towards or through congested links.
        for v in &violations {
            if let RoadefViolation::Capacity { arc_id, .. } = v {
                if let Some(&arc_idx) = model.evaluator.graph.arc_by_id.get(arc_id) {
                    let arc = &model.evaluator.graph.arcs[arc_idx];
                    let (u, to_node) = (arc.from, arc.to);

                    for (d_id, wps) in candidate.waypoints.iter_mut().enumerate() {
                        if !wps.is_empty() {
                            let demand = &model.evaluator.tm.demands[d_id];
                            if wps.contains(&u)
                                || wps.contains(&to_node)
                                || demand.s == u
                                || demand.t == to_node
                            {
                                wps.clear();
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        if !changed {
            return Ok(false);
        }

        // Re-evaluate to verify whether feasibility was restored
        let post_violations = model.evaluate_violations(candidate);
        Ok(post_violations.is_empty())
    }
}

pub struct RoadefImprovement;

impl ImprovementOperator<RoadefGenome, RoadefConstraintModel> for RoadefImprovement {
    type Error = OperatorError;

    fn improve(
        &self,
        candidate: &mut RoadefGenome,
        model: &RoadefConstraintModel,
        _budget: &OperatorBudget,
    ) -> Result<bool, Self::Error> {
        let solution = candidate.to_solution();
        let eval_res = model.evaluator.evaluate_solution(&solution);
        if !eval_res.valid {
            return Ok(true);
        }
        let initial_obj = eval_res.obj;

        // Identify highest-saturation arc across time slots
        let mut max_sat = 0.0;
        let mut bn_arc = None;
        let mut bn_ts = 0;

        for ts in 0..model.evaluator.tm.num_time_slots {
            if let Some(loads) = model.evaluator.compute_loads(ts, &solution) {
                for (arc_id, sat) in loads.arc_saturations {
                    if sat > max_sat {
                        max_sat = sat;
                        bn_arc = Some(arc_id);
                        bn_ts = ts;
                    }
                }
            }
        }

        // Active relief if saturation exceeds 0.85
        if max_sat >= 0.85 {
            if let Some(arc_id) = bn_arc {
                if let Some(&arc_idx) = model.evaluator.graph.arc_by_id.get(&arc_id) {
                    let arc = &model.evaluator.graph.arcs[arc_idx];
                    let (u, to_node) = (arc.from, arc.to);

                    let mut best_demand = None;
                    let mut max_vol = 0.0;

                    for (d_id, demand) in model.evaluator.tm.demands.iter().enumerate() {
                        let vol = demand.v[bn_ts];
                        if vol > max_vol && !candidate.waypoints[d_id].is_empty() {
                            if demand.s == u
                                || demand.t == to_node
                                || candidate.waypoints[d_id].contains(&u)
                                || candidate.waypoints[d_id].contains(&to_node)
                            {
                                max_vol = vol;
                                best_demand = Some(d_id);
                            }
                        }
                    }

                    if let Some(d_id) = best_demand {
                        let orig_wps = candidate.waypoints[d_id].clone();
                        candidate.waypoints[d_id].clear();

                        let test_sol = candidate.to_solution();
                        let test_res = model.evaluator.evaluate_solution(&test_sol);
                        if test_res.valid && test_res.obj < initial_obj {
                            return Ok(true);
                        } else {
                            // Rollback if no improvement
                            candidate.waypoints[d_id] = orig_wps;
                        }
                    }
                }
            }
        }

        Ok(true)
    }
}
