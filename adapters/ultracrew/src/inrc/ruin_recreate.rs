//! `ultracrew::inrc::ruin_recreate` — Adaptive Large Neighborhood Search (ALNS) for Nurse Rostering.
//!
//! Escapes deep combinatorial local minima via multi-strategy destroy and repair:
//! - Destroy Operators:
//!   1. Temporal Window Ruin: unassigns 4-8 nurses across a contiguous 2-3 day window or weekend.
//!   2. High-Penalty Nurse Ruin: identifies the worst-performing nurses and unassigns their schedules.
//!   3. Weekend Stub Ruin: targets incomplete weekends ($S_6$) to realign Saturday/Sunday cohesion.
//! - Repair Operators:
//!   1. Regret-2 Insertion: assigns hardest-to-place shifts first based on regret delta.
//!   2. Soft-Cost Greedy Insertion: re-inserts shifts minimizing marginal nurse soft penalties.

use rand::Rng;
use std::collections::HashMap;
use crate::inrc::optimization::{InrcGenome, InrcOptimizer};

/// Adaptive Large Neighborhood Search (ALNS) Engine.
#[derive(Clone, Debug, Default)]
pub struct ClusterRuinRecreate;

impl ClusterRuinRecreate {
    pub fn new() -> Self {
        Self
    }

    /// Finds nurse clusters grouped by identical contract ID.
    pub fn find_contract_clusters(optimizer: &InrcOptimizer) -> HashMap<String, Vec<usize>> {
        let mut clusters: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, nurse) in optimizer.context.scenario.nurses.iter().enumerate() {
            clusters
                .entry(nurse.contract.clone())
                .or_default()
                .push(idx);
        }
        clusters
    }

    /// Performs one step of Adaptive Large Neighborhood Search on `genome`.
    /// Returns `Some((cand_genome, delta_penalty))` if a feasible move was found.
    pub fn step<R: Rng + ?Sized>(
        &self,
        optimizer: &InrcOptimizer,
        current: &InrcGenome,
        rng: &mut R,
        clusters: &HashMap<String, Vec<usize>>,
    ) -> Option<(InrcGenome, f64)> {
        self.step_alns(optimizer, current, rng, clusters)
    }

    /// Multi-operator ALNS step with destroy and repair selection.
    pub fn step_alns<R: Rng + ?Sized>(
        &self,
        optimizer: &InrcOptimizer,
        current: &InrcGenome,
        rng: &mut R,
        clusters: &HashMap<String, Vec<usize>>,
    ) -> Option<(InrcGenome, f64)> {
        let num_nurses = optimizer.context.num_nurses;
        if num_nurses == 0 {
            return None;
        }

        let destroy_mode = rng.gen_range(0..3);

        let (selected_nurses, days) = match destroy_mode {
            0 => {
                // Destroy 1: Temporal Window Ruin on a contract cluster
                let viable_clusters: Vec<&Vec<usize>> = clusters
                    .values()
                    .filter(|group| group.len() >= 4)
                    .collect();

                if viable_clusters.is_empty() {
                    return None;
                }

                let cluster = viable_clusters[rng.gen_range(0..viable_clusters.len())];
                let k = rng.gen_range(4..=8.min(cluster.len()));

                let mut nurses = Vec::with_capacity(k);
                let mut indices: Vec<usize> = (0..cluster.len()).collect();
                for _ in 0..k {
                    let pick = rng.gen_range(0..indices.len());
                    nurses.push(cluster[indices.swap_remove(pick)]);
                }

                let days = if rng.gen_bool(0.50) {
                    vec![5, 6] // Weekend
                } else {
                    let start_d = rng.gen_range(0..5);
                    vec![start_d, start_d + 1]
                };
                (nurses, days)
            }
            1 => {
                // Destroy 2: High-Penalty Worst Nurses Ruin
                let mut nurse_penalties: Vec<(usize, i32)> = (0..num_nurses)
                    .map(|n| (n, optimizer.eval_nurse_soft_penalty(current, n)))
                    .collect();
                nurse_penalties.sort_by_key(|&(_, p)| std::cmp::Reverse(p));

                let k = rng.gen_range(2..=5.min(num_nurses));
                let nurses: Vec<usize> = nurse_penalties.iter().take(k).map(|&(n, _)| n).collect();
                let days = (0..7).collect(); // Entire week
                (nurses, days)
            }
            _ => {
                // Destroy 3: Weekend Stub Ruin (Nurses with split weekends)
                let mut split_weekend_nurses = Vec::new();
                for n in 0..num_nurses {
                    let sat = optimizer.get_assigned_shift(current, n, 5).is_some();
                    let sun = optimizer.get_assigned_shift(current, n, 6).is_some();
                    if sat != sun {
                        split_weekend_nurses.push(n);
                    }
                }

                if split_weekend_nurses.len() < 2 {
                    return None;
                }

                let k = rng.gen_range(2..=6.min(split_weekend_nurses.len()));
                let mut nurses = Vec::with_capacity(k);
                let mut indices: Vec<usize> = (0..split_weekend_nurses.len()).collect();
                for _ in 0..k {
                    let pick = rng.gen_range(0..indices.len());
                    nurses.push(split_weekend_nurses[indices.swap_remove(pick)]);
                }
                (nurses, vec![5, 6])
            }
        };

        // Ruin: Collect shifts currently worked by selected_nurses on days
        let mut shifts_pool: Vec<(usize, usize)> = Vec::new(); // (day, shift_idx)
        let mut cand = current.clone();

        let mut old_nurses_soft = 0i32;
        for &n in &selected_nurses {
            old_nurses_soft += optimizer.eval_nurse_soft_penalty(current, n);
            for &d in &days {
                if let Some(s) = optimizer.get_assigned_shift(current, n, d) {
                    shifts_pool.push((d, s));
                    optimizer.set_assigned_shift(&mut cand, n, d, None);
                }
            }
        }

        let (_, old_s8) = optimizer.eval_days_coverage(current, &days);

        // Recreate: Regret-2 or Shuffled Greedy Insertion
        let use_regret = rng.gen_bool(0.50);

        if use_regret && shifts_pool.len() > 1 {
            // Regret-2 Recreate
            let mut remaining_pool = shifts_pool;
            while !remaining_pool.is_empty() {
                let mut best_slot_idx = 0;
                let mut max_regret = -1;
                let mut best_nurse_for_slot = None;

                for (idx, &(d, s)) in remaining_pool.iter().enumerate() {
                    let mut nurse_costs = Vec::new();
                    for &n in &selected_nurses {
                        if optimizer.get_assigned_shift(&cand, n, d).is_some() {
                            continue;
                        }
                        if !optimizer.context.inverted.nurse_can_work[n][d][s] {
                            continue;
                        }
                        let can_work = if d == 0 {
                            !optimizer.context.inverted.history_forbidden[n][s]
                        } else if let Some(prev) = optimizer.get_assigned_shift(&cand, n, d - 1) {
                            !optimizer.context.inverted.forbidden_matrix[prev][s]
                        } else {
                            true
                        };
                        if !can_work {
                            continue;
                        }
                        if d + 1 < 7 {
                            if let Some(next) = optimizer.get_assigned_shift(&cand, n, d + 1) {
                                if optimizer.context.inverted.forbidden_matrix[s][next] {
                                    continue;
                                }
                            }
                        }

                        let old_p = optimizer.eval_nurse_soft_penalty(&cand, n);
                        optimizer.set_assigned_shift(&mut cand, n, d, Some(s));
                        let new_p = optimizer.eval_nurse_soft_penalty(&cand, n);
                        optimizer.set_assigned_shift(&mut cand, n, d, None);
                        nurse_costs.push((n, new_p - old_p));
                    }

                    nurse_costs.sort_by_key(|&(_, cost)| cost);
                    if nurse_costs.is_empty() {
                        continue;
                    }

                    let regret = if nurse_costs.len() >= 2 {
                        nurse_costs[1].1 - nurse_costs[0].1
                    } else {
                        50 // high regret if only 1 nurse can work
                    };

                    if regret > max_regret {
                        max_regret = regret;
                        best_slot_idx = idx;
                        best_nurse_for_slot = Some(nurse_costs[0].0);
                    }
                }

                let (day, shift_idx) = remaining_pool.swap_remove(best_slot_idx);
                if let Some(n) = best_nurse_for_slot {
                    optimizer.set_assigned_shift(&mut cand, n, day, Some(shift_idx));
                } else {
                    return None;
                }
            }
        } else {
            // Shuffled Greedy Recreate
            for i in (1..shifts_pool.len()).rev() {
                let j = rng.gen_range(0..=i);
                shifts_pool.swap(i, j);
            }

            for (day, shift_idx) in shifts_pool {
                let mut best_nurse = None;
                let mut best_cost = i32::MAX;

                for &n in &selected_nurses {
                    if optimizer.get_assigned_shift(&cand, n, day).is_some() {
                        continue;
                    }

                    if !optimizer.context.inverted.nurse_can_work[n][day][shift_idx] {
                        continue;
                    }

                    let can_work = if day == 0 {
                        !optimizer.context.inverted.history_forbidden[n][shift_idx]
                    } else if let Some(prev) = optimizer.get_assigned_shift(&cand, n, day - 1) {
                        !optimizer.context.inverted.forbidden_matrix[prev][shift_idx]
                    } else {
                        true
                    };

                    if !can_work {
                        continue;
                    }

                    if day + 1 < 7 {
                        if let Some(next) = optimizer.get_assigned_shift(&cand, n, day + 1) {
                            if optimizer.context.inverted.forbidden_matrix[shift_idx][next] {
                                continue;
                            }
                        }
                    }

                    let old_p = optimizer.eval_nurse_soft_penalty(&cand, n);
                    optimizer.set_assigned_shift(&mut cand, n, day, Some(shift_idx));
                    let new_p = optimizer.eval_nurse_soft_penalty(&cand, n);
                    optimizer.set_assigned_shift(&mut cand, n, day, None);

                    let cost = new_p - old_p;
                    if cost < best_cost {
                        best_cost = cost;
                        best_nurse = Some(n);
                    }
                }

                if let Some(n) = best_nurse {
                    optimizer.set_assigned_shift(&mut cand, n, day, Some(shift_idx));
                } else {
                    return None;
                }
            }
        }

        // Feasibility and Hard Constraint Check
        for &n in &selected_nurses {
            if optimizer.nurse_has_forbidden_succession(&cand, n) {
                return None;
            }

            let nurse = &optimizer.context.scenario.nurses[n];
            let contract = optimizer
                .context
                .scenario
                .contracts
                .iter()
                .find(|c| c.id == nurse.contract)
                .unwrap();
            let hist = optimizer
                .context
                .history
                .nurse_history
                .iter()
                .find(|h| &h.nurse == &nurse.id);
            let prev_total = hist.map(|h| h.number_of_assignments).unwrap_or(0);
            let week_assignments = (0..7)
                .filter(|&d| optimizer.get_assigned_shift(&cand, n, d).is_some())
                .count();
            if prev_total + week_assignments > contract.max_assignments {
                return None;
            }

            let prev_consecutive = hist
                .map(|h| {
                    if h.number_of_consecutive_days_off > 0 {
                        0
                    } else {
                        h.number_of_consecutive_assignments
                    }
                })
                .unwrap_or(0);
            let mut current_streak = prev_consecutive;
            for d in 0..7 {
                if optimizer.get_assigned_shift(&cand, n, d).is_some() {
                    current_streak += 1;
                    if current_streak > contract.max_consecutive_working_days {
                        return None;
                    }
                } else {
                    current_streak = 0;
                }
            }
        }

        let (feasible, new_s8) = optimizer.eval_days_coverage(&cand, &days);
        if !feasible {
            return None;
        }

        let mut new_nurses_soft = 0i32;
        for &n in &selected_nurses {
            new_nurses_soft += optimizer.eval_nurse_soft_penalty(&cand, n);
        }

        let delta = (new_nurses_soft - old_nurses_soft) + (new_s8 - old_s8);
        Some((cand, delta as f64))
    }
}
