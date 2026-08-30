use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;
use rayon::prelude::*;

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;
use ultracrew::global_constructor::generate_feasible_seed;

fn main() {
    let file = File::create("p4_structural_results.csv").unwrap();
    let f = Arc::new(Mutex::new(file));
    {
        let mut file_guard = f.lock().unwrap();
        writeln!(file_guard, "family,instance,utilization_pct,global_constructor_enabled,seed,feasible_at_gen_0,generations_to_first_feasible,final_feasible,initial_fitness,first_feasible_fitness,best_fitness,final_fitness,seed_population_size,seed_feasible_count,construction_ms,optimization_ms,total_wall_ms,construction_overhead_pct").unwrap();
    }

    let num_seeds = 30;

    println!("Running Family A (Control)...");
    run_family_a(f.clone(), num_seeds);

    println!("Running Family B (Heterogeneous Durations)...");
    run_family_b(f.clone(), num_seeds);

    println!("Running Family C (Temporal Concentration)...");
    run_family_c(f.clone(), num_seeds);

    println!("Running Family D (Concurrency Boundary)...");
    run_family_d(f.clone(), num_seeds);
}

fn execute_matrix(f: Arc<Mutex<File>>, family: &str, instance: &str, util_pct: usize, shifts: Vec<Shift>, workers: Vec<Worker>, num_seeds: u64) {
    (1..=num_seeds).into_par_iter().for_each(|seed_val| {
        for &global_on in &[false, true] {
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
                workers: Arc::new(workers.clone()),
                shifts: Arc::new(shifts.clone()),
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
            let gen_first_feas_str = if generations_to_first_feasible == -1 { "null".to_string() } else { generations_to_first_feasible.to_string() };

            let mut file_guard = f.lock().unwrap();
            writeln!(file_guard, "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.2}",
                family, instance, util_pct, global_on, seed_val, feasible_at_gen_0, gen_first_feas_str,
                final_feasible, initial_fitness, first_feasible_fitness, best_fitness, final_fitness,
                seed_population_size, seed_feasible_count, construction_ms, optimization_ms, total_wall_ms, overhead_pct
            ).unwrap();
            file_guard.flush().unwrap();
        }
    });
}

fn create_workers() -> Vec<Worker> {
    let skill = Skill::new("FlightAttendant");
    (0..200).map(|i| Worker { id: (i + 1) as u64, skills: vec![skill.clone()] }).collect()
}

fn run_family_a(f: Arc<Mutex<File>>, num_seeds: u64) {
    let workers = create_workers();
    let skill = Skill::new("FlightAttendant");
    let mut shifts = vec![];
    let num_shifts = 950; // 95% of 8000 capacity / 8 = 950
    for i in 0..num_shifts {
        shifts.push(Shift { id: (i + 1) as u64, start_hour: (i * 8) % 168, duration_hours: 8, required_skill: skill.clone() });
    }
    shifts.sort_by_key(|s| s.start_hour);
    execute_matrix(f.clone(), "A", "Control_95", 95, shifts, workers, num_seeds);
}

fn run_family_b(f: Arc<Mutex<File>>, num_seeds: u64) {
    let workers = create_workers();
    let skill = Skill::new("FlightAttendant");
    let target_hours = 7600; // 95% util

    let instances = vec![
        ("B1", vec![6, 8, 10]),
        ("B2", vec![4, 8, 12]),
        ("B3", vec![4, 4, 12, 12]), // heavily skewed to extremes
    ];

    for (inst_name, dur_mix) in instances {
        let mut shifts = vec![];
        let mut current_hours = 0;
        let mut i = 0;
        while current_hours < target_hours {
            let dur = dur_mix[i % dur_mix.len()];
            if current_hours + dur > target_hours {
                let rem = target_hours - current_hours;
                if rem > 0 { shifts.push(Shift { id: (i + 1) as u64, start_hour: ((i * 4) % (168 - rem as usize)) as u64, duration_hours: rem as u64, required_skill: skill.clone() }); }
                break;
            }
            shifts.push(Shift { id: (i + 1) as u64, start_hour: ((i * 4) % (168 - dur as usize)) as u64, duration_hours: dur as u64, required_skill: skill.clone() });
            current_hours += dur;
            i += 1;
        }
        shifts.sort_by_key(|s| s.start_hour);
        execute_matrix(f.clone(), "B", inst_name, 95, shifts, workers.clone(), num_seeds);
    }
}

fn run_family_c(f: Arc<Mutex<File>>, num_seeds: u64) {
    let workers = create_workers();
    let skill = Skill::new("FlightAttendant");
    let target_hours = 7600;

    // C1: 40% weekend (baseline), C2: 60% weekend, C3: 85% weekend
    let instances = vec![
        ("C1", 0.40),
        ("C2", 0.60),
        ("C3", 0.85),
    ];

    for (inst_name, weekend_ratio) in instances {
        let mut shifts = vec![];
        let weekend_hours = (target_hours as f64 * weekend_ratio) as u64;
        let weekday_hours = target_hours - weekend_hours;
        
        let weekend_shifts = weekend_hours / 8;
        let weekday_shifts = weekday_hours / 8;
        
        for i in 0..weekend_shifts {
            shifts.push(Shift { id: (i + 1) as u64, start_hour: 120 + ((i * 8) % 40), duration_hours: 8, required_skill: skill.clone() });
        }
        for i in 0..weekday_shifts {
            shifts.push(Shift { id: (weekend_shifts + i + 1) as u64, start_hour: (i * 8) % 120, duration_hours: 8, required_skill: skill.clone() });
        }
        shifts.sort_by_key(|s| s.start_hour);
        execute_matrix(f.clone(), "C", inst_name, 95, shifts, workers.clone(), num_seeds);
    }
}

fn run_family_d(f: Arc<Mutex<File>>, num_seeds: u64) {
    let workers = create_workers();
    let skill = Skill::new("FlightAttendant");
    let target_hours = 7200; // 90% aggregate util (strictly feasible mathematically)

    let peaks = vec![195, 198, 199, 200, 201, 202, 205];
    
    for peak in peaks {
        let mut shifts = vec![];
        let peak_hours = peak * 8;
        let rem_hours = target_hours - peak_hours;
        let rem_shifts = rem_hours / 8;
        
        // The Trap: all `peak` shifts start exactly at hour 24
        for i in 0..peak {
            shifts.push(Shift { id: (i + 1) as u64, start_hour: 24, duration_hours: 8, required_skill: skill.clone() });
        }
        
        // The rest are safely distributed away from the trap
        for i in 0..rem_shifts {
            shifts.push(Shift { id: (peak + i + 1) as u64, start_hour: 40 + ((i * 8) % 120), duration_hours: 8, required_skill: skill.clone() });
        }
        
        shifts.sort_by_key(|s| s.start_hour);
        let inst_name = format!("D_{}", peak);
        execute_matrix(f.clone(), "D", &inst_name, 90, shifts, workers.clone(), num_seeds);
    }
}
