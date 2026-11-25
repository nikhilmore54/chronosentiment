use super::optimization::{InrcEvaluation, InrcGenome, InrcOptimizer, SoftConstraintReport};
use coralys_core::{EvaluationResult, Violation};
use coralys_matching::{AssignmentSolver, BipartiteMatchingSolver};
use coralys_moga::traits::FitnessEvaluator;
use rand::rngs::StdRng;
use rand::Rng;
use std::collections::HashMap;

impl InrcOptimizer {
    pub fn get_bit(&self, genome: &InrcGenome, nurse: usize, day: usize, shift: usize) -> bool {
        let index = nurse * (self.context.num_days * self.context.shift_types.len())
            + day * self.context.shift_types.len()
            + shift;
        genome.bits[index]
    }

    pub fn set_bit(&self, genome: &mut InrcGenome, nurse: usize, day: usize, shift: usize, val: bool) {
        let index = nurse * (self.context.num_days * self.context.shift_types.len())
            + day * self.context.shift_types.len()
            + shift;
        genome.bits[index] = val;
    }

    pub fn get_assigned_shift(&self, genome: &InrcGenome, nurse: usize, day: usize) -> Option<usize> {
        let num_shifts = self.context.shift_types.len();
        for s in 0..num_shifts {
            if self.get_bit(genome, nurse, day, s) {
                return Some(s);
            }
        }
        None
    }

    pub fn set_assigned_shift(&self, genome: &mut InrcGenome, nurse: usize, day: usize, shift: Option<usize>) {
        let num_shifts = self.context.shift_types.len();
        for s in 0..num_shifts {
            self.set_bit(genome, nurse, day, s, shift == Some(s));
        }
    }

    pub fn is_forbidden_succession(&self, nurse: usize, day: usize, shift: usize, genome: &InrcGenome) -> bool {
        let nurse_id = &self.context.scenario.nurses[nurse].id;
        let shift_name = &self.context.shift_types[shift];

        // 1. Transition from day - 1 to day
        let prev_shift_name: Option<String> = if day == 0 {
            let hist = self.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id);
            if let Some(h) = hist {
                if h.number_of_consecutive_days_off > 0
                    || h.last_assigned_shift_type.is_empty()
                    || h.last_assigned_shift_type == "None"
                {
                    None
                } else {
                    Some(h.last_assigned_shift_type.clone())
                }
            } else {
                None
            }
        } else {
            self.get_assigned_shift(genome, nurse, day - 1)
                .map(|s| self.context.shift_types[s].clone())
        };

        if let Some(ref prev) = prev_shift_name {
            if let Some(rule) = self
                .context
                .scenario
                .forbidden_shift_type_successions
                .iter()
                .find(|r| &r.preceding == prev)
            {
                if rule.succeeding.contains(shift_name) {
                    return true;
                }
            }
        }

        // 2. Transition from day to day + 1
        if day + 1 < self.context.num_days {
            if let Some(next_s) = self.get_assigned_shift(genome, nurse, day + 1) {
                let next_shift_name = &self.context.shift_types[next_s];
                if let Some(rule) = self
                    .context
                    .scenario
                    .forbidden_shift_type_successions
                    .iter()
                    .find(|r| &r.preceding == shift_name)
                {
                    if rule.succeeding.contains(next_shift_name) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Targeted hard constraint repair pass (HC1-HC4).
    pub fn repair_hard_constraints(&self, genome: &mut InrcGenome) {
        let num_nurses = self.context.num_nurses;
        let num_days = self.context.num_days;
        let num_shifts = self.context.shift_types.len();

        // 1. Clean multiple shifts per nurse per day (HC3 / HC4)
        for n in 0..num_nurses {
            for d in 0..num_days {
                let mut true_shifts = Vec::new();
                for s in 0..num_shifts {
                    if self.get_bit(genome, n, d, s) {
                        true_shifts.push(s);
                    }
                }
                if true_shifts.len() > 1 {
                    for &s in &true_shifts[1..] {
                        self.set_bit(genome, n, d, s, false);
                    }
                }
            }
        }

        // 2. Clean forbidden shift type successions (HC4)
        for n in 0..num_nurses {
            for d in 0..num_days {
                if let Some(s) = self.get_assigned_shift(genome, n, d) {
                    if self.is_forbidden_succession(n, d, s, genome) {
                        let mut fixed = false;
                        for alt_s in 0..num_shifts {
                            if alt_s != s && !self.is_forbidden_succession(n, d, alt_s, genome) {
                                self.set_assigned_shift(genome, n, d, Some(alt_s));
                                fixed = true;
                                break;
                            }
                        }
                        if !fixed {
                            self.set_assigned_shift(genome, n, d, None);
                        }
                    }
                }
            }
        }

        // 3. Ejection chain repair for minimum coverage & skills (HC1 & HC2)
        let days_map = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];

        for _pass in 0..20 {
            let mut any_deficit = false;
            for d in 0..num_days {
                let day_name = days_map[d];
                for s in 0..num_shifts {
                    let shift_name = &self.context.shift_types[s];

                    let mut min_demands = Vec::new();
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
                            if req_level.minimum > 0 {
                                min_demands.push((req.skill.clone(), req_level.minimum));
                            }
                        }
                    }

                    if min_demands.is_empty() {
                        continue;
                    }

                    let mut available_nurses = Vec::new();
                    for n in 0..num_nurses {
                        if self.get_assigned_shift(genome, n, d) == Some(s) {
                            available_nurses.push(n);
                        }
                    }
                    let workers: Vec<(usize, Vec<String>)> = available_nurses
                        .iter()
                        .map(|&n| (n, self.context.scenario.nurses[n].skills.clone()))
                        .collect();

                    let matching = BipartiteMatchingSolver.assign(&workers, &min_demands);
                    let mut fulfilled_map = HashMap::new();
                    for (_, skill) in &matching.assignments {
                        *fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
                    }

                    for req in &self.context.week_data.requirements {
                        if req.shift_type != *shift_name {
                            continue;
                        }
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

                        if req_level.minimum == 0 {
                            continue;
                        }

                        let fulfilled = *fulfilled_map.get(&req.skill).unwrap_or(&0);

                        if fulfilled < req_level.minimum {
                            any_deficit = true;
                            let needed = req_level.minimum - fulfilled;
                            let mut assigned_count = 0;

                            // Step 1: Assign off-duty qualified nurses with NO succession conflict
                            for n in 0..num_nurses {
                                if assigned_count >= needed {
                                    break;
                                }
                                if self.get_assigned_shift(genome, n, d).is_none() {
                                    if self.context.scenario.nurses[n].skills.contains(&req.skill) {
                                        if !self.is_forbidden_succession(n, d, s, genome) {
                                            self.set_assigned_shift(genome, n, d, Some(s));
                                            assigned_count += 1;
                                        }
                                    }
                                }
                            }

                            // Step 1b: If still needed, assign off-duty qualified nurse and clear d+1 succession conflict
                            if assigned_count < needed {
                                for n in 0..num_nurses {
                                    if assigned_count >= needed {
                                        break;
                                    }
                                    if self.get_assigned_shift(genome, n, d).is_none() {
                                        if self.context.scenario.nurses[n].skills.contains(&req.skill) {
                                            let nurse_id = &self.context.scenario.nurses[n].id;
                                            let prev_ok = if d == 0 {
                                                let hist = self.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id);
                                                if let Some(h) = hist {
                                                    if h.last_assigned_shift_type != "None" {
                                                        !self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == h.last_assigned_shift_type && r.succeeding.contains(shift_name))
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if let Some(prev_s) = self.get_assigned_shift(genome, n, d - 1) {
                                                    let prev_name = &self.context.shift_types[prev_s];
                                                    !self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == *prev_name && r.succeeding.contains(shift_name))
                                                } else {
                                                    true
                                                }
                                            };
                                            let next_ok = if d + 1 < num_days {
                                                if let Some(next_s) = self.get_assigned_shift(genome, n, d + 1) {
                                                    let next_name = &self.context.shift_types[next_s];
                                                    let is_forbidden = self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == *shift_name && r.succeeding.contains(next_name));
                                                    if is_forbidden {
                                                        let next_day_name = days_map[d + 1];
                                                        let mut can_spare = true;
                                                        for r in &self.context.week_data.requirements {
                                                            if r.shift_type == *next_name && self.context.scenario.nurses[n].skills.contains(&r.skill) {
                                                                let min_req = match next_day_name {
                                                                    "Monday" => r.monday.minimum,
                                                                    "Tuesday" => r.tuesday.minimum,
                                                                    "Wednesday" => r.wednesday.minimum,
                                                                    "Thursday" => r.thursday.minimum,
                                                                    "Friday" => r.friday.minimum,
                                                                    "Saturday" => r.saturday.minimum,
                                                                    "Sunday" => r.sunday.minimum,
                                                                    _ => 0,
                                                                };
                                                                let count = (0..num_nurses).filter(|&k| {
                                                                    self.get_assigned_shift(genome, k, d + 1) == Some(next_s)
                                                                        && self.context.scenario.nurses[k].skills.contains(&r.skill)
                                                                }).count();
                                                                if count <= min_req {
                                                                    can_spare = false;
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        can_spare
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                true
                                            };
                                            if prev_ok && next_ok {
                                                self.set_assigned_shift(genome, n, d, Some(s));
                                                if d + 1 < num_days {
                                                    if let Some(next_s) = self.get_assigned_shift(genome, n, d + 1) {
                                                        let next_name = &self.context.shift_types[next_s];
                                                        if self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == *shift_name && r.succeeding.contains(next_name)) {
                                                            self.set_assigned_shift(genome, n, d + 1, None);
                                                        }
                                                    }
                                                }
                                                assigned_count += 1;
                                            }
                                        }
                                    }
                                }
                            }

                            // Step 2: Swap surplus nurses from other shifts
                            if assigned_count < needed {
                                for n in 0..num_nurses {
                                    if assigned_count >= needed {
                                        break;
                                    }
                                    if let Some(other_s) = self.get_assigned_shift(genome, n, d) {
                                        if other_s != s
                                            && self.context.scenario.nurses[n]
                                                .skills
                                                .contains(&req.skill)
                                        {
                                            // Check if other_s can spare this nurse on day d
                                            let other_name = &self.context.shift_types[other_s];
                                            let mut can_spare_today = true;
                                            for r in &self.context.week_data.requirements {
                                                if r.shift_type == *other_name && self.context.scenario.nurses[n].skills.contains(&r.skill) {
                                                    let min_req = match day_name {
                                                        "Monday" => r.monday.minimum,
                                                        "Tuesday" => r.tuesday.minimum,
                                                        "Wednesday" => r.wednesday.minimum,
                                                        "Thursday" => r.thursday.minimum,
                                                        "Friday" => r.friday.minimum,
                                                        "Saturday" => r.saturday.minimum,
                                                        "Sunday" => r.sunday.minimum,
                                                        _ => 0,
                                                    };
                                                    let count = (0..num_nurses).filter(|&k| {
                                                        self.get_assigned_shift(genome, k, d) == Some(other_s)
                                                            && self.context.scenario.nurses[k].skills.contains(&r.skill)
                                                    }).count();
                                                    if count <= min_req {
                                                        can_spare_today = false;
                                                        break;
                                                    }
                                                }
                                            }
                                            if !can_spare_today {
                                                continue;
                                            }

                                            let nurse_id = &self.context.scenario.nurses[n].id;
                                            let prev_ok = if d == 0 {
                                                let hist = self.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id);
                                                if let Some(h) = hist {
                                                    if h.last_assigned_shift_type != "None" {
                                                        !self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == h.last_assigned_shift_type && r.succeeding.contains(shift_name))
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                if let Some(prev_s) = self.get_assigned_shift(genome, n, d - 1) {
                                                    let prev_name = &self.context.shift_types[prev_s];
                                                    !self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == *prev_name && r.succeeding.contains(shift_name))
                                                } else {
                                                    true
                                                }
                                            };
                                            let next_ok = if d + 1 < num_days {
                                                if let Some(next_s) = self.get_assigned_shift(genome, n, d + 1) {
                                                    let next_name = &self.context.shift_types[next_s];
                                                    let is_forbidden = self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == *shift_name && r.succeeding.contains(next_name));
                                                    if is_forbidden {
                                                        let next_day_name = days_map[d + 1];
                                                        let mut can_spare = true;
                                                        for r in &self.context.week_data.requirements {
                                                            if r.shift_type == *next_name && self.context.scenario.nurses[n].skills.contains(&r.skill) {
                                                                let min_req = match next_day_name {
                                                                    "Monday" => r.monday.minimum,
                                                                    "Tuesday" => r.tuesday.minimum,
                                                                    "Wednesday" => r.wednesday.minimum,
                                                                    "Thursday" => r.thursday.minimum,
                                                                    "Friday" => r.friday.minimum,
                                                                    "Saturday" => r.saturday.minimum,
                                                                    "Sunday" => r.sunday.minimum,
                                                                    _ => 0,
                                                                };
                                                                let count = (0..num_nurses).filter(|&k| {
                                                                    self.get_assigned_shift(genome, k, d + 1) == Some(next_s)
                                                                        && self.context.scenario.nurses[k].skills.contains(&r.skill)
                                                                }).count();
                                                                if count <= min_req {
                                                                    can_spare = false;
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        can_spare
                                                    } else {
                                                        true
                                                    }
                                                } else {
                                                    true
                                                }
                                            } else {
                                                true
                                            };
                                            if prev_ok && next_ok {
                                                self.set_assigned_shift(genome, n, d, Some(s));
                                                if d + 1 < num_days {
                                                    if let Some(next_s) = self.get_assigned_shift(genome, n, d + 1) {
                                                        let next_name = &self.context.shift_types[next_s];
                                                        if self.context.scenario.forbidden_shift_type_successions.iter().any(|r| r.preceding == *shift_name && r.succeeding.contains(next_name)) {
                                                            self.set_assigned_shift(genome, n, d + 1, None);
                                                        }
                                                    }
                                                }
                                                assigned_count += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !any_deficit {
                break;
            }
        }
    }

    /// Evaluates the total soft constraint penalties for a single nurse.
    pub fn eval_nurse_soft_penalty(&self, genome: &InrcGenome, n: usize) -> i32 {
        let nurse = &self.context.scenario.nurses[n];
        let contract = self
            .context
            .scenario
            .contracts
            .iter()
            .find(|c| c.id == nurse.contract)
            .unwrap();
        let weights = &self.context.weights;

        let mut penalty = 0i32;

        // Preferences for nurse n
        for req in &self.context.week_data.shift_off_requests {
            if req.nurse != nurse.id {
                continue;
            }
            let d = match req.day.as_str() {
                "Monday" => 0,
                "Tuesday" => 1,
                "Wednesday" => 2,
                "Thursday" => 3,
                "Friday" => 4,
                "Saturday" => 5,
                "Sunday" => 6,
                _ => continue,
            };
            if req.shift_type == "Any" || req.shift_type.is_empty() {
                if self.get_assigned_shift(genome, n, d).is_some() {
                    penalty += weights.preferences;
                }
            } else if let Some(s) = self
                .context
                .shift_types
                .iter()
                .position(|st| st == &req.shift_type)
            {
                if self.get_bit(genome, n, d, s) {
                    penalty += weights.preferences;
                }
            }
        }

        // Historical border states
        let hist = self
            .context
            .history
            .nurse_history
            .iter()
            .find(|h| &h.nurse == &nurse.id);
        let initial_streak = hist.map(|h| h.number_of_consecutive_working_days).unwrap_or(0);
        let initial_off_streak = hist.map(|h| h.number_of_consecutive_days_off).unwrap_or(0);
        let initial_shift_streak = hist.map(|h| h.number_of_consecutive_assignments).unwrap_or(0);
        let mut current_last_shift = hist.and_then(|h| {
            if h.last_assigned_shift_type.is_empty()
                || h.last_assigned_shift_type == "None"
            {
                None
            } else {
                Some(h.last_assigned_shift_type.clone())
            }
        });

        let mut current_streak = initial_streak;
        let mut current_off_streak = initial_off_streak;
        let mut current_shift_streak = initial_shift_streak;
        let mut week_assignments = 0;
        let mut works_saturday = false;
        let mut works_sunday = false;

        let num_days = self.context.num_days;

        for d in 0..num_days {
            let worked_shift = self.get_assigned_shift(genome, n, d);
            if let Some(s) = worked_shift {
                let shift_name = &self.context.shift_types[s];
                week_assignments += 1;
                current_streak += 1;
                if d == 5 {
                    works_saturday = true;
                }
                if d == 6 {
                    works_sunday = true;
                }

                if current_off_streak > 0 {
                    if current_off_streak < contract.min_consecutive_days_off {
                        penalty += (contract.min_consecutive_days_off - current_off_streak) as i32
                            * weights.consecutive_days_off;
                    }
                    if current_off_streak > contract.max_consecutive_days_off {
                        let current_excess =
                            current_off_streak - contract.max_consecutive_days_off;
                        let initial_excess =
                            if initial_off_streak > contract.max_consecutive_days_off {
                                initial_off_streak - contract.max_consecutive_days_off
                            } else {
                                0
                            };
                        if current_excess > initial_excess {
                            penalty += (current_excess - initial_excess) as i32
                                * weights.consecutive_days_off;
                        }
                    }
                    current_off_streak = 0;
                }

                if let Some(ref last) = current_last_shift {
                    if last == shift_name {
                        current_shift_streak += 1;
                    } else {
                        if let Some(s_type) = self
                            .context
                            .scenario
                            .shift_types
                            .iter()
                            .find(|st| &st.id == last)
                        {
                            if current_shift_streak > 0
                                && current_shift_streak < s_type.min_consecutive
                            {
                                penalty += (s_type.min_consecutive - current_shift_streak) as i32
                                    * weights.consecutive_shift_days;
                            }
                            if current_shift_streak > s_type.max_consecutive {
                                let current_excess =
                                    current_shift_streak - s_type.max_consecutive;
                                let initial_excess =
                                    if initial_shift_streak > s_type.max_consecutive {
                                        initial_shift_streak - s_type.max_consecutive
                                    } else {
                                        0
                                    };
                                if current_excess > initial_excess {
                                    penalty += (current_excess - initial_excess) as i32
                                        * weights.consecutive_shift_days;
                                }
                            }
                        }
                        current_shift_streak = 1;
                        current_last_shift = Some(shift_name.clone());
                    }
                } else {
                    current_shift_streak = 1;
                    current_last_shift = Some(shift_name.clone());
                }
            } else {
                current_off_streak += 1;
                if let Some(ref last) = current_last_shift {
                    if let Some(s_type) = self
                        .context
                        .scenario
                        .shift_types
                        .iter()
                        .find(|st| &st.id == last)
                    {
                        if current_shift_streak > 0 && current_shift_streak < s_type.min_consecutive
                        {
                            penalty += (s_type.min_consecutive - current_shift_streak) as i32
                                * weights.consecutive_shift_days;
                        }
                        if current_shift_streak > s_type.max_consecutive {
                            let current_excess = current_shift_streak - s_type.max_consecutive;
                            let initial_excess = if initial_shift_streak > s_type.max_consecutive {
                                initial_shift_streak - s_type.max_consecutive
                            } else {
                                0
                            };
                            if current_excess > initial_excess {
                                penalty += (current_excess - initial_excess) as i32
                                    * weights.consecutive_shift_days;
                            }
                        }
                    }
                    current_shift_streak = 0;
                    current_last_shift = None;
                }

                if current_streak > 0 {
                    if current_streak < contract.min_consecutive_working_days {
                        penalty += (contract.min_consecutive_working_days - current_streak) as i32
                            * weights.consecutive_working_days;
                    }
                    if current_streak > contract.max_consecutive_working_days {
                        let current_excess =
                            current_streak - contract.max_consecutive_working_days;
                        let initial_excess =
                            if initial_streak > contract.max_consecutive_working_days {
                                initial_streak - contract.max_consecutive_working_days
                            } else {
                                0
                            };
                        if current_excess > initial_excess {
                            penalty += (current_excess - initial_excess) as i32
                                * weights.consecutive_working_days;
                        }
                    }
                    current_streak = 0;
                }
            }
        }

        // End of week boundary handling
        if current_streak > contract.max_consecutive_working_days {
            let current_excess = current_streak - contract.max_consecutive_working_days;
            let initial_excess = if initial_streak > contract.max_consecutive_working_days {
                initial_streak - contract.max_consecutive_working_days
            } else {
                0
            };
            if current_excess > initial_excess {
                penalty +=
                    (current_excess - initial_excess) as i32 * weights.consecutive_working_days;
            }
        }
        if current_off_streak > contract.max_consecutive_days_off {
            let current_excess = current_off_streak - contract.max_consecutive_days_off;
            let initial_excess = if initial_off_streak > contract.max_consecutive_days_off {
                initial_off_streak - contract.max_consecutive_days_off
            } else {
                0
            };
            if current_excess > initial_excess {
                penalty += (current_excess - initial_excess) as i32 * weights.consecutive_days_off;
            }
        }
        if let Some(ref last) = current_last_shift {
            if let Some(s_type) = self
                .context
                .scenario
                .shift_types
                .iter()
                .find(|st| &st.id == last)
            {
                if current_shift_streak > s_type.max_consecutive {
                    let current_excess = current_shift_streak - s_type.max_consecutive;
                    let initial_excess = if initial_shift_streak > s_type.max_consecutive {
                        initial_shift_streak - s_type.max_consecutive
                    } else {
                        0
                    };
                    if current_excess > initial_excess {
                        penalty +=
                            (current_excess - initial_excess) as i32 * weights.consecutive_shift_days;
                    }
                }
            }
        }

        // Complete weekends
        if contract.complete_weekends > 0 && works_saturday != works_sunday {
            penalty += contract.complete_weekends as i32 * weights.complete_weekends;
        }

        // S7: Max working weekends with pro-rated horizon pacing
        let works_weekend = works_saturday || works_sunday;
        let hist_weekends = hist.map(|h| h.number_of_working_weekends).unwrap_or(0);
        let cum_weekends = hist_weekends + if works_weekend { 1 } else { 0 };
        let num_weeks = self.context.scenario.number_of_weeks;
        let week_k = (self.context.history.week + 1).min(num_weeks);

        if week_k == num_weeks {
            // Final week: official competition evaluation
            if cum_weekends > contract.max_working_weekends {
                penalty += (cum_weekends - contract.max_working_weekends) as i32
                    * weights.max_working_weekends;
            }
        } else {
            // Intermediate weeks (w in 0..W-1): pro-rated horizon pacing band (w+1)/W * max
            let pace_weekends = (week_k as f64 * contract.max_working_weekends as f64) / num_weeks as f64;
            if (cum_weekends as f64) > pace_weekends {
                let excess = (cum_weekends as f64) - pace_weekends;
                penalty += (excess * weights.max_working_weekends as f64).round() as i32;
            }
        }

        // S6: Total assignments with pro-rated cumulative horizon pacing band [(w+1)/W * min, (w+1)/W * max]
        let hist_assignments = hist.map(|h| h.number_of_assignments).unwrap_or(0);
        let cum_assignments = hist_assignments + week_assignments;

        if week_k == num_weeks {
            // Final week: official competition evaluation
            if cum_assignments < contract.min_assignments {
                penalty += (contract.min_assignments - cum_assignments) as i32 * weights.assignments;
            }
            if cum_assignments > contract.max_assignments {
                penalty += (cum_assignments - contract.max_assignments) as i32 * weights.assignments;
            }
        } else {
            // Intermediate weeks (w in 0..W-1): pro-rated cumulative horizon band
            let pace_min = (week_k as f64 * contract.min_assignments as f64) / num_weeks as f64;
            let pace_max = (week_k as f64 * contract.max_assignments as f64) / num_weeks as f64;
            if (cum_assignments as f64) < pace_min {
                let deficit = pace_min - (cum_assignments as f64);
                penalty += (deficit * weights.assignments as f64).round() as i32;
            }
            if (cum_assignments as f64) > pace_max {
                let excess = (cum_assignments as f64) - pace_max;
                penalty += (excess * weights.assignments as f64).round() as i32;
            }
        }

        penalty
    }

    /// Evaluates HC1 (min coverage), HC2 (skills), and S8 (optimal coverage) for day `d`.
    /// Returns (is_hard_feasible, optimal_coverage_penalty).
    pub fn eval_day_coverage(&self, genome: &InrcGenome, d: usize) -> (bool, i32) {
        let num_nurses = self.context.num_nurses;
        let num_shifts = self.context.shift_types.len();
        let days_map = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        let day_name = days_map[d];
        let mut total_s8 = 0i32;

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
                    return (false, 0);
                }
            }

            // Check HC1: Minimum Coverage & S8: Optimal Coverage
            let workers: Vec<(usize, Vec<String>)> = available_nurses
                .iter()
                .map(|&n| (n, self.context.scenario.nurses[n].skills.clone()))
                .collect();
            let matching = BipartiteMatchingSolver.assign(&workers, &demands);
            let mut fulfilled_map = HashMap::new();
            for (_, skill) in &matching.assignments {
                *fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
            }

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
                        return (false, 0);
                    } else if fulfilled < req_level.optimal {
                        total_s8 += (req_level.optimal - fulfilled) as i32 * self.context.weights.optimal_coverage;
                    }
                }
            }
        }
        (true, total_s8)
    }

    /// Evaluates coverage feasibility and S8 penalty across multiple days.
    pub fn eval_days_coverage(&self, genome: &InrcGenome, days: &[usize]) -> (bool, i32) {
        let mut total_s8 = 0i32;
        for &d in days {
            let (feasible, s8) = self.eval_day_coverage(genome, d);
            if !feasible {
                return (false, 0);
            }
            total_s8 += s8;
        }
        (true, total_s8)
    }

    /// Checks whether nurse `n` has any forbidden succession across the week schedule.
    pub fn nurse_has_forbidden_succession(&self, genome: &InrcGenome, n: usize) -> bool {
        let num_days = self.context.num_days;
        for d in 0..num_days {
            if let Some(s) = self.get_assigned_shift(genome, n, d) {
                if self.is_forbidden_succession(n, d, s, genome) {
                    return true;
                }
            }
        }
        false
    }

    /// High-throughput O(1) incremental Simulated Annealing within homogeneous skill groups.
    pub fn simulated_annealing(
        &self,
        genome: &mut InrcGenome,
        time_budget_ms: u64,
        rng: &mut StdRng,
    ) -> InrcEvaluation {
        let start = std::time::Instant::now();
        let num_nurses = self.context.num_nurses;
        let num_days = self.context.num_days;

        // Group nurses by identical skill sets to guarantee HC1 and HC2 invariance
        let mut skill_groups: HashMap<Vec<String>, Vec<usize>> = HashMap::new();
        for (n, nurse) in self.context.scenario.nurses.iter().enumerate() {
            let mut s = nurse.skills.clone();
            s.sort();
            skill_groups.entry(s).or_default().push(n);
        }
        let eligible_groups: Vec<Vec<usize>> = skill_groups
            .into_values()
            .filter(|g| g.len() >= 2)
            .collect();

        if eligible_groups.is_empty() && num_nurses < 2 {
            return self.evaluate(genome, &Default::default());
        }

        let initial_eval = self.evaluate(genome, &Default::default());
        let mut current_hc = initial_eval.hc_coverage
            + initial_eval.hc_skills
            + initial_eval.hc_one_shift_per_day
            + initial_eval.hc_forbidden_successions;

        let num_shifts = self.context.shift_types.len();

        // Phase 1: If genome has any residual hard constraint violations, actively repair them
        if current_hc > 0 {
            self.repair_hard_constraints(genome);
            let post_repair_eval = self.evaluate(genome, &Default::default());
            current_hc = post_repair_eval.hc_coverage
                + post_repair_eval.hc_skills
                + post_repair_eval.hc_one_shift_per_day
                + post_repair_eval.hc_forbidden_successions;
        }

        if current_hc > 0 {
            let mut best_hc = current_hc;
            let mut best_feas_genome = genome.clone();
            while start.elapsed().as_millis() < time_budget_ms as u128 && current_hc > 0 {
                let mut cand = genome.clone();
                let move_type = rng.gen_range(0..2);
                if move_type == 0 {
                    let n = rng.gen_range(0..num_nurses);
                    let d = rng.gen_range(0..num_days);
                    let new_shift = if rng.gen_bool(0.3) {
                        None
                    } else {
                        Some(rng.gen_range(0..num_shifts))
                    };
                    self.set_assigned_shift(&mut cand, n, d, new_shift);
                } else {
                    let n1 = rng.gen_range(0..num_nurses);
                    let mut n2 = rng.gen_range(0..num_nurses);
                    if n1 == n2 { n2 = (n1 + 1) % num_nurses; }
                    let d = rng.gen_range(0..num_days);
                    let s1 = self.get_assigned_shift(&cand, n1, d);
                    let s2 = self.get_assigned_shift(&cand, n2, d);
                    self.set_assigned_shift(&mut cand, n1, d, s2);
                    self.set_assigned_shift(&mut cand, n2, d, s1);
                }

                let cand_eval = self.evaluate(&cand, &Default::default());
                let cand_hc = cand_eval.hc_coverage
                    + cand_eval.hc_skills
                    + cand_eval.hc_one_shift_per_day
                    + cand_eval.hc_forbidden_successions;

                if cand_hc < current_hc || (cand_hc == current_hc && rng.gen_bool(0.05)) {
                    *genome = cand;
                    current_hc = cand_hc;
                    if current_hc < best_hc {
                        best_hc = current_hc;
                        best_feas_genome = genome.clone();
                    }
                }
            }
            if current_hc > 0 && best_hc < current_hc {
                *genome = best_feas_genome;
            }
        }

        // Precompute shift demanded skills for fast HC2 qualification filtering
        let days_map = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];
        let mut shift_demanded_skills: Vec<Vec<Vec<String>>> = vec![vec![Vec::new(); num_shifts]; num_days];
        for d in 0..num_days {
            let day_name = days_map[d];
            for s in 0..num_shifts {
                let shift_name = &self.context.shift_types[s];
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
                            shift_demanded_skills[d][s].push(req.skill.clone());
                        }
                    }
                }
            }
        }

        // Precompute nurse qualification matrix: nurse_can_work[n][d][s]
        let mut nurse_can_work: Vec<Vec<Vec<bool>>> = vec![vec![vec![false; num_shifts]; num_days]; num_nurses];
        for n in 0..num_nurses {
            let nurse_skills = &self.context.scenario.nurses[n].skills;
            for d in 0..num_days {
                for s in 0..num_shifts {
                    let demanded = &shift_demanded_skills[d][s];
                    nurse_can_work[n][d][s] = demanded.is_empty() || demanded.iter().any(|sk| nurse_skills.contains(sk));
                }
            }
        }

        // Pre-compute nurse soft penalties
        let mut nurse_penalties: Vec<i32> = (0..num_nurses)
            .map(|n| self.eval_nurse_soft_penalty(genome, n))
            .collect();
        let mut current_total_soft: i32 = nurse_penalties.iter().sum();

        let mut best_genome = genome.clone();
        let mut best_total_soft = current_total_soft;

        let mut temp = 50.0f64;
        let cool_rate = 0.99998f64;

        while start.elapsed().as_millis() < time_budget_ms as u128 {
            if rng.gen_bool(0.20) {
                // Move type: Single-nurse shift block reassignment (rebalance S8 optimal coverage & S1 assignment quotas)
                let n = rng.gen_range(0..num_nurses);
                let block_len = if rng.gen_bool(0.60) { 2 } else { 1 };
                let start_d = if block_len > 1 {
                    rng.gen_range(0..num_days - 1)
                } else {
                    rng.gen_range(0..num_days)
                };
                let days: Vec<usize> = (start_d..start_d + block_len).collect();

                let new_shift = if rng.gen_bool(0.35) {
                    None
                } else {
                    Some(rng.gen_range(0..num_shifts))
                };

                let mut can_work_all = true;
                if let Some(s_idx) = new_shift {
                    for &d in &days {
                        if !nurse_can_work[n][d][s_idx] {
                            can_work_all = false;
                            break;
                        }
                    }
                }
                if !can_work_all {
                    continue;
                }

                let mut old_shifts = Vec::with_capacity(days.len());
                for &d in &days {
                    old_shifts.push(self.get_assigned_shift(genome, n, d));
                }
                if old_shifts.iter().all(|&s| s == new_shift) {
                    continue;
                }

                let (_, old_s8) = self.eval_days_coverage(genome, &days);

                for &d in &days {
                    self.set_assigned_shift(genome, n, d, new_shift);
                }

                if self.nurse_has_forbidden_succession(genome, n) {
                    for (i, &d) in days.iter().enumerate() {
                        self.set_assigned_shift(genome, n, d, old_shifts[i]);
                    }
                    continue;
                }

                let (new_feasible, new_s8) = self.eval_days_coverage(genome, &days);
                if !new_feasible {
                    for (i, &d) in days.iter().enumerate() {
                        self.set_assigned_shift(genome, n, d, old_shifts[i]);
                    }
                    continue;
                }

                let new_pen = self.eval_nurse_soft_penalty(genome, n);
                let delta = (new_pen - nurse_penalties[n]) + (new_s8 - old_s8);

                let accept = if delta <= 0 {
                    true
                } else {
                    rng.gen_bool((-delta as f64 / temp.max(1.0)).exp().min(1.0))
                };

                if accept {
                    nurse_penalties[n] = new_pen;
                    current_total_soft += delta;
                    if current_total_soft < best_total_soft {
                        best_total_soft = current_total_soft;
                        best_genome = genome.clone();
                    }
                } else {
                    for (i, &d) in days.iter().enumerate() {
                        self.set_assigned_shift(genome, n, d, old_shifts[i]);
                    }
                }

                temp *= cool_rate;
                if temp < 0.1 {
                    temp = 50.0;
                }
                continue;
            }

            let is_cross_skill = rng.gen_bool(0.20) || eligible_groups.is_empty();
            let (n1, n2) = if !is_cross_skill && !eligible_groups.is_empty() {
                let group_idx = rng.gen_range(0..eligible_groups.len());
                let group = &eligible_groups[group_idx];
                let i1 = rng.gen_range(0..group.len());
                let mut i2 = rng.gen_range(0..group.len());
                if i1 == i2 {
                    i2 = (i1 + 1) % group.len();
                }
                (group[i1], group[i2])
            } else {
                let i1 = rng.gen_range(0..num_nurses);
                let mut i2 = rng.gen_range(0..num_nurses);
                if i1 == i2 {
                    i2 = (i1 + 1) % num_nurses;
                }
                (i1, i2)
            };

            let move_type = rng.gen_range(0..4);
            let days_to_swap: Vec<usize> = match move_type {
                0 => vec![rng.gen_range(0..num_days)],
                1 => vec![5, 6],
                2 => {
                    let d = rng.gen_range(0..num_days - 1);
                    vec![d, d + 1]
                }
                _ => {
                    let d = rng.gen_range(0..num_days - 2);
                    vec![d, d + 1, d + 2]
                }
            };

            // Fast HC2 qualification filter for cross-skill moves
            if is_cross_skill {
                let mut qualified = true;
                for &d in &days_to_swap {
                    let s1 = self.get_assigned_shift(genome, n1, d);
                    let s2 = self.get_assigned_shift(genome, n2, d);
                    if let Some(s1_idx) = s1 {
                        if !nurse_can_work[n2][d][s1_idx] {
                            qualified = false;
                            break;
                        }
                    }
                    if let Some(s2_idx) = s2 {
                        if !nurse_can_work[n1][d][s2_idx] {
                            qualified = false;
                            break;
                        }
                    }
                }
                if !qualified {
                    continue;
                }
            }

            let (_, old_s8) = if is_cross_skill {
                self.eval_days_coverage(genome, &days_to_swap)
            } else {
                (true, 0)
            };

            // Apply swap in-place
            let mut old_shifts_1 = Vec::with_capacity(days_to_swap.len());
            let mut old_shifts_2 = Vec::with_capacity(days_to_swap.len());

            for &d in &days_to_swap {
                let s1 = self.get_assigned_shift(genome, n1, d);
                let s2 = self.get_assigned_shift(genome, n2, d);
                old_shifts_1.push(s1);
                old_shifts_2.push(s2);
                self.set_assigned_shift(genome, n1, d, s2);
                self.set_assigned_shift(genome, n2, d, s1);
            }

            // Check succession validity (HC4)
            let valid_succession = !self.nurse_has_forbidden_succession(genome, n1)
                && !self.nurse_has_forbidden_succession(genome, n2);

            if !valid_succession {
                // Revert swap in-place
                for (i, &d) in days_to_swap.iter().enumerate() {
                    self.set_assigned_shift(genome, n1, d, old_shifts_1[i]);
                    self.set_assigned_shift(genome, n2, d, old_shifts_2[i]);
                }
                continue;
            }

            // For cross-skill swap, check exact HC1/HC2 feasibility and S8 optimal coverage
            let (new_feasible, new_s8) = if is_cross_skill {
                self.eval_days_coverage(genome, &days_to_swap)
            } else {
                (true, 0)
            };

            if !new_feasible {
                // Revert swap in-place
                for (i, &d) in days_to_swap.iter().enumerate() {
                    self.set_assigned_shift(genome, n1, d, old_shifts_1[i]);
                    self.set_assigned_shift(genome, n2, d, old_shifts_2[i]);
                }
                continue;
            }

            // Calculate exact delta penalty across 2 nurse rows plus S8 delta
            let new_pen_1 = self.eval_nurse_soft_penalty(genome, n1);
            let new_pen_2 = self.eval_nurse_soft_penalty(genome, n2);
            let delta = (new_pen_1 + new_pen_2) - (nurse_penalties[n1] + nurse_penalties[n2]) + (new_s8 - old_s8);

            let accept = if delta <= 0 {
                true
            } else {
                rng.gen_bool((-delta as f64 / temp.max(1.0)).exp().min(1.0))
            };

            if accept {
                nurse_penalties[n1] = new_pen_1;
                nurse_penalties[n2] = new_pen_2;
                current_total_soft += delta;

                if current_total_soft < best_total_soft {
                    best_total_soft = current_total_soft;
                    best_genome = genome.clone();
                }
            } else {
                // Revert swap in-place
                for (i, &d) in days_to_swap.iter().enumerate() {
                    self.set_assigned_shift(genome, n1, d, old_shifts_1[i]);
                    self.set_assigned_shift(genome, n2, d, old_shifts_2[i]);
                }
            }

            temp *= cool_rate;
            if temp < 0.1 {
                temp = 50.0;
            }
        }

        *genome = best_genome;
        self.evaluate(genome, &Default::default())
    }
}

impl FitnessEvaluator<InrcGenome> for InrcOptimizer {
    type Evaluation = InrcEvaluation;

    fn evaluate(
        &self,
        genome: &InrcGenome,
        _metrics: &coralys_moga::runtime::optimization::metric::MetricReport,
    ) -> Self::Evaluation {
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
                        description: format!(
                            "Nurse {} assigned to {} shifts on day {}",
                            nurse_id, shifts_assigned, d
                        ),
                        penalty: cost as i32,
                    });
                }
            }
        }

        // Check HC4: Forbidden Shift Successions
        for n in 0..num_nurses {
            let nurse_id = &self.context.scenario.nurses[n].id;
            let hist = self
                .context
                .history
                .nurse_history
                .iter()
                .find(|h| &h.nurse == nurse_id);

            let mut previous_shift_type = if let Some(h) = hist {
                if h.number_of_consecutive_days_off > 0 {
                    "None".to_string()
                } else {
                    h.last_assigned_shift_type.clone()
                }
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

                if current_shift_type != "None"
                    && previous_shift_type != "None"
                    && previous_shift_type != ""
                {
                    if let Some(rule) = self
                        .context
                        .scenario
                        .forbidden_shift_type_successions
                        .iter()
                        .find(|r| r.preceding == previous_shift_type)
                    {
                        if rule.succeeding.contains(&current_shift_type) {
                            hc_forbidden_successions += weights.hard_constraint_violation as usize;
                            hard_violations.push(Violation {
                                constraint_id: "HC3_ForbiddenShiftSuccession".to_string(),
                                severity: "Hard".to_string(),
                                value: None,
                                expected: format!(
                                    "No transition from {} to {}",
                                    previous_shift_type, current_shift_type
                                ),
                                actual: format!(
                                    "Transitioned from {} to {}",
                                    previous_shift_type, current_shift_type
                                ),
                                description: format!(
                                    "Nurse {} forbidden succession: {} followed by {} on day {}",
                                    nurse_id, previous_shift_type, current_shift_type, d
                                ),
                                penalty: weights.hard_constraint_violation as i32,
                            });
                        }
                    }
                }

                if current_shift_type != "None"
                    || previous_shift_type == "None"
                    || previous_shift_type == ""
                {
                    previous_shift_type = current_shift_type;
                } else if current_shift_type == "None" {
                    previous_shift_type = "None".to_string();
                }
            }
        }

        // Check HC1 & HC2: Coverage & Skills using BipartiteMatchingSolver
        let days_map = vec![
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday",
        ];

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
                    let has_valid_skill = demands
                        .iter()
                        .any(|(skill, _)| nurse.skills.contains(skill));
                    if !has_valid_skill && !demands.is_empty() {
                        hc_skills += weights.hard_constraint_violation as usize;
                        hard_violations.push(Violation {
                            constraint_id: "HC2_SkillRequirements".to_string(),
                            severity: "Hard".to_string(),
                            value: None,
                            expected: "Nurse possesses required skill".to_string(),
                            actual: format!("Nurse skills: {:?}", nurse.skills),
                            description: format!(
                                "Nurse {} lacks required skill for shift {} on day {}",
                                nurse.id, shift_name, day_name
                            ),
                            penalty: weights.hard_constraint_violation as i32,
                        });
                    }
                }

                // Run AssignmentSolver for Optimal Matching
                let workers: Vec<(usize, Vec<String>)> = available_nurses
                    .iter()
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
                                description: format!(
                                    "Shift {} skill {} on {} lacks {} nurses (minimum)",
                                    shift_name, req.skill, day_name, missing
                                ),
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
                                description: format!(
                                    "Shift {} skill {} on {} lacks {} nurses (optimal)",
                                    shift_name, req.skill, day_name, missing
                                ),
                                penalty: cost,
                            });
                        }
                    }
                }
            }
        }

        // Shift Off Requests (Preferences)
        for req in &self.context.week_data.shift_off_requests {
            let n_idx = self
                .context
                .scenario
                .nurses
                .iter()
                .position(|n| n.id == req.nurse);
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
                                description: format!(
                                    "Nurse {} shift off request violated on day {}",
                                    req.nurse, req.day
                                ),
                                penalty: weights.preferences,
                            });
                        }
                    }
                } else {
                    if let Some(s) = self
                        .context
                        .shift_types
                        .iter()
                        .position(|s| s == &req.shift_type)
                    {
                        if self.get_bit(genome, n, d, s) {
                            preferences_penalty += weights.preferences;
                            soft_violations.push(Violation {
                                constraint_id: "S5_Preferences".to_string(),
                                severity: "Soft".to_string(),
                                value: None,
                                expected: format!("Shift off requested: {}", req.shift_type),
                                actual: format!("Shift assigned: {}", req.shift_type),
                                description: format!(
                                    "Nurse {} shift off request ({}) violated on day {}",
                                    req.nurse, req.shift_type, req.day
                                ),
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
            let contract = self
                .context
                .scenario
                .contracts
                .iter()
                .find(|c| c.id == nurse.contract)
                .unwrap();

            let hist = self
                .context
                .history
                .nurse_history
                .iter()
                .find(|h| &h.nurse == &nurse.id);
            let initial_streak = if let Some(h) = hist {
                h.number_of_consecutive_working_days
            } else {
                0
            };
            let initial_off_streak = if let Some(h) = hist {
                h.number_of_consecutive_days_off
            } else {
                0
            };
            let initial_shift_streak = if let Some(h) = hist {
                h.number_of_consecutive_assignments
            } else {
                0
            };
            let mut current_last_shift = if let Some(h) = hist {
                Some(h.last_assigned_shift_type.clone())
            } else {
                None
            };

            if current_last_shift.as_deref() == Some("")
                || current_last_shift.as_deref() == Some("None")
            {
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

                    if d == 5 {
                        works_saturday = true;
                    }
                    if d == 6 {
                        works_sunday = true;
                    }

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
                            let current_excess =
                                current_off_streak - contract.max_consecutive_days_off;
                            let initial_excess =
                                if initial_off_streak > contract.max_consecutive_days_off {
                                    initial_off_streak - contract.max_consecutive_days_off
                                } else {
                                    0
                                };
                            if current_excess > initial_excess {
                                let cost = (current_excess - initial_excess) as i32
                                    * weights.consecutive_days_off;
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
                                let s_type = self
                                    .context
                                    .scenario
                                    .shift_types
                                    .iter()
                                    .find(|st| &st.id == last)
                                    .unwrap();
                                if current_shift_streak > 0
                                    && current_shift_streak < s_type.min_consecutive
                                {
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
                                    let current_excess =
                                        current_shift_streak - s_type.max_consecutive;
                                    let initial_excess =
                                        if initial_shift_streak > s_type.max_consecutive {
                                            initial_shift_streak - s_type.max_consecutive
                                        } else {
                                            0
                                        };
                                    if current_excess > initial_excess {
                                        let cost = (current_excess - initial_excess) as i32
                                            * weights.consecutive_shift_days;
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
                        let s_type = self
                            .context
                            .scenario
                            .shift_types
                            .iter()
                            .find(|st| &st.id == last)
                            .unwrap();
                        if current_shift_streak > 0 && current_shift_streak < s_type.min_consecutive
                        {
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
                            let initial_excess = if initial_shift_streak > s_type.max_consecutive {
                                initial_shift_streak - s_type.max_consecutive
                            } else {
                                0
                            };
                            if current_excess > initial_excess {
                                let cost = (current_excess - initial_excess) as i32
                                    * weights.consecutive_shift_days;
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
                            let current_excess =
                                current_streak - contract.max_consecutive_working_days;
                            let initial_excess =
                                if initial_streak > contract.max_consecutive_working_days {
                                    initial_streak - contract.max_consecutive_working_days
                                } else {
                                    0
                                };
                            if current_excess > initial_excess {
                                let cost = (current_excess - initial_excess) as i32
                                    * weights.consecutive_working_days;
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
                let initial_excess = if initial_streak > contract.max_consecutive_working_days {
                    initial_streak - contract.max_consecutive_working_days
                } else {
                    0
                };
                if current_excess > initial_excess {
                    let cost =
                        (current_excess - initial_excess) as i32 * weights.consecutive_working_days;
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
                let initial_excess = if initial_off_streak > contract.max_consecutive_days_off {
                    initial_off_streak - contract.max_consecutive_days_off
                } else {
                    0
                };
                if current_excess > initial_excess {
                    let cost =
                        (current_excess - initial_excess) as i32 * weights.consecutive_days_off;
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
                let s_type = self
                    .context
                    .scenario
                    .shift_types
                    .iter()
                    .find(|st| &st.id == last)
                    .unwrap();
                if current_shift_streak > s_type.max_consecutive {
                    let current_excess = current_shift_streak - s_type.max_consecutive;
                    let initial_excess = if initial_shift_streak > s_type.max_consecutive {
                        initial_shift_streak - s_type.max_consecutive
                    } else {
                        0
                    };
                    if current_excess > initial_excess {
                        let cost = (current_excess - initial_excess) as i32
                            * weights.consecutive_shift_days;
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
                        description: format!(
                            "Nurse {} had incomplete working weekend (Sat: {}, Sun: {})",
                            nurse.id, works_saturday, works_sunday
                        ),
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
                    description: format!(
                        "Nurse {} exceeded max working weekends: {} (maximum {})",
                        nurse.id, total_working_weekends, contract.max_working_weekends
                    ),
                    penalty: cost,
                });
            }

            // S1: Assignments (Scaled for 1 week)
            let weekly_min = contract.min_assignments / self.context.scenario.number_of_weeks;
            let weekly_max = (contract.max_assignments as f64
                / self.context.scenario.number_of_weeks as f64)
                .ceil() as usize;

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
                    description: format!(
                        "Nurse {} total assignments {} below weekly minimum {}",
                        nurse.id, week_assignments, weekly_min
                    ),
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
                    description: format!(
                        "Nurse {} total assignments {} above weekly maximum {}",
                        nurse.id, week_assignments, weekly_max
                    ),
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
            total_penalty: assignment_penalty
                + work_streak_penalty
                + day_off_penalty
                + weekend_penalty
                + preferences_penalty
                + optimal_coverage_penalty,
        };

        let total_hc_penalty =
            hc_coverage + hc_skills + hc_one_shift_per_day + hc_forbidden_successions;

        let base_fitness = 10_000_000.0;
        let fitness = base_fitness - total_hc_penalty as f64 - soft_report.total_penalty as f64;

        // Build the platform EvaluationResult (vector of objectives)
        let mut metrics = HashMap::new();
        metrics.insert("hc_coverage".to_string(), hc_coverage as f64);
        metrics.insert("hc_skills".to_string(), hc_skills as f64);
        metrics.insert(
            "hc_one_shift_per_day".to_string(),
            hc_one_shift_per_day as f64,
        );
        metrics.insert(
            "hc_forbidden_successions".to_string(),
            hc_forbidden_successions as f64,
        );
        metrics.insert("assignment_penalty".to_string(), assignment_penalty as f64);
        metrics.insert(
            "work_streak_penalty".to_string(),
            work_streak_penalty as f64,
        );
        metrics.insert("day_off_penalty".to_string(), day_off_penalty as f64);
        metrics.insert("weekend_penalty".to_string(), weekend_penalty as f64);
        metrics.insert(
            "preferences_penalty".to_string(),
            preferences_penalty as f64,
        );
        metrics.insert(
            "optimal_coverage_penalty".to_string(),
            optimal_coverage_penalty as f64,
        );

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
