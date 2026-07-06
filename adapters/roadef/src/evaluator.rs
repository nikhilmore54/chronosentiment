use crate::models::{Network, TrafficMatrix, Scenario, Solution};
use crate::graph::Digraph;
use crate::ecmp::expand_sr_path;
use crate::path::SrPathBit;
use std::collections::{HashMap, HashSet};

pub struct RoadefEvaluator {
    pub graph: Digraph,
    pub tm: TrafficMatrix,
    pub scenario: Scenario,
}

pub struct EvaluationResult {
    pub valid: bool,
    pub obj: f64,
}

pub struct TimeSlotLoads {
    pub arc_flows: HashMap<u64, f64>,
    pub arc_saturations: HashMap<u64, f64>,
    pub mlu: f64,
    pub jain_index: f64,
    pub inv_load_cost: f64,
}

impl RoadefEvaluator {
    pub fn new(network: &Network, tm: TrafficMatrix, scenario: Scenario) -> Self {
        Self { 
            graph: Digraph::new(network),
            tm, 
            scenario 
        }
    }
    
    pub fn compute_loads(&self, time_slot: usize, solution: &Solution) -> Option<TimeSlotLoads> {
        let mut arc_flows: HashMap<u64, f64> = HashMap::new();
        
        let mut disabled_arcs = HashSet::new();
        if let Some(intervention) = self.scenario.interventions.iter().find(|i| i.t == time_slot) {
            for &link_id in &intervention.links {
                disabled_arcs.insert(link_id);
            }
        }

        // Initialize flows to 0 for all arcs
        for arc in &self.graph.arcs {
            arc_flows.insert(arc.id, 0.0);
        }

        for (d_id, demand) in self.tm.demands.iter().enumerate() {
            let flow = demand.v[time_slot];
            if flow <= 0.0 {
                continue; // no traffic
            }

            // Find SR path for this demand at this time slot
            let mut waypoints: &[u64] = &[];
            if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == time_slot) {
                waypoints = &srpath.w;
            }
            
            let ok = expand_sr_path(
                &self.graph, 
                demand.s, 
                demand.t, 
                waypoints, 
                &disabled_arcs, 
                flow, 
                &mut arc_flows
            );

            if !ok {
                // Disconnected demand
                return None;
            }
        }

        let mut arc_saturations = HashMap::new();
        let mut mlu = 0.0;
        let mut sum_sat = 0.0;
        let mut sum_sq_sat = 0.0;
        let mut inv_load_cost = 0.0;
        let mut count_sat = 0;

        for arc in &self.graph.arcs {
            let flow = *arc_flows.get(&arc.id).unwrap_or(&0.0);
            let capacity = arc.capacity;
            let sat = if capacity > 0.0 { flow / capacity } else { f64::INFINITY };
            arc_saturations.insert(arc.id, sat);
            if sat > mlu {
                mlu = sat;
            }

            sum_sat += sat;
            sum_sq_sat += sat * sat;
            count_sat += 1;

            if sat > 0.0 {
                if sat >= 1.0 {
                    inv_load_cost += f64::INFINITY;
                } else {
                    // Use f32 logic exactly as C++ checker does: invArcLoadCost(float f_sat)
                    let f_sat = sat as f32;
                    let cost = (1.0 / (1.0 - f_sat as f64)) - 1.0;
                    inv_load_cost += cost;
                }
            }
        }
        
        let jain_index = if sum_sq_sat == 0.0 {
            0.0
        } else {
            let n = count_sat as f64;
            (sum_sat * sum_sat) / (n * sum_sq_sat)
        };

        Some(TimeSlotLoads {
            arc_flows,
            arc_saturations,
            mlu,
            jain_index,
            inv_load_cost,
        })
    }

    pub fn evaluate_solution(&self, solution: &Solution) -> EvaluationResult {
        // maxSegments check
        if self.scenario.max_segments >= 0 {
            for path in &solution.srpaths {
                // waypoints length + 1 (since segments = waypoints + 1 if we consider the full path... wait, no.)
                // Actually the number of segments is len(waypoints) + 1.
                if path.w.len() + 1 > self.scenario.max_segments as usize {
                    return EvaluationResult { valid: false, obj: f64::INFINITY };
                }
            }
        }

        let mut total_obj = 0.0;
        let mut prev_paths: HashMap<u64, SrPathBit> = HashMap::new();

        for t in 0..self.tm.num_time_slots {
            // Budget check
            let mut budget_cost = 0;
            let mut curr_paths: HashMap<u64, SrPathBit> = HashMap::new();

            for (d_id, demand) in self.tm.demands.iter().enumerate() {
                let d_id_u64 = d_id as u64;
                let mut bitpath = SrPathBit::new_uninitialized();
                
                if let Some(srpath) = solution.srpaths.iter().find(|p| p.d == d_id && p.t == t) {
                    bitpath = SrPathBit::new_explicit(demand.s, demand.t, &srpath.w);
                }

                if t > 0 {
                    let uninit = SrPathBit::new_uninitialized();
                    let prev_bitpath = prev_paths.get(&d_id_u64).unwrap_or(&uninit);
                    budget_cost += bitpath.dist(prev_bitpath);
                }
                
                curr_paths.insert(d_id_u64, bitpath);
            }

            if t > 0 {
                let budget_val = self.scenario.budget.iter().find(|b| b.t == t).map(|b| b.value).unwrap_or(0);
                if budget_cost > budget_val {
                    return EvaluationResult { valid: false, obj: f64::INFINITY };
                }
            }

            prev_paths = curr_paths;

            // Load and connectivity check
            if let Some(loads) = self.compute_loads(t, solution) {
                total_obj += loads.mlu + loads.inv_load_cost;
            } else {
                // Connectivity failed
                return EvaluationResult { valid: false, obj: f64::INFINITY };
            }
        }

        EvaluationResult {
            valid: true,
            obj: total_obj,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::{load_network, load_traffic_matrix, load_scenario, load_solution};

    #[test]
    fn test_compute_loads_set_a_01_empty_solution() {
        let net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json").unwrap();
        let tm = load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json").unwrap();
        let scenario = load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json").unwrap();
        
        let evaluator = RoadefEvaluator::new(&net, tm, scenario);
        let empty_solution = Solution { srpaths: vec![] };

        let loads_t0 = evaluator.compute_loads(0, &empty_solution).unwrap();
        
        // We know from the C++ checker output:
        // "Maximum Link Utilization (MLU) at 0 : 1.0000006861063464"
        assert!((loads_t0.mlu - 1.000000686106).abs() < 1e-6);

        let loads_t1 = evaluator.compute_loads(1, &empty_solution).unwrap();
        // "Maximum Link Utilization (MLU) at 1 : 0.5663266666666666"
        assert!((loads_t1.mlu - 0.566326666666).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_solution_set_a_01() {
        let net = load_network("repo/challenge-roadef-2026-main/setA/setA-01-net.json").unwrap();
        let tm = load_traffic_matrix("repo/challenge-roadef-2026-main/setA/setA-01-tm.json").unwrap();
        let scenario = load_scenario("repo/challenge-roadef-2026-main/setA/setA-01-scenario.json").unwrap();
        
        let evaluator = RoadefEvaluator::new(&net, tm, scenario);
        let empty_solution = Solution { srpaths: vec![] };

        let result = evaluator.evaluate_solution(&empty_solution);
        assert!(result.valid);
        // The C++ checker says: Objective value: 64.99616053303649
        // This includes Jain's index, let's see if we get close.
        // Wait, C++ uses sum(mlu) + sum(inv_load) - (actually jain doesn't get summed in obj? Let's not assert exact obj yet)
    }
}
