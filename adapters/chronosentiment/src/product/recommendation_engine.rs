//! PortfolioRecommendationEngine v0 — orchestrates all product contracts.
//!
//! This is the top-level entry point for the product layer.
//! It takes:
//!   - A list of PortfolioAllocationRequests (from C3-002 + Coralys)
//!   - A UserProfile
//!   - A PortfolioContext
//!
//! And produces:
//!   - Vec<PortfolioRecommendation>
//!
//! The engine has no knowledge of:
//!   - Next.js, REST, JSON serialization format
//!   - Mobile, CLI, or any other consumer
//!   - Database persistence
//!   - Market data feeds
//!
//! It is a pure domain function: inputs → outputs.
//! Consumers (API adapters, UI, notifications) are responsible for
//! serializing and presenting the output.
//!
//! ## Horizon filtering
//!
//! The engine filters out requests whose `maximum_hold_sessions` exceeds
//! the user's `investment_horizon.max_sessions()`. These are emitted as
//! NO_ACTION with a rationale explaining the horizon mismatch.
//!
//! ## Budget enforcement
//!
//! The engine enforces the weekly budget across all instruments:
//! total allocated INR across all ADD recommendations must not exceed
//! `profile.weekly_investment_inr`.
//!
//! Instruments are processed in the order provided. Once the budget is
//! exhausted, remaining LONG/SHORT requests are emitted as ADD with allocation_inr = ₹0.
//! Capital constraints do NOT produce AVOID — the market signal remains valid.

use super::allocation_engine::AllocationEngine;
use super::portfolio_context::PortfolioContext;
use super::recommendation::{PortfolioAllocationRequest, PortfolioRecommendation, RecommendationAction};
use super::user_profile::UserProfile;

/// Version of the recommendation engine.
pub const RECOMMENDATION_ENGINE_VERSION: &str = "portfolio-recommendation-engine-v0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecommendationEngineError {
    InvalidUserProfile(String),
    InvalidPortfolioContext(String),
    EmptyRequests,
}

impl std::fmt::Display for RecommendationEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendationEngineError::InvalidUserProfile(msg) => {
                write!(f, "invalid user profile: {msg}")
            }
            RecommendationEngineError::InvalidPortfolioContext(msg) => {
                write!(f, "invalid portfolio context: {msg}")
            }
            RecommendationEngineError::EmptyRequests => {
                write!(f, "no allocation requests provided")
            }
        }
    }
}

impl std::error::Error for RecommendationEngineError {}

/// PortfolioRecommendationEngine v0.
///
/// Stateless. Construct once and call `recommend` for each cycle.
pub struct PortfolioRecommendationEngine {
    allocation_engine: AllocationEngine,
}

impl PortfolioRecommendationEngine {
    pub fn new() -> Self {
        PortfolioRecommendationEngine {
            allocation_engine: AllocationEngine::new(),
        }
    }

    /// Produce a Vec<PortfolioRecommendation> for all requests in this cycle.
    ///
    /// Processing order:
    /// 1. Validate inputs.
    /// 2. For each request, check horizon compatibility.
    /// 3. Delegate to AllocationEngine for sizing.
    /// 4. Enforce weekly budget cap across all ADD recommendations.
    pub fn recommend(
        &self,
        requests: &[PortfolioAllocationRequest],
        profile: &UserProfile,
        context: &PortfolioContext,
    ) -> Result<Vec<PortfolioRecommendation>, RecommendationEngineError> {
        // Validate inputs.
        profile
            .validate()
            .map_err(|e| RecommendationEngineError::InvalidUserProfile(e.to_string()))?;
        context
            .validate()
            .map_err(|e| RecommendationEngineError::InvalidPortfolioContext(e.to_string()))?;

        if requests.is_empty() {
            return Err(RecommendationEngineError::EmptyRequests);
        }

        let max_sessions = profile.investment_horizon.max_sessions();
        let mut recommendations = Vec::with_capacity(requests.len());
        let mut budget_remaining = profile.weekly_investment_inr;

        for request in requests {
            // Step 1: Horizon filter.
            if request.maximum_hold_sessions > max_sessions {
                recommendations.push(PortfolioRecommendation {
                    instrument: request.instrument.clone(),
                    action: RecommendationAction::NoAction,
                    allocation_inr: 0.0,
                    c3_002_direction: request.c3_002_direction.clone(),
                    entry_price: request.entry_price,
                    target_pct: request.target_pct,
                    target_price: request.target_price,
                    risk_pct: request.risk_pct,
                    risk_boundary: request.risk_boundary,
                    maximum_hold_sessions: request.maximum_hold_sessions,
                    rationale: format!(
                        "Instrument hold period ({} sessions) exceeds user horizon ({} sessions, {}).",
                        request.maximum_hold_sessions,
                        max_sessions,
                        profile.investment_horizon.label(),
                    ),
                    decision_id: request.decision_id.clone(),
                    execution_intent_id: request.execution_intent_id.clone(),
                    allocation_engine_version: super::allocation_engine::ALLOCATION_ENGINE_VERSION.to_string(),
                });
                continue;
            }

            // Step 2: Delegate to AllocationEngine.
            // Pass budget-capped available cash so the engine computes the correct
            // allocation amount. When budget is exhausted, effective_cash = 0 and
            // the engine emits ADD with allocation_inr = ₹0 (deferred).
            // Capital constraints do NOT produce AVOID — the market signal remains valid.
            let effective_cash = context.available_cash_inr.min(budget_remaining);
            let budget_capped_context = PortfolioContext {
                as_of: context.as_of.clone(),
                available_cash_inr: effective_cash,
                holdings: context.holdings.clone(),
                existing_exposure_inr: context.existing_exposure_inr.clone(),
            };

            let rec = self.allocation_engine.allocate(request, profile, &budget_capped_context);

            // Step 4: Deduct from budget if ADD.
            if rec.action == RecommendationAction::Add {
                budget_remaining -= rec.allocation_inr;
                if budget_remaining < 0.0 {
                    budget_remaining = 0.0;
                }
            }

            recommendations.push(rec);
        }

        Ok(recommendations)
    }
}

impl Default for PortfolioRecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::product::portfolio_context::PortfolioContext;
    use crate::product::recommendation::PortfolioAllocationRequest;
    use crate::product::user_profile::{InvestmentHorizon, RiskTolerance, UserProfile};
    use std::collections::HashMap;

    fn moderate_profile() -> UserProfile {
        UserProfile {
            user_id: "user-001".into(),
            weekly_investment_inr: 5000.0,
            risk_tolerance: RiskTolerance::Moderate,
            investment_horizon: InvestmentHorizon::MediumTerm, // max 20 sessions
        }
    }

    fn short_term_profile() -> UserProfile {
        UserProfile {
            user_id: "user-002".into(),
            weekly_investment_inr: 5000.0,
            risk_tolerance: RiskTolerance::Moderate,
            investment_horizon: InvestmentHorizon::ShortTerm, // max 5 sessions
        }
    }

    fn empty_context(cash: f64) -> PortfolioContext {
        PortfolioContext {
            as_of: "2026-08-16T09:15:00+05:30".into(),
            available_cash_inr: cash,
            holdings: vec![],
            existing_exposure_inr: HashMap::new(),
        }
    }

    fn long_request(instrument: &str, sessions: u32) -> PortfolioAllocationRequest {
        PortfolioAllocationRequest {
            instrument: instrument.to_string(),
            c3_002_direction: "LONG".into(),
            entry_price: 1000.0,
            target_pct: 0.062,
            target_price: 1062.0,
            risk_pct: 0.031,
            risk_boundary: 969.0,
            maximum_hold_sessions: sessions,
            decision_rationale: "Bullish.".into(),
            decision_id: format!("dec-{instrument}"),
            execution_intent_id: format!("exec-{instrument}"),
        }
    }

    fn no_trade_request(instrument: &str) -> PortfolioAllocationRequest {
        PortfolioAllocationRequest {
            instrument: instrument.to_string(),
            c3_002_direction: "NO_TRADE".into(),
            entry_price: 1000.0,
            target_pct: 0.05,
            target_price: 1050.0,
            risk_pct: 0.025,
            risk_boundary: 975.0,
            maximum_hold_sessions: 20,
            decision_rationale: "No signal.".into(),
            decision_id: format!("dec-{instrument}"),
            execution_intent_id: format!("exec-{instrument}"),
        }
    }

    #[test]
    fn empty_requests_returns_error() {
        let engine = PortfolioRecommendationEngine::new();
        let result = engine.recommend(&[], &moderate_profile(), &empty_context(10000.0));
        assert_eq!(result, Err(RecommendationEngineError::EmptyRequests));
    }

    #[test]
    fn invalid_profile_returns_error() {
        let engine = PortfolioRecommendationEngine::new();
        let mut bad_profile = moderate_profile();
        bad_profile.user_id = "".into();
        let result = engine.recommend(&[long_request("INFY.NS", 20)], &bad_profile, &empty_context(10000.0));
        assert!(matches!(result, Err(RecommendationEngineError::InvalidUserProfile(_))));
    }

    #[test]
    fn no_trade_direction_produces_no_action() {
        let engine = PortfolioRecommendationEngine::new();
        let recs = engine
            .recommend(&[no_trade_request("INFY.NS")], &moderate_profile(), &empty_context(10000.0))
            .unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].action, RecommendationAction::NoAction);
    }

    #[test]
    fn horizon_exceeded_produces_no_action() {
        // Short-term profile (max 5 sessions) + request with 20 sessions → NO_ACTION
        let engine = PortfolioRecommendationEngine::new();
        let recs = engine
            .recommend(&[long_request("INFY.NS", 20)], &short_term_profile(), &empty_context(10000.0))
            .unwrap();
        assert_eq!(recs[0].action, RecommendationAction::NoAction);
        assert!(recs[0].rationale.contains("horizon"));
    }

    #[test]
    fn within_horizon_produces_add() {
        // Short-term profile (max 5 sessions) + request with 5 sessions → ADD
        let engine = PortfolioRecommendationEngine::new();
        let recs = engine
            .recommend(&[long_request("INFY.NS", 5)], &short_term_profile(), &empty_context(10000.0))
            .unwrap();
        assert_eq!(recs[0].action, RecommendationAction::Add);
    }

    #[test]
    fn budget_is_enforced_across_instruments() {
        // Budget = 5000, Moderate (max_single = 5000 × 0.75 = 3750 → rounds to 3700)
        // First instrument gets 3700. Remaining budget = 1300.
        // Second instrument: cash capped to 1300 → round_down(1300, 100) = 1300 → ADD 1300.
        // Remaining = 0. Third instrument → ADD with ₹0 (budget exhausted, signal still valid).
        let engine = PortfolioRecommendationEngine::new();
        let requests = vec![
            long_request("INFY.NS", 20),
            long_request("TCS.NS", 20),
            long_request("WIPRO.NS", 20),
        ];
        let recs = engine
            .recommend(&requests, &moderate_profile(), &empty_context(50000.0))
            .unwrap();
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[0].action, RecommendationAction::Add);
        assert!((recs[0].allocation_inr - 3700.0).abs() < 1e-6);
        assert_eq!(recs[1].action, RecommendationAction::Add);
        assert!((recs[1].allocation_inr - 1300.0).abs() < 1e-6);
        // Third instrument: budget exhausted → ADD with ₹0 (not AVOID — signal is valid)
        assert_eq!(recs[2].action, RecommendationAction::Add,
            "budget exhaustion must not change ADD to AVOID");
        assert!((recs[2].allocation_inr - 0.0).abs() < 1e-6,
            "allocation must be ₹0 when budget is exhausted");
    }

    #[test]
    fn total_allocated_does_not_exceed_weekly_budget() {
        let engine = PortfolioRecommendationEngine::new();
        let requests = vec![
            long_request("INFY.NS", 20),
            long_request("TCS.NS", 20),
            long_request("WIPRO.NS", 20),
            long_request("HDFCBANK.NS", 20),
        ];
        let recs = engine
            .recommend(&requests, &moderate_profile(), &empty_context(50000.0))
            .unwrap();
        let total: f64 = recs.iter().map(|r| r.allocation_inr).sum();
        assert!(
            total <= moderate_profile().weekly_investment_inr + 1e-6,
            "total allocated ({total}) must not exceed weekly budget ({})",
            moderate_profile().weekly_investment_inr
        );
    }

    #[test]
    fn coralys_parameters_unchanged_through_engine() {
        let engine = PortfolioRecommendationEngine::new();
        let req = long_request("INFY.NS", 20);
        let recs = engine
            .recommend(&[req.clone()], &moderate_profile(), &empty_context(10000.0))
            .unwrap();
        assert!((recs[0].target_pct - req.target_pct).abs() < 1e-9);
        assert!((recs[0].risk_pct - req.risk_pct).abs() < 1e-9);
        assert_eq!(recs[0].maximum_hold_sessions, req.maximum_hold_sessions);
        assert_eq!(recs[0].c3_002_direction, req.c3_002_direction);
    }

    #[test]
    fn mixed_directions_processed_correctly() {
        let engine = PortfolioRecommendationEngine::new();
        let requests = vec![
            long_request("INFY.NS", 20),
            no_trade_request("TCS.NS"),
            long_request("WIPRO.NS", 20),
        ];
        let recs = engine
            .recommend(&requests, &moderate_profile(), &empty_context(10000.0))
            .unwrap();
        assert_eq!(recs[0].action, RecommendationAction::Add);
        assert_eq!(recs[1].action, RecommendationAction::NoAction);
        assert_eq!(recs[2].action, RecommendationAction::Add);
    }
}