use cvrp::{CvrpInstance, CvrpCandidate, RadiusPolicy, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpLocalSearch};
use coralys_moga::traits::{FitnessEvaluator, MutationOperator, LocalSearchOperator, GenomeFactory};
use std::collections::{HashSet, VecDeque};
use rand::SeedableRng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io::Write;

fn hash_partition(routes: &[Vec<usize>]) -> u64 {
    let mut partitioned_sets: Vec<Vec<usize>> = routes.iter()
        .map(|r| {
            let mut sorted = r.clone();
            sorted.sort();
            sorted
        })
        .collect();
    partitioned_sets.sort();
    let mut hasher = DefaultHasher::new();
    partitioned_sets.hash(&mut hasher);
    hasher.finish()
}

// Computes the optimal distance for a single fixed route
fn optimize_route(route: &mut Vec<usize>, instance: &CvrpInstance) -> f64 {
    if route.is_empty() { return 0.0; }
    
    let calc_dist = |r: &[usize]| -> f64 {
        let mut d = 0.0;
        let mut curr = &instance.depot;
        for &c in r {
            let cust = &instance.customers[c];
            d += instance.distance(curr, cust);
            curr = cust;
        }
        d += instance.distance(curr, &instance.depot);
        d
    };

    let mut current_best = calc_dist(route);
    let mut improving = true;
    let n = route.len();

    while improving {
        improving = false;
        
        // 2-opt
        for i in 0..n {
            for j in (i+1)..n {
                let mut test_r = route.clone();
                test_r[i..=j].reverse();
                let dist = calc_dist(&test_r);
                if dist < current_best - 1e-6 {
                    current_best = dist;
                    *route = test_r;
                    improving = true;
                }
            }
        }
        
        // Relocate
        for i in 0..n {
            for j in 0..n {
                if i == j { continue; }
                let mut test_r = route.clone();
                let val = test_r.remove(i);
                let insert_pos = if j > i { j - 1 } else { j };
                test_r.insert(insert_pos, val);
                let dist = calc_dist(&test_r);
                if dist < current_best - 1e-6 {
                    current_best = dist;
                    *route = test_r;
                    improving = true;
                }
            }
        }
    }
    
    current_best
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
    let random_mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    
    for _ in 0..2000 {
        let mut child = best_cand.clone();
        random_mutator.mutate(&mut child, &mut rng);
        let model = cvrp::moga_impl::CvrpConstraintModel { instance: instance.clone() };
        let budget = coralys_core::operators::OperatorBudget { max_iterations: 1, max_time_ms: 1000 };
        coralys_core::operators::ImprovementOperator::improve(&ls, &mut child, &model, &budget).unwrap();
        let eval = evaluator.evaluate(&child, &coralys_moga::runtime::optimization::metric::MetricReport::default());
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
    let root_eval = evaluator.evaluate(&best_cand, &coralys_moga::runtime::optimization::metric::MetricReport::default());
    
    // Decode routes using cust_idx
    let mut root_routes: Vec<Vec<usize>> = Vec::new();
    let mut current_route: Vec<usize> = Vec::new();
    let mut current_load = 0;
    for &cust_idx in &best_cand.permutation {
        let customer = &instance.customers[cust_idx];
        if current_load + customer.demand > instance.capacity {
            root_routes.push(current_route);
            current_route = Vec::new();
            current_load = 0;
        }
        current_route.push(cust_idx);
        current_load += customer.demand;
    }
    if !current_route.is_empty() {
        root_routes.push(current_route);
    }
    
    let source_partition_hash = hash_partition(&root_routes);

    let mut file = std::fs::File::create("m10d2_frozen_edges.csv").unwrap();
    writeln!(file, "source_partition_hash,target_partition_hash,partition_edit_distance,optimum_distance,is_elite,is_valid").unwrap();

    let mut transitions_tested = 0;
    let mut invalid_capacity = 0;

    for route_idx in 0..root_routes.len() {
        for cust_idx in 0..root_routes[route_idx].len() {
            let customer_idx = root_routes[route_idx][cust_idx];
            let customer_demand = instance.customers[customer_idx].demand;
            
            for target_route_idx in 0..root_routes.len() {
                if target_route_idx == route_idx { continue; }
                
                // Check feasibility FIRST
                let target_load: i32 = root_routes[target_route_idx].iter().map(|&c| instance.customers[c].demand).sum();
                if target_load + customer_demand > instance.capacity {
                    invalid_capacity += 1;
                    continue; // Skip invalid transfers
                }
                
                // For frozen partition, insertion index doesn't matter for the partition hash
                // and optimize_route will find the optimal order anyway.
                // We just append it and optimize.
                let mut modified_routes = root_routes.clone();
                modified_routes[route_idx].remove(cust_idx);
                modified_routes[target_route_idx].push(customer_idx);
                
                // Optimize only the two modified routes (the others are already optimal)
                let mut total_distance = 0.0;
                for (i, r) in modified_routes.iter_mut().enumerate() {
                    if i == route_idx || i == target_route_idx {
                        total_distance += optimize_route(r, &instance);
                    } else {
                        // For unmodified routes, we still need their optimal distance
                        total_distance += optimize_route(r, &instance);
                    }
                }
                
                let target_partition_hash = hash_partition(&modified_routes);
                let edit_dist = 1; // Always exactly 1 customer shifted
                let is_elite = total_distance <= 810.0;
                
                writeln!(file, "{},{},{},{},{},true", 
                    source_partition_hash, target_partition_hash, edit_dist, total_distance, is_elite
                ).unwrap();
                
                transitions_tested += 1;
            }
        }
    }
    
    println!("Frozen Probe Finished. Evaluated {} valid transfers. Skipped {} invalid capacity.", transitions_tested, invalid_capacity);
}
