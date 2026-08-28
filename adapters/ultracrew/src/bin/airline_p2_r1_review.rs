use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

use coralys_moga::config::EvolutionConfig;
use coralys_moga::observatory::GenerationObserver;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory, ScheduleGenome};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;
use rand::{rngs::StdRng, SeedableRng, Rng};
use coralys_moga::traits::{Genome, Evaluated};

#[derive(Clone)]
enum SeedStrategy {
    None, // Greedy random fallback
    Optimal, // Balanced
    Mediocre, // Packed
}

struct OracleResult {
    feasible: bool,
    time_ms: u128,
    seed: Option<ScheduleGenome>,
}

fn global_oracle(shifts: &[Shift], workers: &[Worker], strategy: SeedStrategy) -> OracleResult {
    let start = Instant::now();
    let num_shifts = shifts.len();
    let num_workers = workers.len();
    
    // Quick capacity check
    if num_shifts * 8 > num_workers * 40 {
        return OracleResult { feasible: false, time_ms: start.elapsed().as_millis(), seed: None };
    }

    let mut block_needs = vec![0; 21];
    for s in shifts {
        block_needs[(s.start_hour / 8) as usize] += 1;
    }

    let mut csp_assignments = vec![Vec::new(); 21];
    let mut worker_hours = vec![0; num_workers + 1];
    
    fn solve(block: usize, block_needs: &[usize], worker_hours: &mut [u64], csp_assignments: &mut [Vec<u64>], num_workers: usize, strategy: &SeedStrategy) -> bool {
        if block == 21 { return true; }
        if csp_assignments[block].len() == block_needs[block] {
            return solve(block + 1, block_needs, worker_hours, csp_assignments, num_workers, strategy);
        }

        let mut candidates: Vec<u64> = (1..=num_workers as u64).collect();
        match strategy {
            SeedStrategy::Optimal => {
                candidates.sort_by_key(|&wid| worker_hours[wid as usize]); // Prefer least used
            },
            SeedStrategy::Mediocre => {
                // Prefer most used to create extreme variance
                candidates.sort_by(|&a, &b| worker_hours[b as usize].cmp(&worker_hours[a as usize]));
            },
            _ => {}
        }

        for wid in candidates {
            if worker_hours[wid as usize] >= 40 { continue; }
            if csp_assignments[block].contains(&wid) { continue; }
            
            let mut rest_ok = true;
            if block > 0 && csp_assignments[block - 1].contains(&wid) { rest_ok = false; }
            if block < 20 && csp_assignments[block + 1].contains(&wid) { rest_ok = false; }
            
            if rest_ok {
                csp_assignments[block].push(wid);
                worker_hours[wid as usize] += 8;
                
                if solve(block, block_needs, worker_hours, csp_assignments, num_workers, strategy) {
                    return true;
                }
                
                worker_hours[wid as usize] -= 8;
                csp_assignments[block].pop();
            }
        }
        false
    }
    
    let feasible = solve(0, &block_needs, &mut worker_hours, &mut csp_assignments, num_workers, &strategy);
    let time_ms = start.elapsed().as_millis();
    
    if feasible {
        let mut block_workers = csp_assignments.clone();
        let mut assignments = HashMap::new();
        for s in shifts {
            let block = (s.start_hour / 8) as usize;
            let w = block_workers[block].pop().unwrap();
            assignments.insert(s.id, w);
        }
        OracleResult { feasible, time_ms, seed: Some(ScheduleGenome { assignments }) }
    } else {
        OracleResult { feasible, time_ms, seed: None }
    }
}

// Minimal dummy types to pass compilation while calling the standard run_optimization.
// Since run_optimization takes ownership of config and context, we rely on the internal Observatory to get metrics.
// We'll calculate diversity manually by letting MOGA run.

fn run_workload(num_shifts: usize, num_workers: usize, strategy: SeedStrategy, seed_val: u64) -> String {
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

    let oracle_res = match strategy {
        SeedStrategy::None => OracleResult { feasible: false, time_ms: 0, seed: None },
        _ => global_oracle(&shifts, &workers, strategy.clone()),
    };

    if !oracle_res.feasible && matches!(strategy, SeedStrategy::Optimal | SeedStrategy::Mediocre) {
        return format!("Workload: {}/{} | Structurally Infeasible | Oracle Time: {} ms", num_shifts, num_workers, oracle_res.time_ms);
    }

    let precomputed_seeds = match oracle_res.seed {
        Some(s) => Some(Arc::new(Mutex::new(vec![s]))), // Hybrid: 1 seed + 99 greedy
        None => None,
    };

    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0), max_hours_per_worker: Some(40.0), minimum_rest_hours: Some(8), leave_requests: None,
    };
    let ctx = Arc::new(ScheduleContext {
        workers: Arc::new(workers), shifts: Arc::new(shifts), ecology: WorkforceEcology::new(),
        rng_seed: seed_val, observatory: Arc::new(Mutex::new(Observatory::new())), locked_assignments: None,
        scenario: Some(scenario), enable_fatigue: false, fatigue_weight: 0.0, hc3_aware_initialization: true, temporal_scarcity_construction: false,
        precomputed_seeds, disable_global_constructor: false,
    });

    let config = EvolutionConfig {
        population_size: 100, generation_limit: 300, mutation_rate: 0.2, crossover_rate: 0.8, elite_count: 5, ..Default::default()
    };

    let start = Instant::now();
    let res = run_optimization(ctx.clone(), config);
    let opt_ms = start.elapsed().as_millis();

    let obs = ctx.observatory.lock().unwrap();
    let mut first_feasible = None;
    for r in &obs.reports {
        if r.population_valid_count > 0 && first_feasible.is_none() {
            first_feasible = Some(r.generation);
        }
    }

    let r_final = obs.reports.last().unwrap();
    format!("Opt ms: {} | FeasibleGen: {:?} | FinalFit: {} | FinalValid: {} | InitValid: {} | FinalUnique: {}",
        opt_ms, first_feasible, r_final.best_fitness, r_final.hard_violations == 0, obs.reports[0].population_valid_count, r_final.unique_genomes)
}

fn main() {
    println!("=== P2-R1 R2: Separating Feasibility from Fitness ===");
    println!("Note: For 1000/200, there is zero slack (100% capacity). All feasible solutions MUST use 40h/worker, meaning fairness variance is mathematically forced to 0 (Optimal).");
    println!("We test R2 on the Slack workload (500/200) where variance is possible.");
    let r2_control = run_workload(500, 200, SeedStrategy::None, 42);
    println!("Control (Greedy, 500/200): {}", r2_control);
    let r2_optimal = run_workload(500, 200, SeedStrategy::Optimal, 42);
    println!("Hybrid Optimal (500/200): {}", r2_optimal);
    let r2_mediocre = run_workload(500, 200, SeedStrategy::Mediocre, 42);
    println!("Hybrid Mediocre (500/200): {}", r2_mediocre);

    println!("\n=== P2-R1 R3: Workload Generality ===");
    println!("Slack (500/200, Optimal): {}", r2_optimal);
    println!("Zero-Slack (1000/200, Optimal): {}", run_workload(1000, 200, SeedStrategy::Optimal, 42));
    println!("CapVariation (1000/250, Optimal): {}", run_workload(1000, 250, SeedStrategy::Optimal, 42));
    println!("Infeasible (1500/200, Optimal): {}", run_workload(1500, 200, SeedStrategy::Optimal, 42));

    println!("\n=== P2-R1 R1: Reproducibility across seeds (Hybrid Diverse Optimal) ===");
    for seed in [10, 25, 42, 99, 123] {
        let res = run_workload(1000, 200, SeedStrategy::Optimal, seed);
        println!("Seed {}: {}", seed, res);
    }
}
