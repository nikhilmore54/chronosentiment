use ultracrew::models::{Shift, Skill, Worker};
use ultracrew::public_contracts::{InrcScenario, ScheduleRequest};
use ultracrew::pipeline::run_pipeline_from_request;
use ultracrew::constraint_engine::InrcConstraintEvaluator;
use ultracrew::constraint_engine::DomainConstraintEvaluator;
use ultracrew::optimization::ScheduleGenome;
use std::time::Instant;

#[test]
fn test_realistic_20_nurse_196_shift_roster() {
    let mut workers = Vec::new();
    // Exact mapping matching SAMPLE_CSV in WorkflowTypes.ts:
    // 1: Sarah_Chen (HeadNurse;Nurse)
    // 2: Marcus_Webb (Nurse)
    // 3: Priya_Nair (HeadNurse;Nurse)
    // 4: James_Okafor (Nurse)
    // 5: Elena_Vasquez (Nurse)
    // 6: Tom_Lindqvist (Nurse)
    // 7: Aisha_Diallo (HeadNurse)
    // 8: Ravi_Sharma (Nurse)
    // 9: Fatima_Al-Hassan (Nurse)
    // 10: Daniel_Park (Nurse)
    // 11: Ingrid_Sorensen (Nurse)
    // 12: Carlos_Mendez (Nurse)
    // 13: Yuki_Tanaka (HeadNurse;Nurse)
    // 14: Kwame_Asante (Nurse)
    // 15: Lena_Hoffmann (Nurse)
    // 16: Amara_Osei (Nurse)
    // 17: Patrick_Brennan (HeadNurse)
    // 18: Nadia_Petrov (Nurse)
    // 19: Soren_Andersen (Nurse)
    // 20: Mei_Lin_Zhou (Nurse)
    let head_nurses = [1, 3, 7, 13, 17];
    for i in 1..=20 {
        let skills = if head_nurses.contains(&i) {
            if i == 7 || i == 17 {
                vec![Skill::new("HeadNurse")]
            } else {
                vec![Skill::new("HeadNurse"), Skill::new("Nurse")]
            }
        } else {
            vec![Skill::new("Nurse")]
        };
        workers.push(Worker { id: i, skills });
    }

    let mut shifts = Vec::new();
    let mut shift_id = 1u64;
    for day in 0..28 {
        let base_hour = day * 24;
        // 3 Early: 1 HeadNurse, 2 Nurse
        shifts.push(Shift {
            id: shift_id,
            start_hour: base_hour + 6,
            duration_hours: 8,
            required_skill: Skill::new("HeadNurse"),
        });
        shift_id += 1;
        shifts.push(Shift {
            id: shift_id,
            start_hour: base_hour + 6,
            duration_hours: 8,
            required_skill: Skill::new("Nurse"),
        });
        shift_id += 1;
        shifts.push(Shift {
            id: shift_id,
            start_hour: base_hour + 6,
            duration_hours: 8,
            required_skill: Skill::new("Nurse"),
        });
        shift_id += 1;

        // 2 Late: 2 Nurse
        shifts.push(Shift {
            id: shift_id,
            start_hour: base_hour + 14,
            duration_hours: 8,
            required_skill: Skill::new("Nurse"),
        });
        shift_id += 1;
        shifts.push(Shift {
            id: shift_id,
            start_hour: base_hour + 14,
            duration_hours: 8,
            required_skill: Skill::new("Nurse"),
        });
        shift_id += 1;

        // 2 Night: 2 Nurse
        shifts.push(Shift {
            id: shift_id,
            start_hour: base_hour + 22,
            duration_hours: 8,
            required_skill: Skill::new("Nurse"),
        });
        shift_id += 1;
        shifts.push(Shift {
            id: shift_id,
            start_hour: base_hour + 22,
            duration_hours: 8,
            required_skill: Skill::new("Nurse"),
        });
        shift_id += 1;
    }
    assert_eq!(shifts.len(), 196);

    let scenario = InrcScenario {
        planning_horizon_hours: Some(28.0 * 24.0),
        validation_envelope_hours: Some(678.0),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(10),
        leave_requests: None,
        weekly_anchor_offset_hours: None,
        weekly_contract_mode: Some("monday_sunday".to_string()),
    };

    let request = ScheduleRequest {
        workers,
        shifts,
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: Some(100),
        scenario: Some(scenario),
        fatigue: ultracrew::config::FatigueConfig::default(),
    };

    let context = request.to_context();
    let start = Instant::now();
    let solution = run_pipeline_from_request(
        context.clone(),
        Some(100),
        None,
        None,
        None,
        None,
        None,
    ).expect("optimization failed");
    let elapsed = start.elapsed();

    println!("Optimized 196 shifts in {:?}", elapsed);
    println!("Assignments count: {}", solution.assignments.len());
    println!("Hard violations: {}", solution.hard_violations);
    println!("Rest violations: {}", solution.rest_violations);
    println!("Fairness penalty: {}", solution.fairness_penalty);

    let evaluator = InrcConstraintEvaluator::new(context);
    let genome = ScheduleGenome { assignments: solution.assignments.clone() };
    let report = evaluator.evaluate(&genome);

    println!("Report hard violations: {}", report.hard_violations);
    println!("Report rest violations: {}", report.rest_violations);
    println!("Report HC1: {}", report.hc1_violations);
    println!("Report HC2: {}", report.hc2_violations);
    println!("Report HC3: {}", report.hc3_violations);
}

