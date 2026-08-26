use coralys_matching::AssignmentSolver;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use ultracrew::ecology::WorkforceEcology;
use ultracrew::inrc::optimization::{InrcContext, InrcGenome, InrcOptimizer};
use ultracrew::inrc::parser::{parse_history, parse_scenario, parse_week_data};

#[test]
fn test_coverage_parity() {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/n030w4");
    let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
    let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();
    let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
    let ecology = WorkforceEcology::new();
    let context = InrcContext::new(scenario, week_data, history, ecology);
    let optimizer = InrcOptimizer {
        context: Arc::new(context),
    };

    let frozen_bits_json = std::fs::read_to_string(base_dir.join("frozen_genome.json")).unwrap();
    let bits: Vec<bool> = serde_json::from_str(&frozen_bits_json).unwrap();
    let genome = InrcGenome { bits };

    // We will extract a single day/shift where there is a mismatch.
    // Or just run the matching for the entire week and see if it achieves 360 penalty!

    let num_days = optimizer.context.num_days;
    let num_shifts = optimizer.context.shift_types.len();

    let mut total_optimal_penalty = 0;

    for d in 0..num_days {
        let day_name = match d {
            0 => "Mon",
            1 => "Tue",
            2 => "Wed",
            3 => "Thu",
            4 => "Fri",
            5 => "Sat",
            6 => "Sun",
            _ => unreachable!(),
        };
        for s in 0..num_shifts {
            let shift_name = &optimizer.context.shift_types[s];

            let mut available_nurses = Vec::new();
            for n in 0..optimizer.context.num_nurses {
                if optimizer.get_bit(&genome, n, d, s) {
                    available_nurses.push(n);
                }
            }

            let mut demands = Vec::new();
            for req in &optimizer.context.week_data.requirements {
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
                        demands.push((req.skill.clone(), target));
                    }
                }
            }

            let workers: Vec<(usize, Vec<String>)> = available_nurses
                .iter()
                .map(|&n| (n, optimizer.context.scenario.nurses[n].skills.clone()))
                .collect();
            let solver_demands: Vec<(String, usize)> =
                demands.iter().map(|(s, c)| (s.clone(), *c)).collect();
            let matching =
                coralys_matching::BipartiteMatchingSolver.assign(&workers, &solver_demands);

            let mut fulfilled_map = HashMap::new();
            for (_, skill) in &matching.assignments {
                *fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
            }

            for req in &optimizer.context.week_data.requirements {
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
                    let fulfilled = *fulfilled_map.get(&req.skill).unwrap_or(&0);
                    if fulfilled >= req_level.minimum && fulfilled < req_level.optimal {
                        let p = (req_level.optimal - fulfilled) * 30;
                        total_optimal_penalty += p;
                        println!(
                            "Day {} Shift {}: Skill {} Opt {}, Matched {}, Penalty +{}",
                            d, shift_name, req.skill, req_level.optimal, fulfilled, p
                        );
                    }
                }
            }
        }
    }

    println!(
        "Maximum Bipartite Matching Penalty: {}",
        total_optimal_penalty
    );

    assert_eq!(
        total_optimal_penalty, 360,
        "Bipartite matching should yield 360 penalty!"
    );
}
