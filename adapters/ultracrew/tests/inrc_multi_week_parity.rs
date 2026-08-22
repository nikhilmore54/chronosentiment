use std::process::Command;
use std::path::PathBuf;
use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer, InrcGenome};
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::history::extract_next_history;
use coralys_moga::FitnessEvaluator;
use std::sync::Arc;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

struct ParityScheduleGenerator {
    num_nurses: usize,
    num_days: usize,
    num_shifts: usize,
    rng: StdRng,
}

impl ParityScheduleGenerator {
    fn new(num_nurses: usize, num_days: usize, num_shifts: usize, seed: u64) -> Self {
        Self {
            num_nurses,
            num_days,
            num_shifts,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn generate(&mut self) -> InrcGenome {
        let total_bits = self.num_nurses * self.num_days * self.num_shifts;
        let mut bits = vec![false; total_bits];
        
        for n in 0..self.num_nurses {
            for d in 0..self.num_days {
                if self.rng.gen_bool(0.5) {
                    let shift_idx = self.rng.gen_range(0..self.num_shifts);
                    let idx = n * (self.num_days * self.num_shifts) + d * self.num_shifts + shift_idx;
                    bits[idx] = true;
                }
            }
        }
        
        InrcGenome { bits }
    }
}

#[test]
fn test_inrc_multi_week_parity() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let mut current_history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let ecology = WorkforceEcology::new();
    
    let num_nurses = scenario.nurses.len();
    let num_days = 7;
    let num_shifts = scenario.shift_types.len();
    let mut generator = ParityScheduleGenerator::new(num_nurses, num_days, num_shifts, 9999);

    let validator_jar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/validator.jar");
    let mut val_days_off_total = 0;
    let mut val_preferences_total = 0;
    let mut val_weekends_total = 0;
    let mut val_consecutive_total = 0;
    let mut val_optimal_total = 0;

    let mut coralys_days_off_total = 0;
    let mut coralys_preferences_total = 0;
    let mut coralys_weekends_total = 0;
    let mut coralys_consecutive_total = 0;
    let mut coralys_optimal_total = 0;

    for w in 0..4 {
        let wd_path = base_dir.join(format!("WD-n030w4-{}.json", w));
        let week_data = parse_week_data(wd_path).unwrap();
        
        let context = InrcContext::new(scenario.clone(), week_data, current_history.clone(), ecology.clone());
        let optimizer = InrcOptimizer { context: Arc::new(context) };
        
        let genome = generator.generate();
        let metric_report = coralys_moga::runtime::optimization::metric::MetricReport::default();
        let evaluation = optimizer.evaluate(&genome, &metric_report);
        
        coralys_days_off_total += evaluation.soft_report.day_off_penalty;
        coralys_preferences_total += evaluation.soft_report.preferences_penalty;
        coralys_weekends_total += evaluation.soft_report.weekend_penalty;
        coralys_consecutive_total += evaluation.soft_report.work_streak_penalty;
        coralys_optimal_total += evaluation.soft_report.optimal_coverage_penalty;
        
        let out_sol_path = base_dir.join(format!("sol-n030w4-w{}.txt", w));
        ultracrew::inrc::exporter::export_inrc_solution(&genome, optimizer.context.clone(), 0, &out_sol_path).unwrap();
        
        let out_his_path = base_dir.join(format!("his-n030w4-w{}.txt", w));
        if w == 0 {
            // First week uses H0-n030w4-0.txt directly
        } else {
            ultracrew::inrc::exporter::export_inrc_history(&current_history, &out_his_path).unwrap();
        }
        
        let his_arg = if w == 0 {
            base_dir.join("H0-n030w4-0.txt")
        } else {
            out_his_path.clone()
        };
        
        let wd_arg = base_dir.join(format!("WD-n030w4-{}.txt", w));

        let output = Command::new("java")
            .arg("-jar").arg(&validator_jar)
            .arg("--sce").arg(base_dir.join("Sc-n030w4-tmp.txt"))
            .arg("--his").arg(&his_arg)
            .arg("--weeks").arg(&wd_arg)
            .arg("--sols").arg(&out_sol_path)
            .output().expect("Failed to execute validator");
            
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        println!("Validator Output:\n{}", stdout);
        let mut val_days_off = -1;
        let mut val_preferences = -1;
        let mut val_weekends = -1;
        let mut val_optimal = -1;
        let mut val_consecutive = -1;
        
        for line in stdout.lines() {
            if line.starts_with("Non working days constraints:") { val_days_off = line.split(":").nth(1).unwrap().trim().parse().unwrap(); }
            else if line.starts_with("Preferences:") { val_preferences = line.split(":").nth(1).unwrap().trim().parse().unwrap(); }
            else if line.starts_with("Complete weekends:") { val_weekends = line.split(":").nth(1).unwrap().trim().parse().unwrap(); }
            else if line.starts_with("Optimal coverage constraints:") { val_optimal = line.split(":").nth(1).unwrap().trim().parse().unwrap(); }
            else if line.starts_with("Consecutive constraints:") { val_consecutive = line.split(":").nth(1).unwrap().trim().parse().unwrap(); }
        }
        
        val_days_off_total += val_days_off;
        val_preferences_total += val_preferences;
        val_weekends_total += val_weekends;
        val_consecutive_total += val_consecutive;
        val_optimal_total += val_optimal;
        println!("Week {} Validator DaysOff: {}, Coralys: {}", w, val_days_off, evaluation.soft_report.day_off_penalty);

        current_history = extract_next_history(&optimizer.context, &genome);
    }
    
    println!("--- CUMULATIVE WEEK-BY-WEEK PARITY ---");
    println!("Constraint           | Coralys | Validator ");
    println!("---------------------|---------|-----------");
    println!("Days Off             | {:<7} | {:<9} ", coralys_days_off_total, val_days_off_total);
    println!("Preferences          | {:<7} | {:<9} ", coralys_preferences_total, val_preferences_total);
    println!("Complete Weekends    | {:<7} | {:<9} ", coralys_weekends_total, val_weekends_total);
    println!("Consecutive Work     | {:<7} | {:<9} ", coralys_consecutive_total, val_consecutive_total);
    println!("Optimal Coverage     | {:<7} | {:<9} ", coralys_optimal_total, val_optimal_total);

    assert_eq!(coralys_days_off_total, val_days_off_total, "Days off mismatch!");
    assert_eq!(coralys_preferences_total, val_preferences_total, "Preferences mismatch!");
    assert_eq!(coralys_weekends_total, val_weekends_total, "Complete weekends mismatch!");
    assert_eq!(coralys_consecutive_total, val_consecutive_total, "Consecutive mismatch!");
    assert_eq!(coralys_optimal_total, val_optimal_total, "Optimal coverage mismatch!");
}
