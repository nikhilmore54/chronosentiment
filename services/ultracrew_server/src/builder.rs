use std::collections::{HashMap, HashSet};
use ultracrew::inrc::models::{InrcContract, InrcNurse, InrcScenario};

// Phase 1: Skeleton Coverage
pub fn assign_skeleton(scenario: &InrcScenario) -> HashMap<String, Vec<String>> {
    let mut schedule = HashMap::new();
    let num_days = (scenario.number_of_weeks * 7) as usize;
    for nurse in &scenario.nurses {
        schedule.insert(nurse.id.clone(), vec!["".to_string(); num_days]);
    }

    let shift_types = vec!["E", "L", "N"];

    // For each day, we want exactly 6 E, 6 L, 4 N (Total 16 nurses per day out of 30, roughly 53% utilization)
    for day in 0..num_days {
        for &shift in &shift_types {
            let req = if shift == "N" { 4 } else { 6 };
            let mut assigned = 0;
            for nurse in &scenario.nurses {
                if assigned >= req {
                    break;
                }
                if schedule[&nurse.id][day].is_empty() {
                    schedule.get_mut(&nurse.id).unwrap()[day] = shift.to_string();
                    assigned += 1;
                }
            }
        }
    }
    schedule
}
