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

#[derive(Debug, Clone)]
struct ConstructionTelemetry {
    fallback_assignments: usize,
    first_clean_pool_exhaustion: Option<usize>,
    construction_time_ms: u128,
}

fn simulate_greedy_construction(shifts: &[Shift], workers: &[Worker], seed: u64) -> ConstructionTelemetry {
    let mut rng = StdRng::seed_from_u64(seed);
    let start = Instant::now();
    
    let mut worker_assigned: HashMap<u64, Vec<Shift>> = HashMap::new();
    let mut fallback_assignments = 0;
    let mut first_clean_pool_exhaustion = None;
    
    for (shift_idx, shift) in shifts.iter().enumerate() {
        let mut clean = Vec::new();
        let min_rest = 8;
        let hc3_limit = 40;
        
        for w in workers {
            let mut no_overlap = true;
            let mut rest_ok = true;
            let mut current_hours = 0;
            
            if let Some(a_list) = worker_assigned.get(&w.id) {
                let shift_end = shift.start_hour + shift.duration_hours;
                for a in a_list {
                    let a_end = a.start_hour + a.duration_hours;
                    current_hours += a.duration_hours;
                    if shift.start_hour < a_end && a.start_hour < shift_end {
                        no_overlap = false;
                    }
                    if shift.start_hour < a_end + min_rest && a.start_hour < shift_end + min_rest {
                        rest_ok = false;
                    }
                }
            }
            
            let hc3_ok = current_hours + shift.duration_hours <= hc3_limit;
            if w.skills.contains(&shift.required_skill) && no_overlap && rest_ok && hc3_ok {
                clean.push(w.id);
            }
        }
        
        let chosen_id = if !clean.is_empty() {
            clean[rng.gen_range(0..clean.len())]
        } else {
            fallback_assignments += 1;
            if first_clean_pool_exhaustion.is_none() {
                first_clean_pool_exhaustion = Some(shift_idx);
            }
            // fallback pick
            let qualified: Vec<u64> = workers.iter().filter(|w| w.skills.contains(&shift.required_skill)).map(|w| w.id).collect();
            qualified[rng.gen_range(0..qualified.len())]
        };
        
        worker_assigned.entry(chosen_id).or_insert_with(Vec::new).push(shift.clone());
    }
    
    ConstructionTelemetry {
        fallback_assignments,
        first_clean_pool_exhaustion,
        construction_time_ms: start.elapsed().as_millis(),
    }
}

fn run_workload(utilization_pct: usize, seed_val: u64) -> String {
    let num_workers = 200;
    let max_hours = 40;
    let total_capacity = num_workers * max_hours;
    let num_shifts = (total_capacity * utilization_pct) / 100 / 8;
    
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

    let construction_telemetry = simulate_greedy_construction(&shifts, &workers, seed_val);

    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0), max_hours_per_worker: Some(40.0), minimum_rest_hours: Some(8), leave_requests: None,
    };
    let ctx = Arc::new(ScheduleContext {
        workers: Arc::new(workers), shifts: Arc::new(shifts), ecology: WorkforceEcology::new(),
        rng_seed: seed_val, observatory: Arc::new(Mutex::new(Observatory::new())), locked_assignments: None,
        scenario: Some(scenario), enable_fatigue: false, fatigue_weight: 0.0, hc3_aware_initialization: true, temporal_scarcity_construction: false,
        precomputed_seeds: None, disable_global_constructor: false,
        constructor_budget_ms: None,
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
    
    // Stable feasible = generation where population_valid_count > 50 (majority)
    let mut stable_feasible = None;
    for r in &obs.reports {
        if r.population_valid_count > 50 && stable_feasible.is_none() {
            stable_feasible = Some(r.generation);
        }
    }

    let r_init = &obs.reports[0];
    let r_final = obs.reports.last().unwrap();
    
    format!("Util: {:>3}% | InitValid: {:>3} | InitViol: {:>4} (HC3: {:>4}, Rest/HC2: {:>4}) | Fallbacks: {:>3} | FirstExhaust: {:>4} | FeasGen: {:>4} | StableGen: {:>4} | ConstMs: {:>2} | OptMs: {:>5} | FinValid: {} | FinFit: {}",
        utilization_pct,
        r_init.population_valid_count,
        r_init.hard_violations,
        r_init.hc3_violations,
        r_init.hc2_violations + r_init.rest_violations,
        construction_telemetry.fallback_assignments,
        construction_telemetry.first_clean_pool_exhaustion.unwrap_or(0),
        first_feasible.map(|g| g as i64).unwrap_or(-1),
        stable_feasible.map(|g| g as i64).unwrap_or(-1),
        construction_telemetry.construction_time_ms,
        opt_ms,
        r_final.population_valid_count > 0,
        r_final.best_fitness
    )
}

fn main() {
    println!("=== P2-C8 Phase 1: Boundary Characterization ===");
    let utils = vec![85, 90, 93, 95, 97, 99, 100];
    let seeds = vec![10, 42, 99];
    
    for u in utils {
        for s in &seeds {
            let res = run_workload(u, *s);
            println!("{}", res);
        }
        println!("-------------------------------------------------------------------------------------------------------------------------");
    }
}
