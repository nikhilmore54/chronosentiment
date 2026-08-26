//! Cost Model — Layer 1b (monetary cost layer).
//!
//! This module translates [`DutyCredit`] (contractual hours) into monetary
//! cost.  It is intentionally separate from the credit engine so that:
//!
//! - The credit formula can be changed without touching pay-rate logic.
//! - Pay rates can be updated (e.g. annual CBA revision) without touching the
//!   credit formula.
//! - The optimiser can swap cost models (e.g. flat rate vs. seniority-banded)
//!   without recompiling the credit engine.
//!
//! # Architecture (UC-ARCH-001)
//!
//! ```text
//! DutyCredit  ──►  CostModel  ──►  DutyCost
//!                      │
//!                 CostContext
//!                 (crew category, seniority, …)
//! ```
//!
//! [`CostModel`] is a **pure function** (same determinism contract as
//! [`CreditPolicy`]): identical inputs always produce identical outputs.
//!
//! [`CreditPolicy`]: super::credit::CreditPolicy

use serde::{Deserialize, Serialize};

use super::credit::DutyCredit;
use super::flight::AirportCode;

// ── Context ───────────────────────────────────────────────────────────────────

/// Contextual information supplied to [`CostModel::compute_cost`].
///
/// Carries crew-member and scheduling context that the cost model may need
/// beyond the raw [`DutyCredit`].  Fields not used by a given implementation
/// are ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostContext {
    /// The airport at which the crew member is currently based.
    pub crew_base: AirportCode,
    /// Seniority band index (0 = most junior).  Used by banded pay models.
    pub seniority_band: u32,
    // Production extensions (add without breaking existing callers):
    // pub crew_category: Option<CrewCategory>,
    // pub is_augmented_crew: bool,
    // pub applicable_cba_year: u32,
}

// ── Duty cost ─────────────────────────────────────────────────────────────────

/// The monetary cost of a single duty, as computed by a [`CostModel`].
///
/// Kept separate from [`DutyCredit`] so that the credit formula and the pay
/// rate can evolve independently.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DutyCost {
    /// Total monetary cost of this duty (currency units; calibrate to CBA).
    pub credit_cost: f64,
    /// The effective pay rate per credited hour used for this duty.
    pub pay_rate: f64,
}

// ── Cost model trait ──────────────────────────────────────────────────────────

/// A monetary cost formula for a single duty.
///
/// # Determinism contract
///
/// Implementations **must** be pure functions.  The same `(credit, context)`
/// pair must always produce the same [`DutyCost`].  No mutable state, no
/// external I/O, no randomness.
pub trait CostModel: Send + Sync {
    /// Compute the monetary cost of `credit` under `context`.
    fn compute_cost(&self, credit: &DutyCredit, context: &CostContext) -> DutyCost;
}

// ── Flat-rate cost model ──────────────────────────────────────────────────────

/// A simple flat pay-rate cost model: `cost = credited_hours × pay_rate`.
///
/// Suitable for GERAD G-2014-22 where all crew members are paid at the same
/// rate.  For seniority-banded or category-differentiated pay, implement a
/// custom [`CostModel`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlatRateCostModel {
    /// Pay rate per credited hour (currency units / hour).
    pub pay_rate_per_hour: f64,
}

impl Default for FlatRateCostModel {
    fn default() -> Self {
        Self {
            pay_rate_per_hour: 100.0, // placeholder; calibrate from creditedHours
        }
    }
}

impl CostModel for FlatRateCostModel {
    fn compute_cost(&self, credit: &DutyCredit, _ctx: &CostContext) -> DutyCost {
        DutyCost {
            credit_cost: credit.credited_hours * self.pay_rate_per_hour,
            pay_rate: self.pay_rate_per_hour,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::credit::{CreditComponents, DutyCredit};
    use super::*;

    fn make_credit(credited_hours: f64) -> DutyCredit {
        DutyCredit {
            credited_hours,
            credit_cost: 0.0, // not used by CostModel
            components: CreditComponents {
                block_credit: credited_hours,
                deadhead_credit: 0.0,
                layover_credit: 0.0,
                premium_credit: 0.0,
                minimum_guarantee_applied: false,
            },
        }
    }

    fn make_context() -> CostContext {
        CostContext {
            crew_base: AirportCode::new("YUL"),
            seniority_band: 0,
        }
    }

    #[test]
    fn flat_rate_cost_is_hours_times_rate() {
        let model = FlatRateCostModel {
            pay_rate_per_hour: 150.0,
        };
        let credit = make_credit(4.0);
        let ctx = make_context();
        let cost = model.compute_cost(&credit, &ctx);

        assert!(
            (cost.credit_cost - 600.0).abs() < 1e-9,
            "expected 600.0, got {}",
            cost.credit_cost
        );
        assert_eq!(cost.pay_rate, 150.0);
    }

    #[test]
    fn flat_rate_default_pay_rate_is_placeholder() {
        let model = FlatRateCostModel::default();
        assert_eq!(model.pay_rate_per_hour, 100.0);
    }

    #[test]
    fn flat_rate_zero_hours_gives_zero_cost() {
        let model = FlatRateCostModel::default();
        let credit = make_credit(0.0);
        let ctx = make_context();
        let cost = model.compute_cost(&credit, &ctx);
        assert_eq!(cost.credit_cost, 0.0);
    }

    #[test]
    fn flat_rate_fractional_hours() {
        let model = FlatRateCostModel {
            pay_rate_per_hour: 200.0,
        };
        let credit = make_credit(1.5); // 1h 30m
        let ctx = make_context();
        let cost = model.compute_cost(&credit, &ctx);
        assert!(
            (cost.credit_cost - 300.0).abs() < 1e-9,
            "expected 300.0, got {}",
            cost.credit_cost
        );
    }
}
