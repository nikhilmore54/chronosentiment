use coralys_moga::traits::{FitnessEvaluator, MutationOperator, LocalSearchOperator, GenomeFactory};
use cvrp::{CvrpInstance, CvrpCandidate, RadiusPolicy, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpRouteAwareMutator, CvrpLocalSearch};
use std::collections::{HashSet, VecDeque};
use rand::SeedableRng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io::Write;

fn hash_routes(routes: &[Vec<usize>]) -> u64 {
    let mut normalized = routes.to_vec();
    for r in &mut normalized { r.sort(); }
    normalized.sort();
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    hasher.finish()
}

fn run_bfs_probe(
    name: &str,
    mutator: &dyn MutationOperator<CvrpCandidate>,
    root: &CvrpCandidate,
    evaluator: &CvrpEvaluator,
    ls: &CvrpLocalSearch,
    rng: &mut rand::rngs::StdRng
) {
    println!("Starting BFS Probe for {}", name);
    let mut queue = VecDeque::new();
    let mut visited_basins = HashSet::new();
    
    // Evaluate and local search the root just to be safe
    let mut root_opt = root.clone();
    ls.search(&mut root_opt);
    let root_eval = evaluator.evaluate(&root_opt);
    let root_hash = hash_routes(&root_eval.eval.routes);
    
    queue.push_back(root_opt);
    visited_basins.insert(root_hash);
    
    let mut edges = Vec::new();
    let mut edge_count = 0;
    let max_edges = 10000;
    
    let mut file = std::fs::File::create(format!("m10c_{}_edges.csv", name)).unwrap();
    writeln!(file, "source_hash,target_hash,distance,is_elite").unwrap();

    let branching_factor = 50; // Gen 50 children per basin

    while let Some(current_cand) = queue.pop_front() {
        if edge_count >= max_edges { break; }
        
        let current_eval = evaluator.evaluate(&current_cand);
        let current_hash = hash_routes(&current_eval.eval.routes);

        for _ in 0..branching_factor {
            if edge_count >= max_edges { break; }

            let mut child = current_cand.clone();
            mutator.mutate(&mut child, rng);
            
            let mut child_opt = child.clone();
            ls.search(&mut child_opt);
            
            let child_eval = evaluator.evaluate(&child_opt);
            let child_hash = hash_routes(&child_eval.eval.routes);
            let dist = child_eval.eval.total_distance;
            let is_elite = dist <= 810.0;

            writeln!(file, "{},{},{},{}", current_hash, child_hash, dist, is_elite).unwrap();
            edges.push((current_hash, child_hash, dist, is_elite));
            edge_count += 1;

            if is_elite && !visited_basins.contains(&child_hash) {
                visited_basins.insert(child_hash);
                queue.push_back(child_opt);
            }
        }
    }
    
    println!("{} Probe Finished. Unique Elite Basins Found: {}", name, visited_basins.len());
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let ls = CvrpLocalSearch { instance: instance.clone() };
    
    println!("Searching for a 797 basin...");
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    
    let mut best_cand = factory.create(&mut rng);
    let mut best_dist = f64::MAX;
    
    // Quick random search to find an elite starting point
    let random_mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    
    for _ in 0..2000 {
        let mut child = best_cand.clone();
        random_mutator.mutate(&mut child, &mut rng);
        ls.search(&mut child);
        let eval = evaluator.evaluate(&child);
        if eval.eval.total_distance < best_dist {
            best_dist = eval.eval.total_distance;
            best_cand = child;
            println!("Found improved distance: {}", best_dist);
            if best_dist <= 798.0 {
                break;
            }
        }
    }
    
    println!("Root Basin Distance: {}", best_dist);

    let route_aware_mutator = CvrpRouteAwareMutator { instance: instance.clone() };
    
    run_bfs_probe("route_aware", &route_aware_mutator, &best_cand, &evaluator, &ls, &mut rng);
    run_bfs_probe("random", &random_mutator, &best_cand, &evaluator, &ls, &mut rng);
}
