use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use rand::SeedableRng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use coralys_moga::traits::{FitnessEvaluator, MutationOperator, CrossoverOperator, GenomeFactory, Evaluated, Genome};
use cvrp::{CvrpInstance, moga_impl::*};
use cvrp::CvrpGenomeFactory;

#[derive(Serialize)]
struct FamilyStats {
    count: usize,
    best_distance: f64,
    avg_distance: f64,
}

#[derive(Serialize)]
struct Report {
    phenotype_compression_ratio: f64,
    unique_genotypes: usize,
    unique_route_families: usize,
    families: HashMap<String, FamilyStats>,
    seed_details: Vec<SeedDetail>,
}

#[derive(Serialize)]
struct SeedDetail {
    seed: u64,
    best_distance: f64,
    family_id: String,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn get_edges(routes: &Vec<Vec<usize>>) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for route in routes {
        let mut prev = 0; // Depot is 0 conceptually
        for &node in route {
            // customers are 1-indexed conceptually or 0-indexed in array?
            // in instance.customers, depot is separate.
            // Let's use 9999 for Depot ID
            let mut a = prev;
            let mut b = node + 1; // +1 to distinguish from depot
            if a > b { std::mem::swap(&mut a, &mut b); }
            edges.push((a, b));
            prev = node + 1;
        }
        let mut a = prev;
        let mut b = 0;
        if a > b { std::mem::swap(&mut a, &mut b); }
        edges.push((a, b));
    }
    edges.sort_unstable();
    edges.dedup();
    edges
}

fn get_family_id(routes: &Vec<Vec<usize>>) -> String {
    let edges = get_edges(routes);
    let mut id = String::new();
    for (a, b) in edges {
        id.push_str(&format!("{}-{},", a, b));
    }
    let hash = calculate_hash(&id);
    format!("FAM_{:016x}", hash)
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mut mutator = CvrpMutator::new(instance.clone(), cvrp::RadiusPolicy::Control);
    mutator.entropy_scale = 1.0;
    let crossover = cvrp::moga_impl::CvrpCrossoverVariant::OX1(cvrp::moga_impl::CvrpCrossover);
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };

    let num_seeds = 30;
    let generations = 10000;
    let population_size = 100;

    let mut seed_details = Vec::new();
    let mut unique_genotypes = HashSet::new();
    let mut family_map: HashMap<String, FamilyStats> = HashMap::new();

    println!("=== M8D: Characterizing the CVRP Basin ===");
    println!("Running {} seeds for {} generations...", num_seeds, generations);

    for seed_idx in 0..num_seeds {
        let seed = 42 + seed_idx as u64 * 100; // deterministic offsets
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        let mut population: Vec<_> = (0..population_size)
            .map(|_| factory.create(&mut rng))
            .collect();

        let mut global_best_distance = f64::INFINITY;
        let mut global_best_eval = None;

        for _ in 1..=generations {
            let mut evals: Vec<_> = population
                .iter()
                .map(|c| evaluator.evaluate(c, &coralys_moga::runtime::optimization::metric::MetricReport::default()))
                .filter(|e| e.is_valid())
                .collect();

            if evals.is_empty() {
                population = (0..population_size).map(|_| factory.create(&mut rng)).collect();
                continue;
            }

            evals.sort_by(|a, b| b.fitness().partial_cmp(&a.fitness()).unwrap_or(Ordering::Equal));
            let gen_best = evals[0].clone();
            
            if gen_best.eval.total_distance < global_best_distance {
                global_best_distance = gen_best.eval.total_distance;
                global_best_eval = Some(gen_best.clone());
            }

            // Tournament + Crossover + Mutation
            let mut next_gen = Vec::with_capacity(population_size);
            
            // Elitism
            let mut elite_count = 0;
            let mut iter = evals.iter();
            while elite_count < 10 {
                if let Some(e) = iter.next() {
                    next_gen.push(e.genome().clone());
                    elite_count += 1;
                } else {
                    break;
                }
            }

            while next_gen.len() < population_size {
                use rand::seq::SliceRandom;
                
                let tournament = |rng: &mut rand::rngs::StdRng| {
                    let mut best = evals.choose(rng).unwrap();
                    for _ in 0..4 {
                        let candidate = evals.choose(rng).unwrap();
                        if candidate.fitness() > best.fitness() {
                            best = candidate;
                        }
                    }
                    best.genome().clone()
                };

                let parent1 = tournament(&mut rng);
                
                if rand::Rng::gen_bool(&mut rng, 0.8) {
                    let parent2 = tournament(&mut rng);
                    let (mut child1, _child2) = crossover.crossover(&parent1, &parent2, &mut rng);
                    mutator.mutate(&mut child1, &mut rng);
                    next_gen.push(child1);
                } else {
                    let mut child = parent1.clone();
                    mutator.mutate(&mut child, &mut rng);
                    next_gen.push(child);
                }
            }

            population = next_gen;
        }

        let final_eval = global_best_eval.unwrap();
        let distance = final_eval.eval.total_distance;
        let genotype_hash = calculate_hash(&final_eval.genome().permutation);
        let family_id = get_family_id(&final_eval.eval.routes);

        unique_genotypes.insert(genotype_hash);

        let stats = family_map.entry(family_id.clone()).or_insert(FamilyStats {
            count: 0,
            best_distance: f64::INFINITY,
            avg_distance: 0.0,
        });

        stats.count += 1;
        if distance < stats.best_distance {
            stats.best_distance = distance;
        }
        stats.avg_distance += distance;

        seed_details.push(SeedDetail {
            seed,
            best_distance: distance,
            family_id: family_id.clone(),
        });

        println!("Seed {} -> Dist: {:.2} (Family {})", seed, distance, &family_id[..12]);
    }

    for stats in family_map.values_mut() {
        stats.avg_distance /= stats.count as f64;
    }

    let report = Report {
        phenotype_compression_ratio: unique_genotypes.len() as f64 / family_map.len().max(1) as f64,
        unique_genotypes: unique_genotypes.len(),
        unique_route_families: family_map.len(),
        families: family_map,
        seed_details,
    };

    let json = serde_json::to_string_pretty(&report).unwrap();
    std::fs::write("../../basin_characterization_report.json", json).unwrap();
    
    println!("Report written to basin_characterization_report.json");
}
