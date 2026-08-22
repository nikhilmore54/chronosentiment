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

fn main() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();

    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario, week_data, history, ecology);
    println!("num_days: {}", context.num_days);

    let optimizer = InrcOptimizer {
        context: Arc::new(context),
    };
    
    let num_nurses = optimizer.context.num_nurses;
    let num_days = optimizer.context.num_days;
    let num_shifts = optimizer.context.shift_types.len();

    let mut generator = ParityScheduleGenerator::new(num_nurses, num_days, num_shifts, 12345);
    
    let bucket = "sparse";
    println!("Testing bucket: {}", bucket);
    
    let genome = generator.generate(bucket);
    let evaluation = optimizer.evaluate(&genome, &coralys_moga::runtime::optimization::metric::MetricReport::default());
    
    let out_path = base_dir.join("sol-rand-test-debug.txt");
    ultracrew::inrc::exporter::export_inrc_solution(&genome, optimizer.context.clone(), 0, &out_path).unwrap();
    
    let validator_jar = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/validator.jar");
    let output = Command::new("java")
        .arg("-jar")
        .arg(&validator_jar)
        .arg("--sce")
        .arg(base_dir.join("Sc-n030w4-tmp.txt"))
        .arg("--his")
        .arg(base_dir.join("H0-n030w4-0.txt"))
        .arg("--weeks")
        .arg(base_dir.join("WD-n030w4-0.txt"))
        .arg("--sols")
        .arg(&out_path)
        .output()
        .expect("Failed to execute validator");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    println!("--- VALIDATOR OUTPUT ---");
    println!("{}", stdout);
    
    println!("--- CORALYS EVALUATION ---");
    println!("Consecutive work penalty: {}", evaluation.soft_report.work_streak_penalty);
    println!("Preferences penalty: {}", evaluation.soft_report.preferences_penalty);
    println!("Complete weekends penalty: {}", evaluation.soft_report.weekend_penalty);
    println!("Optimal coverage penalty: {}", evaluation.soft_report.optimal_coverage_penalty);
}
