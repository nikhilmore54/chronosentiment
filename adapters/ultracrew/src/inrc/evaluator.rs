use coralys_moga::traits::FitnessEvaluator;
use coralys_core::{EvaluationResult, Violation};
use super::optimization::{InrcOptimizer, InrcEvaluation, InrcGenome, SoftConstraintReport};
use coralys_matching::{BipartiteMatchingSolver, AssignmentSolver};
use std::collections::HashMap;

impl InrcOptimizer {
    pub fn get_bit(&self, genome: &InrcGenome, nurse: usize, day: usize, shift: usize) -> bool {
        let index = nurse * (self.context.num_days * self.context.shift_types.len()) 
                  + day * self.context.shift_types.len() 
                  + shift;
        genome.bits[index]
    }
}

impl FitnessEvaluator<InrcGenome> for InrcOptimizer {
    type Evaluation = InrcEvaluation;

    fn evaluate(&self, genome: &InrcGenome, _metrics: &coralys_moga::runtime::optimization::metric::MetricReport) -> Self::Evaluation {
        let weights = &self.context.weights;

        let mut hc_coverage = 0;
        let mut hc_skills = 0;
        let mut hc_one_shift_per_day = 0;
        let mut hc_forbidden_successions = 0;

        let mut assignment_penalty: i32 = 0;
        let mut work_streak_penalty: i32 = 0;
        let mut day_off_penalty: i32 = 0;
        let mut weekend_penalty: i32 = 0;
        let mut preferences_penalty: i32 = 0;
        let mut optimal_coverage_penalty: i32 = 0;

        let num_nurses = self.context.num_nurses;
        let num_days = self.context.num_days;
        let num_shifts = self.context.shift_types.len();

        let mut hard_violations = Vec::new();
        let mut soft_violations = Vec::new();

        // Check HC3: One Shift Per Nurse Per Day
        for n in 0..num_nurses {
            let nurse_id = &self.context.scenario.nurses[n].id;
            for d in 0..num_days {
                let mut shifts_assigned = 0;
                for s in 0..num_shifts {
                    if self.get_bit(genome, n, d, s) {
                        shifts_assigned += 1;
                    }
                }
                if shifts_assigned > 1 {
                    let cost = (shifts_assigned - 1) * weights.hard_constraint_violation as usize;
                    hc_one_shift_per_day += cost;
                    hard_violations.push(Violation {
                        constraint_id: "HC4_SingleAssignmentPerDay".to_string(),
                        severity: "Hard".to_string(),
                        value: Some(shifts_assigned as f64),
                        expected: "1".to_string(),
                        actual: shifts_assigned.to_string(),
                        description: format!("Nurse {} assigned to {} shifts on day {}", nurse_id, shifts_assigned, d),
                        penalty: cost as i32,
                    });
                }
            }
        }

        // Check HC4: Forbidden Shift Successions
        for n in 0..num_nurses {
            let nurse_id = &self.context.scenario.nurses[n].id;
            let hist = self.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id);
            
            let mut previous_shift_type = if let Some(h) = hist {
                h.last_assigned_shift_type.clone()
            } else {
                "None".to_string()
            };

            for d in 0..num_days {
                let mut current_shift_type = "None".to_string();
                for s in 0..num_shifts {
                    if self.get_bit(genome, n, d, s) {
                        current_shift_type = self.context.shift_types[s].clone();
                        break;
                    }
                }

                if current_shift_type != "None" && previous_shift_type != "None" && previous_shift_type != "" {
                    if let Some(rule) = self.context.scenario.forbidden_shift_type_successions.iter().find(|r| r.preceding == previous_shift_type) {
                        if rule.succeeding.contains(&current_shift_type) {
                            hc_forbidden_successions += weights.hard_constraint_violation as usize;
                            hard_violations.push(Violation {
                                constraint_id: "HC3_ForbiddenShiftSuccession".to_string(),
                                severity: "Hard".to_string(),
                                value: None,
                                expected: format!("No transition from {} to {}", previous_shift_type, current_shift_type),
                                actual: format!("Transitioned from {} to {}", previous_shift_type, current_shift_type),
                                description: format!("Nurse {} forbidden succession: {} followed by {} on day {}", nurse_id, previous_shift_type, current_shift_type, d),
                                penalty: weights.hard_constraint_violation as i32,
                            });
                        }
                    }
                }
                
                if current_shift_type != "None" || previous_shift_type == "None" || previous_shift_type == "" {
                    previous_shift_type = current_shift_type;
                } else if current_shift_type == "None" {
                    previous_shift_type = "None".to_string();
                }
            }
        }

        // Check HC1 & HC2: Coverage & Skills using BipartiteMatchingSolver
        let days_map = vec!["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        
        for d in 0..num_days {
            let day_name = days_map[d];
            for s in 0..num_shifts {
                let shift_name = &self.context.shift_types[s];
                
                let mut demands = Vec::new();
                for req in &self.context.week_data.requirements {
                    if req.shift_type == *shift_name {
                        let req_level = match day_name {
                            "Monday" => &req.monday,
                            "Tuesday" => &req.tuesday,
                            "Wednesday" => &req.wednesday,
                            "Thursday" => &req.thursday,
                            "Friday" => &req.friday,
                            "Saturday" => &req.saturday,
                            "Sunday" => &req.sunday,
                            _ => unreachable!(),
                        };
                        let target = std::cmp::max(req_level.minimum, req_level.optimal);
                        if target > 0 {
                            demands.push((req.skill.clone(), target));
                        }
                    }
                }

                let mut available_nurses = Vec::new();
                for n in 0..num_nurses {
                    if self.get_bit(genome, n, d, s) {
                        available_nurses.push(n);
                    }
                }

                // Check HC2: Skills
                for &n in &available_nurses {
                    let nurse = &self.context.scenario.nurses[n];
                    let has_valid_skill = demands.iter().any(|(skill, _)| nurse.skills.contains(skill));
                    if !has_valid_skill && !demands.is_empty() {
                        hc_skills += weights.hard_constraint_violation as usize;
                        hard_violations.push(Violation {
                            constraint_id: "HC2_SkillRequirements".to_string(),
                            severity: "Hard".to_string(),
                            value: None,
                            expected: "Nurse possesses required skill".to_string(),
                            actual: format!("Nurse skills: {:?}", nurse.skills),
                            description: format!("Nurse {} lacks required skill for shift {} on day {}", nurse.id, shift_name, day_name),
                            penalty: weights.hard_constraint_violation as i32,
                        });
                    }
                }

                // Run AssignmentSolver for Optimal Matching
                let workers: Vec<(usize, Vec<String>)> = available_nurses.iter()
                    .map(|&n| (n, self.context.scenario.nurses[n].skills.clone()))
                    .collect();

                let matching = BipartiteMatchingSolver.assign(&workers, &demands);

                // Count fulfilled requirements by skill
                let mut fulfilled_map = HashMap::new();
                for (_, skill) in &matching.assignments {
                    *fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
                }

                // Score coverage penalties
                for req in &self.context.week_data.requirements {
                    if req.shift_type == *shift_name {
                        let req_level = match day_name {
                            "Monday" => &req.monday,
                            "Tuesday" => &req.tuesday,
                            "Wednesday" => &req.wednesday,
                            "Thursday" => &req.thursday,
                            "Friday" => &req.friday,
                            "Saturday" => &req.saturday,
                            "Sunday" => &req.sunday,
                            _ => unreachable!(),
                        };

                        let fulfilled = *fulfilled_map.get(&req.skill).unwrap_or(&0);
                        if fulfilled < req_level.minimum {
                            let missing = req_level.minimum - fulfilled;
                            let cost = missing * weights.hard_constraint_violation as usize;
                            hc_coverage += cost;
                            hard_violations.push(Violation {
                                constraint_id: "HC1_MinimumCoverage".to_string(),
                                severity: "Hard".to_string(),
                                value: Some(missing as f64),
                                expected: req_level.minimum.to_string(),
                                actual: fulfilled.to_string(),
                                description: format!("Shift {} skill {} on {} lacks {} nurses (minimum)", shift_name, req.skill, day_name, missing),
                                penalty: cost as i32,
                            });
                        } else if fulfilled < req_level.optimal {
                            let missing = req_level.optimal - fulfilled;
                            let cost = missing as i32 * weights.optimal_coverage;
                            optimal_coverage_penalty += cost;
                            soft_violations.push(Violation {
                                constraint_id: "S8_OptimalCoverage".to_string(),
                                severity: "Soft".to_string(),
                                value: Some(missing as f64),
                                expected: req_level.optimal.to_string(),
                                actual: fulfilled.to_string(),
                                description: format!("Shift {} skill {} on {} lacks {} nurses (optimal)", shift_name, req.skill, day_name, missing),
                                penalty: cost,
                            });
                        }
                    }
                }
            }
        }

        // Shift Off Requests (Preferences)
        for req in &self.context.week_data.shift_off_requests {
            let n_idx = self.context.scenario.nurses.iter().position(|n| n.id == req.nurse);
            let d_idx = days_map.iter().position(|&d| d == req.day);
            
            if let (Some(n), Some(d)) = (n_idx, d_idx) {
                if req.shift_type == "Any" {
                    for s in 0..num_shifts {
                        if self.get_bit(genome, n, d, s) {
                            preferences_penalty += weights.preferences;
                            soft_violations.push(Violation {
                                constraint_id: "S5_Preferences".to_string(),
                                severity: "Soft".to_string(),
                                value: None,
                                expected: "Shift off requested".to_string(),
                                actual: "Shift assigned".to_string(),
                                description: format!("Nurse {} shift off request violated on day {}", req.nurse, req.day),
                                penalty: weights.preferences,
                            });
                        }
                    }
                } else {
                    if let Some(s) = self.context.shift_types.iter().position(|s| s == &req.shift_type) {
                        if self.get_bit(genome, n, d, s) {
                            preferences_penalty += weights.preferences;
                            soft_violations.push(Violation {
                                constraint_id: "S5_Preferences".to_string(),
                                severity: "Soft".to_string(),
                                value: None,
                                expected: format!("Shift off requested: {}", req.shift_type),
                                actual: format!("Shift assigned: {}", req.shift_type),
                                description: format!("Nurse {} shift off request ({}) violated on day {}", req.nurse, req.shift_type, req.day),
                                penalty: weights.preferences,
                            });
                        }
                    }
                }
            }
        }

        // Soft Constraints: S1, S2, S3, S4, S6, S7
        for n in 0..num_nurses {
            let nurse = &self.context.scenario.nurses[n];
            let contract = self.context.scenario.contracts.iter().find(|c| c.id == nurse.contract).unwrap();
            
            let hist = self.context.history.nurse_history.iter().find(|h| &h.nurse == &nurse.id);
            let initial_streak = if let Some(h) = hist { h.number_of_consecutive_working_days } else { 0 };
            let initial_off_streak = if let Some(h) = hist { h.number_of_consecutive_days_off } else { 0 };
            let initial_shift_streak = if let Some(h) = hist { h.number_of_consecutive_assignments } else { 0 };
            let mut current_last_shift = if let Some(h) = hist { Some(h.last_assigned_shift_type.clone()) } else { None };
            
            if current_last_shift.as_deref() == Some("") || current_last_shift.as_deref() == Some("None") {
                current_last_shift = None;
            }
            
            let mut current_streak = initial_streak;
            let mut current_off_streak = initial_off_streak;
            let mut current_shift_streak = initial_shift_streak;
            
            let mut week_assignments = 0;
            
            let mut works_saturday = false;
            let mut works_sunday = false;
            
            for d in 0..num_days {
                let mut works = false;
                let mut worked_shift = None;
                
                for s in 0..num_shifts {
                    if self.get_bit(genome, n, d, s) {
                        works = true;
                        worked_shift = Some(&self.context.shift_types[s]);
                        break;
                    }
                }

                if works {
                    week_assignments += 1;
                    current_streak += 1;
                    
                    if d == 5 { works_saturday = true; }
                    if d == 6 { works_sunday = true; }
                    
                    // Day off streak ended
                    if current_off_streak > 0 {
                        if current_off_streak < contract.min_consecutive_days_off {
                            let diff = contract.min_consecutive_days_off - current_off_streak;
                            let cost = diff as i32 * weights.consecutive_days_off;
                            day_off_penalty += cost;
                            soft_violations.push(Violation {
                                constraint_id: "S3_ConsecutiveDaysOff".to_string(),
                                severity: "Soft".to_string(),
                                value: Some(current_off_streak as f64),
                                expected: format!(">= {}", contract.min_consecutive_days_off),
                                actual: current_off_streak.to_string(),
                                description: format!("Nurse {} had consecutive days off streak of {} (minimum {}) ending on day {}", nurse.id, current_off_streak, contract.min_consecutive_days_off, d),
                                penalty: cost,
                            });
                        }
                        if current_off_streak > contract.max_consecutive_days_off {
                            let current_excess = current_off_streak - contract.max_consecutive_days_off;
                            let initial_excess = if initial_off_streak > contract.max_consecutive_days_off { initial_off_streak - contract.max_consecutive_days_off } else { 0 };
                            if current_excess > initial_excess {
                                let cost = (current_excess - initial_excess) as i32 * weights.consecutive_days_off;
                                day_off_penalty += cost;
                                soft_violations.push(Violation {
                                    constraint_id: "S3_ConsecutiveDaysOff".to_string(),
                                    severity: "Soft".to_string(),
                                    value: Some(current_off_streak as f64),
                                    expected: format!("<= {}", contract.max_consecutive_days_off),
                                    actual: current_off_streak.to_string(),
                                    description: format!("Nurse {} had consecutive days off streak of {} (maximum {}) ending on day {}", nurse.id, current_off_streak, contract.max_consecutive_days_off, d),
                                    penalty: cost,
                                });
                            }
                        }
                        current_off_streak = 0;
                    }
                    
                    // Shift Streak logic
                    if let Some(w_shift) = worked_shift {
                        if let Some(ref last) = current_last_shift {
                            if last == w_shift {
                                current_shift_streak += 1;
                            } else {
                                // Ended previous shift streak
                                let s_type = self.context.scenario.shift_types.iter().find(|st| &st.id == last).unwrap();
                                if current_shift_streak > 0 && current_shift_streak < s_type.min_consecutive {
                                    let diff = s_type.min_consecutive - current_shift_streak;
                                    let cost = diff as i32 * weights.consecutive_shift_days;
                                    work_streak_penalty += cost;
                                    soft_violations.push(Violation {
                                        constraint_id: "S4_ConsecutiveShiftTypes".to_string(),
                                        severity: "Soft".to_string(),
                                        value: Some(current_shift_streak as f64),
                                        expected: format!(">= {}", s_type.min_consecutive),
                                        actual: current_shift_streak.to_string(),
                                        description: format!("Nurse {} had consecutive shift streak for {} of {} (minimum {}) ending on day {}", nurse.id, last, current_shift_streak, s_type.min_consecutive, d),
                                        penalty: cost,
                                    });
                                }
                                if current_shift_streak > s_type.max_consecutive {
                                    let current_excess = current_shift_streak - s_type.max_consecutive;
                                    let initial_excess = if initial_shift_streak > s_type.max_consecutive { initial_shift_streak - s_type.max_consecutive } else { 0 };
                                    if current_excess > initial_excess {
                                        let cost = (current_excess - initial_excess) as i32 * weights.consecutive_shift_days;
                                        work_streak_penalty += cost;
                                        soft_violations.push(Violation {
                                            constraint_id: "S4_ConsecutiveShiftTypes".to_string(),
                                            severity: "Soft".to_string(),
                                            value: Some(current_shift_streak as f64),
                                            expected: format!("<= {}", s_type.max_consecutive),
                                            actual: current_shift_streak.to_string(),
                                            description: format!("Nurse {} had consecutive shift streak for {} of {} (maximum {}) ending on day {}", nurse.id, last, current_shift_streak, s_type.max_consecutive, d),
                                            penalty: cost,
                                        });
                                    }
                                }
                                current_shift_streak = 1;
                                current_last_shift = Some(w_shift.clone());
                            }
                        } else {
                            current_shift_streak = 1;
                            current_last_shift = Some(w_shift.clone());
                        }
                    }
                } else {
                    current_off_streak += 1;
                    
                    // Shift Streak ended
                    if let Some(ref last) = current_last_shift {
                        let s_type = self.context.scenario.shift_types.iter().find(|st| &st.id == last).unwrap();
                        if current_shift_streak > 0 && current_shift_streak < s_type.min_consecutive {
                            let diff = s_type.min_consecutive - current_shift_streak;
                            let cost = diff as i32 * weights.consecutive_shift_days;
                            work_streak_penalty += cost;
                            soft_violations.push(Violation {
                                constraint_id: "S4_ConsecutiveShiftTypes".to_string(),
                                severity: "Soft".to_string(),
                                value: Some(current_shift_streak as f64),
                                expected: format!(">= {}", s_type.min_consecutive),
                                actual: current_shift_streak.to_string(),
                                description: format!("Nurse {} had consecutive shift streak for {} of {} (minimum {}) ending on day {}", nurse.id, last, current_shift_streak, s_type.min_consecutive, d),
                                penalty: cost,
                            });
                        }
                        if current_shift_streak > s_type.max_consecutive {
                            let current_excess = current_shift_streak - s_type.max_consecutive;
                            let initial_excess = if initial_shift_streak > s_type.max_consecutive { initial_shift_streak - s_type.max_consecutive } else { 0 };
                            if current_excess > initial_excess {
                                let cost = (current_excess - initial_excess) as i32 * weights.consecutive_shift_days;
                                work_streak_penalty += cost;
                                soft_violations.push(Violation {
                                    constraint_id: "S4_ConsecutiveShiftTypes".to_string(),
                                    severity: "Soft".to_string(),
                                    value: Some(current_shift_streak as f64),
                                    expected: format!("<= {}", s_type.max_consecutive),
                                    actual: current_shift_streak.to_string(),
                                    description: format!("Nurse {} had consecutive shift streak for {} of {} (maximum {}) ending on day {}", nurse.id, last, current_shift_streak, s_type.max_consecutive, d),
                                    penalty: cost,
                                });
                            }
                        }
                        current_shift_streak = 0;
                        current_last_shift = None;
                    }
                    
                    // Work streak ended
                    if current_streak > 0 {
                        if current_streak < contract.min_consecutive_working_days {
                            let diff = contract.min_consecutive_working_days - current_streak;
                            let cost = diff as i32 * weights.consecutive_working_days;
                            work_streak_penalty += cost;
                            soft_violations.push(Violation {
                                constraint_id: "S2_ConsecutiveWorkingDays".to_string(),
                                severity: "Soft".to_string(),
                                value: Some(current_streak as f64),
                                expected: format!(">= {}", contract.min_consecutive_working_days),
                                actual: current_streak.to_string(),
                                description: format!("Nurse {} had consecutive working streak of {} (minimum {}) ending on day {}", nurse.id, current_streak, contract.min_consecutive_working_days, d),
                                penalty: cost,
                            });
                        }
                        if current_streak > contract.max_consecutive_working_days {
                            let current_excess = current_streak - contract.max_consecutive_working_days;
                            let initial_excess = if initial_streak > contract.max_consecutive_working_days { initial_streak - contract.max_consecutive_working_days } else { 0 };
                            if current_excess > initial_excess {
                                let cost = (current_excess - initial_excess) as i32 * weights.consecutive_working_days;
                                work_streak_penalty += cost;
                                soft_violations.push(Violation {
                                    constraint_id: "S2_ConsecutiveWorkingDays".to_string(),
                                    severity: "Soft".to_string(),
                                    value: Some(current_streak as f64),
                                    expected: format!("<= {}", contract.max_consecutive_working_days),
                                    actual: current_streak.to_string(),
                                    description: format!("Nurse {} had consecutive working streak of {} (maximum {}) ending on day {}", nurse.id, current_streak, contract.max_consecutive_working_days, d),
                                    penalty: cost,
                                });
                            }
                        }
                        current_streak = 0;
                    }
                }
            }
            
            // End of stage handling (only MAX, MIN might continue next week)
            if current_streak > contract.max_consecutive_working_days {
                let current_excess = current_streak - contract.max_consecutive_working_days;
                let initial_excess = if initial_streak > contract.max_consecutive_working_days { initial_streak - contract.max_consecutive_working_days } else { 0 };
                if current_excess > initial_excess {
                    let cost = (current_excess - initial_excess) as i32 * weights.consecutive_working_days;
                    work_streak_penalty += cost;
                    soft_violations.push(Violation {
                        constraint_id: "S2_ConsecutiveWorkingDays".to_string(),
                        severity: "Soft".to_string(),
                        value: Some(current_streak as f64),
                        expected: format!("<= {}", contract.max_consecutive_working_days),
                        actual: current_streak.to_string(),
                        description: format!("Nurse {} had consecutive working streak of {} (maximum {}) extending past week end", nurse.id, current_streak, contract.max_consecutive_working_days),
                        penalty: cost,
                    });
                }
            }
            if current_off_streak > contract.max_consecutive_days_off {
                let current_excess = current_off_streak - contract.max_consecutive_days_off;
                let initial_excess = if initial_off_streak > contract.max_consecutive_days_off { initial_off_streak - contract.max_consecutive_days_off } else { 0 };
                if current_excess > initial_excess {
                    let cost = (current_excess - initial_excess) as i32 * weights.consecutive_days_off;
                    day_off_penalty += cost;
                    soft_violations.push(Violation {
                        constraint_id: "S3_ConsecutiveDaysOff".to_string(),
                        severity: "Soft".to_string(),
                        value: Some(current_off_streak as f64),
                        expected: format!("<= {}", contract.max_consecutive_days_off),
                        actual: current_off_streak.to_string(),
                        description: format!("Nurse {} had consecutive days off streak of {} (maximum {}) extending past week end", nurse.id, current_off_streak, contract.max_consecutive_days_off),
                        penalty: cost,
                    });
                }
            }
            
            // S3: End of stage handling for shift streak (only evaluate MAX, MIN might continue next week)
            if let Some(ref last) = current_last_shift {
                let s_type = self.context.scenario.shift_types.iter().find(|st| &st.id == last).unwrap();
                if current_shift_streak > s_type.max_consecutive {
                    let current_excess = current_shift_streak - s_type.max_consecutive;
                    let initial_excess = if initial_shift_streak > s_type.max_consecutive { initial_shift_streak - s_type.max_consecutive } else { 0 };
                    if current_excess > initial_excess {
                        let cost = (current_excess - initial_excess) as i32 * weights.consecutive_shift_days;
                        work_streak_penalty += cost;
                        soft_violations.push(Violation {
                            constraint_id: "S4_ConsecutiveShiftTypes".to_string(),
                            severity: "Soft".to_string(),
                            value: Some(current_shift_streak as f64),
                            expected: format!("<= {}", s_type.max_consecutive),
                            actual: current_shift_streak.to_string(),
                            description: format!("Nurse {} had consecutive shift streak for {} of {} (maximum {}) extending past week end", nurse.id, last, current_shift_streak, s_type.max_consecutive),
                            penalty: cost,
                        });
                    }
                }
            }
            
            // Complete Weekends
            if contract.complete_weekends > 0 {
                if works_saturday != works_sunday {
                    let cost = contract.complete_weekends as i32 * weights.complete_weekends;
                    weekend_penalty += cost;
                    soft_violations.push(Violation {
                        constraint_id: "S6_CompleteWeekends".to_string(),
                        severity: "Soft".to_string(),
                        value: None,
                        expected: "Complete working weekend (both worked or both off)".to_string(),
                        actual: format!("Sat: {}, Sun: {}", works_saturday, works_sunday),
                        description: format!("Nurse {} had incomplete working weekend (Sat: {}, Sun: {})", nurse.id, works_saturday, works_sunday),
                        penalty: cost,
                    });
                }
            }

            // S7: Max Working Weekends
            let works_weekend = works_saturday || works_sunday;
            // The validator does not accumulate history weekends for S7 when validating single weeks.
            // To achieve perfect parity with the official validator, we match this behavior.
            let total_working_weekends = if works_weekend { 1 } else { 0 };
            if total_working_weekends > contract.max_working_weekends {
                let excess = total_working_weekends - contract.max_working_weekends;
                let cost = excess as i32 * weights.max_working_weekends;
                weekend_penalty += cost;
                soft_violations.push(Violation {
                    constraint_id: "S7_MaxWorkingWeekends".to_string(),
                    severity: "Soft".to_string(),
                    value: Some(total_working_weekends as f64),
                    expected: format!("<= {}", contract.max_working_weekends),
                    actual: total_working_weekends.to_string(),
                    description: format!("Nurse {} exceeded max working weekends: {} (maximum {})", nurse.id, total_working_weekends, contract.max_working_weekends),
                    penalty: cost,
                });
            }

            // S1: Assignments (Scaled for 1 week)
            let weekly_min = contract.min_assignments / self.context.scenario.number_of_weeks;
            let weekly_max = (contract.max_assignments as f64 / self.context.scenario.number_of_weeks as f64).ceil() as usize;
            
            if week_assignments < weekly_min {
                let diff = weekly_min - week_assignments;
                let cost = diff as i32 * weights.assignments;
                assignment_penalty += cost;
                soft_violations.push(Violation {
                    constraint_id: "S1_TotalAssignments".to_string(),
                    severity: "Soft".to_string(),
                    value: Some(week_assignments as f64),
                    expected: format!(">= {}", weekly_min),
                    actual: week_assignments.to_string(),
                    description: format!("Nurse {} total assignments {} below weekly minimum {}", nurse.id, week_assignments, weekly_min),
                    penalty: cost,
                });
            }
            if week_assignments > weekly_max {
                let diff = week_assignments - weekly_max;
                let cost = diff as i32 * weights.assignments;
                assignment_penalty += cost;
                soft_violations.push(Violation {
                    constraint_id: "S1_TotalAssignments".to_string(),
                    severity: "Soft".to_string(),
                    value: Some(week_assignments as f64),
                    expected: format!("<= {}", weekly_max),
                    actual: week_assignments.to_string(),
                    description: format!("Nurse {} total assignments {} above weekly maximum {}", nurse.id, week_assignments, weekly_max),
                    penalty: cost,
                });
            }
        }

        let soft_report = SoftConstraintReport {
            assignment_penalty,
            work_streak_penalty,
            day_off_penalty,
            weekend_penalty,
            preferences_penalty,
            optimal_coverage_penalty,
            total_penalty: assignment_penalty + work_streak_penalty + day_off_penalty + weekend_penalty + preferences_penalty + optimal_coverage_penalty,
        };

        let total_hc_penalty = hc_coverage + hc_skills + hc_one_shift_per_day + hc_forbidden_successions;
        
        let base_fitness = 100_000.0;
        let fitness = base_fitness - total_hc_penalty as f64 - soft_report.total_penalty as f64;

        // Build the platform EvaluationResult (vector of objectives)
        let mut metrics = HashMap::new();
        metrics.insert("hc_coverage".to_string(), hc_coverage as f64);
        metrics.insert("hc_skills".to_string(), hc_skills as f64);
        metrics.insert("hc_one_shift_per_day".to_string(), hc_one_shift_per_day as f64);
        metrics.insert("hc_forbidden_successions".to_string(), hc_forbidden_successions as f64);
        metrics.insert("assignment_penalty".to_string(), assignment_penalty as f64);
        metrics.insert("work_streak_penalty".to_string(), work_streak_penalty as f64);
        metrics.insert("day_off_penalty".to_string(), day_off_penalty as f64);
        metrics.insert("weekend_penalty".to_string(), weekend_penalty as f64);
        metrics.insert("preferences_penalty".to_string(), preferences_penalty as f64);
        metrics.insert("optimal_coverage_penalty".to_string(), optimal_coverage_penalty as f64);

        let platform_result = EvaluationResult {
            objectives: vec![soft_report.total_penalty as f64],
            hard_constraint_violations: hard_violations,
            soft_constraint_violations: soft_violations,
            metrics,
        };

        InrcEvaluation {
            genome: genome.clone(),
            fitness: fitness.max(1.0),
            hc_coverage,
            hc_skills,
            hc_one_shift_per_day,
            hc_forbidden_successions,
            soft_report,
            platform_result,
        }
    }
}
