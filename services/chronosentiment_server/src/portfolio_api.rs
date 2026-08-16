//! POST /api/v0/portfolio/recommendations — thin adapter over PortfolioRecommendationEngine.
//!
//! ## Responsibility
//!
//! This module is a pure HTTP adapter. It:
//!   1. Deserializes the JSON request (UserProfile + PortfolioContext only).
//!   2. Fetches certified decisions from decisions_api (backend-owned intelligence).
//!   3. Calls PortfolioRecommendationEngine::recommend().
//!   4. Serializes the domain output to JSON.
//!
//! No allocation logic, Coralys logic, or decision logic lives here.
//! The frontend does NOT supply decisions — the backend owns the intelligence.
//!
//! ## Request contract (v0.2)
//!
//! ```json
//! {
//!   "user_profile": {
//!     "user_id": "user-001",
//!     "weekly_investment_inr": 5000.0,
//!     "risk_tolerance": "Moderate",
//!     "investment_horizon": "MediumTerm"
//!   },
//!   "portfolio": {
//!     "as_of": "2026-08-16T09:15:00+05:30",
//!     "available_cash_inr": 5000.0,
//!     "holdings": [
//!       {
//!         "instrument": "INFY.NS",
//!         "quantity": 10,
//!         "average_cost_inr": 1450.0,
//!         "current_value_inr": 16200.0
//!       }
//!     ],
//!     "existing_exposure_inr": {
//!       "INFY.NS": 16200.0
//!     }
//!   }
//! }
//! ```
//!
//! ## Response contract
//!
//! ```json
//! {
//!   "recommendations": [...],
//!   "engine_version": "portfolio-recommendation-engine-v0",
//!   "as_of": "2026-08-16T09:15:00+05:30",
//!   "certified_at": "2026-08-16",
//!   "coralys_artifact": "3876ffa2..."
//! }
//! ```
//!
//! ## Error responses
//!
//! 400 Bad Request — invalid profile or invalid context.
//! 422 Unprocessable Entity — JSON deserialization failure (handled by Axum).

use axum::{http::StatusCode, Json};
use chronosentiment_adapter::product::{
    portfolio_context::{PortfolioContext, PortfolioPosition},
    recommendation::PortfolioAllocationRequest,
    recommendation_engine::{PortfolioRecommendationEngine, RECOMMENDATION_ENGINE_VERSION},
    user_profile::{InvestmentHorizon, RiskTolerance, UserProfile},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Request types ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RecommendationsRequest {
    pub user_profile: UserProfileReq,
    pub portfolio: PortfolioReq,
    // v0.2: decisions are no longer accepted from the client.
    // The backend fetches certified decisions from decisions_api.
}

#[derive(Debug, Deserialize)]
pub struct UserProfileReq {
    pub user_id: String,
    pub weekly_investment_inr: f64,
    pub risk_tolerance: String,
    pub investment_horizon: String,
}

#[derive(Debug, Deserialize)]
pub struct PortfolioReq {
    pub as_of: String,
    pub available_cash_inr: f64,
    #[serde(default)]
    pub holdings: Vec<PortfolioPositionReq>,
    #[serde(default)]
    pub existing_exposure_inr: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
pub struct PortfolioPositionReq {
    pub instrument: String,
    pub quantity: f64,
    pub average_cost_inr: f64,
    pub current_value_inr: f64,
}

// ─── Response types ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct RecommendationsResponse {
    pub recommendations: Vec<RecommendationResp>,
    pub engine_version: String,
    pub as_of: String,
    /// Date the certified decisions were frozen.
    pub certified_at: String,
    /// Coralys execution artifact hash (frozen).
    pub coralys_artifact: String,
}

#[derive(Debug, Serialize)]
pub struct RecommendationResp {
    pub instrument: String,
    pub action: String,
    pub allocation_inr: f64,
    pub c3_002_direction: String,
    pub entry_price: f64,
    pub target_pct: f64,
    pub target_price: f64,
    pub risk_pct: f64,
    pub risk_boundary: f64,
    pub maximum_hold_sessions: u32,
    pub rationale: String,
    pub decision_id: String,
    pub execution_intent_id: String,
    pub allocation_engine_version: String,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// POST /api/v0/portfolio/recommendations
///
/// Thin adapter: deserialize → domain call → serialize.
/// No business logic lives here.
pub async fn post_recommendations(
    Json(req): Json<RecommendationsRequest>,
) -> Result<Json<RecommendationsResponse>, (StatusCode, Json<ApiError>)> {
    // 1. Map risk_tolerance string → domain enum.
    let risk_tolerance = parse_risk_tolerance(&req.user_profile.risk_tolerance)
        .map_err(|e| bad_request(e))?;

    // 2. Map investment_horizon string → domain enum.
    let investment_horizon = parse_investment_horizon(&req.user_profile.investment_horizon)
        .map_err(|e| bad_request(e))?;

    // 3. Build UserProfile.
    let profile = UserProfile {
        user_id: req.user_profile.user_id,
        weekly_investment_inr: req.user_profile.weekly_investment_inr,
        risk_tolerance,
        investment_horizon,
    };

    // 4. Build PortfolioContext.
    let holdings: Vec<PortfolioPosition> = req
        .portfolio
        .holdings
        .into_iter()
        .map(|h| PortfolioPosition {
            instrument: h.instrument,
            quantity: h.quantity,
            average_cost_inr: h.average_cost_inr,
            current_value_inr: Some(h.current_value_inr),
        })
        .collect();

    let context = PortfolioContext {
        as_of: req.portfolio.as_of.clone(),
        available_cash_inr: req.portfolio.available_cash_inr,
        holdings,
        existing_exposure_inr: req.portfolio.existing_exposure_inr,
    };

    // 5. Fetch certified decisions from the backend's intelligence source.
    //    The frontend does NOT supply decisions — the backend owns the intelligence.
    //    In v0.3+, this will load from the live P.E.3 execution ledger.
    let certified = super::decisions_api::get_certified_decisions();
    let requests: Vec<PortfolioAllocationRequest> = certified
        .decisions
        .into_iter()
        .map(|d| PortfolioAllocationRequest {
            instrument: d.instrument,
            c3_002_direction: d.c3_002_direction,
            entry_price: d.entry_price,
            target_pct: d.target_pct,
            target_price: d.target_price,
            risk_pct: d.risk_pct,
            risk_boundary: d.risk_boundary,
            maximum_hold_sessions: d.maximum_hold_sessions,
            decision_rationale: d.decision_rationale,
            decision_id: d.decision_id,
            execution_intent_id: d.execution_intent_id,
        })
        .collect();

    // 6. Call domain engine.
    let engine = PortfolioRecommendationEngine::new();
    let recs = engine
        .recommend(&requests, &profile, &context)
        .map_err(|e| bad_request(e.to_string()))?;

    // 7. Map domain output → response.
    let recommendations = recs
        .into_iter()
        .map(|r| RecommendationResp {
            instrument: r.instrument,
            action: format!("{:?}", r.action),
            allocation_inr: r.allocation_inr,
            c3_002_direction: r.c3_002_direction,
            entry_price: r.entry_price,
            target_pct: r.target_pct,
            target_price: r.target_price,
            risk_pct: r.risk_pct,
            risk_boundary: r.risk_boundary,
            maximum_hold_sessions: r.maximum_hold_sessions,
            rationale: r.rationale,
            decision_id: r.decision_id,
            execution_intent_id: r.execution_intent_id,
            allocation_engine_version: r.allocation_engine_version,
        })
        .collect();

    Ok(Json(RecommendationsResponse {
        recommendations,
        engine_version: RECOMMENDATION_ENGINE_VERSION.to_string(),
        as_of: req.portfolio.as_of,
        certified_at: certified.certified_at,
        coralys_artifact: certified.coralys_artifact,
    }))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn parse_risk_tolerance(s: &str) -> Result<RiskTolerance, String> {
    match s {
        "Conservative" => Ok(RiskTolerance::Conservative),
        "Moderate" => Ok(RiskTolerance::Moderate),
        "Aggressive" => Ok(RiskTolerance::Aggressive),
        other => Err(format!(
            "unknown risk_tolerance '{}'. Valid values: Conservative, Moderate, Aggressive",
            other
        )),
    }
}

fn parse_investment_horizon(s: &str) -> Result<InvestmentHorizon, String> {
    match s {
        "ShortTerm" => Ok(InvestmentHorizon::ShortTerm),
        "MediumTerm" => Ok(InvestmentHorizon::MediumTerm),
        "LongTerm" => Ok(InvestmentHorizon::LongTerm),
        other => Err(format!(
            "unknown investment_horizon '{}'. Valid values: ShortTerm, MediumTerm, LongTerm",
            other
        )),
    }
}

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { error: msg.into() }),
    )
}