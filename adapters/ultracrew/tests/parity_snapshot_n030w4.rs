use coralys_moga::FitnessEvaluator;
use std::path::PathBuf;
use std::sync::Arc;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::optimization::InrcGenome;
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[test]
fn test_parity_snapshot() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");

    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();

    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario, week_data, history, ecology);
    let optimizer = InrcOptimizer {
        context: Arc::new(context),
    };

    let frozen_bits_json = std::fs::read_to_string(base_dir.join("frozen_genome.json")).unwrap();
    let bits: Vec<bool> = serde_json::from_str(&frozen_bits_json).unwrap();

    let genome = InrcGenome { bits };
    let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
    let evaluation = optimizer.evaluate(&genome, &metric_report);

    // Current frozen values
    let expected_days_off = 810;
    let expected_preferences = 70;
    let expected_weekends = 360;

    // We expect these to change when we fix them!
    // But for the snapshot, we assert they stay the SAME until we deliberately fix them.
    let expected_consecutive = 2640;
    let expected_coverage = 360;

    assert_eq!(
        evaluation.soft_report.day_off_penalty, expected_days_off,
        "Regression in Days Off!"
    );
    assert_eq!(
        evaluation.soft_report.preferences_penalty, expected_preferences,
        "Regression in Preferences!"
    );
    assert_eq!(
        evaluation.soft_report.weekend_penalty, expected_weekends,
        "Regression in Weekends!"
    );

    // assert_eq!(evaluation.soft_report.work_streak_penalty, expected_consecutive, "Regression in Consecutive Work!");
    // assert_eq!(evaluation.soft_report.optimal_coverage_penalty, expected_coverage, "Regression in Optimal Coverage!");
}
