use coralys_moga::traits::{
    CrossoverOperator, FitnessEvaluator, GenomeFactory, LocalSearchOperator, MutationOperator,
    RegionIdentifier,
};
use cvrp::moga_impl::{CvrpCrossover, CvrpEvaluator, CvrpLocalSearch, CvrpMutator};
use cvrp::{CvrpCandidate, CvrpGenomeFactory, CvrpInstance, RadiusPolicy};
use rand::Rng;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::Write;

fn decode_routes(cand: &CvrpCandidate, instance: &CvrpInstance) -> Vec<Vec<usize>> {
    let mut routes: Vec<Vec<usize>> = Vec::new();
    let mut current_route: Vec<usize> = Vec::new();
    let mut current_load = 0;
    for &cust_idx in &cand.permutation {
        let customer = &instance.customers[cust_idx];
        if current_load + customer.demand > instance.capacity {
            routes.push(current_route);
            current_route = Vec::new();
            current_load = 0;
        }
        current_route.push(cust_idx);
        current_load += customer.demand;
    }
    if !current_route.is_empty() {
        routes.push(current_route);
    }
    routes
}

fn extract_edges(routes: &Vec<Vec<usize>>) -> HashSet<(usize, usize)> {
    let mut edges = HashSet::new();
    for r in routes {
        let mut prev = usize::MAX; // Represents depot
        for &c in r {
            let e = if prev < c { (prev, c) } else { (c, prev) };
            edges.insert(e);
            prev = c;
        }
        let e = if prev < usize::MAX {
            (prev, usize::MAX)
        } else {
            (usize::MAX, prev)
        };
        edges.insert(e);
    }
    edges
}

pub struct CvrpRegionIdentifier {
    pub instance: CvrpInstance,
}

impl RegionIdentifier<CvrpCandidate> for CvrpRegionIdentifier {
    type RegionId = u64;

    fn region_of(&self, state: &CvrpCandidate) -> Self::RegionId {
        let routes = decode_routes(state, &self.instance);
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
}

fn flatten_routes(routes: Vec<Vec<usize>>) -> Vec<usize> {
    let mut perm = Vec::new();
    for r in routes {
        perm.extend(r);
    }
    perm
}

fn measure_edge_preservation(
    s1_routes: &Vec<Vec<usize>>,
    root_edges: &HashSet<(usize, usize)>,
) -> f64 {
    let s1_edges = extract_edges(s1_routes);
    let preserved = root_edges.intersection(&s1_edges).count();
    preserved as f64 / root_edges.len() as f64
}

fn measure_signature_overlap(s1_routes: &Vec<Vec<usize>>, root_routes: &Vec<Vec<usize>>) -> f64 {
    let mut r1_matched = vec![false; s1_routes.len()];
    let mut surviving_signatures = 0;

    for r0 in root_routes {
        let mut best_match = None;
        let mut best_inter = 0;
        for (i, r1) in s1_routes.iter().enumerate() {
            if r1_matched[i] {
                continue;
            }
            let inter = r0.iter().filter(|x| r1.contains(x)).count();
            if inter > best_inter {
                best_inter = inter;
                best_match = Some(i);
            }
        }
        if let Some(i) = best_match {
            r1_matched[i] = true;
            if best_inter as f64 >= 0.75 * (r0.len() as f64) {
                surviving_signatures += 1;
            }
        }
    }
    surviving_signatures as f64 / root_routes.len() as f64
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
    let ls = CvrpLocalSearch {
        instance: instance.clone(),
    };
    let crossover = CvrpCrossover;
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };
    let region_id = CvrpRegionIdentifier {
        instance: instance.clone(),
    };

    println!("Collecting Parents...");
    let mut elite_pool = Vec::new();
    let mut root_cand = factory.create(&mut rng);
    let mut root_dist = f64::MAX;

    let mut curr_cand = factory.create(&mut rng);
    let random_mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);

    for _ in 0..5000 {
        let mut child = curr_cand.clone();
        random_mutator.mutate(&mut child, &mut rng);
        let model = cvrp::moga_impl::CvrpConstraintModel {
            instance: instance.clone(),
        };
        let budget = coralys_core::operators::OperatorBudget {
            max_iterations: 1,
            max_time_ms: 1000,
        };
        coralys_core::operators::ImprovementOperator::improve(&ls, &mut child, &model, &budget)
            .unwrap();
        let eval = evaluator.evaluate(
            &child,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        );
        let d = eval.eval.total_distance;

        if d <= 810.0 {
            elite_pool.push(child.clone());
        }

        if d < root_dist {
            root_dist = d;
            root_cand = child.clone();
        }
        curr_cand = child;
    }

    let root_region = region_id.region_of(&root_cand);
    let root_routes = decode_routes(&root_cand, &instance);
    let root_edges = extract_edges(&root_routes);

    println!("Root Distance: {}", root_dist);

    let mut file = std::fs::File::create("m15_decoder_independence.csv").unwrap();
    writeln!(file, "source_type,s1_edge_preservation,s1_signature_overlap,s2_dist,returned_to_same_region,retained_elite,s2_region").unwrap();

    let mut evaluate_child = |s1: &mut CvrpCandidate, source_type: &str| {
        let s1_routes = decode_routes(s1, &instance);
        let edge_preservation = measure_edge_preservation(&s1_routes, &root_edges);
        let signature_overlap = measure_signature_overlap(&s1_routes, &root_routes);

        // Repair phase
        let mut s2 = s1.clone();
        let model = cvrp::moga_impl::CvrpConstraintModel {
            instance: instance.clone(),
        };
        let budget = coralys_core::operators::OperatorBudget {
            max_iterations: 1,
            max_time_ms: 1000,
        };
        coralys_core::operators::ImprovementOperator::improve(&ls, &mut s2, &model, &budget)
            .unwrap();
        let s2_eval = evaluator.evaluate(
            &s2,
            &coralys_moga::runtime::optimization::metric::MetricReport::default(),
        );
        let s2_dist = s2_eval.eval.total_distance;
        let s2_region = region_id.region_of(&s2);

        let returned = s2_region == root_region;
        let elite = s2_dist <= 810.0;

        writeln!(
            file,
            "{},{:.4},{:.4},{:.4},{},{},{}",
            source_type, edge_preservation, signature_overlap, s2_dist, returned, elite, s2_region
        )
        .unwrap();

        if s2_dist < root_dist {
            println!(
                "🚨 NEW BEST DISCOVERED: {:.4} by {} 🚨",
                s2_dist, source_type
            );
        }
    };

    let target_min = 0.50;
    let target_max = 0.75; // Deep in the Reconstruct Mode

    // Random Destruction Operator
    println!("Running Random Destruction...");
    let mut random_samples = 0;
    while random_samples < 2000 {
        let mut child = root_cand.clone();
        // apply completely random swaps to the permutation to degrade structure blindly
        let num_swaps = rng.gen_range(5..=25);
        for _ in 0..num_swaps {
            let i = rng.gen_range(0..child.permutation.len());
            let j = rng.gen_range(0..child.permutation.len());
            child.permutation.swap(i, j);
        }
        let routes = decode_routes(&child, &instance);
        let ep = measure_edge_preservation(&routes, &root_edges);
        if ep >= target_min && ep <= target_max {
            evaluate_child(&mut child, "Random");
            random_samples += 1;
        }
    }

    // Evolutionary Destruction Operator (L3: Segment Exchange)
    println!("Running L3 Evolutionary Destruction...");
    let mut l3_samples = 0;
    while l3_samples < 1000 {
        let mut routes = root_routes.clone();
        let valid: Vec<usize> = routes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.len() >= 2)
            .map(|(i, _)| i)
            .collect();
        if !valid.is_empty() {
            let r1 = *valid.choose(&mut rng).unwrap();
            let mut r2 = rng.gen_range(0..routes.len());
            while r2 == r1 {
                r2 = rng.gen_range(0..routes.len());
            }

            let n = routes[r1].len();
            let len = rng.gen_range(2..=std::cmp::min(4, n));
            let start = rng.gen_range(0..=(n - len));

            let segment: Vec<usize> = routes[r1].drain(start..start + len).collect();
            let j = if routes[r2].is_empty() {
                0
            } else {
                rng.gen_range(0..=routes[r2].len())
            };

            for (k, &c) in segment.iter().enumerate() {
                routes[r2].insert(j + k, c);
            }

            let mut child = root_cand.clone();
            child.permutation = flatten_routes(routes);
            let ep = measure_edge_preservation(&decode_routes(&child, &instance), &root_edges);

            if ep >= target_min && ep <= target_max {
                evaluate_child(&mut child, "Evolution_L3");
                l3_samples += 1;
            }
        }
    }

    // Evolutionary Destruction Operator (Recombination)
    println!("Running Recombination Evolutionary Destruction...");
    let mut recomb_samples = 0;
    while recomb_samples < 1000 {
        if let Some(p2) = elite_pool.choose(&mut rng) {
            let (mut child, _) = crossover.crossover(&root_cand, p2, &mut rng);
            let ep = measure_edge_preservation(&decode_routes(&child, &instance), &root_edges);
            if ep >= target_min && ep <= target_max {
                evaluate_child(&mut child, "Evolution_Crossover");
                recomb_samples += 1;
            }
        }
    }

    println!("M15 Decoder Independence Test Complete.");
}
