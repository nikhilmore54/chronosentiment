use std::collections::HashMap;
use std::fs;

fn main() {
    let mut demands = HashMap::new();
    let days = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let wd_content = fs::read_to_string("/Users/nikhil/.gemini/antigravity/brain/98bf099f-ccfe-46d1-a83f-6bcce5b3fb15/scratch/DynamicNurseScheduler/datasets/n030w4/WD-n030w4-0.txt").unwrap();
    for line in wd_content.lines() {
        if line.contains("(") && !line.starts_with("SHIFT") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let shift = parts[0];
                let skill = parts[1];
                for d in 0..7 {
                    let req = parts[2 + d];
                    let req = req.trim_matches(|c| c == '(' || c == ')');
                    let min_opt: Vec<&str> = req.split(',').collect();
                    let min: usize = min_opt[0].parse().unwrap();
                    let opt: usize = min_opt[1].parse().unwrap();
                    demands.insert(
                        (days[d].to_string(), shift.to_string(), skill.to_string()),
                        (min, opt),
                    );
                }
            }
        }
    }

    let mut penalty_no_double = 0;
    for (_, (min, opt)) in &demands {
        let count = 0; // ZERO ASSIGNMENTS
        let effective_count = std::cmp::max(count, *min);
        if effective_count < *opt {
            penalty_no_double += (*opt - effective_count) * 30;
        }
    }
    println!("Penalty No Double (0 assignments): {}", penalty_no_double);
}
