use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use roadef::models::{Solution, SrPath};
use roadef::evaluator::RoadefEvaluator;
use roadef::graph::Digraph;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};

// Simulated Vault
struct ContextStats {
    success: usize,
    failure: usize,
}

struct CoralysVault {
    stats: HashMap<String, ContextStats>,
}

impl CoralysVault {
    fn new() -> Self {
        Self { stats: HashMap::new() }
    }
    
    fn query_pressure(&self, context: &str) -> f64 {
        if let Some(st) = self.stats.get(context) {
            let total = st.success + st.failure;
            if total > 0 {
                st.failure as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    fn record_success(&mut self, context: String) {
        let entry = self.stats.entry(context).or_insert(ContextStats { success: 0, failure: 0 });
        entry.success += 1;
    }
    
    fn record_failure(&mut self, context: String) {
        let entry = self.stats.entry(context).or_insert(ContextStats { success: 0, failure: 0 });
        entry.failure += 1;
    }
}

// Data model for logging M27.1 Operator Telemetry
struct OperatorObservation {
    node_id: u64,
    context: String,
    operator: String,
    
    parent_pressure: f64,
    child_pressure: f64,
    pressure_delta: f64,
    
    parent_objective: f64,
    child_objective: f64,
    objective_delta: f64,
    
    depth: usize,
    survival_depth: usize,
    incumbent_gap: f64,
    
    elite_descendant: bool,
    operator_outcome: String,
    incumbent_improvement: f64,
}

fn build_context_tag(
    path_nodes: &[u64],
    vol: f64,
    max_cap: f64,
    avg_congestion: f64,
) -> String {
    let vol_ratio = vol / max_cap;
    let vol_cat = if vol_ratio > 0.8 { "Vol:Extreme" } 
        else if vol_ratio > 0.5 { "Vol:High" } 
        else if vol_ratio > 0.3 { "Vol:Med" } 
        else if vol_ratio > 0.1 { "Vol:Low" } 
        else { "Vol:Tiny" };
        
    let len_cat = if path_nodes.len() > 8 { "Len:Extreme" }
        else if path_nodes.len() > 6 { "Len:Long" } 
        else if path_nodes.len() > 4 { "Len:Med" } 
        else if path_nodes.len() > 2 { "Len:Short" }
        else { "Len:Tiny" };
        
    let crosses_core = if path_nodes.iter().any(|&n| n < 5) { "Core:Deep" } 
        else if path_nodes.iter().any(|&n| n < 10) { "Core:Yes" } 
        else { "Core:No" };
        
    let cong_cat = if avg_congestion > 0.9 { "Cong:Extreme" }
        else if avg_congestion > 0.7 { "Cong:High" } 
        else if avg_congestion > 0.5 { "Cong:Med" } 
        else if avg_congestion > 0.3 { "Cong:Low" }
        else { "Cong:None" };
        
    format!("{}|{}|{}|{}", vol_cat, len_cat, crosses_core, cong_cat)
}

struct Solver<'a> {
    evaluator: &'a RoadefEvaluator,
    graph: &'a Digraph,
    max_cap: f64,
    precomputed_paths: Vec<Vec<Vec<u64>>>,
    vault: CoralysVault,
    logs: Vec<OperatorObservation>,
    node_counter: u64,
    best_obj: f64,
}

impl<'a> Solver<'a> {
    fn solve_recursive(
        &mut self,
        demand_idx: usize,
        current_solution: &mut Solution,
        current_avg_congestion: f64,
        parent_obj: f64,
    ) -> (bool, f64, usize) { // Returns (subtree_survived, best_subtree_obj, max_survival_depth)
        if self.node_counter > 2000 {
            return (false, f64::MAX, demand_idx);
        }
        if demand_idx >= self.evaluator.tm.demands.len() {
            // Reached leaf -> valid solution!
            let res = self.evaluator.evaluate_solution(current_solution);
            if res.valid && res.obj.is_finite() {
                if res.obj < self.best_obj {
                    self.best_obj = res.obj;
                }
                return (true, res.obj, demand_idx);
            }
            return (false, f64::MAX, demand_idx);
        }

        let mut best_subtree_obj = f64::MAX;
        let mut any_success = false;
        let mut max_survival_depth = demand_idx;

        let paths = self.precomputed_paths[demand_idx].clone();
        
        for (op_idx, path) in paths.iter().enumerate() {
            if self.node_counter > 2000 { break; }
            let demand = &self.evaluator.tm.demands[demand_idx];
            
            let mut path_nodes = vec![demand.s];
            path_nodes.extend_from_slice(&path);
            path_nodes.push(demand.t);
            
            let context = build_context_tag(&path_nodes, demand.v[0], self.max_cap, current_avg_congestion);
            let parent_pressure = self.vault.query_pressure(&context);
            
            let node_id = self.node_counter;
            self.node_counter += 1;

            // Apply decision
            current_solution.srpaths.push(SrPath { d: demand_idx, t: 0, w: path.clone() });
            
            // Fast fail evaluation
            let res = self.evaluator.evaluate_solution(current_solution);
            let child_obj = if res.valid { res.obj } else { f64::MAX };
            
            // "Operator" is mapped to path index here purely to simulate an action policy choice
            let operator = match op_idx {
                0 => "Destroy_MaxSlack".to_string(),
                1 => "Destroy_Random".to_string(),
                _ => "Repair_Greedy".to_string(),
            };
            
            let mut subtree_survived = false;
            let mut branch_best_obj = f64::MAX;
            let mut branch_max_depth = demand_idx;
            
            if res.valid {
                let next_cong = current_avg_congestion + (demand.v[0] / self.max_cap) * 0.05;
                let (sub_success, sub_obj, sub_depth) = self.solve_recursive(demand_idx + 1, current_solution, next_cong, child_obj);
                
                subtree_survived = sub_success;
                branch_best_obj = sub_obj;
                branch_max_depth = sub_depth;
                
                if sub_success {
                    any_success = true;
                    best_subtree_obj = best_subtree_obj.min(sub_obj);
                    max_survival_depth = max_survival_depth.max(sub_depth);
                }
            }
            
            current_solution.srpaths.pop();
            
            let child_pressure = if subtree_survived { 0.0 } else { 1.0 }; 
            let pressure_delta = child_pressure - parent_pressure;
            
            let objective_delta = if child_obj < f64::MAX && parent_obj < f64::MAX { child_obj - parent_obj } else { 0.0 };
            
            let incumbent_gap = if child_obj < f64::MAX && self.best_obj < f64::MAX { child_obj - self.best_obj } else { f64::MAX };
            let incumbent_improvement = if child_obj < self.best_obj { self.best_obj - child_obj } else { 0.0 };
            
            let operator_outcome = if child_obj < parent_obj { "Improved" } else if child_obj == parent_obj { "Neutral" } else { "Degraded" };
            
            let elite_descendant = branch_best_obj <= self.best_obj && branch_best_obj < f64::MAX;
            
            if subtree_survived {
                self.vault.record_success(context.clone());
            } else {
                self.vault.record_failure(context.clone());
            }
            
            self.logs.push(OperatorObservation {
                node_id,
                context,
                operator,
                parent_pressure,
                child_pressure,
                pressure_delta,
                parent_objective: if parent_obj == f64::MAX { -1.0 } else { parent_obj },
                child_objective: if child_obj == f64::MAX { -1.0 } else { child_obj },
                objective_delta,
                depth: demand_idx,
                survival_depth: branch_max_depth,
                incumbent_gap: if incumbent_gap == f64::MAX { -1.0 } else { incumbent_gap },
                elite_descendant,
                operator_outcome: operator_outcome.to_string(),
                incumbent_improvement,
            });
        }

        (any_success, best_subtree_obj, max_survival_depth)
    }
}

fn find_k_shortest(graph: &Digraph, src: u64, dst: u64, k: usize) -> Vec<Vec<u64>> {
    let mut paths = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(vec![src]);
    
    let mut explored_count = 0;
    while let Some(path) = queue.pop_front() {
        explored_count += 1;
        if explored_count > 10000 { break; } 
        
        let last = *path.last().unwrap();
        if last == dst {
            let inner_nodes: Vec<u64> = path[1..path.len()-1].to_vec();
            paths.push(inner_nodes);
            if paths.len() >= k { break; }
            continue;
        }
        
        if path.len() > 6 { continue; } 
        
        for arc in &graph.arcs {
            if arc.from == last && !path.contains(&arc.to) {
                let mut new_path = path.clone();
                new_path.push(arc.to);
                queue.push_back(new_path);
            }
        }
    }
    
    if paths.is_empty() {
        paths.push(vec![]);
    }
    paths
}

fn main() -> anyhow::Result<()> {
    println!("=== M27.1 Passive Operator Opportunity Telemetry ===");
    
    let mut f = File::create("m27_1_passive_operator_telemetry.csv")?;
    writeln!(f, "instance,node_id,context,operator,parent_pressure,child_pressure,pressure_delta,parent_objective,child_objective,objective_delta,depth,survival_depth,incumbent_gap,elite_descendant,operator_outcome,incumbent_improvement")?;
    
    for instance_id in 1..=5 {
        let instance_str = format!("{:02}", instance_id);
        println!("Processing setA-{}...", instance_str);
        
        let net_path = format!("repo/challenge-roadef-2026-main/setA/setA-{}-net.json", instance_str);
        let tm_path = format!("repo/challenge-roadef-2026-main/setA/setA-{}-tm.json", instance_str);
        let sc_path = format!("repo/challenge-roadef-2026-main/setA/setA-{}-scenario.json", instance_str);
        
        let net = load_network(&net_path)?;
        let mut tm = load_traffic_matrix(&tm_path)?;
        let scenario = load_scenario(&sc_path)?;
        
        let graph = Digraph::new(&net);
        let max_cap = net.links.iter().map(|l| l.capacity).fold(0.0_f64, f64::max);
        
        let total_demands = tm.demands.len().min(40);
        let evaluator = RoadefEvaluator::new(&net, tm.clone(), scenario);
        let mut truncated_eval = evaluator;
        truncated_eval.tm.demands.truncate(total_demands);
        
        let mut precomputed_paths = Vec::new();
        for i in 0..total_demands {
            let d = &truncated_eval.tm.demands[i];
            let paths = find_k_shortest(&graph, d.s, d.t, 3);
            precomputed_paths.push(paths);
        }
        
        let mut solver = Solver {
            evaluator: &truncated_eval,
            graph: &graph,
            max_cap,
            precomputed_paths,
            vault: CoralysVault::new(),
            logs: Vec::new(),
            node_counter: 0,
            best_obj: f64::MAX,
        };
        
        let mut initial_sol = Solution { srpaths: Vec::new() };
        solver.solve_recursive(0, &mut initial_sol, 0.1, f64::MAX);
        
        println!("Instance {} complete. Nodes visited: {}", instance_str, solver.node_counter);
        
        for log in &solver.logs {
            writeln!(f, "{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{},{:.4},{},{},{:.4}", 
                instance_id, log.node_id, log.context, log.operator, 
                log.parent_pressure, log.child_pressure, log.pressure_delta,
                log.parent_objective, log.child_objective, log.objective_delta,
                log.depth, log.survival_depth, log.incumbent_gap, log.elite_descendant,
                log.operator_outcome, log.incumbent_improvement)?;
        }
    }
    
    println!("Logs written to m27_1_passive_operator_telemetry.csv");
    Ok(())
}
