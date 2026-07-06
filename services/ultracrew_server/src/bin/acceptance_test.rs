use std::fs::File;
use std::io::Write;
use serde::Serialize;
use ultracrew::inrc::models::{InrcScenario, InrcNurse, InrcContract, InrcShiftType, InrcForbiddenSuccession};
use ultracrew_server::simulation::{generate_baseline_schedule};
use ultracrew_server::validator::validate_schedule;

#[derive(Serialize)]
struct BenchmarkResult {
    total_runs: usize,
    integrity_failures: usize,
    legal_schedules: usize,
    repair_failed: usize,
    avg_min_work_violations: f64,
    avg_max_work_violations: f64,
    avg_min_off_violations: f64,
    avg_max_off_violations: f64,
    mean_deviation: f64,
    max_deviation: f64,
    avg_1_shift_islands: f64,
    avg_2_shift_islands: f64,
    avg_longest_off_streak: f64,
    avg_daily_coverage_deficits: f64,
    ui_consistency_failures: usize,
}

fn main() {
    let mut scenario = InrcScenario {
        id: "acceptance_benchmark".to_string(),
        number_of_weeks: 8,
        skills: vec![],
        shift_types: vec![],
        forbidden_shift_type_successions: vec![],
        contracts: vec![
            InrcContract {
                id: "FullTime".to_string(),
                min_assignments: 20,
                max_assignments: 24,
                min_consecutive_working_days: 2,
                max_consecutive_working_days: 5,
                min_consecutive_days_off: 2,
                max_consecutive_days_off: 4,
                max_working_weekends: 2,
                complete_weekends: 1,
            }
        ],
        nurses: vec![],
    };
    
    for i in 0..15 {
        let name = if i < 4 { format!("HN_{}", i) } else { format!("NU_{}", i) };
        scenario.nurses.push(InrcNurse {
            id: name,
            contract: "FullTime".to_string(),
            skills: vec![],
        });
    }

    let total_runs = 100;
    
    let mut integrity_failures = 0;
    let mut legal_schedules = 0;
    let mut min_work_sum = 0;
    let mut max_work_sum = 0;
    let mut min_off_sum = 0;
    let mut max_off_sum = 0;
    
    let mut global_dev_sum = 0;
    let mut global_max_dev = 0;
    let mut dev_count = 0;
    
    let mut sum_1_islands = 0;
    let mut sum_2_islands = 0;
    let mut sum_longest_off = 0;
    
    let mut ui_consistency_failures = 0;
    let mut total_coverage_deficits = 0;

    
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../adapters/ultracrew/tests/data/n030w4");
    let week_data = ultracrew::inrc::parser::parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let requirements = &week_data.requirements;

    for _ in 0..total_runs {
        let schedule_res = generate_baseline_schedule(&scenario, requirements);
        if schedule_res.is_err() { continue; }
        let genome = schedule_res.unwrap();
        let schedule = genome.to_flat_schedule();
        
        let report = validate_schedule(&schedule, &scenario);

        if report.is_legal {
            legal_schedules += 1;
        }
        
        // Integrity Check
        if schedule.len() != scenario.nurses.len() { integrity_failures += 1; }
        for n in &scenario.nurses {
            if schedule[&n.id].len() != 56 { integrity_failures += 1; }
        }
        
        // Constraints
        let mut lw = 0;
        let mut hw = 0;
        let mut lo = 0;
        let mut ho = 0;
        for det in &report.details {
            if det.constraint == "min_consecutive_working_days" { lw += 1; }
            if det.constraint == "max_consecutive_working_days" { hw += 1; }
            if det.constraint == "min_consecutive_days_off" { lo += 1; }
            if det.constraint == "max_consecutive_days_off" { ho += 1; }
        }
        min_work_sum += lw;
        max_work_sum += hw;
        min_off_sum += lo;
        max_off_sum += ho;
        
        // Workload & Fragmentation
        let mut actual = 0;
        let mut daily = vec![0; 56];
        for shifts in schedule.values() {
            let expected = 56; // Mock expected workload per 56 days
            let mut curr_work = 0;
            let mut curr_off = 0;
            let mut nurse_1_islands = 0;
            let mut nurse_2_islands = 0;
            let mut nurse_max_off = 0;
            let mut assigned_shifts = 0;
            
            for d in 0..56 {
                if !shifts[d].is_empty() {
                    actual += 1;
                    daily[d] += 1;
                    assigned_shifts += 1;
                    
                    if curr_off > nurse_max_off { nurse_max_off = curr_off; }
                    curr_off = 0;
                    curr_work += 1;
                } else {
                    if curr_work == 1 { nurse_1_islands += 1; }
                    if curr_work == 2 { nurse_2_islands += 1; }
                    curr_work = 0;
                    curr_off += 1;
                }
            }
            if curr_off > nurse_max_off { nurse_max_off = curr_off; }
            if curr_work == 1 { nurse_1_islands += 1; }
            if curr_work == 2 { nurse_2_islands += 1; }
            
            let dev = (assigned_shifts as i32) - expected;
            global_dev_sum += dev.abs();
            if dev.abs() > global_max_dev { global_max_dev = dev.abs(); }
            dev_count += 1;
            
            sum_1_islands += nurse_1_islands;
            sum_2_islands += nurse_2_islands;
            sum_longest_off += nurse_max_off;
        }
        
        let mut coverage_deficits = 0;
        for d in 0..56 {
            if daily[d] < 16 {
                coverage_deficits += 16 - daily[d];
            }
        }
        total_coverage_deficits += coverage_deficits;
        
        // UI Consistency
        if (actual as f64 / (16.0 * 56.0)) > 1.0 { ui_consistency_failures += 1; }
    }

    let benchmark = BenchmarkResult {
        total_runs,
        integrity_failures,
        legal_schedules,
        repair_failed: total_runs - legal_schedules,
        avg_min_work_violations: min_work_sum as f64 / total_runs as f64,
        avg_max_work_violations: max_work_sum as f64 / total_runs as f64,
        avg_min_off_violations: min_off_sum as f64 / total_runs as f64,
        avg_max_off_violations: max_off_sum as f64 / total_runs as f64,
        mean_deviation: global_dev_sum as f64 / dev_count as f64,
        max_deviation: global_max_dev as f64,
        avg_1_shift_islands: sum_1_islands as f64 / total_runs as f64,
        avg_2_shift_islands: sum_2_islands as f64 / total_runs as f64,
        avg_longest_off_streak: sum_longest_off as f64 / (total_runs * scenario.nurses.len()) as f64,
        avg_daily_coverage_deficits: total_coverage_deficits as f64 / total_runs as f64,
        ui_consistency_failures,
    };

    let json = serde_json::to_string_pretty(&benchmark).unwrap();
    let mut file = File::create("../../artifacts/acceptance_benchmark.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();

    let md = format!(
        "# Coralys Acceptance Benchmark\n\n\
        ## Schedule Integrity\n\
        - Integrity Failures (truncated rosters): {}\n\n\
        ## Constraint Completeness\n\
        - Legal Schedules: {} / {}\n\
        - Repair Failed: {}\n\
        - Avg Min Work Violations: {:.2}\n\
        - Avg Max Work Violations: {:.2}\n\
        - Avg Min Off Violations: {:.2}\n\
        - Avg Max Off Violations: {:.2}\n\n\
        ## Workload Equilibrium\n\
        - Mean Deviation (abs): {:.2} shifts\n\
        - Max Deviation: {} shifts\n\n\
        ## Fragmentation Audit\n\
        - Avg 1-Shift Islands: {:.2}\n\
        - Avg 2-Shift Islands: {:.2}\n\
        - Avg Longest Off Streak: {:.2} days\n\n\
        ## Daily Coverage Audit\n\
        - Avg Total Missing Shifts across 56 days: {:.2}\n\n\
        ## UI Consistency\n\
        - Math Discrepancies: {}\n",
        benchmark.integrity_failures,
        benchmark.legal_schedules, benchmark.total_runs,
        benchmark.repair_failed,
        benchmark.avg_min_work_violations,
        benchmark.avg_max_work_violations,
        benchmark.avg_min_off_violations,
        benchmark.avg_max_off_violations,
        benchmark.mean_deviation,
        benchmark.max_deviation,
        benchmark.avg_1_shift_islands,
        benchmark.avg_2_shift_islands,
        benchmark.avg_longest_off_streak,
        benchmark.avg_daily_coverage_deficits,
        benchmark.ui_consistency_failures
    );
    let mut md_file = File::create("../../artifacts/acceptance_benchmark.md").unwrap();
    md_file.write_all(md.as_bytes()).unwrap();
}
