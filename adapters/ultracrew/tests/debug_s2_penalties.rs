use coralys_moga::traits::FitnessEvaluator;
use std::path::PathBuf;
use std::sync::Arc;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::optimization::InrcGenome;
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[test]
fn test_debug_s2_penalties() {
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

    // Instead of using evaluate(), let's re-implement the evaluator logic here but with print statements!
    // Or we can just modify evaluator.rs to print if a flag is set, but copying it here is safer.
    // Wait, let's just temporarily add println! to evaluator.rs and run the snapshot test!
}
