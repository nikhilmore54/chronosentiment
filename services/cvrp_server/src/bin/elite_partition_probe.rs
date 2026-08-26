use coralys_moga::traits::{
    FitnessEvaluator, GenomeFactory, LocalSearchOperator, MutationOperator,
};
use cvrp::moga_impl::{CvrpEvaluator, CvrpLocalSearch, CvrpMutator};
use cvrp::{CvrpCandidate, CvrpGenomeFactory, CvrpInstance, RadiusPolicy};
use rand::SeedableRng;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::io::Write;

fn hash_routes_exact(routes: &[Vec<usize>]) -> u64 {
    let mut normalized = routes.to_vec();
    for r in &mut normalized {
        r.sort();
    }
    normalized.sort();
    let mut hasher = DefaultHasher::new();
    normalized.hash(&mut hasher);
    hasher.finish()
}

fn hash_partition(routes: &[Vec<usize>]) -> u64 {
    let mut partitioned_sets: Vec<Vec<usize>> = routes
        .iter()
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

fn partition_edit_distance(routes1: &[Vec<usize>], routes2: &[Vec<usize>]) -> usize {
    let mut r2_matched = vec![false; routes2.len()];
    let mut total_intersection = 0;

    // Greedily match routes to maximize intersection
    for r1 in routes1 {
        let mut best_match = None;
        let mut best_inter = 0;

        for (i, r2) in routes2.iter().enumerate() {
            if r2_matched[i] {
                continue;
            }
            let inter = r1.iter().filter(|x| r2.contains(x)).count();
            if inter > best_inter {
                best_inter = inter;
                best_match = Some(i);
            }
        }

        if let Some(i) = best_match {
            r2_matched[i] = true;
            total_intersection += best_inter;
        }
    }

    let total_customers = routes1.iter().map(|r| r.len()).sum::<usize>();
    total_customers - total_intersection
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let ls = CvrpLocalSearch {
        instance: instance.clone(),
    };

    println!("Searching for a 797 basin...");
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };
    let mut best_cand = factory.create(&mut rng);
    let mut best_dist = f64::MAX;
    let random_mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);

    for _ in 0..2000 {
        let mut child = best_cand.clone();
        random_mutator.mutate(&mut child, &mut rng);

        {
            let model = cvrp::moga_impl::CvrpConstraintModel {
                instance: instance.clone(),
            };
            let budget = coralys_core::operators::OperatorBudget {
                max_iterations: 1,
                max_time_ms: 1000,
            };
            coralys_core::operators::ImprovementOperator::improve(&ls, &mut child, &model, &budget)
                .unwrap();
        }

        let eval = evaluator.evaluate(
            &child,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        );
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
    let root_eval = evaluator.evaluate(
        &best_cand,
        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
    );

    // Decode routes using cust_idx, not customer.id
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

    let mut file = std::fs::File::create("m10d_partition_edges.csv").unwrap();
    writeln!(file, "source_partition_hash,target_partition_hash,partition_edit_distance,optimum_distance,is_elite").unwrap();

    let mut transitions_tested = 0;

    for route_idx in 0..root_routes.len() {
        for cust_idx in 0..root_routes[route_idx].len() {
            let customer = root_routes[route_idx][cust_idx];

            for target_route_idx in 0..root_routes.len() {
                if target_route_idx == route_idx {
                    continue;
                }

                for insert_idx in 0..=root_routes[target_route_idx].len() {
                    let mut modified_routes = root_routes.clone();
                    modified_routes[route_idx].remove(cust_idx);
                    modified_routes[target_route_idx].insert(insert_idx, customer);

                    // Flatten modified_routes into a new permutation
                    let mut new_perm = Vec::new();
                    for r in modified_routes {
                        new_perm.extend(r);
                    }

                    let mut test_cand = best_cand.clone();
                    test_cand.permutation = new_perm;

                    // Exhaustive local search

                    {
                        let model = cvrp::moga_impl::CvrpConstraintModel {
                            instance: instance.clone(),
                        };
                        let budget = coralys_core::operators::OperatorBudget {
                            max_iterations: 1,
                            max_time_ms: 1000,
                        };
                        coralys_core::operators::ImprovementOperator::improve(
                            &ls,
                            &mut test_cand,
                            &model,
                            &budget,
                        )
                        .unwrap();
                    }

                    let final_eval = evaluator.evaluate(
                        &test_cand,
                        &coralys_moga::runtime::optimization::metric::MetricReport::default(),
                    );

                    // Decode final_routes using cust_idx
                    let mut final_routes: Vec<Vec<usize>> = Vec::new();
                    let mut current_route: Vec<usize> = Vec::new();
                    let mut current_load = 0;
                    for &cust_idx in &test_cand.permutation {
                        let customer = &instance.customers[cust_idx];
                        if current_load + customer.demand > instance.capacity {
                            final_routes.push(current_route);
                            current_route = Vec::new();
                            current_load = 0;
                        }
                        current_route.push(cust_idx);
                        current_load += customer.demand;
                    }
                    if !current_route.is_empty() {
                        final_routes.push(current_route);
                    }

                    let target_partition_hash = hash_partition(&final_routes);
                    let edit_dist = partition_edit_distance(&root_routes, &final_routes);
                    let dist = final_eval.eval.total_distance;
                    let is_elite = dist <= 810.0;

                    writeln!(
                        file,
                        "{},{},{},{},{}",
                        source_partition_hash, target_partition_hash, edit_dist, dist, is_elite
                    )
                    .unwrap();

                    transitions_tested += 1;
                }
            }
        }
    }

    println!(
        "Partition Probe Finished. Evaluated {} transfers.",
        transitions_tested
    );
}
