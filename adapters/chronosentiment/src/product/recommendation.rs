//! PortfolioRecommendation — the first true product object.
//!
//! This is what ChronoSentiment delivers to a user:
//!   "Add ₹1,000 to INFY this week. Target: 6.2%. Risk: 3.1%. Max hold: 20 sessions."
//!
//! Architectural invariant:
//!   PortfolioRecommendation MAY interpret and allocate a decision,
//!   but it CANNOT alter the underlying C3-002 direction or Coralys execution parameters.
//!
//! The recommendation carries full provenance:
//!   - decision_id links back to the C3-002 TradingDecision
//!   - execution_intent_id links back to the CoralysExecutionIntent
//!   - allocation_engine_version documents which allocation logic was applied
//!
//! This enables post-hoc attribution:
//!   Was C3-002 wrong?          → check decision_id
//!   Was execution target poor? → check execution_intent_id
//!   Was allocation wrong?      → check allocation_engine_version + rationale
//!   Was portfolio overexposed? → check rationale + portfolio_context snapshot

use serde::{Deserialize, Serialize};

/// The action the allocation engine recommends for a specific instrument.
///
/// These are portfolio-level actions, not market-intelligence directions.
/// The underlying C3-002 direction (LONG/SHORT) is preserved separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecommendationAction {
    /// Open a new position or add to an existing one.
    Add,
    /// Maintain the current position without change.
    Hold,
    /// Reduce an existing position (partial exit).
    Reduce,
    /// Do not open or add to a position (signal present but constraints prevent allocation).
    Avoid,
    /// No actionable signal for this instrument in this cycle.
    NoAction,
}

impl RecommendationAction {
    pub fn label(&self) -> &str {
        match self {
            RecommendationAction::Add => "ADD",
            RecommendationAction::Hold => "HOLD",
            RecommendationAction::Reduce => "REDUCE",
            RecommendationAction::Avoid => "AVOID",
            RecommendationAction::NoAction => "NO_ACTION",
        }
    }

    /// Whether this action involves deploying new capital.
    pub fn deploys_capital(&self) -> bool {
        matches!(self, RecommendationAction::Add)
    }
}

/// Input to the allocation engine for a single instrument.
///
/// This bundles the C3-002 decision reference and the Coralys execution intent
/// into a single request object. The allocation engine reads this alongside
/// UserProfile and PortfolioContext to produce a PortfolioRecommendation.
///
/// The allocation engine must not modify `c3_002_direction`, `target_pct`,
/// `risk_pct`, `risk_boundary`, or `maximum_hold_sessions`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortfolioAllocationRequest {
    /// Canonical instrument symbol, e.g. "INFY.NS".
    pub instrument: String,
    /// C3-002 direction: "LONG", "SHORT", or "NO_TRADE".
    /// Sealed at T by C3-002. The allocation engine must not alter this.
    pub c3_002_direction: String,
    /// Entry price at next session open (E). From Coralys execution intent.
    pub entry_price: f64,
    /// Target percentage from Coralys execution intent. Frozen. Must not be altered.
    pub target_pct: f64,
    /// Target price from Coralys execution intent.
    pub target_price: f64,
    /// Risk percentage from Coralys execution intent. Frozen. Must not be altered.
    pub risk_pct: f64,
    /// Risk boundary price from Coralys execution intent.
    pub risk_boundary: f64,
    /// Maximum hold in sessions from Coralys execution intent.
    pub maximum_hold_sessions: u32,
    /// Rationale from the C3-002 decision.
    pub decision_rationale: String,
    /// UUID of the C3-002 TradingDecision. Provenance link.
    pub decision_id: String,
    /// Hash of the CoralysExecutionIntent. Provenance link.
    pub execution_intent_id: String,
}

/// The product recommendation for a single instrument.
///
/// This is the output of the PortfolioRecommendationEngine.
/// It is the object that a UI, API, or notification system consumes.
///
/// The allocation engine fills `action`, `allocation_inr`, and `rationale`.
/// All other fields are passed through unchanged from the input contracts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortfolioRecommendation {
    /// Canonical instrument symbol.
    pub instrument: String,
    /// Portfolio-level action recommended by the allocation engine.
    pub action: RecommendationAction,
    /// Amount to allocate in INR. Zero for HOLD, AVOID, NO_ACTION.
    pub allocation_inr: f64,
    /// C3-002 direction (LONG/SHORT/NO_TRADE). Passed through unchanged.
    pub c3_002_direction: String,
    /// Entry price at next session open. From Coralys execution intent.
    pub entry_price: f64,
    /// Target percentage. From Coralys execution intent. Unchanged.
    pub target_pct: f64,
    /// Target price. From Coralys execution intent.
    pub target_price: f64,
    /// Risk percentage. From Coralys execution intent. Unchanged.
    pub risk_pct: f64,
    /// Risk boundary price. From Coralys execution intent.
    pub risk_boundary: f64,
    /// Maximum hold in sessions. From Coralys execution intent.
    pub maximum_hold_sessions: u32,
    /// Human-readable rationale for the allocation decision.
    /// Explains why this action and amount were chosen given the portfolio context.
    pub rationale: String,
    /// UUID of the C3-002 TradingDecision. Provenance link.
    pub decision_id: String,
    /// Hash of the CoralysExecutionIntent. Provenance link.
    pub execution_intent_id: String,
    /// Version of the allocation engine that produced this recommendation.
    pub allocation_engine_version: String,
}

impl PortfolioRecommendation {
    /// Human-readable summary suitable for display in a UI or notification.
    ///
    /// Example: "ADD ₹1,000 to INFY.NS | Target: 6.2% | Risk: 3.1% | Max hold: 20 sessions"
    pub fn summary(&self) -> String {
        if self.allocation_inr > 0.0 {
            format!(
                "{} ₹{:.0} to {} | Target: {:.1}% | Risk: {:.1}% | Max hold: {} sessions",
                self.action.label(),
                self.allocation_inr,
                self.instrument,
                self.target_pct * 100.0,
                self.risk_pct * 100.0,
                self.maximum_hold_sessions,
            )
        } else {
            format!(
                "{} {} | {}",
                self.action.label(),
                self.instrument,
                self.rationale,
            )
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> PortfolioAllocationRequest {
        PortfolioAllocationRequest {
            instrument: "INFY.NS".into(),
            c3_002_direction: "LONG".into(),
            entry_price: 1076.30,
            target_pct: 0.062,
            target_price: 1143.23,
            risk_pct: 0.031,
            risk_boundary: 1042.94,
            maximum_hold_sessions: 20,
            decision_rationale: "Bullish trend with positive momentum.".into(),
            decision_id: "dec-uuid-001".into(),
            execution_intent_id: "exec-hash-001".into(),
        }
    }

    fn sample_recommendation() -> PortfolioRecommendation {
        let req = sample_request();
        PortfolioRecommendation {
            instrument: req.instrument.clone(),
            action: RecommendationAction::Add,
            allocation_inr: 1000.0,
            c3_002_direction: req.c3_002_direction.clone(),
            entry_price: req.entry_price,
            target_pct: req.target_pct,
            target_price: req.target_price,
            risk_pct: req.risk_pct,
            risk_boundary: req.risk_boundary,
            maximum_hold_sessions: req.maximum_hold_sessions,
            rationale: "Allocated ₹1,000 (20% of weekly budget). Existing exposure: ₹25,000.".into(),
            decision_id: req.decision_id.clone(),
            execution_intent_id: req.execution_intent_id.clone(),
            allocation_engine_version: "allocation-engine-v0".into(),
        }
    }

    #[test]
    fn add_action_deploys_capital() {
        assert!(RecommendationAction::Add.deploys_capital());
    }

    #[test]
    fn hold_action_does_not_deploy_capital() {
        assert!(!RecommendationAction::Hold.deploys_capital());
    }

    #[test]
    fn avoid_action_does_not_deploy_capital() {
        assert!(!RecommendationAction::Avoid.deploys_capital());
    }

    #[test]
    fn no_action_does_not_deploy_capital() {
        assert!(!RecommendationAction::NoAction.deploys_capital());
    }

    #[test]
    fn summary_includes_allocation_and_percentages() {
        let rec = sample_recommendation();
        let s = rec.summary();
        assert!(s.contains("ADD"), "summary must contain action: {s}");
        assert!(s.contains("INFY.NS"), "summary must contain instrument: {s}");
        assert!(s.contains("1000"), "summary must contain allocation: {s}");
        assert!(s.contains("6.2%"), "summary must contain target_pct: {s}");
        assert!(s.contains("3.1%"), "summary must contain risk_pct: {s}");
        assert!(s.contains("20 sessions"), "summary must contain max hold: {s}");
    }

    #[test]
    fn summary_for_no_action_omits_allocation() {
        let mut rec = sample_recommendation();
        rec.action = RecommendationAction::NoAction;
        rec.allocation_inr = 0.0;
        rec.rationale = "No actionable signal.".into();
        let s = rec.summary();
        assert!(s.contains("NO_ACTION"), "summary must contain action: {s}");
        assert!(s.contains("INFY.NS"), "summary must contain instrument: {s}");
        assert!(!s.contains("₹0"), "zero allocation should not appear in summary: {s}");
    }

    #[test]
    fn recommendation_preserves_coralys_parameters_unchanged() {
        // The allocation engine must not alter target_pct, risk_pct, or maximum_hold_sessions.
        let req = sample_request();
        let rec = sample_recommendation();
        assert!((rec.target_pct - req.target_pct).abs() < 1e-9,
            "target_pct must be passed through unchanged");
        assert!((rec.risk_pct - req.risk_pct).abs() < 1e-9,
            "risk_pct must be passed through unchanged");
        assert_eq!(rec.maximum_hold_sessions, req.maximum_hold_sessions,
            "maximum_hold_sessions must be passed through unchanged");
        assert_eq!(rec.c3_002_direction, req.c3_002_direction,
            "c3_002_direction must be passed through unchanged");
    }

    #[test]
    fn provenance_ids_are_preserved() {
        let req = sample_request();
        let rec = sample_recommendation();
        assert_eq!(rec.decision_id, req.decision_id);
        assert_eq!(rec.execution_intent_id, req.execution_intent_id);
    }
}