//! `ultracrew::inrc::rotation_pool` — Zero-Defect Ergonomic Rotation Mining Engine.
//!
//! Precomputes the universe of all zero-defect 7-day shift patterns for each INRC contract.
//! Every rotation in the pool satisfies internal ergonomic constraints:
//! - HC4: Zero forbidden shift successions
//! - S6: Complete weekends (Saturday and Sunday identical working status if required)
//! - S2: No internal isolated working days (all internal streaks within min/max bounds)
//! - S3: No internal isolated days off (all internal rest streaks within min/max bounds)
//! - S4: No internal shift-type fragmentation (all shift streaks within min/max bounds)

use std::collections::HashMap;
use crate::inrc::models::{InrcContract, InrcNurseHistory, InrcScenario};

/// A 7-day rotation sequence.
/// `shifts[d] == 0` indicates Off duty.
/// `shifts[d] == s + 1` indicates shift index `s` (0-based) on day `d`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rotation7 {
    pub shifts: [u8; 7],
}

impl Rotation7 {
    pub const OFF: Rotation7 = Rotation7 { shifts: [0; 7] };

    #[inline(always)]
    pub fn get_shift(&self, day: usize) -> Option<usize> {
        let v = self.shifts[day];
        if v == 0 {
            None
        } else {
            Some((v - 1) as usize)
        }
    }

    #[inline(always)]
    pub fn is_working(&self, day: usize) -> bool {
        self.shifts[day] > 0
    }

    #[inline(always)]
    pub fn total_assignments(&self) -> usize {
        let mut count = 0;
        for &s in &self.shifts {
            if s > 0 {
                count += 1;
            }
        }
        count
    }

    #[inline(always)]
    pub fn works_weekend(&self) -> bool {
        self.shifts[5] > 0 || self.shifts[6] > 0
    }

    #[inline(always)]
    pub fn complete_weekend(&self) -> bool {
        (self.shifts[5] > 0) == (self.shifts[6] > 0)
    }

    #[inline(always)]
    pub fn from_shifts(shifts: &[Option<usize>; 7]) -> Self {
        let mut arr = [0u8; 7];
        for d in 0..7 {
            arr[d] = match shifts[d] {
                None => 0,
                Some(idx) => (idx + 1) as u8,
            };
        }
        Self { shifts: arr }
    }
}

/// Pool of precomputed zero-defect rotations per contract.
#[derive(Clone, Debug)]
pub struct RotationPool {
    /// Zero-defect rotations grouped by contract ID
    pub rotations_by_contract: HashMap<String, Vec<Rotation7>>,
    /// Shift type ID to index mapping
    pub shift_type_indices: HashMap<String, usize>,
    /// Forbidden succession matrix: `[prev][next]` is true if forbidden
    pub forbidden_matrix: Vec<Vec<bool>>,
}

impl RotationPool {
    /// Builds a rotation pool from the given INRC scenario.
    pub fn build(scenario: &InrcScenario) -> Self {
        let num_shifts = scenario.shift_types.len();
        let mut shift_type_indices = HashMap::new();
        for (idx, st) in scenario.shift_types.iter().enumerate() {
            shift_type_indices.insert(st.id.clone(), idx);
        }

        let mut forbidden_matrix = vec![vec![false; num_shifts]; num_shifts];
        for rule in &scenario.forbidden_shift_type_successions {
            if let Some(&prev_idx) = shift_type_indices.get(&rule.preceding) {
                for succ in &rule.succeeding {
                    if let Some(&succ_idx) = shift_type_indices.get(succ) {
                        forbidden_matrix[prev_idx][succ_idx] = true;
                    }
                }
            }
        }

        let mut rotations_by_contract = HashMap::new();
        for contract in &scenario.contracts {
            let valid = Self::generate_contract_rotations(
                contract,
                scenario,
                &forbidden_matrix,
                num_shifts,
            );
            rotations_by_contract.insert(contract.id.clone(), valid);
        }

        Self {
            rotations_by_contract,
            shift_type_indices,
            forbidden_matrix,
        }
    }

    /// Recursively enumerates all 7-day assignments and filters those satisfying zero internal defects.
    fn generate_contract_rotations(
        contract: &InrcContract,
        scenario: &InrcScenario,
        forbidden: &[Vec<bool>],
        num_shifts: usize,
    ) -> Vec<Rotation7> {
        let mut results = Vec::new();
        let mut current = [0u8; 7];

        Self::search_rotations(
            0,
            &mut current,
            contract,
            scenario,
            forbidden,
            num_shifts,
            &mut results,
        );

        results
    }

    fn search_rotations(
        day: usize,
        current: &mut [u8; 7],
        contract: &InrcContract,
        scenario: &InrcScenario,
        forbidden: &[Vec<bool>],
        num_shifts: usize,
        results: &mut Vec<Rotation7>,
    ) {
        if day == 7 {
            if Self::validate_rotation(current, contract, scenario) {
                results.push(Rotation7 { shifts: *current });
            }
            return;
        }

        // Branch 0: Off
        current[day] = 0;
        // Prune Sunday if complete_weekends is required and Saturday was worked
        if day == 6 && contract.complete_weekends > 0 && current[5] > 0 {
            // Cannot be Off on Sunday if worked on Saturday
        } else {
            Self::search_rotations(
                day + 1,
                current,
                contract,
                scenario,
                forbidden,
                num_shifts,
                results,
            );
        }

        // Branch 1..=num_shifts: Shifts
        for s in 0..num_shifts {
            // Check HC4 succession with day - 1
            if day > 0 && current[day - 1] > 0 {
                let prev_s = (current[day - 1] - 1) as usize;
                if forbidden[prev_s][s] {
                    continue; // Pruned by forbidden succession
                }
            }

            // Prune Sunday if complete_weekends is required and Saturday was Off
            if day == 6 && contract.complete_weekends > 0 && current[5] == 0 {
                continue; // Cannot work on Sunday if Off on Saturday
            }

            current[day] = (s + 1) as u8;
            Self::search_rotations(
                day + 1,
                current,
                contract,
                scenario,
                forbidden,
                num_shifts,
                results,
            );
        }
    }

    /// Validates internal ergonomic constraints (S2, S3, S4, S6) for a candidate 7-day rotation.
    fn validate_rotation(
        shifts: &[u8; 7],
        contract: &InrcContract,
        scenario: &InrcScenario,
    ) -> bool {
        // Complete weekends check
        if contract.complete_weekends > 0 && (shifts[5] > 0) != (shifts[6] > 0) {
            return false;
        }

        // S2: Working Streaks & S3: Off Streaks
        // We evaluate streaks that start and end strictly inside the week [1..5].
        // For streaks touching the boundary (day 0 or day 6), they must not exceed max.
        let mut d = 0;
        while d < 7 {
            let is_work = shifts[d] > 0;
            let start = d;
            while d < 7 && (shifts[d] > 0) == is_work {
                d += 1;
            }
            let len = d - start;

            if is_work {
                if len > contract.max_consecutive_working_days {
                    return false;
                }
                // Strictly internal work streak: must satisfy min
                if start > 0 && d < 7 && len < contract.min_consecutive_working_days {
                    return false;
                }
            } else {
                if len > contract.max_consecutive_days_off {
                    return false;
                }
                // Strictly internal off streak: must satisfy min
                if start > 0 && d < 7 && len < contract.min_consecutive_days_off {
                    return false;
                }
            }
        }

        // S4: Shift Type Streaks
        let mut d = 0;
        while d < 7 {
            if shifts[d] > 0 {
                let s_val = shifts[d];
                let s_idx = (s_val - 1) as usize;
                let s_type = &scenario.shift_types[s_idx];
                let start = d;
                while d < 7 && shifts[d] == s_val {
                    d += 1;
                }
                let len = d - start;

                if len > s_type.max_consecutive {
                    return false;
                }
                // Strictly internal shift streak: must satisfy min
                if start > 0 && d < 7 && len < s_type.min_consecutive {
                    return false;
                }
            } else {
                d += 1;
            }
        }

        true
    }

    /// Evaluates the boundary transition cost of applying `rotation` to a nurse with `history`.
    /// Returns `(is_hard_legal, boundary_penalty)`.
    pub fn eval_boundary_cost(
        &self,
        rotation: &Rotation7,
        history: Option<&InrcNurseHistory>,
        contract: &InrcContract,
        scenario: &InrcScenario,
    ) -> (bool, i32) {
        let hist = match history {
            Some(h) => h,
            None => return (true, 0),
        };

        let mut penalty = 0i32;

        // HC4 Check: Preceding shift from history
        if hist.last_assigned_shift_type != "None" && !hist.last_assigned_shift_type.is_empty() {
            if let Some(first_s) = rotation.get_shift(0) {
                if let Some(&prev_idx) = self.shift_type_indices.get(&hist.last_assigned_shift_type) {
                    if self.forbidden_matrix[prev_idx][first_s] {
                        return (false, 0); // Hard violation
                    }
                }
            }
        }

        // S2 / S3 Boundary at Day 0
        if rotation.is_working(0) {
            // Nurse works on Monday
            if hist.number_of_consecutive_days_off > 0 {
                // Days off streak ended on Sunday!
                if hist.number_of_consecutive_days_off < contract.min_consecutive_days_off {
                    let diff = contract.min_consecutive_days_off - hist.number_of_consecutive_days_off;
                    penalty += diff as i32 * 30; // consecutive_days_off weight
                }
            }
            if hist.number_of_consecutive_working_days > 0 {
                // Working streak continues into Monday!
                let mut monday_streak = 0;
                while monday_streak < 7 && rotation.is_working(monday_streak) {
                    monday_streak += 1;
                }
                let total_streak = hist.number_of_consecutive_working_days + monday_streak;
                if total_streak > contract.max_consecutive_working_days {
                    let excess = total_streak - contract.max_consecutive_working_days;
                    penalty += excess as i32 * 30;
                }
            }
        } else {
            // Nurse is off on Monday
            if hist.number_of_consecutive_working_days > 0 {
                // Working streak ended on Sunday!
                if hist.number_of_consecutive_working_days < contract.min_consecutive_working_days {
                    let diff = contract.min_consecutive_working_days - hist.number_of_consecutive_working_days;
                    penalty += diff as i32 * 30;
                }
            }
            if hist.number_of_consecutive_days_off > 0 {
                // Days off streak continues into Monday!
                let mut monday_off = 0;
                while monday_off < 7 && !rotation.is_working(monday_off) {
                    monday_off += 1;
                }
                let total_off = hist.number_of_consecutive_days_off + monday_off;
                if total_off > contract.max_consecutive_days_off {
                    let excess = total_off - contract.max_consecutive_days_off;
                    penalty += excess as i32 * 30;
                }
            }
        }

        // S4 Shift Type Boundary at Day 0
        if let Some(first_s) = rotation.get_shift(0) {
            let first_s_name = &scenario.shift_types[first_s].id;
            if hist.last_assigned_shift_type != "None" && !hist.last_assigned_shift_type.is_empty() {
                if hist.last_assigned_shift_type == *first_s_name {
                    // Streak continues
                    let s_type = &scenario.shift_types[first_s];
                    let mut streak = 0;
                    while streak < 7 && rotation.get_shift(streak) == Some(first_s) {
                        streak += 1;
                    }
                    let total = hist.number_of_consecutive_assignments + streak;
                    if total > s_type.max_consecutive {
                        penalty += (total - s_type.max_consecutive) as i32 * 15;
                    }
                } else if hist.number_of_consecutive_assignments > 0 {
                    // Shift streak ended on Sunday!
                    if let Some(&prev_idx) = self.shift_type_indices.get(&hist.last_assigned_shift_type) {
                        let prev_type = &scenario.shift_types[prev_idx];
                        if hist.number_of_consecutive_assignments < prev_type.min_consecutive {
                            let diff = prev_type.min_consecutive - hist.number_of_consecutive_assignments;
                            penalty += diff as i32 * 15;
                        }
                    }
                }
            }
        }

        (true, penalty)
    }

    /// Returns all rotations for `contract_id` that have zero boundary penalty with `history`.
    pub fn get_zero_defect_rotations_for_nurse(
        &self,
        contract_id: &str,
        history: Option<&InrcNurseHistory>,
        contract: &InrcContract,
        scenario: &InrcScenario,
    ) -> Vec<Rotation7> {
        let pool = match self.rotations_by_contract.get(contract_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        pool.iter()
            .copied()
            .filter(|r| {
                let (legal, pen) = self.eval_boundary_cost(r, history, contract, scenario);
                legal && pen == 0
            })
            .collect()
    }

    /// Checks if nurse `nurse_idx` has the required skills to work `rotation` on all 7 days.
    #[inline]
    pub fn is_rotation_qualified(
        rotation: &Rotation7,
        nurse_idx: usize,
        nurse_can_work: &[Vec<Vec<bool>>],
    ) -> bool {
        for d in 0..7 {
            if let Some(s) = rotation.get_shift(d) {
                if !nurse_can_work[nurse_idx][d][s] {
                    return false;
                }
            }
        }
        true
    }

    /// Returns candidate zero-defect rotations for a nurse that:
    /// 1. Are skill-qualified for the nurse across all 7 days
    /// 2. Satisfy HC4 (no forbidden succession with Sunday history)
    /// Sorted by boundary penalty (lowest penalty first).
    pub fn get_candidate_rotations_for_nurse(
        &self,
        nurse_idx: usize,
        contract_id: &str,
        history: Option<&InrcNurseHistory>,
        contract: &InrcContract,
        scenario: &InrcScenario,
        nurse_can_work: &[Vec<Vec<bool>>],
        target_band: (usize, usize),
    ) -> Vec<(Rotation7, i32)> {
        let pool = match self.rotations_by_contract.get(contract_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        let target = (target_band.0 + target_band.1) / 2;

        let mut candidates: Vec<(Rotation7, i32)> = pool
            .iter()
            .copied()
            .filter_map(|r| {
                if !Self::is_rotation_qualified(&r, nurse_idx, nurse_can_work) {
                    return None;
                }
                let (legal, pen) = self.eval_boundary_cost(&r, history, contract, scenario);
                if legal {
                    Some((r, pen))
                } else {
                    None
                }
            })
            .collect();

        candidates.sort_by_key(|&(r, pen)| {
            let a_diff = (r.total_assignments() as i32 - target as i32).abs();
            (pen, a_diff)
        });

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::inrc::parser::{parse_history, parse_scenario};

    #[test]
    fn test_rotation_pool_generation_n030w4() {
        let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("n030w4");
        let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).expect("Scenario parse failed");
        let history = parse_history(base_dir.join("H0-n030w4-0.json")).expect("History parse failed");

        let pool = RotationPool::build(&scenario);
        assert!(!pool.rotations_by_contract.is_empty(), "Pool must contain contracts");

        for (cid, rots) in &pool.rotations_by_contract {
            println!("Contract {}: {} zero-defect rotations generated", cid, rots.len());
            assert!(rots.len() > 10, "Each contract must have at least 10 valid rotations");

            // Verify all rotations strictly satisfy complete weekends
            let contract = scenario.contracts.iter().find(|c| &c.id == cid).unwrap();
            for r in rots {
                if contract.complete_weekends > 0 {
                    assert_eq!(r.shifts[5] > 0, r.shifts[6] > 0, "Complete weekend violated");
                }
            }
        }

        // Test boundary matching for nurse 0
        let n0 = &scenario.nurses[0];
        let contract = scenario.contracts.iter().find(|c| c.id == n0.contract).unwrap();
        let hist = history.nurse_history.iter().find(|h| h.nurse == n0.id);
        let zero_defect = pool.get_zero_defect_rotations_for_nurse(&n0.contract, hist, contract, &scenario);
        println!("Nurse {} has {} zero-defect boundary-compatible rotations", n0.id, zero_defect.len());
        assert!(!zero_defect.is_empty(), "Must find zero-defect rotations compatible with history");
    }
}
