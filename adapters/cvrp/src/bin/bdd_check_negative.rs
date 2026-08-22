use std::fs;
use cvrp::{CvrpInstance, CvrpGenomeFactory, DistanceMetric, RadiusPolicy, Node};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpCrossover, CvrpLocalSearch};
use coralys_moga::{EvolutionConfig, EvolutionEngineBuilder};

fn parse_vrp_file(content: &str) -> CvrpInstance {
    let mut capacity = 0;
    let mut coords = Vec::new();
    let mut demands = Vec::new();
    
    let mut section = "";
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line.starts_with("CAPACITY") {
            let parts: Vec<&str> = line.split(':').collect();
            capacity = parts[1].trim().parse().unwrap();
        } else if line.starts_with("NODE_COORD_SECTION") {
            section = "coords";
            continue;
        } else if line.starts_with("DEMAND_SECTION") {
            section = "demands";
            continue;
        } else if line.starts_with("DEPOT_SECTION") || line.starts_with("EOF") {
            section = "";
            continue;
        }
        
        if section == "coords" {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let id: usize = parts[0].parse().unwrap();
                let x: f64 = parts[1].parse().unwrap();
                let y: f64 = parts[2].parse().unwrap();
                coords.push((id, x, y));
            }
        } else if section == "demands" {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id: usize = parts[0].parse().unwrap();
                let demand: i32 = parts[1].parse().unwrap();
                demands.push((id, demand));
            }
        }
    }
    
    let depot_coord = coords.iter().find(|(id, _, _)| *id == 1).unwrap();
    let depot_demand = demands.iter().find(|(id, _)| *id == 1).unwrap();
    let depot = Node { id: depot_coord.0, x: depot_coord.1, y: depot_coord.2, demand: depot_demand.1 };
    
    let mut customers = Vec::new();
    for coord in coords {
        if coord.0 == 1 { continue; }
        let demand = demands.iter().find(|(id, _)| *id == coord.0).unwrap();
        customers.push(Node { id: coord.0, x: coord.1, y: coord.2, demand: demand.1 });
    }
    
    CvrpInstance {
        capacity,
        depot,
        customers,
        distance_metric: DistanceMetric::TspLibEuc2D,
        max_vehicles: None,
        explicit_matrix: vec![],
    }
}

fn check_instance(name: &str, target_bks: f64) {
    println!("\n=== Checking Coralys Solution for {} ===", name);
    let vrp_path = format!("/Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/cvrp/data/instances/{}.vrp", name);
    let content = fs::read_to_string(vrp_path).unwrap();
    let mut instance = parse_vrp_file(&content);
    let k_limit: usize = name.split("-k").last().unwrap().parse().unwrap();
    instance.max_vehicles = Some(k_limit);
    
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    let crossover = CvrpCrossover;
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    let local_search = CvrpLocalSearch { instance: instance.clone() };
    
    let config = EvolutionConfig {
        population_size: 200,
        elite_count: 20,
        generation_limit: 50,
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        seed: Some(42),
        tournament_size: Some(5),
        ..Default::default()
    };
    
    let engine = EvolutionEngineBuilder::new()
        .with_evaluator(evaluator)
        .with_mutator(mutator)
        .with_crossover(crossover)
        .with_factory(factory)
        .build()
        .unwrap();
        
    let result = engine.run_ga_evolution(config).unwrap();
    let best = result.global_best;
    
    println!("Coralys Result Cost: {}", best.eval.total_distance_integer);
    println!("Coralys Result Vehicles: {}", best.eval.num_vehicles);
    println!("Published BKS Cost: {}", target_bks);
    
    // Print all routes and their demands
    let mut all_visited = std::collections::HashSet::new();
    let mut capacity_violations = 0;
    
    for (r_idx, route) in best.eval.routes.iter().enumerate() {
        let load: i32 = route.iter().map(|&node_id| {
            if node_id == instance.depot.id {
                0
            } else {
                instance.customers.iter().find(|c| c.id == node_id).unwrap().demand
            }
        }).sum();
        
        println!("  Route #{}: {:?} (Load: {}/{})", r_idx + 1, route, load, instance.capacity);
        if load > instance.capacity {
            capacity_violations += 1;
        }
        for &node_id in route {
            all_visited.insert(node_id);
        }
    }
    
    println!("Capacity violations: {}", capacity_violations);
    println!("Unique customers visited: {} (Expected {})", all_visited.len(), instance.customers.len());
}

fn main() {
    check_instance("B-n51-k7", 1032.0);
    check_instance("B-n57-k7", 1153.0);
    check_instance("P-n22-k8", 603.0);
    check_instance("P-n55-k8", 588.0);
    check_instance("P-n55-k15", 989.0);
}
