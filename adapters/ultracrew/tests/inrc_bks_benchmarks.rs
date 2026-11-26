use std::path::PathBuf;
use std::sync::Arc;
use coralys_core::{DecisionPlugin, ReasoningEngine};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::decision_plugin::{
    get_inrc_bks_entry, CANONICAL_INRC_BKS_TABLE, CoralysInrcEngine, InrcDecisionPlugin,
    InrcOptimizationConfig,
};
use ultracrew::inrc::history::extract_next_history;
use ultracrew::inrc::optimization::{InrcContext, InrcGenome};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[test]
fn test_canonical_inrc_bks_table_integrity() {
    assert_eq!(
        CANONICAL_INRC_BKS_TABLE.len(),
        28,
        "Canonical INRC-II BKS table must contain all 28 published competition instances"
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
            "Fleet size {} must have 2 4-week benchmark configurations",
            n
        );

        let w8_entries: Vec<_> = CANONICAL_INRC_BKS_TABLE
            .iter()
            .filter(|e| e.num_nurses == n && e.num_weeks == 8)
            .collect();
        assert_eq!(
            w8_entries.len(),
            2,
            "Fleet size {} must have 2 8-week benchmark configurations",
            n
        );
    }

    // Every entry must have valid bounds
    for entry in CANONICAL_INRC_BKS_TABLE {
        assert!(entry.best_known_lb > 0, "LB must be positive");
        assert!(entry.best_known_ub >= entry.best_known_lb, "UB must be >= LB");
        assert_eq!(entry.week_indices.len(), entry.num_weeks, "Week count mismatch");
    }
}

#[test]
fn test_inrc_multi_week_decision_plugin_benchmarking() {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data");

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
        population_size: 400,
        generation_limit: 250,
        seed: Some(42),
    };

    let mut cumulative_penalty = 0i32;
    let mut all_feasible = true;

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
        assert!(!proposals.is_empty());

        let next_state = plugin
            .simulate(&plugin.current_state(), &proposals[0])
            .expect("Simulation failed");

        let eval = plugin.evaluate(&next_state);
        let hard_violations = eval.hard_constraint_violations.len();
        if hard_violations > 0 {
            all_feasible = false;
        }
        cumulative_penalty += next_state.total_soft_penalty;

        println!(
            "Week {} (WD-{}) -> Hard Violations: {}, Soft Penalty: {}",
            week_num, wd_idx, hard_violations, next_state.total_soft_penalty
        );

        let genome = InrcGenome {
            bits: next_state.genome_bits.clone(),
        };
        current_history = extract_next_history(&context, &genome);
    }

    println!(
        "Final Multi-Week Result for {}: Total Soft Penalty = {}, BKS UB = {}, Feasible = {}",
        entry.instance_code, cumulative_penalty, entry.best_known_ub, all_feasible
    );

    assert!(
        all_feasible,
        "All weeks must satisfy 100% statutory hard constraints under DecisionPlugin"
    );
}
