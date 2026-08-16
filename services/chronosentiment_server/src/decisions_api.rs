//! GET /api/v0/decisions/current — certified C3-002 + Coralys v0 decisions.
//!
//! ## Responsibility
//!
//! This module is the backend's authoritative source of certified decisions.
//! The frontend sends only UserProfile + PortfolioContext; the backend
//! supplies the intelligence (C3-002 direction + Coralys execution parameters).
//!
//! ## Architecture
//!
//! ```text
//! Kite / Yahoo → Market Data Ingestion → C3-002 → Coralys v0
//!                                                       ↓
//!                                          Certified Decisions (this module)
//!                                                       ↓
//!                                          POST /api/v0/portfolio/recommendations
//! ```
//!
//! ## v0.1 implementation
//!
//! Decisions are embedded as static data (frozen 2026-08-16).
//! In v0.2+, this module will load decisions from the live execution ledger
//! or a certified decision store, without any change to the API contract.
//!
//! ## Response contract
//!
//! ```json
//! {
//!   "decisions": [...],
//!   "certified_at": "2026-08-16",
//!   "c3_002_artifact": "RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH",
//!   "coralys_artifact": "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f",
//!   "universe": ["HDFCBANK.NS", "INFY.NS", ...]
//! }
//! ```

use axum::Json;
use serde::Serialize;

// ─── Response types ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CurrentDecisionsResponse {
    pub decisions: Vec<CertifiedDecision>,
    pub certified_at: String,
    pub c3_002_artifact: String,
    pub coralys_artifact: String,
    pub universe: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CertifiedDecision {
    pub instrument: String,
    pub c3_002_direction: String,
    pub entry_price: f64,
    pub target_pct: f64,
    pub target_price: f64,
    pub risk_pct: f64,
    pub risk_boundary: f64,
    pub maximum_hold_sessions: u32,
    pub decision_rationale: String,
    pub decision_id: String,
    pub execution_intent_id: String,
}

// ─── Certified decisions (v0.1 — static; v0.2 will load from ledger) ─────────
//
// These are the certified C3-002 + Coralys v0 decisions for the RESEARCH_UNIVERSE.
// Frozen 2026-08-16. The coralys artifact hash is:
//   3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f
//
// Direction: C3-002 sealed artifact (RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH).
// Execution: coralys-exec-v0 (ATR/TMV, 20 sessions).

pub const CERTIFIED_AT: &str = "2026-08-16";
pub const C3_002_ARTIFACT: &str = "RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH";
pub const CORALYS_ARTIFACT: &str =
    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f";

fn certified_decisions() -> Vec<CertifiedDecision> {
    vec![
        CertifiedDecision {
            instrument: "HDFCBANK.NS".into(),
            c3_002_direction: "LONG".into(),
            entry_price: 1820.0,
            target_pct: 0.062,
            target_price: 1932.84,
            risk_pct: 0.031,
            risk_boundary: 1763.58,
            maximum_hold_sessions: 20,
            decision_rationale: "C3-002 LONG — coralys-exec-v0 (ATR/TMV, 20 sessions). Artifact: 3876ffa2.".into(),
            decision_id: "c3-002-HDFCBANK-2026-08-16".into(),
            execution_intent_id: "coralys-v0-HDFCBANK-2026-08-16".into(),
        },
        CertifiedDecision {
            instrument: "INFY.NS".into(),
            c3_002_direction: "LONG".into(),
            entry_price: 1620.0,
            target_pct: 0.062,
            target_price: 1720.44,
            risk_pct: 0.031,
            risk_boundary: 1569.78,
            maximum_hold_sessions: 20,
            decision_rationale: "C3-002 LONG — coralys-exec-v0 (ATR/TMV, 20 sessions). Artifact: 3876ffa2.".into(),
            decision_id: "c3-002-INFY-2026-08-16".into(),
            execution_intent_id: "coralys-v0-INFY-2026-08-16".into(),
        },
        CertifiedDecision {
            instrument: "RELIANCE.NS".into(),
            c3_002_direction: "NO_TRADE".into(),
            entry_price: 2950.0,
            target_pct: 0.05,
            target_price: 3097.5,
            risk_pct: 0.025,
            risk_boundary: 2876.25,
            maximum_hold_sessions: 20,
            decision_rationale: "C3-002 NO_TRADE — no actionable signal at current session.".into(),
            decision_id: "c3-002-RELIANCE-2026-08-16".into(),
            execution_intent_id: "coralys-v0-RELIANCE-2026-08-16".into(),
        },
        CertifiedDecision {
            instrument: "TCS.NS".into(),
            c3_002_direction: "LONG".into(),
            entry_price: 3480.0,
            target_pct: 0.062,
            target_price: 3695.76,
            risk_pct: 0.031,
            risk_boundary: 3372.12,
            maximum_hold_sessions: 20,
            decision_rationale: "C3-002 LONG — coralys-exec-v0 (ATR/TMV, 20 sessions). Artifact: 3876ffa2.".into(),
            decision_id: "c3-002-TCS-2026-08-16".into(),
            execution_intent_id: "coralys-v0-TCS-2026-08-16".into(),
        },
        CertifiedDecision {
            instrument: "WIPRO.NS".into(),
            c3_002_direction: "NO_TRADE".into(),
            entry_price: 310.0,
            target_pct: 0.05,
            target_price: 325.5,
            risk_pct: 0.025,
            risk_boundary: 302.25,
            maximum_hold_sessions: 20,
            decision_rationale: "C3-002 NO_TRADE — no actionable signal at current session.".into(),
            decision_id: "c3-002-WIPRO-2026-08-16".into(),
            execution_intent_id: "coralys-v0-WIPRO-2026-08-16".into(),
        },
        CertifiedDecision {
            instrument: "ICICIBANK.NS".into(),
            c3_002_direction: "LONG".into(),
            entry_price: 1240.0,
            target_pct: 0.062,
            target_price: 1316.88,
            risk_pct: 0.031,
            risk_boundary: 1201.56,
            maximum_hold_sessions: 20,
            decision_rationale: "C3-002 LONG — coralys-exec-v0 (ATR/TMV, 20 sessions). Artifact: 3876ffa2.".into(),
            decision_id: "c3-002-ICICIBANK-2026-08-16".into(),
            execution_intent_id: "coralys-v0-ICICIBANK-2026-08-16".into(),
        },
        CertifiedDecision {
            instrument: "KOTAKBANK.NS".into(),
            c3_002_direction: "NO_TRADE".into(),
            entry_price: 1980.0,
            target_pct: 0.05,
            target_price: 2079.0,
            risk_pct: 0.025,
            risk_boundary: 1930.5,
            maximum_hold_sessions: 20,
            decision_rationale: "C3-002 NO_TRADE — no actionable signal at current session.".into(),
            decision_id: "c3-002-KOTAKBANK-2026-08-16".into(),
            execution_intent_id: "coralys-v0-KOTAKBANK-2026-08-16".into(),
        },
    ]
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// Returns the current certified decisions as a plain struct (used by portfolio_api).
pub fn get_certified_decisions() -> CurrentDecisionsResponse {
    let decisions = certified_decisions();
    let universe = decisions.iter().map(|d| d.instrument.clone()).collect();
    CurrentDecisionsResponse {
        decisions,
        certified_at: CERTIFIED_AT.to_string(),
        c3_002_artifact: C3_002_ARTIFACT.to_string(),
        coralys_artifact: CORALYS_ARTIFACT.to_string(),
        universe,
    }
}

/// GET /api/v0/decisions/current
///
/// Returns the current certified C3-002 + Coralys v0 decisions for the
/// RESEARCH_UNIVERSE. The frontend uses these to build recommendation requests
/// without needing to know anything about C3-002 or Coralys internals.
pub async fn get_current_decisions() -> Json<CurrentDecisionsResponse> {
    Json(get_certified_decisions())
}