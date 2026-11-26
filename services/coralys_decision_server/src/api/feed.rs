//! MVP-005 — Decision Feed API: `GET /decisions`
//!
//! Returns all certified decisions from the ledger in reverse chronological
//! order (newest first). This is the only ordering exposed — no ranking,
//! confidence, or capital-efficiency sort is available.
//!
//! **Acceptance criteria:**
//! - AC-F1: Only certified decisions are returned.
//! - AC-F2: Returned decisions exist in the canonical ledger.
//! - AC-F3: No decision is mutated during serialization.
//! - AC-F4: No allocation/quantity is inferred.
//! - AC-F5: No confidence/ranking/probability appears.
//! - AC-F6: Ordering is deterministic (decision_timestamp DESC).
//! - AC-F7: Empty ledger returns a valid empty response.

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use super::{ApiCertificationStatus, ApiDirection};
use crate::AppState;

// ─── Feed response ────────────────────────────────────────────────────────────

/// A single entry in the Decision Feed.
///
/// Contains only the fields needed for the feed view — the full record is
/// available via `GET /decisions/{id}`.
///
/// **No confidence, ranking, probability, or allocation fields.**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntry {
    pub decision_id: String,
    pub instrument: String,
    pub decision_timestamp: String,
    pub direction: ApiDirection,
    pub certification_status: ApiCertificationStatus,
    pub target_price: Option<f64>,
    pub reference_risk_boundary_price: Option<f64>,
    pub reference_risk_boundary_type: String,
    pub outcome_status: String,
    pub execution_status: String,
    /// Trend label from certified TMV state (e.g. "Bullish", "Bearish").
    pub trend: String,
    /// Momentum label from certified TMV state (e.g. "Positive", "Negative").
    pub momentum: String,
    /// ATR-14 in price units at decision time T. Null when unavailable.
    pub atr_14: Option<f64>,
    /// Last traded price / previous close at decision time T.
    pub reference_price: Option<f64>,
    /// Next NSE trading session date (YYYY-MM-DD) this decision applies to.
    pub effective_session: Option<String>,
}

/// Response envelope for `GET /decisions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedResponse {
    pub decisions: Vec<FeedEntry>,
    pub total: usize,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// `GET /decisions`
///
/// Returns all certified decisions from the ledger, newest first.
/// The ledger is the authoritative source — no decisions are reconstructed
/// from C3-002 at request time.
pub async fn get_decisions(State(state): State<AppState>) -> Json<FeedResponse> {
    let ledger = state.ledger.read().await;

    // AC-F6: newest first (decision_timestamp DESC).
    let entries: Vec<FeedEntry> = ledger
        .decisions_newest_first()
        .into_iter()
        // AC-F1: only certified decisions.
        .filter(|r| r.is_certified())
        .map(|r| FeedEntry {
            decision_id: r.identity.decision_id.clone(),
            instrument: r.identity.instrument.clone(),
            decision_timestamp: r.identity.decision_timestamp.to_rfc3339(),
            direction: ApiDirection::from(&r.decision.direction),
            certification_status: ApiCertificationStatus::from(&r.certification.status),
            target_price: r.decision.target_price,
            reference_risk_boundary_price: r.reference_risk.boundary_price,
            reference_risk_boundary_type: r.reference_risk.boundary_type.clone(),
            outcome_status: format!("{:?}", r.outcome.status),
            execution_status: format!("{:?}", r.execution.status),
            trend: r.decision.trend.clone(),
            momentum: r.decision.momentum.clone(),
            atr_14: r.decision.atr_14,
            reference_price: r.decision.reference_price,
            effective_session: r.decision.effective_session.clone(),
        })
        .collect();

    let total = entries.len();
    Json(FeedResponse {
        decisions: entries,
        total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{make_app, seal_sample_decision};
    use axum_test::TestServer;
    use serde_json::Value;

    #[tokio::test]
    async fn empty_ledger_returns_valid_empty_response() {
        // AC-F7
        let app = make_app().await;
        let server = TestServer::new(app);
        let resp = server.get("/decisions").await;
        resp.assert_status_ok();
        let body: FeedResponse = resp.json();
        assert_eq!(body.total, 0);
        assert!(body.decisions.is_empty());
    }

    #[tokio::test]
    async fn certified_decision_appears_in_feed() {
        // AC-F1, AC-F2
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server.get("/decisions").await;
        resp.assert_status_ok();
        let body: FeedResponse = resp.json();
        assert_eq!(body.total, 1);
        assert_eq!(
            body.decisions[0].decision_id,
            "coralys-ADANIENT-20260817T101500Z-001"
        );
        assert_eq!(body.decisions[0].instrument, "ADANIENT.NS");
    }

    #[tokio::test]
    async fn feed_has_no_confidence_or_allocation_fields() {
        // AC-F4, AC-F5
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server.get("/decisions").await;
        let raw: Value = resp.json();
        let text = serde_json::to_string(&raw).unwrap();
        assert!(!text.contains("confidence"));
        assert!(!text.contains("probability"));
        assert!(!text.contains("allocation"));
        assert!(!text.contains("capital"));
        assert!(!text.contains("ranking"));
        assert!(!text.contains("quantity"));
    }

    #[tokio::test]
    async fn feed_ordering_is_newest_first() {
        // AC-F6
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;
        seal_sample_decision(&state, "coralys-BPCL-20260817T103000Z-002", "BPCL.NS").await;

        let server = TestServer::new(app);
        let resp = server.get("/decisions").await;
        let body: FeedResponse = resp.json();
        assert_eq!(body.total, 2);
        // BPCL was sealed second (later timestamp) — should appear first.
        assert_eq!(body.decisions[0].instrument, "BPCL.NS");
        assert_eq!(body.decisions[1].instrument, "ADANIENT.NS");
    }

    // Helper: make_app_with_state returns both the router and the shared state
    // so tests can pre-populate the ledger.
    async fn make_app_with_state() -> (axum::Router, crate::AppState) {
        crate::test_helpers::make_app_with_state().await
    }
}
