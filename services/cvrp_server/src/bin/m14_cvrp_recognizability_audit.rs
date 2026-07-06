use cvrp::{CvrpInstance, CvrpCandidate, RadiusPolicy, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpLocalSearch, CvrpCrossover};
use coralys_moga::traits::{FitnessEvaluator, MutationOperator, LocalSearchOperator, GenomeFactory, CrossoverOperator, RegionIdentifier};
use std::collections::{HashSet, HashMap};
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
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
        let e = if prev < usize::MAX { (prev, usize::MAX) } else { (usize::MAX, prev) };
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
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let ls = CvrpLocalSearch { instance: instance.clone() };
    let crossover = CvrpCrossover;
    let random_mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    let region_id = CvrpRegionIdentifier { instance: instance.clone() };
    
    println!("Collecting Parents & Elite Backbone...");
    let mut elite_pool = Vec::new();
    let mut good_pool = Vec::new();
    let mut root_cand = factory.create(&mut rng);
    let mut root_dist = f64::MAX;
    
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
    let mut num_elites = 0;

    let mut curr_cand = factory.create(&mut rng);
    for _ in 0..5000 {
        let mut child = curr_cand.clone();
        random_mutator.mutate(&mut child, &mut rng);
        ls.search(&mut child);
        let eval = evaluator.evaluate(&child);
        let d = eval.eval.total_distance;
        
        if d <= 810.0 {
            elite_pool.push(child.clone());
            num_elites += 1;
            let routes = decode_routes(&child, &instance);
            let edges = extract_edges(&routes);
            for e in edges {
                *edge_counts.entry(e).or_insert(0) += 1;
            }
        } else if d <= 850.0 {
            good_pool.push(child.clone());
        }
        
        if d < root_dist {
            root_dist = d;
            root_cand = child.clone();
        }
        curr_cand = child;
    }
    
    // Identify Backbone (top 50 edges that appear in at least 50% of elites)
    let mut backbone: Vec<((usize, usize), usize)> = edge_counts.into_iter()
        .filter(|&(_, c)| c > num_elites / 2)
        .collect();
    backbone.sort_by(|a, b| b.1.cmp(&a.1));
    let backbone_edges: HashSet<(usize, usize)> = backbone.into_iter().take(50).map(|(e, _)| e).collect();

    let root_region = region_id.region_of(&root_cand);
    let root_routes = decode_routes(&root_cand, &instance);
    let root_edges = extract_edges(&root_routes);
    
    println!("Root Distance: {}", root_dist);
    println!("Backbone Size: {}", backbone_edges.len());
    let num_samples = 500;

    let mut file = std::fs::File::create("m14_cvrp_recognizability_audit.csv").unwrap();
    writeln!(file, "transition_level,s1_capacity_violations,s1_route_count_diff,s1_edge_preservation,s1_backbone_preservation,s1_signature_overlap,s1_partition_dist,ar,s2_dist,returned_to_same_region,retained_elite").unwrap();

    let mut evaluate_child = |s1: &mut CvrpCandidate, level: &str| {
        let s1_routes = decode_routes(s1, &instance);
        let s1_edges = extract_edges(&s1_routes);
        
        // 1. Capacity Violations
        let mut capacity_violations = 0;
        for r in &s1_routes {
            let load: i32 = r.iter().map(|&c| instance.customers[c].demand).sum();
            if load > instance.capacity {
                capacity_violations += load - instance.capacity;
            }
        }

        // 2. Route Count Diff
        let route_count_diff = s1_routes.len() as i32 - root_routes.len() as i32;

        // 3. Edge Preservation
        let preserved_edges = root_edges.intersection(&s1_edges).count();
        let edge_preservation = preserved_edges as f64 / root_edges.len() as f64;

        // 4. Backbone Preservation
        let preserved_backbone = backbone_edges.intersection(&s1_edges).count();
        let backbone_preservation = if backbone_edges.is_empty() { 0.0 } else { preserved_backbone as f64 / backbone_edges.len() as f64 };

        // 5. Signature Overlap & Partition Dist
        let mut r1_matched = vec![false; s1_routes.len()];
        let mut total_intersection = 0;
        let mut surviving_signatures = 0;
        
        for r0 in &root_routes {
            let mut best_match = None;
            let mut best_inter = 0;
            for (i, r1) in s1_routes.iter().enumerate() {
                if r1_matched[i] { continue; }
                let inter = r0.iter().filter(|x| r1.contains(x)).count();
                if inter > best_inter {
                    best_inter = inter;
                    best_match = Some(i);
                }
            }
            if let Some(i) = best_match {
                r1_matched[i] = true;
                total_intersection += best_inter;
                
                // Signature survives if >= 75% of original customers are preserved in the matching route
                if best_inter as f64 >= 0.75 * (r0.len() as f64) {
                    surviving_signatures += 1;
                }
            }
        }
        
        let signature_overlap = surviving_signatures as f64 / root_routes.len() as f64;
        let total_customers = root_routes.iter().map(|r| r.len()).sum::<usize>();
        let partition_dist = (total_customers - total_intersection) as f64;
        let raw_magnitude = partition_dist;

        // Repair phase
        let mut s2 = s1.clone();
        ls.search(&mut s2);
        let s2_eval = evaluator.evaluate(&s2);
        
        // Final State (S2) Measurements
        let s2_routes = decode_routes(&s2, &instance);
        let mut r2_matched = vec![false; s2_routes.len()];
        let mut s2_intersection = 0;
        for r0 in &root_routes {
            let mut best_match = None;
            let mut best_inter = 0;
            for (i, r2) in s2_routes.iter().enumerate() {
                if r2_matched[i] { continue; }
                let inter = r0.iter().filter(|x| r2.contains(x)).count();
                if inter > best_inter {
                    best_inter = inter;
                    best_match = Some(i);
                }
            }
            if let Some(i) = best_match {
                r2_matched[i] = true;
                s2_intersection += best_inter;
            }
        }
        let residual_magnitude = (total_customers - s2_intersection) as f64;
        
        let ar = if raw_magnitude > 0.0 {
            (raw_magnitude - residual_magnitude) / raw_magnitude
        } else {
            0.0 // No damage
        };
        
        let s2_region = region_id.region_of(&s2);
        let returned = s2_region == root_region;
        let elite = s2_eval.eval.total_distance <= 810.0;
        
        writeln!(
            file, "{},{},{},{:.4},{:.4},{:.4},{},{:.4},{:.4},{},{}", 
            level, 
            capacity_violations,
            route_count_diff,
            edge_preservation,
            backbone_preservation,
            signature_overlap,
            partition_dist,
            ar,
            s2_eval.eval.total_distance,
            returned,
            elite
        ).unwrap();
    };

    println!("Running L0: Intra-Route Swap");
    for _ in 0..num_samples {
        let mut routes = root_routes.clone();
        let valid_routes: Vec<usize> = routes.iter().enumerate().filter(|(_, r)| r.len() >= 2).map(|(i, _)| i).collect();
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
            if routes.iter().all(|r| r.is_empty()) { break; }
        }
        if !routes[r1].is_empty() {
            let i = rng.gen_range(0..routes[r1].len());
            let c = routes[r1].remove(i);
            let j = if routes[r2].is_empty() { 0 } else { rng.gen_range(0..=routes[r2].len()) };
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
                if routes.iter().all(|r| r.is_empty()) { break; }
            }
            if !routes[r1].is_empty() {
                let i = rng.gen_range(0..routes[r1].len());
                let c = routes[r1].remove(i);
                let j = if routes[r2].is_empty() { 0 } else { rng.gen_range(0..=routes[r2].len()) };
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
        let valid: Vec<usize> = routes.iter().enumerate().filter(|(_, r)| r.len() >= 2).map(|(i, _)| i).collect();
        if !valid.is_empty() {
            let r1 = *valid.choose(&mut rng).unwrap();
            let mut r2 = rng.gen_range(0..routes.len());
            while r2 == r1 { r2 = rng.gen_range(0..routes.len()); }
            
            let n = routes[r1].len();
            let len = rng.gen_range(2..=std::cmp::min(4, n));
            let start = rng.gen_range(0..=(n - len));
            
            let segment: Vec<usize> = routes[r1].drain(start..start+len).collect();
            let j = if routes[r2].is_empty() { 0 } else { rng.gen_range(0..=routes[r2].len()) };
            
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
    
    println!("M14 Audit Complete.");
}
