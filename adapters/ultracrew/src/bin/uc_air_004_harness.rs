use std::env;
use std::process;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use serde_json::json;

use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::public_contracts::InrcScenario;

#[derive(Serialize)]
struct HarnessReport {
    uc_air_004: HarnessMeta,
    scenarios: Vec<ScenarioReport>,
}

#[derive(Serialize)]
struct HarnessMeta {
    mode: String,
    health_gate_required: bool,
    optimizer_executed: bool,
}

#[derive(Serialize)]
struct ScenarioReport {
    id: String,
    status: String,
    workers: usize,
    shifts: usize,
    demands: usize,
    fatigue_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    fatigue_weight: Option<f64>,
    optimizer_execution: String,
    legality: String,
    fitness: String,
}

fn build_context(id: &str, num_workers: usize, num_shifts: usize, enable_fatigue: bool, fatigue_weight: f64) -> Arc<ScheduleContext> {
    let skill = Skill::new("FlightAttendant");

    let mut workers = vec![];
    for i in 0..num_workers {
        workers.push(Worker { id: (i + 1) as u64, skills: vec![skill.clone()] });
    }

    let mut shifts = vec![];
    for i in 0..num_shifts {
        shifts.push(Shift {
            id: (i + 1) as u64,
            start_hour: (i * 8) as u64 % 168,
            duration_hours: 8,
            required_skill: skill.clone(),
        });
    }

    let mut ecology = WorkforceEcology::new();
    if enable_fatigue {
        // Constrain workforce: make half of them highly fatigued
        for i in 0..(num_workers / 2) {
            ecology.record_historical_hours((i + 1) as u64, 40.0);
        }
    }

    let scenario = InrcScenario {
        planning_horizon_hours: Some(if id == "stress" { 168.0 * 4.0 } else { 168.0 }),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    };

    Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
        rng_seed: 42,
        observatory: Arc::new(Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario: Some(scenario),
        enable_fatigue,
        fatigue_weight,
        hc3_aware_initialization: false,
        temporal_scarcity_construction: false,
        disable_global_constructor: false,
        precomputed_seeds: None,
    })
}

fn generate_scenario_report(id: &str, ctx: &ScheduleContext) -> ScenarioReport {
    ScenarioReport {
        id: id.to_string(),
        status: "constructed".to_string(),
        workers: ctx.workers.len(),
        shifts: ctx.shifts.len(),
        demands: ctx.shifts.len(),
        fatigue_enabled: ctx.enable_fatigue,
        fatigue_weight: if ctx.enable_fatigue { Some(ctx.fatigue_weight) } else { None },
        optimizer_execution: "NOT RUN".to_string(),
        legality: "NOT EVALUATED".to_string(),
        fitness: "NOT EVALUATED".to_string(),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let execute = args.contains(&"--execute".to_string());

    if !dry_run && !execute {
        eprintln!("Usage: uc_air_004_harness [--dry-run | --execute]");
        process::exit(1);
    }

    if execute {
        // Health gate check
        eprintln!("ERROR: UC-AIR-004 execution is BLOCKED by the health gate.");
        process::exit(1);
    }

    // Dry-run mode
    let baseline_ctx = build_context("baseline", 50, 100, false, 0.0);
    let edge_ctx = build_context("edge", 100, 500, true, 320.0);
    let stress_ctx = build_context("stress", 1000, 5000, false, 0.0);

    let report = HarnessReport {
        uc_air_004: HarnessMeta {
            mode: "dry-run".to_string(),
            health_gate_required: true,
            optimizer_executed: false,
        },
        scenarios: vec![
            generate_scenario_report("baseline", &baseline_ctx),
            generate_scenario_report("edge", &edge_ctx),
            generate_scenario_report("stress", &stress_ctx),
        ],
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
