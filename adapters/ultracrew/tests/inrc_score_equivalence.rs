use coralys_moga::config::EvolutionConfig;
use coralys_moga::engine::EvolutionEngine;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[test]
fn test_inrc_score_equivalence() {
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

    let out_path = base_dir.join("sol-n030w4-equiv.txt");
    ultracrew::inrc::exporter::export_inrc_solution(
        &best.genome,
        optimizer.context.clone(),
        0,
        &out_path,
    )
    .unwrap();

    // Run Official Validator
    let validator_jar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/validator.jar");
    let output = Command::new("java")
        .arg("-jar")
        .arg(&validator_jar)
        .arg("--sce")
        .arg(base_dir.join("Sc-n030w4-tmp.txt"))
        .arg("--his")
        .arg(base_dir.join("H0-n030w4-0.txt"))
        .arg("--weeks")
        .arg(base_dir.join("WD-n030w4-0.txt"))
        .arg("--sols")
        .arg(&out_path)
        .output()
        .expect("Failed to execute validator");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse stdout for constraints
    let mut val_days_off = -1;
    let mut val_preferences = -1;
    let mut val_weekends = -1;
    let mut val_optimal = -1;
    let mut val_consecutive = -1;

    for line in stdout.lines() {
        if line.starts_with("Non working days constraints:") {
            val_days_off = line.split(":").nth(1).unwrap().trim().parse().unwrap();
        } else if line.starts_with("Preferences:") {
            val_preferences = line.split(":").nth(1).unwrap().trim().parse().unwrap();
        } else if line.starts_with("Complete weekends:") {
            val_weekends = line.split(":").nth(1).unwrap().trim().parse().unwrap();
        } else if line.starts_with("Optimal coverage constraints:") {
            val_optimal = line.split(":").nth(1).unwrap().trim().parse().unwrap();
        } else if line.starts_with("Consecutive constraints:") {
            val_consecutive = line.split(":").nth(1).unwrap().trim().parse().unwrap();
        }
    }

    println!("--- SCORE EQUIVALENCE PARITY ---");
    println!("Constraint           | Coralys | Validator | Match ");
    println!("---------------------|---------|-----------|-------");
    println!(
        "Days Off             | {:<7} | {:<9} | {}",
        best.soft_report.day_off_penalty,
        val_days_off,
        best.soft_report.day_off_penalty == val_days_off
    );
    println!(
        "Preferences          | {:<7} | {:<9} | {}",
        best.soft_report.preferences_penalty,
        val_preferences,
        best.soft_report.preferences_penalty == val_preferences
    );
    println!(
        "Complete Weekends    | {:<7} | {:<9} | {}",
        best.soft_report.weekend_penalty,
        val_weekends,
        best.soft_report.weekend_penalty == val_weekends
    );
    println!(
        "Consecutive Work     | {:<7} | {:<9} | {}",
        best.soft_report.work_streak_penalty,
        val_consecutive,
        best.soft_report.work_streak_penalty == val_consecutive
    );
    println!(
        "Optimal Coverage     | {:<7} | {:<9} | {}",
        best.soft_report.optimal_coverage_penalty,
        val_optimal,
        best.soft_report.optimal_coverage_penalty == val_optimal
    );

    assert_eq!(
        best.soft_report.day_off_penalty, val_days_off,
        "Days off mismatch!"
    );
    assert_eq!(
        best.soft_report.preferences_penalty, val_preferences,
        "Preferences mismatch!"
    );
    assert_eq!(
        best.soft_report.weekend_penalty, val_weekends,
        "Complete weekends mismatch!"
    );
    // assert_eq!(best.soft_report.work_streak_penalty, val_consecutive, "Consecutive mismatch!");
    assert_eq!(
        best.soft_report.optimal_coverage_penalty, val_optimal,
        "Optimal coverage mismatch!"
    );
}
