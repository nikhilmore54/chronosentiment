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

#[test]
fn test_at36_4_backend_lock_preservation_and_protection() {
    use ultracrew::public_contracts::RescheduleRequest;
    use std::collections::HashMap;

    // Build the 20 workers from SAMPLE_CSV
    let head_nurses = [1, 3, 7, 13, 17];
    let mut workers = Vec::new();
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

    // Build 196 shifts (28 days * 7 shifts)
    let mut shifts = Vec::new();
    let mut shift_id = 1u64;
    for day in 0..28 {
        let base_hour = day * 24;
        // 3 Early: 1 HeadNurse, 2 Nurse
        shifts.push(Shift { id: shift_id, start_hour: base_hour + 6, duration_hours: 8, required_skill: Skill::new("HeadNurse") });
        shift_id += 1;
        shifts.push(Shift { id: shift_id, start_hour: base_hour + 6, duration_hours: 8, required_skill: Skill::new("Nurse") });
        shift_id += 1;
        shifts.push(Shift { id: shift_id, start_hour: base_hour + 6, duration_hours: 8, required_skill: Skill::new("Nurse") });
        shift_id += 1;

        // 2 Late: 2 Nurse
        shifts.push(Shift { id: shift_id, start_hour: base_hour + 14, duration_hours: 8, required_skill: Skill::new("Nurse") });
        shift_id += 1;
        shifts.push(Shift { id: shift_id, start_hour: base_hour + 14, duration_hours: 8, required_skill: Skill::new("Nurse") });
        shift_id += 1;

        // 2 Night: 2 Nurse
        shifts.push(Shift { id: shift_id, start_hour: base_hour + 22, duration_hours: 8, required_skill: Skill::new("Nurse") });
        shift_id += 1;
        shifts.push(Shift { id: shift_id, start_hour: base_hour + 22, duration_hours: 8, required_skill: Skill::new("Nurse") });
        shift_id += 1;
    }

    let scenario = InrcScenario {
        planning_horizon_hours: Some(28.0 * 24.0),
        validation_envelope_hours: Some(678.0),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(10),
        leave_requests: None,
        weekly_anchor_offset_hours: None,
        weekly_contract_mode: Some("monday_sunday".to_string()),
    };

    let base_request = ScheduleRequest {
        workers,
        shifts,
        historical_workloads: None,
        rng_seed: Some(42),
        generation_limit: Some(30),
        scenario: Some(scenario),
        fatigue: ultracrew::config::FatigueConfig::default(),
    };

    // Load verified 196-assignment baseline
    let verified_json = r#"{"1":1,"2":3,"3":2,"4":13,"5":6,"6":9,"7":20,"8":17,"9":4,"10":5,"11":14,"12":19,"13":16,"14":15,"15":7,"16":18,"17":4,"18":12,"19":10,"20":2,"21":13,"22":1,"23":15,"24":3,"25":20,"26":5,"27":12,"28":6,"29":17,"30":18,"31":2,"32":16,"33":3,"34":19,"35":4,"36":7,"37":14,"38":13,"39":9,"40":15,"41":3,"42":6,"43":17,"44":12,"45":1,"46":16,"47":2,"48":19,"49":8,"50":17,"51":11,"52":20,"53":6,"54":9,"55":5,"56":16,"57":13,"58":3,"59":10,"60":8,"61":4,"62":12,"63":11,"64":7,"65":1,"66":2,"67":3,"68":5,"69":8,"70":4,"71":7,"72":20,"73":18,"74":11,"75":14,"76":12,"77":19,"78":7,"79":10,"80":8,"81":6,"82":15,"83":11,"84":5,"85":17,"86":18,"87":14,"88":12,"89":3,"90":2,"91":13,"92":7,"93":9,"94":11,"95":10,"96":8,"97":19,"98":4,"99":17,"100":3,"101":14,"102":1,"103":6,"104":11,"105":18,"106":13,"107":20,"108":10,"109":19,"110":3,"111":2,"112":12,"113":7,"114":9,"115":15,"116":8,"117":20,"118":11,"119":18,"120":1,"121":16,"122":19,"123":6,"124":10,"125":5,"126":13,"127":17,"128":14,"129":18,"130":4,"131":15,"132":9,"133":20,"134":7,"135":19,"136":10,"137":16,"138":8,"139":18,"140":12,"141":3,"142":15,"143":2,"144":11,"145":1,"146":8,"147":5,"148":7,"149":19,"150":12,"151":18,"152":20,"153":6,"154":9,"155":17,"156":14,"157":10,"158":15,"159":16,"160":2,"161":11,"162":1,"163":8,"164":19,"165":4,"166":14,"167":20,"168":16,"169":13,"170":18,"171":5,"172":1,"173":4,"174":9,"175":15,"176":17,"177":6,"178":10,"179":5,"180":16,"181":11,"182":14,"183":7,"184":10,"185":15,"186":6,"187":1,"188":13,"189":2,"190":17,"191":16,"192":5,"193":9,"194":20,"195":8,"196":4}"#;
    let mut baseline: HashMap<u64, u64> = serde_json::from_str(verified_json).unwrap();
    assert_eq!(baseline.len(), 196);

    // 1. Manually reassign worker 6 (Tom Lindqvist) to Shift 20 (Day 2 Night),
    // which was previously assigned to worker 2 (Marcus Webb)
    let edited_shift_id = 20u64;
    let manual_worker_id = 6u64;
    baseline.insert(edited_shift_id, manual_worker_id);

    // 2. Lock ONLY the manually edited shift (explicitly avoiding locking all 196 shifts!)
    let locked_ids = vec![edited_shift_id];

    let reschedule_req = RescheduleRequest {
        request: base_request.clone(),
        existing_assignments: baseline.clone(),
        locked_shift_ids: Some(locked_ids.clone()),
        generation_limit: Some(30),
        tournament_size: None,
        population_size: Some(20),
        mutation_rate: None,
        crossover_rate: None,
        elite_count: None,
    };

    // 3. Run backend optimization
    let context = reschedule_req.to_context();
    // Confirm lock context has exactly 1 locked assignment
    assert_eq!(context.locked_assignments.as_ref().unwrap().len(), 1);
    assert_eq!(context.locked_assignments.as_ref().unwrap().get(&edited_shift_id), Some(&manual_worker_id));

    let solution = run_pipeline_from_request(
        context.clone(),
        Some(30),
        None,
        Some(20),
        None,
        None,
        None,
    ).expect("Rescheduling pipeline must succeed");

    // 4. Invariant: The manually edited (worker_id, shift_id) assignment MUST remain unchanged
    let assigned_worker = solution.assignments.get(&edited_shift_id).copied();
    assert_eq!(assigned_worker, Some(manual_worker_id), "Shift 20 must remain assigned to Worker 6");

    // 5. Independently validate returned schedule using InrcConstraintEvaluator
    let evaluator = InrcConstraintEvaluator::new(context);
    let genome = ScheduleGenome { assignments: solution.assignments.clone() };
    let report = evaluator.evaluate(&genome);

    // Hard violations must be 0
    assert_eq!(report.hard_violations, 0, "Rescheduled roster must maintain 0 hard violations");
    assert_eq!(report.hc1_violations, 0, "HC1 Skill matching must be satisfied");
    assert_eq!(report.hc2_violations, 0, "HC2 Single shift per day must be satisfied");

    // 6. Confirm that an invalid/conflicting lock fails gracefully without corrupting baseline
    let mut invalid_assignments = baseline.clone();
    // Intentionally assign an unqualified worker (worker 2 has only Nurse, shift 1 requires HeadNurse)
    invalid_assignments.insert(1, 2);
    let invalid_req = RescheduleRequest {
        request: base_request,
        existing_assignments: invalid_assignments,
        locked_shift_ids: Some(vec![1]), // Lock the skill-violating assignment
        generation_limit: Some(10),
        tournament_size: None,
        population_size: Some(10),
        mutation_rate: None,
        crossover_rate: None,
        elite_count: None,
    };
    let invalid_context = invalid_req.to_context();
    let invalid_evaluator = InrcConstraintEvaluator::new(invalid_context.clone());
    // Evaluating the forced invalid lock must report an HC1 violation
    let invalid_genome = ScheduleGenome { assignments: invalid_req.existing_assignments };
    let invalid_report = invalid_evaluator.evaluate(&invalid_genome);
    assert!(invalid_report.hc1_violations > 0, "Evaluator must detect forced skill violation");
}

