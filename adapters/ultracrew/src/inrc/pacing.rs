//! `ultracrew::inrc::pacing` — Dynamic Multi-Week Horizon Shadow-Price Pacing Engine.
//!
//! Replaces static weekly quota clamping with dynamic, horizon-aware assignment targets.
//! In multi-week rolling horizon scenarios, demand is non-uniform across weeks.
//! This module tracks historical capacity consumption and calculates exact weekly target bands:
//! - Prevents nurses from exhausting contract assignments prematurely in slack weeks
//! - Reserves qualified capacity for peak demand weeks
//! - Eliminates S7 max working weekend violations by identifying exhausted weekend quotas

use std::collections::HashMap;
use crate::inrc::models::{InrcHistory, InrcScenario, InrcWeekData};

/// Dynamic pacing recommendations for a single nurse in the current week.
#[derive(Clone, Copy, Debug)]
pub struct NursePacingTarget {
    /// Nurse index in `scenario.nurses`
    pub nurse_idx: usize,
    /// Recommended ideal assignments this week to stay on pace for horizon contract bounds
    pub target_assignments: usize,
    /// Minimum recommended assignments this week
    pub min_assignments: usize,
    /// Maximum recommended assignments this week
    pub max_assignments: usize,
    /// Remaining multi-week assignment capacity
    pub remaining_contract_capacity: usize,
    /// Remaining working weekends allowed under contract
    pub remaining_working_weekends: usize,
    /// Whether this nurse should avoid working on the weekend (quota exhausted)
    pub avoid_weekend: bool,
}

/// Horizon Pacing Context for the current planning week.
#[derive(Clone, Debug)]
pub struct HorizonPacing {
    /// Pacing targets per nurse (indexed by nurse index)
    pub nurse_targets: Vec<NursePacingTarget>,
    /// Total mandatory shift demand this week
    pub total_min_demand: usize,
    /// Total optimal shift demand this week
    pub total_opt_demand: usize,
    /// Current week index (0-based)
    pub current_week: usize,
    /// Total weeks in scenario
    pub total_weeks: usize,
    /// Remaining weeks including current week
    pub remaining_weeks: usize,
}

impl HorizonPacing {
    /// Computes horizon pacing targets for the given scenario, week data, and history.
    pub fn compute(
        scenario: &InrcScenario,
        week_data: &InrcWeekData,
        history: &InrcHistory,
    ) -> Self {
        let num_nurses = scenario.nurses.len();
        let total_weeks = scenario.number_of_weeks;
        let current_week = (history.week).min(total_weeks.saturating_sub(1));
        let remaining_weeks = (total_weeks - current_week).max(1);

        let mut nurse_hist_map = HashMap::new();
        for nh in &history.nurse_history {
            nurse_hist_map.insert(&nh.nurse, nh);
        }

        // Calculate this week's total demand
        let mut total_min_demand = 0;
        let mut total_opt_demand = 0;
        for req in &week_data.requirements {
            let levels = [
                &req.monday,
                &req.tuesday,
                &req.wednesday,
                &req.thursday,
                &req.friday,
                &req.saturday,
                &req.sunday,
            ];
            for lvl in levels {
                total_min_demand += lvl.minimum;
                total_opt_demand += lvl.optimal;
            }
        }

        let mut nurse_targets = Vec::with_capacity(num_nurses);

        for (n_idx, nurse) in scenario.nurses.iter().enumerate() {
            let contract = scenario
                .contracts
                .iter()
                .find(|c| c.id == nurse.contract)
                .expect("Contract not found");

            let hist = nurse_hist_map.get(&nurse.id);
            let hist_assignments = hist.map(|h| h.number_of_assignments).unwrap_or(0);
            let hist_weekends = hist.map(|h| h.number_of_working_weekends).unwrap_or(0);

            let remaining_contract_capacity =
                contract.max_assignments.saturating_sub(hist_assignments);
            let remaining_contract_min =
                contract.min_assignments.saturating_sub(hist_assignments);

            let remaining_working_weekends =
                contract.max_working_weekends.saturating_sub(hist_weekends);
            let avoid_weekend = remaining_working_weekends == 0;

            // Target pacing calculation
            let min_week = contract.min_assignments / total_weeks;
            let max_week = (contract.max_assignments as f64 / total_weeks as f64).ceil() as usize;

            let ideal_remaining = (remaining_contract_min + remaining_contract_capacity) as f64 / 2.0;
            let paced_target = (ideal_remaining / remaining_weeks as f64).round() as usize;

            let (min_assignments, max_assignments, target_assignments) = if remaining_weeks == 1 {
                let min_a = remaining_contract_min.min(7);
                let max_a = remaining_contract_capacity.min(7);
                let tgt = ((min_a + max_a) as f64 / 2.0).round() as usize;
                (min_a, max_a, tgt)
            } else {
                let tgt = paced_target.clamp(min_week, max_week);
                let min_a = (remaining_contract_min / remaining_weeks).clamp(min_week, tgt);
                let max_a = ((remaining_contract_capacity as f64 / remaining_weeks as f64).ceil() as usize)
                    .clamp(tgt, max_week);
                (min_a, max_a, tgt)
            };

            nurse_targets.push(NursePacingTarget {
                nurse_idx: n_idx,
                target_assignments,
                min_assignments,
                max_assignments,
                remaining_contract_capacity,
                remaining_working_weekends,
                avoid_weekend,
            });
        }

        Self {
            nurse_targets,
            total_min_demand,
            total_opt_demand,
            current_week,
            total_weeks,
            remaining_weeks,
        }
    }

    /// Evaluates S1 penalty for a nurse given actual week assignments using dynamic pacing.
    #[inline(always)]
    pub fn eval_pacing_penalty(&self, nurse_idx: usize, week_assignments: usize, assignment_weight: i32) -> i32 {
        let target = &self.nurse_targets[nurse_idx];
        let mut penalty = 0i32;

        if week_assignments < target.min_assignments {
            penalty += (target.min_assignments - week_assignments) as i32 * assignment_weight;
        } else if week_assignments > target.max_assignments {
            penalty += (week_assignments - target.max_assignments) as i32 * assignment_weight;
        }

        penalty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::inrc::parser::{parse_history, parse_scenario, parse_week_data};

    #[test]
    fn test_horizon_pacing_n030w4() {
        let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("n030w4");
        let scenario = parse_scenario(base_dir.join("Sc-n030w4.json")).unwrap();
        let week_data = parse_week_data(base_dir.join("WD-n030w4-0.json")).unwrap();
        let history = parse_history(base_dir.join("H0-n030w4-0.json")).unwrap();

        let pacing = HorizonPacing::compute(&scenario, &week_data, &history);
        assert_eq!(pacing.nurse_targets.len(), 30);
        assert!(pacing.total_min_demand > 0);
        assert!(pacing.total_opt_demand >= pacing.total_min_demand);

        for target in &pacing.nurse_targets {
            assert!(target.min_assignments <= target.target_assignments);
            assert!(target.target_assignments <= target.max_assignments);
        }
    }
}
