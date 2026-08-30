use std::collections::HashMap;
use ultracrew::models::{Worker, Shift, Skill};
use rand::{rngs::StdRng, SeedableRng, Rng};

fn main() {
    let num_workers = 200;
    let num_shifts = 1000;
    let seed = 42;
    let min_rest = 8;
    let hc3_limit = 40;

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

    // Sort shifts by start_hour as the factory does
    shifts.sort_by_key(|s| s.start_hour);

    let mut rng = StdRng::seed_from_u64(seed);
    let mut worker_assigned: HashMap<u64, Vec<Shift>> = HashMap::new();
    
    let mut clean_assignments = 0;
    let mut fallback_assignments = 0;
    let mut first_empty_pool_shift = None;
    let mut skill_shortages = 0;

    // Track violations strictly attributable to fallback
    let mut fallback_hc2 = 0;
    let mut fallback_rest = 0;
    let mut fallback_hc3_exceedances = 0;

    for (shift_idx, shift) in shifts.iter().enumerate() {
        let shift_end = shift.start_hour + shift.duration_hours;
        
        let clean: Vec<u64> = workers.iter()
            .filter(|w| w.skills.contains(&shift.required_skill))
            .filter(|w| {
                match worker_assigned.get(&w.id) {
                    None => true,
                    Some(assigned) => {
                        let mut no_overlap = true;
                        let mut rest_ok = true;
                        let mut current_hours = 0;
                        
                        for a in assigned {
                            let a_end = a.start_hour + a.duration_hours;
                            current_hours += a.duration_hours;
                            if shift.start_hour < a_end && a.start_hour < shift_end {
                                no_overlap = false;
                            }
                            if shift.start_hour < a_end + min_rest && a.start_hour < shift_end + min_rest {
                                rest_ok = false;
                            }
                        }
                        let hc3_ok = current_hours + shift.duration_hours <= hc3_limit;
                        no_overlap && rest_ok && hc3_ok
                    }
                }
            })
            .map(|w| w.id)
            .collect();

        if clean.is_empty() {
            if first_empty_pool_shift.is_none() {
                first_empty_pool_shift = Some(shift_idx);
            }
            fallback_assignments += 1;

            // Fallback: skill-aware pick
            let qualified: Vec<u64> = workers.iter()
                .filter(|w| w.skills.contains(&shift.required_skill))
                .map(|w| w.id)
                .collect();
                
            if qualified.is_empty() {
                skill_shortages += 1;
                // pick random worker to prevent panic
                let w_id = workers[rng.gen_range(0..workers.len())].id;
                worker_assigned.entry(w_id).or_default().push(shift.clone());
            } else {
                let chosen_id = qualified[rng.gen_range(0..qualified.len())];
                
                // Track what violation this fallback caused
                if let Some(assigned) = worker_assigned.get(&chosen_id) {
                    let mut caused_overlap = false;
                    let mut caused_rest_fail = false;
                    let mut current_hours = 0;
                    
                    for a in assigned {
                        let a_end = a.start_hour + a.duration_hours;
                        current_hours += a.duration_hours;
                        if shift.start_hour < a_end && a.start_hour < shift_end {
                            caused_overlap = true;
                        }
                        if shift.start_hour < a_end + min_rest && a.start_hour < shift_end + min_rest {
                            caused_rest_fail = true;
                        }
                    }
                    if caused_overlap { fallback_hc2 += 1; }
                    if caused_rest_fail { fallback_rest += 1; }
                    if current_hours + shift.duration_hours > hc3_limit {
                        // we count how many times an assignment exceeded the limit
                        fallback_hc3_exceedances += 1;
                    }
                }
                
                worker_assigned.entry(chosen_id).or_default().push(shift.clone());
            }
        } else {
            clean_assignments += 1;
            let chosen_id = clean[rng.gen_range(0..clean.len())];
            worker_assigned.entry(chosen_id).or_default().push(shift.clone());
        }
    }

    // Final hours per worker
    let mut final_hours = vec![0; num_workers];
    let mut total_hc3_workers = 0;
    let mut max_worker_hours = 0;
    
    for (w_id, assigned) in &worker_assigned {
        let mut hours = 0;
        for a in assigned { hours += a.duration_hours; }
        final_hours[(*w_id as usize) - 1] = hours;
        if hours > hc3_limit {
            total_hc3_workers += 1;
        }
        if hours > max_worker_hours {
            max_worker_hours = hours;
        }
    }

    println!("| Measurement | Value |");
    println!("|---|---|");
    println!("| Number of assignments made through HC3-clean path | {} |", clean_assignments);
    println!("| Number of fallback assignments | {} |", fallback_assignments);
    println!("| First shift index (time) at which clean pool becomes empty | {} |", first_empty_pool_shift.unwrap_or(0));
    println!("| HC2 violations attributable to fallback | {} |", fallback_hc2);
    println!("| Rest violations attributable to fallback | {} |", fallback_rest);
    println!("| Fallback assignments exceeding HC3 limit | {} |", fallback_hc3_exceedances);
    println!("| Final number of workers with >40 hours (HC3) | {} |", total_hc3_workers);
    println!("| Max hours for a single worker | {} |", max_worker_hours);
    println!("| Skill shortages at construction time | {} |", skill_shortages);
    
    // Remaining eligible workers before fallback - this is asking for the clean pool size before it hit 0. 
    // It's effectively 0 at the fallback point. But it might be interesting to look at average clean pool size.
    
    // Print distribution of hours
    println!("| Hours Distribution | |");
    let mut hours_counts: HashMap<u64, usize> = HashMap::new();
    for h in final_hours {
        *hours_counts.entry(h).or_default() += 1;
    }
    let mut sorted_hours: Vec<u64> = hours_counts.keys().cloned().collect();
    sorted_hours.sort_unstable();
    for h in sorted_hours {
        println!("| Workers with {} hours | {} |", h, hours_counts[&h]);
    }
}
