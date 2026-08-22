use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use cvrp::moga_impl::{CvrpEvaluator, CvrpRouteAwareMutator, CvrpCrossoverRoutePreserving};
use cvrp::{CvrpInstance, CvrpCandidate, CvrpGenomeFactory, CvrpEvaluation};
use coralys_moga::traits::*;

struct TelemetryRecord {
    generation: usize,
    parent_distance: f64,
    parent_route_count: usize,
    parent_longest_route: usize,
    parent_route_balance: i64, // rounded stddev
    child_genome: CvrpCandidate,
}

fn tournament_selection<'a>(evals: &'a [CvrpEvaluation], k: usize, rng: &mut StdRng) -> &'a CvrpEvaluation {
    let mut best: Option<&'a CvrpEvaluation> = None;
    for _ in 0..k {
        let idx = rng.gen_range(0..evals.len());
        let e = &evals[idx];
        if best.is_none() || e.total_distance < best.unwrap().total_distance {
            best = Some(e);
        }
    }
    best.unwrap()
}

fn compute_route_balance(routes: &[Vec<usize>]) -> i64 {
    if routes.is_empty() { return 0; }
    let mean = routes.iter().map(|r| r.len() as f64).sum::<f64>() / routes.len() as f64;
    let var = routes.iter().map(|r| (r.len() as f64 - mean).powi(2)).sum::<f64>() / routes.len() as f64;
    var.sqrt().round() as i64
}

fn compute_longest_route(routes: &[Vec<usize>]) -> usize {
    routes.iter().map(|r| r.len()).max().unwrap_or(0)
}

fn main() {
    let seed = 42;
    let instance = CvrpInstance::a_n32_k5();
    let evaluator = CvrpEvaluator { instance: instance.clone() };
    let mutator = CvrpRouteAwareMutator { instance: instance.clone() };
    let crossover = CvrpCrossoverRoutePreserving { instance: instance.clone() };
    let factory = CvrpGenomeFactory { num_customers: instance.customers.len() };
    
    let mut rng = StdRng::seed_from_u64(seed);
    
    let population_size = 100;
    let generations = 500;
    
    let mut population: Vec<CvrpCandidate> = (0..population_size)
        .map(|_| factory.create(&mut rng))
        .collect();
        
    let mut global_best: Option<CvrpEvaluation> = None;
    
    let mut out_csv = File::create("m30_2a_1_ecology_audit.csv").unwrap();
    writeln!(out_csv, "generation,parent_distance,parent_route_count,parent_longest_route,parent_route_balance,objective_delta,survived_generation,became_champion").unwrap();

    println!("Starting M30.2A.1 CVRP Passive Ecology Audit...");
    
    let mut pending_telemetry: Vec<TelemetryRecord> = Vec::new();

    for generation in 0..generations {
        let mut evals: Vec<CvrpEvaluation> = population.drain(..)
            .map(|g| evaluator.evaluate(&g, &coralys_moga::runtime::optimization::metric::MetricReport::default()).eval)
            .collect();
            
        // Sort by ascending distance
        evals.sort_by(|a, b| a.total_distance.partial_cmp(&b.total_distance).unwrap_or(Ordering::Equal));
        let gen_best = evals[0].clone();
        
        let mut new_global_best = false;
        if global_best.is_none() || gen_best.total_distance < global_best.as_ref().unwrap().total_distance {
            global_best = Some(gen_best.clone());
            new_global_best = true;
        }
        
        // Resolve pending telemetry
        for record in pending_telemetry.drain(..) {
            let mut child_distance = 0.0;
            let mut found = false;
            let mut survived_generation = false;
            let mut became_champion = false;
            
            for (rank, eval) in evals.iter().enumerate() {
                if record.child_genome == eval.candidate {
                    child_distance = eval.total_distance;
                    found = true;
                    if rank < population_size / 2 {
                        survived_generation = true;
                    }
                    if rank == 0 && new_global_best {
                        became_champion = true;
                    }
                    break;
                }
            }
            
            if found {
                // Negative delta is good (distance decreased)
                let objective_delta = child_distance - record.parent_distance;
                writeln!(out_csv, "{},{},{},{},{},{},{},{}",
                    record.generation, 
                    record.parent_distance,
                    record.parent_route_count, 
                    record.parent_longest_route, 
                    record.parent_route_balance,
                    objective_delta, 
                    survived_generation, 
                    became_champion
                ).unwrap();
            }
        }
        
        let mut next_gen = Vec::with_capacity(population_size);
        for i in 0..std::cmp::min(5, evals.len()) {
            next_gen.push(evals[i].candidate.clone());
        }
        
        while next_gen.len() < population_size {
            let p1 = tournament_selection(&evals, 3, &mut rng);
            let p2 = tournament_selection(&evals, 3, &mut rng);
            
            let (mut c1, mut c2) = if rng.gen_bool(0.8) {
                crossover.crossover(&p1.candidate, &p2.candidate, &mut rng)
            } else {
                (p1.candidate.clone(), p2.candidate.clone())
            };
            
            mutator.mutate(&mut c1, &mut rng);
            mutator.mutate(&mut c2, &mut rng);
            
            pending_telemetry.push(TelemetryRecord {
                generation,
                parent_distance: p1.total_distance,
                parent_route_count: p1.num_vehicles,
                parent_longest_route: compute_longest_route(&p1.routes),
                parent_route_balance: compute_route_balance(&p1.routes),
                child_genome: c1.clone(),
            });
            
            pending_telemetry.push(TelemetryRecord {
                generation,
                parent_distance: p2.total_distance,
                parent_route_count: p2.num_vehicles,
                parent_longest_route: compute_longest_route(&p2.routes),
                parent_route_balance: compute_route_balance(&p2.routes),
                child_genome: c2.clone(),
            });
            
            next_gen.push(c1);
            if next_gen.len() < population_size {
                next_gen.push(c2);
            }
        }
        population = next_gen;
    }
    
    println!("Done. Written to m30_2a_1_ecology_audit.csv");
}
