use coralys_moga::config::EvolutionConfig;
use ultracrew::constraint_engine::{validate_context, validate_schedule};
use ultracrew::decision_intelligence::{analyze_solution, generate_insights};
use ultracrew::models::{Shift, Skill, Worker};
use ultracrew::pipeline::run_pipeline;
use ultracrew::public_contracts::ScheduleRequest;

#[test]
fn test_dataset_validation() {
    // Valid context
    let workers = vec![
        Worker {
            id: 1,
            skills: vec![Skill::new("Forklift"), Skill::new("GeneralLabor")],
        },
        Worker {
            id: 2,
            skills: vec![Skill::new("Supervisor")],
        },
    ];
    let shifts = vec![
        Shift {
            id: 101,
            start_hour: 8,
            duration_hours: 8,
            required_skill: Skill::new("Forklift"),
        },
        Shift {
            id: 102,
            start_hour: 16,
            duration_hours: 8,
            required_skill: Skill::new("Supervisor"),
        },
    ];
    let request = ScheduleRequest {
        workers,
        shifts,
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: None,
        scenario: None,
    };
    let context = request.to_context();
    assert!(validate_context(&context).is_ok());

    // Invalid context - duplicate worker ID
    let bad_workers = vec![
        Worker {
            id: 1,
            skills: vec![Skill::new("Forklift")],
        },
        Worker {
            id: 1,
            skills: vec![Skill::new("Supervisor")],
        },
    ];
    let bad_request = ScheduleRequest {
        workers: bad_workers,
        shifts: request.shifts.clone(),
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: None,
        scenario: None,
    };
    let bad_context = bad_request.to_context();
    assert!(validate_context(&bad_context).is_err());

    // Invalid context - no worker with required skill
    let bad_shifts = vec![Shift {
        id: 101,
        start_hour: 8,
        duration_hours: 8,
        required_skill: Skill::new("FirstAid"),
    }];
    let bad_request2 = ScheduleRequest {
        workers: request.workers.clone(),
        shifts: bad_shifts,
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: None,
        scenario: None,
    };
    let bad_context2 = bad_request2.to_context();
    assert!(validate_context(&bad_context2).is_err());
}

#[test]
fn test_optimization_and_explanation_pipeline() {
    let workers = vec![
        Worker {
            id: 1,
            skills: vec![Skill::new("Forklift"), Skill::new("GeneralLabor")],
        },
        Worker {
            id: 2,
            skills: vec![Skill::new("Supervisor")],
        },
    ];
    let shifts = vec![
        Shift {
            id: 101,
            start_hour: 8,
            duration_hours: 8,
            required_skill: Skill::new("Forklift"),
        },
        Shift {
            id: 102,
            start_hour: 16,
            duration_hours: 8,
            required_skill: Skill::new("Supervisor"),
        },
    ];
    let request = ScheduleRequest {
        workers,
        shifts,
        historical_workloads: None,
        rng_seed: Some(123),
        generation_limit: None,
        scenario: None,
    };
    let context = request.to_context();

    let config = EvolutionConfig {
        population_size: 10,
        generation_limit: 5,
        seed: Some(123),
        ..Default::default()
    };

    let solution = run_pipeline(context, config).expect("Pipeline should succeed");

    // Verify solution structure
    assert_eq!(solution.assignments.len(), 2);
    assert!(solution.assignments.contains_key(&101));
    assert!(solution.assignments.contains_key(&102));

    // Verify constraint validation on solution
    assert!(validate_schedule(&solution));

    // Verify decision intelligence metrics
    let metrics = analyze_solution(&solution);
    assert!(metrics.contains_key("fitness"));
    assert!(metrics.contains_key("hard_violations"));
    assert_eq!(*metrics.get("hard_violations").unwrap(), 0.0);

    // Verify insights generation
    let insights = generate_insights(&solution);
    assert!(!insights.is_empty());
    assert!(insights
        .iter()
        .any(|s| s.contains("No hard‑constraint violations detected.")));
}

#[test]
fn test_schema_serialization_compliance() {
    let request_json = r#"{
        "workers": [
            { "id": 1, "skills": ["Forklift", "GeneralLabor"] }
        ],
        "shifts": [
            { "id": 101, "start_hour": 8, "duration_hours": 8, "required_skill": "Forklift" }
        ],
        "historical_workloads": {
            "1": [32.5, 40.0]
        },
        "rng_seed": 42
    }"#;
    let request: ScheduleRequest = serde_json::from_str(request_json)
        .expect("Should deserialize valid request JSON matching schema");
    assert_eq!(request.workers.len(), 1);
    assert_eq!(request.shifts.len(), 1);
    assert_eq!(request.rng_seed, Some(42));

    let solution_json = r#"{
        "assignments": {
            "101": 1
        },
        "fitness": 9500.0,
        "hard_violations": 0,
        "fairness_penalty": 10.0,
        "fatigue_penalty": 5.0,
        "rest_violations": 0
    }"#;
    let solution: ultracrew::schedule_solution::ScheduleSolution =
        serde_json::from_str(solution_json)
            .expect("Should deserialize valid solution JSON matching schema");
    assert_eq!(*solution.assignments.get(&101).unwrap(), 1);
    assert_eq!(solution.fitness, 9500.0);
}

#[test]
fn test_constraint_engine_report_details() {
    use std::collections::HashMap;
    use ultracrew::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};
    use ultracrew::optimization::ScheduleGenome;

    let workers = vec![Worker {
        id: 1,
        skills: vec![Skill::new("Forklift")],
    }];
    let shifts = vec![Shift {
        id: 101,
        start_hour: 8,
        duration_hours: 8,
        required_skill: Skill::new("Supervisor"),
    }];
    let request = ScheduleRequest {
        workers,
        shifts,
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: None,
        scenario: None,
    };
    let context = request.to_context();
    let engine = InrcConstraintEvaluator::new(context);

    let mut assignments = HashMap::new();
    assignments.insert(101, 1);
    let genome = ScheduleGenome { assignments };

    let report = engine.evaluate(&genome);
    assert_eq!(report.hc1_violations, 1);
    assert_eq!(report.hard_violations, 1);
    assert!(report.violated_constraints.contains(&"HC1".to_string()));
    assert!(report.satisfied_constraints.contains(&"HC2".to_string()));
    assert!(report.constraint_scores.contains_key("HC1"));
    assert!(*report.constraint_scores.get("HC1").unwrap() > 0.0);
}

#[test]
fn test_recommendation_generation() {
    use std::collections::HashMap;
    use ultracrew::constraint_engine::{DomainConstraintEvaluator, InrcConstraintEvaluator};
    use ultracrew::optimization::ScheduleGenome;
    use ultracrew::recommendation::RecommendationEngine;

    let workers = vec![Worker {
        id: 1,
        skills: vec![Skill::new("Forklift")],
    }];
    let shifts = vec![Shift {
        id: 101,
        start_hour: 8,
        duration_hours: 8,
        required_skill: Skill::new("Supervisor"),
    }];
    let request = ScheduleRequest {
        workers,
        shifts,
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: None,
        scenario: None,
    };
    let context = request.to_context();
    let constraint_engine = InrcConstraintEvaluator::new(context);

    let mut assignments = HashMap::new();
    assignments.insert(101, 1);
    let genome = ScheduleGenome { assignments };

    let report = constraint_engine.evaluate(&genome);
    let rec_engine = RecommendationEngine::new();
    let recs = rec_engine.generate_recommendations(&report);

    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].constraint_id, "HC1");
    assert_eq!(recs[0].severity, "Hard");
    assert!(recs[0]
        .explanation
        .contains("worker does not possess the required skill"));
    assert!(recs[0].recommended_action.contains("Reassign the shifts"));
}

#[test]
fn test_optimizer_telemetry_generation() {
    let workers = vec![Worker {
        id: 1,
        skills: vec![Skill::new("Forklift")],
    }];
    let shifts = vec![Shift {
        id: 101,
        start_hour: 8,
        duration_hours: 8,
        required_skill: Skill::new("Forklift"),
    }];
    let request = ScheduleRequest {
        workers,
        shifts,
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: None,
        scenario: None,
    };
    let context = request.to_context();
    let config = EvolutionConfig {
        population_size: 10,
        generation_limit: 5,
        seed: Some(42),
        ..Default::default()
    };

    let solution = run_pipeline(context, config).expect("Pipeline should run");
    assert!(solution.telemetry.is_some());
    let telemetry = solution.telemetry.unwrap();
    assert_eq!(telemetry.generations.len(), 5);

    for (idx, gen) in telemetry.generations.iter().enumerate() {
        assert_eq!(gen.generation, idx);
        assert!(gen.best_fitness <= 10000.0);
        assert!(gen.average_fitness <= 10000.0);
        // Ensure elapsed time has been recorded (not strictly 0 unless executed instantly, but >= 0)
        assert!(gen.elapsed_time_ms >= 0);
    }
}
