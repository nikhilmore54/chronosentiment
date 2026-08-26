use coralys_moga::traits::{Evaluated, FitnessEvaluator};
use cvrp::moga_impl::CvrpEvaluator;
use cvrp::{CvrpCandidate, CvrpInstance, DistanceMetric, Node};
use std::env;
use std::fs;
use std::time::Instant;

fn parse_vrp_file(content: &str) -> CvrpInstance {
    let mut capacity = 0;
    let mut coords = Vec::new();
    let mut demands = Vec::new();

    let mut section = "";
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
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
    let depot = Node {
        id: depot_coord.0,
        x: depot_coord.1,
        y: depot_coord.2,
        demand: depot_demand.1,
    };

    let mut customers = Vec::new();
    for coord in coords {
        if coord.0 == 1 {
            continue;
        }
        let demand = demands.iter().find(|(id, _)| *id == coord.0).unwrap();
        customers.push(Node {
            id: coord.0,
            x: coord.1,
            y: coord.2,
            demand: demand.1,
        });
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: a5_smoke <instance_path>");
        std::process::exit(1);
    }
    let instance_path = &args[1];
    let content = fs::read_to_string(instance_path).unwrap();

    let mut instance = parse_vrp_file(&content);
    if instance_path.contains("k5") {
        instance.max_vehicles = Some(5);
    } else if instance_path.contains("k6") {
        instance.max_vehicles = Some(6);
    }

    let strategy_str =
        env::var("CVRP_SPLIT_STRATEGY").unwrap_or_else(|_| "DPFallbackToGreedy".to_string());

    let seed = 42;
    let n = instance.customers.len();

    let perm: Vec<usize> = (0..n).collect();
    let candidate = CvrpCandidate {
        permutation: perm,
        last_mutation_op: None,
        last_mutation_radius: None,
        route_boundary_changes: None,
    };

    let start = Instant::now();
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let outcome = evaluator.evaluate(
        &candidate,
        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
    );
    let elapsed = start.elapsed();

    let final_objective = 100000.0 - outcome.fitness();
    let feasible = outcome.eval.routes.len() <= instance.max_vehicles.unwrap_or(n);
    let num_routes = outcome.eval.routes.len();

    println!("=== A5 Smoke Test: Single Candidate Evaluation ===");
    println!("Instance: {}", instance_path);
    println!("Strategy: {}", strategy_str);
    println!("Seed: {}", seed);
    println!("Final Objective: {:.2}", final_objective);
    println!("Feasible: {}", feasible);
    println!("Num Routes: {}", num_routes);
    println!("Runtime: {:?}", elapsed);
    println!("Routes:");
    for (i, r) in outcome.eval.routes.iter().enumerate() {
        println!("  Route {}: {:?}", i + 1, r);
    }
}
