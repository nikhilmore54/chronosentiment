use cvrp::{CvrpInstance, CvrpGenomeFactory};
use cvrp::moga_impl::{CvrpEvaluator};
use coralys_moga::traits::{GenomeFactory, FitnessEvaluator, Evaluated};
use rand::{Rng, SeedableRng};
use std::time::Instant;

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

fn random_neighbor(instance: &CvrpInstance, routes: &Vec<Vec<usize>>, rng: &mut impl Rng) -> Option<Vec<Vec<usize>>> {
    let num_routes = routes.len();
    if num_routes == 0 { return None; }

    for _ in 0..100 { 
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
                if num_routes < 2 { continue; }
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

fn main() {
    let instance = CvrpInstance::a_n32_k5();
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    let evaluator = CvrpEvaluator { instance: instance.clone() };

    let num_starts = 100;
    // MOGA budget: 100 pop * 10,000 gen = 1,000,000 steps
    let sa_steps = 1_000_000;
    
    let mut successes_to_790 = 0;
    let mut successes_to_bks = 0;
    let mut best_overall_dist = f64::INFINITY;
    let mut total_duration = 0.0;
    
    println!("=== M22F-CVRP-Control: Pure Route-Aware SA ===");
    println!("Starts: {}", num_starts);
    println!("Budget: {} steps per start", sa_steps);
    println!("BKS: 787.08\n");

    for start_idx in 0..num_starts {
        let seed = 42 + start_idx as u64;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        
        // 1. Generate random start
        let initial_cand = factory.create(&mut rng);
        let initial_eval = evaluator.evaluate(&initial_cand, &coralys_moga::runtime::optimization::metric::MetricReport::default());
        
        let mut current_routes = initial_eval.eval.routes.clone();
        let mut current_dist = calc_total_dist(&instance, &current_routes);
        let mut best_dist = current_dist;
        
        let t_start = 500.0;
        let alpha = (0.01_f64 / t_start).powf(1.0 / sa_steps as f64);
        let mut t = t_start;
        
        let start_time = Instant::now();
        
        for _ in 0..sa_steps {
            if let Some(new_routes) = random_neighbor(&instance, &current_routes, &mut rng) {
                let new_dist = calc_total_dist(&instance, &new_routes);
                let delta = new_dist - current_dist;
                
                if delta < 0.0 || rng.gen_range(0.0..1.0) < (-delta / t).exp() {
                    current_routes = new_routes;
                    current_dist = new_dist;
                    
                    if current_dist < best_dist {
                        best_dist = current_dist;
                    }
                }
            }
            t *= alpha;
        }
        
        let duration = start_time.elapsed().as_secs_f64();
        total_duration += duration;
        
        if best_dist < best_overall_dist {
            best_overall_dist = best_dist;
        }
        
        if best_dist <= 790.0 {
            successes_to_790 += 1;
        }
        if best_dist <= 788.0 {
            successes_to_bks += 1;
        }
        
        println!("Start {:>3} | Best Dist: {:>6.2} | Time: {:.2}s", start_idx + 1, best_dist, duration);
    }
    
    println!("\n=== Final Results ===");
    println!("Best Overall Dist : {:.2}", best_overall_dist);
    println!("Success < 790     : {} / {}", successes_to_790, num_starts);
    println!("Success <= 788    : {} / {}", successes_to_bks, num_starts);
    println!("Average Runtime   : {:.2}s per start", total_duration / num_starts as f64);
    println!("Total Runtime     : {:.2}s", total_duration);
}
