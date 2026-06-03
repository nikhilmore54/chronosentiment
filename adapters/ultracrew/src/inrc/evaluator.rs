use coralys_moga::traits::FitnessEvaluator;
use super::optimization::{InrcOptimizer, InrcEvaluation, InrcGenome, SoftConstraintReport};

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

    fn evaluate(&self, genome: &InrcGenome) -> Self::Evaluation {
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

        // Check HC3: One Shift Per Nurse Per Day
        for n in 0..num_nurses {
            for d in 0..num_days {
                let mut shifts_assigned = 0;
                for s in 0..num_shifts {
                    if self.get_bit(genome, n, d, s) {
                        shifts_assigned += 1;
                    }
                }
                if shifts_assigned > 1 {
                    hc_one_shift_per_day += (shifts_assigned - 1) * 1000;
                }
            }
        }

        // Check HC4: Forbidden Shift Successions
        // E.g., Late -> Early not allowed.
        for n in 0..num_nurses {
            // Also check history for day 0 (if history says last day was Late, and today is Early)
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

                if current_shift_type != "None" && previous_shift_type != "None" {
                    // Check if forbidden
                    if let Some(rule) = self.context.scenario.forbidden_shift_type_successions.iter().find(|r| r.preceding == previous_shift_type) {
                        if rule.succeeding.contains(&current_shift_type) {
                            hc_forbidden_successions += 1000;
                        }
                    }
                }
                
                // If multiple shifts assigned (HC3 fails), we still track the first one for succession, 
                // but since HC3 heavily penalizes, it's fine.
                if current_shift_type != "None" || previous_shift_type == "None" {
                    previous_shift_type = current_shift_type;
                } else if current_shift_type == "None" {
                    previous_shift_type = "None".to_string();
                }
            }
        }

        // Check HC1 & HC2: Coverage & Skills
        // The week_data specifies requirements per day, shiftType, and skill.
        // E.g. Day=Monday, Shift=Early, Skill=Nurse, min=1.
        
        let days_map = vec!["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        
        for req in &self.context.week_data.requirements {
            let shift_idx = self.context.shift_types.iter().position(|s| s == &req.shift_type).unwrap();
            
            for (d_idx, day_name) in days_map.iter().enumerate() {
                let req_level = match *day_name {
                    "Monday" => &req.monday,
                    "Tuesday" => &req.tuesday,
                    "Wednesday" => &req.wednesday,
                    "Thursday" => &req.thursday,
                    "Friday" => &req.friday,
                    "Saturday" => &req.saturday,
                    "Sunday" => &req.sunday,
                    _ => unreachable!(),
                };

                let mut assigned_with_skill = 0;
                let required_skill = &req.skill;

                for n in 0..num_nurses {
                    if self.get_bit(genome, n, d_idx, shift_idx) {
                        let nurse = &self.context.scenario.nurses[n];
                        if nurse.skills.contains(required_skill) {
                            assigned_with_skill += 1;
                        } else {
                            // HC2: Nurse assigned to a shift but lacks the required skill for the demand?
                            // Wait, in INRC-II, nurses are assigned to a shift, and implicitly they fill a requirement for a skill they possess.
                            // If a nurse is assigned to a shift, they MUST cover some requirement. 
                            // This means we need to evaluate if the total nurses assigned to a shift covers the total required skills.
                            // The problem says: "Nurses can only be assigned to a shift if they have the required skill".
                            // If a nurse is assigned but isn't covering any skill, it's a violation.
                            // Actually, it's simpler: count the number of assigned nurses that have the required skill.
                            // But wait, a nurse can have multiple skills. Can they cover multiple requirements? No, one assignment covers one skill.
                            // For Bronze tier, we can just say:
                            // We need to fulfill the minimum coverage exactly. 
                        }
                    }
                }
            }
        }

        // Let's implement a more precise HC1/HC2 logic.
        // For a given day and shift, we have a list of required skills and their minimums.
        // We look at all nurses assigned to this day and shift.
        // We must map each assigned nurse to exactly ONE required skill they possess.
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
                        if req_level.minimum > 0 || req_level.optimal > 0 {
                            demands.push((&req.skill, req_level.minimum, req_level.optimal));
                        }
                    }
                }

                // Collect available assigned nurses
                let mut available_nurses = Vec::new();
                for n in 0..num_nurses {
                    if self.get_bit(genome, n, d, s) {
                        available_nurses.push(n);
                    }
                }

                // Check HC2: For each assigned nurse, they MUST possess at least one skill that is demanded by this shift
                for &n in &available_nurses {
                    let nurse = &self.context.scenario.nurses[n];
                    let has_valid_skill = demands.iter().any(|(skill, _, _)| nurse.skills.contains(*skill));
                    if !has_valid_skill {
                        // Nurse doesn't have any skill required for this shift!
                        hc_skills += 1000;
                    }
                }

                // Try to fulfill minimum and optimal demands
                for (skill, min_count, opt_count) in demands {
                    let mut fulfilled = 0;
                    let mut to_remove = Vec::new();
                    
                    // We attempt to fulfill up to max(min_count, opt_count)
                    let target_count = std::cmp::max(min_count, opt_count);
                    
                    for (i, &n) in available_nurses.iter().enumerate() {
                        let nurse = &self.context.scenario.nurses[n];
                        if nurse.skills.contains(skill) {
                            fulfilled += 1;
                            to_remove.push(i);
                            if fulfilled == target_count {
                                break;
                            }
                        }
                    }

                    if fulfilled < min_count {
                        hc_coverage += (min_count - fulfilled) * 1000;
                    }
                    
                    if fulfilled >= min_count && fulfilled < opt_count {
                        optimal_coverage_penalty += ((opt_count - fulfilled) * 30) as i32;
                    }
                    
                    // Remove nurses that have been assigned to this skill so they can't be assigned to another
                    for &i in to_remove.iter().rev() {
                        available_nurses.remove(i);
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
                            preferences_penalty += 10;
                        }
                    }
                } else {
                    if let Some(s) = self.context.shift_types.iter().position(|s| s == &req.shift_type) {
                        if self.get_bit(genome, n, d, s) {
                            preferences_penalty += 10;
                        }
                    }
                }
            }
        }

        // Soft Constraints S6 & S2 & S3 & Weekends
        for n in 0..num_nurses {
            let nurse = &self.context.scenario.nurses[n];
            let contract = self.context.scenario.contracts.iter().find(|c| c.id == nurse.contract).unwrap();
            
            let hist = self.context.history.nurse_history.iter().find(|h| &h.nurse == &nurse.id);
            let initial_streak = if let Some(h) = hist { h.number_of_consecutive_working_days } else { 0 };
            let initial_off_streak = if let Some(h) = hist { h.number_of_consecutive_days_off } else { 0 };
            let initial_shift_streak = if let Some(h) = hist { h.number_of_consecutive_assignments } else { 0 };
            let mut current_last_shift = if let Some(h) = hist { Some(h.last_assigned_shift_type.clone()) } else { None };
            
            // "None" in history is represented as "" or "None" sometimes. Let's fix that.
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
                        break; // Can only work 1 shift per day in valid schedules, but for soft constraints just take the first one
                    }
                }
                if let Some(w_shift) = worked_shift {
                    println!("Nurse {} Day {}: works {}", nurse.id, d, w_shift);
                } else {
                    println!("Nurse {} Day {}: off", nurse.id, d);
                }
                if works {
                    week_assignments += 1;
                    current_streak += 1;
                    
                    if d == 5 { works_saturday = true; }
                    if d == 6 { works_sunday = true; }
                    
                    // Day off streak ended
                    if current_off_streak > 0 {
                        if current_off_streak < contract.min_consecutive_days_off {
                            day_off_penalty += ((contract.min_consecutive_days_off - current_off_streak) * 30) as i32;
                        }
                        if current_off_streak > contract.max_consecutive_days_off {
                            let current_excess = current_off_streak - contract.max_consecutive_days_off;
                            let initial_excess = if initial_off_streak > contract.max_consecutive_days_off { initial_off_streak - contract.max_consecutive_days_off } else { 0 };
                            if current_excess > initial_excess {
                                day_off_penalty += ((current_excess - initial_excess) * 30) as i32;
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
                                    let p = ((s_type.min_consecutive - current_shift_streak) * 15) as i32;
                                    println!("Nurse {}, S3 Min {} Ended Day {}: +{}", nurse.id, last, d, p);
                                    work_streak_penalty += p;
                                }
                                if current_shift_streak > s_type.max_consecutive {
                                    let current_excess = current_shift_streak - s_type.max_consecutive;
                                    let initial_excess = if initial_shift_streak > s_type.max_consecutive { initial_shift_streak - s_type.max_consecutive } else { 0 };
                                    if current_excess > initial_excess {
                                        let p = ((current_excess - initial_excess) * 15) as i32;
                                        println!("Nurse {}, S3 Max {} Ended Day {}: +{}", nurse.id, last, d, p);
                                        work_streak_penalty += p;
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
                            let p = ((s_type.min_consecutive - current_shift_streak) * 15) as i32;
                            println!("Nurse {}, S3 Min {} Ended Day {} (Off): +{}", nurse.id, last, d, p);
                            work_streak_penalty += p;
                        }
                        if current_shift_streak > s_type.max_consecutive {
                            let current_excess = current_shift_streak - s_type.max_consecutive;
                            let initial_excess = if initial_shift_streak > s_type.max_consecutive { initial_shift_streak - s_type.max_consecutive } else { 0 };
                            if current_excess > initial_excess {
                                let p = ((current_excess - initial_excess) * 15) as i32;
                                println!("Nurse {}, S3 Max {} Ended Day {} (Off): +{}", nurse.id, last, d, p);
                                work_streak_penalty += p;
                            }
                        }
                        current_shift_streak = 0;
                        current_last_shift = None;
                    }
                    
                    // Work streak ended
                    if current_streak > 0 {
                        if current_streak < contract.min_consecutive_working_days {
                            let p = ((contract.min_consecutive_working_days - current_streak) * 30) as i32;
                            println!("Nurse {}, S2 Min Work Ended Day {}: +{}", nurse.id, d, p);
                            work_streak_penalty += p;
                        }
                        if current_streak > contract.max_consecutive_working_days {
                            let current_excess = current_streak - contract.max_consecutive_working_days;
                            let initial_excess = if initial_streak > contract.max_consecutive_working_days { initial_streak - contract.max_consecutive_working_days } else { 0 };
                            if current_excess > initial_excess {
                                let p = ((current_excess - initial_excess) * 30) as i32;
                                println!("Nurse {}, S2 Max Work Ended Day {}: +{}", nurse.id, d, p);
                                work_streak_penalty += p;
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
                    let p = ((current_excess - initial_excess) * 30) as i32;
                    println!("Nurse {}, S2 Max Work Horizon: +{}", nurse.id, p);
                    work_streak_penalty += p;
                }
            }
            if current_off_streak > contract.max_consecutive_days_off {
                let current_excess = current_off_streak - contract.max_consecutive_days_off;
                let initial_excess = if initial_off_streak > contract.max_consecutive_days_off { initial_off_streak - contract.max_consecutive_days_off } else { 0 };
                if current_excess > initial_excess {
                    day_off_penalty += ((current_excess - initial_excess) * 30) as i32;
                }
            }
            
            // S3: End of stage handling for shift streak (only evaluate MAX, MIN might continue next week)
            if let Some(ref last) = current_last_shift {
                let s_type = self.context.scenario.shift_types.iter().find(|st| &st.id == last).unwrap();
                if current_shift_streak > s_type.max_consecutive {
                    let current_excess = current_shift_streak - s_type.max_consecutive;
                    let initial_excess = if initial_shift_streak > s_type.max_consecutive { initial_shift_streak - s_type.max_consecutive } else { 0 };
                    if current_excess > initial_excess {
                        let p = ((current_excess - initial_excess) * 15) as i32;
                        println!("Nurse {}, S3 Max {} Horizon: +{}", nurse.id, last, p);
                        work_streak_penalty += p;
                    }
                }
            }
            
            // Complete Weekends
            if contract.complete_weekends > 0 {
                if works_saturday != works_sunday {
                    weekend_penalty += (contract.complete_weekends * 30) as i32;
                }
            }

            // S6: Assignments (Scaled for 1 week)
            let weekly_min = contract.min_assignments / self.context.scenario.number_of_weeks;
            let weekly_max = (contract.max_assignments as f64 / self.context.scenario.number_of_weeks as f64).ceil() as usize;
            
            if week_assignments < weekly_min {
                assignment_penalty += ((weekly_min - week_assignments) * 20) as i32;
            }
            if week_assignments > weekly_max {
                assignment_penalty += ((week_assignments - weekly_max) * 20) as i32;
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

        InrcEvaluation {
            genome: genome.clone(),
            fitness: fitness.max(1.0),
            hc_coverage,
            hc_skills,
            hc_one_shift_per_day,
            hc_forbidden_successions,
            soft_report,
        }
    }
}
