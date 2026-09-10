use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use roadef::moga_impl::{EvalComparator, LexicographicComparator, RoadefEvaluation, RoadefGenome, RoadefObjective};

/// Helper to parse a specific instance's golden vector from the official sprint CSV.
fn parse_golden_vector(instance_name: &str) -> Vec<f64> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/golden/loads_vector.csv");

    let file = File::open(&path).unwrap_or_else(|_| panic!("Failed to open golden CSV: {:?}", path));
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result.unwrap();
        if line.starts_with(instance_name) {
            let parts: Vec<&str> = line.split(',').collect();
            // Format: Instance, Best team, 1, 2, 3...
            // Skip the first two columns.
            let mut vector = Vec::with_capacity(parts.len() - 2);
            for p in &parts[2..] {
                let p = p.trim();
                if p.is_empty() {
                    continue;
                }
                let val: f64 = p.parse().unwrap_or_else(|_| panic!("Failed to parse float: '{}'", p));
                vector.push(val);
            }
            return vector;
        }
    }
    panic!("Instance {} not found in golden CSV", instance_name);
}

// Helper to construct a minimal RoadefEvaluation
fn make_eval(load_vector: Vec<f64>) -> RoadefEvaluation {
    RoadefEvaluation {
        genome: RoadefGenome {
            waypoints: vec![],
            num_time_slots: 0,
        },
        objective: RoadefObjective { loads_desc: load_vector },
        feasible: true, // We test pure representation, so feasible doesn't matter for LexicographicComparator
        mlu: 0.0,
        operator: "test",
        max_sat: 0.0,
    }
}

#[test]
fn test_golden_vector_representation() {
    // G3E-1: Parse the official golden vector for setA-01
    let golden_seta01 = parse_golden_vector("setA-01");
    
    assert!(!golden_seta01.is_empty(), "Vector should not be empty");
    
    // G3E-2: Validate descending order invariant.
    // The vector must be monotonically non-increasing.
    for i in 0..golden_seta01.len() - 1 {
        assert!(
            golden_seta01[i] >= golden_seta01[i + 1],
            "Vector is not descending at index {}: {} < {}",
            i, golden_seta01[i], golden_seta01[i + 1]
        );
    }
    
    // G3E-3: Ensure our `LexicographicComparator` natively handles this representation.
    let cmp = LexicographicComparator;
    
    let obj = make_eval(golden_seta01.clone());
    
    // 4a. Equality
    let obj_eq = make_eval(golden_seta01.clone());
    assert_eq!(
        cmp.cmp_evals(&obj, &obj_eq),
        Ordering::Equal,
        "Identical vectors must compare Equal"
    );
    
    // 4b. Superiority
    // Create a vector that is strictly better (lower load) at the very first index.
    let mut better_vec = golden_seta01.clone();
    better_vec[0] -= 0.000001; 
    
    let obj_better = make_eval(better_vec);
    assert_eq!(
        cmp.cmp_evals(&obj_better, &obj),
        Ordering::Greater, // 'Greater' in GA means "better" / "wins"
        "Lower load must win under lexicographic ordering"
    );

    assert_eq!(
        cmp.cmp_evals(&obj, &obj_better),
        Ordering::Less,
        "Higher load must lose"
    );
}
