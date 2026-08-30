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

fn main() {
    let num_workers = 200;
    let num_shifts = 1000;
    let seed = 42;

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
        hc3_aware_initialization: false, disable_global_constructor: false, precomputed_seeds: None, temporal_scarcity_construction: false,
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

    eprintln!("Running P2-C0 Convergence Characterization (Medium Workload)...");
    
    // Start timing
    let start_opt = Instant::now();
    let _res = run_optimization(ctx.clone(), config);
    let _sched_ms = start_opt.elapsed().as_secs_f64() * 1000.0;

    let obs = ctx.observatory.lock().unwrap();

    let mut first_generation_with_valid_candidate = None;
    let mut min_violations = usize::MAX;
    let mut min_violation_gen = 0;

    // Output CSV
    println!("generation,best_fitness,best_total_hard_violations,hc1,hc2,hc3,hc4,rest,population_valid_count,unique_genomes");
    for r in &obs.reports {
        println!("{},{:.2},{},{},{},{},{},{},{},{}",
            r.generation,
            r.best_fitness,
            r.hard_violations,
            r.hc1_violations,
            r.hc2_violations,
            r.hc3_violations,
            r.hc4_violations,
            r.rest_violations,
            r.population_valid_count,
            r.unique_genomes
        );

        if r.population_valid_count > 0 && first_generation_with_valid_candidate.is_none() {
            first_generation_with_valid_candidate = Some(r.generation);
        }

        if r.hard_violations < min_violations {
            min_violations = r.hard_violations;
            min_violation_gen = r.generation;
        }
    }

    eprintln!("--- First-Hit Feasibility ---");
    match first_generation_with_valid_candidate {
        Some(gen) => eprintln!("first_generation_with_valid_candidate: {}", gen),
        None => {
            eprintln!("first_generation_with_valid_candidate: NONE");
            eprintln!("minimum_violation_generation: {}", min_violation_gen);
            eprintln!("final_minimum_violation: {}", min_violations);
        }
    }

    if let Some(r0) = obs.reports.first() {
        eprintln!("--- Initial Population Audit (G0) ---");
        eprintln!("initial_valid_candidates: {}", r0.population_valid_count);
        eprintln!("initial_min_hard_violations: {}", r0.hard_violations);
        eprintln!("(Note: Initial mean/max hard violations are not tracked in GenerationTelemetry, relying on best constraint profile)");
    }
}
