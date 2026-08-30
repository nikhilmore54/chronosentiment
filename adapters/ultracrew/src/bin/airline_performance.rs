use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::Write;
use serde::Serialize;

use coralys_moga::config::EvolutionConfig;
use ultracrew::models::{Worker, Shift, Skill};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::optimization::{ScheduleContext, Observatory};
use ultracrew::public_contracts::InrcScenario;
use ultracrew::helpers::run_optimization;

#[derive(Serialize)]
struct PerformanceReport {
    meta: ReportMeta,
    workloads: Vec<WorkloadMetrics>,
}

#[derive(Serialize)]
struct ReportMeta {
    title: String,
    description: String,
    optimizer_executed: bool,
    thresholds_enforced: bool,
}

#[derive(Serialize)]
struct WorkloadMetrics {
    id: String,
    workers: usize,
    shifts: usize,
    demands: usize,
    repetitions: usize,
    generation_time_ms: TimingStats,
    scheduling_time_ms: TimingStats,
    results: Vec<RunResult>,
}

#[derive(Serialize)]
struct TimingStats {
    n: usize,
    min: f64,
    max: f64,
    mean: f64,
    median: f64,
    std_dev: f64,
}

#[derive(Serialize, Clone)]
struct RunResult {
    iteration: usize,
    generation_ms: f64,
    scheduling_ms: f64,
    validity: bool,
    hard_violations: usize,
    fitness: f64,
}

fn calculate_stats(times: &[f64]) -> TimingStats {
    let n = times.len();
    if n == 0 {
        return TimingStats { n: 0, min: 0.0, max: 0.0, mean: 0.0, median: 0.0, std_dev: 0.0 };
    }
    
    let mut sorted = times.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let min = sorted[0];
    let max = sorted[n - 1];
    let mean = times.iter().sum::<f64>() / (n as f64);
    
    let median = if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };

    let variance = times.iter().map(|v| {
        let diff = mean - *v;
        diff * diff
    }).sum::<f64>() / (n as f64);
    
    TimingStats {
        n,
        min,
        max,
        mean,
        median,
        std_dev: variance.sqrt(),
    }
}

fn build_context(id: &str, num_workers: usize, num_shifts: usize, seed: u64) -> (Arc<ScheduleContext>, Duration) {
    let start = Instant::now();
    
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

    let ecology = WorkforceEcology::new();

    let scenario = InrcScenario {
        planning_horizon_hours: Some(if id == "stress" { 168.0 * 4.0 } else { 168.0 }),
        max_hours_per_worker: Some(40.0),
        minimum_rest_hours: Some(8),
        leave_requests: None,
    };

    let ctx = Arc::new(ScheduleContext {
        workers: Arc::new(workers),
        shifts: Arc::new(shifts),
        ecology,
        rng_seed: seed,
        observatory: Arc::new(Mutex::new(Observatory::new())),
        locked_assignments: None,
        scenario: Some(scenario),
        enable_fatigue: false,
        fatigue_weight: 0.0,
        hc3_aware_initialization: false, disable_global_constructor: false, precomputed_seeds: None, temporal_scarcity_construction: false,
        constructor_budget_ms: None,
    });
    
    (ctx, start.elapsed())
}

fn run_workload(id: &str, workers: usize, shifts: usize, reps: usize) -> WorkloadMetrics {
    let mut results = Vec::new();
    let mut gen_times = Vec::new();
    let mut sched_times = Vec::new();

    let config = EvolutionConfig {
        population_size: 100,
        generation_limit: 100, // Fixed generation limit across all runs
        mutation_rate: 0.2,
        crossover_rate: 0.8,
        elite_count: 5,
        ..Default::default()
    };

    println!("Running {} workload ({} duties) for {} iterations...", id, shifts, reps);
    for i in 0..reps {
        let (ctx, gen_dur) = build_context(id, workers, shifts, 42 + i as u64);
        let gen_ms = gen_dur.as_secs_f64() * 1000.0;
        
        let start_opt = Instant::now();
        let res = run_optimization(ctx, config.clone());
        let sched_ms = start_opt.elapsed().as_secs_f64() * 1000.0;
        
        let best = res.global_best;
        let hard = best.hc1_violations + best.hc2_violations + best.hc3_violations + best.rest_violations;
        
        gen_times.push(gen_ms);
        sched_times.push(sched_ms);
        
        results.push(RunResult {
            iteration: i + 1,
            generation_ms: gen_ms,
            scheduling_ms: sched_ms,
            validity: best.is_valid,
            hard_violations: hard,
            fitness: best.fitness,
        });
    }

    WorkloadMetrics {
        id: id.to_string(),
        workers,
        shifts,
        demands: shifts,
        repetitions: reps,
        generation_time_ms: calculate_stats(&gen_times),
        scheduling_time_ms: calculate_stats(&sched_times),
        results,
    }
}

fn main() {
    println!("P2 UC-AIR-001 Performance Hardening - Benchmark Run\n");

    let mut workloads = Vec::new();
    
    workloads.push(run_workload("small", 50, 100, 5));
    workloads.push(run_workload("medium", 200, 1000, 3));
    workloads.push(run_workload("stress", 1000, 5000, 3));

    let report = PerformanceReport {
        meta: ReportMeta {
            title: "P2 Airline Performance Baseline".to_string(),
            description: "No performance acceptance threshold is established by this characterization run.".to_string(),
            optimizer_executed: true,
            thresholds_enforced: false,
        },
        workloads,
    };

    let json_str = serde_json::to_string_pretty(&report).unwrap();
    let mut file = File::create("airline_performance_report.json").unwrap();
    file.write_all(json_str.as_bytes()).unwrap();

    println!("\nReport generated: airline_performance_report.json");
}
