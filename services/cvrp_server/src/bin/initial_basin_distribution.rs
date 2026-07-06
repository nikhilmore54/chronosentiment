use serde::Serialize;
use std::collections::HashMap;
use rand::SeedableRng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use coralys_moga::traits::{FitnessEvaluator, LocalSearchOperator, GenomeFactory};
use cvrp::{CvrpInstance, moga_impl::*};
use cvrp::CvrpGenomeFactory;

#[derive(Serialize)]
struct BasinStats {
    hash: String,
    frequency: usize,
    best_distance: f64,
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn get_canonical_signature(routes: &Vec<Vec<usize>>) -> String {
    let mut canonical = routes.clone();
    for r in &mut canonical {
        r.sort_unstable();
    }
    canonical.sort_unstable();
    let hash = calculate_hash(&canonical);
    format!("B{:016x}", hash)
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let local_search = cvrp::moga_impl::CvrpLocalSearch { instance: instance.clone() };
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };

    let num_starts = 1000;
    let mut basin_map: HashMap<String, BasinStats> = HashMap::new();

    println!("Running {} random starts for A-n32-k5...", num_starts);

    for seed_idx in 0..num_starts {
        let seed = 42 + seed_idx as u64;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        // 1. Generate purely random permutation
        let mut cand = factory.create(&mut rng);

        // 2. Exhaustive local search descent
        coralys_moga::traits::LocalSearchOperator::search(&local_search, &mut cand);

        // 3. Evaluate final optimum
        let eval = evaluator.evaluate(&cand);
        let distance = eval.eval.total_distance;
        let basin_hash = get_canonical_signature(&eval.eval.routes);

        let stats = basin_map.entry(basin_hash.clone()).or_insert(BasinStats {
            hash: basin_hash,
            frequency: 0,
            best_distance: distance,
        });

        stats.frequency += 1;
        if distance < stats.best_distance {
            stats.best_distance = distance;
        }

        if seed_idx % 100 == 0 {
            println!("Processed {} starts...", seed_idx);
        }
    }

    let mut results: Vec<_> = basin_map.into_values().collect();
    results.sort_by(|a, b| b.frequency.cmp(&a.frequency));

    println!("\n| Basin Hash | Frequency | Distance |");
    println!("| ---------- | --------- | -------- |");
    for stats in &results {
        println!("| {:<10} | {:<9} | {:.2}   |", &stats.hash[0..10], stats.frequency, stats.best_distance);
    }
    
    println!("\nTotal Unique Basins Discovered: {}", results.len());
    let top_capture = results.first().unwrap().frequency as f64 / num_starts as f64 * 100.0;
    println!("Largest Basin Capture Volume: {:.1}%", top_capture);
}
