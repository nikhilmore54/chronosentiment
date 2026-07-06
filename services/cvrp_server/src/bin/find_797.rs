use cvrp::{CvrpInstance, CvrpCandidate};
use cvrp::moga_impl::{CvrpEvaluator, CvrpLocalSearch};
use coralys_moga::traits::{FitnessEvaluator, LocalSearchOperator};
use rand::SeedableRng;
use rand::seq::SliceRandom;
use std::time::Instant;

fn random_ls(instance: &CvrpInstance, ls: &CvrpLocalSearch, rng: &mut rand::rngs::StdRng) -> CvrpCandidate {
    let mut cand = CvrpCandidate { permutation: (0..instance.customers.len()).collect(), last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None };
    cand.permutation.shuffle(rng);
    ls.search(&mut cand);
    cand
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let ls = CvrpLocalSearch { instance: instance.clone() };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    
    let mut best_dist = f64::MAX;
    let mut attempts = 0;
    
    loop {
        let cand = random_ls(&instance, &ls, &mut rng);
        let eval = evaluator.evaluate(&cand);
        let dist = eval.eval.total_distance;
        attempts += 1;
        
        if dist < best_dist {
            best_dist = dist;
            println!("New best: {:.4} at attempt {}", best_dist, attempts);
        }
        
        if dist <= 797.46 {
            println!("Found 797 solution!");
            println!("Distance: {}", dist);
            for (i, r) in eval.eval.routes.iter().enumerate() {
                println!("Route {}: {:?}", i+1, r);
            }
            break;
        }
    }
}
