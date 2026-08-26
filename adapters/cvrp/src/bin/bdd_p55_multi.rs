use coralys_moga::traits::{
    CrossoverOperator, FitnessEvaluator, GenomeFactory, ImprovementOperator, MutationOperator,
};
use cvrp::moga_impl::{CvrpCrossover, CvrpEvaluator, CvrpLocalSearch, CvrpMutator};
use cvrp::{CvrpGenomeFactory, CvrpInstance, DistanceMetric};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::fs;

fn parse_vrp_file(content: &str) -> CvrpInstance {
    let mut capacity = 0;
    let mut coords = Vec::new();
    let mut demands = Vec::new();

    let mut in_coord_section = false;
    let mut in_demand_section = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

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
    let depot = cvrp::Node {
        id: depot_coord.0,
        x: depot_coord.1,
        y: depot_coord.2,
        demand: depot_demand.1 as i32,
    };

    let mut customers = Vec::new();
    for coord in coords {
        if coord.0 == 1 {
            continue;
        }
        let demand = demands.iter().find(|(id, _)| *id == coord.0).unwrap();
        customers.push(cvrp::Node {
            id: coord.0,
            x: coord.1,
            y: coord.2,
            demand: demand.1 as i32,
        });
    }

    CvrpInstance {
        capacity: capacity as i32,
        depot,
        customers,
        distance_metric: DistanceMetric::TspLibEuc2D,
        max_vehicles: None,
        explicit_matrix: vec![],
    }
}

fn run_ga(seed: u64, instance: &CvrpInstance) -> f64 {
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    let crossover = CvrpCrossover;
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };
    let local_search = CvrpLocalSearch {
        instance: instance.clone(),
    };

    let mut rng = StdRng::seed_from_u64(seed);
    let pop_size = 200;
    let elite_count = 20;
    let generation_limit = 50;

    let mut population = Vec::with_capacity(pop_size);
    for _ in 0..pop_size {
        population.push(factory.create(&mut rng));
    }

    let mut best_overall = f64::INFINITY;

    for _generation in 1..=generation_limit {
        let mut evals = Vec::new();
        for ind in &population {
            evals.push(evaluator.evaluate(
                ind,
                &coralys_moga::runtime::optimization::metric::MetricReport::default(),
            ));
        }

        for ev in &evals {
            if ev.eval.total_distance < best_overall {
                best_overall = ev.eval.total_distance;
            }
        }

        evals.sort_by(|a, b| {
            a.eval
                .total_distance
                .partial_cmp(&b.eval.total_distance)
                .unwrap()
        });

        let mut next_gen = Vec::with_capacity(pop_size);
        for i in 0..elite_count {
            next_gen.push(evals[i].eval.candidate.clone());
        }

        while next_gen.len() < pop_size {
            // Simple selection
            let parent1 = &evals[0].eval.candidate;
            let parent2 = &evals[1].eval.candidate;

            let (mut child, _) = if rand::Rng::r#gen::<f64>(&mut rng) < 0.8 {
                crossover.crossover(parent1, parent2, &mut rng)
            } else {
                (parent1.clone(), parent2.clone())
            };

            if rand::Rng::r#gen::<f64>(&mut rng) < 0.2 {
                mutator.mutate(&mut child, &mut rng);
            }

            let model = cvrp::moga_impl::CvrpConstraintModel {
                instance: instance.clone(),
            };
            let budget = coralys_core::operators::OperatorBudget {
                max_iterations: 1,
                max_time_ms: 1000,
            };
            coralys_core::operators::ImprovementOperator::improve(
                &local_search,
                &mut child,
                &model,
                &budget,
            )
            .unwrap();
            next_gen.push(child);
        }

        population = next_gen;
    }

    best_overall
}

fn main() {
    println!("=== Running 30-seed Replication Experiment on P-n55-k8 ===");
    let vrp_path =
        "/Users/nikhil/ChronoSentiment_MEGA_FINAL/adapters/cvrp/data/instances/P-n55-k8.vrp";
    let content = fs::read_to_string(vrp_path).unwrap();
    let mut instance = parse_vrp_file(&content);
    instance.max_vehicles = Some(8);

    let mut costs = Vec::new();
    let mut rediscoveries = 0;

    for seed in 1..=30 {
        let cost = run_ga(seed, &instance);
        costs.push(cost);
        if cost <= 576.0 {
            rediscoveries += 1;
        }
        println!("Seed {:02}: Best Cost = {:.2}", seed, cost);
    }

    costs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let best = costs[0];
    let worst = costs[29];
    let median = costs[15];
    let mean = costs.iter().sum::<f64>() / 30.0;

    let variance = costs.iter().map(|c| (c - mean) * (c - mean)).sum::<f64>() / 30.0;
    let std_dev = variance.sqrt();

    println!("\n=== Statistical Summary (P-n55-k8) ===");
    println!("Best: {:.2}", best);
    println!("Worst: {:.2}", worst);
    println!("Mean: {:.2}", mean);
    println!("Median: {:.2}", median);
    println!("Std Dev: {:.4}", std_dev);
    println!(
        "576 Rediscovered: {}/30 runs ({:.1}%)",
        rediscoveries,
        (rediscoveries as f64 / 30.0) * 100.0
    );
}
