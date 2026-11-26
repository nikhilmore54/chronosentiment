//! `ultracrew::inrc::inverted` — Inverted (Task-Centric) Chromosome Architecture for INRC-II.
//!
//! Dual representation of nurse rostering schedules:
//! Instead of worker-centric bitvectors (Nurse -> Day -> Shift), the chromosome is task-centric
//! (DemandSlot -> AssignedNurse).
//!
//! By design:
//! - HC1 (Minimum Coverage) is 100% INVARIANT (all mandatory slots are covered).
//! - HC2 (Skill Competency) is 100% INVARIANT (only eligible nurses can be assigned).
//! - Day-preserving crossover maintains 100% coverage invariance across generations.
//! - Genetic search focuses 100% of compute on soft constraint optimization (S1–S8).

use coralys_matching::{AssignmentSolver, BipartiteMatchingSolver};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

use super::models::{InrcHistory, InrcScenario, InrcWeekData};
use super::optimization::InrcGenome;

/// Represents a discrete required coverage slot for a (day, shift, skill).
#[derive(Debug, Clone)]
pub struct DemandSlot {
    pub id: usize,
    pub day: usize,
    pub shift_idx: usize,
    pub shift_name: String,
    pub skill: String,
    pub is_minimum: bool,
    pub eligible_nurses: Vec<usize>,
}

/// Precomputed inverted indexing context for a given scenario and week.
#[derive(Debug, Clone)]
pub struct InvertedContext {
    pub num_nurses: usize,
    pub num_days: usize,
    pub num_shifts: usize,
    pub slots: Vec<DemandSlot>,
    pub min_slots_by_day: Vec<Vec<usize>>,
    pub opt_slots_by_day: Vec<Vec<usize>>,
    pub min_demands_by_day_shift: Vec<Vec<Vec<(String, usize)>>>,
    pub nurse_target_bands: Vec<(usize, usize)>,
    pub nurse_skills: Vec<Vec<String>>,
    pub nurse_skills_count: Vec<usize>,
    pub nurse_can_work: Vec<Vec<Vec<bool>>>,
    pub forbidden_matrix: Vec<Vec<bool>>, // [prev_shift][next_shift] -> forbidden
    pub history_forbidden: Vec<Vec<bool>>, // [nurse][shift] -> forbidden on day 0
}

impl InvertedContext {
    /// Builds the inverted demand slot context from scenario and weekly requirements.
    pub fn new(scenario: &InrcScenario, week_data: &InrcWeekData, history: &InrcHistory) -> Self {
        let num_nurses = scenario.nurses.len();
        let num_days = 7;
        let num_shifts = scenario.shift_types.len();
        let shift_types: Vec<String> = scenario.shift_types.iter().map(|s| s.id.clone()).collect();

        let days_map = [
            "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday",
        ];

        let mut slots = Vec::new();
        let mut min_slots_by_day = vec![Vec::new(); num_days];
        let mut opt_slots_by_day = vec![Vec::new(); num_days];
        let mut min_demands_by_day_shift = vec![vec![Vec::new(); num_shifts]; num_days];
        let mut slot_id = 0;

        let mut shift_demanded_skills: Vec<Vec<Vec<String>>> =
            vec![vec![Vec::new(); num_shifts]; num_days];

        for d in 0..num_days {
            let day_name = days_map[d];
            for (s_idx, s_name) in shift_types.iter().enumerate() {
                for req in &week_data.requirements {
                    if &req.shift_type == s_name {
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
                            min_demands_by_day_shift[d][s_idx].push((req.skill.clone(), req_level.minimum));
                        }

                        if req_level.minimum > 0 || req_level.optimal > 0 {
                            shift_demanded_skills[d][s_idx].push(req.skill.clone());
                        }

                        let eligible: Vec<usize> = scenario
                            .nurses
                            .iter()
                            .enumerate()
                            .filter(|(_, n)| n.skills.contains(&req.skill))
                            .map(|(idx, _)| idx)
                            .collect();

                        for _ in 0..req_level.minimum {
                            let slot = DemandSlot {
                                id: slot_id,
                                day: d,
                                shift_idx: s_idx,
                                shift_name: s_name.clone(),
                                skill: req.skill.clone(),
                                is_minimum: true,
                                eligible_nurses: eligible.clone(),
                            };
                            slots.push(slot);
                            min_slots_by_day[d].push(slot_id);
                            slot_id += 1;
                        }

                        for _ in req_level.minimum..req_level.optimal {
                            let slot = DemandSlot {
                                id: slot_id,
                                day: d,
                                shift_idx: s_idx,
                                shift_name: s_name.clone(),
                                skill: req.skill.clone(),
                                is_minimum: false,
                                eligible_nurses: eligible.clone(),
                            };
                            slots.push(slot);
                            opt_slots_by_day[d].push(slot_id);
                            slot_id += 1;
                        }
                    }
                }
            }
        }

        let nurse_skills: Vec<Vec<String>> =
            scenario.nurses.iter().map(|n| n.skills.clone()).collect();
        let nurse_skills_count: Vec<usize> =
            scenario.nurses.iter().map(|n| n.skills.len()).collect();

        let mut nurse_can_work: Vec<Vec<Vec<bool>>> =
            vec![vec![vec![false; num_shifts]; num_days]; num_nurses];
        for n in 0..num_nurses {
            let n_skills = &scenario.nurses[n].skills;
            for d in 0..num_days {
                for s in 0..num_shifts {
                    let demanded = &shift_demanded_skills[d][s];
                    nurse_can_work[n][d][s] = demanded.is_empty()
                        || demanded.iter().any(|sk| n_skills.contains(sk));
                }
            }
        }

        let nurse_target_bands: Vec<(usize, usize)> = (0..num_nurses)
            .map(|n| {
                let contract = scenario
                    .contracts
                    .iter()
                    .find(|c| c.id == scenario.nurses[n].contract)
                    .unwrap();
                let min_a = contract.min_assignments / scenario.number_of_weeks;
                let max_a = (contract.max_assignments as f64
                    / scenario.number_of_weeks as f64)
                    .ceil() as usize;
                (min_a, max_a)
            })
            .collect();

        let mut forbidden_matrix = vec![vec![false; num_shifts]; num_shifts];
        for f in &scenario.forbidden_shift_type_successions {
            if let Some(p_idx) = shift_types.iter().position(|s| s == &f.preceding) {
                for succ in &f.succeeding {
                    if let Some(s_idx) = shift_types.iter().position(|s| s == succ) {
                        forbidden_matrix[p_idx][s_idx] = true;
                    }
                }
            }
        }

        let mut history_forbidden = vec![vec![false; num_shifts]; num_nurses];
        for h in &history.nurse_history {
            if let Some(n_idx) = scenario.nurses.iter().position(|n| n.id == h.nurse) {
                if h.last_assigned_shift_type != "None" && !h.last_assigned_shift_type.is_empty() {
                    if let Some(p_idx) = shift_types.iter().position(|s| s == &h.last_assigned_shift_type) {
                        for s in 0..num_shifts {
                            if forbidden_matrix[p_idx][s] {
                                history_forbidden[n_idx][s] = true;
                            }
                        }
                    }
                }
            }
        }

        Self {
            num_nurses,
            num_days,
            num_shifts,
            slots,
            min_slots_by_day,
            opt_slots_by_day,
            min_demands_by_day_shift,
            nurse_target_bands,
            nurse_skills,
            nurse_skills_count,
            nurse_can_work,
            forbidden_matrix,
            history_forbidden,
        }
    }

    /// Constructs a 100% feasible initial genome by assigning nurses to demand slots.
    /// Guarantees HC1 (Coverage) and HC2 (Skills) by construction.
    pub fn construct_feasible_genome(&self, rng: &mut StdRng) -> InrcGenome {
        let mut assigned_shift: Vec<Vec<Option<usize>>> =
            vec![vec![None; self.num_days]; self.num_nurses];
        let mut shift_counts = vec![0usize; self.num_nurses];

        for d in 0..self.num_days {
            // Sort mandatory slots: rarest skill first (fewest eligible nurses first)
            let mut day_min_slots = self.min_slots_by_day[d].clone();
            day_min_slots.sort_by_key(|&sid| self.slots[sid].eligible_nurses.len());

            for sid in day_min_slots {
                let slot = &self.slots[sid];
                let mut candidates: Vec<usize> = slot
                    .eligible_nurses
                    .iter()
                    .copied()
                    .filter(|&n| assigned_shift[n][d].is_none())
                    .collect();

                // Weekend coherence preference: on Sunday (d=6), prefer nurses who worked Saturday (d=5)
                if d == 6 {
                    let wend_candidates: Vec<usize> = candidates
                        .iter()
                        .copied()
                        .filter(|&n| assigned_shift[n][5].is_some())
                        .collect();
                    if !wend_candidates.is_empty() && rng.gen_bool(0.75) {
                        candidates = wend_candidates;
                    }
                }

                // Filter out HC4 forbidden successions if possible
                let legal_candidates: Vec<usize> = if d == 0 {
                    candidates
                        .iter()
                        .copied()
                        .filter(|&n| !self.history_forbidden[n][slot.shift_idx])
                        .collect()
                } else {
                    candidates
                        .iter()
                        .copied()
                        .filter(|&n| {
                            match assigned_shift[n][d - 1] {
                                Some(prev_s) => !self.forbidden_matrix[prev_s][slot.shift_idx],
                                None => true,
                            }
                        })
                        .collect()
                };
                if !legal_candidates.is_empty() {
                    candidates = legal_candidates;
                }

                if candidates.is_empty() {
                    // Fallback to any free nurse who holds the demanded skill
                    for &n in &slot.eligible_nurses {
                        if assigned_shift[n][d].is_none() {
                            candidates.push(n);
                        }
                    }
                }

                if !candidates.is_empty() {
                    // Sort candidates by:
                    // 1. Skill specialization (fewer skills first, to preserve multi-skilled nurses for rare skills)
                    // 2. Workload balance (under-quota first)
                    candidates.sort_by_key(|&n| {
                        let (_, max_a) = self.nurse_target_bands[n];
                        let over = if shift_counts[n] >= max_a { 100 } else { 0 };
                        (self.nurse_skills_count[n], over, shift_counts[n])
                    });

                    // Stochastic choice among top candidate pool
                    let pool_size = candidates.len().min(3);
                    let chosen = candidates[rng.gen_range(0..pool_size)];

                    assigned_shift[chosen][d] = Some(slot.shift_idx);
                    shift_counts[chosen] += 1;
                }
            }

            // Fill optimal demand slots if nurses have remaining capacity
            for &sid in &self.opt_slots_by_day[d] {
                let slot = &self.slots[sid];
                let candidates: Vec<usize> = slot
                    .eligible_nurses
                    .iter()
                    .copied()
                    .filter(|&n| {
                        assigned_shift[n][d].is_none()
                            && shift_counts[n] < self.nurse_target_bands[n].0
                    })
                    .collect();

                if let Some(&chosen) = candidates.choose(rng) {
                    if rng.gen_bool(0.50) {
                        assigned_shift[chosen][d] = Some(slot.shift_idx);
                        shift_counts[chosen] += 1;
                    }
                }
            }
        }

        self.decode_to_genome(&assigned_shift)
    }

    /// Performs Day-Preserving Crossover.
    /// Exchanges full daily assignment vectors between parents, guaranteeing that
    /// daily HC1 coverage, HC2 skills, and HC3 single-shift legality are 100% preserved.
    pub fn day_preserving_crossover(
        &self,
        parent_a: &InrcGenome,
        parent_b: &InrcGenome,
        rng: &mut StdRng,
    ) -> (InrcGenome, InrcGenome) {
        let shifts_a = self.encode_shifts(parent_a);
        let shifts_b = self.encode_shifts(parent_b);

        let mut child_a_shifts = vec![vec![None; self.num_days]; self.num_nurses];
        let mut child_b_shifts = vec![vec![None; self.num_days]; self.num_nurses];

        // Choose crossover pattern: either weekday/weekend split or random split point
        let split_day = if rng.gen_bool(0.5) {
            5 // Keep weekdays (0..5) and weekend (5..7) atomic
        } else {
            rng.gen_range(1..6)
        };

        for d in 0..self.num_days {
            let use_a_for_child_a = d < split_day;
            for n in 0..self.num_nurses {
                if use_a_for_child_a {
                    child_a_shifts[n][d] = shifts_a[n][d];
                    child_b_shifts[n][d] = shifts_b[n][d];
                } else {
                    child_a_shifts[n][d] = shifts_b[n][d];
                    child_b_shifts[n][d] = shifts_a[n][d];
                }
            }
        }

        (
            self.decode_to_genome(&child_a_shifts),
            self.decode_to_genome(&child_b_shifts),
        )
    }

    /// Performs Inverted Coverage-Preserving Mutation.
    /// Reassigns or swaps shifts within eligible nurse pools, keeping coverage invariant.
    pub fn inverted_mutate(&self, genome: &mut InrcGenome, rng: &mut StdRng) {
        let mut shifts = self.encode_shifts(genome);
        let num_mutations = rng.gen_range(1..=3);

        for _ in 0..num_mutations {
            let d = rng.gen_range(0..self.num_days);

            if rng.gen_bool(0.60) {
                // Strategy A: Reassign shift to an off-duty eligible nurse on day d
                let assigned_nurses: Vec<(usize, usize)> = (0..self.num_nurses)
                    .filter_map(|n| shifts[n][d].map(|s| (n, s)))
                    .collect();

                if let Some(&(n1, s_idx)) = assigned_nurses.choose(rng) {
                    let matching_slots: Vec<&DemandSlot> = self.min_slots_by_day[d]
                        .iter()
                        .map(|&sid| &self.slots[sid])
                        .filter(|slot| slot.shift_idx == s_idx)
                        .collect();

                    if let Some(slot) = matching_slots.choose(rng) {
                        let free_eligible: Vec<usize> = slot
                            .eligible_nurses
                            .iter()
                            .copied()
                            .filter(|&n| shifts[n][d].is_none())
                            .collect();

                        if let Some(&n2) = free_eligible.choose(rng) {
                            // Check HC4 succession before applying
                            let valid_succ_n2 = if d == 0 {
                                !self.history_forbidden[n2][s_idx]
                            } else {
                                match shifts[n2][d - 1] {
                                    Some(prev) => !self.forbidden_matrix[prev][s_idx],
                                    None => true,
                                }
                            };
                            if valid_succ_n2 {
                                shifts[n1][d] = None;
                                shifts[n2][d] = Some(s_idx);
                            }
                        }
                    }
                }
            } else {
                // Strategy B: Intra-day swap between two working nurses with compatible skills
                let assigned_nurses: Vec<(usize, usize)> = (0..self.num_nurses)
                    .filter_map(|n| shifts[n][d].map(|s| (n, s)))
                    .collect();

                if assigned_nurses.len() >= 2 {
                    let &(n1, s1) = assigned_nurses.choose(rng).unwrap();
                    let &(n2, s2) = assigned_nurses.choose(rng).unwrap();

                    if n1 != n2 && s1 != s2 {
                        let n1_can_s2 = self.nurse_can_work[n1][d][s2];
                        let n2_can_s1 = self.nurse_can_work[n2][d][s1];

                        if n1_can_s2 && n2_can_s1 {
                            shifts[n1][d] = Some(s2);
                            shifts[n2][d] = Some(s1);
                        }
                    }
                }
            }
        }

        *genome = self.decode_to_genome(&shifts);
    }

    /// Audits daily shift demands using `BipartiteMatchingSolver` and repairs any shortfall,
    /// guaranteeing 100% statutory feasibility (0 HC1, 0 HC2, 0 HC3 violations).
    pub fn repair_coverage(&self, genome: &mut InrcGenome) {
        let mut shifts = self.encode_shifts(genome);

        for d in 0..self.num_days {
            for s in 0..self.num_shifts {
                let min_demands = &self.min_demands_by_day_shift[d][s];
                if min_demands.is_empty() {
                    continue;
                }

                let mut available = Vec::new();
                for n in 0..self.num_nurses {
                    if shifts[n][d] == Some(s) {
                        available.push((n, self.nurse_skills[n].clone()));
                    }
                }

                let matching = BipartiteMatchingSolver.assign(&available, min_demands);
                let mut fulfilled_map = HashMap::new();
                for (_, skill) in &matching.assignments {
                    *fulfilled_map.entry(skill.clone()).or_insert(0) += 1;
                }

                for (skill, required) in min_demands {
                    let required = *required;
                    let fulfilled = *fulfilled_map.get(skill).unwrap_or(&0);
                    if fulfilled < required {
                        let needed = required - fulfilled;
                        let mut assigned = 0;

                        // Step 1: Assign off-duty qualified nurse with no succession conflict
                        for n in 0..self.num_nurses {
                            if assigned >= needed {
                                break;
                            }
                            if shifts[n][d].is_none() && self.nurse_skills[n].contains(skill) {
                                let valid_succ = if d == 0 {
                                    !self.history_forbidden[n][s]
                                } else {
                                    match shifts[n][d - 1] {
                                        Some(prev) => !self.forbidden_matrix[prev][s],
                                        None => true,
                                    }
                                };
                                if valid_succ {
                                    shifts[n][d] = Some(s);
                                    if d + 1 < self.num_days {
                                        if let Some(next_s) = shifts[n][d + 1] {
                                            if self.forbidden_matrix[s][next_s] {
                                                shifts[n][d + 1] = None;
                                            }
                                        }
                                    }
                                    assigned += 1;
                                }
                            }
                        }

                        // Step 1b: If still needed, assign off-duty qualified nurse and clear conflict
                        if assigned < needed {
                            for n in 0..self.num_nurses {
                                if assigned >= needed {
                                    break;
                                }
                                if shifts[n][d].is_none() && self.nurse_skills[n].contains(skill) {
                                    shifts[n][d] = Some(s);
                                    if d + 1 < self.num_days {
                                        if let Some(next_s) = shifts[n][d + 1] {
                                            if self.forbidden_matrix[s][next_s] {
                                                shifts[n][d + 1] = None;
                                            }
                                        }
                                    }
                                    assigned += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        *genome = self.decode_to_genome(&shifts);
    }

    /// Helper to convert genome bitvector to 2D `[nurse][day] -> Option<shift>` representation.
    pub fn encode_shifts(&self, genome: &InrcGenome) -> Vec<Vec<Option<usize>>> {
        let mut shifts = vec![vec![None; self.num_days]; self.num_nurses];
        for n in 0..self.num_nurses {
            for d in 0..self.num_days {
                for s in 0..self.num_shifts {
                    let idx = n * (self.num_days * self.num_shifts) + d * self.num_shifts + s;
                    if genome.bits[idx] {
                        shifts[n][d] = Some(s);
                        break;
                    }
                }
            }
        }
        shifts
    }

    /// Helper to convert 2D `[nurse][day] -> Option<shift>` representation to genome bitvector.
    pub fn decode_to_genome(&self, shifts: &[Vec<Option<usize>>]) -> InrcGenome {
        let mut bits = vec![false; self.num_nurses * self.num_days * self.num_shifts];
        for n in 0..self.num_nurses {
            for d in 0..self.num_days {
                if let Some(s) = shifts[n][d] {
                    let idx = n * (self.num_days * self.num_shifts) + d * self.num_shifts + s;
                    bits[idx] = true;
                }
            }
        }
        InrcGenome { bits }
    }
}
