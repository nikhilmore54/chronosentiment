use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use std::path::PathBuf;

#[test]
fn test_inrc_parser() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("n030w4");

    let sc_path = base_dir.join("Sc-n030w4.json");
    let scenario = parse_scenario(&sc_path).expect("Failed to parse scenario");
    assert_eq!(scenario.id, "n030w4");
    assert_eq!(scenario.number_of_weeks, 4);
    assert_eq!(scenario.nurses.len(), 30);
    assert_eq!(scenario.shift_types.len(), 4);

    let h0_path = base_dir.join("H0-n030w4-0.json");
    let history = parse_history(&h0_path).expect("Failed to parse history");
    assert_eq!(history.week, 0);
    assert_eq!(history.scenario, "n030w4");
    assert_eq!(history.nurse_history.len(), 30);

    let wd_path = base_dir.join("WD-n030w4-0.json");
    let week_data = parse_week_data(&wd_path).expect("Failed to parse week data");
    assert_eq!(week_data.scenario, "n030w4");
    assert!(week_data.requirements.len() > 0);
}
