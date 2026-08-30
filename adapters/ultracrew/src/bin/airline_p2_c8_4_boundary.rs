use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;
use ultracrew::global_constructor::generate_feasible_seed;

fn main() {
    let utilizations = vec![100, 101, 102, 103, 105, 108, 109, 110];
    let num_seeds = 3; // We only need a few seeds to prove structural boundaries
    
    let mut f = File::create("p2_c8_boundary_results.csv").unwrap();
    writeln!(f, "utilization_pct,global_constructor_enabled,seed,feasible_at_gen_0,generations_to_first_feasible,final_feasible,initial_fitness,first_feasible_fitness,best_fitness,final_fitness,seed_population_size,seed_feasible_count,construction_ms,optimization_ms,total_wall_ms,construction_overhead_pct").unwrap();
    
    println!("Running Boundary Matrix...");
    run_matrix(&mut f, utilizations, num_seeds);
}

fn run_matrix(f: &mut File, utilizations: Vec<usize>, num_seeds: u64) {
    for &utilization_pct in &utilizations {
        for seed_val in 1..=num_seeds {
            for &global_on in &[false, true] {
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
                
                let scenario = InrcScenario {
                    planning_horizon_hours: Some(168.0),
                    max_hours_per_worker: Some(40.0),
                    minimum_rest_hours: Some(8),
                    leave_requests: None,
                };
                
                let total_wall_start = Instant::now();
                
                let mut construction_ms = 0;
                let mut precomputed = None;
                
                if global_on {
                    let c_start = Instant::now();
                    if let Some(seed) = generate_feasible_seed(&shifts, &workers, 8, 40, 1000) {
                        precomputed = Some(Arc::new(Mutex::new(vec![seed])));
                    }
                    construction_ms = c_start.elapsed().as_millis();
                }
                
                let ctx = Arc::new(ScheduleContext {
                    workers: Arc::new(workers),
                    shifts: Arc::new(shifts),
                    ecology: WorkforceEcology::new(),
                    rng_seed: seed_val,
                    observatory: Arc::new(Mutex::new(Observatory::new())),
                    locked_assignments: None,
                    scenario: Some(scenario),
                    enable_fatigue: false,
                    fatigue_weight: 0.0,
                    hc3_aware_initialization: true,
                    temporal_scarcity_construction: false,
                    disable_global_constructor: !global_on,
                    precomputed_seeds: precomputed,
                    constructor_budget_ms: None,
                });
                
                let config = EvolutionConfig {
                    population_size: 100,
                    generation_limit: 300,
                    mutation_rate: 0.2,
                    crossover_rate: 0.8,
                    elite_count: 5,
                    seed: Some(seed_val),
                    ..Default::default()
                };
                
                let opt_start = Instant::now();
                let _res = run_optimization(ctx.clone(), config);
                let run_end = Instant::now();
                
                let optimization_ms = (run_end - opt_start).as_millis();
                let total_wall_ms = (run_end - total_wall_start).as_millis();
                let overhead_pct = if !global_on || optimization_ms == 0 { 0.0 } else { (construction_ms as f64 / optimization_ms as f64) * 100.0 };
                
                let obs = ctx.observatory.lock().unwrap();
                let mut generations_to_first_feasible = -1;
                let mut first_feasible_fitness = 0.0;
                
                for r in &obs.reports {
                    if r.population_valid_count > 0 && generations_to_first_feasible == -1 {
                        generations_to_first_feasible = r.generation as i64;
                        first_feasible_fitness = r.best_fitness;
                    }
                }
                
                let r_init = &obs.reports[0];
                let r_final = obs.reports.last().unwrap();
                
                let feasible_at_gen_0 = r_init.population_valid_count > 0;
                let final_feasible = r_final.population_valid_count > 0;
                let initial_fitness = r_init.best_fitness;
                let final_fitness = r_final.best_fitness;
                
                let mut best_fitness = -100_000_000.0;
                for r in &obs.reports {
                    if r.best_fitness > best_fitness {
                        best_fitness = r.best_fitness;
                    }
                }
                
                let seed_population_size = 100;
                let seed_feasible_count = r_init.population_valid_count;
                
                let gen_first_feas_str = if generations_to_first_feasible == -1 {
                    "null".to_string()
                } else {
                    generations_to_first_feasible.to_string()
                };
                
                writeln!(f, "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.2}",
                    utilization_pct, global_on, seed_val, feasible_at_gen_0, gen_first_feas_str,
                    final_feasible, initial_fitness, first_feasible_fitness, best_fitness, final_fitness,
                    seed_population_size, seed_feasible_count, construction_ms, optimization_ms, total_wall_ms, overhead_pct
                ).unwrap();
                
                f.flush().unwrap();
                println!("Processed util={} seed={} on={}", utilization_pct, seed_val, global_on);
            }
        }
    }
}
