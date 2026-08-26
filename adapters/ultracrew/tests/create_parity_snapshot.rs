use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::EvolutionEngine;
use std::path::PathBuf;
use std::sync::Arc;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[test]
fn test_create_parity_snapshot() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");

    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();

    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario, week_data, history, ecology);
    let optimizer = InrcOptimizer {
        context: Arc::new(context),
    };

    let mut engine = EvolutionEngine::new(
        optimizer.clone(),
        optimizer.clone(),
        optimizer.clone(),
        optimizer.clone(),
    );

    let config = EvolutionConfig {
        population_size: 20,
        generation_limit: 50,
        seed: Some(42),
        ..Default::default()
    };

    let result = engine.run_ga_evolution(config).expect("GA failed");
    let best = &result.global_best;

    println!("Days Off: {}", best.soft_report.day_off_penalty);
    println!("Preferences: {}", best.soft_report.preferences_penalty);
    println!("Weekends: {}", best.soft_report.weekend_penalty);
    println!("Consecutive: {}", best.soft_report.work_streak_penalty);
    println!("Coverage: {}", best.soft_report.optimal_coverage_penalty);

    std::fs::write(
        base_dir.join("new_frozen_genome.json"),
        serde_json::to_string(&best.genome.bits).unwrap(),
    )
    .unwrap();
}
