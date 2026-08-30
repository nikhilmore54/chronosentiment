use std::sync::Arc;
use ultracrew::optimization::{ScheduleContext, ScheduleGenome, ScheduleEvaluation};
use ultracrew::models::{Worker, Skill, Shift};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};
use std::collections::HashMap;

fn setup_context(enable_fatigue: bool) -> Arc<ScheduleContext> {
    let mut workers = Vec::new();
    // Worker 1: Fatigued
    workers.push(Worker { id: 1, skills: vec![Skill::new("Pilot")] });
    // Worker 2: Fresh
    workers.push(Worker { id: 2, skills: vec![Skill::new("Pilot")] });

    let mut shifts = Vec::new();
    // One 10-hour shift
    shifts.push(Shift { id: 101, start_hour: 8, duration_hours: 10, required_skill: Skill::new("Pilot")});

    let mut ecology = WorkforceEcology::new();
    // Worker 1 worked 80 hours recently (very fatigued)
    ecology.record_historical_hours(1, 80.0);
    ecology.record_historical_hours(1, 80.0);
    ecology.record_historical_hours(1, 80.0);
    ecology.record_historical_hours(1, 80.0);
    // Worker 2 worked 0 hours (fresh)

    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
        rng_seed: 42,
        observatory: Arc::new(std::sync::Mutex::new(ultracrew::optimization::Observatory::new())),
        locked_assignments: None,
        scenario: None,
        enable_fatigue,
        fatigue_weight: 2.0, // Weight = 2.0
        hc3_aware_initialization: false,
        temporal_scarcity_construction: false,
        disable_global_constructor: false,
        precomputed_seeds: None,
        constructor_budget_ms: None,
    })
}

#[test]
fn test_m2_causal_integration() {
    let context_off = setup_context(false);
    let context_on = setup_context(true);

    let eval_off = InrcConstraintEvaluator::new(context_off);
    let eval_on = InrcConstraintEvaluator::new(context_on);

    // Genome A: Assign shift to fatigued worker 1
    let mut assignments_a = HashMap::new();
    assignments_a.insert(101, 1);
    let genome_a = ScheduleGenome { assignments: assignments_a };

    // Genome B: Assign shift to fresh worker 2
    let mut assignments_b = HashMap::new();
    assignments_b.insert(101, 2);
    let genome_b = ScheduleGenome { assignments: assignments_b };

    // 1. Fatigue OFF, fatigued worker -> baseline score
    let report_off_a = eval_off.evaluate(&genome_a);
    
    // 2. Fatigue OFF, fresh worker -> same relevant baseline behavior
    let report_off_b = eval_off.evaluate(&genome_b);
    
    // In baseline, both should have same fitness (SC1 variance is identical since both assign 1 shift)
    assert_eq!(report_off_a.fitness, report_off_b.fitness, "OFF: Baseline fitness should be identical");
    assert_eq!(report_off_a.fatigue_penalty, 0.0, "OFF: Fatigue penalty must be 0");
    assert_eq!(report_off_b.fatigue_penalty, 0.0, "OFF: Fatigue penalty must be 0");

    // 3. Fatigue ON, fatigued worker -> fatigue penalty applied
    let report_on_a = eval_on.evaluate(&genome_a);
    
    // 4. Fatigue ON, fresh worker -> lower/no fatigue penalty
    let report_on_b = eval_on.evaluate(&genome_b);

    // Worker 1 fatigue = (80 / 40) clamped to 1.0. Shift is 10 hours. Weight 2.0.
    // Penalty = 1.0 * 10 * 2.0 = 20.0
    assert_eq!(report_on_a.fatigue_penalty, 20.0, "ON: Fatigued worker must incur penalty");
    
    // Worker 2 fatigue = 0.0. Penalty = 0.0.
    assert_eq!(report_on_b.fatigue_penalty, 0.0, "ON: Fresh worker must incur 0 penalty");

    // 5. Bit-exact invariant
    assert_eq!(report_on_a.fitness, report_off_a.fitness - report_on_a.fatigue_penalty, "ON must equal OFF - penalty");
    assert_eq!(report_on_b.fitness, report_off_b.fitness - report_on_b.fatigue_penalty, "ON must equal OFF - penalty");

    // 6. Fresh-worker candidate preferred
    assert!(report_on_b.fitness > report_on_a.fitness, "ON: Fresh worker must be preferred over fatigued worker");
}
