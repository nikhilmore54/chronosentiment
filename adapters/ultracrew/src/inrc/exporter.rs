use super::optimization::{InrcContext, InrcGenome, InrcOptimizer};
use crate::inrc::models::InrcHistory;
use coralys_matching::AssignmentSolver;
use std::fs::File;
use std::io::{Result, Write};
use std::path::Path;

pub fn export_inrc_solution(
    genome: &InrcGenome,
    context: std::sync::Arc<InrcContext>,
    week_idx: usize,
    path: &Path,
) -> Result<()> {
    let mut file = File::create(path)?;

    writeln!(file, "SOLUTION")?;
    writeln!(file, "{} {}", week_idx, context.scenario.id)?;

    // Count assignments
    let mut assignments = Vec::new();
    let num_nurses = context.num_nurses;
    let num_days = context.num_days;
    let num_shifts = context.shift_types.len();

    let days_map = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let optimizer = InrcOptimizer {
        context: context.clone(),
    };

    for d in 0..num_days {
        for s in 0..num_shifts {
            let shift_name = &context.shift_types[s];
            let day_name = days_map[d];

            // Get all demands for this shift (only minimum for hard constraints)
            let mut demands = Vec::new();
            for req in &context.week_data.requirements {
                if req.shift_type == *shift_name {
                    let req_level = match day_name {
                        "Mon" => &req.monday,
                        "Tue" => &req.tuesday,
                        "Wed" => &req.wednesday,
                        "Thu" => &req.thursday,
                        "Fri" => &req.friday,
                        "Sat" => &req.saturday,
                        "Sun" => &req.sunday,
                        _ => unreachable!(),
                    };
                    let target = std::cmp::max(req_level.minimum, req_level.optimal);
                    if target > 0 {
                        demands.push((&req.skill, target));
                    }
                }
            }

            // Get nurses working this shift
            let mut available_nurses = Vec::new();
            for n in 0..num_nurses {
                if optimizer.get_bit(genome, n, d, s) {
                    available_nurses.push(n);
                }
            }

            let workers: Vec<(usize, Vec<String>)> = available_nurses
                .iter()
                .map(|&n| (n, context.scenario.nurses[n].skills.clone()))
                .collect();

            let solver_demands: Vec<(String, usize)> =
                demands.iter().map(|(s, c)| ((*s).clone(), *c)).collect();

            let matching =
                coralys_matching::BipartiteMatchingSolver.assign(&workers, &solver_demands);

            let mut nurse_to_skill = std::collections::HashMap::new();
            for (n, skill) in &matching.assignments {
                nurse_to_skill.insert(*n, skill.clone());
            }

            // For leftovers (unmatched nurses), assign them to their first skill
            for &n in &available_nurses {
                if !nurse_to_skill.contains_key(&n) {
                    nurse_to_skill.insert(n, context.scenario.nurses[n].skills[0].clone());
                }
            }

            // Output assignments
            for n in 0..num_nurses {
                if optimizer.get_bit(genome, n, d, s) {
                    let nurse_name = &context.scenario.nurses[n].id;
                    let assigned_skill = &nurse_to_skill[&n];
                    assignments.push(format!(
                        "{} {} {} {}",
                        nurse_name, day_name, shift_name, assigned_skill
                    ));
                }
            }
        }
    }

    writeln!(file, "ASSIGNMENTS = {}", assignments.len())?;
    for assign in assignments {
        writeln!(file, "{}", assign)?;
    }

    Ok(())
}

pub fn export_inrc_history(history: &InrcHistory, path: &Path) -> Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "HISTORY")?;
    writeln!(file, "{} {}", history.week, history.scenario)?;
    writeln!(file, "\nNURSE_HISTORY")?;

    for nh in &history.nurse_history {
        writeln!(
            file,
            "{} {} {} {} {} {} {}",
            nh.nurse,
            nh.number_of_assignments,
            nh.number_of_working_weekends,
            nh.last_assigned_shift_type,
            nh.number_of_consecutive_assignments,
            nh.number_of_consecutive_working_days,
            nh.number_of_consecutive_days_off
        )?;
    }
    Ok(())
}
