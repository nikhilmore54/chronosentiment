use std::fs;
use std::path::Path;

fn check_no_domain_leaks_in_dir(dir: &Path) {
    if dir.is_dir() {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() {
                check_no_domain_leaks_in_dir(&path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = fs::read_to_string(&path).expect("Failed to read file");
                for (line_num, line) in content.lines().enumerate() {
                    // We only check for structural dependencies: `use ultracrew` or `extern crate ultracrew`
                    if line.contains("use ultracrew") || line.contains("extern crate ultracrew") || line.contains("use roadef") {
                        panic!(
                            "Architectural Violation in {}:{}: Coralys Runtime must not depend on domain implementations.",
                            path.display(),
                            line_num + 1
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_no_domain_leaks() {
    // 1. Check for `use ultracrew` or `extern crate ultracrew` in `src/`
    let src_dir = Path::new("src");
    check_no_domain_leaks_in_dir(src_dir);

    // 2. Check Cargo.toml for domain dependencies
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    if cargo_toml.contains("ultracrew") || cargo_toml.contains("roadef") {
        panic!("Architectural Violation: Cargo.toml contains domain dependencies.");
    }
}
