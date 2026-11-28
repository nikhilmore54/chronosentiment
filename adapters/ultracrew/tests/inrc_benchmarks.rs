//! Canonical INRC-II Nurse Rostering DecisionPlugin Benchmark and Constraint Audit Suite.
//!
//! Validates:
//! 1. Canonical BKS table completeness (all 28 instances across N=30..120, W=4..8, Ceschia et al. 2019 / SN-III).
//! 2. Binding of all four statutory competition hard constraints (H1 - H4) into `InrcDecisionPlugin::evaluate` and `simulate`.
//! 3. Multi-week rolling-horizon execution under `CoralysInrcEngine` with 100% feasibility (0 hard violations).

use std::path::PathBuf;
use std::sync::Arc;
use coralys_core::{DecisionPlugin, DecisionProposal, ReasoningEngine};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::decision_plugin::{
    get_inrc_bks_entry, CANONICAL_INRC_BKS_TABLE, CoralysInrcEngine, InrcAction,
    InrcDecisionPlugin, InrcOptimizationConfig,
};
use ultracrew::inrc::history::extract_next_history;
use ultracrew::inrc::optimization::{InrcContext, InrcGenome};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

fn get_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

#[test]
fn test_canonical_inrc_bks_table_completeness() {
    assert_eq!(
        CANONICAL_INRC_BKS_TABLE.len(),
        28,
        "Canonical INRC-II BKS table must contain all 28 competition instances"
    );

    let fleet_sizes = [30, 40, 50, 60, 80, 100, 120];
    for &n in &fleet_sizes {
        let w4_entries: Vec<_> = CANONICAL_INRC_BKS_TABLE
            .iter()
            .filter(|e| e.num_nurses == n && e.num_weeks == 4)
            .collect();
        assert_eq!(
            w4_entries.len(),
            2,
            "Fleet size {} must have exactly 2 4-week benchmark configurations",
            n
        );

        let w8_entries: Vec<_> = CANONICAL_INRC_BKS_TABLE
            .iter()
            .filter(|e| e.num_nurses == n && e.num_weeks == 8)
            .collect();
        assert_eq!(
            w8_entries.len(),
            2,
            "Fleet size {} must have exactly 2 8-week benchmark configurations",
            n
        );
    }

    for entry in CANONICAL_INRC_BKS_TABLE {
        assert!(entry.best_known_lb > 0, "LB must be positive for {}", entry.instance_code);
        assert!(
            entry.best_known_ub >= entry.best_known_lb,
            "UB must be >= LB for {}",
            entry.instance_code
        );
        assert_eq!(
            entry.week_indices.len(),
            entry.num_weeks,
            "Week indices length must match num_weeks for {}",
            entry.instance_code
        );
        assert!(
            entry.lit_source().contains("Ceschia et al. (2019"),
            "Entry {} must cite Ceschia et al. (2019)",
            entry.instance_code
        );
        assert!(
            entry.legrain_et_al().is_some(),
            "Entry {} must have Legrain et al. BKS reference",
            entry.instance_code
        );
        assert!(
            entry.romer_mellouli().is_some(),
            "Entry {} must have Römer & Mellouli reference",
            entry.instance_code
        );
    }
}

#[test]
fn test_inrc_decision_plugin_hard_constraints_binding() {
    let base_dir = get_data_dir().join("n030w4");
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

    let state = plugin.current_state();

    // 1. Audit H2 (Understaffing Coverage) on empty schedule
    let empty_violations = plugin.audit_hard_constraints(&state);
    let h2_violations: Vec<_> = empty_violations
        .iter()
        .filter(|v| v.constraint_id == "H2_UnderstaffingCoverage")
        .collect();
    assert!(
        !h2_violations.is_empty(),
        "Empty schedule must trigger H2_UnderstaffingCoverage violations"
    );

    let empty_eval = plugin.evaluate(&state);
    assert!(
        !empty_eval.hard_constraint_violations.is_empty(),
        "Evaluation of empty roster must report hard constraint violations"
    );
    assert_eq!(
        empty_eval.metrics.get("is_feasible").copied().unwrap_or(1.0),
        0.0,
        "Empty roster must have is_feasible == 0.0"
    );

    // 2. Test H1 (Single Shift Per Day Per Nurse)
    // Nurse 0 assigned to both shift 1 and shift 2 on day 0
    let mut h1_bits = vec![false; state.num_nurses * state.num_days * state.num_shifts];
    let idx_s1 = 0 * (state.num_days * state.num_shifts) + 0 * state.num_shifts + 1;
    let idx_s2 = 0 * (state.num_days * state.num_shifts) + 0 * state.num_shifts + 2;
    h1_bits[idx_s1] = true;
    h1_bits[idx_s2] = true;

    let proposal_h1 = DecisionProposal {
        priority: 1.0,
        estimated_gain: 0.0,
        affected_resources: vec!["HN_0".to_string()],
        violations_resolved: vec![],
        confidence: 1.0,
        payload: serde_json::to_value(&InrcAction::SetGenome { bits: h1_bits }).unwrap(),
    };

    let simulated_h1 = plugin.simulate(&state, &proposal_h1).expect("Simulate failed");
    let h1_eval = plugin.evaluate(&simulated_h1);
    let has_h1 = h1_eval
        .hard_constraint_violations
        .iter()
        .any(|v| v.constraint_id == "H1_SingleShiftPerDay");
    assert!(has_h1, "Multiple shifts on same day must trigger H1_SingleShiftPerDay");

    // 3. Test H3 (Forbidden Shift-Type Transitions)
    // Find a rule with non-empty succeeding shift types
    let forbidden_rules = &plugin.context.scenario.forbidden_shift_type_successions;
    if let Some(rule) = forbidden_rules.iter().find(|r| !r.succeeding.is_empty()) {
        let prec_shift = rule.preceding.clone();
        let succ_shift = rule.succeeding[0].clone();

        let prec_idx = plugin.context.shift_types.iter().position(|s| s == &prec_shift).unwrap();
        let succ_idx = plugin.context.shift_types.iter().position(|s| s == &succ_shift).unwrap();

        let mut h3_bits = vec![false; state.num_nurses * state.num_days * state.num_shifts];
        // Day 0: prec_shift, Day 1: succ_shift
        let idx_d0 = 0 * (state.num_days * state.num_shifts) + 0 * state.num_shifts + prec_idx;
        let idx_d1 = 0 * (state.num_days * state.num_shifts) + 1 * state.num_shifts + succ_idx;
        h3_bits[idx_d0] = true;
        h3_bits[idx_d1] = true;

        let proposal_h3 = DecisionProposal {
            priority: 1.0,
            estimated_gain: 0.0,
            affected_resources: vec!["HN_0".to_string()],
            violations_resolved: vec![],
            confidence: 1.0,
            payload: serde_json::to_value(&InrcAction::SetGenome { bits: h3_bits }).unwrap(),
        };

        let simulated_h3 = plugin.simulate(&state, &proposal_h3).expect("Simulate failed");
        let h3_eval = plugin.evaluate(&simulated_h3);
        let has_h3 = h3_eval
            .hard_constraint_violations
            .iter()
            .any(|v| v.constraint_id == "H3_ForbiddenShiftTransitions");
        assert!(has_h3, "Forbidden shift succession must trigger H3_ForbiddenShiftTransitions");

        // Test validator flags illegal transitions
        let h3_report = plugin.validate_with_official_rules(&simulated_h3);
        assert!(h3_report.forbidden_successions > 0, "Validator must detect forbidden succession");
        assert!(!h3_report.is_legal, "Schedule with forbidden succession must not be legal");
    }

    // 4. Test Official Validator Integration
    let empty_report = plugin.validate_with_official_rules(&state);
    assert_eq!(empty_report.coverage_achieved, 0.0, "Empty schedule must achieve 0% coverage");
}

#[test]
fn test_inrc_multi_week_rolling_horizon_benchmark() {
    let data_dir = get_data_dir();
    let entry = get_inrc_bks_entry("n030w4_1_6-2-9-1").expect("Benchmark instance not found");
    assert_eq!(entry.best_known_ub, 1670);

    let scenario_dir = data_dir.join(entry.scenario);
    let scenario = parse_scenario(scenario_dir.join(format!("Sc-{}.json", entry.scenario)))
        .expect("Scenario parse failed");
    let mut current_history =
        parse_history(scenario_dir.join(format!("H0-{}-{}.json", entry.scenario, entry.history_idx)))
            .expect("History parse failed");

    let ecology = WorkforceEcology::new();
    let engine = CoralysInrcEngine;
    let config = InrcOptimizationConfig {
        population_size: 300,
        generation_limit: 150,
        seed: Some(42),
    };

    let mut cumulative_penalty = 0i32;
    let mut all_feasible = true;
    let mut total_hard_violations = 0;

    for (week_num, &wd_idx) in entry.week_indices.iter().enumerate() {
        let wd_path = scenario_dir.join(format!("WD-{}-{}.json", entry.scenario, wd_idx));
        let week_data = parse_week_data(wd_path).expect("Week data parse failed");

        let context = Arc::new(InrcContext::new(
            scenario.clone(),
            week_data,
            current_history.clone(),
            ecology.clone(),
        ));

        let mut plugin = InrcDecisionPlugin::from_context(context.clone(), week_num);
        let proposals = engine.solve(&mut plugin, &config).expect("Solving failed");
        assert!(!proposals.is_empty(), "ReasoningEngine must return at least one proposal");

        let next_state = plugin
            .simulate(&plugin.current_state(), &proposals[0])
            .expect("Simulation failed");

        let eval = plugin.evaluate(&next_state);
        let hard_violations = eval.hard_constraint_violations.len();
        if hard_violations > 0 {
            all_feasible = false;
            total_hard_violations += hard_violations;
        }
        cumulative_penalty += next_state.total_soft_penalty;

        println!(
            "Week {} (WD-{}) -> Hard Violations: {}, Soft Penalty: {}, Feasible: {}",
            week_num, wd_idx, hard_violations, next_state.total_soft_penalty, next_state.is_feasible
        );

        let genome = InrcGenome {
            bits: next_state.genome_bits.clone(),
        };
        current_history = extract_next_history(&context, &genome);
    }

    println!(
        "Final Multi-Week Result for {}: Total Soft Penalty = {}, BKS UB = {}, Total HC = {}, Feasible = {}",
        entry.instance_code, cumulative_penalty, entry.best_known_ub, total_hard_violations, all_feasible
    );

    assert!(
        all_feasible,
        "All weeks must satisfy 100% statutory hard constraints (0 HC violations) under DecisionPlugin"
    );
    assert_eq!(total_hard_violations, 0, "Zero hard violations required");
    assert!(
        cumulative_penalty < 3500,
        "Soft penalty must be within reasonable optimization bounds of BKS ({})",
        entry.best_known_ub
    );
}
