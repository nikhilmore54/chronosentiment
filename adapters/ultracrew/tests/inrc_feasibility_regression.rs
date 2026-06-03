use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::ecology::WorkforceEcology;
use coralys_moga::engine::EvolutionEngine;
use coralys_moga::config::EvolutionConfig;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_inrc_feasibility_regression() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("n030w4");
    
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    
    // Ecology
    let mut ecology = WorkforceEcology::new();
    
    let context = InrcContext::new(scenario, week_data, history, ecology);
    
    let optimizer = InrcOptimizer {
        context: Arc::new(context),
    };
    
    let engine = EvolutionEngine::new(
        optimizer.clone(),
        optimizer.clone(),
        optimizer.clone(),
        optimizer.clone(),
    );
    
    let config = EvolutionConfig {
        population_size: 500,
        generation_limit: 300,
        seed: Some(42),
        ..Default::default()
    };
    
    // Just run and verify the best is feasible
    let result = engine.run_ga_evolution(config);
    
    let best = &result.global_best;
    println!("Best Feasibility: Fit={} | Cov={} Skills={} OneShift={} Succ={}", 
        best.fitness,
        best.hc_coverage,
        best.hc_skills,
        best.hc_one_shift_per_day,
        best.hc_forbidden_successions
    );

    // Export the official solution
    let out_path = base_dir.join("sol-n030w4-0.txt");
    ultracrew::inrc::exporter::export_inrc_solution(&best.genome, optimizer.context.clone(), 0, &out_path).unwrap();

    // F.2C.5: Validator Penalty Attribution
    println!("\n| Penalty Type       | Score |");
    println!("| ------------------ | ----- |");
    println!("| Assignments        | {} |", best.soft_report.assignment_penalty);
    println!("| Consecutive Work   | {} |", best.soft_report.work_streak_penalty);
    println!("| Days Off           | {} |", best.soft_report.day_off_penalty);
    println!("| Weekends           | {} |", best.soft_report.weekend_penalty);
    println!("| Preferences        | {} |", best.soft_report.preferences_penalty);
    println!("| Optimal Coverage   | {} |", best.soft_report.optimal_coverage_penalty);

    // Establish Baseline Gap
    println!("\n| Instance        | Coralys | Best Known | Gap % |");
    println!("| --------------- | ------- | ---------- | ----- |");
    let coralys_score = 5910; // From Validator for now until we support everything! Actually, let's print our INTERNAL total penalty since we want to see it converge!
    println!("| 030-4-1-6-2-9-1 | {}     | 1670       | {:.1}%|", 
        best.soft_report.total_penalty,
        ((best.soft_report.total_penalty as f64 - 1670.0) / 1670.0) * 100.0);

    assert!(best.is_feasible(), "Failed to reach 0 hard constraints. Best: {:?}", best);
}
