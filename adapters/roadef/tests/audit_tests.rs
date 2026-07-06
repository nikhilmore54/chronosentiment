use roadef::loader::{load_network, load_traffic_matrix, load_scenario, load_solution};
use roadef::evaluator::RoadefEvaluator;

fn evaluate_audit_case(case_name: &str) -> roadef::evaluator::EvaluationResult {
    let net = load_network(&format!("tests/audit/{}/net.json", case_name)).unwrap();
    let tm = load_traffic_matrix(&format!("tests/audit/{}/tm.json", case_name)).unwrap();
    let scenario = load_scenario(&format!("tests/audit/{}/scenario.json", case_name)).unwrap();
    let solution = load_solution(&format!("tests/audit/{}/srpaths.json", case_name)).unwrap();
    
    let evaluator = RoadefEvaluator::new(&net, tm, scenario);
    evaluator.evaluate_solution(&solution)
}

#[test]
fn test_audit_feasible() {
    let result = evaluate_audit_case("feasible");
    assert!(result.valid, "Feasible case should be valid");
}

#[test]
fn test_audit_budget_violation() {
    let result = evaluate_audit_case("budget_violation");
    assert!(!result.valid, "Budget violation should be invalid");
    assert_eq!(result.obj, f64::INFINITY);
}

#[test]
fn test_audit_maxsegments_violation() {
    let result = evaluate_audit_case("maxsegments_violation");
    assert!(!result.valid, "Max segments violation should be invalid");
}

#[test]
fn test_audit_disconnected() {
    let result = evaluate_audit_case("disconnected");
    assert!(!result.valid, "Disconnected demand should be invalid");
}

#[test]
fn test_audit_intervention_edge() {
    let net = load_network("tests/audit/intervention_edge/net.json").unwrap();
    let tm = load_traffic_matrix("tests/audit/intervention_edge/tm.json").unwrap();
    let scenario = load_scenario("tests/audit/intervention_edge/scenario.json").unwrap();
    let solution = load_solution("tests/audit/intervention_edge/srpaths.json").unwrap();
    
    let evaluator = RoadefEvaluator::new(&net, tm, scenario);
    let loads = evaluator.compute_loads(1, &solution).expect("Should still be connected due to fallback");
    
    // Original path: 0 -> 1 -> 3. Edge 1->3 is ID 4.
    // At t=1, 1->3 is intervened. Route goes 0->1, then 1->0->2->3.
    // Edge IDs: 0->1 (0), 1->0 (1), 0->2 (2), 2->3 (6), 1->3 (4)
    println!("arc_flows: {:?}", loads.arc_flows);
    assert!((loads.arc_flows[&0] - 100.0).abs() < 1e-6, "0->1 should have 100 flow");
    assert!((loads.arc_flows[&1] - 100.0).abs() < 1e-6, "1->0 should have 100 flow");
    assert!((loads.arc_flows[&2] - 100.0).abs() < 1e-6, "0->2 should have 100 flow");
    assert!((loads.arc_flows[&6] - 100.0).abs() < 1e-6, "2->3 should have 100 flow");
    
    // 1->3 should have 0 flow
    assert!((loads.arc_flows.get(&4).unwrap_or(&0.0) - 0.0).abs() < 1e-6, "1->3 should have 0 flow");

    let result = evaluator.evaluate_solution(&solution);
    assert!(result.valid, "Intervention reroutes flow, remains valid");
}

#[test]
fn test_audit_ecmp_tie() {
    let net = load_network("tests/audit/ecmp_tie/net.json").unwrap();
    let tm = load_traffic_matrix("tests/audit/ecmp_tie/tm.json").unwrap();
    let scenario = load_scenario("tests/audit/ecmp_tie/scenario.json").unwrap();
    let solution = load_solution("tests/audit/ecmp_tie/srpaths.json").unwrap();
    
    let evaluator = RoadefEvaluator::new(&net, tm, scenario);
    let loads = evaluator.compute_loads(0, &solution).expect("ECMP Tie should be connected");
    
    // Assert load(AB) == load(AC) == 50.0
    // Edge IDs: 0->1 is 0, 0->2 is 2
    assert!((loads.arc_flows[&0] - 50.0).abs() < 1e-6);
    assert!((loads.arc_flows[&2] - 50.0).abs() < 1e-6);
    
    // Assert load(BD) == load(CD) == 50.0
    // Edge IDs: 1->3 is 4, 2->3 is 6
    assert!((loads.arc_flows[&4] - 50.0).abs() < 1e-6);
    assert!((loads.arc_flows[&6] - 50.0).abs() < 1e-6);
    
    let result = evaluator.evaluate_solution(&solution);
    assert!(result.valid);
}
