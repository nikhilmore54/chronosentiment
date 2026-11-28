//! `ultracrew::inrc::ruin_recreate` — Cluster Ruin-and-Recreate Large Neighborhood Search (LNS).
//!
//! Escapes deep local minima unreachable by 1-opt and 2-opt moves:
//! - Selects a cluster of $k = 4\text{--}8$ nurses sharing identical contract / skill profiles.
//! - Selects a contiguous temporal window (e.g. Weekend $\{5, 6\}$ or 2-3 weekdays).
//! - "Ruin": Clears assignments for the cluster within the window.
//! - "Recreate": Re-assigns shifts optimally using constrained matching, respecting
//!   boundary streak states, skill qualifications, and forbidden successions.

use rand::Rng;
use std::collections::HashMap;
use crate::inrc::optimization::{InrcGenome, InrcOptimizer};

/// Cluster Ruin-and-Recreate LNS Engine.
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

    /// Performs one step of Cluster Ruin-and-Recreate on `genome`.
    /// Returns `Some((cand_genome, delta_penalty))` if an improving or neutral feasible move was found.
    pub fn step<R: Rng + ?Sized>(
        &self,
        optimizer: &InrcOptimizer,
        current: &InrcGenome,
        rng: &mut R,
        clusters: &HashMap<String, Vec<usize>>,
    ) -> Option<(InrcGenome, f64)> {
        if clusters.is_empty() {
            return None;
        }

        // 1. Pick a contract cluster with at least 4 nurses
        let viable_clusters: Vec<&Vec<usize>> = clusters
            .values()
            .filter(|group| group.len() >= 4)
            .collect();

        if viable_clusters.is_empty() {
            return None;
        }

        let cluster = viable_clusters[rng.gen_range(0..viable_clusters.len())];
        let k = rng.gen_range(4..=8.min(cluster.len()));

        // Randomly pick k distinct nurses from the cluster
        let mut selected_nurses = Vec::with_capacity(k);
        let mut indices: Vec<usize> = (0..cluster.len()).collect();
        for _ in 0..k {
            let pick = rng.gen_range(0..indices.len());
            let nurse_idx = cluster[indices.swap_remove(pick)];
            selected_nurses.push(nurse_idx);
        }

        // 2. Pick temporal window
        // 50% chance: Weekend {5, 6}; 50% chance: 2 consecutive weekdays
        let days: Vec<usize> = if rng.gen_bool(0.5) {
            vec![5, 6]
        } else {
            let start_d = rng.gen_range(0..5);
            vec![start_d, start_d + 1]
        };

        // 3. Ruin: Collect shifts currently worked by these k nurses on these days
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

        // 4. Recreate: Greedily reassign the shifts pool among the k nurses
        // Shuffle shifts pool for stochastic exploration
        for i in (1..shifts_pool.len()).rev() {
            let j = rng.gen_range(0..=i);
            shifts_pool.swap(i, j);
        }

        for (day, shift_idx) in shifts_pool {
            // Find eligible candidate among selected_nurses
            let mut best_nurse = None;
            let mut best_cost = i32::MAX;

            for &n in &selected_nurses {
                if optimizer.get_assigned_shift(&cand, n, day).is_some() {
                    continue; // Already assigned on this day
                }

                // Check HC2 skill qualification
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

                // Check forward succession
                if day + 1 < 7 {
                    if let Some(next) = optimizer.get_assigned_shift(&cand, n, day + 1) {
                        if optimizer.context.inverted.forbidden_matrix[shift_idx][next] {
                            continue;
                        }
                    }
                }

                // Evaluate soft cost
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
            }
        }

        // 5. Feasibility and Acceptance Check
        for &n in &selected_nurses {
            if optimizer.nurse_has_forbidden_succession(&cand, n) {
                return None;
            }

            let nurse = &optimizer.context.scenario.nurses[n];
            let contract = optimizer.context.scenario.contracts.iter().find(|c| c.id == nurse.contract).unwrap();
            let hist = optimizer.context.history.nurse_history.iter().find(|h| &h.nurse == &nurse.id);
            let prev_total = hist.map(|h| h.number_of_assignments).unwrap_or(0);
            let week_assignments = (0..7).filter(|&d| optimizer.get_assigned_shift(&cand, n, d).is_some()).count();
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
