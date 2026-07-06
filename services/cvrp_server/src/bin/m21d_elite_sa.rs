use cvrp::CvrpInstance;
use std::collections::HashSet;
use rand::Rng;

fn get_node_demand(instance: &CvrpInstance, id: usize) -> i32 {
    if id == 1 { 0 } else { instance.customers.iter().find(|c| c.id == id).unwrap().demand }
}

fn get_node_by_id<'a>(instance: &'a CvrpInstance, id: usize) -> &'a cvrp::Node {
    if id == 1 { &instance.depot } else { instance.customers.iter().find(|c| c.id == id).unwrap() }
}

fn calc_route_distance(instance: &CvrpInstance, route: &Vec<usize>) -> f64 {
    if route.is_empty() { return 0.0; }
    let mut dist = 0.0;
    let depot = &instance.depot;
    let first = get_node_by_id(instance, route[0]);
    dist += instance.distance(depot, first);
    for i in 0..(route.len() - 1) {
        let a = get_node_by_id(instance, route[i]);
        let b = get_node_by_id(instance, route[i+1]);
        dist += instance.distance(a, b);
    }
    let last = get_node_by_id(instance, *route.last().unwrap());
    dist += instance.distance(last, depot);
    dist
}

fn calc_total_dist(instance: &CvrpInstance, routes: &Vec<Vec<usize>>) -> f64 {
    routes.iter().map(|r| calc_route_distance(instance, r)).sum()
}

fn get_canonical_edges(routes: &Vec<Vec<usize>>) -> HashSet<(usize, usize)> {
    let mut edges = HashSet::new();
    let depot_id = 1;
    for r in routes {
        if r.is_empty() { continue; }
        let mut prev = depot_id;
        for &node in r {
            edges.insert((prev.min(node), prev.max(node)));
            prev = node;
        }
        edges.insert((prev.min(depot_id), prev.max(depot_id)));
    }
    edges
}

// Randomly generate ONE valid neighbor
fn random_neighbor(instance: &CvrpInstance, routes: &Vec<Vec<usize>>, rng: &mut impl Rng) -> Option<Vec<Vec<usize>>> {
    let num_routes = routes.len();
    if num_routes == 0 { return None; }

    for _ in 0..100 { // Try to find a valid move
        let op = rng.gen_range(0..4);
        let mut new_routes = routes.clone();
        
        match op {
            0 => { // Relocate
                let r1 = rng.gen_range(0..num_routes);
                if new_routes[r1].is_empty() { continue; }
                let i = rng.gen_range(0..new_routes[r1].len());
                let node = new_routes[r1].remove(i);
                
                let r2 = rng.gen_range(0..num_routes);
                let j = rng.gen_range(0..=new_routes[r2].len());
                new_routes[r2].insert(j, node);
            },
            1 => { // Exchange
                let r1 = rng.gen_range(0..num_routes);
                let r2 = rng.gen_range(0..num_routes);
                if new_routes[r1].is_empty() || new_routes[r2].is_empty() { continue; }
                let i = rng.gen_range(0..new_routes[r1].len());
                let j = rng.gen_range(0..new_routes[r2].len());
                let temp = new_routes[r1][i];
                new_routes[r1][i] = new_routes[r2][j];
                new_routes[r2][j] = temp;
            },
            2 => { // 2-opt
                let r1 = rng.gen_range(0..num_routes);
                let len = new_routes[r1].len();
                if len < 2 { continue; }
                let i = rng.gen_range(0..len - 1);
                let j = rng.gen_range(i + 1..len);
                new_routes[r1][i..=j].reverse();
            },
            3 => { // 2-opt*
                let r1 = rng.gen_range(0..num_routes);
                let mut r2 = rng.gen_range(0..num_routes);
                while r1 == r2 { r2 = rng.gen_range(0..num_routes); }
                let i = rng.gen_range(0..=new_routes[r1].len());
                let j = rng.gen_range(0..=new_routes[r2].len());
                let tail1: Vec<usize> = new_routes[r1].drain(i..).collect();
                let tail2: Vec<usize> = new_routes[r2].drain(j..).collect();
                new_routes[r1].extend(tail2);
                new_routes[r2].extend(tail1);
            },
            _ => unreachable!(),
        }

        new_routes.retain(|r| !r.is_empty());
        
        let mut valid = true;
        for r in &new_routes {
            let mut load = 0;
            for &n in r { load += get_node_demand(instance, n); }
            if load > instance.capacity { valid = false; break; }
        }
        if valid { return Some(new_routes); }
    }
    None
}

struct SearchStats {
    best_dist: f64,
    best_overlap: usize,
    max_dist_seen: f64,
    fp_797: bool,
    fp_795: bool,
    fp_792: bool,
    fp_790: bool,
    fp_788: bool,
}

fn run_search(
    instance: &CvrpInstance, 
    start_routes: &Vec<Vec<usize>>, 
    bks_edges: &HashSet<(usize, usize)>,
    t_start: f64, 
    ceiling: f64,
    rng: &mut impl Rng
) -> SearchStats {
    let mut current_routes = start_routes.clone();
    let mut current_dist = calc_total_dist(instance, &current_routes);
    
    let mut best_dist = current_dist;
    let mut best_overlap = get_canonical_edges(&current_routes).intersection(bks_edges).count();
    let mut max_dist_seen = current_dist;
    
    let mut stats = SearchStats {
        best_dist, best_overlap, max_dist_seen,
        fp_797: false, fp_795: false, fp_792: false, fp_790: false, fp_788: false,
    };
    
    let iterations = 1_000_000;
    let alpha = (0.01 / t_start).powf(1.0 / iterations as f64);
    let mut t = t_start;

    for _ in 0..iterations {
        if let Some(new_routes) = random_neighbor(instance, &current_routes, rng) {
            let new_dist = calc_total_dist(instance, &new_routes);
            
            if new_dist > ceiling {
                continue; // HARD CEILING
            }
            
            let delta = new_dist - current_dist;
            
            let mut accept = false;
            if delta < 0.0 {
                accept = true;
            } else {
                let p = (-delta / t).exp();
                if rng.gen_range(0.0..1.0) < p { accept = true; }
            }

            if accept {
                current_routes = new_routes;
                current_dist = new_dist;
                
                max_dist_seen = max_dist_seen.max(current_dist);
                
                if current_dist < best_dist {
                    best_dist = current_dist;
                    if best_dist < 797.0 { stats.fp_797 = true; }
                    if best_dist < 795.0 { stats.fp_795 = true; }
                    if best_dist < 792.0 { stats.fp_792 = true; }
                    if best_dist < 790.0 { stats.fp_790 = true; }
                    if best_dist < 788.0 { stats.fp_788 = true; }
                }
                
                let overlap = get_canonical_edges(&current_routes).intersection(bks_edges).count();
                best_overlap = best_overlap.max(overlap);
            }
        }
        t *= alpha;
    }
    
    stats.best_dist = best_dist;
    stats.best_overlap = best_overlap;
    stats.max_dist_seen = max_dist_seen;
    stats
}

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    
    let r_797 = vec![
        vec![15, 19, 9, 12, 5, 29, 24, 3, 4],
        vec![7, 18, 20, 32, 22, 14, 27],
        vec![21, 6, 26, 11, 16, 10, 23, 30],
        vec![13, 2, 8, 17, 31],
        vec![28, 25]
    ];
    
    let r_bks = vec![
        vec![15, 29, 12, 5, 24, 4, 3, 7],
        vec![27, 8, 14, 18, 20, 32, 22],
        vec![30, 19, 9, 10, 23, 16, 11, 26, 6, 21],
        vec![13, 2, 17, 31],
        vec![28, 25]
    ];
    
    let bks_edges = get_canonical_edges(&r_bks);
    let mut rng = rand::thread_rng();
    
    // Quick calibration
    let mut deltas = Vec::new();
    let base_dist = calc_total_dist(&instance, &r_797);
    for _ in 0..10000 {
        if let Some(new_routes) = random_neighbor(&instance, &r_797, &mut rng) {
            let d = calc_total_dist(&instance, &new_routes) - base_dist;
            if d > 0.0 { deltas.push(d); }
        }
    }
    deltas.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let d_90 = deltas[(deltas.len() as f64 * 0.90) as usize];
    let t_start = -d_90 / 0.2f64.ln();
    
    let ceilings = vec![850.0, 900.0, 1000.0, 2500.0];
    
    for ceiling in ceilings {
        println!("\n=== M21D: SA with Ceiling = {:.0} ===", ceiling);
        let mut fp_788 = 0;
        let mut absolute_best_dist = 9999.0;
        let mut absolute_best_overlap = 0;
        
        for i in 0..10 {
            println!("  Starting run {}/10...", i+1);
            let stats = run_search(&instance, &r_797, &bks_edges, t_start, ceiling, &mut rng);
            if stats.fp_788 { fp_788 += 1; }
            if stats.best_dist < absolute_best_dist { absolute_best_dist = stats.best_dist; }
            if stats.best_overlap > absolute_best_overlap { absolute_best_overlap = stats.best_overlap; }
        }
        
        println!("  Absolute Best Dist: {:.4}", absolute_best_dist);
        println!("  Absolute Best BKS Overlap: {}", absolute_best_overlap);
        println!("  Success Rate (< 788): {}%", fp_788 * 10);
    }
}
