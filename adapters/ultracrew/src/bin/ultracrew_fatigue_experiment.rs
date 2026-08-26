//! Ultracrew Fatigue Candidate Mapping Experiment
//! This binary runs deterministic candidates to observe historical fatigue and assigned hours.
//! It does NOT modify any optimizer behavior.

use std::collections::HashMap;

use serde_json::json;
use serde_json::Value;
use std::process::Command;
use ultracrew::config::fatigue_config::FatigueConfig;
use ultracrew::constraint_engine::DomainConstraintEvaluator;
use ultracrew::constraint_engine::InrcConstraintEvaluator;
use ultracrew::models::Skill;
use ultracrew::models::{Shift, Worker};
use ultracrew::optimization::ScheduleGenome;
use ultracrew::public_contracts::ScheduleRequest;

fn build_request(
    fatigue_hours: f64,
    num_assignments: usize,
    fatigue_cfg: FatigueConfig,
) -> (ScheduleRequest, Vec<Shift>) {
    // Single generic skill
    let generic_skill = Skill::new("generic");
    // One worker
    let worker = Worker {
        id: 1,
        skills: vec![generic_skill.clone()],
    };
    // Create a set of up to 4 shifts (max needed for candidate F)
    let mut shifts = Vec::new();
    for i in 0..4 {
        let shift = Shift {
            id: i as u64,
            start_hour: i as u64 * 8,
            duration_hours: 8,
            required_skill: generic_skill.clone(),
        };
        shifts.push(shift);
    }
    // Historical workloads mapping worker_id -> weekly hours vector (4 weeks)
    let mut historical = HashMap::new();
    // Use the same value for each week to achieve the intended mean
    historical.insert(1u64, vec![fatigue_hours; 4]);

    let request = ScheduleRequest {
        workers: vec![worker],
        shifts: shifts.clone(),
        historical_workloads: Some(historical),
        rng_seed: None,
        generation_limit: None,
        scenario: None,
        fatigue: fatigue_cfg,
    };
    (request, shifts)
}

fn build_genome(num_assignments: usize) -> ScheduleGenome {
    let mut assignments = HashMap::new();
    for i in 0..num_assignments {
        assignments.insert(i as u64, 1u64); // assign to worker 1
    }
    ScheduleGenome { assignments }
}

/// Helper to collect experiment provenance metadata
fn record_metadata(scenario_id: &str, seed: usize, arm: &str, fatigue_weight: f64) -> Value {
    // Git commit SHA
    let git_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to get git commit")
        .stdout;
    let git_commit = String::from_utf8(git_commit)
        .unwrap_or_default()
        .trim()
        .to_string();

    // Git dirty status
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("Failed to get git status")
        .stdout;
    let git_dirty = !String::from_utf8(status_output)
        .unwrap_or_default()
        .trim()
        .is_empty();

    // Toolchain versions
    let rustc_version = String::from_utf8(
        Command::new("rustc")
            .arg("--version")
            .output()
            .expect("Failed to get rustc version")
            .stdout,
    )
    .unwrap_or_default()
    .trim()
    .to_string();

    let cargo_version = String::from_utf8(
        Command::new("cargo")
            .arg("--version")
            .output()
            .expect("Failed to get cargo version")
            .stdout,
    )
    .unwrap_or_default()
    .trim()
    .to_string();

    // Build JSON metadata object
    json!({
        "git_commit": git_commit,
        "git_dirty": git_dirty,
        "rustc_version": rustc_version,
        "cargo_version": cargo_version,
        "seed": seed,
        "arm": arm,
        "enable_fatigue": fatigue_weight != 0.0,
        "fatigue_weight": fatigue_weight,
        "scenario": scenario_id,
        "experimental_default_weight": fatigue_weight,
    })
}

fn main() {
    // Parse optional default fatigue weight from CLI args
    let args: Vec<String> = std::env::args().collect();
    let mut default_weight_opt: Option<f64> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--default-fatigue-weight" && i + 1 < args.len() {
            if let Ok(w) = args[i + 1].parse::<f64>() {
                default_weight_opt = Some(w);
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    let default_weight = match default_weight_opt {
        Some(w) => {
            if w == 0.0 {
                eprintln!("Configured default fatigue_weight is 0.0 – aborting experiment as per governance.");
                std::process::exit(1);
            }
            w
        }
        None => {
            eprintln!("--default-fatigue-weight is required to run M2-A experiment. Provide a non‑zero value.");
            std::process::exit(1);
        }
    };

    // Define seeds (5 seeds as per approved design)
    let seeds = vec![42usize, 43, 44, 45, 46];

    // Header for TSV output (including metadata columns)
    println!("scenario_id\tseed\tarm\tfatigue_weight\thistorical_fatigue_off\thistorical_fatigue_on\tsc2_penalty_off\tsc2_penalty_on\tsc2_delta\tmetadata_json");

    for seed in seeds {
        // Arm A: fatigue disabled (control)
        let cfg_a = FatigueConfig {
            enable_fatigue: false,
            fatigue_weight: 0.0,
        };
        let (req_a, _) = build_request(0.0, 0, cfg_a.clone());
        let ctx_a = req_a.to_context();
        let hist_fatigue_off = ctx_a.ecology.get_historical_fatigue(1);
        let sc2_penalty_off = 0.0;
        let metadata_a = record_metadata("M2A", seed, "A", 0.0);
        println!(
            "M2A\t{}\tA\t0.0\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{}",
            seed,
            hist_fatigue_off,
            hist_fatigue_off,
            sc2_penalty_off,
            sc2_penalty_off,
            0.0,
            metadata_a.to_string()
        );

        // Arm B: fatigue enabled with configured default weight
        let cfg_b = FatigueConfig {
            enable_fatigue: true,
            fatigue_weight: default_weight,
        };
        let (req_b, _) = build_request(40.0, 0, cfg_b.clone());
        let ctx_b = req_b.to_context();
        let hist_fatigue_on = ctx_b.ecology.get_historical_fatigue(1);
        let evaluator_b = InrcConstraintEvaluator::new(ctx_b.clone());
        let report_b = evaluator_b.evaluate(&build_genome(0));
        let sc2_penalty_on = report_b.fatigue_penalty;
        let sc2_delta = sc2_penalty_on - sc2_penalty_off;
        let metadata_b = record_metadata("M2A", seed, "B", default_weight);
        println!(
            "M2A\t{}\tB\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{}",
            seed,
            default_weight,
            hist_fatigue_off,
            hist_fatigue_on,
            sc2_penalty_off,
            sc2_penalty_on,
            sc2_delta,
            metadata_b.to_string()
        );

        // Arm C: swept weights
        for &wt in &[0.25_f64, 0.50, 1.0, 2.0] {
            let cfg_c = FatigueConfig {
                enable_fatigue: true,
                fatigue_weight: wt,
            };
            let (req_c, _) = build_request(40.0, 0, cfg_c.clone());
            let ctx_c = req_c.to_context();
            let hist_fatigue_c = ctx_c.ecology.get_historical_fatigue(1);
            let evaluator_c = InrcConstraintEvaluator::new(ctx_c.clone());
            let report_c = evaluator_c.evaluate(&build_genome(0));
            let sc2_penalty_c = report_c.fatigue_penalty;
            let delta_c = sc2_penalty_c - sc2_penalty_off;
            let metadata_c = record_metadata("M2A", seed, "C", wt);
            println!(
                "M2A\t{}\tC\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{}",
                seed,
                wt,
                hist_fatigue_off,
                hist_fatigue_c,
                sc2_penalty_off,
                sc2_penalty_c,
                delta_c,
                metadata_c.to_string()
            );
        }
    }
}
