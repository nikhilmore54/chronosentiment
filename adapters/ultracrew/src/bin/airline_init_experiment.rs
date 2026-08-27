use std::time::Instant;
use std::sync::Arc;
use std::fs::File;
use std::io::Write;

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;
use std::sync::Mutex;

struct ExperimentResult {
    initial_valid_candidates: usize,
    initial_min_hc3: usize,
    initial_mean_hc3: f64,
    initial_total_violations: usize,
    first_feasible_generation: Option<usize>,
    first_stable_feasible_generation: Option<usize>,
    gen_50_min_hc3: Option<usize>,
    gen_100_min_hc3: Option<usize>,
    gen_150_min_hc3: Option<usize>,
    gen_200_min_hc3: Option<usize>,
    gen_253_min_hc3: Option<usize>,
    final_fitness: f64,
    final_validity: bool,
}

fn run_experiment(num_workers: usize, num_shifts: usize, seed: u64, hc3_aware: bool) -> ExperimentResult {
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
    let mut gen_50 = None;
    let mut gen_100 = None;
    let mut gen_150 = None;
    let mut gen_200 = None;
    let mut gen_253 = None;

    let mut stable_counter = 0;

    for r in &obs.reports {
        if r.generation == 50 { gen_50 = Some(r.hc3_violations); }
        if r.generation == 100 { gen_100 = Some(r.hc3_violations); }
        if r.generation == 150 { gen_150 = Some(r.hc3_violations); }
        if r.generation == 200 { gen_200 = Some(r.hc3_violations); }
        if r.generation == 253 { gen_253 = Some(r.hc3_violations); }

        if r.population_valid_count > 0 {
            if first_feasible.is_none() {
                first_feasible = Some(r.generation);
            }
            stable_counter += 1;
            if stable_counter >= 10 && first_stable.is_none() {
                // consider 10 continuous generations of having valid candidates as "stable"
                first_stable = Some(r.generation - 9);
            }
        } else {
            stable_counter = 0;
        }
    }

    let r0 = &obs.reports[0];
    let r_final = obs.reports.last().unwrap();

    ExperimentResult {
        initial_valid_candidates: r0.population_valid_count,
        initial_min_hc3: r0.hc3_violations,
        initial_mean_hc3: r0.average_hc3_violations,
        initial_total_violations: r0.hard_violations,
        first_feasible_generation: first_feasible,
        first_stable_feasible_generation: first_stable,
        gen_50_min_hc3: gen_50,
        gen_100_min_hc3: gen_100,
        gen_150_min_hc3: gen_150,
        gen_200_min_hc3: gen_200,
        gen_253_min_hc3: gen_253,
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

    println!("| Metric | Control | HC3-aware |");
    println!("|---|---|---|");
    println!("| Initial valid candidates | {} | {} |", control.initial_valid_candidates, treatment.initial_valid_candidates);
    println!("| Initial min HC3 | {} | {} |", control.initial_min_hc3, treatment.initial_min_hc3);
    println!("| Initial mean HC3 | {:.2} | {:.2} |", control.initial_mean_hc3, treatment.initial_mean_hc3);
    println!("| Initial total violations | {} | {} |", control.initial_total_violations, treatment.initial_total_violations);
    println!("| First feasible generation | {} | {} |", format_opt(control.first_feasible_generation), format_opt(treatment.first_feasible_generation));
    println!("| Gen 50 min HC3 | {} | {} |", format_opt(control.gen_50_min_hc3), format_opt(treatment.gen_50_min_hc3));
    println!("| Gen 100 min HC3 | {} | {} |", format_opt(control.gen_100_min_hc3), format_opt(treatment.gen_100_min_hc3));
    println!("| Gen 150 min HC3 | {} | {} |", format_opt(control.gen_150_min_hc3), format_opt(treatment.gen_150_min_hc3));
    println!("| Gen 200 min HC3 | {} | {} |", format_opt(control.gen_200_min_hc3), format_opt(treatment.gen_200_min_hc3));
    println!("| Gen 253 min HC3 | {} | {} |", format_opt(control.gen_253_min_hc3), format_opt(treatment.gen_253_min_hc3));
    println!("| First stable-feasible generation | {} | {} |", format_opt(control.first_stable_feasible_generation), format_opt(treatment.first_stable_feasible_generation));
    println!("| Final fitness | {:.2} | {:.2} |", control.final_fitness, treatment.final_fitness);
    println!("| Final validity | {} | {} |", control.final_validity, treatment.final_validity);
}
