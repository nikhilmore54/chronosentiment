//! Scheduling objectives.
//!
//! A [`SchedulingObjective`] maps a [`Roster`] to a scalar score.  Lower
//! scores are better (minimisation convention throughout Layer 4).
//!
//! # Concrete objectives
//!
//! | Struct | Measures |
//! |--------|---------|
//! | [`WorkloadBalanceObjective`] | Variance of total flight-leg counts across crew |
//! | [`CoverageCostObjective`] | Penalty for uncovered or over-covered legs |
//! | [`RestQualityObjective`] | Penalty for rest periods close to the legal minimum |
//! | [`CreditedHoursBalanceObjective`] | Variance of total credited hours across crew (Phase 6) |
//! | [`CreditCostObjective`] | Total monetary credit cost across all crew (Phase 7) |
//!
//! # Minimisation convention
//!
//! All objectives return `f64` where **lower is better**.  A perfectly
//! balanced roster with full coverage and generous rest periods scores 0.0
//! on each objective.
//!
//! # Credit Engine integration (UC-ARCH-001)
//!
//! [`CreditedHoursBalanceObjective`] and [`CreditCostObjective`] are wired to
//! the Credit Engine (Layer 1a).  They use [`GeradCreditPolicy`] by default
//! and accept any [`CreditPolicy`] + [`CostModel`] implementation.

use crate::domain::credit::{CreditContext, CreditPolicy, GeradCreditPolicy};
use crate::domain::cost::{CostContext, CostModel, FlatRateCostModel};
use crate::domain::flight::AirportCode;
use crate::domain::roster::Roster;
use chrono::NaiveDate;

// ── Trait ─────────────────────────────────────────────────────────────────────

/// A scheduling objective function.
///
/// Objectives are pure functions: they do not modify the roster and do not
/// check legality.  The legality engine (Layer 2) is the sole arbiter of
/// feasibility.
pub trait SchedulingObjective: Send + Sync {
    /// A stable identifier for this objective (e.g. `"workload_balance"`).
    fn objective_id(&self) -> &str;

    /// A human-readable name (e.g. `"Workload Balance"`).
    fn objective_name(&self) -> &str;

    /// Evaluate the roster and return a non-negative score.
    ///
    /// Lower is better.  A score of `0.0` means the objective is perfectly
    /// satisfied.
    fn evaluate(&self, roster: &Roster) -> f64;
}

// ── WorkloadBalanceObjective ──────────────────────────────────────────────────

/// Minimises the variance of total assigned flight-leg counts across crew.
///
/// A perfectly balanced roster (every crew member has the same number of
/// assigned legs) scores `0.0`.  The score increases with the variance.
///
/// # Metric
///
/// Score = variance of per-crew leg counts = Σ(count_i − mean)² / N
pub struct WorkloadBalanceObjective;

impl SchedulingObjective for WorkloadBalanceObjective {
    fn objective_id(&self) -> &str {
        "workload_balance"
    }

    fn objective_name(&self) -> &str {
        "Workload Balance"
    }

    fn evaluate(&self, roster: &Roster) -> f64 {
        let counts: Vec<f64> = roster
            .rotations()
            .map(|r| {
                r.pairings().iter()
                    .flat_map(|p| p.duties().iter())
                    .flat_map(|d| d.legs().iter())
                    .count() as f64
            })
            .collect();

        if counts.is_empty() {
            return 0.0;
        }

        let mean = counts.iter().sum::<f64>() / counts.len() as f64;
        let variance = counts.iter().map(|&c| (c - mean).powi(2)).sum::<f64>()
            / counts.len() as f64;
        variance
    }
}

// ── CoverageCostObjective ─────────────────────────────────────────────────────

/// Penalises uncovered and over-covered legs.
///
/// Each uncovered leg (assigned to zero rotations) contributes
/// `uncovered_penalty` to the score.  Each over-covered leg (assigned to
/// more than one rotation) contributes `overcoverage_penalty` per extra
/// assignment.
///
/// A roster with perfect coverage scores `0.0`.
pub struct CoverageCostObjective {
    /// Penalty per uncovered leg.  Default: `100.0`.
    pub uncovered_penalty: f64,
    /// Penalty per extra assignment on an over-covered leg.  Default: `50.0`.
    pub overcoverage_penalty: f64,
}

impl Default for CoverageCostObjective {
    fn default() -> Self {
        Self {
            uncovered_penalty: 100.0,
            overcoverage_penalty: 50.0,
        }
    }
}

impl SchedulingObjective for CoverageCostObjective {
    fn objective_id(&self) -> &str {
        "coverage_cost"
    }

    fn objective_name(&self) -> &str {
        "Coverage Cost"
    }

    fn evaluate(&self, roster: &Roster) -> f64 {
        use std::collections::HashMap;

        // Count assignments per leg.
        let mut assignment_count: HashMap<String, usize> = HashMap::new();
        for rotation in roster.rotations() {
            for pairing in rotation.pairings().iter() {
                for duty in pairing.duties().iter() {
                    for leg in duty.legs().iter() {
                        *assignment_count.entry(leg.id.as_str().to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut score = 0.0;
        for leg in roster.legs() {
            let count = assignment_count.get(leg.id.as_str()).copied().unwrap_or(0);
            if count == 0 {
                score += self.uncovered_penalty;
            } else if count > 1 {
                score += self.overcoverage_penalty * (count - 1) as f64;
            }
        }
        score
    }
}

// ── RestQualityObjective ──────────────────────────────────────────────────────

/// Penalises rest periods that are close to the legal minimum.
///
/// A rest period that exactly meets the minimum rest requirement is
/// considered low quality.  The penalty decreases as the rest period
/// increases above the minimum.
///
/// # Metric
///
/// For each rest period of duration `r` minutes and minimum `m` minutes:
///
/// ```text
/// penalty = max(0, target_rest - r) / target_rest
/// ```
///
/// where `target_rest` is the desired rest duration (default: 720 min / 12 h).
/// The total score is the sum of penalties across all rest periods.
pub struct RestQualityObjective {
    /// Minimum legal rest in minutes.  Default: 600 (10 h).
    pub minimum_rest_minutes: f64,
    /// Target rest in minutes.  Default: 720 (12 h).
    pub target_rest_minutes: f64,
}

impl Default for RestQualityObjective {
    fn default() -> Self {
        Self {
            minimum_rest_minutes: 600.0,
            target_rest_minutes: 720.0,
        }
    }
}

impl SchedulingObjective for RestQualityObjective {
    fn objective_id(&self) -> &str {
        "rest_quality"
    }

    fn objective_name(&self) -> &str {
        "Rest Quality"
    }

    fn evaluate(&self, roster: &Roster) -> f64 {
        let mut score = 0.0;

        for rotation in roster.rotations() {
            for pairing in rotation.pairings().iter() {
                let duties = pairing.duties();
                for window in duties.windows(2) {
                    let rest_start = window[0].end();
                    let rest_end = window[1].start();
                    let rest_minutes = (rest_end - rest_start).num_minutes() as f64;
                    let shortfall = (self.target_rest_minutes - rest_minutes).max(0.0);
                    score += shortfall / self.target_rest_minutes;
                }
            }
        }

        score
    }
}

// ── CreditedHoursBalanceObjective (Phase 6) ───────────────────────────────────

/// Minimises the variance of total **credited hours** across crew members.
///
/// Per UC-ARCH-001 §6, the fairness objective should balance contractually
/// credited hours (not raw flight-leg counts or block hours), because credited
/// hours are the quantity that determines crew pay and workload perception.
///
/// # Metric
///
/// Score = variance of per-crew credited hours = Σ(hours_i − mean)² / N
///
/// A perfectly balanced roster (every crew member has the same total credited
/// hours) scores `0.0`.
///
/// # Credit policy
///
/// Uses [`GeradCreditPolicy`] by default.  Supply a custom [`CreditPolicy`]
/// via [`CreditedHoursBalanceObjective::with_policy`] to use a different
/// contractual formula.
pub struct CreditedHoursBalanceObjective {
    policy: Box<dyn CreditPolicy>,
    context_base: AirportCode,
}

impl CreditedHoursBalanceObjective {
    /// Create with the default GERAD credit policy.
    pub fn new() -> Self {
        Self {
            policy: Box::new(GeradCreditPolicy::default()),
            context_base: AirportCode::new("YUL"),
        }
    }

    /// Create with a custom credit policy and crew base.
    pub fn with_policy(policy: Box<dyn CreditPolicy>, base: AirportCode) -> Self {
        Self { policy, context_base: base }
    }

    fn make_context(&self) -> CreditContext {
        CreditContext {
            crew_base: self.context_base.clone(),
            home_base: self.context_base.clone(),
            applicable_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        }
    }
}

impl Default for CreditedHoursBalanceObjective {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulingObjective for CreditedHoursBalanceObjective {
    fn objective_id(&self) -> &str {
        "credited_hours_balance"
    }

    fn objective_name(&self) -> &str {
        "Credited Hours Balance"
    }

    fn evaluate(&self, roster: &Roster) -> f64 {
        let ctx = self.make_context();
        let per_crew_hours: Vec<f64> = roster
            .rotations()
            .map(|rotation| {
                rotation
                    .pairings()
                    .iter()
                    .flat_map(|p| p.duties().iter())
                    .map(|d| self.policy.compute(&d.metrics, &ctx).credited_hours)
                    .sum::<f64>()
            })
            .collect();

        if per_crew_hours.is_empty() {
            return 0.0;
        }

        let mean = per_crew_hours.iter().sum::<f64>() / per_crew_hours.len() as f64;
        per_crew_hours
            .iter()
            .map(|&h| (h - mean).powi(2))
            .sum::<f64>()
            / per_crew_hours.len() as f64
    }
}

// ── CreditCostObjective (Phase 7) ─────────────────────────────────────────────

/// Minimises the total monetary credit cost across all crew members.
///
/// Per UC-ARCH-001 §7, the cost objective should use the [`CostModel`] output
/// (not a raw block-hour count), so that pay-rate changes are reflected
/// automatically without touching the objective function.
///
/// # Metric
///
/// Score = Σ DutyCost.credit_cost across all duties in the roster.
///
/// An empty roster scores `0.0`.
///
/// # Credit and cost policies
///
/// Uses [`GeradCreditPolicy`] + [`FlatRateCostModel`] by default.  Supply
/// custom implementations via [`CreditCostObjective::with_models`].
pub struct CreditCostObjective {
    credit_policy: Box<dyn CreditPolicy>,
    cost_model: Box<dyn CostModel>,
    context_base: AirportCode,
}

impl CreditCostObjective {
    /// Create with the default GERAD credit policy and flat-rate cost model.
    pub fn new() -> Self {
        Self {
            credit_policy: Box::new(GeradCreditPolicy::default()),
            cost_model: Box::new(FlatRateCostModel::default()),
            context_base: AirportCode::new("YUL"),
        }
    }

    /// Create with custom credit policy, cost model, and crew base.
    pub fn with_models(
        credit_policy: Box<dyn CreditPolicy>,
        cost_model: Box<dyn CostModel>,
        base: AirportCode,
    ) -> Self {
        Self { credit_policy, cost_model, context_base: base }
    }

    fn make_credit_context(&self) -> CreditContext {
        CreditContext {
            crew_base: self.context_base.clone(),
            home_base: self.context_base.clone(),
            applicable_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        }
    }

    fn make_cost_context(&self) -> CostContext {
        CostContext {
            crew_base: self.context_base.clone(),
            seniority_band: 0,
        }
    }
}

impl Default for CreditCostObjective {
    fn default() -> Self {
        Self::new()
    }
}

impl SchedulingObjective for CreditCostObjective {
    fn objective_id(&self) -> &str {
        "credit_cost"
    }

    fn objective_name(&self) -> &str {
        "Credit Cost"
    }

    fn evaluate(&self, roster: &Roster) -> f64 {
        let credit_ctx = self.make_credit_context();
        let cost_ctx = self.make_cost_context();

        roster
            .rotations()
            .flat_map(|rotation| {
                rotation
                    .pairings()
                    .iter()
                    .flat_map(|p| p.duties().iter())
                    .map(|d| {
                        let credit = self.credit_policy.compute(&d.metrics, &credit_ctx);
                        self.cost_model.compute_cost(&credit, &cost_ctx).credit_cost
                    })
                    .collect::<Vec<_>>()
            })
            .sum()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::test_helpers::*;

    fn make_balanced_roster() -> Roster {
        // Two rotations, each with 2 legs — perfectly balanced.
        let d1a = make_duty("D1a", vec![make_leg("L1a", "LHR", "CDG", 8, 10)]);
        let d1b = make_duty("D1b", vec![make_leg("L1b", "CDG", "LHR", 22, 24)]);
        let d2a = make_duty("D2a", vec![make_leg("L2a", "LHR", "CDG", 8, 10)]);
        let d2b = make_duty("D2b", vec![make_leg("L2b", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1a, d1b]);
        let p2 = make_pairing("P2", "LHR", vec![d2a, d2b]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        make_roster(vec![], vec![r1, r2])
    }

    fn make_unbalanced_roster() -> Roster {
        // C1 has 3 legs, C2 has 1 leg — unbalanced.
        let d1a = make_duty("D1a", vec![
            make_leg("L1a", "LHR", "CDG", 8, 10),
            make_leg("L1b", "CDG", "AMS", 11, 13),
        ]);
        let d1b = make_duty("D1b", vec![make_leg("L1c", "AMS", "LHR", 22, 24)]);
        let d2a = make_duty("D2a", vec![make_leg("L2a", "LHR", "CDG", 8, 10)]);
        let d2b = make_duty("D2b", vec![make_leg("L2b", "CDG", "LHR", 22, 24)]);
        let p1 = make_pairing("P1", "LHR", vec![d1a, d1b]);
        let p2 = make_pairing("P2", "LHR", vec![d2a, d2b]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        make_roster(vec![], vec![r1, r2])
    }

    // ── WorkloadBalanceObjective ──────────────────────────────────────────────

    #[test]
    fn balanced_roster_scores_zero() {
        let roster = make_balanced_roster();
        let obj = WorkloadBalanceObjective;
        assert!((obj.evaluate(&roster) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn unbalanced_roster_scores_positive() {
        let roster = make_unbalanced_roster();
        let obj = WorkloadBalanceObjective;
        assert!(obj.evaluate(&roster) > 0.0);
    }

    #[test]
    fn empty_roster_workload_balance_zero() {
        let roster = make_roster(vec![], vec![]);
        let obj = WorkloadBalanceObjective;
        assert_eq!(obj.evaluate(&roster), 0.0);
    }

    // ── CoverageCostObjective ─────────────────────────────────────────────────

    #[test]
    fn full_coverage_scores_zero() {
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 22, 24);
        let d1 = make_duty("D1", vec![l1.clone()]);
        let d2 = make_duty("D2", vec![l2.clone()]);
        let p = make_pairing("P1", "LHR", vec![d1, d2]);
        let r = make_rotation("R1", "C1", vec![p]);
        let roster = make_roster(vec![l1, l2], vec![r]);
        let obj = CoverageCostObjective::default();
        assert_eq!(obj.evaluate(&roster), 0.0);
    }

    #[test]
    fn uncovered_leg_adds_penalty() {
        let l1 = make_leg("L1", "LHR", "CDG", 8, 10);
        let l2 = make_leg("L2", "CDG", "LHR", 22, 24);
        let l_uncovered = make_leg("L3", "LHR", "AMS", 12, 14);
        let d1 = make_duty("D1", vec![l1.clone()]);
        let d2 = make_duty("D2", vec![l2.clone()]);
        let p = make_pairing("P1", "LHR", vec![d1, d2]);
        let r = make_rotation("R1", "C1", vec![p]);
        let roster = make_roster(vec![l1, l2, l_uncovered], vec![r]);
        let obj = CoverageCostObjective::default();
        assert!((obj.evaluate(&roster) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn empty_roster_coverage_zero() {
        let roster = make_roster(vec![], vec![]);
        let obj = CoverageCostObjective::default();
        assert_eq!(obj.evaluate(&roster), 0.0);
    }

    // ── RestQualityObjective ──────────────────────────────────────────────────

    #[test]
    fn generous_rest_scores_zero() {
        // 14-hour rest between duties — above 12h target → score 0.
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 0, 2)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 16, 18)]);
        let p = make_pairing("P1", "LHR", vec![d1, d2]);
        let r = make_rotation("R1", "C1", vec![p]);
        let roster = make_roster(vec![], vec![r]);
        let obj = RestQualityObjective::default();
        assert_eq!(obj.evaluate(&roster), 0.0);
    }

    #[test]
    fn short_rest_scores_positive() {
        // 11-hour rest — below 12h target → positive score.
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 0, 2)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 13, 15)]);
        let p = make_pairing("P1", "LHR", vec![d1, d2]);
        let r = make_rotation("R1", "C1", vec![p]);
        let roster = make_roster(vec![], vec![r]);
        let obj = RestQualityObjective::default();
        assert!(obj.evaluate(&roster) > 0.0);
    }

    #[test]
    fn objective_ids_are_stable() {
        assert_eq!(WorkloadBalanceObjective.objective_id(), "workload_balance");
        assert_eq!(CoverageCostObjective::default().objective_id(), "coverage_cost");
        assert_eq!(RestQualityObjective::default().objective_id(), "rest_quality");
        assert_eq!(CreditedHoursBalanceObjective::default().objective_id(), "credited_hours_balance");
        assert_eq!(CreditCostObjective::default().objective_id(), "credit_cost");
    }

    // ── CreditedHoursBalanceObjective ─────────────────────────────────────────

    #[test]
    fn credited_hours_balance_empty_roster_scores_zero() {
        let roster = make_roster(vec![], vec![]);
        let obj = CreditedHoursBalanceObjective::default();
        assert_eq!(obj.evaluate(&roster), 0.0);
    }

    #[test]
    fn credited_hours_balance_equal_duties_scores_zero() {
        // Two crew members, each with one identical 2h round-trip duty → variance = 0.
        // Pairings must end at their base (LHR), so use LHR→CDG→LHR.
        let d1 = make_duty("D1", vec![
            make_leg("L1a", "LHR", "CDG", 8, 10),
            make_leg("L1b", "CDG", "LHR", 11, 13),
        ]);
        let d2 = make_duty("D2", vec![
            make_leg("L2a", "LHR", "CDG", 8, 10),
            make_leg("L2b", "CDG", "LHR", 11, 13),
        ]);
        let p1 = make_pairing("P1", "LHR", vec![d1]);
        let p2 = make_pairing("P2", "LHR", vec![d2]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        let roster = make_roster(vec![], vec![r1, r2]);
        let obj = CreditedHoursBalanceObjective::default();
        assert!((obj.evaluate(&roster) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn credited_hours_balance_unequal_duties_scores_positive() {
        // C1 has a 4h round-trip duty, C2 has a 2h round-trip duty → variance > 0.
        let d1 = make_duty("D1", vec![
            make_leg("L1a", "LHR", "CDG", 8, 10),
            make_leg("L1b", "CDG", "LHR", 12, 14), // 4h total block
        ]);
        let d2 = make_duty("D2", vec![
            make_leg("L2a", "LHR", "CDG", 8, 9),
            make_leg("L2b", "CDG", "LHR", 10, 11), // 2h total block
        ]);
        let p1 = make_pairing("P1", "LHR", vec![d1]);
        let p2 = make_pairing("P2", "LHR", vec![d2]);
        let r1 = make_rotation("R1", "C1", vec![p1]);
        let r2 = make_rotation("R2", "C2", vec![p2]);
        let roster = make_roster(vec![], vec![r1, r2]);
        let obj = CreditedHoursBalanceObjective::default();
        assert!(obj.evaluate(&roster) > 0.0);
    }

    #[test]
    fn credited_hours_balance_more_balanced_scores_lower() {
        // Balanced: C1=4h, C2=4h → variance=0.
        // Unbalanced: C1=4h, C2=2h → variance>0.
        let d_bal1 = make_duty("D1", vec![
            make_leg("L1a", "LHR", "CDG", 8, 10),
            make_leg("L1b", "CDG", "LHR", 12, 14),
        ]);
        let d_bal2 = make_duty("D2", vec![
            make_leg("L2a", "LHR", "CDG", 8, 10),
            make_leg("L2b", "CDG", "LHR", 12, 14),
        ]);
        let p_bal1 = make_pairing("P1", "LHR", vec![d_bal1]);
        let p_bal2 = make_pairing("P2", "LHR", vec![d_bal2]);
        let r_bal1 = make_rotation("R1", "C1", vec![p_bal1]);
        let r_bal2 = make_rotation("R2", "C2", vec![p_bal2]);
        let balanced = make_roster(vec![], vec![r_bal1, r_bal2]);

        let d_unb1 = make_duty("D3", vec![
            make_leg("L3a", "LHR", "CDG", 8, 10),
            make_leg("L3b", "CDG", "LHR", 12, 14), // 4h
        ]);
        let d_unb2 = make_duty("D4", vec![
            make_leg("L4a", "LHR", "CDG", 8, 9),
            make_leg("L4b", "CDG", "LHR", 10, 11), // 2h
        ]);
        let p_unb1 = make_pairing("P3", "LHR", vec![d_unb1]);
        let p_unb2 = make_pairing("P4", "LHR", vec![d_unb2]);
        let r_unb1 = make_rotation("R3", "C3", vec![p_unb1]);
        let r_unb2 = make_rotation("R4", "C4", vec![p_unb2]);
        let unbalanced = make_roster(vec![], vec![r_unb1, r_unb2]);

        let obj = CreditedHoursBalanceObjective::default();
        assert!(obj.evaluate(&balanced) < obj.evaluate(&unbalanced));
    }

    // ── CreditCostObjective ───────────────────────────────────────────────────

    #[test]
    fn credit_cost_empty_roster_scores_zero() {
        let roster = make_roster(vec![], vec![]);
        let obj = CreditCostObjective::default();
        assert_eq!(obj.evaluate(&roster), 0.0);
    }

    #[test]
    fn credit_cost_single_duty_is_positive() {
        // Round-trip duty: LHR→CDG→LHR (2h block each leg = 4h total).
        let d = make_duty("D1", vec![
            make_leg("L1a", "LHR", "CDG", 8, 10),
            make_leg("L1b", "CDG", "LHR", 11, 13),
        ]);
        let p = make_pairing("P1", "LHR", vec![d]);
        let r = make_rotation("R1", "C1", vec![p]);
        let roster = make_roster(vec![], vec![r]);
        let obj = CreditCostObjective::default();
        assert!(obj.evaluate(&roster) > 0.0);
    }

    #[test]
    fn credit_cost_more_duties_costs_more() {
        // One round-trip duty vs two round-trip duties — two should cost more.
        let d1 = make_duty("D1", vec![
            make_leg("L1a", "LHR", "CDG", 8, 10),
            make_leg("L1b", "CDG", "LHR", 11, 13),
        ]);
        let d2 = make_duty("D2", vec![
            make_leg("L2a", "LHR", "CDG", 14, 16),
            make_leg("L2b", "CDG", "LHR", 17, 19),
        ]);
        let p_one = make_pairing("P1", "LHR", vec![d1.clone()]);
        let p_two = make_pairing("P2", "LHR", vec![d1, d2]);
        let r_one = make_rotation("R1", "C1", vec![p_one]);
        let r_two = make_rotation("R2", "C2", vec![p_two]);
        let roster_one = make_roster(vec![], vec![r_one]);
        let roster_two = make_roster(vec![], vec![r_two]);
        let obj = CreditCostObjective::default();
        assert!(obj.evaluate(&roster_two) > obj.evaluate(&roster_one));
    }
}
