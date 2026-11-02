//! AT-36.4 final operator check: every chromosome-mutating path must preserve locks
//! across many generations (not only seed + crossover).

use coralys_core::operators::{OperatorBudget, RepairOperator};
use coralys_moga::traits::{CrossoverOperator, GenomeFactory, MutationOperator};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::sync::Arc;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::models::{Shift, Skill, Worker};
use ultracrew::optimization::{
    Observatory, ScheduleContext, ScheduleGenome, ScheduleOptimizer, SeededScheduleFactory,
};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::repair::{InrcConstraintModel, ReassignRepairOperator, RestConstraint, SkillConstraint};

fn skill(name: &str) -> Skill {
    Skill::new(name)
}

fn test_context(locked: HashMap<u64, u64>) -> Arc<ScheduleContext> {
    let workers = vec![
        Worker {
            id: 1,
            skills: vec![skill("RN")],
        },
        Worker {
            id: 2,
            skills: vec![skill("RN")],
        },
        Worker {
            id: 3,
            skills: vec![skill("RN")],
        },
    ];
    // Non-overlapping shifts so unlocked genes can mutate freely
    let shifts = vec![
        Shift {
            id: 10,
            start_hour: 0,
            duration_hours: 8,
            required_skill: skill("RN"),
        },
        Shift {
            id: 20,
            start_hour: 24,
            duration_hours: 8,
            required_skill: skill("RN"),
        },
        Shift {
            id: 30,
            start_hour: 48,
            duration_hours: 8,
            required_skill: skill("RN"),
        },
        Shift {
            id: 40,
            start_hour: 72,
            duration_hours: 8,
            required_skill: skill("RN"),
        },
    ];
    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology: WorkforceEcology::new(),
        rng_seed: 42,
        observatory: Arc::new(std::sync::Mutex::new(Observatory::new())),
        locked_assignments: Some(locked),
        scenario: None,
        enable_fatigue: false,
        fatigue_weight: 0.0,
        hc3_aware_initialization: false,
        temporal_scarcity_construction: false,
        disable_global_constructor: true,
        constructor_budget_ms: None,
        precomputed_seeds: None,
    })
}

fn assert_locks(genome: &ScheduleGenome, locked: &HashMap<u64, u64>, label: &str) {
    for (&shift_id, &worker_id) in locked {
        assert_eq!(
            genome.assignments.get(&shift_id).copied(),
            Some(worker_id),
            "{label}: locked shift {shift_id} must stay on worker {worker_id}"
        );
    }
}

#[test]
fn at36_4_mutate_and_crossover_preserve_locks_over_many_generations() {
    let locked: HashMap<u64, u64> = HashMap::from([(10, 2)]); // shift 10 → worker 2
    let context = test_context(locked.clone());
    let optimizer = ScheduleOptimizer::new(context);
    let mut rng = StdRng::seed_from_u64(7);

    let mut a = optimizer.create(&mut rng);
    let mut b = optimizer.create(&mut rng);
    assert_locks(&a, &locked, "create_a");
    assert_locks(&b, &locked, "create_b");

    for gen in 0..200 {
        optimizer.mutate(&mut a, &mut rng);
        optimizer.mutate(&mut b, &mut rng);
        assert_locks(&a, &locked, &format!("mutate_a_gen{gen}"));
        assert_locks(&b, &locked, &format!("mutate_b_gen{gen}"));

        let (c1, c2) = optimizer.crossover(&a, &b, &mut rng);
        assert_locks(&c1, &locked, &format!("xover_c1_gen{gen}"));
        assert_locks(&c2, &locked, &format!("xover_c2_gen{gen}"));
        a = c1;
        b = c2;
    }
}

#[test]
fn at36_4_seeded_factory_restamps_locks_on_popped_seed() {
    let locked: HashMap<u64, u64> = HashMap::from([(10, 2)]);
    let context = test_context(locked.clone());
    let base = ScheduleOptimizer::new(context);

    // Tampered seed: locked shift assigned to the wrong worker
    let mut bad = HashMap::new();
    bad.insert(10, 1);
    bad.insert(20, 1);
    bad.insert(30, 3);
    bad.insert(40, 2);

    let factory = SeededScheduleFactory {
        seeds: std::cell::RefCell::new(vec![ScheduleGenome { assignments: bad }]),
        base_optimizer: base,
    };
    let mut rng = StdRng::seed_from_u64(1);
    let genome = factory.create(&mut rng);
    assert_locks(&genome, &locked, "seeded_factory");
}

#[test]
fn at36_4_repair_skips_locked_shifts() {
    let locked: HashMap<u64, u64> = HashMap::from([(102, 1)]);
    let workers = Arc::new(vec![
        Worker {
            id: 1,
            skills: vec![skill("RN")],
        },
        Worker {
            id: 2,
            skills: vec![skill("RN")],
        },
    ]);
    let shifts = Arc::new(vec![
        Shift {
            id: 101,
            start_hour: 8,
            duration_hours: 8,
            required_skill: skill("RN"),
        },
        Shift {
            id: 102,
            start_hour: 16,
            duration_hours: 8,
            required_skill: skill("RN"),
        },
    ]);
    let ctx = Arc::new(ScheduleContext {
        workers,
        shifts,
        ecology: WorkforceEcology::new(),
        rng_seed: 9,
        observatory: Arc::new(std::sync::Mutex::new(Observatory::new())),
        locked_assignments: Some(locked),
        scenario: Some(InrcScenario {
            minimum_rest_hours: Some(10),
            ..Default::default()
        }),
        enable_fatigue: false,
        fatigue_weight: 0.0,
        hc3_aware_initialization: false,
        temporal_scarcity_construction: false,
        disable_global_constructor: true,
        constructor_budget_ms: None,
        precomputed_seeds: None,
    });

    let repair_op = ReassignRepairOperator {
        context: ctx.clone(),
        rng: std::sync::Mutex::new(StdRng::seed_from_u64(9)),
    };
    let model = InrcConstraintModel {
        rest: RestConstraint {
            context: ctx.clone(),
        },
        skill: SkillConstraint {
            context: ctx.clone(),
        },
    };

    let mut genome = ScheduleGenome {
        assignments: HashMap::from([(101, 1), (102, 1)]), // rest violation; 102 locked to 1
    };
    let budget = OperatorBudget {
        max_iterations: 10,
        max_time_ms: 1000,
    };
    let _ = repair_op
        .repair(&mut genome, &model, &budget)
        .expect("repair");
    assert_eq!(
        genome.assignments.get(&102).copied(),
        Some(1),
        "repair must not move locked shift 102"
    );
}
