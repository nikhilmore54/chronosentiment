use std::process::Command;
use std::path::PathBuf;
use ultracrew::inrc::parser::{parse_scenario, parse_history, parse_week_data};
use ultracrew::inrc::optimization::{InrcContext, InrcOptimizer, InrcGenome};
use ultracrew::ecology::WorkforceEcology;
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

    fn generate(&mut self, bucket: &str) -> InrcGenome {
        let total_bits = self.num_nurses * self.num_days * self.num_shifts;
        let mut bits = vec![false; total_bits];
        
        for n in 0..self.num_nurses {
            for d in 0..self.num_days {
                let work_prob = match bucket {
                    "sparse" => 0.1,
                    "medium" => 0.5,
                    "dense" => 0.9,
                    "pathological" => {
                        // e.g. alternate days, or weekends only
                        if n % 2 == 0 {
                            if d % 2 == 0 { 0.9 } else { 0.1 }
                        } else {
                            if d >= 5 { 1.0 } else { 0.0 }
                        }
                    },
                    _ => 0.5,
                };
                
                if self.rng.gen_bool(work_prob) {
                    // Pick exactly one shift randomly
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
fn test_inrc_full_objective_parity() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();

    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario, week_data, history, ecology);

    let optimizer = InrcOptimizer {
        context: Arc::new(context),
    };
    
    let num_nurses = optimizer.context.num_nurses;
    let num_days = optimizer.context.num_days;
    let num_shifts = optimizer.context.shift_types.len();

    let mut generator = ParityScheduleGenerator::new(num_nurses, num_days, num_shifts, 12345);
    
    let buckets = vec!["sparse", "medium", "dense", "pathological"];
    let count_per_bucket = 125;
    let mut total_passed = 0;
    let total_schedules = buckets.len() * count_per_bucket;
    
    let validator_jar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/validator.jar");
    let out_path = base_dir.join("sol-rand-test.txt");

    for bucket in &buckets {
        println!("Testing bucket: {}", bucket);
        for i in 0..count_per_bucket {
            let genome = generator.generate(bucket);
            let evaluation = optimizer.evaluate(&genome);
            
            ultracrew::inrc::exporter::export_inrc_solution(&genome, optimizer.context.clone(), 0, &out_path).unwrap();
            
            let output = Command::new("java")
                .arg("-jar")
                .arg(&validator_jar)
                .arg("--sce")
                .arg(base_dir.join("Sc-n030w4-tmp.txt"))
                .arg("--his")
                .arg("/Users/nikhil/.gemini/antigravity/brain/98bf099f-ccfe-46d1-a83f-6bcce5b3fb15/scratch/DynamicNurseScheduler/datasets/n030w4/H0-n030w4-0.txt")
                .arg("--weeks")
                .arg("/Users/nikhil/.gemini/antigravity/brain/98bf099f-ccfe-46d1-a83f-6bcce5b3fb15/scratch/DynamicNurseScheduler/datasets/n030w4/WD-n030w4-0.txt")
                .arg("--sols")
                .arg(&out_path)
                .output()
                .expect("Failed to execute validator");

            let stdout = String::from_utf8_lossy(&output.stdout);
            
            let mut val_days_off = -1;
            let mut val_preferences = -1;
            let mut val_weekends = -1;
            let mut val_optimal = -1;
            let mut val_consecutive = -1;
            
            for line in stdout.lines() {
                if line.starts_with("Non working days constraints:") {
                    val_days_off = line.split(":").nth(1).unwrap().trim().parse().unwrap();
                } else if line.starts_with("Preferences:") {
                    val_preferences = line.split(":").nth(1).unwrap().trim().parse().unwrap();
                } else if line.starts_with("Complete weekends:") {
                    val_weekends = line.split(":").nth(1).unwrap().trim().parse().unwrap();
                } else if line.starts_with("Optimal coverage constraints:") {
                    val_optimal = line.split(":").nth(1).unwrap().trim().parse().unwrap();
                } else if line.starts_with("Consecutive constraints:") {
                    val_consecutive = line.split(":").nth(1).unwrap().trim().parse().unwrap();
                }
            }

            let pass_days_off = evaluation.soft_report.day_off_penalty == val_days_off as i32;
            let pass_prefs = evaluation.soft_report.preferences_penalty == val_preferences as i32;
            let pass_weekends = evaluation.soft_report.weekend_penalty == val_weekends as i32;
            let pass_consec = evaluation.soft_report.work_streak_penalty == val_consecutive as i32;
            let pass_opt = evaluation.soft_report.optimal_coverage_penalty == val_optimal as i32;

            if pass_days_off && pass_prefs && pass_weekends && pass_consec && pass_opt {
                total_passed += 1;
            } else {
                println!("--- SCORE EQUIVALENCE PARITY FAILURE in {} #{} ---", bucket, i);
                println!("Constraint           | Coralys | Validator ");
                println!("---------------------|---------|-----------");
                println!("Days Off             | {:<7} | {:<9} ", evaluation.soft_report.day_off_penalty, val_days_off);
                println!("Preferences          | {:<7} | {:<9} ", evaluation.soft_report.preferences_penalty, val_preferences);
                println!("Complete Weekends    | {:<7} | {:<9} ", evaluation.soft_report.weekend_penalty, val_weekends);
                println!("Consecutive Work     | {:<7} | {:<9} ", evaluation.soft_report.work_streak_penalty, val_consecutive);
                println!("Optimal Coverage     | {:<7} | {:<9} ", evaluation.soft_report.optimal_coverage_penalty, val_optimal);
                panic!("Parity failed on {} schedule {}", bucket, i);
            }
        }
    }
    
    println!("Total Passed: {} / {}", total_passed, total_schedules);
    assert_eq!(total_passed, total_schedules, "Not all parity schedules passed!");
}
