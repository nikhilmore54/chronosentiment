use ultracrew::models::{Shift, Worker, Skill};
use ultracrew::optimization::{ScheduleContext, ScheduleGenome, ScheduleOptimizer, Observatory};
use ultracrew::public_contracts::{Scenario, SchedulingDomain};
use ultracrew::ecology::WorkforceEcology;
use coralys_moga::traits::FitnessEvaluator;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

fn build_worker(id: u64, skill: &str) -> Worker {
    Worker {
        id,
        skills: vec![Skill::new(skill)],
    }
}

fn build_shift(id: u64, start: u64, dur: u64, skill: &str, flight_id: Option<&str>) -> Shift {
    Shift {
        id,
        start_hour: start,
        duration_hours: dur,
        required_skill: Skill::new(skill),
        flight_id: flight_id.map(|s| s.to_string()),
        crew_role: None,
    }
}

fn dummy_context(scenario: Scenario, shifts: Vec<Shift>, workers: Vec<Worker>) -> Arc<ScheduleContext> {
    Arc::new(ScheduleContext {
        shifts: Arc::new(shifts),
        workers: Arc::new(workers),
        ecology: WorkforceEcology::new(),
        rng_seed: 42,
        observatory: Arc::new(Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario: Some(scenario),
    })
}

// Test F: Domain routing test
#[test]
fn test_f_domain_routing_unknown_rejected() {
    let scenario = Scenario {
        domain: None, // UNKNOWN
        planning_horizon_hours: None,
        max_hours_per_worker: None,
        minimum_rest_hours: None,
        leave_requests: None,
    };
    
    let context = dummy_context(scenario, vec![build_shift(1, 0, 8, "RN", None)], vec![build_worker(1, "RN")]);
    let optimizer = ScheduleOptimizer::new(context);
    
    let mut genome = ScheduleGenome { assignments: HashMap::new() };
    genome.assignments.insert(1, 1);

    // Call evaluate from ConstraintModel
    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
    let eval = optimizer.evaluate(&genome, &metric_report);
    
    // Must be heavily penalized
    assert!(!eval.is_valid);
    assert!(eval.fitness <= -1_000_000_000.0);
}

// Test F: INRC domain routing
#[test]
fn test_f_domain_routing_inrc() {
    let scenario = Scenario {
        domain: Some(SchedulingDomain::Inrc),
        planning_horizon_hours: None,
        max_hours_per_worker: None,
        minimum_rest_hours: None,
        leave_requests: None,
    };
    
    let context = dummy_context(scenario, vec![build_shift(1, 0, 8, "RN", None)], vec![build_worker(1, "RN")]);
    let optimizer = ScheduleOptimizer::new(context);
    
    let mut genome = ScheduleGenome { assignments: HashMap::new() };
    genome.assignments.insert(1, 1);

    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
    let eval = optimizer.evaluate(&genome, &metric_report);
    
    // Inrc evaluates successfully
    assert!(eval.is_valid);
    assert!(eval.fitness > 0.0);
}

// Test A, D, E: INRC Rest semantics
#[test]
fn test_inrc_rest_semantics_no_aviation_concepts() {
    let scenario = Scenario {
        domain: Some(SchedulingDomain::Inrc),
        planning_horizon_hours: None,
        max_hours_per_worker: None,
        minimum_rest_hours: Some(12),
        leave_requests: None,
    };
    
    let context = dummy_context(
        scenario, 
        vec![
            build_shift(1, 0, 8, "RN", None),
            build_shift(2, 23, 8, "RN", None), // 15h gap
        ],
        vec![build_worker(1, "RN")]
    );

    let optimizer = ScheduleOptimizer::new(context.clone());
    let mut genome = ScheduleGenome { assignments: HashMap::new() };
    genome.assignments.insert(1, 1);
    genome.assignments.insert(2, 1);

    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
    let eval = optimizer.evaluate(&genome, &metric_report);
    
    // No rest violations, since gap (15h) >= minimum_rest (12h)
    assert_eq!(eval.rest_violations, 0);

    // Explicitly verify the constraint engine does NOT penalize for missing pairing/Captain.
    use ultracrew::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};
    let evaluator = InrcConstraintEvaluator::new(context);
    let report = evaluator.evaluate(&genome);

    // The fitness shouldn't have the severe -5000 pairing penalties
    assert!(report.fitness > 0.0);
}
