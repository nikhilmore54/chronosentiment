use coralys_core::{DecisionPlugin, DecisionProposal, ReasoningEngine};
use coralys_moga::FitnessEvaluator;
use rand::SeedableRng;
use std::path::PathBuf;
use ultracrew::inrc::optimization::InrcOptimizer;
use ultracrew::inrc::{
    get_inrc_bks_entry, CoralysInrcEngine, InrcAction, InrcDecisionPlugin, InrcOptimizationConfig,
};

#[test]
fn test_inrc_decision_plugin_lifecycle() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("n030w4");

    let scenario_path = base_dir.join("Sc-n030w4.json");
    let week_data_path = base_dir.join("WD-n030w4-0.json");
    let history_path = base_dir.join("H0-n030w4-0.json");

    // 1. Initialize plugin
    let mut plugin = InrcDecisionPlugin::from_files(
        &scenario_path,
        &week_data_path,
        &history_path,
        0,
    )
    .expect("Plugin initialization failed");

    // 2. Query initial state
    let state = plugin.current_state();
    assert_eq!(state.scenario_id, "n030w4");
    assert_eq!(state.week_index, 0);
    assert_eq!(state.num_nurses, 30);
    assert_eq!(state.num_days, 7);
    assert_eq!(state.num_shifts, 4);

    // Initial all-false genome has coverage violations
    let initial_eval = plugin.evaluate(&state);
    assert!(
        !initial_eval.hard_constraint_violations.is_empty(),
        "Empty roster must have coverage violations"
    );

    // 3. Test InrcAction::AssignShift simulation
    let action_assign = InrcAction::AssignShift {
        nurse_idx: 0,
        day: 0,
        shift_idx: 1, // Day shift
        assigned: true,
    };
    let proposal_assign = DecisionProposal {
        priority: 1.0,
        estimated_gain: 0.0,
        affected_resources: vec!["HN_0".to_string()],
        violations_resolved: vec![],
        confidence: 1.0,
        payload: serde_json::to_value(&action_assign).unwrap(),
    };

    let simulated_state = plugin
        .simulate(&state, &proposal_assign)
        .expect("Simulate assign failed");
    let target_idx = 0 * (7 * 4) + 0 * 4 + 1;
    assert!(simulated_state.genome_bits[target_idx]);

    // 4. Test InrcAction::SwapNursesDay
    let action_swap = InrcAction::SwapNursesDay {
        nurse_a: 0,
        nurse_b: 1,
        day: 0,
    };
    let proposal_swap = DecisionProposal {
        priority: 1.0,
        estimated_gain: 0.0,
        affected_resources: vec!["HN_0".to_string(), "HN_1".to_string()],
        violations_resolved: vec![],
        confidence: 1.0,
        payload: serde_json::to_value(&action_swap).unwrap(),
    };

    let swapped_state = plugin
        .simulate(&simulated_state, &proposal_swap)
        .expect("Simulate swap failed");
    let swapped_target_idx = 1 * (7 * 4) + 0 * 4 + 1;
    assert!(swapped_state.genome_bits[swapped_target_idx]);
    assert!(!swapped_state.genome_bits[target_idx]);

    // 5. Test execute mutating current state
    plugin.execute(&proposal_assign);
    assert!(plugin.current_state().genome_bits[target_idx]);

    // 6. Test ReasoningEngine solving
    let engine = CoralysInrcEngine;
    let config = InrcOptimizationConfig {
        population_size: 200,
        generation_limit: 100,
        seed: Some(42),
    };

    let proposals = engine.solve(&mut plugin, &config).expect("Solving failed");
    assert!(!proposals.is_empty());

    let best_proposal = &proposals[0];
    let next_state = plugin
        .simulate(&plugin.current_state(), best_proposal)
        .expect("Simulation of engine proposal failed");

    let eval = plugin.evaluate(&next_state);
    println!(
        "Engine Evaluation: hard violations = {}, soft penalty = {}",
        eval.hard_constraint_violations.len(),
        next_state.total_soft_penalty
    );

    // 7. Test BKS lookup and gap calculation
    let bks_entry = get_inrc_bks_entry("n030w4_1_6-2-9-1").expect("BKS entry not found");
    assert_eq!(bks_entry.best_known_ub, 1670);
    assert_eq!(bks_entry.num_nurses, 30);
    assert_eq!(bks_entry.num_weeks, 4);

    let gap = next_state.optimality_gap(bks_entry.best_known_ub);
    println!("Optimality gap against BKS: {:.2}%", gap);
}

#[test]
fn test_inverted_chromosome_properties() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("n030w4");

    let scenario_path = base_dir.join("Sc-n030w4.json");
    let week_data_path = base_dir.join("WD-n030w4-0.json");
    let history_path = base_dir.join("H0-n030w4-0.json");

    let plugin = InrcDecisionPlugin::from_files(
        &scenario_path,
        &week_data_path,
        &history_path,
        0,
    )
    .expect("Plugin initialization failed");

    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);

    // 1. Test Inverted Construction Feasibility
    let optimizer = InrcOptimizer { context: plugin.context.clone() };
    for _iter in 0..10 {
        let mut genome = plugin.context.inverted.construct_feasible_genome(&mut rng);
        optimizer.repair_hard_constraints(&mut genome);
        let eval = optimizer.evaluate(&genome, &Default::default());
        assert_eq!(eval.hc_coverage, 0, "Inverted constructor must have 0 coverage violations");
        assert_eq!(eval.hc_skills, 0, "Inverted constructor must have 0 skill violations");
        assert_eq!(eval.hc_one_shift_per_day, 0, "Inverted constructor must have 0 one-shift violations");
    }

    // 2. Test Day-Preserving Crossover Invariance
    let mut p1 = plugin.context.inverted.construct_feasible_genome(&mut rng);
    let mut p2 = plugin.context.inverted.construct_feasible_genome(&mut rng);
    optimizer.repair_hard_constraints(&mut p1);
    optimizer.repair_hard_constraints(&mut p2);
    let (mut c1, mut c2) = plugin.context.inverted.day_preserving_crossover(&p1, &p2, &mut rng);
    optimizer.repair_hard_constraints(&mut c1);
    optimizer.repair_hard_constraints(&mut c2);

    let eval_c1 = optimizer.evaluate(&c1, &Default::default());
    let eval_c2 = optimizer.evaluate(&c2, &Default::default());
    assert_eq!(eval_c1.hc_coverage, 0, "Crossover child 1 must maintain 0 coverage violations");
    assert_eq!(eval_c1.hc_skills, 0, "Crossover child 1 must maintain 0 skill violations");
    assert_eq!(eval_c2.hc_coverage, 0, "Crossover child 2 must maintain 0 coverage violations");
    assert_eq!(eval_c2.hc_skills, 0, "Crossover child 2 must maintain 0 skill violations");
}
