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

fn generate_csp_seed(shifts: &[Shift], workers: &[Worker]) -> ScheduleGenome {
    let mut block_needs = vec![0; 21];
    for s in shifts {
        block_needs[(s.start_hour / 8) as usize] += 1;
    }

    let mut csp_assignments = vec![Vec::new(); 21];
    let mut worker_hours = vec![0; workers.len() + 1];
    let num_workers = workers.len();
    
    // Simple DFS solver
    fn solve(block: usize, block_needs: &[usize], worker_hours: &mut [u64], csp_assignments: &mut [Vec<u64>], num_workers: usize) -> bool {
        if block == 21 {
            return true;
        }

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
    
    solve(0, &block_needs, &mut worker_hours, &mut csp_assignments, num_workers);
    
    let mut block_workers = csp_assignments.clone();
    let mut assignments = HashMap::new();
    for s in shifts {
        let block = (s.start_hour / 8) as usize;
        let w = block_workers[block].pop().unwrap();
        assignments.insert(s.id, w);
    }
    
    ScheduleGenome { assignments }
}

struct ExperimentResult {
    initial_valid_candidates: usize,
    first_feasible_generation: Option<usize>,
    final_fitness: f64,
    final_validity: bool,
    construction_ms: u128,
    optimization_ms: u128,
}

fn run_experiment(mode: &str) -> ExperimentResult {
    let num_workers = 200;
    let num_shifts = 1000;
    let seed = 42;

    let mut start = Instant::now();
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

    let precomputed_seeds = if mode == "control" {
        None
    } else {
        let csp_seed = generate_csp_seed(&shifts, &workers);
        let mut seeds = Vec::new();
        if mode == "monoculture" {
            for _ in 0..100 { seeds.push(csp_seed.clone()); }
        } else if mode == "hybrid_diverse" {
            seeds.push(csp_seed.clone());
            // other 99 will be greedy random fallbacks
        }
        Some(Arc::new(Mutex::new(seeds)))
    };
    let construction_ms = start.elapsed().as_millis();

    start = Instant::now();
    let ecology = WorkforceEcology::new();
    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    };

    let ctx = Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
        rng_seed: seed,
        observatory: Arc::new(Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario: Some(scenario),
        enable_fatigue: false,
        fatigue_weight: 0.0,
        hc3_aware_initialization: true,
        temporal_scarcity_construction: false,
        disable_global_constructor: false,
        precomputed_seeds: None,
        constructor_budget_ms: None,
    });

    let config = EvolutionConfig {
        population_size: 100,
        generation_limit: 300,
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        elite_count: 5,
        ..Default::default()
    };

    let _res = run_optimization(ctx.clone(), config);
    let optimization_ms = start.elapsed().as_millis();

    let obs = ctx.observatory.lock().unwrap();

    let mut first_feasible = None;
    for r in &obs.reports {
        if r.population_valid_count > 0 {
            if first_feasible.is_none() {
                first_feasible = Some(r.generation);
            }
        }
    }

    let r0 = &obs.reports[0];
    let r_final = obs.reports.last().unwrap();

    ExperimentResult {
        initial_valid_candidates: r0.population_valid_count,
        first_feasible_generation: first_feasible,
        final_fitness: r_final.best_fitness,
        final_validity: r_final.hard_violations == 0,
        construction_ms,
        optimization_ms,
    }
}

fn format_opt(opt: Option<usize>) -> String {
    match opt {
        Some(v) => v.to_string(),
        None => "N/A".to_string(),
    }
}

fn main() {
    eprintln!("Running Control...");
    let control = run_experiment("control");

    eprintln!("Running Monoculture...");
    let mono = run_experiment("monoculture");

    eprintln!("Running Hybrid Diverse...");
    let diverse = run_experiment("hybrid_diverse");

    println!("| Metric | Control (Greedy) | Monoculture (100 CSP) | Hybrid Diverse (1 CSP + 99 Greedy) |");
    println!("|---|---|---|---|");
    println!("| Construction time (ms) | {} | {} | {} |", control.construction_ms, mono.construction_ms, diverse.construction_ms);
    println!("| Initial valid candidates | {} | {} | {} |", control.initial_valid_candidates, mono.initial_valid_candidates, diverse.initial_valid_candidates);
    println!("| First feasible generation | {} | {} | {} |", format_opt(control.first_feasible_generation), format_opt(mono.first_feasible_generation), format_opt(diverse.first_feasible_generation));
    println!("| Final validity | {} | {} | {} |", control.final_validity, mono.final_validity, diverse.final_validity);
    println!("| Final fitness | {} | {} | {} |", control.final_fitness, mono.final_fitness, diverse.final_fitness);
    println!("| Optimization time (ms) | {} | {} | {} |", control.optimization_ms, mono.optimization_ms, diverse.optimization_ms);
}
