use std::time::Instant;
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;
use rayon::prelude::*;

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory, run_partitioned_evolution, ScheduleEvaluation};
use ultracrew::partitioning::{Partitioner, Reconciler, TemporalPartitioner, ResourceClusterPartitioner, BoundaryReconciler};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;

fn main() {
    let file = File::create("p5_structural_decomposition.csv").unwrap();
    let f = Arc::new(Mutex::new(file));
    {
        let mut file_guard = f.lock().unwrap();
        writeln!(file_guard, "instance,treatment,num_partitions,seed,local_feasible_pct,boundary_conflicts,reconciled_feasible,global_feasible_gen_0,final_feasible,initial_fitness,final_fitness,optimization_ms").unwrap();
    }

    let num_seeds = 10;
    
    // We test on C2 and C3
    let workers = create_workers();
    
    // C2 (60% weekend)
    let shifts_c2 = generate_family_c(0.60);
    println!("Running C2...");
    run_treatments(f.clone(), "C2", shifts_c2, workers.clone(), num_seeds);
    
    // C3 (85% weekend)
    let shifts_c3 = generate_family_c(0.85);
    println!("Running C3...");
    run_treatments(f.clone(), "C3", shifts_c3, workers.clone(), num_seeds);
}

fn create_workers() -> Vec<Worker> {
    let skill = Skill::new("FlightAttendant");
    (0..200).map(|i| Worker { id: (i + 1) as u64, skills: vec![skill.clone()] }).collect()
}

fn generate_family_c(weekend_ratio: f64) -> Vec<Shift> {
    let skill = Skill::new("FlightAttendant");
    let target_hours = 7600;
    let weekend_hours = (target_hours as f64 * weekend_ratio) as u64;
    let weekday_hours = target_hours - weekend_hours;
    
    let weekend_shifts = weekend_hours / 8;
    let weekday_shifts = weekday_hours / 8;
    
    let mut shifts = vec![];
    for i in 0..weekend_shifts {
        shifts.push(Shift { id: (i + 1) as u64, start_hour: 120 + ((i * 8) % 40), duration_hours: 8, required_skill: skill.clone() });
    }
    for i in 0..weekday_shifts {
        shifts.push(Shift { id: (weekend_shifts + i + 1) as u64, start_hour: (i * 8) % 120, duration_hours: 8, required_skill: skill.clone() });
    }
    shifts.sort_by_key(|s| s.start_hour);
    shifts
}

fn run_treatments(f: Arc<Mutex<File>>, instance: &str, shifts: Vec<Shift>, workers: Vec<Worker>, num_seeds: u64) {
    let treatments = vec!["P0", "P1", "P2", "P4"];
    let num_partitions_options = vec![2, 4, 8];
    
    for treatment in treatments {
        if treatment == "P0" {
            execute_matrix(f.clone(), instance, treatment, 1, &shifts, &workers, num_seeds, None);
        } else {
            for &k in &num_partitions_options {
                let partitioner: Box<dyn Partitioner> = match treatment {
                    "P1" => Box::new(TemporalPartitioner { num_partitions: k, halo_hours: 0 }),
                    "P2" => Box::new(TemporalPartitioner { num_partitions: k, halo_hours: 24 }), // 24 hour halo
                    "P4" => Box::new(ResourceClusterPartitioner { num_partitions: k }),
                    _ => unreachable!(),
                };
                execute_matrix(f.clone(), instance, treatment, k, &shifts, &workers, num_seeds, Some(partitioner));
            }
        }
    }
}

fn execute_matrix(
    f: Arc<Mutex<File>>, 
    instance: &str, 
    treatment: &str, 
    num_partitions: usize, 
    shifts: &[Shift], 
    workers: &[Worker], 
    num_seeds: u64,
    partitioner: Option<Box<dyn Partitioner>>
) {
    (1..=num_seeds).into_par_iter().for_each(|seed_val| {
        let scenario = InrcScenario {
            planning_horizon_hours: Some(168.0),
            max_hours_per_worker: Some(40.0),
            minimum_rest_hours: Some(8),
            leave_requests: None,
        };

        let ctx = Arc::new(ScheduleContext {
            workers: Arc::new(workers.to_vec()),
            shifts: Arc::new(shifts.to_vec()),
            ecology: WorkforceEcology::new(),
            rng_seed: seed_val,
            observatory: Arc::new(Mutex::new(Observatory::new())),
            locked_assignments: None,
            scenario: Some(scenario),
            enable_fatigue: false,
            fatigue_weight: 0.0,
            hc3_aware_initialization: true,
            temporal_scarcity_construction: false,
            disable_global_constructor: false, // Let P0 try it
            precomputed_seeds: None,
            constructor_budget_ms: None,
        });

        let local_config = EvolutionConfig {
            population_size: 20,
            generation_limit: 10, // Fast local attempt
            mutation_rate: 0.2,
            crossover_rate: 0.8,
            elite_count: 2,
            seed: Some(seed_val),
            ..Default::default()
        };
        
        let global_config = EvolutionConfig {
            population_size: 100,
            generation_limit: 100, // global refinement
            mutation_rate: 0.2,
            crossover_rate: 0.8,
            elite_count: 5,
            seed: Some(seed_val),
            ..Default::default()
        };

        let opt_start = Instant::now();
        let (local_feasible_pct, boundary_conflicts, reconciled_feasible, global_result) = if let Some(p) = &partitioner {
            let reconciler = BoundaryReconciler;
            let part_res = run_partitioned_evolution(ctx.clone(), p.as_ref(), &reconciler, local_config, global_config);
            let local_pct = (part_res.local_feasible_count as f64 / part_res.total_partitions as f64) * 100.0;
            (format!("{:.1}", local_pct), part_res.boundary_conflicts.to_string(), part_res.reconciled_feasible.to_string(), part_res.global_result)
        } else {
            (String::from("N/A"), String::from("N/A"), String::from("N/A"), run_optimization(ctx.clone(), global_config))
        };
        let run_end = Instant::now();
        
        let optimization_ms = (run_end - opt_start).as_millis();
        
        let obs = ctx.observatory.lock().unwrap();
        
        let (global_feasible_gen_0, initial_fitness, final_feasible, final_fitness) = if obs.reports.is_empty() {
            (false, 0.0, false, 0.0)
        } else {
            let r_init = &obs.reports[0];
            let r_final = obs.reports.last().unwrap();
            (r_init.population_valid_count > 0, r_init.best_fitness, r_final.population_valid_count > 0, r_final.best_fitness)
        };

        let mut file_guard = f.lock().unwrap();
        writeln!(file_guard, "{},{},{},{},{},{},{},{},{},{},{},{}",
            instance, treatment, num_partitions, seed_val, local_feasible_pct, boundary_conflicts, reconciled_feasible, global_feasible_gen_0, final_feasible,
            initial_fitness, final_fitness, optimization_ms
        ).unwrap();
        file_guard.flush().unwrap();
    });
}
