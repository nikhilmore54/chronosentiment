use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory, ScheduleGenome};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;
use rand::{rngs::StdRng, SeedableRng, Rng};
use rand::seq::SliceRandom;

fn global_oracle_optimal(shifts: &[Shift], workers: &[Worker]) -> Option<ScheduleGenome> {
    let num_shifts = shifts.len();
    let num_workers = workers.len();
    
    if num_shifts * 8 > num_workers * 40 {
        return None;
    }

    let mut block_needs = vec![0; 21];
    for s in shifts {
        block_needs[(s.start_hour / 8) as usize] += 1;
    }

    let mut csp_assignments = vec![Vec::new(); 21];
    let mut worker_hours = vec![0; num_workers + 1];
    
    fn solve(block: usize, block_needs: &[usize], worker_hours: &mut [u64], csp_assignments: &mut [Vec<u64>], num_workers: usize) -> bool {
        if block == 21 { return true; }
        if csp_assignments[block].len() == block_needs[block] {
            return solve(block + 1, block_needs, worker_hours, csp_assignments, num_workers);
        }

        let mut candidates: Vec<u64> = (1..=num_workers as u64).collect();
        candidates.sort_by_key(|&wid| worker_hours[wid as usize]);

        for wid in candidates {
            if worker_hours[wid as usize] >= 40 { continue; }
            if csp_assignments[block].contains(&wid) { continue; }
            
            let mut rest_ok = true;
            if block > 0 && csp_assignments[block - 1].contains(&wid) { rest_ok = false; }
            if block < 20 && csp_assignments[block + 1].contains(&wid) { rest_ok = false; }
            
            if rest_ok {
                csp_assignments[block].push(wid);
                worker_hours[wid as usize] += 8;
                
                if solve(block, block_needs, worker_hours, csp_assignments, num_workers) {
                    return true;
                }
                
                worker_hours[wid as usize] -= 8;
                csp_assignments[block].pop();
            }
        }
        false
    }
    
    if solve(0, &block_needs, &mut worker_hours, &mut csp_assignments, num_workers) {
        let mut block_workers = csp_assignments.clone();
        let mut assignments = HashMap::new();
        for s in shifts {
            let block = (s.start_hour / 8) as usize;
            let w = block_workers[block].pop().unwrap();
            assignments.insert(s.id, w);
        }
        Some(ScheduleGenome { assignments })
    } else {
        None
    }
}

fn permute_genome(base: &ScheduleGenome, num_workers: usize, rng: &mut StdRng) -> ScheduleGenome {
    let mut worker_ids: Vec<u64> = (1..=num_workers as u64).collect();
    worker_ids.shuffle(rng);
    
    let mut mapping = HashMap::new();
    for i in 1..=num_workers {
        mapping.insert(i as u64, worker_ids[i - 1]);
    }
    
    let mut new_assignments = HashMap::new();
    for (&sid, &wid) in &base.assignments {
        new_assignments.insert(sid, *mapping.get(&wid).unwrap());
    }
    
    ScheduleGenome { assignments: new_assignments }
}

fn generate_portfolio(shifts: &[Shift], workers: &[Worker], num_csp: usize, seed: u64) -> Option<Vec<ScheduleGenome>> {
    if num_csp == 0 { return None; }
    
    let base_seed = global_oracle_optimal(shifts, workers)?;
    let mut portfolio = Vec::new();
    portfolio.push(base_seed.clone());
    
    let mut rng = StdRng::seed_from_u64(seed + 1000);
    for _ in 1..num_csp {
        portfolio.push(permute_genome(&base_seed, workers.len(), &mut rng));
    }
    
    Some(portfolio)
}

fn run_portfolio_test(name: &str, num_csp_seeds: usize, seed_val: u64) -> String {
    let num_workers = 200;
    let num_shifts = 1000;
    
    let skill = Skill::new("FlightAttendant");
    let mut workers = vec![];
    for i in 0..num_workers {
        workers.push(Worker { id: (i + 1) as u64, skills: vec![skill.clone()] });
    }
    let mut shifts = vec![];
    for i in 0..num_shifts {
        shifts.push(Shift {
            id: (i + 1) as u64,
            start_hour: (i * 8) as u64 % 168,
            duration_hours: 8,
            required_skill: skill.clone(),
        });
    }
    shifts.sort_by_key(|s| s.start_hour);

    let portfolio = generate_portfolio(&shifts, &workers, num_csp_seeds, seed_val);
    let precomputed_seeds = portfolio.map(|p| Arc::new(Mutex::new(p)));

    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0), max_hours_per_worker: Some(40.0), minimum_rest_hours: Some(8), leave_requests: None,
    };
    let ctx = Arc::new(ScheduleContext {
        workers: Arc::new(workers), shifts: Arc::new(shifts), ecology: WorkforceEcology::new(),
        rng_seed: seed_val, observatory: Arc::new(Mutex::new(Observatory::new())), locked_assignments: None,
        scenario: Some(scenario), enable_fatigue: false, fatigue_weight: 0.0, hc3_aware_initialization: true, temporal_scarcity_construction: false,
        disable_global_constructor: false,
        precomputed_seeds,
    });

    let config = EvolutionConfig {
        population_size: 100, generation_limit: 300, mutation_rate: 0.2, crossover_rate: 0.8, elite_count: 5, seed: Some(seed_val), ..Default::default()
    };

    let start = Instant::now();
    let _res = run_optimization(ctx.clone(), config);
    let opt_ms = start.elapsed().as_millis();

    let obs = ctx.observatory.lock().unwrap();
    
    let mut first_feasible = None;
    for r in &obs.reports {
        if r.population_valid_count > 0 && first_feasible.is_none() {
            first_feasible = Some(r.generation);
        }
    }

    let mut stable_feasible = None;
    for r in &obs.reports {
        if r.population_valid_count > 50 && stable_feasible.is_none() {
            stable_feasible = Some(r.generation);
        }
    }

    // Measure time to best fitness
    let r_final = obs.reports.last().unwrap();
    let final_fitness = r_final.best_fitness;
    let mut gen_best = 0;
    for r in &obs.reports {
        if r.best_fitness >= final_fitness {
            gen_best = r.generation;
            break;
        }
    }

    let r_init = &obs.reports[0];
    
    format!("{:20} | InitValid: {:>3} | InitUnique: {:>3} | FinFit: {:>7.1} | BestGen: {:>3} | FinUnique: {:>3} | FinValid: {:>3} | OptMs: {:>5}",
        name,
        r_init.population_valid_count,
        r_init.unique_genomes,
        r_final.best_fitness,
        gen_best,
        r_final.unique_genomes,
        r_final.population_valid_count,
        opt_ms
    )
}

fn main() {
    println!("=== P2-C8-2: Initial Population Portfolio Characterization (1000/200 Zero-Slack) ===");
    let scenarios = vec![
        ("P0 (100% Greedy)", 0),
        ("P1 (1 CSP)", 1),
        ("P2 (10 CSP)", 10),
        ("P3 (25 CSP)", 25),
        ("P4 (50 CSP)", 50),
        ("P5 (100 CSP)", 100),
    ];
    
    for (name, csp_count) in scenarios {
        let res = run_portfolio_test(name, csp_count, 42);
        println!("{}", res);
    }
}
