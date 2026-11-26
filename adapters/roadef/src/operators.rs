use crate::constraints::{RoadefConstraintModel, RoadefViolation};
use crate::moga_impl::RoadefGenome;
use coralys_core::operators::{
    ConstraintModel, ImprovementOperator, OperatorBudget, RepairOperator,
};
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;

thread_local! {
    static REPAIR_WAS_INVOKED: Cell<bool> = const { Cell::new(false) };
}

/// Takes the flag indicating whether `RoadefRepair::repair` was invoked on this thread, resetting it to false.
pub fn take_repair_invoked() -> bool {
    REPAIR_WAS_INVOKED.with(|cell| cell.replace(false))
}

/// Sets the flag indicating that `RoadefRepair::repair` was invoked on this thread.
pub fn mark_repair_invoked() {
    REPAIR_WAS_INVOKED.with(|cell| cell.set(true));
}

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
        mark_repair_invoked();

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

        // 2. Active Capacity Repair
        // 2a. Relieve bottleneck arcs by clearing explicit waypoints for demands
        // routed towards or through congested links.
        let mut congested_arcs: HashMap<u64, (usize, f64)> = HashMap::new();
        for v in &violations {
            if let RoadefViolation::Capacity {
                arc_id,
                time_slot,
                sat,
                ..
            } = v
            {
                let entry = congested_arcs.entry(*arc_id).or_insert((*time_slot, *sat));
                if *sat > entry.1 {
                    *entry = (*time_slot, *sat);
                }
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

        // 2b. Option 1: Active Detour Injection.
        // For demands with empty waypoints (ECMP default) traversing congested links,
        // clearing waypoints is a no-op. Actively inject an alternative detour waypoint
        // for the highest-volume demand touching each congested arc to steer traffic away.
        let max_seg = if model.evaluator.scenario.max_segments >= 0 {
            model.evaluator.scenario.max_segments as usize
        } else {
            usize::MAX
        };

        if max_seg >= 2 {
            let disabled_arcs: HashSet<u64> = model
                .evaluator
                .scenario
                .interventions
                .iter()
                .flat_map(|iv| iv.links.iter().copied())
                .collect();

            for (&arc_id, &(time_slot, _sat)) in &congested_arcs {
                if let Some(&arc_idx) = model.evaluator.graph.arc_by_id.get(&arc_id) {
                    let arc = &model.evaluator.graph.arcs[arc_idx];
                    let (u, to_node) = (arc.from, arc.to);

                    // Find candidate demands with empty waypoints touching u or to_node
                    let mut cand_demands: Vec<(usize, f64, bool)> = Vec::new(); // (d_id, volume, is_source_u)
                    for (d_id, wps) in candidate.waypoints.iter().enumerate() {
                        if wps.is_empty() {
                            let demand = &model.evaluator.tm.demands[d_id];
                            let vol = demand.v.get(time_slot).copied().unwrap_or(0.0);
                            if vol > 0.0 {
                                if demand.s == u {
                                    cand_demands.push((d_id, vol, true));
                                } else if demand.t == to_node {
                                    cand_demands.push((d_id, vol, false));
                                }
                            }
                        }
                    }

                    // Sort descending by volume: detouring highest-volume demand yields largest relief
                    cand_demands.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

                    // Detour up to 1 highest-volume demand on this congested link
                    for (d_id, _vol, is_source_u) in cand_demands.into_iter().take(1) {
                        let demand = &model.evaluator.tm.demands[d_id];
                        let mut detour_w = None;

                        if is_source_u {
                            // Detour via alternative out-neighbor of u
                            if let Some(out_indices) = model.evaluator.graph.out_arcs.get(&u) {
                                let mut best_cap = -1.0;
                                for &out_idx in out_indices {
                                    let out_arc = &model.evaluator.graph.arcs[out_idx];
                                    let w = out_arc.to;
                                    if w != to_node
                                        && w != demand.s
                                        && w != demand.t
                                        && !disabled_arcs.contains(&out_arc.id)
                                    {
                                        if out_arc.capacity > best_cap {
                                            best_cap = out_arc.capacity;
                                            detour_w = Some(w);
                                        }
                                    }
                                }
                            }
                        } else {
                            // Detour via alternative in-neighbor of to_node
                            if let Some(in_indices) = model.evaluator.graph.in_arcs.get(&to_node) {
                                let mut best_cap = -1.0;
                                for &in_idx in in_indices {
                                    let in_arc = &model.evaluator.graph.arcs[in_idx];
                                    let w = in_arc.from;
                                    if w != u
                                        && w != demand.s
                                        && w != demand.t
                                        && !disabled_arcs.contains(&in_arc.id)
                                    {
                                        if in_arc.capacity > best_cap {
                                            best_cap = in_arc.capacity;
                                            detour_w = Some(w);
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(w) = detour_w {
                            candidate.waypoints[d_id] = vec![w];
                            changed = true;
                        }
                    }
                }
            }
        }

        if !changed {
            return Ok(false);
        }

        // Re-evaluate to verify whether feasibility was restored using staged early-exit
        Ok(model.is_feasible(candidate))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::RoadefEvaluator;
    use crate::models::{Demand, Network, NetworkLink, NetworkNode, Scenario, TrafficMatrix};
    use std::sync::Arc;

    #[test]
    fn test_take_repair_invoked() {
        take_repair_invoked(); // reset
        assert!(!take_repair_invoked());

        mark_repair_invoked();
        assert!(take_repair_invoked());
        assert!(!take_repair_invoked()); // second call returns false
    }

    #[test]
    fn test_active_detour_injection() {
        // Topology:
        // 0 -> 1 (metric 10, cap 10 - bottleneck)
        // 0 -> 2 (metric 20, cap 100 - detour)
        // 2 -> 1 (metric 20, cap 100 - detour)
        // 1 -> 3 (metric 10, cap 100)
        let network = Network {
            directed: true,
            multigraph: false,
            nodes: vec![
                NetworkNode { id: 0, name: None },
                NetworkNode { id: 1, name: None },
                NetworkNode { id: 2, name: None },
                NetworkNode { id: 3, name: None },
            ],
            links: vec![
                NetworkLink {
                    id: 1,
                    from: 0,
                    to: 1,
                    metric: 10.0,
                    capacity: 10.0,
                },
                NetworkLink {
                    id: 2,
                    from: 0,
                    to: 2,
                    metric: 20.0,
                    capacity: 100.0,
                },
                NetworkLink {
                    id: 3,
                    from: 2,
                    to: 1,
                    metric: 20.0,
                    capacity: 100.0,
                },
                NetworkLink {
                    id: 4,
                    from: 1,
                    to: 3,
                    metric: 10.0,
                    capacity: 100.0,
                },
            ],
        };

        // Demand from 0 to 1 with volume 15.0 (exceeds link 0->1 capacity 10.0)
        let tm = TrafficMatrix {
            num_time_slots: 1,
            demands: vec![Demand {
                s: 0,
                t: 1,
                v: vec![15.0],
            }],
        };

        let scenario = Scenario {
            max_segments: 2,
            budget: vec![],
            interventions: vec![],
        };

        let evaluator = Arc::new(RoadefEvaluator::new(&network, tm, scenario));
        let model = RoadefConstraintModel { evaluator };

        // Candidate has empty waypoints (ECMP default routes 0->1 directly)
        let mut candidate = RoadefGenome {
            waypoints: vec![vec![]],
            num_time_slots: 1,
        };

        // Check that initial candidate has capacity violation
        let viols = model.evaluate_violations(&candidate);
        assert!(
            !viols.is_empty(),
            "Initial candidate should violate capacity on arc 0->1"
        );

        // Run RoadefRepair
        let repair_op = RoadefRepair;
        let budget = OperatorBudget {
            max_iterations: 1,
            max_time_ms: 1000,
        };
        let res = repair_op.repair(&mut candidate, &model, &budget).unwrap();

        assert!(
            res,
            "RoadefRepair should successfully detour and restore feasibility"
        );
        assert_eq!(
            candidate.waypoints[0],
            vec![2],
            "Demand should be detoured through node 2"
        );
        assert!(
            take_repair_invoked(),
            "take_repair_invoked should be true after repair"
        );
    }
}
