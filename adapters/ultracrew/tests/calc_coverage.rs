use std::collections::HashMap;
use std::fs;

fn main() {
    // 1. Read demands from WD
    let mut demands = HashMap::new(); // (Day, Shift, Skill) -> (min, opt)
    
    // Hardcoded from WD-n030w4-0.txt
    let days = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    
    // We will just hardcode the ones that have opt > 0
    let wd_content = fs::read_to_string("/Users/nikhil/.gemini/antigravity/brain/98bf099f-ccfe-46d1-a83f-6bcce5b3fb15/scratch/DynamicNurseScheduler/datasets/n030w4/WD-n030w4-0.txt").unwrap();
    for line in wd_content.lines() {
        if line.contains("(") && !line.starts_with("SHIFT") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                let shift = parts[0];
                let skill = parts[1];
                for d in 0..7 {
                    let req = parts[2 + d]; // e.g. "(1,2)"
                    let req = req.trim_matches(|c| c == '(' || c == ')');
                    let min_opt: Vec<&str> = req.split(',').collect();
                    let min: usize = min_opt[0].parse().unwrap();
                    let opt: usize = min_opt[1].parse().unwrap();
                    demands.insert((days[d].to_string(), shift.to_string(), skill.to_string()), (min, opt));
                }
            }
        }
    }
    
    // 2. Read assignments from sol
    let mut assigned = HashMap::new(); // (Day, Shift, Skill) -> count
    
    let sol_content = fs::read_to_string("adapters/ultracrew/tests/data/n030w4/sol-n030w4-equiv.txt").unwrap();
    for line in sol_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 4 && parts[0] != "ASSIGNMENTS" {
            // Nurse Day Shift Skill
            let day = parts[1].to_string();
            let shift = parts[2].to_string();
            let skill = parts[3].to_string();
            
            *assigned.entry((day, shift, skill)).or_insert(0) += 1;
        }
    }
    
    // 3. Calculate penalty
    let mut penalty_normal = 0;
    let mut penalty_no_double = 0;
    
    for ((day, shift, skill), (min, opt)) in &demands {
        let count = *assigned.get(&(day.clone(), shift.clone(), skill.clone())).unwrap_or(&0);
        
        if count < *opt {
            penalty_normal += (*opt - count) * 30;
        }
        
        let effective_count = std::cmp::max(count, *min);
        if effective_count < *opt {
            penalty_no_double += (*opt - effective_count) * 30;
        }
    }
    
    println!("Penalty Normal: {}", penalty_normal);
    println!("Penalty No Double (ignoring gap below min): {}", penalty_no_double);
}
