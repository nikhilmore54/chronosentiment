//! AllocationEngine v0 — deterministic, transparent portfolio sizing.
//!
//! ## Responsibility
//!
//! The allocation engine answers two questions:
//!   1. What action should the user take? (ADD / HOLD / NO_ACTION)
//!   2. How much INR can actually be allocated this week?
//!
//! Action and allocation amount are **separate concepts**. A valid market
//! opportunity (ADD) may have a reduced or zero allocation_inr if capital
//! is constrained. The action still reflects the market decision.
//!
//! ## What it does NOT do
//!
//! - It does NOT alter C3-002 direction (LONG/SHORT/NO_TRADE).
//! - It does NOT alter Coralys target_pct, risk_pct, or maximum_hold_sessions.
//! - It does NOT perform market intelligence.
//! - It does NOT learn or adapt.
//!
//! ## Three separate responsibilities (invariant)
//!
//! ```text
//! Market intelligence    → C3-002
//! Execution intelligence → Coralys v0
//! Portfolio constraint   → AllocationEngine v0  ← this module
//! ```
//!
//! ## Action semantics (v0, frozen)
//!
//! | C3-002 direction | Existing holding | Exposure state   | Action     |
//! |------------------|------------------|------------------|------------|
//! | NO_TRADE         | none             | —                | NO_ACTION  |
//! | NO_TRADE         | present          | —                | HOLD       |
//! | LONG / SHORT     | any              | overexposed      | HOLD       |
//! | LONG / SHORT     | any              | not overexposed  | ADD        |
//!
//! ## Allocation amount (v0, frozen)
//!
//! When action = ADD:
//!   base     = min(max_single_instrument_inr, available_cash)
//!   amount   = round_down_to_nearest_100(base)   [may be ₹0]
//!
//! Capital constraints (insufficient cash, budget exhausted) reduce
//! `allocation_inr` but do NOT change the action to AVOID.
//! AVOID is reserved for future policy-level exclusions (e.g. instrument
//! on a restricted list, direction conflicts with a hard portfolio rule).
//!
//! This separation means the UI can show:
//!   "ADD INFY — Recommended ₹2,000 | Available this week: ₹500 | Allocate ₹500"
//! rather than incorrectly showing AVOID when the market signal is valid.

use super::portfolio_context::PortfolioContext;
use super::recommendation::{PortfolioAllocationRequest, PortfolioRecommendation, RecommendationAction};
use super::user_profile::UserProfile;

/// Version string for this allocation engine. Embedded in every recommendation for provenance.
pub const ALLOCATION_ENGINE_VERSION: &str = "allocation-engine-v0";

/// Minimum allocation in INR. Allocations below this are rounded to AVOID.
/// Frozen design parameter for v0.
pub const MIN_ALLOCATION_INR: f64 = 100.0;

/// If existing exposure in an instrument exceeds this multiple of the weekly budget,
/// the engine emits AVOID (overexposed). Frozen design parameter for v0.
pub const MAX_EXPOSURE_MULTIPLIER: f64 = 5.0;

/// Allocation rounding granularity in INR. Allocations are rounded down to the
/// nearest multiple of this value. Frozen design parameter for v0.
pub const ALLOCATION_ROUNDING_INR: f64 = 100.0;

/// AllocationEngine v0.
///
/// Stateless. All inputs are explicit. No hidden state.
/// Construct once and call `allocate` for each request.
pub struct AllocationEngine;

impl AllocationEngine {
    pub fn new() -> Self {
        AllocationEngine
    }

    /// Produce a PortfolioRecommendation for a single instrument.
    ///
    /// The engine reads:
    ///   - `request`  — C3-002 direction + Coralys execution parameters (passed through unchanged)
    ///   - `profile`  — user budget and risk tolerance
    ///   - `context`  — existing holdings and available cash
    ///
    /// It writes:
    ///   - `action`         — ADD / HOLD / AVOID / NO_ACTION
    ///   - `allocation_inr` — amount to deploy (0 for non-ADD actions)
    ///   - `rationale`      — human-readable explanation of the sizing decision
    ///
    /// All Coralys parameters (target_pct, risk_pct, etc.) are passed through unchanged.
    ///
    /// ## Action semantics (v0, frozen)
    ///
    /// | C3-002 direction | Existing holding | Cash / exposure | Action     |
    /// |------------------|------------------|-----------------|------------|
    /// | NO_TRADE         | none             | —               | NO_ACTION  |
    /// | NO_TRADE         | present          | —               | HOLD       |
    /// | LONG / SHORT     | any              | overexposed     | HOLD       |
    /// | LONG / SHORT     | any              | insufficient    | AVOID      |
    /// | LONG / SHORT     | any              | adequate        | ADD        |
    ///
    /// REDUCE is reserved for a future version when C3-002 flips direction
    /// against an existing position (e.g. C3-002 = SHORT, user holds LONG).
    pub fn allocate(
        &self,
        request: &PortfolioAllocationRequest,
        profile: &UserProfile,
        context: &PortfolioContext,
    ) -> PortfolioRecommendation {
        let existing_exposure = context.exposure_for(&request.instrument);
        let has_existing_holding = existing_exposure > 0.0;

        // Step 1: NO_TRADE direction.
        // If the user holds the instrument, HOLD (keep it, don't add).
        // If the user has no position, NO_ACTION (nothing to do).
        if request.c3_002_direction == "NO_TRADE" {
            if has_existing_holding {
                return self.make_recommendation(
                    request,
                    RecommendationAction::Hold,
                    0.0,
                    format!(
                        "C3-002 direction is NO_TRADE. Existing holding in {} (₹{:.0}): maintain position.",
                        request.instrument, existing_exposure
                    ),
                );
            } else {
                return self.make_recommendation(
                    request,
                    RecommendationAction::NoAction,
                    0.0,
                    "C3-002 direction is NO_TRADE. No existing position. No action.".into(),
                );
            }
        }

        // Step 2: Overexposure check → HOLD (already adequately positioned, don't add more).
        let max_exposure = profile.weekly_investment_inr * MAX_EXPOSURE_MULTIPLIER;
        if existing_exposure >= max_exposure {
            return self.make_recommendation(
                request,
                RecommendationAction::Hold,
                0.0,
                format!(
                    "Existing exposure in {} (₹{:.0}) is at or above {:.0}× weekly budget (₹{:.0}). \
                     Position is adequate — hold, do not add.",
                    request.instrument,
                    existing_exposure,
                    MAX_EXPOSURE_MULTIPLIER,
                    profile.weekly_investment_inr,
                ),
            );
        }

        // Step 3: Compute allocation amount.
        // Capital constraints (insufficient cash, budget exhausted) reduce allocation_inr
        // but do NOT change the action to AVOID. The market signal (ADD) remains valid.
        // AVOID is reserved for future policy-level exclusions.
        let max_single = profile.max_single_instrument_inr();
        let base = max_single.min(context.available_cash_inr.max(0.0));
        let allocation = round_down(base, ALLOCATION_ROUNDING_INR);

        let rationale = if allocation >= MIN_ALLOCATION_INR {
            format!(
                "ADD ₹{:.0} to {} ({}). \
                 Weekly budget: ₹{:.0}, risk tolerance: {} (×{:.2}). \
                 Available cash: ₹{:.0}. Existing exposure: ₹{:.0}.",
                allocation,
                request.instrument,
                request.c3_002_direction,
                profile.weekly_investment_inr,
                profile.risk_tolerance.label(),
                profile.risk_tolerance.sizing_factor(),
                context.available_cash_inr,
                existing_exposure,
            )
        } else {
            format!(
                "ADD {} ({}) — signal valid. \
                 Allocation deferred: available cash ₹{:.0} is below rounding threshold ₹{:.0}. \
                 Allocate when cash is available.",
                request.instrument,
                request.c3_002_direction,
                context.available_cash_inr,
                MIN_ALLOCATION_INR,
            )
        };

        self.make_recommendation(request, RecommendationAction::Add, allocation, rationale)
    }

    fn make_recommendation(
        &self,
        request: &PortfolioAllocationRequest,
        action: RecommendationAction,
        allocation_inr: f64,
        rationale: String,
    ) -> PortfolioRecommendation {
        PortfolioRecommendation {
            instrument: request.instrument.clone(),
            action,
            allocation_inr,
            c3_002_direction: request.c3_002_direction.clone(),
            entry_price: request.entry_price,
            target_pct: request.target_pct,
            target_price: request.target_price,
            risk_pct: request.risk_pct,
            risk_boundary: request.risk_boundary,
            maximum_hold_sessions: request.maximum_hold_sessions,
            rationale,
            decision_id: request.decision_id.clone(),
            execution_intent_id: request.execution_intent_id.clone(),
            allocation_engine_version: ALLOCATION_ENGINE_VERSION.to_string(),
        }
    }
}

impl Default for AllocationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Round `value` down to the nearest multiple of `granularity`.
fn round_down(value: f64, granularity: f64) -> f64 {
    (value / granularity).floor() * granularity
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::portfolio_context::PortfolioContext;
    use crate::product::user_profile::{InvestmentHorizon, RiskTolerance, UserProfile};
    use crate::product::recommendation::PortfolioAllocationRequest;
    use std::collections::HashMap;

    fn moderate_profile() -> UserProfile {
        UserProfile {
            user_id: "user-001".into(),
            weekly_investment_inr: 5000.0,
            risk_tolerance: RiskTolerance::Moderate,   // sizing_factor = 0.75 → max = 3750
            investment_horizon: InvestmentHorizon::MediumTerm,
        }
    }

    fn context_with_cash(cash: f64) -> PortfolioContext {
        PortfolioContext {
            as_of: "2026-08-16T09:15:00+05:30".into(),
            available_cash_inr: cash,
            holdings: vec![],
            existing_exposure_inr: HashMap::new(),
        }
    }

    fn context_with_exposure(cash: f64, instrument: &str, exposure: f64) -> PortfolioContext {
        let mut map = HashMap::new();
        map.insert(instrument.to_string(), exposure);
        PortfolioContext {
            as_of: "2026-08-16T09:15:00+05:30".into(),
            available_cash_inr: cash,
            holdings: vec![],
            existing_exposure_inr: map,
        }
    }

    fn infy_long_request() -> PortfolioAllocationRequest {
        PortfolioAllocationRequest {
            instrument: "INFY.NS".into(),
            c3_002_direction: "LONG".into(),
            entry_price: 1076.30,
            target_pct: 0.062,
            target_price: 1143.23,
            risk_pct: 0.031,
            risk_boundary: 1042.94,
            maximum_hold_sessions: 20,
            decision_rationale: "Bullish trend.".into(),
            decision_id: "dec-001".into(),
            execution_intent_id: "exec-001".into(),
        }
    }

    fn no_trade_request() -> PortfolioAllocationRequest {
        PortfolioAllocationRequest {
            instrument: "INFY.NS".into(),
            c3_002_direction: "NO_TRADE".into(),
            entry_price: 1076.30,
            target_pct: 0.05,
            target_price: 1130.12,
            risk_pct: 0.025,
            risk_boundary: 1049.39,
            maximum_hold_sessions: 20,
            decision_rationale: "No signal.".into(),
            decision_id: "dec-002".into(),
            execution_intent_id: "exec-002".into(),
        }
    }

    #[test]
    fn no_trade_with_no_holding_produces_no_action() {
        // NO_TRADE + no existing position → NO_ACTION (nothing to do)
        let engine = AllocationEngine::new();
        let rec = engine.allocate(&no_trade_request(), &moderate_profile(), &context_with_cash(10000.0));
        assert_eq!(rec.action, RecommendationAction::NoAction);
        assert!((rec.allocation_inr - 0.0).abs() < 1e-9);
    }

    #[test]
    fn no_trade_with_existing_holding_produces_hold() {
        // NO_TRADE + existing position → HOLD (keep what you have, don't add)
        let engine = AllocationEngine::new();
        let ctx = context_with_exposure(10000.0, "INFY.NS", 8000.0);
        let rec = engine.allocate(&no_trade_request(), &moderate_profile(), &ctx);
        assert_eq!(rec.action, RecommendationAction::Hold);
        assert!((rec.allocation_inr - 0.0).abs() < 1e-9);
        assert!(rec.rationale.contains("maintain position"));
    }

    #[test]
    fn insufficient_cash_still_produces_add_with_zero_allocation() {
        // Capital constraint does NOT change the action to AVOID.
        // The market signal (ADD) remains valid; allocation_inr is ₹0 (deferred).
        let engine = AllocationEngine::new();
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &context_with_cash(50.0));
        assert_eq!(rec.action, RecommendationAction::Add,
            "insufficient cash must not change ADD to AVOID");
        assert!((rec.allocation_inr - 0.0).abs() < 1e-9,
            "allocation must be ₹0 when cash is below rounding threshold");
        assert!(rec.rationale.contains("deferred"),
            "rationale must explain deferral: {}", rec.rationale);
    }

    #[test]
    fn overexposed_instrument_produces_hold() {
        // MAX_EXPOSURE_MULTIPLIER = 5.0; weekly = 5000 → max_exposure = 25000
        // Overexposed → HOLD (position is adequate, don't add more)
        let engine = AllocationEngine::new();
        let ctx = context_with_exposure(10000.0, "INFY.NS", 25000.0);
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &ctx);
        assert_eq!(rec.action, RecommendationAction::Hold);
        assert!((rec.allocation_inr - 0.0).abs() < 1e-9);
        assert!(rec.rationale.contains("adequate"));
    }

    #[test]
    fn normal_case_produces_add_with_correct_allocation() {
        // Moderate profile: max_single = 5000 × 0.75 = 3750
        // cash = 10000 → base = min(3750, 10000) = 3750
        // round_down(3750, 100) = 3700 (3750 / 100 = 37.5 → floor = 37 → 3700)
        let engine = AllocationEngine::new();
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &context_with_cash(10000.0));
        assert_eq!(rec.action, RecommendationAction::Add);
        assert!((rec.allocation_inr - 3700.0).abs() < 1e-6,
            "expected 3700, got {}", rec.allocation_inr);
    }

    #[test]
    fn cash_constrained_allocation_uses_available_cash() {
        // cash = 1500 < max_single (3750) → base = 1500
        // round_down(1500, 100) = 1500
        let engine = AllocationEngine::new();
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &context_with_cash(1500.0));
        assert_eq!(rec.action, RecommendationAction::Add);
        assert!((rec.allocation_inr - 1500.0).abs() < 1e-6,
            "expected 1500, got {}", rec.allocation_inr);
    }

    #[test]
    fn coralys_parameters_are_passed_through_unchanged() {
        let engine = AllocationEngine::new();
        let req = infy_long_request();
        let rec = engine.allocate(&req, &moderate_profile(), &context_with_cash(10000.0));
        assert!((rec.target_pct - req.target_pct).abs() < 1e-9, "target_pct must be unchanged");
        assert!((rec.risk_pct - req.risk_pct).abs() < 1e-9, "risk_pct must be unchanged");
        assert_eq!(rec.maximum_hold_sessions, req.maximum_hold_sessions, "max hold must be unchanged");
        assert_eq!(rec.c3_002_direction, req.c3_002_direction, "direction must be unchanged");
    }

    #[test]
    fn provenance_ids_are_preserved() {
        let engine = AllocationEngine::new();
        let req = infy_long_request();
        let rec = engine.allocate(&req, &moderate_profile(), &context_with_cash(10000.0));
        assert_eq!(rec.decision_id, req.decision_id);
        assert_eq!(rec.execution_intent_id, req.execution_intent_id);
        assert_eq!(rec.allocation_engine_version, ALLOCATION_ENGINE_VERSION);
    }

    #[test]
    fn allocation_engine_version_is_embedded() {
        let engine = AllocationEngine::new();
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &context_with_cash(10000.0));
        assert_eq!(rec.allocation_engine_version, ALLOCATION_ENGINE_VERSION);
    }

    #[test]
    fn round_down_to_nearest_100() {
        assert!((round_down(3750.0, 100.0) - 3700.0).abs() < 1e-9);
        assert!((round_down(3700.0, 100.0) - 3700.0).abs() < 1e-9);
        assert!((round_down(150.0, 100.0) - 100.0).abs() < 1e-9);
        assert!((round_down(99.0, 100.0) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn just_below_rounding_threshold_produces_add_with_zero_allocation() {
        // cash = 99 → base = 99 → round_down(99, 100) = 0 < MIN_ALLOCATION_INR
        // Action is still ADD (signal valid); allocation_inr = ₹0 (deferred).
        let engine = AllocationEngine::new();
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &context_with_cash(99.0));
        assert_eq!(rec.action, RecommendationAction::Add,
            "sub-threshold cash must not change ADD to AVOID");
        assert!((rec.allocation_inr - 0.0).abs() < 1e-9,
            "allocation must be ₹0 when rounded amount is below threshold");
    }

    #[test]
    fn exactly_at_overexposure_threshold_produces_hold() {
        // max_exposure = 5000 × 5.0 = 25000; exposure = 25000 → HOLD (position adequate)
        let engine = AllocationEngine::new();
        let ctx = context_with_exposure(10000.0, "INFY.NS", 25000.0);
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &ctx);
        assert_eq!(rec.action, RecommendationAction::Hold);
    }

    #[test]
    fn just_below_overexposure_threshold_produces_add() {
        // exposure = 24999 < 25000 → should proceed to ADD
        let engine = AllocationEngine::new();
        let ctx = context_with_exposure(10000.0, "INFY.NS", 24999.0);
        let rec = engine.allocate(&infy_long_request(), &moderate_profile(), &ctx);
        assert_eq!(rec.action, RecommendationAction::Add);
    }
}