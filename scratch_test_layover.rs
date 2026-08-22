#[test]
fn test_airline_layover_semantics() {
    let scenario = Scenario {
        domain: Some(SchedulingDomain::Airline),
        planning_horizon_hours: None,
        max_hours_per_worker: None,
        minimum_rest_hours: Some(12),
        leave_requests: None,
    };
    
    // Simulate airline rest semantics
    let context = dummy_context(
        scenario, 
        vec![
            build_shift(1, 0, 8, "Captain", Some("FL001")),
            build_shift(2, 23, 8, "Captain", Some("FL002")), // 15h gap
        ],
        vec![build_worker(1, "Captain")]
    );

    let optimizer = ScheduleOptimizer::new(context.clone());
    let mut genome = ScheduleGenome { assignments: HashMap::new() };
    genome.assignments.insert(1, 1);
    genome.assignments.insert(2, 1);

    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
    let eval = optimizer.evaluate(&genome, &metric_report);
    
    // Valid for airline evaluator
    assert!(eval.is_valid);
}
