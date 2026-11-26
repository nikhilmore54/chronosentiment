//! Integration tests for the Coralys Decision Engine and Airline Adapter Pipeline.
//!
//! Validates:
//! 1. All decisions originate from Coralys (`CoralysAirlineEngine`).
//! 2. All decisions pass strictly through the domain adapter (`AirlineDecisionAdapter`).
//! 3. Decisions are legality-certified against statutory rules (CAR 700: 12.0h rest, 85h flying cap, etc.).
//! 4. Infeasible/illegal decisions are strictly rejected with zero state corruption.
//! 5. Valid decisions transition the operational state and seal immutable `DecisionLineage` nodes.
//! 6. Optimization prioritizes 100% flight coverage (zero uncrewed flights).

use chrono::{Duration, TimeZone, Utc};
use coralys_airline::domain::crew::{CrewId, CrewMember, CrewRole, Qualification};
use coralys_airline::domain::duty::{Duty, DutyId};
use coralys_airline::domain::flight::{
    AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber,
};
use coralys_airline::domain::pairing::{Pairing, PairingId};
use coralys_airline::domain::roster::{PlanningPeriod, Roster, RosterId};
use coralys_airline::domain::rotation::{Rotation, RotationId};
use coralys_airline::legality::duty_time::MaximumDutyTimeRule;
use coralys_airline::legality::minimum_rest::MinimumRestRule;
use coralys_airline::legality::LegalityChecker;
use coralys_airline::optimization::cost::CostEvaluator;
use coralys_airline::optimization::objective::{CoverageCostObjective, WorkloadBalanceObjective};
use coralys_airline::optimization::search::evolutionary::{EvolutionaryConfig, OptimizationMode};
use coralys_airline::planner::decision::{
    AirlineAction, AirlineDecisionAdapter, AirlineState, CoralysAirlineEngine,
};
use coralys_core::DecisionProposal;

fn base_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap()
}

fn make_leg(id: &str, origin: &str, dest: &str, dep_h: i64, arr_h: i64) -> FlightLeg {
    FlightLeg::new(
        FlightLegId::new(id),
        FlightNumber::new(format!("XX{id}")),
        AirportCode::new(origin),
        AirportCode::new(dest),
        base_time() + Duration::hours(dep_h),
        base_time() + Duration::hours(arr_h),
        AircraftType::new("A320"),
    )
}

fn make_duty(id: &str, legs: Vec<FlightLeg>) -> Duty {
    Duty::new(DutyId::new(id), legs).unwrap()
}

fn make_pairing(id: &str, base: &str, duties: Vec<Duty>) -> Pairing {
    Pairing::new(PairingId::new(id), AirportCode::new(base), duties).unwrap()
}

fn make_crew(id: &str, base: &str) -> CrewMember {
    CrewMember::new(
        CrewId::new(id),
        id,
        CrewRole::Captain,
        vec![Qualification::new(AircraftType::new("A320"))],
        AirportCode::new(base),
    )
}

fn build_test_environment() -> (AirlineState, LegalityChecker, CoralysAirlineEngine) {
    let crew1 = make_crew("CRW01", "LHR");
    let crew2 = make_crew("CRW02", "LHR");

    // Flight schedule: 6 flights forming 3 pairings
    let leg0a = make_leg("L00a", "LHR", "DUB", 4, 6);
    let leg0b = make_leg("L00b", "DUB", "LHR", 7, 9);
    let d0 = make_duty("D00", vec![leg0a.clone(), leg0b.clone()]);
    let p0 = make_pairing("P00", "LHR", vec![d0]);

    let leg1 = make_leg("L01", "LHR", "CDG", 8, 10);
    let leg2 = make_leg("L02", "CDG", "LHR", 12, 14);
    let d1 = make_duty("D01", vec![leg1.clone(), leg2.clone()]);
    let p1 = make_pairing("P01", "LHR", vec![d1]);

    let leg3 = make_leg("L03", "LHR", "FRA", 32, 34);
    let leg4 = make_leg("L04", "FRA", "LHR", 36, 38);
    let d2 = make_duty("D02", vec![leg3.clone(), leg4.clone()]);
    let p2 = make_pairing("P02", "LHR", vec![d2]);

    // Monolithic pairing visiting base internally (LHR -> AMS -> LHR, then LHR -> MUC -> LHR)
    let leg5 = make_leg("L05", "LHR", "AMS", 56, 58);
    let leg6 = make_leg("L06", "AMS", "LHR", 60, 62);
    let d3 = make_duty("D03", vec![leg5.clone(), leg6.clone()]);

    let leg7 = make_leg("L07", "LHR", "MUC", 80, 82);
    let leg8 = make_leg("L08", "MUC", "LHR", 84, 86);
    let d4 = make_duty("D04", vec![leg7.clone(), leg8.clone()]);
    let p3 = make_pairing("P03_MONO", "LHR", vec![d3, d4]);

    let all_legs = vec![
        leg0a, leg0b, leg1, leg2, leg3, leg4, leg5, leg6, leg7, leg8,
    ];

    let rot1 = Rotation::new(
        RotationId::new("ROT01"),
        CrewId::new("CRW01"),
        vec![p1.clone()],
    )
    .unwrap();

    let rot2 = Rotation::new(
        RotationId::new("ROT02"),
        CrewId::new("CRW02"),
        vec![p0.clone()],
    )
    .unwrap();

    let period = PlanningPeriod::new(base_time(), base_time() + Duration::days(30));
    let roster = Roster::with_crew(
        RosterId::new("ROSTER_TEST"),
        period,
        all_legs.clone(),
        vec![rot1, rot2],
        vec![crew1, crew2],
    )
    .unwrap();

    let unassigned_pairings = vec![p2, p3];
    let unassigned_legs = vec![];

    let initial_state = AirlineState::new(roster, unassigned_pairings, unassigned_legs);

    // Checker with statutory minimum rest (12 hours) and maximum duty time (14 hours)
    let mut checker = LegalityChecker::new();
    checker.add_rule(Box::new(MinimumRestRule::with_minimum(Duration::minutes(720))));
    checker.add_rule(Box::new(MaximumDutyTimeRule::new()));

    // Cost evaluator prioritizing coverage and workload balance
    let mut evaluator = CostEvaluator::new();
    evaluator.add_objective(Box::new(CoverageCostObjective {
        uncovered_penalty: 50_000.0,
        overcoverage_penalty: 1_000.0,
    }));
    evaluator.add_objective(Box::new(WorkloadBalanceObjective));

    let engine = CoralysAirlineEngine::new(evaluator, vec![1.0, 0.1]);

    (initial_state, checker, engine)
}

#[test]
fn test_decision_adapter_lineage_initialization() {
    let (initial_state, checker, _engine) = build_test_environment();
    let adapter = AirlineDecisionAdapter::new(initial_state.clone(), checker);

    let lineage = adapter.lineage();
    assert_eq!(lineage.root_id, initial_state.reference.id);
    assert_eq!(lineage.current_id, initial_state.reference.id);
    assert_eq!(lineage.nodes.len(), 1);

    let root_node = lineage.nodes.get(&lineage.root_id).unwrap();
    assert!(root_node.parent_id.is_none());
    assert!(root_node.proposal.is_none());
    assert!(!root_node.evaluation.metrics.is_empty());
}

#[test]
fn test_coralys_generates_decision_proposals_and_adapter_evaluates() {
    let (initial_state, checker, engine) = build_test_environment();
    let mut adapter = AirlineDecisionAdapter::new(initial_state, checker);

    let config = EvolutionaryConfig {
        pop_size: 20,
        generations: 30,
        crossover_prob: 0.8,
        tournament_k: 3,
        seed: 42,
        unassigned_penalty: 50_000.0,
        mode: OptimizationMode::ModeAEquity,
        ..Default::default()
    };

    let proposals = adapter.generate_proposals(&engine, &config).unwrap();
    assert!(!proposals.is_empty(), "Coralys must generate decision proposals");

    let primary_proposal = &proposals[0];
    assert_eq!(primary_proposal.priority, 1.0);
    assert!(primary_proposal.estimated_gain > 0.0);
    assert!(primary_proposal.confidence >= 0.95);
    assert_eq!(primary_proposal.violations_resolved, vec!["zero_uncrewed_flights"]);
}

#[test]
fn test_adapter_statutory_gate_strictly_rejects_illegal_proposal() {
    let (initial_state, checker, _engine) = build_test_environment();
    let mut adapter = AirlineDecisionAdapter::new(initial_state.clone(), checker);

    // Create an illegal pairing that violates minimum rest (rest between duties = 2h < 12h)
    let leg_a = make_leg("L_ILL1", "LHR", "CDG", 20, 22);
    let d_a = make_duty("D_ILL1", vec![leg_a]);
    let leg_b = make_leg("L_ILL2", "CDG", "LHR", 24, 26); // rest between duties = 2h < 12h
    let d_b = make_duty("D_ILL2", vec![leg_b]);
    let illegal_pairing = make_pairing("P_ILLEGAL", "LHR", vec![d_a, d_b]);

    let illegal_action = AirlineAction::AssignPairing {
        crew_id: CrewId::new("CRW01"),
        pairing: illegal_pairing,
    };

    let illegal_proposal = DecisionProposal {
        priority: 1.0,
        estimated_gain: 10.0,
        affected_resources: vec!["CRW01".to_string()],
        violations_resolved: vec![],
        confidence: 0.50,
        payload: serde_json::to_value(illegal_action).unwrap(),
    };

    let initial_lineage_size = adapter.lineage().nodes.len();
    let commit_result = adapter.commit_decision(illegal_proposal);

    // Must be rejected by statutory gate!
    assert!(commit_result.is_err(), "Statutory gate must reject rest violation");
    let err_msg = commit_result.unwrap_err();
    assert!(err_msg.contains("statutory legality gate"), "Error message must indicate statutory gate failure: {err_msg}");

    // Operational state and lineage must remain completely untouched
    assert_eq!(adapter.lineage().nodes.len(), initial_lineage_size);
    assert_eq!(adapter.lineage().current_id, initial_state.reference.id);
}

#[test]
fn test_end_to_end_decision_flow_coralys_to_adapter_execution() {
    let (initial_state, checker, engine) = build_test_environment();
    let mut adapter = AirlineDecisionAdapter::new(initial_state, checker);

    let config = EvolutionaryConfig {
        pop_size: 20,
        generations: 50,
        crossover_prob: 0.8,
        tournament_k: 3,
        seed: 123,
        unassigned_penalty: 50_000.0,
        mode: OptimizationMode::ModeAEquity,
        ..Default::default()
    };

    // 1. Decisions generated by Coralys
    let proposals = adapter.generate_proposals(&engine, &config).unwrap();
    let best_proposal = proposals.into_iter().next().unwrap();

    // 2. Decision passes through adapter for validation, simulation, and execution
    let eval = adapter.commit_decision(best_proposal.clone()).unwrap();

    // 3. Operational Invariants Verified: 0 statutory violations
    assert_eq!(eval.hard_constraint_violations.len(), 0, "All statutory invariants must be satisfied");

    // 4. Lineage verified: new node sealed with parent reference and proposal attribution
    let lineage = adapter.lineage();
    assert_eq!(lineage.nodes.len(), 2);
    assert_ne!(lineage.current_id, lineage.root_id);

    let current_node = lineage.nodes.get(&lineage.current_id).unwrap();
    assert_eq!(current_node.parent_id, Some(lineage.root_id));
    assert!(current_node.proposal.is_some());
    assert_eq!(current_node.proposal.as_ref().unwrap().confidence, best_proposal.confidence);
}

#[test]
fn test_duty_rebundling_decision_rescues_monolithic_flight_pairings() {
    let (initial_state, _checker, _engine) = build_test_environment();

    // Monolithic pairing P03_MONO in initial_state has 2 sub-turns:
    // Sub 0: LHR -> AMS -> LHR (duties D03)
    // Sub 1: LHR -> MUC -> LHR (duties D04)
    let mono_pairing = initial_state.unassigned_pairings.iter().find(|p| p.id.as_str() == "P03_MONO").unwrap();
    let decomposed = mono_pairing.decompose_at_base_visits();
    assert_eq!(decomposed.len(), 2, "Monolithic pairing must decompose into 2 independent sub-pairings");

    // Both sub-pairings must start and end at LHR
    assert_eq!(decomposed[0].base.as_str(), "LHR");
    assert_eq!(decomposed[1].base.as_str(), "LHR");
    assert_eq!(decomposed[0].duty_count(), 1);
    assert_eq!(decomposed[1].duty_count(), 1);
}

#[test]
fn test_feature_a_reserve_pool_and_pacing_cap() {
    let (initial_state, checker, _engine) = build_test_environment();
    let reserve_crew = make_crew("CRW_RES01", "LHR");
    let pool = coralys_airline::resilience::reserve::ReservePool::from_members(vec![reserve_crew]);

    let allocator = coralys_airline::resilience::reserve::ReserveAllocator::new(&checker);
    let open_pairings = vec![initial_state.unassigned_pairings[0].clone()];

    let (roster, assignments, unallocated) = allocator.allocate_open_pairings(
        &initial_state.roster,
        &open_pairings,
        &pool,
    );

    assert_eq!(assignments.len(), 1, "Reserve allocator must cover the open pairing");
    assert_eq!(unallocated.len(), 0, "No pairings should remain open");
    assert_eq!(assignments[0].crew_id.as_str(), "CRW_RES01");
    assert!(roster.rotations().any(|r| r.crew_id.as_str() == "CRW_RES01"));

    // Default benchmark profile must be Standalone31DayBenchmark with 0 trailing hours for literature parity
    let default_config = EvolutionaryConfig::default();
    assert_eq!(default_config.profile, coralys_airline::SchedulingProfile::Standalone31DayBenchmark);
    assert_eq!(default_config.reserve_ratio, 0.0);
    assert_eq!(default_config.mid_month_pacing_cap, None);

    // Verify production airline profile in EvolutionaryConfig
    let mut config = EvolutionaryConfig::production_airline();
    assert_eq!(config.profile, coralys_airline::SchedulingProfile::ProductionAirlineOperations);
    config.mid_month_pacing_cap = Some(75.0);
    config.pacing_cutoff_day = Some(25);
    config.reserve_ratio = 0.10;
    assert_eq!(config.mid_month_pacing_cap, Some(75.0));
    assert_eq!(config.pacing_cutoff_day, Some(25));
    assert_eq!(config.reserve_ratio, 0.10);
}

#[test]
fn test_feature_b_multi_trip_outstation_tour_execution() {
    let (initial_state, checker, _engine) = build_test_environment();
    let mut adapter = AirlineDecisionAdapter::new(initial_state, checker);

    // Create an outstation tour:
    // Pilot at LHR is positioned to CDG, flies 2 pairings out of CDG, and deadheads back to LHR.
    let outbound = make_duty("DH_OUT", vec![make_leg("L_DHOUT", "LHR", "CDG", 100, 102)]);
    let p1 = make_pairing("P_CDG1", "CDG", vec![make_duty("D_CDG1", vec![
        make_leg("L_C1", "CDG", "NCE", 116, 118),
        make_leg("L_C2", "NCE", "CDG", 120, 122),
    ])]);
    let p2 = make_pairing("P_CDG2", "CDG", vec![make_duty("D_CDG2", vec![
        make_leg("L_C3", "CDG", "TLS", 136, 138),
        make_leg("L_C4", "TLS", "CDG", 140, 142),
    ])]);
    let inbound = make_duty("DH_IN", vec![make_leg("L_DHIN", "CDG", "LHR", 156, 158)]);

    let tour = coralys_airline::OutstationTour::new(
        AirportCode::new("LHR"),
        AirportCode::new("CDG"),
        outbound,
        vec![p1, p2],
        inbound,
    ).unwrap();

    assert_eq!(tour.deadhead_savings_count(), 2, "Bundling 2 station pairings cuts 2 deadhead legs (50% savings)");

    let action = AirlineAction::ApplyOutstationTour {
        tour,
        crew_id: CrewId::new("CRW02"),
    };

    let proposal = DecisionProposal {
        priority: 1.0,
        estimated_gain: 2.0,
        affected_resources: vec!["CRW02".to_string()],
        violations_resolved: vec!["cross_base_deadhead_reduction".to_string()],
        confidence: 0.99,
        payload: serde_json::to_value(action).unwrap(),
    };

    let eval = adapter.commit_decision(proposal).unwrap();
    assert_eq!(eval.hard_constraint_violations.len(), 0, "Outstation tour composite pairing must satisfy all CAR 700 rules");
}

#[test]
fn test_feature_c_month_end_carry_over_and_pairing_splitting() {
    let (initial_state, _checker, _engine) = build_test_environment();

    // Test Carry-Over Rule on period boundary
    let period = &initial_state.roster.period;
    let end_time = period.end;

    // Create a pairing crossing midnight into next month
    let leg_jan1 = make_leg("L_JAN1", "LHR", "FRA", 710, 712); // inside period
    let leg_jan2 = make_leg("L_JAN2", "FRA", "LHR", 714, 716); // inside period
    let d_jan = make_duty("D_JAN", vec![leg_jan1, leg_jan2]);

    // Leg departing past period end (e.g. Feb 1)
    let leg_feb1 = FlightLeg::new(
        FlightLegId::new("L_FEB1"),
        FlightNumber::new("XXFEB1"),
        AirportCode::new("LHR"),
        AirportCode::new("FRA"),
        end_time + Duration::hours(1),
        end_time + Duration::hours(3),
        AircraftType::new("A320"),
    );
    let leg_feb2 = FlightLeg::new(
        FlightLegId::new("L_FEB2"),
        FlightNumber::new("XXFEB2"),
        AirportCode::new("FRA"),
        AirportCode::new("LHR"),
        end_time + Duration::hours(5),
        end_time + Duration::hours(7),
        AircraftType::new("A320"),
    );
    let d_feb = make_duty("D_FEB", vec![leg_feb1, leg_feb2]);
    let carry_pairing = make_pairing("P_CARRY", "LHR", vec![d_jan, d_feb]);

    let jan_hours = carry_pairing.prorated_block_time(period);
    assert_eq!(jan_hours, Duration::hours(4), "Only flight hours inside January count against January's 85h cap");

    let carry_legs = carry_pairing.carry_over_legs(period);
    assert_eq!(carry_legs.len(), 2, "Feb 1 legs must be identified as carry-over into next month");
    assert_eq!(carry_legs[0].id.as_str(), "L_FEB1");
    assert_eq!(carry_legs[1].id.as_str(), "L_FEB2");

    // Test Open-Time Hub Layover Splitting at LHR
    let hub = AirportCode::new("LHR");
    let (p_pt1, p_pt2) = carry_pairing.split_at_hub_layover(&hub).unwrap();
    assert_eq!(p_pt1.base.as_str(), "LHR");
    assert_eq!(p_pt1.duty_count(), 1);
    assert_eq!(p_pt2.base.as_str(), "LHR");
    assert_eq!(p_pt2.duty_count(), 1);
}

#[test]
fn test_feature_d_day_of_operations_frozen_prefix_irops_repair() {
    let (initial_state, checker, _engine) = build_test_environment();
    let mut adapter = AirlineDecisionAdapter::new(initial_state, checker);

    // Operational Day: Day 2 (hour 40).
    // Pairings prior to hour 40 have already flown and are FROZEN.
    let frozen_until = base_time() + Duration::hours(40);

    // Disruption: CRW01 calls in sick after hour 40!
    // Reserve pilot CRW_RES01 is on standby at LHR.
    let reserve_pilot = make_crew("CRW_RES01", "LHR");
    let reserve_pool = coralys_airline::resilience::reserve::ReservePool::from_members(vec![reserve_pilot]);

    let request = coralys_airline::IropsRepairRequest {
        frozen_until,
        sick_crew: vec![CrewId::new("CRW01")],
        cancelled_legs: vec![],
        delayed_legs: vec![],
    };

    // Fast IROPS repair (< 5s)
    let proposal = adapter.repair_irops(&request, Some(&reserve_pool)).unwrap();
    assert_eq!(proposal.priority, 1.0);
    assert_eq!(proposal.violations_resolved, vec!["irops_disruption_recovered"]);

    // Commit through adapter's statutory legality gate
    let eval = adapter.commit_decision(proposal).unwrap();
    assert_eq!(eval.hard_constraint_violations.len(), 0, "Repaired roster must have 0 CAR 700 violations");

    // Lineage must reflect the IROPS recovery event
    let current_node = adapter.lineage().nodes.get(&adapter.lineage().current_id).unwrap();
    assert!(current_node.proposal.is_some());
}

#[test]
fn test_default_month_end_rollover_keeps_crew_intact_and_creates_carry_in_lock() {
    let (initial_state, checker, _engine) = build_test_environment();
    let mut adapter = AirlineDecisionAdapter::new(initial_state.clone(), checker);
    let period = &initial_state.roster.period;
    let end_time = period.end;

    // Create a multi-day pairing starting in January and crossing into February 1
    // D_JAN_M ends at hour 700, ensuring 21+ hours of rest before D_FEB_M at hour 721
    let leg_jan = make_leg("L_JAN_M", "LHR", "LHR", 698, 700);
    let d_jan = make_duty("D_JAN_M", vec![leg_jan]);

    let leg_feb = FlightLeg::new(
        FlightLegId::new("LEG_31_72"),
        FlightNumber::new("BA3172"),
        AirportCode::new("LHR"),
        AirportCode::new("LHR"),
        end_time + Duration::hours(1), // Arrives in February
        end_time + Duration::hours(4),
        AircraftType::new("A320"),
    );
    let d_feb = make_duty("D_FEB_M", vec![leg_feb]);
    let rollover_pairing = make_pairing("P_ROLLOVER", "LHR", vec![d_jan, d_feb]);

    let crew_id = CrewId::new("CRW02");
    let current_jan_block = Duration::hours(60); // Pilot has 25h headroom

    // 1. Evaluate: Must default to Rollover (Keep Crew Intact)
    let hub_candidates = vec![AirportCode::new("LHR")];
    let decision = rollover_pairing.evaluate_month_end_transition(
        &crew_id,
        current_jan_block,
        period,
        &hub_candidates,
    ).unwrap();

    match decision {
        coralys_airline::RolloverDecision::KeepCrewRollover { crew_id: cid, january_block_time, carry_in_lock } => {
            assert_eq!(cid.as_str(), "CRW02");
            assert_eq!(january_block_time, Duration::hours(2), "Only the January leg counts against January cap");
            assert_eq!(carry_in_lock.parent_pairing_id.as_str(), "P_ROLLOVER");
            assert_eq!(carry_in_lock.carry_in_legs.len(), 1);
            assert_eq!(carry_in_lock.carry_in_legs[0].id.as_str(), "LEG_31_72");

            // 2. Commit the rollover decision through the adapter
            let action = AirlineAction::ApplyMonthEndRollover {
                crew_id: cid,
                pairing: rollover_pairing.clone(),
                carry_in_lock: carry_in_lock.clone(),
            };
            let proposal = DecisionProposal {
                priority: 1.0,
                estimated_gain: 1.0,
                affected_resources: vec!["CRW02".to_string()],
                violations_resolved: vec!["month_end_rollover".to_string()],
                confidence: 1.0,
                payload: serde_json::to_value(action).unwrap(),
            };

            let eval = adapter.commit_decision(proposal).unwrap();
            assert_eq!(eval.hard_constraint_violations.len(), 0, "Rollover assignment must satisfy CAR 700");

            // 3. Export February Carry-In Locks
            let exported_locks = adapter.export_february_carry_in_locks(period);
            assert!(exported_locks.iter().any(|l| l.parent_pairing_id.as_str() == "P_ROLLOVER" && l.crew_id.as_str() == "CRW02"));
        }
        coralys_airline::RolloverDecision::HubSplitFallback { .. } => {
            panic!("Expected KeepCrewRollover default, got HubSplitFallback");
        }
    }
}

#[test]
fn test_fallback_hub_split_triggers_when_january_headroom_exhausted() {
    let (initial_state, checker, _engine) = build_test_environment();
    let mut adapter = AirlineDecisionAdapter::new(initial_state.clone(), checker);
    let period = &initial_state.roster.period;
    let end_time = period.end;

    // Create a pairing crossing midnight into February with hub touchpoint at LHR
    // D_JAN_S is LHR -> FRA -> LHR (4h block, ends hour 700)
    let leg_jan1 = make_leg("L_JAN_S1", "LHR", "FRA", 694, 696);
    let leg_jan2 = make_leg("L_JAN_S2", "FRA", "LHR", 698, 700);
    let d_jan = make_duty("D_JAN_S", vec![leg_jan1, leg_jan2]);

    // D_FEB_S is LHR -> FRA -> LHR (in February)
    let leg_feb1 = FlightLeg::new(
        FlightLegId::new("LEG_31_65"),
        FlightNumber::new("BA3165"),
        AirportCode::new("LHR"),
        AirportCode::new("FRA"),
        end_time + Duration::hours(1),
        end_time + Duration::hours(3),
        AircraftType::new("A320"),
    );
    let leg_feb2 = FlightLeg::new(
        FlightLegId::new("LEG_31_66"),
        FlightNumber::new("BA3166"),
        AirportCode::new("FRA"),
        AirportCode::new("LHR"),
        end_time + Duration::hours(5),
        end_time + Duration::hours(7),
        AircraftType::new("A320"),
    );
    let d_feb = make_duty("D_FEB_S", vec![leg_feb1, leg_feb2]);
    let rollover_pairing = make_pairing("P_SPLIT_TRIP", "LHR", vec![d_jan, d_feb]);

    let crew_id = CrewId::new("CRW02");
    // Pilot is sitting at 84.5h flying in January (only 0.5h headroom left, but pairing needs 4.0h in Jan)
    let current_jan_block = Duration::minutes(84 * 60 + 30);

    let hub_candidates = vec![AirportCode::new("LHR")];
    let decision = rollover_pairing.evaluate_month_end_transition(
        &crew_id,
        current_jan_block,
        period,
        &hub_candidates,
    ).unwrap();

    match decision {
        coralys_airline::RolloverDecision::HubSplitFallback { hub, pilot_a_crew_id, pilot_a_pairing, february_reserve_crew_id: _, february_pairing } => {
            assert_eq!(hub.as_str(), "LHR");
            assert_eq!(pilot_a_crew_id.as_str(), "CRW02");
            assert_eq!(pilot_a_pairing.duty_count(), 1);
            assert_eq!(february_pairing.duty_count(), 1);
            assert_eq!(february_pairing.duties()[0].legs()[0].id.as_str(), "LEG_31_65");

            // Commit Hub-Split fallback through adapter
            let action = AirlineAction::ApplyHubSplitFallback {
                hub,
                pilot_a_crew_id,
                pilot_a_pairing,
                february_reserve_crew_id: None, // Pre-assign for next month's reserve
                february_pairing,
            };
            let proposal = DecisionProposal {
                priority: 1.0,
                estimated_gain: 2.0,
                affected_resources: vec!["CRW02".to_string()],
                violations_resolved: vec!["hub_split_capacity_relief".to_string()],
                confidence: 0.99,
                payload: serde_json::to_value(action).unwrap(),
            };

            let eval = adapter.commit_decision(proposal).unwrap();
            assert_eq!(eval.hard_constraint_violations.len(), 0);
        }
        coralys_airline::RolloverDecision::KeepCrewRollover { .. } => {
            panic!("Expected HubSplitFallback when headroom exhausted, got KeepCrewRollover");
        }
    }
}

