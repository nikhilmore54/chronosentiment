use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use roadef::models::SrPath;
use roadef::loader::{load_network, load_scenario, load_traffic_matrix};
use roadef::evaluator::RoadefEvaluator;
use roadef::moga_impl::RoadefGenome;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CheckerSat {
    t: f64,
    from: u64,
    to: u64,
    sat: f64,
}

#[derive(Deserialize, Debug)]

struct SrPathJson {
    d: usize,
    t: usize,
    w: Vec<u64>,
}

#[derive(Deserialize, Debug)]
struct SrPathsWrapper {
    srpaths: Vec<SrPathJson>,
}

fn load_solution(path: &str) -> Vec<SrPath> {
    let content = fs::read_to_string(path).unwrap();
    let wrapper: SrPathsWrapper = serde_json::from_str(&content).unwrap();
    wrapper
        .srpaths
        .into_iter()
        .map(|p| SrPath {
            d: p.d,
            t: p.t,
            w: p.w,
        })
        .collect()
}

#[test]
fn test_checker_parity_seta14() {
    let repo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("repo/challenge-roadef-2026-main");
    
    let net_path = repo_dir.join("setA/setA-14-net.json");
    let tm_path = repo_dir.join("setA/setA-14-tm.json");
    let scenario_path = repo_dir.join("setA/setA-14-scenario.json");
    let srpaths_path = repo_dir.join("setA/setA-14-srpaths.json");
    let checker_bin = repo_dir.join("checker/src/checker");

    if !checker_bin.exists() {
        println!("Gate 4 unavailable: Official checker not found at {:?}", checker_bin);
        return;
    }

    // 1. Run official checker
    let output = Command::new(&checker_bin)
        .arg("--instance")
        .arg(repo_dir.join("setA/setA-14"))
        .output()
        .expect("Failed to run official checker");

    assert!(output.status.success(), "Checker failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // Extract JSON part
    // The official checker output is not parsed as a complete JSON document because its `objectives`
    // section may contain non-standard JSON numeric values (`Infinity`). Gate 4 intentionally extracts
    // only the checker's saturation array, which is the authoritative artifact being compared.
    let sat_key = "\"saturations\":";
    let sat_start = stdout.find(sat_key).expect("Could not find saturations in checker output");
    let array_start = sat_start + sat_key.len();
    let array_end = stdout[array_start..].rfind(']').expect("Could not find end of saturations array") + array_start;
    let json_str = &stdout[array_start..=array_end];
    
    let checker_sats: Vec<CheckerSat> = serde_json::from_str(json_str)
        .expect("Failed to parse checker JSON output");
        
    // Build lookup for checker saturations
    let mut expected_sats: HashMap<(usize, u64, u64), f64> = HashMap::new();
    for c in &checker_sats {
        expected_sats.insert((c.t as usize, c.from, c.to), c.sat);
    }
    
    // Compute expected descending vector from checker saturations
    let mut expected_vector: Vec<f64> = checker_sats.iter().map(|s| s.sat).collect();
    expected_vector.sort_by(|a, b| b.partial_cmp(a).unwrap());


    // 2. Run our simulator
    let net = load_network(net_path.to_str().unwrap()).unwrap();
    let tm = load_traffic_matrix(tm_path.to_str().unwrap()).unwrap();
    let scenario = load_scenario(scenario_path.to_str().unwrap()).unwrap();
    let srpaths = load_solution(srpaths_path.to_str().unwrap());

    // Build solution
    let solution = roadef::models::Solution {
        srpaths: srpaths.clone(),
    };

    let evaluator = roadef::evaluator::RoadefEvaluator::new(&net, tm, scenario);
    
    // 3. Compare (t, from, to) saturations explicitly
    let mut actual_sats: HashMap<(usize, u64, u64), f64> = HashMap::new();
    let mut actual_vector = Vec::new();
    
    for ts in 0..evaluator.tm.num_time_slots {
        if let Some(ts_loads) = evaluator.compute_loads(ts, &solution) {
            for (&arc_id, &sat) in &ts_loads.arc_saturations {
                actual_vector.push(sat);
                
                // Map arc_id to from/to
                // arc_id is the index in evaluator.graph.arcs in evaluate()
                if let Some(arc) = evaluator.graph.arcs.iter().find(|a| a.id == arc_id) {
                    actual_sats.insert((ts, arc.from, arc.to), sat);
                } else {
                    // if arc.id is not exactly index, but it is in Roadef, wait, graph.arcs uses internal ID?
                    // In Digraph, arcs have `source` and `target` which correspond to node IDs.
                    let arc = &evaluator.graph.arcs[arc_id as usize];
                    actual_sats.insert((ts, arc.from, arc.to), sat);
                }
            }
        }
    }
    
    // Sort actual vector
    actual_vector.sort_by(|a, b| b.partial_cmp(a).unwrap());

    
    // Do not drop trailing 0.0s. The objective vector represents the entire load vector.
    // Wait, let's keep all elements and check lengths.
    
    let mut off_positive = 0;
    let mut off_zero = 0;
    for &s in &expected_vector {
        if s > 0.0 { off_positive += 1; } else { off_zero += 1; }
    }
    
    let mut our_positive = 0;
    let mut our_zero = 0;
    for &s in &actual_vector {
        if s > 0.0 { our_positive += 1; } else { our_zero += 1; }
    }
    
    let mut max_diff: f64 = 0.0;
    let mut first_diff_tft: Option<(usize, u64, u64)> = None;
    let mut first_diff_idx: Option<usize> = None;
    
    // Check every element of expected vs actual maps
    for (key, &exp_sat) in &expected_sats {
        let act_sat = actual_sats.get(key).copied().unwrap_or(0.0);
        let diff = (exp_sat - act_sat).abs();
        if diff > max_diff { max_diff = diff; }
        if diff > 1e-6 && first_diff_tft.is_none() {
            first_diff_tft = Some(*key);
        }
    }
    
    // Also check elements in actual not in expected
    for (key, &act_sat) in &actual_sats {
        if !expected_sats.contains_key(key) {
            if act_sat > max_diff { max_diff = act_sat; }
            if act_sat > 1e-6 && first_diff_tft.is_none() {
                first_diff_tft = Some(*key);
            }
        }
    }

    // Ensure descending vector matches exactly
    for i in 0..std::cmp::max(expected_vector.len(), actual_vector.len()) {
        let e = expected_vector.get(i).copied().unwrap_or(0.0);
        let a = actual_vector.get(i).copied().unwrap_or(0.0);
        let diff = (e - a).abs();
        if diff > 1e-6 && first_diff_idx.is_none() {
            first_diff_idx = Some(i);
        }
    }

    println!("\nGate 4 setA-14\n");
    println!("Official saturation records: {}", expected_sats.len());
    println!("Our saturation records:      {}\n", actual_sats.len());
    println!("Positive saturation records:");
    println!("  official: {}", off_positive);
    println!("  ours:     {}\n", our_positive);
    println!("Zero saturation records:");
    println!("  official: {}", off_zero);
    println!("  ours:     {}\n", our_zero);
    println!("Maximum |Δsat|: {:.10}\n", max_diff);
    println!("Objective vector length:");
    println!("  official: {}", expected_vector.len());
    println!("  ours:     {}\n", actual_vector.len());
    if let Some(tft) = first_diff_tft {
        println!("First differing (t, from, to): {:?}", tft);
    } else {
        println!("First differing (t, from, to): NONE");
    }
    if let Some(idx) = first_diff_idx {
        println!("First differing objective index: {}", idx);
    } else {
        println!("First differing objective index: NONE");
    }
    
    if first_diff_tft.is_none() && first_diff_idx.is_none() && expected_vector.len() == actual_vector.len() && expected_sats.len() == actual_sats.len() {
        println!("\nRESULT: PASS");
    } else {
        println!("\nRESULT: FAIL");
        panic!("Parity check failed. See output above.");
    }
}
