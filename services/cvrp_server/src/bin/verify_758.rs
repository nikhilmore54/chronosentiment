use cvrp::{CvrpInstance, CvrpCandidate};
use cvrp::moga_impl::{CvrpEvaluator, CvrpLocalSearch};
use coralys_moga::traits::{FitnessEvaluator, LocalSearchOperator};
use rand::SeedableRng;
use rand::seq::SliceRandom;

fn random_ls(instance: &CvrpInstance, ls: &CvrpLocalSearch, rng: &mut rand::rngs::StdRng) -> CvrpCandidate {
    let mut cand = CvrpCandidate { permutation: (1..instance.customers.len()).collect(), last_mutation_op: None, last_mutation_radius: None, route_boundary_changes: None };
    cand.permutation.shuffle(rng);
    let model = cvrp::moga_impl::CvrpConstraintModel { instance: instance.clone() };
    let budget = coralys_core::operators::OperatorBudget { max_iterations: 1, max_time_ms: 1000 };
    coralys_core::operators::ImprovementOperator::improve(ls, &mut cand, &model, &budget).unwrap();
    cand
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let ls = CvrpLocalSearch { instance: instance.clone() };
    
    println!("Hunting for < 797 solution...");
    let mut min_dist = f64::INFINITY;
    
    for i in 0..1000 {
        let cand = random_ls(&instance, &ls, &mut rng);
        let eval = evaluator.evaluate(&cand, &coralys_moga::runtime::optimization::metric::MetricReport::default());
        
        if eval.eval.total_distance < min_dist {
            min_dist = eval.eval.total_distance;
            println!("Iter {}: New Best = {:.2} | Vehicles = {}", i, min_dist, eval.eval.num_vehicles);
        }
        
        if eval.eval.total_distance < 797.0 {
            println!("================================");
            println!("FOUND ELITE BREAKTHROUGH!");
            println!("Distance: {}", eval.eval.total_distance);
            println!("Vehicles: {}", eval.eval.num_vehicles);
            println!("Routes:");
            for (idx, r) in eval.eval.routes.iter().enumerate() {
                let load: i32 = r.iter().map(|&c| instance.customers[c].demand).sum();
                println!("  Route {}: {:?} (Load: {} / {})", idx+1, r, load, instance.capacity);
            }
            return;
        }
    }
}
