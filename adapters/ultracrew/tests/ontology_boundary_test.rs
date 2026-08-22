use std::fs;
use std::path::PathBuf;

#[test]
fn test_ontology_isolation_no_airline_concepts() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("src");
    
    let mut files_to_check = Vec::new();
    fn visit_dirs(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dirs(&path, files);
                    } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                        files.push(path);
                    }
                }
            }
        }
    }
    visit_dirs(&d, &mut files_to_check);

    let forbidden_terms = vec![
        "Layover",
        "FDP",
        "FDTL",
        "AirlineConstraintEvaluator",
        "AirlineScenario",
        "flight",
        "crew_role",
        "duty",
        "rotation",
        "aircraft",
        "flight_leg",
        "block_time",
        "duty_time",
        "DGCA",
        "FAA",
        "airline",
        "aviation",
    ];

    for file in files_to_check {
        let content = fs::read_to_string(&file).expect("Failed to read file");
        for term in &forbidden_terms {
            // we ignore case where the term is just used in a comment, but for strictness
            // we can just forbid the exact case-sensitive word from appearing at all outside of comments.
            // A simple string search is enough for a boundary test.
            if content.contains(term) {
                panic!("Ontology violation in file {:?}: Found forbidden airline concept '{}'", file, term);
            }
        }
    }
}
