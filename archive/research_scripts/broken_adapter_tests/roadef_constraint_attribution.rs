use roadef::evaluator::RoadefEvaluator;
use roadef::models::{Network, Scenario, TrafficMatrix, Solution, SrPath, NetworkNode, NetworkLink, Demand, Intervention, BudgetConstraint};

fn mock_network() -> Network {
    Network {
        directed: true,
        multigraph: false,
        nodes: vec![
            NetworkNode { id: 0, name: None },
            NetworkNode { id: 1, name: None },
        ],
        links: vec![
            NetworkLink { id: 0, from: 0, to: 1, capacity: 100.0, metric: 10.0 },
        ],
    }
}

fn mock_tm() -> TrafficMatrix {
    TrafficMatrix {
        num_time_slots: 1,
        demands: vec![
            Demand { s: 0, t: 1, v: vec![50.0] }
        ],
    }
}

fn mock_scenario() -> Scenario {
    Scenario {
        budget: vec![BudgetConstraint { t: 0, value: 100 }],
        max_segments: 5,
        interventions: vec![],
    }
}

#[test]
fn test_constraint_attribution() {
    let mut net = mock_network();
    let mut tm = mock_tm();
    let mut sc = mock_scenario();

    // Baseline: Everything valid
    let eval = RoadefEvaluator::new(&net, tm.clone(), sc.clone());
    let sol = Solution { srpaths: vec![] };
    let (sim, _) = eval.simulate_roadef(&sol);
    
    assert!(sim.structural_valid, "Baseline must be structural valid");
    assert!(sim.budget_valid, "Baseline must be budget valid");
    assert!(sim.max_segments_valid, "Baseline must be max segments valid");
    assert!(sim.capacity_feasible, "Baseline must be capacity feasible");

    // E3-A: Structural Invalid (Disconnect link 0 via intervention)
    sc.interventions.push(Intervention { t: 0, links: vec![0] });
    let eval_e3a = RoadefEvaluator::new(&net, tm.clone(), sc.clone());
    let (sim_e3a, _) = eval_e3a.simulate_roadef(&sol);
    assert!(!sim_e3a.structural_valid, "E3-A: must fail routing");
    assert!(sim_e3a.budget_valid, "E3-A: budget unaffected");
    assert!(sim_e3a.max_segments_valid, "E3-A: max_segments unaffected");
    assert!(sim_e3a.capacity_feasible, "E3-A: capacity feasible (0 flow)");
    sc.interventions.clear();

    // E3-B: Budget Invalid (Budget is 0, path requires distance cost, but wait, distance is 0 for direct.
    // Actually, budget is checked between time slots. Let's add 2 time slots and change the path.
    let mut tm2 = mock_tm();
    tm2.num_time_slots = 2;
    tm2.demands[0].v = vec![50.0, 50.0];
    let mut sc2 = mock_scenario();
    sc2.budget = vec![BudgetConstraint { t: 1, value: 0 }]; // 0 budget for t=1
    let eval_e3b = RoadefEvaluator::new(&net, tm2.clone(), sc2.clone());
    
    let sol_budget_fail = Solution { srpaths: vec![
        SrPath { d: 0, t: 0, w: vec![] },
        SrPath { d: 0, t: 1, w: vec![1] }, // This change costs > 0
    ]};
    let (sim_e3b, _) = eval_e3b.simulate_roadef(&sol_budget_fail);
    // Since waypoint [1] is invalid (node 1 is target), wait, distance will be positive.
    assert!(sim_e3b.structural_valid, "E3-B: structural unaffected");
    assert!(!sim_e3b.budget_valid, "E3-B: must fail budget");
    assert!(sim_e3b.max_segments_valid, "E3-B: max_segments unaffected");
    assert!(sim_e3b.capacity_feasible, "E3-B: capacity feasible");

    // E3-C: Max Segments Invalid
    let mut sc3 = mock_scenario();
    sc3.max_segments = 0; // max 0 segments = impossible
    let eval_e3c = RoadefEvaluator::new(&net, tm.clone(), sc3.clone());
    let sol_seg_fail = Solution { srpaths: vec![SrPath { d: 0, t: 0, w: vec![1] }] };
    let (sim_e3c, _) = eval_e3c.simulate_roadef(&sol_seg_fail);
    // simulate_roadef doesn't fail fast on max_segments, it just sets the flag
    assert!(sim_e3c.structural_valid, "E3-C: structural unaffected");
    assert!(sim_e3c.budget_valid, "E3-C: budget unaffected");
    assert!(!sim_e3c.max_segments_valid, "E3-C: must fail max segments");
    assert!(sim_e3c.capacity_feasible, "E3-C: capacity unaffected");

    // E4-A: Capacity Invalid
    let mut tm_cap = mock_tm();
    tm_cap.demands[0].v = vec![200.0]; // Capacity is 100
    let eval_e4a = RoadefEvaluator::new(&net, tm_cap.clone(), sc.clone());
    let (sim_e4a, _) = eval_e4a.simulate_roadef(&sol);
    assert!(sim_e4a.structural_valid, "E4-A: structural unaffected");
    assert!(sim_e4a.budget_valid, "E4-A: budget unaffected");
    assert!(sim_e4a.max_segments_valid, "E4-A: max segments unaffected");
    assert!(!sim_e4a.capacity_feasible, "E4-A: must fail capacity");
}
