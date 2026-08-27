use std::sync::Arc;
use std::collections::HashMap;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::public_contracts::InrcScenario;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::helpers::run_optimization;
use coralys_moga::config::EvolutionConfig;

fn build_context(num_workers: usize, num_shifts: usize, disable_global: bool, seed: u64) -> Arc<ScheduleContext> {
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
    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology: WorkforceEcology::new(),
        rng_seed: seed,
        observatory: Arc::new(std::sync::Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario: Some(scenario),
        enable_fatigue: false,
        fatigue_weight: 0.0,
        hc3_aware_initialization: true,
        temporal_scarcity_construction: false,
        disable_global_constructor: disable_global,
        precomputed_seeds: None,
    })
}

#[test]
fn test_routing_less_than_98_percent() {
    let ctx = build_context(200, 900, false, 42); // 90% utilization
    let config = EvolutionConfig { population_size: 10, generation_limit: 1, ..Default::default() };
    
    let _res = run_optimization(ctx.clone(), config);
    
    let obs = ctx.observatory.lock().unwrap();
    assert_eq!(obs.reports[0].unique_genomes, 10);
}

#[test]
fn test_routing_greater_than_or_equal_to_98_percent() {
    let ctx = build_context(200, 1000, false, 42); // 100% utilization
    let config = EvolutionConfig { population_size: 10, generation_limit: 1, ..Default::default() };
    
    let _res = run_optimization(ctx.clone(), config);
    
    let obs = ctx.observatory.lock().unwrap();
    assert!(obs.reports[0].population_valid_count >= 1);
    assert_eq!(obs.reports[0].best_fitness, 90000.0);
}

#[test]
fn test_routing_off_switch() {
    let ctx = build_context(200, 1000, true, 42); // 100% utilization, but global constructor disabled
    let config = EvolutionConfig { population_size: 10, generation_limit: 1, ..Default::default() };
    
    let _res = run_optimization(ctx.clone(), config);
    
    let obs = ctx.observatory.lock().unwrap();
    assert_eq!(obs.reports[0].population_valid_count, 0); // greedy fails
}

#[test]
fn test_routing_safe_fallback() {
    let ctx = build_context(200, 1100, false, 42); // 110% utilization - impossible
    let config = EvolutionConfig { population_size: 10, generation_limit: 1, ..Default::default() };
    
    let _res = run_optimization(ctx.clone(), config); 
    
    let obs = ctx.observatory.lock().unwrap();
    assert_eq!(obs.reports[0].population_valid_count, 0); // fallback to greedy, which also fails
}
