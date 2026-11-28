use super::annealing::{AnnealingConfig, AnnealingDomain, SimulatedAnnealingEngine};
use super::optimization::{InrcEvaluation, InrcGenome, InrcOptimizer, SoftConstraintReport};
use coralys_core::{EvaluationResult, Violation};
use coralys_matching::{AssignmentSolver, BipartiteMatchingSolver};
use coralys_moga::traits::FitnessEvaluator;
use rand::rngs::StdRng;
use std::collections::HashMap;

#[inline(always)]
fn dfs_kuhn(
    w: usize,
    demand_slots: &[u8],
    worker_mask: u8,
    match_slot: &mut [i8],
    match_worker: &mut [i8],
    visited: &mut [bool],
    worker_masks: &[u8],
) -> bool {
    for (slot_idx, &req_skill) in demand_slots.iter().enumerate() {
        if (worker_mask & (1 << req_skill)) != 0 && !visited[slot_idx] {
            visited[slot_idx] = true;
            let current_matched_w = match_slot[slot_idx];
            if current_matched_w < 0
                || dfs_kuhn(
                    current_matched_w as usize,
                    demand_slots,
                    worker_masks[current_matched_w as usize],
                    match_slot,
                    match_worker,
                    visited,
                    worker_masks,
                )
            {
                match_slot[slot_idx] = w as i8;
                match_worker[w] = slot_idx as i8;
                return true;
            }
        }
    }
    false
}

#[inline(always)]
pub fn fast_eval_shift_coverage(
    demand: &super::optimization::FastShiftDemand,
    worker_skill_masks: &[u8],
    optimal_coverage_weight: i32,
) -> (bool, i32) {
    let num_workers = worker_skill_masks.len();
    if num_workers < demand.total_min {
        return (false, 0);
    }
    if demand.total_min == 0 && demand.opt_slots.is_empty() {
        return (true, 0);
    }

    const MAX_SLOTS: usize = 64;
    const MAX_WORKERS: usize = 64;

    let num_min = demand.min_slots.len();
    let num_opt = demand.opt_slots.len();

    if num_min > MAX_SLOTS || num_opt > MAX_SLOTS || num_workers > MAX_WORKERS {
        return (false, 0);
    }

    // Stage 1: Match Mandatory Minimums
    let mut match_slot = [-1i8; MAX_SLOTS];
    let mut match_worker = [-1i8; MAX_WORKERS];
    let mut cardinality_min = 0;

    for w in 0..num_workers {
        let mut visited = [false; MAX_SLOTS];
        if dfs_kuhn(
            w,
            &demand.min_slots,
            worker_skill_masks[w],
            &mut match_slot[0..num_min],
            &mut match_worker,
            &mut visited[0..num_min],
            worker_skill_masks,
        ) {
            cardinality_min += 1;
        }
    }

    if cardinality_min < demand.total_min {
        return (false, 0);
    }

    // Stage 2: Match Optimal Increments using only workers not assigned in Stage 1
    let mut opt_match_slot = [-1i8; MAX_SLOTS];
    let mut opt_match_worker = [-1i8; MAX_WORKERS];
    if num_opt > 0 {
        for w in 0..num_workers {
            if match_worker[w] >= 0 {
                continue;
            }
            let mut visited = [false; MAX_SLOTS];
            dfs_kuhn(
                w,
                &demand.opt_slots,
                worker_skill_masks[w],
                &mut opt_match_slot[0..num_opt],
                &mut opt_match_worker,
                &mut visited[0..num_opt],
                worker_skill_masks,
            );
        }
    }

    let mut fulfilled = [0usize; 8];
    for s in 0..num_min {
        if match_slot[s] >= 0 {
            let skill_idx = demand.min_slots[s] as usize;
            if skill_idx < 8 {
                fulfilled[skill_idx] += 1;
            }
        }
    }
    for s in 0..num_opt {
        if opt_match_slot[s] >= 0 {
            let skill_idx = demand.opt_slots[s] as usize;
            if skill_idx < 8 {
                fulfilled[skill_idx] += 1;
            }
        }
    }

    let mut total_s8 = 0i32;
    for (k, &opt_needed) in demand.opt_demands_by_skill.iter().enumerate() {
        if k < 8 && fulfilled[k] < opt_needed {
            total_s8 += (opt_needed - fulfilled[k]) as i32 * optimal_coverage_weight;
        }
    }

    (true, total_s8)
}

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
        let days_map = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];
        for n in 0..num_nurses {
            for d in 0..num_days {
                if let Some(s) = self.get_assigned_shift(genome, n, d) {
                    if self.is_forbidden_succession(n, d, s, genome) {
                        let mut fixed = false;
                        for alt_s in 0..num_shifts {
                            if alt_s != s && !self.is_forbidden_succession(n, d, alt_s, genome) {
                                let alt_shift_name = &self.context.shift_types[alt_s];
                                let alt_day_name = days_map[d];
                                let nurse_skills = &self.context.scenario.nurses[n].skills;
                                let has_demanded_skill = self.context.week_data.requirements.iter().any(|req| {
                                    if req.shift_type == *alt_shift_name {
                                        let req_level = match alt_day_name {
                                            "Monday" => &req.monday,
                                            "Tuesday" => &req.tuesday,
                                            "Wednesday" => &req.wednesday,
                                            "Thursday" => &req.thursday,
                                            "Friday" => &req.friday,
                                            "Saturday" => &req.saturday,
                                            "Sunday" => &req.sunday,
                                            _ => unreachable!(),
                                        };
                                        (req_level.minimum > 0 || req_level.optimal > 0) && nurse_skills.contains(&req.skill)
                                    } else {
                                        false
                                    }
                                });
                                if has_demanded_skill {
                                    self.set_assigned_shift(genome, n, d, Some(alt_s));
                                    fixed = true;
                                    break;
                                }
                            }
                        }
                        if !fixed {
                            self.set_assigned_shift(genome, n, d, None);
                        }
                    }
                }
            }
        }

        // 2b. Eject any nurse assigned to a shift where they lack required skills (HC2)
        for n in 0..num_nurses {
            for d in 0..num_days {
                if let Some(s) = self.get_assigned_shift(genome, n, d) {
                    if !self.context.inverted.nurse_can_work[n][d][s] {
                        self.set_assigned_shift(genome, n, d, None);
                    }
                }
            }
        }

        // 3. Ejection chain repair for minimum coverage & skills (HC1 & HC2)

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
        let num_weeks = self.context.scenario.number_of_weeks;
        let week_k = (self.context.history.week + 1).min(num_weeks);

        if week_k < num_weeks {
            // Sunday Border Lookahead (w < W - 1): penalize active work/shift stubs ending on Sunday
            if current_streak > 0 && current_streak < contract.min_consecutive_working_days {
                penalty += (contract.min_consecutive_working_days - current_streak) as i32
                    * weights.consecutive_working_days;
            }
            // Sunday Streak Saturation Lookahead: penalize nurses at max consecutive days on Sunday to prevent Monday lockouts
            if current_streak >= contract.max_consecutive_working_days {
                penalty += weights.consecutive_working_days;
            }
            if let Some(ref last) = current_last_shift {
                if let Some(s_type) = self
                    .context
                    .scenario
                    .shift_types
                    .iter()
                    .find(|st| &st.id == last)
                {
                    if current_shift_streak > 0 && current_shift_streak < s_type.min_consecutive {
                        penalty += (s_type.min_consecutive - current_shift_streak) as i32
                            * weights.consecutive_shift_days;
                    }
                }
            }
        }

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

        // S7: Max working weekends
        let works_weekend = works_saturday || works_sunday;
        let total_working_weekends = if works_weekend { 1 } else { 0 };
        if total_working_weekends > contract.max_working_weekends {
            penalty += (total_working_weekends - contract.max_working_weekends) as i32
                * weights.max_working_weekends;
        }

        // S1: Assignments
        let weekly_min = contract.min_assignments / self.context.scenario.number_of_weeks;
        let weekly_max = (contract.max_assignments as f64
            / self.context.scenario.number_of_weeks as f64)
            .ceil() as usize;
        if week_assignments < weekly_min {
            penalty += (weekly_min - week_assignments) as i32 * weights.assignments;
        }
        if week_assignments > weekly_max {
            penalty += (week_assignments - weekly_max) as i32 * weights.assignments;
        }

        penalty
    }

    /// Evaluates HC1 (min coverage), HC2 (skills), and S8 (optimal coverage) for day `d`.
    /// Returns (is_hard_feasible, optimal_coverage_penalty).
    pub fn eval_day_coverage(&self, genome: &InrcGenome, d: usize) -> (bool, i32) {
        let num_nurses = self.context.num_nurses;
        let num_shifts = self.context.shift_types.len();
        let mut total_s8 = 0i32;

        for s in 0..num_shifts {
            let demand = &self.context.fast_shift_demands[d][s];
            let mut worker_masks = [0u8; 64];
            let mut worker_count = 0;
            for n in 0..num_nurses {
                if self.get_bit(genome, n, d, s) {
                    if worker_count < 64 {
                        worker_masks[worker_count] = self.context.nurse_skill_masks[n];
                        worker_count += 1;
                    }
                }
            }

            let (feas, s8) = fast_eval_shift_coverage(
                demand,
                &worker_masks[0..worker_count],
                self.context.weights.optimal_coverage,
            );
            if !feas {
                return (false, 0);
            }
            total_s8 += s8;
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

    /// High-throughput O(1) incremental Simulated Annealing powered by Coralys.
    /// All decision-making (cooling schedules, acceptance gates, stagnation restarts)
    /// is executed directly by [`coralys_moga::annealing::SimulatedAnnealingEngine`].
    pub fn simulated_annealing(
        &self,
        genome: &mut InrcGenome,
        time_budget_ms: u64,
        rng: &mut StdRng,
    ) -> InrcEvaluation {
        let initial_eval = self.evaluate(genome, &Default::default());
        let current_hc = initial_eval.hc_coverage
            + initial_eval.hc_skills
            + initial_eval.hc_one_shift_per_day
            + initial_eval.hc_forbidden_successions;

        if current_hc > 0 {
            self.repair_hard_constraints(genome);
        }

        // Pre-SA Rotation Projection Pass (seed SA with zero-defect rotation patterns)
        self.optimize_rotations(genome);

        let domain = InrcAnnealingDomain::new(self);
        // Pre-SA Min-Cost Flow Augmentation (staff unmet optimal slots with qualified idle nurses)
        domain.optimal_flow.optimize_all_days(self, genome, &domain.pacing);

        let sa_config = AnnealingConfig {
            initial_temperature: 50.0,
            cooling_rate: 0.99998,
            min_temperature: 0.1,
            restart_temperature: 50.0,
            time_budget_ms,
        };
        let sa_engine = SimulatedAnnealingEngine::new(sa_config);

        let (best_genome, _, _) = sa_engine.optimize(&domain, genome, rng);
        *genome = best_genome;

        // Post-SA Min-Cost Flow Polish (capture remaining free optimal coverage)
        domain.optimal_flow.optimize_all_days(self, genome, &domain.pacing);

        // Post-SA Rotation Projection Polish Pass
        self.optimize_rotations(genome);

        self.evaluate(genome, &Default::default())
    }

    /// Performs greedy descent in rotation space:
    /// Pass 1: Proposes candidate zero-defect rotations from `RotationPool` for nurses with soft defects,
    ///         accepting any move that preserves hard constraint feasibility and improves soft penalties.
    /// Pass 2: Pairwise rotation Kempe swaps between nurses sharing identical contract and skill profiles
    ///         (strictly invariant coverage) to resolve boundary/weekend/assignment defects.
    pub fn optimize_rotations(&self, genome: &mut InrcGenome) {
        let num_nurses = self.context.num_nurses;
        let mut improved = true;
        let mut iter = 0;

        while improved && iter < 3 {
            improved = false;
            iter += 1;

            // Pass 1: Single-nurse rotation projection
            for n in 0..num_nurses {
                let old_p = self.eval_nurse_soft_penalty(genome, n);
                if old_p == 0 {
                    continue;
                }

                let candidates = &self.context.candidate_rotations[n];
                let mut best_cand: Option<(InrcGenome, i32)> = None;
                let mut max_gain = 0;

                for (cand_rot, _) in candidates.iter().take(80) {
                    let mut test_genome = genome.clone();
                    let mut changed_days = Vec::new();
                    for d in 0..self.context.num_days {
                        let cur_s = self.get_assigned_shift(genome, n, d);
                        let rot_s = cand_rot.get_shift(d);
                        if cur_s != rot_s {
                            changed_days.push(d);
                            self.set_assigned_shift(&mut test_genome, n, d, rot_s);
                        }
                    }

                    if changed_days.is_empty() {
                        continue;
                    }

                    if self.nurse_has_forbidden_succession(&test_genome, n) {
                        continue;
                    }

                    let (is_feas, new_s8) = self.eval_days_coverage(&test_genome, &changed_days);
                    if !is_feas {
                        continue;
                    }

                    let (_, old_s8) = self.eval_days_coverage(genome, &changed_days);
                    let new_p = self.eval_nurse_soft_penalty(&test_genome, n);
                    let gain = (old_p - new_p) + (old_s8 - new_s8);

                    if gain > max_gain {
                        max_gain = gain;
                        best_cand = Some((test_genome, gain));
                    }
                }

                if let Some((better_genome, _)) = best_cand {
                    *genome = better_genome;
                    improved = true;
                }
            }

            // Pass 2: Pairwise Rotation Kempe Swaps
            let mut map: HashMap<String, Vec<usize>> = HashMap::new();
            for n in 0..num_nurses {
                let contract_id = self.context.scenario.nurses[n].contract.clone();
                map.entry(contract_id).or_default().push(n);
            }

            for nurses in map.values() {
                if nurses.len() < 2 {
                    continue;
                }
                for i in 0..nurses.len() {
                    for j in (i + 1)..nurses.len() {
                        let u = nurses[i];
                        let v = nurses[j];

                        let mut qualified = true;
                        for d in 0..self.context.num_days {
                            if let Some(s_v) = self.get_assigned_shift(genome, v, d) {
                                if !self.context.inverted.nurse_can_work[u][d][s_v] {
                                    qualified = false;
                                    break;
                                }
                            }
                            if let Some(s_u) = self.get_assigned_shift(genome, u, d) {
                                if !self.context.inverted.nurse_can_work[v][d][s_u] {
                                    qualified = false;
                                    break;
                                }
                            }
                        }
                        if !qualified {
                            continue;
                        }

                        let old_p_u = self.eval_nurse_soft_penalty(genome, u);
                        let old_p_v = self.eval_nurse_soft_penalty(genome, v);

                        let mut cand = genome.clone();
                        for d in 0..self.context.num_days {
                            let s_u = self.get_assigned_shift(genome, u, d);
                            let s_v = self.get_assigned_shift(genome, v, d);
                            self.set_assigned_shift(&mut cand, u, d, s_v);
                            self.set_assigned_shift(&mut cand, v, d, s_u);
                        }

                        if self.nurse_has_forbidden_succession(&cand, u)
                            || self.nurse_has_forbidden_succession(&cand, v)
                        {
                            continue;
                        }

                        let (is_feas, new_s8) = self.eval_days_coverage(&cand, &[0, 1, 2, 3, 4, 5, 6]);
                        if !is_feas {
                            continue;
                        }

                        let (_, old_s8) = self.eval_days_coverage(genome, &[0, 1, 2, 3, 4, 5, 6]);
                        let new_p_u = self.eval_nurse_soft_penalty(&cand, u);
                        let new_p_v = self.eval_nurse_soft_penalty(&cand, v);
                        let gain = ((old_p_u + old_p_v) - (new_p_u + new_p_v)) + (old_s8 - new_s8);

                        if gain > 0 {
                            *genome = cand;
                            improved = true;
                        }
                    }
                }
            }

            // Pass 3: Targeted Stub Transfer (Heal isolated 1-day work stubs to adjacent working nurses)
            for u in 0..num_nurses {
                for d in 0..self.context.num_days {
                    let s_u = match self.get_assigned_shift(genome, u, d) {
                        Some(s) => s,
                        None => continue,
                    };

                    let prev_works = if d == 0 {
                        let hist = self.context.history.nurse_history.iter().find(|h| h.nurse == self.context.scenario.nurses[u].id);
                        hist.map(|h| h.number_of_consecutive_working_days > 0).unwrap_or(false)
                    } else {
                        self.get_assigned_shift(genome, u, d - 1).is_some()
                    };

                    let next_works = if d + 1 < self.context.num_days {
                        self.get_assigned_shift(genome, u, d + 1).is_some()
                    } else {
                        false
                    };

                    // Isolated work stub
                    if !prev_works && !next_works {
                        for v in 0..num_nurses {
                            if v == u || self.get_assigned_shift(genome, v, d).is_some() {
                                continue;
                            }
                            if !self.context.inverted.nurse_can_work[v][d][s_u] {
                                continue;
                            }

                            let v_prev_works = if d == 0 {
                                let hist = self.context.history.nurse_history.iter().find(|h| h.nurse == self.context.scenario.nurses[v].id);
                                hist.map(|h| h.number_of_consecutive_working_days > 0).unwrap_or(false)
                            } else {
                                self.get_assigned_shift(genome, v, d - 1).is_some()
                            };

                            let v_next_works = if d + 1 < self.context.num_days {
                                self.get_assigned_shift(genome, v, d + 1).is_some()
                            } else {
                                false
                            };

                            if v_prev_works || v_next_works {
                                let mut cand = genome.clone();
                                self.set_assigned_shift(&mut cand, u, d, None);
                                self.set_assigned_shift(&mut cand, v, d, Some(s_u));

                                if !self.nurse_has_forbidden_succession(&cand, u)
                                    && !self.nurse_has_forbidden_succession(&cand, v)
                                {
                                    let (is_feas, new_s8) = self.eval_days_coverage(&cand, &[d]);
                                    if is_feas {
                                        let (_, old_s8) = self.eval_days_coverage(genome, &[d]);
                                        let old_p = self.eval_nurse_soft_penalty(genome, u) + self.eval_nurse_soft_penalty(genome, v);
                                        let new_p = self.eval_nurse_soft_penalty(&cand, u) + self.eval_nurse_soft_penalty(&cand, v);
                                        let gain = (old_p - new_p) + (old_s8 - new_s8);

                                        if gain > 0 {
                                            *genome = cand;
                                            improved = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Pass 4: Shift Streak Kempe Harmonizer (Direct S4 Eliminator)
            // Eliminates shift type fragmentation by swapping shifts on day d between two working nurses
            for d in 0..self.context.num_days {
                for u in 0..num_nurses {
                    let s_u = match self.get_assigned_shift(genome, u, d) {
                        Some(s) => s,
                        None => continue,
                    };

                    let prev_s_u = if d == 0 {
                        let hist = self.context.history.nurse_history.iter().find(|h| h.nurse == self.context.scenario.nurses[u].id);
                        hist.and_then(|h| {
                            if h.number_of_consecutive_days_off > 0 || h.last_assigned_shift_type.is_empty() || h.last_assigned_shift_type == "None" {
                                None
                            } else {
                                self.context.shift_types.iter().position(|s| s == &h.last_assigned_shift_type)
                            }
                        })
                    } else {
                        self.get_assigned_shift(genome, u, d - 1)
                    };

                    if let Some(target_shift) = prev_s_u {
                        if s_u != target_shift {
                            for v in 0..num_nurses {
                                if v == u {
                                    continue;
                                }
                                if self.get_assigned_shift(genome, v, d) != Some(target_shift) {
                                    continue;
                                }

                                if !self.context.inverted.nurse_can_work[u][d][target_shift]
                                    || !self.context.inverted.nurse_can_work[v][d][s_u]
                                {
                                    continue;
                                }

                                let mut cand = genome.clone();
                                self.set_assigned_shift(&mut cand, u, d, Some(target_shift));
                                self.set_assigned_shift(&mut cand, v, d, Some(s_u));

                                if !self.nurse_has_forbidden_succession(&cand, u)
                                    && !self.nurse_has_forbidden_succession(&cand, v)
                                {
                                    let (is_feas, new_s8) = self.eval_days_coverage(&cand, &[d]);
                                    if is_feas {
                                        let (_, old_s8) = self.eval_days_coverage(genome, &[d]);
                                        let old_p = self.eval_nurse_soft_penalty(genome, u) + self.eval_nurse_soft_penalty(genome, v);
                                        let new_p = self.eval_nurse_soft_penalty(&cand, u) + self.eval_nurse_soft_penalty(&cand, v);
                                        let gain = (old_p - new_p) + (old_s8 - new_s8);

                                        if gain > 0 {
                                            *genome = cand;
                                            improved = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Passive domain adapter for Coralys Simulated Annealing Decision Engine.
/// Provides state evaluation and candidate neighborhood move proposals.
/// ALL decisions (acceptance gates, cooling schedules, budget termination)
/// are performed strictly within Coralys (`coralys_moga::annealing::SimulatedAnnealingEngine`).
pub struct InrcAnnealingDomain<'a> {
    pub optimizer: &'a InrcOptimizer,
    pub eligible_groups: Vec<Vec<usize>>,
    pub contract_skill_groups: Vec<Vec<usize>>,
    pub nurse_can_work: Vec<Vec<Vec<bool>>>,
    pub nurse_target_bands: Vec<(usize, usize)>,
    pub pacing: super::pacing::HorizonPacing,
    pub optimal_flow: super::optimal_flow::DailyOptimalFlow,
    pub ruin_recreate: super::ruin_recreate::ClusterRuinRecreate,
    pub contract_clusters: HashMap<String, Vec<usize>>,
    pub lookahead: super::lookahead::SundayLookahead,
}

impl<'a> InrcAnnealingDomain<'a> {
    pub fn new(optimizer: &'a InrcOptimizer) -> Self {
        let num_nurses = optimizer.context.num_nurses;
        let num_days = optimizer.context.num_days;
        let num_shifts = optimizer.context.shift_types.len();

        let mut skill_groups: HashMap<Vec<String>, Vec<usize>> = HashMap::new();
        for (n, nurse) in optimizer.context.scenario.nurses.iter().enumerate() {
            let mut s = nurse.skills.clone();
            s.sort();
            skill_groups.entry(s).or_default().push(n);
        }
        let eligible_groups: Vec<Vec<usize>> = skill_groups
            .into_values()
            .filter(|g| g.len() >= 2)
            .collect();

        let mut contract_skill_map: HashMap<(String, u8), Vec<usize>> = HashMap::new();
        for n in 0..num_nurses {
            let contract_id = optimizer.context.scenario.nurses[n].contract.clone();
            let mask = optimizer.context.nurse_skill_masks[n];
            contract_skill_map.entry((contract_id, mask)).or_default().push(n);
        }
        let contract_skill_groups: Vec<Vec<usize>> = contract_skill_map
            .into_values()
            .filter(|g| g.len() >= 2)
            .collect();

        let days_map = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];
        let mut shift_demanded_skills: Vec<Vec<Vec<String>>> = vec![vec![Vec::new(); num_shifts]; num_days];
        for d in 0..num_days {
            let day_name = days_map[d];
            for (s, st_name) in optimizer.context.shift_types.iter().enumerate() {
                for req in &optimizer.context.week_data.requirements {
                    if &req.shift_type == st_name {
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

        let mut nurse_can_work: Vec<Vec<Vec<bool>>> = vec![vec![vec![false; num_shifts]; num_days]; num_nurses];
        for n in 0..num_nurses {
            let nurse_skills = &optimizer.context.scenario.nurses[n].skills;
            for d in 0..num_days {
                for s in 0..num_shifts {
                    let demanded = &shift_demanded_skills[d][s];
                    nurse_can_work[n][d][s] = demanded.is_empty() || demanded.iter().any(|sk| nurse_skills.contains(sk));
                }
            }
        }

        let nurse_target_bands: Vec<(usize, usize)> = (0..num_nurses)
            .map(|n| {
                let contract = optimizer
                    .context
                    .scenario
                    .contracts
                    .iter()
                    .find(|c| c.id == optimizer.context.scenario.nurses[n].contract)
                    .unwrap();
                let min_a = contract.min_assignments / optimizer.context.scenario.number_of_weeks;
                let max_a = (contract.max_assignments as f64
                    / optimizer.context.scenario.number_of_weeks as f64)
                    .ceil() as usize;
                (min_a, max_a)
            })
            .collect();

        let pacing = super::pacing::HorizonPacing::compute(
            &optimizer.context.scenario,
            &optimizer.context.week_data,
            &optimizer.context.history,
        );
        let optimal_flow = super::optimal_flow::DailyOptimalFlow::new();
        let ruin_recreate = super::ruin_recreate::ClusterRuinRecreate::new();
        let contract_clusters = super::ruin_recreate::ClusterRuinRecreate::find_contract_clusters(optimizer);
        let lookahead = super::lookahead::SundayLookahead::new();

        Self {
            optimizer,
            eligible_groups,
            contract_skill_groups,
            nurse_can_work,
            nurse_target_bands,
            pacing,
            optimal_flow,
            ruin_recreate,
            contract_clusters,
            lookahead,
        }
    }
}

impl<'a> AnnealingDomain<InrcGenome> for InrcAnnealingDomain<'a> {
    fn initial_cost(&self, genome: &InrcGenome) -> f64 {
        let num_nurses = self.optimizer.context.num_nurses;
        let soft: i32 = (0..num_nurses)
            .map(|n| self.optimizer.eval_nurse_soft_penalty(genome, n))
            .sum();
        let num_days = self.optimizer.context.num_days;
        let days: Vec<usize> = (0..num_days).collect();
        let (_, s8) = self.optimizer.eval_days_coverage(genome, &days);
        let lookahead_pen = self.lookahead.eval_total_terminal_penalty(self.optimizer, genome);
        (soft + s8 + lookahead_pen) as f64
    }

    fn propose_move(
        &self,
        current: &InrcGenome,
        rng_core: &mut dyn rand::RngCore,
    ) -> Option<(InrcGenome, f64)> {
        use rand::Rng;
        let num_nurses = self.optimizer.context.num_nurses;
        let num_days = self.optimizer.context.num_days;
        let num_shifts = self.optimizer.context.shift_types.len();

        let op_choice = rng_core.gen_range(0..100);

        if op_choice < 15 {
            // Operator 9: Pairwise Full-Week Rotation Kempe Swap (Zero Coverage Disruption)
            if !self.contract_skill_groups.is_empty() {
                let grp_idx = rng_core.gen_range(0..self.contract_skill_groups.len());
                let grp = &self.contract_skill_groups[grp_idx];
                let i1 = rng_core.gen_range(0..grp.len());
                let mut i2 = rng_core.gen_range(0..grp.len());
                if i1 == i2 {
                    i2 = (i1 + 1) % grp.len();
                }
                let (u, v) = (grp[i1], grp[i2]);

                let old_p_u = self.optimizer.eval_nurse_soft_penalty(current, u);
                let old_p_v = self.optimizer.eval_nurse_soft_penalty(current, v);

                let mut cand = current.clone();
                for d in 0..num_days {
                    let s_u = self.optimizer.get_assigned_shift(current, u, d);
                    let s_v = self.optimizer.get_assigned_shift(current, v, d);
                    self.optimizer.set_assigned_shift(&mut cand, u, d, s_v);
                    self.optimizer.set_assigned_shift(&mut cand, v, d, s_u);
                }

                if !self.optimizer.nurse_has_forbidden_succession(&cand, u)
                    && !self.optimizer.nurse_has_forbidden_succession(&cand, v)
                {
                    let new_p_u = self.optimizer.eval_nurse_soft_penalty(&cand, u);
                    let new_p_v = self.optimizer.eval_nurse_soft_penalty(&cand, v);
                    let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v);
                    return Some((cand, delta as f64));
                }
            }
        } else if op_choice < 30 {
            // Operator 10: Zero-Defect Rotation Injection / LNS
            let n = rng_core.gen_range(0..num_nurses);
            let candidates = &self.optimizer.context.candidate_rotations[n];

            if !candidates.is_empty() {
                let pick = rng_core.gen_range(0..candidates.len().min(15));
                let (cand_rot, _) = candidates[pick];

                let mut cand = current.clone();
                let mut changed_days = Vec::new();
                for d in 0..num_days {
                    let cur_s = self.optimizer.get_assigned_shift(current, n, d);
                    let rot_s = cand_rot.get_shift(d);
                    if cur_s != rot_s {
                        changed_days.push(d);
                        self.optimizer.set_assigned_shift(&mut cand, n, d, rot_s);
                    }
                }

                if !changed_days.is_empty() && !self.optimizer.nurse_has_forbidden_succession(&cand, n) {
                    let (is_feas, new_s8) = self.optimizer.eval_days_coverage(&cand, &changed_days);
                    if is_feas {
                        let (_, old_s8) = self.optimizer.eval_days_coverage(current, &changed_days);
                        let old_p = self.optimizer.eval_nurse_soft_penalty(current, n);
                        let new_p = self.optimizer.eval_nurse_soft_penalty(&cand, n);
                        let delta = (new_p - old_p) + (new_s8 - old_s8);
                        return Some((cand, delta as f64));
                    }
                }
            }
        } else if op_choice < 42 {
            // Operator 5: Targeted Weekend Kempe Harmonizer (Directly eliminates S6 incomplete weekends)
            let mut sat_only = Vec::new();
            let mut sun_only = Vec::new();

            for n in 0..num_nurses {
                let sat = self.optimizer.get_assigned_shift(current, n, 5);
                let sun = self.optimizer.get_assigned_shift(current, n, 6);
                if sat.is_some() && sun.is_none() {
                    sat_only.push((n, sat.unwrap()));
                } else if sat.is_none() && sun.is_some() {
                    sun_only.push((n, sun.unwrap()));
                }
            }

            if !sat_only.is_empty() && !sun_only.is_empty() {
                let u_idx = rng_core.gen_range(0..sat_only.len());
                let v_idx = rng_core.gen_range(0..sun_only.len());
                let (u, s_sat) = sat_only[u_idx];
                let (v, s_sun) = sun_only[v_idx];

                let u_can_work_sun = self.nurse_can_work[u][6][s_sun];
                let u_succ_ok = !self.optimizer.context.inverted.forbidden_matrix[s_sat][s_sun];

                if u_can_work_sun && u_succ_ok {
                    let mut cand = current.clone();
                    let old_p_u = self.optimizer.eval_nurse_soft_penalty(current, u);
                    let old_p_v = self.optimizer.eval_nurse_soft_penalty(current, v);

                    self.optimizer.set_assigned_shift(&mut cand, v, 6, None);
                    self.optimizer.set_assigned_shift(&mut cand, u, 6, Some(s_sun));

                    if !self.optimizer.nurse_has_forbidden_succession(&cand, u)
                        && !self.optimizer.nurse_has_forbidden_succession(&cand, v)
                    {
                        let (new_feas, new_s8) = self.optimizer.eval_days_coverage(&cand, &[6]);
                        if new_feas {
                            let (_, old_s8) = self.optimizer.eval_days_coverage(current, &[6]);
                            let new_p_u = self.optimizer.eval_nurse_soft_penalty(&cand, u);
                            let new_p_v = self.optimizer.eval_nurse_soft_penalty(&cand, v);
                            let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v) + (new_s8 - old_s8);
                            return Some((cand, delta as f64));
                        }
                    }
                }
            }
        } else if op_choice < 54 {
            // Operator 6: Targeted Optimal Demand Staffing (Directly reduces S8 optimal coverage deficit)
            let d = rng_core.gen_range(0..num_days);
            let opt_slots = &self.optimizer.context.inverted.opt_slots_by_day[d];
            if !opt_slots.is_empty() {
                let sid = opt_slots[rng_core.gen_range(0..opt_slots.len())];
                let slot = &self.optimizer.context.inverted.slots[sid];
                let s_idx = slot.shift_idx;

                let candidates: Vec<usize> = slot
                    .eligible_nurses
                    .iter()
                    .copied()
                    .filter(|&n| {
                        if self.optimizer.get_assigned_shift(current, n, d).is_some() {
                            return false;
                        }
                        if d == 0 {
                            !self.optimizer.context.inverted.history_forbidden[n][s_idx]
                        } else {
                            match self.optimizer.get_assigned_shift(current, n, d - 1) {
                                Some(prev_s) => !self.optimizer.context.inverted.forbidden_matrix[prev_s][s_idx],
                                None => true,
                            }
                        }
                    })
                    .collect();

                if !candidates.is_empty() {
                    let u = candidates[rng_core.gen_range(0..candidates.len())];
                    let mut cand = current.clone();
                    let (_, old_s8) = self.optimizer.eval_days_coverage(current, &[d]);
                    let old_p_u = self.optimizer.eval_nurse_soft_penalty(current, u);

                    self.optimizer.set_assigned_shift(&mut cand, u, d, Some(s_idx));

                    if !self.optimizer.nurse_has_forbidden_succession(&cand, u) {
                        let (new_feas, new_s8) = self.optimizer.eval_days_coverage(&cand, &[d]);
                        if new_feas {
                            let new_p_u = self.optimizer.eval_nurse_soft_penalty(&cand, u);
                            let delta = (new_p_u - old_p_u) + (new_s8 - old_s8);
                            return Some((cand, delta as f64));
                        }
                    }
                }
            }
        } else if op_choice < 66 {
            // Operator 7: Daily Min-Cost Flow Augmentation (Exact bipartite S8 <-> S1 co-optimization)
            let d = rng_core.gen_range(0..num_days);
            let mut cand = current.clone();
            let gain = self.optimal_flow.optimize_day(self.optimizer, &mut cand, d, &self.pacing);
            if gain > 0 {
                return Some((cand, -gain as f64));
            }
        } else if op_choice < 76 {
            // Operator 8: Cluster Ruin-and-Recreate LNS (Multi-nurse escape from local minima)
            if let Some((cand, delta)) = self.ruin_recreate.step(self.optimizer, current, rng_core, &self.contract_clusters) {
                return Some((cand, delta));
            }
        }

        if op_choice < 88 {
            // Operator 1: 3-Day Contiguous Streak-Repair / Stub Ejection & Consolidation (S2/S4)
            let n = rng_core.gen_range(0..num_nurses);
            let mut stub_d = None;
            for d in 0..num_days {
                if self.optimizer.get_assigned_shift(current, n, d).is_some() {
                    let prev = d > 0 && self.optimizer.get_assigned_shift(current, n, d - 1).is_some();
                    let next = d + 1 < num_days && self.optimizer.get_assigned_shift(current, n, d + 1).is_some();
                    if !prev && !next {
                        stub_d = Some(d);
                        break;
                    }
                }
            }

            if let Some(d) = stub_d {
                let s_n = self.optimizer.get_assigned_shift(current, n, d);
                let mut donor = None;
                for _ in 0..15 {
                    let u = rng_core.gen_range(0..num_nurses);
                    if u != n && self.optimizer.get_assigned_shift(current, u, d).is_none() {
                        let u_adj_works = (d > 0 && self.optimizer.get_assigned_shift(current, u, d - 1).is_some())
                            || (d + 1 < num_days && self.optimizer.get_assigned_shift(current, u, d + 1).is_some());
                        if u_adj_works {
                            if let Some(s) = s_n {
                                if self.nurse_can_work[u][d][s] {
                                    donor = Some(u);
                                    break;
                                }
                            }
                        }
                    }
                }

                if let Some(u) = donor {
                    let mut cand = current.clone();
                    let (_, old_s8) = self.optimizer.eval_days_coverage(current, &[d]);
                    let old_p_n = self.optimizer.eval_nurse_soft_penalty(current, n);
                    let old_p_u = self.optimizer.eval_nurse_soft_penalty(current, u);

                    self.optimizer.set_assigned_shift(&mut cand, n, d, None);
                    self.optimizer.set_assigned_shift(&mut cand, u, d, s_n);

                    let valid = !self.optimizer.nurse_has_forbidden_succession(&cand, n)
                        && !self.optimizer.nurse_has_forbidden_succession(&cand, u);
                    let (new_feas, new_s8) = if valid {
                        self.optimizer.eval_days_coverage(&cand, &[d])
                    } else {
                        (false, 0)
                    };

                    if new_feas {
                        let new_p_n = self.optimizer.eval_nurse_soft_penalty(&cand, n);
                        let new_p_u = self.optimizer.eval_nurse_soft_penalty(&cand, u);
                        let delta = (new_p_n + new_p_u) - (old_p_n + old_p_u) + (new_s8 - old_s8);
                        return Some((cand, delta as f64));
                    }
                }
            }
            return None;
        }

        if op_choice < 88 {
            // Operator 2: Multi-Day Kempe-Chain Workload Transfer (S1)
            let mut over_nurses = Vec::new();
            let mut under_nurses = Vec::new();

            for n in 0..num_nurses {
                let mut count = 0;
                for d in 0..num_days {
                    if self.optimizer.get_assigned_shift(current, n, d).is_some() {
                        count += 1;
                    }
                }
                let (min_a, max_a) = self.nurse_target_bands[n];
                if count > max_a {
                    over_nurses.push(n);
                } else if count < min_a {
                    under_nurses.push(n);
                }
            }

            let (u, v) = if !over_nurses.is_empty() && !under_nurses.is_empty() {
                let u_idx = rng_core.gen_range(0..over_nurses.len());
                let v_idx = rng_core.gen_range(0..under_nurses.len());
                (over_nurses[u_idx], under_nurses[v_idx])
            } else {
                let i1 = rng_core.gen_range(0..num_nurses);
                let mut i2 = rng_core.gen_range(0..num_nurses);
                if i1 == i2 { i2 = (i1 + 1) % num_nurses; }
                (i1, i2)
            };

            if u != v {
                let block_len = if rng_core.gen_bool(0.65) { 2 } else { 3 };
                let max_start = if num_days >= block_len { num_days - block_len } else { 0 };
                let start_d = if max_start > 0 { rng_core.gen_range(0..=max_start) } else { 0 };
                let days: Vec<usize> = (start_d..start_d + block_len).collect();

                let mut u_works_all = true;
                let mut v_off_all = true;
                let mut v_can_work_all = true;

                for &d in &days {
                    let s_u = self.optimizer.get_assigned_shift(current, u, d);
                    let s_v = self.optimizer.get_assigned_shift(current, v, d);
                    if s_u.is_none() { u_works_all = false; break; }
                    if s_v.is_some() { v_off_all = false; break; }
                    if let Some(s) = s_u {
                        if !self.nurse_can_work[v][d][s] {
                            v_can_work_all = false;
                            break;
                        }
                    }
                }

                if u_works_all && v_off_all && v_can_work_all {
                    let mut cand = current.clone();
                    let (_, old_s8) = self.optimizer.eval_days_coverage(current, &days);
                    let old_p_u = self.optimizer.eval_nurse_soft_penalty(current, u);
                    let old_p_v = self.optimizer.eval_nurse_soft_penalty(current, v);

                    for &d in &days {
                        let s = self.optimizer.get_assigned_shift(current, u, d);
                        self.optimizer.set_assigned_shift(&mut cand, u, d, None);
                        self.optimizer.set_assigned_shift(&mut cand, v, d, s);
                    }

                    let valid = !self.optimizer.nurse_has_forbidden_succession(&cand, u)
                        && !self.optimizer.nurse_has_forbidden_succession(&cand, v);
                    let (new_feas, new_s8) = if valid {
                        self.optimizer.eval_days_coverage(&cand, &days)
                    } else {
                        (false, 0)
                    };

                    if new_feas {
                        let new_p_u = self.optimizer.eval_nurse_soft_penalty(&cand, u);
                        let new_p_v = self.optimizer.eval_nurse_soft_penalty(&cand, v);
                        let delta = (new_p_u + new_p_v) - (old_p_u + old_p_v) + (new_s8 - old_s8);
                        return Some((cand, delta as f64));
                    }
                }
            }
            return None;
        }

        if op_choice < 94 {
            // Operator 3: Single-nurse shift block reassignment (rebalance S8 optimal coverage & S1 assignment quotas)
            let n = rng_core.gen_range(0..num_nurses);
            let block_len = if rng_core.gen_bool(0.60) { 2 } else { 1 };
            let start_d = if block_len > 1 {
                rng_core.gen_range(0..num_days - 1)
            } else {
                rng_core.gen_range(0..num_days)
            };
            let days: Vec<usize> = (start_d..start_d + block_len).collect();

            let new_shift = if rng_core.gen_bool(0.35) {
                None
            } else {
                Some(rng_core.gen_range(0..num_shifts))
            };

            let mut can_work_all = true;
            if let Some(s_idx) = new_shift {
                for &d in &days {
                    if !self.nurse_can_work[n][d][s_idx] {
                        can_work_all = false;
                        break;
                    }
                }
            }
            if !can_work_all {
                return None;
            }

            let mut old_shifts = Vec::with_capacity(days.len());
            for &d in &days {
                old_shifts.push(self.optimizer.get_assigned_shift(current, n, d));
            }
            if old_shifts.iter().all(|&s| s == new_shift) {
                return None;
            }

            let mut cand = current.clone();
            let (_, old_s8) = self.optimizer.eval_days_coverage(current, &days);
            let old_p_n = self.optimizer.eval_nurse_soft_penalty(current, n);

            for &d in &days {
                self.optimizer.set_assigned_shift(&mut cand, n, d, new_shift);
            }

            if self.optimizer.nurse_has_forbidden_succession(&cand, n) {
                return None;
            }

            let (new_feasible, new_s8) = self.optimizer.eval_days_coverage(&cand, &days);
            if !new_feasible {
                return None;
            }

            let new_pen = self.optimizer.eval_nurse_soft_penalty(&cand, n);
            let delta = (new_pen - old_p_n) + (new_s8 - old_s8);
            return Some((cand, delta as f64));
        }

        // Operator 4: Intra-group or cross-skill day swap
        let is_cross_skill = rng_core.gen_bool(0.20) || self.eligible_groups.is_empty();
        let (n1, n2) = if !is_cross_skill && !self.eligible_groups.is_empty() {
            let group_idx = rng_core.gen_range(0..self.eligible_groups.len());
            let group = &self.eligible_groups[group_idx];
            let i1 = rng_core.gen_range(0..group.len());
            let mut i2 = rng_core.gen_range(0..group.len());
            if i1 == i2 {
                i2 = (i1 + 1) % group.len();
            }
            (group[i1], group[i2])
        } else {
            let i1 = rng_core.gen_range(0..num_nurses);
            let mut i2 = rng_core.gen_range(0..num_nurses);
            if i1 == i2 {
                i2 = (i1 + 1) % num_nurses;
            }
            (i1, i2)
        };

        let move_type = rng_core.gen_range(0..4);
        let days_to_swap: Vec<usize> = match move_type {
            0 => vec![rng_core.gen_range(0..num_days)],
            1 => vec![5, 6],
            2 => {
                let d = rng_core.gen_range(0..num_days - 1);
                vec![d, d + 1]
            }
            _ => {
                let d = rng_core.gen_range(0..num_days - 2);
                vec![d, d + 1, d + 2]
            }
        };

        // Fast HC2 qualification filter for cross-skill moves
        if is_cross_skill {
            let mut qualified = true;
            for &d in &days_to_swap {
                let s1 = self.optimizer.get_assigned_shift(current, n1, d);
                let s2 = self.optimizer.get_assigned_shift(current, n2, d);
                if let Some(s1_idx) = s1 {
                    if !self.nurse_can_work[n2][d][s1_idx] {
                        qualified = false;
                        break;
                    }
                }
                if let Some(s2_idx) = s2 {
                    if !self.nurse_can_work[n1][d][s2_idx] {
                        qualified = false;
                        break;
                    }
                }
            }
            if !qualified {
                return None;
            }
        }

        let mut cand = current.clone();
        let (_, old_s8) = if is_cross_skill {
            self.optimizer.eval_days_coverage(current, &days_to_swap)
        } else {
            (true, 0)
        };
        let old_p_1 = self.optimizer.eval_nurse_soft_penalty(current, n1);
        let old_p_2 = self.optimizer.eval_nurse_soft_penalty(current, n2);

        for &d in &days_to_swap {
            let s1 = self.optimizer.get_assigned_shift(current, n1, d);
            let s2 = self.optimizer.get_assigned_shift(current, n2, d);
            self.optimizer.set_assigned_shift(&mut cand, n1, d, s2);
            self.optimizer.set_assigned_shift(&mut cand, n2, d, s1);
        }

        let valid_succession = !self.optimizer.nurse_has_forbidden_succession(&cand, n1)
            && !self.optimizer.nurse_has_forbidden_succession(&cand, n2);

        if !valid_succession {
            return None;
        }

        let (new_feasible, new_s8) = if is_cross_skill {
            self.optimizer.eval_days_coverage(&cand, &days_to_swap)
        } else {
            (true, 0)
        };

        if !new_feasible {
            return None;
        }

        let new_pen_1 = self.optimizer.eval_nurse_soft_penalty(&cand, n1);
        let new_pen_2 = self.optimizer.eval_nurse_soft_penalty(&cand, n2);
        let delta = (new_pen_1 + new_pen_2) - (old_p_1 + old_p_2) + (new_s8 - old_s8);

        Some((cand, delta as f64))
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

                let mut min_demands = Vec::new();
                let mut opt_demands = Vec::new();
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
                        if req_level.optimal > req_level.minimum {
                            opt_demands.push((req.skill.clone(), req_level.optimal - req_level.minimum));
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
                    let has_valid_skill = min_demands
                        .iter()
                        .any(|(skill, _)| nurse.skills.contains(skill))
                        || opt_demands
                            .iter()
                            .any(|(skill, _)| nurse.skills.contains(skill));
                    if !has_valid_skill && (!min_demands.is_empty() || !opt_demands.is_empty()) {
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

                // Run AssignmentSolver via Two-Stage Matching (Stage 1: Minimum, Stage 2: Optimal)
                let workers: Vec<(usize, Vec<String>)> = available_nurses
                    .iter()
                    .map(|&n| (n, self.context.scenario.nurses[n].skills.clone()))
                    .collect();

                let min_matching = BipartiteMatchingSolver.assign(&workers, &min_demands);
                let mut min_fulfilled_map = HashMap::new();
                let mut matched_nurse_indices = std::collections::HashSet::new();
                for (nurse_id, skill) in &min_matching.assignments {
                    *min_fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
                    matched_nurse_indices.insert(*nurse_id);
                }

                let unassigned_workers: Vec<(usize, Vec<String>)> = workers
                    .iter()
                    .filter(|(n, _)| !matched_nurse_indices.contains(n))
                    .cloned()
                    .collect();

                let mut opt_fulfilled_map = HashMap::new();
                if !unassigned_workers.is_empty() && !opt_demands.is_empty() {
                    let opt_matching = BipartiteMatchingSolver.assign(&unassigned_workers, &opt_demands);
                    for (_, skill) in &opt_matching.assignments {
                        *opt_fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
                    }
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

                        let min_fulfilled = *min_fulfilled_map.get(&req.skill).unwrap_or(&0);
                        if min_fulfilled < req_level.minimum {
                            let missing = req_level.minimum - min_fulfilled;
                            let cost = missing * weights.hard_constraint_violation as usize;
                            hc_coverage += cost;
                            hard_violations.push(Violation {
                                constraint_id: "HC1_MinimumCoverage".to_string(),
                                severity: "Hard".to_string(),
                                value: Some(missing as f64),
                                expected: req_level.minimum.to_string(),
                                actual: min_fulfilled.to_string(),
                                description: format!(
                                    "Shift {} skill {} on {} lacks {} nurses (minimum)",
                                    shift_name, req.skill, day_name, missing
                                ),
                                penalty: cost as i32,
                            });
                        }

                        let opt_fulfilled = *opt_fulfilled_map.get(&req.skill).unwrap_or(&0);
                        let total_fulfilled = min_fulfilled + opt_fulfilled;
                        if total_fulfilled < req_level.optimal {
                            let missing = req_level.optimal - total_fulfilled;
                            let cost = missing as i32 * weights.optimal_coverage;
                            optimal_coverage_penalty += cost;
                            soft_violations.push(Violation {
                                constraint_id: "S8_OptimalCoverage".to_string(),
                                severity: "Soft".to_string(),
                                value: Some(missing as f64),
                                expected: req_level.optimal.to_string(),
                                actual: total_fulfilled.to_string(),
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
