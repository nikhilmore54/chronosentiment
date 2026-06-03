use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer, InrcGenome};
use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::ecology::WorkforceEcology;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::{HashMap, HashSet, VecDeque};

fn max_bipartite_matching(
    nurses: &[usize], 
    nurse_skills: &HashMap<usize, Vec<String>>,
    demands: &[(String, usize)] // skill -> required count
) -> (usize, HashMap<usize, String>) {
    // 1. Create a flow network
    // Source = 0
    // Sink = 1
    // Nurses = 2 to 2+n-1
    // Demands = (2+n) to (2+n)+m-1. Each demand node represents a single unit of demand!
    // Wait, since demands can be > 1, we can just split demands into multiple nodes, 
    // each requiring 1 unit, to make it a simple bipartite matching.
    
    let mut demand_nodes = Vec::new();
    for (skill, count) in demands {
        for _ in 0..*count {
            demand_nodes.push(skill.clone());
        }
    }
    
    let num_nurses = nurses.len();
    let num_demands = demand_nodes.len();
    
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); num_nurses];
    for (i, n) in nurses.iter().enumerate() {
        if let Some(skills) = nurse_skills.get(n) {
            for (j, req_skill) in demand_nodes.iter().enumerate() {
                if skills.contains(req_skill) {
                    adj[i].push(j);
                }
            }
        }
    }
    
    // Hopcroft-Karp or simple augmenting paths
    let mut match_nurse = vec![None; num_nurses];
    let mut match_demand = vec![None; num_demands];
    
    let mut assignments = 0;
    
    for i in 0..num_nurses {
        let mut visited = vec![false; num_demands];
        if dfs(i, &adj, &mut match_nurse, &mut match_demand, &mut visited) {
            assignments += 1;
        }
    }
    
    let mut final_assignment = HashMap::new();
    for (i, d) in match_nurse.into_iter().enumerate() {
        if let Some(demand_idx) = d {
            final_assignment.insert(nurses[i], demand_nodes[demand_idx].clone());
        }
    }
    
    (assignments, final_assignment)
}

fn dfs(
    u: usize, 
    adj: &Vec<Vec<usize>>, 
    match_nurse: &mut Vec<Option<usize>>, 
    match_demand: &mut Vec<Option<usize>>, 
    visited: &mut Vec<bool>
) -> bool {
    for &v in &adj[u] {
        if !visited[v] {
            visited[v] = true;
            if match_demand[v].is_none() || dfs(match_demand[v].unwrap(), adj, match_nurse, match_demand, visited) {
                match_nurse[u] = Some(v);
                match_demand[v] = Some(u);
                return true;
            }
        }
    }
    false
}

#[test]
fn test_coverage_parity() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario, week_data, history, ecology);
    let optimizer = InrcOptimizer { context: Arc::new(context) };

    let frozen_bits_json = std::fs::read_to_string(base_dir.join("frozen_genome.json")).unwrap();
    let bits: Vec<bool> = serde_json::from_str(&frozen_bits_json).unwrap();
    let genome = InrcGenome { bits };

    // We will extract a single day/shift where there is a mismatch.
    // Or just run the matching for the entire week and see if it achieves 360 penalty!
    
    let num_days = optimizer.context.num_days;
    let num_shifts = optimizer.context.shift_types.len();
    
    let mut total_optimal_penalty = 0;
    
    for d in 0..num_days {
        let day_name = match d {
            0 => "Mon", 1 => "Tue", 2 => "Wed", 3 => "Thu", 4 => "Fri", 5 => "Sat", 6 => "Sun", _ => unreachable!()
        };
        for s in 0..num_shifts {
            let shift_name = &optimizer.context.shift_types[s];
            
            let mut available_nurses = Vec::new();
            for n in 0..optimizer.context.num_nurses {
                if optimizer.get_bit(&genome, n, d, s) {
                    available_nurses.push(n);
                }
            }
            
            let mut demands = Vec::new();
            for req in &optimizer.context.week_data.requirements {
                if req.shift_type == *shift_name {
                    let req_level = match day_name {
                        "Mon" => &req.monday, "Tue" => &req.tuesday, "Wed" => &req.wednesday,
                        "Thu" => &req.thursday, "Fri" => &req.friday, "Sat" => &req.saturday, "Sun" => &req.sunday,
                        _ => unreachable!(),
                    };
                    if req_level.optimal > 0 {
                        demands.push((req.skill.clone(), req_level.optimal));
                    }
                }
            }
            
            let mut nurse_skills = HashMap::new();
            for &n in &available_nurses {
                nurse_skills.insert(n, optimizer.context.scenario.nurses[n].skills.clone());
            }
            
            let (matched, _) = max_bipartite_matching(&available_nurses, &nurse_skills, &demands);
            
            let total_optimal: usize = demands.iter().map(|(_, c)| c).sum();
            if matched < total_optimal {
                let p = (total_optimal - matched) * 30;
                total_optimal_penalty += p;
                println!("Day {} Shift {}: Opt {}, Matched {}, Penalty +{}", d, shift_name, total_optimal, matched, p);
            }
        }
    }
    
    println!("Maximum Bipartite Matching Penalty: {}", total_optimal_penalty);
    assert_eq!(total_optimal_penalty, 360, "Bipartite matching should yield 360 penalty!");
}
