use std::collections::HashMap;
use ultracrew::models::{Worker, Shift, Skill};
use rand::{rngs::StdRng, SeedableRng, Rng};

fn is_legal(worker_id: u64, shift: &Shift, assigned: Option<&Vec<Shift>>, min_rest: u64, hc3_limit: u64) -> bool {
    match assigned {
        None => true,
        Some(a_list) => {
            let shift_end = shift.start_hour + shift.duration_hours;
            let mut no_overlap = true;
            let mut rest_ok = true;
            let mut current_hours = 0;
            
            for a in a_list {
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
}

fn main() {
    let num_workers = 200;
    let num_shifts = 1000;
    let seed = 42;
    let min_rest = 8;
    let hc3_limit = 40;
    let lookahead_n = 10;
    let scarcity_threshold = 15; // If a future shift has <15 candidates, it's considered scarce

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

    shifts.sort_by_key(|s| s.start_hour);

    let mut rng = StdRng::seed_from_u64(seed);
    let mut worker_assigned: HashMap<u64, Vec<Shift>> = HashMap::new();
    
    let mut total_scarce_shifts_encountered = 0;
    let mut accidental_scarce_consumptions = 0;
    
    let mut pool_size_at_100 = 0;
    let mut pool_size_at_500 = 0;
    let mut pool_size_at_800 = 0;
    let mut pool_size_at_900 = 0;
    let mut pool_size_at_940 = 0;

    for shift_idx in 0..shifts.len() {
        let shift = &shifts[shift_idx];
        
        let mut clean: Vec<u64> = workers.iter()
            .filter(|w| w.skills.contains(&shift.required_skill))
            .filter(|w| is_legal(w.id, shift, worker_assigned.get(&w.id), min_rest, hc3_limit))
            .map(|w| w.id)
            .collect();
            
        if shift_idx == 100 { pool_size_at_100 = clean.len(); }
        if shift_idx == 500 { pool_size_at_500 = clean.len(); }
        if shift_idx == 800 { pool_size_at_800 = clean.len(); }
        if shift_idx == 900 { pool_size_at_900 = clean.len(); }
        if shift_idx == 940 { pool_size_at_940 = clean.len(); }

        // Measure future scarcity
        let mut future_scarcities = Vec::new();
        let end_idx = std::cmp::min(shift_idx + 1 + lookahead_n, shifts.len());
        
        for future_idx in (shift_idx + 1)..end_idx {
            let future_shift = &shifts[future_idx];
            let mut future_legal_count = 0;
            let mut future_legal_workers = Vec::new();
            
            for w in &workers {
                if w.skills.contains(&future_shift.required_skill) && is_legal(w.id, future_shift, worker_assigned.get(&w.id), min_rest, hc3_limit) {
                    future_legal_count += 1;
                    future_legal_workers.push(w.id);
                }
            }
            future_scarcities.push((future_idx, future_legal_count, future_legal_workers));
        }

        if clean.is_empty() {
            // Fallback
            let qualified: Vec<u64> = workers.iter()
                .filter(|w| w.skills.contains(&shift.required_skill))
                .map(|w| w.id)
                .collect();
            let chosen_id = if qualified.is_empty() {
                workers[rng.gen_range(0..workers.len())].id
            } else {
                qualified[rng.gen_range(0..qualified.len())]
            };
            worker_assigned.entry(chosen_id).or_default().push(shift.clone());
        } else {
            let chosen_id = clean[rng.gen_range(0..clean.len())];
            worker_assigned.entry(chosen_id).or_default().push(shift.clone());
            
            // Did we consume a scarce worker?
            let mut consumed_scarce = false;
            for (f_idx, count, f_workers) in &future_scarcities {
                if *count <= scarcity_threshold {
                    total_scarce_shifts_encountered += 1;
                    if f_workers.contains(&chosen_id) {
                        consumed_scarce = true;
                    }
                }
            }
            if consumed_scarce {
                accidental_scarce_consumptions += 1;
            }
        }
    }

    println!("| Metric | Value |");
    println!("|---|---|");
    println!("| Clean pool size @ Shift 100 | {} |", pool_size_at_100);
    println!("| Clean pool size @ Shift 500 | {} |", pool_size_at_500);
    println!("| Clean pool size @ Shift 800 | {} |", pool_size_at_800);
    println!("| Clean pool size @ Shift 900 | {} |", pool_size_at_900);
    println!("| Clean pool size @ Shift 940 | {} |", pool_size_at_940);
    println!("| Instances where an upcoming shift (in next 10) had <= {} candidates | {} |", scarcity_threshold, total_scarce_shifts_encountered);
    println!("| Times a random choice consumed a scarce worker | {} |", accidental_scarce_consumptions);
}
