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
    scarce_worker_consumptions: usize,
    future_zero_candidate_events: usize,
}

fn is_legal(worker_id: u64, shift: &Shift, assigned: Option<&Vec<Shift>>, min_rest: u64, hc3_limit: u64) -> bool {
    match assigned {
        None => true,
        Some(a_list) => {
            let shift_end = shift.start_hour + shift.duration_hours;
            let mut no_overlap = true;
            let mut rest_ok = true;
            let mut current_hours = 0;
            
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
            
            let hc3_ok = current_hours + shift.duration_hours <= hc3_limit;
            no_overlap && rest_ok && hc3_ok
        }
    }
}

fn simulate_construction(num_workers: usize, num_shifts: usize, seed: u64, temporal_scarcity: bool) -> ConstructionStats {
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
    let lookahead_n = 10;
    let scarcity_threshold = 15;

    let mut clean_assignments = 0;
    let mut fallback_assignments = 0;
    let mut first_empty_pool_shift = None;
    let mut scarce_worker_consumptions = 0;
    let mut future_zero_candidate_events = 0;

    for shift_idx in 0..shifts.len() {
        let shift = &shifts[shift_idx];
        
        let clean: Vec<u64> = workers.iter()
            .filter(|w| w.skills.contains(&shift.required_skill))
            .filter(|w| is_legal(w.id, shift, worker_assigned.get(&w.id), min_rest, hc3_limit))
            .map(|w| w.id)
            .collect();

        // Measure future scarcity *before* assignment
        let mut future_scarcities = Vec::new();
        let end_idx = std::cmp::min(shift_idx + 1 + lookahead_n, shifts.len());
        
        for future_idx in (shift_idx + 1)..end_idx {
            let future_shift = &shifts[future_idx];
            let mut future_legal_count = 0;
            let mut future_legal_workers = Vec::new();
            
            for w in &workers {
                if w.skills.contains(&future_shift.required_skill) && is_legal(w.id, future_shift, worker_assigned.get(&w.id), min_rest, hc3_limit) {
                    future_legal_count += 1;
                    future_legal_workers.push(w.id);
                }
            }
            if future_legal_count == 0 {
                future_zero_candidate_events += 1;
            }
            future_scarcities.push((future_idx, future_legal_count, future_legal_workers));
        }

        if clean.is_empty() {
            if first_empty_pool_shift.is_none() {
                first_empty_pool_shift = Some(shift_idx);
            }
            fallback_assignments += 1;

            let qualified: Vec<u64> = workers.iter()
                .filter(|w| w.skills.contains(&shift.required_skill))
                .map(|w| w.id)
                .collect();
                
            let chosen_id = if qualified.is_empty() {
                workers[rng.gen_range(0..workers.len())].id
            } else {
                qualified[rng.gen_range(0..qualified.len())]
            };
            
            worker_assigned.entry(chosen_id).or_default().push(shift.clone());
        } else {
            clean_assignments += 1;
            
            let chosen_id = if temporal_scarcity {
                let mut best_id = clean[0];
                let mut min_scarcity_criticality = usize::MAX;
                
                for &wid in &clean {
                    let mut future_scarce_count = 0;
                    for (_, count, f_workers) in &future_scarcities {
                        if *count <= scarcity_threshold && f_workers.contains(&wid) {
                            future_scarce_count += 1;
                        }
                    }
                    if future_scarce_count < min_scarcity_criticality {
                        min_scarcity_criticality = future_scarce_count;
                        best_id = wid;
                    } else if future_scarce_count == min_scarcity_criticality {
                        if rng.gen_bool(0.5) {
                            best_id = wid;
                        }
                    }
                }
                best_id
            } else {
                clean[rng.gen_range(0..clean.len())]
            };
            
            // Did we consume a scarce worker? (Measure regardless of Control vs Treatment)
            let mut consumed_scarce = false;
            for (_, count, f_workers) in &future_scarcities {
                if *count <= scarcity_threshold && f_workers.contains(&chosen_id) {
                    consumed_scarce = true;
                }
            }
            if consumed_scarce {
                scarce_worker_consumptions += 1;
            }
            
            worker_assigned.entry(chosen_id).or_default().push(shift.clone());
        }
    }

    ConstructionStats {
        clean_assignments,
        fallback_assignments,
        first_empty_pool_shift,
        scarce_worker_consumptions,
        future_zero_candidate_events,
    }
}

struct ExperimentResult {
    c_stats: ConstructionStats,
    initial_valid_candidates: usize,
    initial_min_hc3: usize,
    initial_hc2_rest_violations: usize,
    initial_total_violations: usize,
    first_feasible_generation: Option<usize>,
    stable_feasible_generation: Option<usize>,
    final_fitness: f64,
    final_validity: bool,
}

fn run_experiment(num_workers: usize, num_shifts: usize, seed: u64, temporal_scarcity: bool) -> ExperimentResult {
    let c_stats = simulate_construction(num_workers, num_shifts, seed, temporal_scarcity);

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
        hc3_aware_initialization: true, // Always true for P2-C4
        temporal_scarcity_construction: temporal_scarcity,
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

    let obs = ctx.observatory.lock().unwrap();

    let mut first_feasible = None;
    let mut first_stable = None;
    let mut stable_counter = 0;

    for r in &obs.reports {
        if r.population_valid_count > 0 {
            if first_feasible.is_none() {
                first_feasible = Some(r.generation);
            }
            stable_counter += 1;
            if stable_counter >= 10 && first_stable.is_none() {
                first_stable = Some(r.generation - 9);
            }
        } else {
            stable_counter = 0;
        }
    }

    let r0 = &obs.reports[0];
    let r_final = obs.reports.last().unwrap();
    let initial_hc2_rest = r0.hard_violations - r0.hc3_violations; // approximate

    ExperimentResult {
        c_stats,
        initial_valid_candidates: r0.population_valid_count,
        initial_min_hc3: r0.hc3_violations,
        initial_hc2_rest_violations: initial_hc2_rest,
        initial_total_violations: r0.hard_violations,
        first_feasible_generation: first_feasible,
        stable_feasible_generation: first_stable,
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
    let control = run_experiment(num_workers, num_shifts, seed, false);

    eprintln!("Running Treatment...");
    let treatment = run_experiment(num_workers, num_shifts, seed, true);

    println!("| Metric | Control (P2-C1b) | Treatment (P2-C4) |");
    println!("|---|---|---|");
    println!("| Fallback assignments | {} | {} |", control.c_stats.fallback_assignments, treatment.c_stats.fallback_assignments);
    println!("| First clean-pool exhaustion shift | {} | {} |", format_opt(control.c_stats.first_empty_pool_shift), format_opt(treatment.c_stats.first_empty_pool_shift));
    println!("| Initial HC3 violations | {} | {} |", control.initial_min_hc3, treatment.initial_min_hc3);
    println!("| Initial HC2/Rest violations | {} | {} |", control.initial_hc2_rest_violations, treatment.initial_hc2_rest_violations);
    println!("| Initial total violations | {} | {} |", control.initial_total_violations, treatment.initial_total_violations);
    println!("| Scarce-worker consumptions | {} | {} |", control.c_stats.scarce_worker_consumptions, treatment.c_stats.scarce_worker_consumptions);
    println!("| Future zero-candidate events | {} | {} |", control.c_stats.future_zero_candidate_events, treatment.c_stats.future_zero_candidate_events);
    println!("| First feasible generation | {} | {} |", format_opt(control.first_feasible_generation), format_opt(treatment.first_feasible_generation));
    println!("| Stable-feasible generation | {} | {} |", format_opt(control.stable_feasible_generation), format_opt(treatment.stable_feasible_generation));
    println!("| Final validity | {} | {} |", control.final_validity, treatment.final_validity);
    println!("| Final fitness | {} | {} |", control.final_fitness, treatment.final_fitness);
}
