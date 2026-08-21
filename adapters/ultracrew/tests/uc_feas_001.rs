use std::collections::HashMap;
use std::sync::Arc;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::public_contracts::Scenario;
use ultracrew::constraint_engine::ConstraintEngine;
use ultracrew::optimization::{ScheduleOptimizer, ScheduleContext, ScheduleGenome};
use coralys_moga::traits::FitnessEvaluator;
use coralys_moga::runtime::optimization::metric::MetricReport;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::Observatory;

fn build_context(scenario: Option<Scenario>, shifts: Vec<Shift>) -> Arc<ScheduleContext> {
    let workers = vec![
        Worker { id: 1, skills: vec![Skill::new("Pilot")] },
        Worker { id: 2, skills: vec![Skill::new("Copilot")] },
    ];
    
    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology: WorkforceEcology::new(),
        rng_seed: 42,
        observatory: Arc::new(std::sync::Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario,
    })
}

fn build_genome(assignments: Vec<(u64, u64)>) -> ScheduleGenome {
    let mut map = HashMap::new();
    for (shift_id, worker_id) in assignments {
        map.insert(shift_id, worker_id);
    }
    ScheduleGenome { assignments: map }
}

fn check_evaluation(context: Arc<ScheduleContext>, genome: &ScheduleGenome) -> (usize, bool) {
    let constraint_engine = ConstraintEngine::new(context.clone());
    let report = constraint_engine.evaluate(genome);
    
    let optimizer = ScheduleOptimizer::new(context);
    let eval = optimizer.evaluate(genome, &MetricReport::default());
    
    (report.hard_violations, eval.is_valid)
}

#[test]
fn case_001_double_booking() {
    let shifts = vec![
        Shift { id: 101, start_hour: 0, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None },
        Shift { id: 103, start_hour: 4, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None },
    ];
    let context = build_context(None, shifts);
    
    let genome = build_genome(vec![(101, 1), (103, 1)]);
    let (hard_violations, is_valid) = check_evaluation(context, &genome);
    
    assert!(hard_violations > 0, "Expected HC2 (double booking) violation");
    assert!(!is_valid, "Genome with hard violations must be invalid");
}

#[test]
fn case_002_rest_7h59() {
    let shift1 = Shift { id: 101, start_hour: 0, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None };
    let shift2 = Shift { id: 102, start_hour: 15, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None };
    
    let context = build_context(Some(Scenario {
        planning_horizon_hours: None,
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    }), vec![shift1, shift2]);
    
    let genome = build_genome(vec![(101, 1), (102, 1)]);
    let (hard_violations, is_valid) = check_evaluation(context, &genome);
    
    assert!(hard_violations > 0, "Expected Rest violation (7h gap < 8h)");
    assert!(!is_valid, "Genome with hard violations must be invalid");
}

#[test]
fn case_003_rest_8h00() {
    let shift1 = Shift { id: 101, start_hour: 0, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None };
    let shift2 = Shift { id: 102, start_hour: 16, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None };
    
    let context = build_context(Some(Scenario {
        planning_horizon_hours: None,
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    }), vec![shift1, shift2]);
    
    let genome = build_genome(vec![(101, 1), (102, 1)]);
    let (hard_violations, is_valid) = check_evaluation(context, &genome);
    
    assert_eq!(hard_violations, 0, "Expected no hard violations for exactly 8h rest");
    assert!(is_valid, "Genome without hard violations must be valid");
}

#[test]
fn case_004_weekly_40h00() {
    let mut shifts = Vec::new();
    let mut assignments = Vec::new();
    for i in 0..5 {
        shifts.push(Shift { id: 100 + i as u64, start_hour: i * 24, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None });
        assignments.push((100 + i as u64, 1));
    }
    
    let context = build_context(Some(Scenario {
        planning_horizon_hours: None,
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    }), shifts);
    
    let genome = build_genome(assignments);
    let (hard_violations, is_valid) = check_evaluation(context, &genome);
    
    assert_eq!(hard_violations, 0, "Expected no violations for exactly 40h");
    assert!(is_valid, "Genome without hard violations must be valid");
}

#[test]
fn case_005_weekly_40h01() {
    let mut shifts = Vec::new();
    let mut assignments = Vec::new();
    for i in 0..5 {
        shifts.push(Shift { id: 100 + i as u64, start_hour: i * 24, duration_hours: 8, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None });
        assignments.push((100 + i as u64, 1));
    }
    shifts.push(Shift { id: 200, start_hour: 120, duration_hours: 1, required_skill: Skill::new("Pilot"), crew_role: None, flight_id: None });
    assignments.push((200, 1));
    
    let context = build_context(Some(Scenario {
        planning_horizon_hours: None,
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    }), shifts);
    
    let genome = build_genome(assignments);
    let (hard_violations, is_valid) = check_evaluation(context, &genome);
    
    assert!(hard_violations > 0, "Expected HC3 violation (41h > 40h)");
    assert!(!is_valid, "Genome with hard violations must be invalid");
}

#[test]
fn case_006_unqualified_worker() {
    let shift1 = Shift { id: 101, start_hour: 0, duration_hours: 8, required_skill: Skill::new("Copilot"), crew_role: None, flight_id: None };
    
    let context = build_context(None, vec![shift1]);
    
    let genome = build_genome(vec![(101, 1)]);
    let (hard_violations, is_valid) = check_evaluation(context, &genome);
    
    assert!(hard_violations > 0, "Expected HC1 violation (unqualified worker)");
    assert!(!is_valid, "Genome with hard violations must be invalid");
}
