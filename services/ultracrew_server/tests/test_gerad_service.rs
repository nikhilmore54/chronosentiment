use ultracrew_server::gerad_service::{
    get_benchmark_catalog, run_gerad_coralys_optimization, run_gerad_irops_recovery,
    AirlineIropsRequest, AirlineOptimizeRequest, DelayedFlightInput,
};

#[test]
fn test_gerad_benchmark_catalog() {
    let catalog = get_benchmark_catalog();
    assert_eq!(catalog.len(), 7);
    assert_eq!(catalog[0].id, "instance1");
    assert_eq!(catalog[6].crew_count, 305);
    assert_eq!(catalog[6].pairings_count, 1104);
}

#[test]
fn test_gerad_coralys_optimization_mode_a() {
    let req = AirlineOptimizeRequest {
        instance_id: Some("instance1".to_string()),
        mode: "equity".to_string(),
        generations: 200,
        pop_size: 40,
        seed: 42,
        credit_rigs: true,
        crew_availability: true,
        reserve_ratio: 0.08,
    };

    let resp = run_gerad_coralys_optimization(req);
    assert_eq!(resp.instance_id, "instance1");
    assert!(resp.audit_matrix.tier1.overall_tier1_passed);
    assert_eq!(resp.audit_matrix.tier1.base_compatibility_pct, 100.0);
    assert!(resp.audit_matrix.tier1.hard_85h_flying_cap_passed);
    assert!(resp.audit_matrix.tier2.workload_disparity_sigma <= 4.0);
    assert_eq!(resp.rotations.len(), 85);
    assert!(resp.speedup_vs_gencol > 100.0);
}

#[test]
fn test_gerad_irops_recovery() {
    let req = AirlineIropsRequest {
        frozen_until_day: 10,
        sick_crew_ids: vec!["CRW-002".to_string()],
        cancelled_flight_ids: vec![],
        delayed_flights: vec![DelayedFlightInput {
            flight_id: "FL-001-00-1".to_string(),
            delay_minutes: 45,
        }],
    };

    let resp = run_gerad_irops_recovery(req);
    assert!(resp.rescued_pairings_count >= 2);
    assert!(resp.recovery_runtime_secs < 5.0);
    assert!(resp.delta_audit_matrix.tier1.overall_tier1_passed);
}
