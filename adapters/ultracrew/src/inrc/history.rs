use crate::inrc::models::{InrcHistory, InrcNurseHistory};
use crate::inrc::optimization::{InrcContext, InrcGenome};
use std::sync::Arc;

pub fn extract_next_history(context: &Arc<InrcContext>, genome: &InrcGenome) -> InrcHistory {
    let mut next_nurse_histories = Vec::new();
    
    let num_nurses = context.num_nurses;
    let num_days = context.num_days;
    let num_shifts = context.shift_types.len();

    for n in 0..num_nurses {
        let prev_hist = &context.history.nurse_history[n];
        
        let mut number_of_assignments = prev_hist.number_of_assignments;
        let mut number_of_working_weekends = prev_hist.number_of_working_weekends;
        let mut last_assigned_shift_type = prev_hist.last_assigned_shift_type.clone();
        let mut number_of_consecutive_assignments = prev_hist.number_of_consecutive_assignments;
        let mut number_of_consecutive_working_days = prev_hist.number_of_consecutive_working_days;
        let mut number_of_consecutive_days_off = prev_hist.number_of_consecutive_days_off;
        
        let mut worked_weekend = false;
        
        for d in 0..num_days {
            let mut assigned_shift = None;
            for s in 0..num_shifts {
                let idx = n * (num_days * num_shifts) + d * num_shifts + s;
                if genome.bits[idx] {
                    assigned_shift = Some(s);
                    break;
                }
            }
            
            if let Some(s) = assigned_shift {
                number_of_assignments += 1;
                
                if d == 5 || d == 6 {
                    worked_weekend = true;
                }
                
                number_of_consecutive_working_days += 1;
                number_of_consecutive_days_off = 0;
                
                let shift_name = context.shift_types[s].clone();
                if shift_name == last_assigned_shift_type {
                    number_of_consecutive_assignments += 1;
                } else {
                    last_assigned_shift_type = shift_name;
                    number_of_consecutive_assignments = 1;
                }
            } else {
                number_of_consecutive_days_off += 1;
                number_of_consecutive_working_days = 0;
                
                // When off, consecutive assignments of shift type is broken
                number_of_consecutive_assignments = 0;
                // INRC-II rule: last_assigned_shift_type doesn't change just because of day off, 
                // but its consecutive counter goes to 0 (or is it preserved?)
                // Actually, the Validator expects last_assigned_shift_type to remain the same 
                // so we don't clear it. 
            }
        }
        
        if worked_weekend {
            number_of_working_weekends += 1;
        }

        next_nurse_histories.push(InrcNurseHistory {
            nurse: prev_hist.nurse.clone(),
            number_of_assignments,
            number_of_working_weekends,
            last_assigned_shift_type,
            number_of_consecutive_assignments,
            number_of_consecutive_working_days,
            number_of_consecutive_days_off,
        });
    }

    InrcHistory {
        week: context.history.week + 1,
        scenario: context.history.scenario.clone(),
        nurse_history: next_nurse_histories,
    }
}
