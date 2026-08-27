use std::collections::{HashMap, HashSet};
use rand::{rngs::StdRng, SeedableRng, Rng};
use std::time::Instant;

#[derive(Clone, Debug)]
struct Shift {
    id: u64,
    start_hour: u64,
    duration_hours: u64,
}

#[derive(Clone, Debug)]
struct Worker {
    id: u64,
}

fn main() {
    let num_workers = 200;
    let num_shifts = 1000;
    let min_rest = 8;
    let hc3_limit = 40;

    let mut workers = vec![];
    for i in 0..num_workers {
        workers.push(Worker { id: (i + 1) as u64 });
    }

    let mut shifts = vec![];
    for i in 0..num_shifts {
        shifts.push(Shift {
            id: (i + 1) as u64,
            start_hour: (i as u64 * 8) % 168,
            duration_hours: 8,
        });
    }

    shifts.sort_by_key(|s| s.start_hour);

    // 1. Generate P2-C1b (Greedy Random) initialization
    let mut greedy_assigned: HashMap<u64, Vec<usize>> = HashMap::new(); // worker -> shift indices
    let mut greedy_shift_to_worker = vec![0; num_shifts];
    let mut rng = StdRng::seed_from_u64(42);
    let mut greedy_first_fallback = None;

    for (shift_idx, shift) in shifts.iter().enumerate() {
        let mut clean = Vec::new();
        for w in &workers {
            let mut valid = true;
            let mut current_hours = 0;
            if let Some(assigned) = greedy_assigned.get(&w.id) {
                for &a_idx in assigned {
                    let a = &shifts[a_idx];
                    current_hours += a.duration_hours;
                    // overlap
                    if shift.start_hour < a.start_hour + a.duration_hours && a.start_hour < shift.start_hour + shift.duration_hours {
                        valid = false;
                    }
                    // rest
                    if shift.start_hour < a.start_hour + a.duration_hours + min_rest && a.start_hour < shift.start_hour + shift.duration_hours + min_rest {
                        valid = false;
                    }
                }
            }
            if current_hours + shift.duration_hours > hc3_limit {
                valid = false;
            }
            if valid {
                clean.push(w.id);
            }
        }

        let chosen_id = if clean.is_empty() {
            if greedy_first_fallback.is_none() {
                greedy_first_fallback = Some(shift_idx);
            }
            workers[rng.gen_range(0..workers.len())].id
        } else {
            clean[rng.gen_range(0..clean.len())]
        };

        greedy_assigned.entry(chosen_id).or_default().push(shift_idx);
        greedy_shift_to_worker[shift_idx] = chosen_id;
    }

    // 2. Offline CSP solver using MRV (Minimum Remaining Values)
    println!("Starting CSP solver...");
    let start = Instant::now();
    let mut global_shift_to_worker = vec![0; num_shifts];
    let mut worker_hours = vec![0; num_workers + 1];
    
    // shifts are aligned to 8-hour blocks starting at 0, 8, 16... up to 160.
    // block_idx = start_hour / 8
    
    // We can group shifts by block
    let mut block_needs = vec![0; 21];
    for s in &shifts {
        block_needs[(s.start_hour / 8) as usize] += 1;
    }

    // Since all shifts in a block are identical in constraints, we just need to assign `block_needs[b]` unique workers to block `b`.
    // Workers cannot take adjacent blocks (because min_rest = 8, and shift duration = 8. If they take block 0 (0-8), next can be block 2 (16-24)).
    // Actually, block 20 (160-168) is adjacent to block 0 (0-8) across the weekly boundary?
    // The scenario says planning_horizon_hours = 168.0, but doesn't explicitly wrap around in `is_legal`. 
    // We will use the exact logic from the engine: no wrap-around rest check in the engine currently.
    // Wait, the engine just checks absolute hours. 160-168 and 0-8 don't overlap. 
    
    let mut csp_assignments = vec![Vec::new(); 21]; // block -> workers

    fn solve(block: usize, block_needs: &[usize], worker_hours: &mut [u64], csp_assignments: &mut [Vec<u64>], num_workers: usize) -> bool {
        if block == 21 {
            return true;
        }

        if csp_assignments[block].len() == block_needs[block] {
            return solve(block + 1, block_needs, worker_hours, csp_assignments, num_workers);
        }

        // Try to assign a worker to this block
        // Heuristic: pick the worker with the fewest hours so far to balance the load
        let mut candidates: Vec<u64> = (1..=num_workers as u64).collect();
        candidates.sort_by_key(|&wid| worker_hours[wid as usize]);

        for wid in candidates {
            // Check if valid
            if worker_hours[wid as usize] >= 40 { continue; }
            if csp_assignments[block].contains(&wid) { continue; }
            
            // Check rest (no adjacent blocks)
            let mut rest_ok = true;
            if block > 0 && csp_assignments[block - 1].contains(&wid) { rest_ok = false; }
            if block < 20 && csp_assignments[block + 1].contains(&wid) { rest_ok = false; }
            
            if rest_ok {
                csp_assignments[block].push(wid);
                worker_hours[wid as usize] += 8;
                
                if solve(block, block_needs, worker_hours, csp_assignments, num_workers) {
                    return true;
                }
                
                worker_hours[wid as usize] -= 8;
                csp_assignments[block].pop();
            }
        }
        false
    }

    let feasible = solve(0, &block_needs, &mut worker_hours, &mut csp_assignments, num_workers);
    let duration = start.elapsed();

    println!("| Metric | Value |");
    println!("|---|---|");
    println!("| Is the instance globally feasible? | {} |", feasible);
    println!("| CSP solve time | {:?} |", duration);

    if feasible {
        // Map CSP block assignments back to individual shifts
        let mut block_workers = csp_assignments.clone();
        for (i, s) in shifts.iter().enumerate() {
            let block = (s.start_hour / 8) as usize;
            global_shift_to_worker[i] = block_workers[block].pop().unwrap();
        }

        // Compare Greedy to Global
        let mut exact_matches = 0;
        let mut worker_distribution = HashMap::new();
        for &w in &global_shift_to_worker {
            *worker_distribution.entry(w).or_insert(0) += 1;
        }
        
        let mut workers_40h = 0;
        let mut workers_less = 0;
        for &count in worker_distribution.values() {
            if count == 5 { workers_40h += 1; }
            else { workers_less += 1; }
        }

        let mut first_divergence = None;
        for i in 0..num_shifts {
            if greedy_shift_to_worker[i] == global_shift_to_worker[i] {
                exact_matches += 1;
            } else if first_divergence.is_none() {
                first_divergence = Some(i);
            }
        }

        println!("| Global: Workers at exactly 40h | {} |", workers_40h);
        println!("| Global: Workers < 40h | {} |", workers_less);
        println!("| Greedy: First fallback shift | {} |", greedy_first_fallback.unwrap_or(9999));
        println!("| Exact matches between Greedy and Global | {}/{} |", exact_matches, num_shifts);
        println!("| First divergent assignment index | {} |", first_divergence.unwrap_or(9999));
    }
}
