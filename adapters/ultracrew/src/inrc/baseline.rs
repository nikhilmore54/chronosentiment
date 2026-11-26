/// Baseline schedule constructor for the INRC2 domain.
///
/// Produces a `ScheduleGenome` seeded with a greedy, load-balanced assignment
/// that satisfies coverage requirements as closely as possible.  This genome
/// is used to seed the Pareto evolution engine at startup.
///
/// Moved from `services/ultracrew_server/src/simulation.rs` into the adapter
/// so that the application layer does not need to import INRC domain types.
use crate::inrc::models::{InrcRequirement, InrcScenario};
use crate::inrc::schedule_optimizer::{AssignmentSlot, ScheduleGenome};

pub fn generate_baseline_schedule(
    scenario: &InrcScenario,
    requirements: &[InrcRequirement],
) -> Result<ScheduleGenome, String> {
    let mut slots = Vec::new();
    let mut slot_id_counter: usize = 0;

    let mut nurse_load: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    let mut nurses_list = Vec::new();
    for nurse in &scenario.nurses {
        nurse_load.insert(nurse.id.clone(), 0);
        nurses_list.push(nurse.id.clone());
    }

    let mapped_shift_types = [
        ("Early", "Early"),
        ("Day", "Day"),
        ("Late", "Late"),
        ("Night", "Night"),
    ];

    let num_days = (scenario.number_of_weeks * 7) as usize;
    for d in 0..num_days {
        let weekday = d % 7;

        let mut daily_slots: Vec<(&str, String)> = Vec::new();
        for req in requirements {
            let required = match weekday {
                0 => req.monday.optimal,
                1 => req.tuesday.optimal,
                2 => req.wednesday.optimal,
                3 => req.thursday.optimal,
                4 => req.friday.optimal,
                5 => req.saturday.optimal,
                6 => req.sunday.optimal,
                _ => 0,
            };
            let mapped_shift = mapped_shift_types
                .iter()
                .find(|(k, _)| *k == req.shift_type)
                .map(|(_, v)| *v)
                .unwrap_or("");
            for _ in 0..required {
                daily_slots.push((mapped_shift, req.skill.clone()));
            }
        }

        let mut available_nurses: Vec<String> =
            scenario.nurses.iter().map(|n| n.id.clone()).collect();
        let mut rng = rand::thread_rng();

        for (shift, req_skill) in daily_slots {
            let mut best_nurse: Option<String> = None;
            let mut min_load = i32::MAX;

            let mut candidates = available_nurses.clone();
            use rand::seq::SliceRandom;
            candidates.shuffle(&mut rng);

            for candidate in candidates {
                let nurse_obj = scenario.nurses.iter().find(|n| n.id == candidate).unwrap();
                if nurse_obj.skills.contains(&req_skill) {
                    let load = *nurse_load.get(&candidate).unwrap();
                    if load < min_load {
                        min_load = load;
                        best_nurse = Some(candidate);
                    }
                }
            }

            if let Some(nurse) = best_nurse {
                available_nurses.retain(|n| n != &nurse);
                *nurse_load.get_mut(&nurse).unwrap() += 1;

                slots.push(AssignmentSlot {
                    slot_id: slot_id_counter,
                    day: d,
                    shift_type: shift.to_string(),
                    required_skill: req_skill,
                    assigned_nurse: nurse,
                });
                slot_id_counter += 1;
            }
            // If no nurse is available for this slot, skip it (volume deficit).
        }
    }

    Ok(ScheduleGenome {
        slots,
        num_days,
        nurses: nurses_list,
    })
}
