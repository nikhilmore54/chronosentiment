use std::fs;
use std::collections::HashSet;
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::Rng;
use serde::Deserialize;
use coralys_moga::traits::{FitnessEvaluator, MutationOperator, CrossoverOperator, ImprovementOperator, GenomeFactory};
use cvrp::{CvrpInstance, CvrpClusteredGenomeFactory, DistanceMetric};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpCrossover, CvrpLocalSearch};

#[derive(Debug, Deserialize, Clone)]
struct InstanceMetadata {
    name: String,
    family: String,
    customers: usize,
    vehicles: usize,
    capacity: usize,
    bks: f64,
    instance_url: String,
}

fn parse_vrp_file(content: &str) -> CvrpInstance {
    let mut capacity = 0;
    let mut coords = Vec::new();
    let mut demands = Vec::new();
    
    let mut in_coord_section = false;
    let mut in_demand_section = false;
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        if line.starts_with("CAPACITY") {
            let parts: Vec<&str> = line.split(':').collect();
            capacity = parts[1].trim().parse().unwrap();
        } else if line.starts_with("NODE_COORD_SECTION") {
            in_coord_section = true;
            in_demand_section = false;
        } else if line.starts_with("DEMAND_SECTION") {
            in_coord_section = false;
            in_demand_section = true;
        } else if line.starts_with("DEPOT_SECTION") || line.starts_with("EOF") {
            in_coord_section = false;
            in_demand_section = false;
        } else if in_coord_section {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let id: usize = parts[0].parse().unwrap();
                let x: f64 = parts[1].parse().unwrap();
                let y: f64 = parts[2].parse().unwrap();
                coords.push((id, x, y));
            }
        } else if in_demand_section {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id: usize = parts[0].parse().unwrap();
                let demand: usize = parts[1].parse().unwrap();
                demands.push((id, demand));
            }
        }
    }
    
    let depot_coord = coords.iter().find(|(id, _, _)| *id == 1).unwrap();
    let depot_demand = demands.iter().find(|(id, _)| *id == 1).unwrap();
    let depot = cvrp::Node { id: depot_coord.0, x: depot_coord.1, y: depot_coord.2, demand: depot_demand.1 as i32 };
    
    let mut customers = Vec::new();
    for coord in coords {
        if coord.0 == 1 { continue; }
        let demand = demands.iter().find(|(id, _)| *id == coord.0).unwrap();
        customers.push(cvrp::Node { id: coord.0, x: coord.1, y: coord.2, demand: demand.1 as i32 });
    }
    
    CvrpInstance {
        capacity: capacity as i32,
        depot,
        customers,
        distance_metric: DistanceMetric::TspLibEuc2D,
        max_vehicles: None,
    }
}

fn run_telemetry(name: &str, metadata: &InstanceMetadata) {
    println!("\n========================================================");
    println!("RUNNING DETAILED TELEMETRY FOR INSTANCE: {}", name);
    println!("k_limit: {}, Capacity: {}, BKS: {}", metadata.vehicles, metadata.capacity, metadata.bks);
    println!("========================================================");

    let vrp_path = format!("/Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/cvrp/data/instances/{}.vrp", name);
    let content = fs::read_to_string(vrp_path).unwrap();
    let mut instance = parse_vrp_file(&content);
    instance.max_vehicles = Some(metadata.vehicles);

    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    let crossover = CvrpCrossover;
    let factory = CvrpClusteredGenomeFactory { instance: instance.clone() };
    let local_search = CvrpLocalSearch { instance: instance.clone() };

    let mut rng = StdRng::seed_from_u64(42);
    let pop_size = 200;
    let elite_count = 20;
    let generation_limit = 50;

    // Initialize population
    let mut population = Vec::with_capacity(pop_size);
    for _ in 0..pop_size {
        population.push(factory.create(&mut rng));
    }

    for generation in 1..=generation_limit {
        // Evaluate population
        let mut evals = Vec::new();
        for ind in &population {
            evals.push(evaluator.evaluate(ind));
        }

        // Gather telemetry metrics
        let mut feasible_count = 0;
        let mut infeasible_count = 0;
        let mut total_feasible_vehicles = 0;
        let mut best_feasible_obj = f64::INFINITY;
        let mut unique_perms = HashSet::new();

        for ev in &evals {
            unique_perms.insert(ev.eval.candidate.permutation.clone());
            let is_feasible = ev.eval.total_distance < 100000.0;
            if is_feasible {
                feasible_count += 1;
                total_feasible_vehicles += ev.eval.routes.len();
                if ev.eval.total_distance < best_feasible_obj {
                    best_feasible_obj = ev.eval.total_distance;
                }
            } else {
                infeasible_count += 1;
            }
        }

        let avg_feasible_vehicles = if feasible_count > 0 {
            total_feasible_vehicles as f64 / feasible_count as f64
        } else {
            0.0
        };
        let diversity = unique_perms.len() as f64 / pop_size as f64 * 100.0;

        // Sort by fitness (descending, i.e., lower total_distance is higher fitness)
        evals.sort_by(|a, b| a.eval.total_distance.partial_cmp(&b.eval.total_distance).unwrap());

        let mut unique_elites = HashSet::new();
        for ev in evals.iter().take(elite_count) {
            unique_elites.insert(ev.eval.candidate.permutation.clone());
        }
        let unique_elite_count = unique_elites.len();

        println!(
            "Gen {:02}: Feasible: {:3} | Infeasible: {:3} | Avg Feasible Vehicles: {:.2} | Best Feasible: {:.2} | Diversity: {:.1}% | Unique Elites: {}",
            generation, feasible_count, infeasible_count, avg_feasible_vehicles, best_feasible_obj, diversity, unique_elite_count
        );

        // Select elite
        let mut next_gen = Vec::with_capacity(pop_size);
        for i in 0..elite_count {
            next_gen.push(evals[i].eval.candidate.clone());
        }

        // Generate rest
        while next_gen.len() < pop_size {
            // Tournament selection
            let t_size = 5;
            let mut best_idx1 = rng.gen_range(0..pop_size);
            for _ in 0..t_size-1 {
                let idx = rng.gen_range(0..pop_size);
                if evals[idx].eval.total_distance < evals[best_idx1].eval.total_distance {
                    best_idx1 = idx;
                }
            }
            let parent1 = &evals[best_idx1].eval.candidate;

            let mut best_idx2 = rng.gen_range(0..pop_size);
            for _ in 0..t_size-1 {
                let idx = rng.gen_range(0..pop_size);
                if evals[idx].eval.total_distance < evals[best_idx2].eval.total_distance {
                    best_idx2 = idx;
                }
            }
            let parent2 = &evals[best_idx2].eval.candidate;

            // Crossover
            let (mut child, _) = if rng.r#gen::<f64>() < 0.8 {
                crossover.crossover(parent1, parent2, &mut rng)
            } else {
                (parent1.clone(), parent2.clone())
            };

            // Mutation
            if rng.r#gen::<f64>() < 0.2 {
                mutator.mutate(&mut child, &mut rng);
            }

            // Local Search
            local_search.improve(&mut child);

            next_gen.push(child);
        }

        population = next_gen;
    }
}

fn main() {
    let metadata_path = "/Users/nikhil/.gemini/antigravity/brain/262ffe5d-aed4-43c6-a002-28b6911113bc/scratch/instances.json";
    let metadata_json = fs::read_to_string(metadata_path).unwrap();
    let all_instances: Vec<InstanceMetadata> = serde_json::from_str(&metadata_json).unwrap();

    let targets = vec!["B-n57-k7", "P-n50-k8", "A-n61-k9", "P-n55-k15"];

    for target in targets {
        if let Some(meta) = all_instances.iter().find(|inst| inst.name == target) {
            run_telemetry(target, meta);
        }
    }
}
