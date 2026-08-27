use std::time::Instant;
use std::sync::Arc;
use std::collections::HashMap;

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;
use std::sync::Mutex;
use rand::{rngs::StdRng, SeedableRng, Rng};

struct ConstructionStats {
    clean_assignments: usize,
    fallback_assignments: usize,
    first_empty_pool_shift: Option<usize>,
    fallback_hc2: usize,
    fallback_rest: usize,
    fallback_hc3: usize,
    skill_shortages: usize,
    workers_24h: usize,
    workers_32h: usize,
    workers_40h: usize,
    workers_48h: usize,
    workers_64h: usize,
}

fn simulate_construction(num_workers: usize, num_shifts: usize, seed: u64, hc3_aware: bool, best_fit: bool) -> ConstructionStats {
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

    let mut rng = StdRng::seed_from_u64(seed);
    let mut worker_assigned: HashMap<u64, Vec<Shift>> = HashMap::new();
    
    let min_rest = 8;
    let hc3_limit = 40;

    let mut clean_assignments = 0;
    let mut fallback_assignments = 0;
    let mut first_empty_pool_shift = None;
    let mut fallback_hc2 = 0;
    let mut fallback_rest = 0;
    let mut fallback_hc3 = 0;
    let mut skill_shortages = 0;

    for (shift_idx, shift) in shifts.iter().enumerate() {
        let shift_end = shift.start_hour + shift.duration_hours;
        
        let clean: Vec<u64> = workers.iter()
            .filter(|w| w.skills.contains(&shift.required_skill))
            .filter(|w| {
                match worker_assigned.get(&w.id) {
                    None => true,
                    Some(assigned) => {
                        let mut no_overlap = true;
                        let mut rest_ok = true;
                        let mut current_hours = 0;
                        
                        for a in assigned {
                            let a_end = a.start_hour + a.duration_hours;
                            current_hours += a.duration_hours;
                            if shift.start_hour < a_end && a.start_hour < shift_end {
                                no_overlap = false;
                            }
                            if shift.start_hour < a_end + min_rest && a.start_hour < shift_end + min_rest {
                                rest_ok = false;
                            }
                        }
                        
                        let hc3_ok = if hc3_aware {
                            current_hours + shift.duration_hours <= hc3_limit
                        } else {
                            true
                        };
                        
                        no_overlap && rest_ok && hc3_ok
                    }
                }
            })
            .map(|w| w.id)
            .collect();

        if clean.is_empty() {
            if first_empty_pool_shift.is_none() {
                first_empty_pool_shift = Some(shift_idx);
            }
            fallback_assignments += 1;

            let qualified: Vec<u64> = workers.iter()
                .filter(|w| w.skills.contains(&shift.required_skill))
                .map(|w| w.id)
                .collect();
                
            if qualified.is_empty() {
                skill_shortages += 1;
                let w_id = workers[rng.gen_range(0..workers.len())].id;
                worker_assigned.entry(w_id).or_default().push(shift.clone());
            } else {
                let chosen_id = qualified[rng.gen_range(0..qualified.len())];
                
                if let Some(assigned) = worker_assigned.get(&chosen_id) {
                    let mut caused_overlap = false;
                    let mut caused_rest_fail = false;
                    let mut current_hours = 0;
                    
                    for a in assigned {
                        let a_end = a.start_hour + a.duration_hours;
                        current_hours += a.duration_hours;
                        if shift.start_hour < a_end && a.start_hour < shift_end {
                            caused_overlap = true;
                        }
                        if shift.start_hour < a_end + min_rest && a.start_hour < shift_end + min_rest {
                            caused_rest_fail = true;
                        }
                    }
                    if caused_overlap { fallback_hc2 += 1; }
                    if caused_rest_fail { fallback_rest += 1; }
                    if current_hours + shift.duration_hours > hc3_limit {
                        fallback_hc3 += 1;
                    }
                }
                
                worker_assigned.entry(chosen_id).or_default().push(shift.clone());
            }
        } else {
            clean_assignments += 1;
            let chosen_id = if best_fit {
                let mut best_id = clean[0];
                let mut max_current_hours = 0;
                for &wid in &clean {
                    let mut current_hours = 0;
                    if let Some(assigned) = worker_assigned.get(&wid) {
                        for a in assigned {
                            current_hours += a.duration_hours;
                        }
                    }
                    if current_hours > max_current_hours {
                        max_current_hours = current_hours;
                        best_id = wid;
                    } else if current_hours == max_current_hours {
                        if rng.gen_bool(0.5) {
                            best_id = wid;
                        }
                    }
                }
                best_id
            } else {
                clean[rng.gen_range(0..clean.len())]
            };
            
            worker_assigned.entry(chosen_id).or_default().push(shift.clone());
        }
    }

    let mut workers_24h = 0;
    let mut workers_32h = 0;
    let mut workers_40h = 0;
    let mut workers_48h = 0;
    let mut workers_64h = 0;
    
    for (_, assigned) in &worker_assigned {
        let mut hours = 0;
        for a in assigned { hours += a.duration_hours; }
        match hours {
            24 => workers_24h += 1,
            32 => workers_32h += 1,
            40 => workers_40h += 1,
            48 => workers_48h += 1,
            64 => workers_64h += 1,
            _ => {}
        }
    }

    ConstructionStats {
        clean_assignments,
        fallback_assignments,
        first_empty_pool_shift,
        fallback_hc2,
        fallback_rest,
        fallback_hc3,
        skill_shortages,
        workers_24h,
        workers_32h,
        workers_40h,
        workers_48h,
        workers_64h,
    }
}

struct ExperimentResult {
    c_stats: ConstructionStats,
    initial_valid_candidates: usize,
    initial_min_hc3: usize,
    initial_mean_hc3: f64,
    initial_total_violations: usize,
    first_feasible_generation: Option<usize>,
    final_fitness: f64,
    final_validity: bool,
}

fn run_experiment(num_workers: usize, num_shifts: usize, seed: u64, hc3_aware: bool, best_fit: bool) -> ExperimentResult {
    let c_stats = simulate_construction(num_workers, num_shifts, seed, hc3_aware, best_fit);

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
        hc3_aware_initialization: hc3_aware,
        best_fit_capacity_construction: best_fit,
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

    let obs = ctx.observatory.lock().unwrap();

    let mut first_feasible = None;

    for r in &obs.reports {
        if r.population_valid_count > 0 && first_feasible.is_none() {
            first_feasible = Some(r.generation);
        }
    }

    let r0 = &obs.reports[0];
    let r_final = obs.reports.last().unwrap();

    ExperimentResult {
        c_stats,
        initial_valid_candidates: r0.population_valid_count,
        initial_min_hc3: r0.hc3_violations,
        initial_mean_hc3: r0.average_hc3_violations,
        initial_total_violations: r0.hard_violations,
        first_feasible_generation: first_feasible,
        final_fitness: r_final.best_fitness,
        final_validity: r_final.hard_violations == 0,
    }
}

fn format_opt(opt: Option<usize>) -> String {
    match opt {
        Some(v) => v.to_string(),
        None => "N/A".to_string(),
    }
}

fn main() {
    let num_workers = 200;
    let num_shifts = 1000;
    let seed = 42;

    eprintln!("Running Control...");
    let control = run_experiment(num_workers, num_shifts, seed, true, false);

    eprintln!("Running Treatment...");
    let treatment = run_experiment(num_workers, num_shifts, seed, true, true);

    println!("| Metric | Control (P2-C1b) | Treatment (Best-Fit) |");
    println!("|---|---|---|");
    println!("| Clean pool assignments | {} | {} |", control.c_stats.clean_assignments, treatment.c_stats.clean_assignments);
    println!("| Fallback assignments | {} | {} |", control.c_stats.fallback_assignments, treatment.c_stats.fallback_assignments);
    println!("| First empty pool shift | {} | {} |", format_opt(control.c_stats.first_empty_pool_shift), format_opt(treatment.c_stats.first_empty_pool_shift));
    println!("| Skill shortages | {} | {} |", control.c_stats.skill_shortages, treatment.c_stats.skill_shortages);
    println!("| Fallback HC3 violations | {} | {} |", control.c_stats.fallback_hc3, treatment.c_stats.fallback_hc3);
    println!("| Fallback HC2/Rest violations | {} | {} |", control.c_stats.fallback_hc2 + control.c_stats.fallback_rest, treatment.c_stats.fallback_hc2 + treatment.c_stats.fallback_rest);
    println!("| Workers @ 40h | {} | {} |", control.c_stats.workers_40h, treatment.c_stats.workers_40h);
    println!("| Workers @ 24h | {} | {} |", control.c_stats.workers_24h, treatment.c_stats.workers_24h);
    println!("| Workers @ 48h | {} | {} |", control.c_stats.workers_48h, treatment.c_stats.workers_48h);
    println!("| Initial valid candidates | {} | {} |", control.initial_valid_candidates, treatment.initial_valid_candidates);
    println!("| Initial min HC3 | {} | {} |", control.initial_min_hc3, treatment.initial_min_hc3);
    println!("| Initial total violations | {} | {} |", control.initial_total_violations, treatment.initial_total_violations);
    println!("| First feasible generation | {} | {} |", format_opt(control.first_feasible_generation), format_opt(treatment.first_feasible_generation));
}
