//! `ultracrew::inrc::lookahead` — Sunday Boundary Terminal Valuation & Lookahead Pricing Engine.
//!
//! Evaluates the terminal shadow prices of Sunday boundary states ($d = 6$)
//! to prevent cross-week handoff poisoning between Week $w$ and Week $w+1$.
//!
//! Identifies and penalizes:
//! 1. Sunday Night Succession Traps ($HC_4$): Prevents assigning Sunday Night shifts
//!    that forbid Monday Early/Day shifts in Week $w+1$.
//! 2. Short Work Streak Stubs ($S_2$): Prevents ending Sunday on 1- or 2-day work streaks
//!    that force illegal stubs or mandatory overtime in Week $w+1$.
//! 3. Short Rest Streak Stubs ($S_3$): Prevents ending Sunday on 1-day rest streaks
//!    that force Monday off or mandatory $S_3$ violations in Week $w+1$.
//! 4. Shift Type Fragmentation ($S_4$): Prevents ending Sunday on incomplete shift type streaks.

use crate::inrc::optimization::{InrcGenome, InrcOptimizer};

/// Sunday terminal valuation configuration and evaluator.
#[derive(Clone, Debug)]
pub struct SundayLookahead {
    /// Shadow price for Sunday Night shift assignment (+30)
    pub sunday_night_price: i32,
    /// Shadow price multiplier for incomplete Sunday work streaks (+30 per day short)
    pub short_work_streak_price: i32,
    /// Shadow price multiplier for incomplete Sunday rest streaks (+30 per day short)
    pub short_rest_streak_price: i32,
    /// Shadow price for shift-type streak stubs (+15 per day short)
    pub short_shift_streak_price: i32,
}

impl Default for SundayLookahead {
    fn default() -> Self {
        Self {
            sunday_night_price: 30,
            short_work_streak_price: 30,
            short_rest_streak_price: 30,
            short_shift_streak_price: 15,
        }
    }
}

impl SundayLookahead {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes the terminal lookahead shadow penalty for a single nurse on Sunday ($d = 6$).
    #[inline]
    pub fn eval_nurse_terminal_penalty(
        &self,
        optimizer: &InrcOptimizer,
        genome: &InrcGenome,
        nurse_idx: usize,
    ) -> i32 {
        let nurse = &optimizer.context.scenario.nurses[nurse_idx];
        let contract = optimizer
            .context
            .scenario
            .contracts
            .iter()
            .find(|c| c.id == nurse.contract)
            .expect("Contract not found");

        let sunday_shift = optimizer.get_assigned_shift(genome, nurse_idx, 6);
        let mut penalty = 0i32;

        match sunday_shift {
            Some(s_idx) => {
                let shift_id = &optimizer.context.shift_types[s_idx];

                // 1. Sunday Night Trap: Forbids Monday Early / Day
                // INRC-II night shift typically has ID "N" or is forbidden before early/day
                let is_night = shift_id.eq_ignore_ascii_case("N")
                    || shift_id.eq_ignore_ascii_case("Night")
                    || optimizer
                        .context
                        .inverted
                        .forbidden_matrix
                        .get(s_idx)
                        .map(|row| row.iter().any(|&forbid| forbid))
                        .unwrap_or(false);

                if is_night {
                    penalty += self.sunday_night_price;
                }

                // 2. Sunday Work Streak Lookahead (S2)
                // Calculate streak of consecutive working days ending on Sunday
                let mut work_streak = 0;
                for d in (0..=6).rev() {
                    if optimizer.get_assigned_shift(genome, nurse_idx, d).is_some() {
                        work_streak += 1;
                    } else {
                        break;
                    }
                }

                // If work streak includes all 7 days, check history
                if work_streak == 7 {
                    let nurse_id = &optimizer.context.scenario.nurses[nurse_idx].id;
                    if let Some(h) = optimizer.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id) {
                        work_streak += h.number_of_consecutive_working_days;
                    }
                }

                let min_work = contract.min_consecutive_working_days;
                if work_streak > 0 && work_streak < min_work {
                    // This nurse is leaving week w on an incomplete work streak.
                    // Week w+1 must assign them on Monday to avoid S2 penalty.
                    let deficit = min_work - work_streak;
                    penalty += deficit as i32 * self.short_work_streak_price;
                }

                // 3. Shift Type Streak Lookahead (S4)
                if let Some(st_info) = optimizer
                    .context
                    .scenario
                    .shift_types
                    .iter()
                    .find(|st| st.id == *shift_id)
                {
                    let min_consec = st_info.min_consecutive;
                    if min_consec > 1 {
                        let mut shift_streak = 0;
                        for d in (0..=6).rev() {
                            if optimizer.get_assigned_shift(genome, nurse_idx, d) == Some(s_idx) {
                                shift_streak += 1;
                            } else {
                                break;
                            }
                        }
                        if shift_streak == 7 {
                            let nurse_id = &optimizer.context.scenario.nurses[nurse_idx].id;
                            if let Some(h) = optimizer.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id) {
                                if h.last_assigned_shift_type == *shift_id {
                                    shift_streak += h.number_of_consecutive_assignments;
                                }
                            }
                        }
                        if shift_streak > 0 && shift_streak < min_consec {
                            let deficit = min_consec - shift_streak;
                            penalty += deficit as i32 * self.short_shift_streak_price;
                        }
                    }
                }
            }
            None => {
                // Sunday is Day Off: Rest Streak Lookahead (S3)
                let mut rest_streak = 0;
                for d in (0..=6).rev() {
                    if optimizer.get_assigned_shift(genome, nurse_idx, d).is_none() {
                        rest_streak += 1;
                    } else {
                        break;
                    }
                }

                if rest_streak == 7 {
                    let nurse_id = &optimizer.context.scenario.nurses[nurse_idx].id;
                    if let Some(h) = optimizer.context.history.nurse_history.iter().find(|h| &h.nurse == nurse_id) {
                        rest_streak += h.number_of_consecutive_days_off;
                    }
                }

                let min_rest = contract.min_consecutive_days_off;
                if rest_streak > 0 && rest_streak < min_rest {
                    let deficit = min_rest - rest_streak;
                    penalty += deficit as i32 * self.short_rest_streak_price;
                }
            }
        }

        penalty
    }

    /// Evaluates total Sunday lookahead penalty across all nurses.
    pub fn eval_total_terminal_penalty(
        &self,
        optimizer: &InrcOptimizer,
        genome: &InrcGenome,
    ) -> i32 {
        let mut total = 0i32;
        for n in 0..optimizer.context.num_nurses {
            total += self.eval_nurse_terminal_penalty(optimizer, genome, n);
        }
        total
    }

    /// Evaluates the delta in terminal lookahead penalty for a candidate Sunday shift change on nurse `n`.
    #[inline]
    pub fn eval_nurse_sunday_delta(
        &self,
        optimizer: &InrcOptimizer,
        current_genome: &InrcGenome,
        cand_genome: &InrcGenome,
        nurse_idx: usize,
    ) -> i32 {
        let old_p = self.eval_nurse_terminal_penalty(optimizer, current_genome, nurse_idx);
        let new_p = self.eval_nurse_terminal_penalty(optimizer, cand_genome, nurse_idx);
        new_p - old_p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use crate::ecology::WorkforceEcology;
    use crate::inrc::optimization::InrcContext;
    use crate::inrc::parser::{parse_history, parse_scenario, parse_week_data};
    use rand::SeedableRng;

    #[test]
    fn test_sunday_lookahead_evaluation() {
        let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("n030w4");
        let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
        let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
        let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();

        let context = Arc::new(InrcContext::new(scenario, week_data, history, WorkforceEcology::new()));
        let optimizer = InrcOptimizer { context };

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let genome = optimizer.context.inverted.construct_feasible_genome(&mut rng);

        let lookahead = SundayLookahead::new();
        let penalty = lookahead.eval_total_terminal_penalty(&optimizer, &genome);
        assert!(penalty >= 0);
    }
}
