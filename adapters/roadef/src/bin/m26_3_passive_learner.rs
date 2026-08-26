use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use roadef::evaluator::RoadefEvaluator;
use roadef::graph::Digraph;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::models::{Solution, SrPath};

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
        Self {
            stats: HashMap::new(),
        }
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
        let entry = self.stats.entry(context).or_insert(ContextStats {
            success: 0,
            failure: 0,
        });
        entry.success += 1;
    }

    fn record_failure(&mut self, context: String) {
        let entry = self.stats.entry(context).or_insert(ContextStats {
            success: 0,
            failure: 0,
        });
        entry.failure += 1;
    }
}

// Data model for logging
struct DecisionObservation {
    node_id: u64,
    depth: usize,
    context: String,
    pressure_score: f64,
    subtree_survived: bool,
    best_descendant_objective: f64,
    became_champion: bool,
}

fn build_context_tag(
    graph: &Digraph,
    path_nodes: &[u64],
    vol: f64,
    max_cap: f64,
    avg_congestion: f64,
) -> String {
    let vol_ratio = vol / max_cap;
    let vol_cat = if vol_ratio > 0.8 {
        "Vol:Extreme"
    } else if vol_ratio > 0.5 {
        "Vol:High"
    } else if vol_ratio > 0.3 {
        "Vol:Med"
    } else if vol_ratio > 0.1 {
        "Vol:Low"
    } else {
        "Vol:Tiny"
    };

    let len_cat = if path_nodes.len() > 8 {
        "Len:Extreme"
    } else if path_nodes.len() > 6 {
        "Len:Long"
    } else if path_nodes.len() > 4 {
        "Len:Med"
    } else if path_nodes.len() > 2 {
        "Len:Short"
    } else {
        "Len:Tiny"
    };

    let crosses_core = if path_nodes.iter().any(|&n| n < 5) {
        "Core:Deep"
    } else if path_nodes.iter().any(|&n| n < 10) {
        "Core:Yes"
    } else {
        "Core:No"
    };

    let cong_cat = if avg_congestion > 0.9 {
        "Cong:Extreme"
    } else if avg_congestion > 0.7 {
        "Cong:High"
    } else if avg_congestion > 0.5 {
        "Cong:Med"
    } else if avg_congestion > 0.3 {
        "Cong:Low"
    } else {
        "Cong:None"
    };

    format!("{}|{}|{}|{}", vol_cat, len_cat, crosses_core, cong_cat)
}

struct Solver<'a> {
    evaluator: &'a RoadefEvaluator,
    graph: &'a Digraph,
    max_cap: f64,
    precomputed_paths: Vec<Vec<Vec<u64>>>,
    vault: CoralysVault,
    logs: Vec<DecisionObservation>,
    node_counter: u64,
    best_obj: f64,
}

impl<'a> Solver<'a> {
    fn solve_recursive(
        &mut self,
        demand_idx: usize,
        current_solution: &mut Solution,
        current_avg_congestion: f64,
    ) -> (bool, f64) {
        // Returns (subtree_survived, best_subtree_obj)
        if self.node_counter > 50_000 {
            return (false, f64::MAX);
        }
        if demand_idx >= self.evaluator.tm.demands.len() {
            // Reached leaf -> valid solution!
            let res = self.evaluator.evaluate_solution(current_solution);
            if res.valid && res.obj.is_finite() {
                if res.obj < self.best_obj {
                    self.best_obj = res.obj;
                }
                return (true, res.obj);
            }
            return (false, f64::MAX);
        }

        let mut best_subtree_obj = f64::MAX;
        let mut any_success = false;

        let paths = self.precomputed_paths[demand_idx].clone();
        for path in paths {
            if self.node_counter > 50_000 {
                break;
            }
            let demand = &self.evaluator.tm.demands[demand_idx];

            let mut path_nodes = vec![demand.s];
            path_nodes.extend_from_slice(&path);
            path_nodes.push(demand.t);

            let context = build_context_tag(
                self.graph,
                &path_nodes,
                demand.v[0],
                self.max_cap,
                current_avg_congestion,
            );
            let pressure = self.vault.query_pressure(&context);

            let node_id = self.node_counter;
            self.node_counter += 1;

            // Apply decision
            current_solution.srpaths.push(SrPath {
                d: demand_idx,
                t: 0,
                w: path.clone(),
            });

            // Fast fail evaluation
            let res = self.evaluator.evaluate_solution(current_solution);

            let mut subtree_survived = false;
            let mut branch_best_obj = f64::MAX;

            if res.valid {
                let next_cong = current_avg_congestion + (demand.v[0] / self.max_cap) * 0.05;
                let (sub_success, sub_obj) =
                    self.solve_recursive(demand_idx + 1, current_solution, next_cong);

                subtree_survived = sub_success;
                branch_best_obj = sub_obj;

                if sub_success {
                    any_success = true;
                    best_subtree_obj = best_subtree_obj.min(sub_obj);
                }
            }

            // Backtrack
            current_solution.srpaths.pop();

            // Update Vault memory EXACTLY ONCE per decision (Fixes DFS credit leakage)
            if subtree_survived {
                self.vault.record_success(context.clone());
            } else {
                self.vault.record_failure(context.clone());
            }

            let became_champion = branch_best_obj <= self.best_obj && branch_best_obj < f64::MAX;

            self.logs.push(DecisionObservation {
                node_id,
                depth: demand_idx,
                context,
                pressure_score: pressure,
                subtree_survived,
                best_descendant_objective: branch_best_obj,
                became_champion,
            });
        }

        (any_success, best_subtree_obj)
    }
}

// Simple BFS to find K shortest paths for a demand
fn find_k_shortest(graph: &Digraph, src: u64, dst: u64, k: usize) -> Vec<Vec<u64>> {
    let mut paths = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(vec![src]);

    let mut explored_count = 0;
    while let Some(path) = queue.pop_front() {
        explored_count += 1;
        if explored_count > 10000 {
            break;
        } // prevent exponential blowup on dense graphs

        let last = *path.last().unwrap();
        if last == dst {
            let inner_nodes: Vec<u64> = path[1..path.len() - 1].to_vec();
            paths.push(inner_nodes);
            if paths.len() >= k {
                break;
            }
            continue;
        }

        if path.len() > 6 {
            continue;
        } // max depth

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
    println!("=== M26.3A.1 & M26.3A.2 Passive Learner ===");

    let mut f = File::create("m26_3_passive_logs.csv")?;
    writeln!(f, "instance,node_id,search_depth,context,pressure_score,subtree_survived,best_descendant_objective,became_champion")?;

    for instance_id in 1..=7 {
        let instance_str = format!("{:02}", instance_id);
        println!("Processing setA-{}...", instance_str);

        let net_path = format!(
            "repo/challenge-roadef-2026-main/setA/setA-{}-net.json",
            instance_str
        );
        let tm_path = format!(
            "repo/challenge-roadef-2026-main/setA/setA-{}-tm.json",
            instance_str
        );
        let sc_path = format!(
            "repo/challenge-roadef-2026-main/setA/setA-{}-scenario.json",
            instance_str
        );

        let net = load_network(&net_path)?;
        let mut tm = load_traffic_matrix(&tm_path)?;
        let scenario = load_scenario(&sc_path)?;

        // M26.3A.0 Natural Ecology Audit - No demand inflation, No capacity scaling
        // We will process a larger number of demands to see if natural failures occur
        let graph = Digraph::new(&net);
        let max_cap = net.links.iter().map(|l| l.capacity).fold(0.0_f64, f64::max);

        let total_demands = tm.demands.len().min(12);
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

        let mut initial_sol = Solution {
            srpaths: Vec::new(),
        };
        solver.solve_recursive(0, &mut initial_sol, 0.1);

        println!(
            "Instance {} complete. Nodes visited: {}",
            instance_str, solver.node_counter
        );

        for log in &solver.logs {
            let obj_str = if log.best_descendant_objective == f64::MAX {
                -1.0
            } else {
                log.best_descendant_objective
            };
            writeln!(
                f,
                "{},{},{},{},{:.4},{},{:.4},{}",
                instance_id,
                log.node_id,
                log.depth,
                log.context,
                log.pressure_score,
                log.subtree_survived,
                obj_str,
                log.became_champion
            )?;
        }
    }

    println!("Logs written to m26_3_passive_logs.csv");
    Ok(())
}
