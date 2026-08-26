use std::process::Command;

#[test]
fn optimization_has_no_financial_dependencies() {
    let output = Command::new("cargo")
        .args(&["tree", "-p", "chronosentiment_optimization"])
        .output()
        .expect("Failed to execute cargo tree");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let forbidden_crates = [
        "chronosentiment_strategies",
        "chronosentiment_financial_core",
        "replay",
    ];

    for forbidden in forbidden_crates.iter() {
        assert!(
            !stdout.contains(forbidden),
            "CONSTITUTIONAL VIOLATION: Optimization layer depends on financial/semantic crate: {}",
            forbidden
        );
    }
}
