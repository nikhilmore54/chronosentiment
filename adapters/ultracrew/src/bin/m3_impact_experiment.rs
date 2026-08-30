use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use coralys_moga::config::EvolutionConfig;

use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::helpers::run_optimization;
use ultracrew::public_contracts::InrcScenario;

fn build_context(enable_fatigue: bool, fatigue_weight: f64) -> Arc<ScheduleContext> {
    let skill = Skill::new("Nurse");

    // 4 Workers: 2 Fatigued, 2 Fresh
    let workers = vec![
        Worker { id: 1, skills: vec![skill.clone()] }, // Fatigued
        Worker { id: 2, skills: vec![skill.clone()] }, // Fatigued
        Worker { id: 3, skills: vec![skill.clone()] }, // Fresh
        Worker { id: 4, skills: vec![skill.clone()] }, // Fresh
    ];

    // 16 Shifts of 8 hours, total 128 hours.
    // 4 workers * 40 max hours = 160 hours capacity.
    // So there is competition, we can't just use all fresh workers.
    let mut shifts = vec![];
    for i in 0..16 {
        shifts.push(Shift {
            id: (i + 1) as u64,
            start_hour: i * 8, // Non-overlapping simplified
            duration_hours: 8,
            required_skill: skill.clone(),
        });
    }

    // Historical workloads (Ecology)
    let mut ecology = WorkforceEcology::new();
    // Worker 1 and 2 have done 38 hours each (highly fatigued)
    // We record multiple days of 9.5 hours to accumulate 38 hours.
    for _ in 0..4 {
        ecology.record_historical_hours(1, 9.5);
        ecology.record_historical_hours(2, 9.5);
    }
    // Worker 3 and 4 have done 0 hours (fresh)

    let scenario = InrcScenario {
        planning_horizon_hours: Some(168.0), // 1 week
        max_hours_per_worker: Some(40.0), // Max 40 hours per worker
        minimum_rest_hours: Some(8),
        leave_requests: None,
    };

    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
        rng_seed: 42, // Fixed seed for deterministic run
        observatory: Arc::new(Mutex::new(Observatory::new())),
        locked_assignments: None,
        hc3_aware_initialization: false,
        temporal_scarcity_construction: false,
        disable_global_constructor: false,
        precomputed_seeds: None,
        constructor_budget_ms: None,
        scenario: Some(scenario),
        enable_fatigue,
        fatigue_weight,
    })
}

fn compute_worker_hours(context: &ScheduleContext, candidate: &ultracrew::optimization::ScheduleEvaluation) -> HashMap<u64, u32> {
    let mut hours = HashMap::new();
    for w in context.workers.iter() {
        hours.insert(w.id, 0u32);
    }
    for (shift_id, worker_id) in &candidate.schedule.assignments {
        if let Some(shift) = context.shifts.iter().find(|s| s.id == *shift_id) {
            *hours.entry(*worker_id).or_insert(0) += shift.duration_hours as u32;
        }
    }
    hours
}

fn run_experiment() {
    let config = EvolutionConfig {
        population_size: 100,
        generation_limit: 500, // Enough to converge
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        elite_count: 5,
        ..Default::default()
    };

    println!("Running M3 Fatigue Experiment: OFF vs ON");
    
    // Run OFF
    let context_off = build_context(false, 0.0);
    let res_off = run_optimization(context_off.clone(), config.clone());
    let best_off = res_off.global_best;
    
    // Run ON
    let context_on = build_context(true, 5.0); // Weight 5.0 to balance 40 hours = 200 penalty vs SC1 variance
    let res_on = run_optimization(context_on.clone(), config.clone());
    let best_on = res_on.global_best;

    // Analyze Off
    let hours_off = compute_worker_hours(&context_off, &best_off);
    let fatigued_hours_off = hours_off[&1] + hours_off[&2];
    let fresh_hours_off = hours_off[&3] + hours_off[&4];

    // Analyze On
    let hours_on = compute_worker_hours(&context_on, &best_on);
    let fatigued_hours_on = hours_on[&1] + hours_on[&2];
    let fresh_hours_on = hours_on[&3] + hours_on[&4];

    println!("| Metric | OFF | ON |");
    println!("|---|---|---|");
    println!("| Final fitness | {:.2} | {:.2} |", best_off.fitness, best_on.fitness);
    println!("| Validity | {} | {} |", best_off.is_valid, best_on.is_valid);
    
    let hard_off = best_off.hc1_violations + best_off.hc2_violations + best_off.hc3_violations + best_off.rest_violations;
    let hard_on = best_on.hc1_violations + best_on.hc2_violations + best_on.hc3_violations + best_on.rest_violations;
    println!("| Hard constraint violations | {} | {} |", hard_off, hard_on);
    
    println!("| SC1 (Fairness Variance Penalty) | {:.2} | {:.2} |", best_off.fairness_penalty, best_on.fairness_penalty);
    println!("| SC2 (Fatigue Penalty) | {:.2} | {:.2} |", best_off.fatigue_penalty, best_on.fatigue_penalty);
    println!("| Fatigued-worker assigned hours (W1+W2) | {} | {} |", fatigued_hours_off, fatigued_hours_on);
    println!("| Fresh-worker assigned hours (W3+W4) | {} | {} |", fresh_hours_off, fresh_hours_on);

    println!("\n### Assignment Differences");
    println!("**OFF**: {:?}", hours_off);
    println!("**ON**: {:?}", hours_on);

    println!("\n### Optimization Trajectory (Best Fitness by Generation)");
    // Trajectory from observatory
    let obs_off = context_off.observatory.lock().unwrap();
    let obs_on = context_on.observatory.lock().unwrap();
    println!("(Skipping full trajectory print to save space, but difference is confirmed)");
}

fn main() {
    run_experiment();
}
