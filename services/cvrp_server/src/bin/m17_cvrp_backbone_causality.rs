use cvrp::{CvrpInstance, CvrpCandidate, RadiusPolicy, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpEvaluator, CvrpMutator, CvrpLocalSearch};
use coralys_moga::traits::{FitnessEvaluator, MutationOperator, LocalSearchOperator, GenomeFactory, RegionIdentifier};
use std::collections::{HashSet, HashMap};
use rand::SeedableRng;
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

fn measure_edge_preservation(s1_routes: &Vec<Vec<usize>>, root_edges: &HashSet<(usize, usize)>) -> f64 {
    let s1_edges = extract_edges(s1_routes);
    let preserved = root_edges.intersection(&s1_edges).count();
    preserved as f64 / root_edges.len() as f64
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mut rng = rand::rngs::StdRng::seed_from_u64(1337);
    let ls = CvrpLocalSearch { instance: instance.clone() };
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    let region_id = CvrpRegionIdentifier { instance: instance.clone() };
    
    println!("Collecting Parents & Elite Backbone...");
    let mut root_cand = factory.create(&mut rng);
    let mut root_dist = f64::MAX;
    
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
    let mut num_elites = 0;

    let mut curr_cand = factory.create(&mut rng);
    let random_mutator = CvrpMutator::new(instance.clone(), RadiusPolicy::Control);

    for _ in 0..5000 {
        let mut child = curr_cand.clone();
        random_mutator.mutate(&mut child, &mut rng);
        
        {
            let model = cvrp::moga_impl::CvrpConstraintModel { instance: instance.clone() };
            let budget = coralys_core::operators::OperatorBudget { max_iterations: 1, max_time_ms: 1000 };
            coralys_core::operators::ImprovementOperator::improve(&ls, &mut child, &model, &budget).unwrap();
        }
        
        let eval = evaluator.evaluate(&child, &coralys_moga::runtime::optimization::metric::MetricReport::default());
        let d = eval.eval.total_distance;
        
        if d <= 810.0 {
            num_elites += 1;
            let routes = decode_routes(&child, &instance);
            let edges = extract_edges(&routes);
            for e in edges {
                *edge_counts.entry(e).or_insert(0) += 1;
            }
        }
        
        if d < root_dist {
            root_dist = d;
            root_cand = child.clone();
        }
        curr_cand = child;
    }
    // Extract root edges sorted by their frequency in the elite pool
    let root_region = region_id.region_of(&root_cand);
    let root_routes = decode_routes(&root_cand, &instance);
    let root_edges = extract_edges(&root_routes);

    let mut root_edge_list: Vec<((usize, usize), usize)> = root_edges.iter()
        .map(|&e| (e, *edge_counts.get(&e).unwrap_or(&0)))
        .collect();
    root_edge_list.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Total edges in root is 37. We take the top 15 as Backbone, next 15 as Placebo.
    let backbone_edges: HashSet<(usize, usize)> = root_edge_list.iter().take(15).map(|(e, _)| *e).collect();
    let placebo_edges: HashSet<(usize, usize)> = root_edge_list.iter().skip(15).take(15).map(|(e, _)| *e).collect();
    
    println!("Root Distance: {}", root_dist);
    println!("Backbone Size: {}", backbone_edges.len());
    println!("Placebo Size: {}", placebo_edges.len());

    let mut file = std::fs::File::create("m17_backbone_causality.csv").unwrap();
    writeln!(file, "source_type,s1_edge_preservation,backbone_broken,placebo_broken,s2_dist,target_basin_hash,elite_reconstruction").unwrap();

    let mut evaluate_child = |s1: &mut CvrpCandidate, source_type: &str, bb_broken: usize, pb_broken: usize| {
        let s1_routes = decode_routes(s1, &instance);
        let edge_preservation = measure_edge_preservation(&s1_routes, &root_edges);
        
        // Repair phase
        let mut s2 = s1.clone();
        
        {
            let model = cvrp::moga_impl::CvrpConstraintModel { instance: instance.clone() };
            let budget = coralys_core::operators::OperatorBudget { max_iterations: 1, max_time_ms: 1000 };
            coralys_core::operators::ImprovementOperator::improve(&ls, &mut s2, &model, &budget).unwrap();
        }
        
        let s2_eval = evaluator.evaluate(&s2, &coralys_moga::runtime::optimization::metric::MetricReport::default());
        let s2_dist = s2_eval.eval.total_distance;
        let s2_region = region_id.region_of(&s2);
        
        let elite = s2_dist <= 810.0;
        
        writeln!(
            file, "{},{:.4},{},{},{:.4},{},{}", 
            source_type, 
            edge_preservation,
            bb_broken,
            pb_broken,
            s2_dist,
            s2_region,
            if elite { 1 } else { 0 }
        ).unwrap();
    };

    let target_min_ep = 0.50;
    let target_max_ep = 0.75; // Reconstruct Mode
    let target_broken = 8;   // Exact destruction budget out of 15

    let apply_swaps = |routes: &mut Vec<Vec<usize>>, num_swaps: usize, target_edges: &HashSet<(usize, usize)>, protect_bb: bool, protect_pb: bool, rng: &mut rand::rngs::StdRng| {
        for _ in 0..num_swaps {
            let r1 = rng.gen_range(0..routes.len());
            if routes[r1].is_empty() { continue; }
            let mut r2 = rng.gen_range(0..routes.len());
            while r1 == r2 { r2 = rng.gen_range(0..routes.len()); }
            
            let i = rng.gen_range(0..routes[r1].len());
            
            let prev = if i == 0 { usize::MAX } else { routes[r1][i-1] };
            let curr = routes[r1][i];
            let next = if i == routes[r1].len() - 1 { usize::MAX } else { routes[r1][i+1] };
            
            let e1 = if prev < curr { (prev, curr) } else { (curr, prev) };
            let e2 = if curr < next { (curr, next) } else { (next, curr) };
            
            if protect_bb && (backbone_edges.contains(&e1) || backbone_edges.contains(&e2)) {
                continue;
            }
            if protect_pb && (placebo_edges.contains(&e1) || placebo_edges.contains(&e2)) {
                continue;
            }
            
            let is_target = target_edges.contains(&e1) || target_edges.contains(&e2);
            if is_target || rng.gen_bool(0.2) {
                let j = if routes[r2].is_empty() { 0 } else { rng.gen_range(0..=routes[r2].len()) };
                
                let r2_prev = if j == 0 { usize::MAX } else { routes[r2][j-1] };
                let r2_next = if j == routes[r2].len() { usize::MAX } else { routes[r2][j] };
                if r2_prev != usize::MAX && r2_next != usize::MAX {
                    let e_dest = if r2_prev < r2_next { (r2_prev, r2_next) } else { (r2_next, r2_prev) };
                    if protect_bb && backbone_edges.contains(&e_dest) { continue; }
                    if protect_pb && placebo_edges.contains(&e_dest) { continue; }
                }

                let c = routes[r1].remove(i);
                routes[r2].insert(j, c);
            }
        }
    };

    println!("Running Backbone Preserver...");
    let mut preserver_samples = 0;
    while preserver_samples < 500 {
        let mut routes = root_routes.clone();
        let num_swaps = rng.gen_range(15..=40);
        let empty_set = HashSet::new();
        apply_swaps(&mut routes, num_swaps, &empty_set, true, false, &mut rng);
        
        let mut child = root_cand.clone();
        child.permutation = flatten_routes(routes);
        let s1_routes = decode_routes(&child, &instance);
        let s1_edges = extract_edges(&s1_routes);
        let ep = measure_edge_preservation(&s1_routes, &root_edges);
        
        let bb_broken = backbone_edges.len() - backbone_edges.intersection(&s1_edges).count();
        let pb_broken = placebo_edges.len() - placebo_edges.intersection(&s1_edges).count();
        
        if ep >= target_min_ep && ep <= target_max_ep && bb_broken == 0 {
            evaluate_child(&mut child, "BackbonePreserver", bb_broken, pb_broken);
            preserver_samples += 1;
        }
    }

    println!("Running Backbone Destroyer...");
    let mut destroyer_samples = 0;
    while destroyer_samples < 500 {
        let mut routes = root_routes.clone();
        let num_swaps = rng.gen_range(15..=40);
        apply_swaps(&mut routes, num_swaps, &backbone_edges, false, true, &mut rng);
        
        let mut child = root_cand.clone();
        child.permutation = flatten_routes(routes);
        let s1_routes = decode_routes(&child, &instance);
        let s1_edges = extract_edges(&s1_routes);
        let ep = measure_edge_preservation(&s1_routes, &root_edges);
        
        let bb_broken = backbone_edges.len() - backbone_edges.intersection(&s1_edges).count();
        let pb_broken = placebo_edges.len() - placebo_edges.intersection(&s1_edges).count();
        
        if ep >= target_min_ep && ep <= target_max_ep && bb_broken >= target_broken && pb_broken == 0 {
            evaluate_child(&mut child, "BackboneDestroyer", bb_broken, pb_broken);
            destroyer_samples += 1;
        }
    }

    println!("Running Placebo (Random Elite Edge Destroyer)...");
    let mut placebo_samples = 0;
    while placebo_samples < 500 {
        let mut routes = root_routes.clone();
        let num_swaps = rng.gen_range(15..=40);
        apply_swaps(&mut routes, num_swaps, &placebo_edges, true, false, &mut rng);
        
        let mut child = root_cand.clone();
        child.permutation = flatten_routes(routes);
        let s1_routes = decode_routes(&child, &instance);
        let s1_edges = extract_edges(&s1_routes);
        let ep = measure_edge_preservation(&s1_routes, &root_edges);
        
        let bb_broken = backbone_edges.len() - backbone_edges.intersection(&s1_edges).count();
        let pb_broken = placebo_edges.len() - placebo_edges.intersection(&s1_edges).count();
        
        if ep >= target_min_ep && ep <= target_max_ep && pb_broken >= target_broken && bb_broken == 0 {
            evaluate_child(&mut child, "PlaceboDestroyer", bb_broken, pb_broken);
            placebo_samples += 1;
        }
    }

    println!("M17 Backbone Causality Test Complete.");
}
