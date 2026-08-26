use coralys_moga::observatory::{ReachabilityObservation, ReachabilityProbe};
use coralys_moga::traits::{
    CrossoverOperator, FitnessEvaluator, GenomeFactory, LocalSearchOperator, MutationOperator,
    ObservedTransitionMetric, RegionIdentifier,
};
use cvrp::moga_impl::{CvrpCrossover, CvrpEvaluator, CvrpLocalSearch, CvrpMutator};
use cvrp::{CvrpCandidate, CvrpGenomeFactory, CvrpInstance, RadiusPolicy};
use rand::Rng;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;

pub struct CvrpPartitionMetric {
    pub instance: CvrpInstance,
}

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

impl ObservedTransitionMetric<CvrpCandidate> for CvrpPartitionMetric {
    fn magnitude(&self, source: &CvrpCandidate, result_after_repair: &CvrpCandidate) -> f64 {
        let routes1 = decode_routes(source, &self.instance);
        let routes2 = decode_routes(result_after_repair, &self.instance);

        let mut r2_matched = vec![false; routes2.len()];
        let mut total_intersection = 0;

        for r1 in &routes1 {
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
        (total_customers - total_intersection) as f64
    }
}

pub struct CvrpEdgeMetric {
    pub instance: CvrpInstance,
}

impl ObservedTransitionMetric<CvrpCandidate> for CvrpEdgeMetric {
    fn magnitude(&self, source: &CvrpCandidate, result_after_repair: &CvrpCandidate) -> f64 {
        let routes1 = decode_routes(source, &self.instance);
        let routes2 = decode_routes(result_after_repair, &self.instance);

        let mut edges1 = HashSet::new();
        for r in &routes1 {
            let mut prev = 0; // Depot
            for &c in r {
                let e = if prev < c { (prev, c) } else { (c, prev) };
                edges1.insert(e);
                prev = c;
            }
            let e = if prev < 0 { (prev, 0) } else { (0, prev) }; // Depot is 0 conceptually, but we use customers
            // Actually, instance customers are 0 to N-1. Depot is separate.
            // Let's just use raw consecutive indices for edges.
        }

        // Let's refine edge extraction:
        let get_edges = |routes: &Vec<Vec<usize>>| -> HashSet<(usize, usize)> {
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
        };

        let e1 = get_edges(&routes1);
        let e2 = get_edges(&routes2);

        // Edge magnitude is number of edges in e2 NOT in e1
        e2.difference(&e1).count() as f64
    }
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

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator {
        instance: instance.clone(),
    };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let ls = CvrpLocalSearch {
        instance: instance.clone(),
    };
    let crossover = CvrpCrossover;
    let random_mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    let factory = CvrpGenomeFactory {
        num_customers: instance.customers.len(),
    };

    let partition_metric = CvrpPartitionMetric {
        instance: instance.clone(),
    };
    let edge_metric = CvrpEdgeMetric {
        instance: instance.clone(),
    };
    let region_id = CvrpRegionIdentifier {
        instance: instance.clone(),
    };
    let probe_ls = |cand: &mut cvrp::CvrpCandidate| {
        let model = cvrp::moga_impl::CvrpConstraintModel {
            instance: instance.clone(),
        };
        let budget = coralys_core::operators::OperatorBudget {
            max_iterations: 1,
            max_time_ms: 1000,
        };
        coralys_core::operators::ImprovementOperator::improve(&ls, cand, &model, &budget).unwrap();
    };
    let probe = coralys_moga::observatory::ReachabilityProbe::new(
        &evaluator,
        probe_ls,
        &partition_metric,
        &region_id,
        810.0,
    );

    println!("Collecting Parents...");
    let mut elite_pool = Vec::new();
    let mut good_pool = Vec::new();
    let mut root_cand = factory.create(&mut rng);
    let mut root_dist = f64::MAX;

    let mut curr_cand = factory.create(&mut rng);
    for _ in 0..5000 {
        let mut child = curr_cand.clone();
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
        let d = eval.eval.total_distance;

        if d <= 810.0 {
            elite_pool.push(child.clone());
        } else if d <= 850.0 {
            good_pool.push(child.clone());
        }

        if d < root_dist {
            root_dist = d;
            root_cand = child.clone();
        }
        curr_cand = child;
    }

    let root_region = region_id.region_of(&root_cand);
    let root_fitness = 100000.0 - root_dist;
    println!("Root Distance: {}", root_dist);
    let root_routes = decode_routes(&root_cand, &instance);
    let num_samples = 500;

    let mut file = std::fs::File::create("m12_cvrp_repair_atlas.csv").unwrap();
    writeln!(file, "transition_level,raw_magnitude,residual_magnitude,repair_delta,s1_dist,s2_dist,s1_returned_to_same_region,returned_to_same_region,retained_elite").unwrap();

    let mut evaluate_child = |child: &mut CvrpCandidate, level: &str| {
        let obs = probe.evaluate_transition(&root_cand, child, root_fitness, &root_region);

        let s1_dist = 100000.0 - obs.s1_fitness;
        let s2_dist = 100000.0 - obs.s2_fitness;

        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{}",
            level,
            obs.raw_magnitude,
            obs.residual_magnitude,
            obs.repair_delta,
            s1_dist,
            s2_dist,
            obs.s1_returned_to_same_region,
            obs.returned_to_same_region,
            obs.retained_elite
        )
        .unwrap();
    };

    println!("Running L0: Intra-Route Swap");
    for _ in 0..num_samples {
        let mut routes = root_routes.clone();
        let valid_routes: Vec<usize> = routes
            .iter()
            .enumerate()
            .filter(|(_, r)| r.len() >= 2)
            .map(|(i, _)| i)
            .collect();
        if !valid_routes.is_empty() {
            let r_idx = *valid_routes.choose(&mut rng).unwrap();
            let n = routes[r_idx].len();
            let i = rng.gen_range(0..n);
            let j = rng.gen_range(0..n);
            routes[r_idx].swap(i, j);
            let mut child = root_cand.clone();
            child.permutation = flatten_routes(routes);
            evaluate_child(&mut child, "L0");
        }
    }

    println!("Running L1: 1 Customer Transfer");
    for _ in 0..num_samples {
        let mut routes = root_routes.clone();
        let r1 = rng.gen_range(0..routes.len());
        let mut r2 = rng.gen_range(0..routes.len());
        while r2 == r1 || routes[r1].is_empty() {
            r2 = rng.gen_range(0..routes.len());
            if routes.iter().all(|r| r.is_empty()) {
                break;
            }
        }
        if !routes[r1].is_empty() {
            let i = rng.gen_range(0..routes[r1].len());
            let c = routes[r1].remove(i);
            let j = if routes[r2].is_empty() {
                0
            } else {
                rng.gen_range(0..=routes[r2].len())
            };
            routes[r2].insert(j, c);
            let mut child = root_cand.clone();
            child.permutation = flatten_routes(routes);
            evaluate_child(&mut child, "L1");
        }
    }

    println!("Running L2: 2 Customer Transfer");
    for _ in 0..num_samples {
        let mut routes = root_routes.clone();
        for _ in 0..2 {
            let r1 = rng.gen_range(0..routes.len());
            let mut r2 = rng.gen_range(0..routes.len());
            while r2 == r1 || routes[r1].is_empty() {
                r2 = rng.gen_range(0..routes.len());
                if routes.iter().all(|r| r.is_empty()) {
                    break;
                }
            }
            if !routes[r1].is_empty() {
                let i = rng.gen_range(0..routes[r1].len());
                let c = routes[r1].remove(i);
                let j = if routes[r2].is_empty() {
                    0
                } else {
                    rng.gen_range(0..=routes[r2].len())
                };
                routes[r2].insert(j, c);
            }
        }
        let mut child = root_cand.clone();
        child.permutation = flatten_routes(routes);
        evaluate_child(&mut child, "L2");
    }

    println!("Running L3: Segment Exchange");
    for _ in 0..num_samples {
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
            evaluate_child(&mut child, "L3");
        }
    }

    println!("Running L4A: Elite Recombination");
    for _ in 0..num_samples {
        if let Some(p2) = elite_pool.choose(&mut rng) {
            let (mut c1, _) = crossover.crossover(&root_cand, p2, &mut rng);
            evaluate_child(&mut c1, "L4A");
        }
    }

    println!("Running L4B: Good Recombination");
    for _ in 0..num_samples {
        if let Some(p2) = good_pool.choose(&mut rng) {
            let (mut c1, _) = crossover.crossover(&root_cand, p2, &mut rng);
            evaluate_child(&mut c1, "L4B");
        }
    }

    println!("Running L4C: Random Recombination");
    for _ in 0..num_samples {
        let p2 = factory.create(&mut rng);
        let (mut c1, _) = crossover.crossover(&root_cand, &p2, &mut rng);
        evaluate_child(&mut c1, "L4C");
    }

    println!("Atlas Construction Complete.");
}
