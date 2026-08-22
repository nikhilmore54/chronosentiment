use coralys_core::operators::ConstraintModel;
use crate::moga_impl::RoadefGenome;
use crate::evaluator::RoadefEvaluator;
use crate::path::SrPathBit;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use crate::ecmp::expand_sr_path;

#[derive(Debug, Clone)]
pub enum RoadefViolation {
    SegmentLimit { demand_id: usize, time_slot: usize, waypoints: usize, max_segments: usize },
    Budget { time_slot: usize, cost: usize, limit: usize },
    Connectivity { demand_id: usize, time_slot: usize },
    Capacity { arc_id: u64, time_slot: usize, flow: f64, capacity: f64, sat: f64 },
}



pub struct RoadefConstraintModel {
    pub evaluator: Arc<RoadefEvaluator>,
}

impl ConstraintModel<RoadefGenome> for RoadefConstraintModel {
    type Violation = RoadefViolation;

    fn evaluate_violations(&self, candidate: &RoadefGenome) -> Vec<Self::Violation> {
        let solution = candidate.to_solution();
        let mut violations = Vec::new();
        let scenario = &self.evaluator.scenario;

        // Stage 1: Segment limit
        if scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                if path.w.len() + 1 > scenario.max_segments as usize {
                    violations.push(RoadefViolation::SegmentLimit {
                        demand_id: path.d,
                        time_slot: path.t,
                        waypoints: path.w.len(),
                        max_segments: scenario.max_segments as usize,
                    });
                }
            }
        }

        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();
        let tm = &self.evaluator.tm;
        
        for ts in 0..tm.num_time_slots {
            // Stage 2: Budget
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();

            for (d_id, demand) in tm.demands.iter().enumerate() {
                let mut bitpath = SrPathBit::new_uninitialized();
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }
                if ts > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&(d_id as u64)).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }
                curr_paths.insert(d_id as u64, bitpath);
            }
            if ts > 0 {
                let budget_val = scenario.budget.iter().find(|b| b.t == ts).map(|b| b.value).unwrap_or(0);
                if budget_cost > budget_val {
                    violations.push(RoadefViolation::Budget { time_slot: ts, cost: budget_cost, limit: budget_val });
                }
            }
            prev_paths = curr_paths;

            // Stage 3 & 4: Routing and Capacity
            let mut disabled_arcs = HashSet::new();
            if let Some(intervention) = scenario.interventions.iter().find(|i| i.t == ts) {
                for &link_id in &intervention.links {
                    disabled_arcs.insert(link_id);
                }
            }

            let mut arc_flows: HashMap<u64, f64> = HashMap::new();
            for arc in &self.evaluator.graph.arcs {
                arc_flows.insert(arc.id, 0.0);
            }

            for (d_id, demand) in tm.demands.iter().enumerate() {
                let flow = demand.v[ts];
                if flow <= 0.0 { continue; }
                let mut waypoints: &[u64] = &[];
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == ts) {
                    waypoints = &srpath.w;
                }
                
                let ok = expand_sr_path(
                    &self.evaluator.graph, demand.s, demand.t, waypoints,
                    &disabled_arcs, flow, &mut arc_flows,
                );
                
                if !ok {
                    violations.push(RoadefViolation::Connectivity { demand_id: d_id, time_slot: ts });
                }
            }

            for arc in &self.evaluator.graph.arcs {
                let flow = *arc_flows.get(&arc.id).unwrap_or(&0.0);
                let sat = if arc.capacity > 0.0 { flow / arc.capacity } else { f64::INFINITY };
                if sat >= 1.0 - 1e-6 {
                    violations.push(RoadefViolation::Capacity {
                        arc_id: arc.id,
                        time_slot: ts,
                        flow,
                        capacity: arc.capacity,
                        sat,
                    });
                }
            }
        }

        violations
    }
}
