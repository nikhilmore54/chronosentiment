//! MVP-008 — Outcome Recording: `POST /decisions/{id}/outcome`
//!
//! Records an observed outcome for a certified decision.
//! The caller must confirm that the observation boundary has passed.
//! The original certified decision is never modified.
//!
//! **Acceptance criteria:**
//! - AC-O1: Outcome before observation boundary → 422.
//! - AC-O2: Outcome after boundary → 200 with updated record.
//! - AC-O3: Unknown decision → 404.
//! - AC-O4: Outcome cannot alter certification.
//! - AC-O5: Outcome cannot alter decision core.
//! - AC-O6: Outcome cannot inject evidence into the original decision.
//! - AC-O7: Outcome appears as an appended lifecycle event.
//! - AC-O8: Original decision remains semantically unchanged.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use coralys_decision::record::{OutcomeRecord, OutcomeStatus};

use super::DecisionResponse;
use crate::AppState;

// ─── Request ──────────────────────────────────────────────────────────────────

/// Request body for `POST /decisions/{id}/outcome`.
///
/// The caller must explicitly confirm that the observation boundary has passed
/// by setting `observation_boundary_passed: true`. This is a hard invariant —
/// the ledger will reject the outcome if this is false.
#[derive(Debug, Clone, Deserialize)]
pub struct RecordOutcomeRequest {
    /// Observed outcome status.
    pub status: OutcomeStatusRequest,
    /// The caller must confirm the observation boundary has passed.
    /// The ledger enforces this as a hard invariant.
    pub observation_boundary_passed: bool,
    /// Timestamp of the outcome observation.
    pub exit_timestamp: Option<DateTime<Utc>>,
    /// Exit price at which the outcome was observed.
    pub exit_price: Option<f64>,
    /// Human-readable reason for the exit.
    pub exit_reason: Option<String>,
    /// Realized P&L. Only present when the user explicitly supplies it.
    pub realized_pnl: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutcomeStatusRequest {
    Target,
    ReferenceRisk,
    Horizon,
    UserClosed,
    Open,
}

impl From<OutcomeStatusRequest> for OutcomeStatus {
    fn from(s: OutcomeStatusRequest) -> Self {
        match s {
            OutcomeStatusRequest::Target => OutcomeStatus::Target,
            OutcomeStatusRequest::ReferenceRisk => OutcomeStatus::ReferenceRisk,
            OutcomeStatusRequest::Horizon => OutcomeStatus::Horizon,
            OutcomeStatusRequest::UserClosed => OutcomeStatus::UserClosed,
            OutcomeStatusRequest::Open => OutcomeStatus::Open,
        }
    }
}

// ─── Response ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct OutcomeRecordedResponse {
    pub decision: DecisionResponse,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub decision_id: String,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// `POST /decisions/{id}/outcome`
///
/// Appends an observed outcome to a certified decision.
/// The original certified decision is never modified.
pub async fn record_outcome(
    State(state): State<AppState>,
    Path(decision_id): Path<String>,
    Json(body): Json<RecordOutcomeRequest>,
) -> impl IntoResponse {
    let mut ledger = state.ledger.write().await;

    let event_ts = body.exit_timestamp.unwrap_or_else(Utc::now);

    let outcome = OutcomeRecord {
        status: body.status.into(),
        exit_reason: body.exit_reason,
        exit_timestamp: body.exit_timestamp,
        exit_price: body.exit_price,
        realized_pnl: body.realized_pnl,
    };

    match ledger.record_outcome(
        &decision_id,
        outcome,
        event_ts,
        body.observation_boundary_passed,
    ) {
        Ok(()) => {
            let record = ledger.get_decision(&decision_id).unwrap();
            let response = OutcomeRecordedResponse {
                decision: DecisionResponse::from(record),
                message: "Outcome recorded.".to_string(),
            };
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
        }
        Err(coralys_decision::ledger::LedgerError::DecisionNotFound(_)) => {
            let err = ErrorResponse {
                error: "decision not found".to_string(),
                decision_id: decision_id.clone(),
            };
            (StatusCode::NOT_FOUND, Json(serde_json::to_value(err).unwrap())).into_response()
        }
        Err(coralys_decision::ledger::LedgerError::ObservationBoundaryNotPassed(_)) => {
            let err = ErrorResponse {
                error: "observation boundary has not passed — outcome cannot be recorded yet"
                    .to_string(),
                decision_id: decision_id.clone(),
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::to_value(err).unwrap()),
            )
                .into_response()
        }
        Err(coralys_decision::ledger::LedgerError::TemporalFirewallViolation { .. }) => {
            let err = ErrorResponse {
                error: "outcome timestamp is before the decision timestamp".to_string(),
                decision_id: decision_id.clone(),
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::to_value(err).unwrap()),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse {
                error: e.to_string(),
                decision_id: decision_id.clone(),
            };
            (
                StatusCode::CONFLICT,
                Json(serde_json::to_value(err).unwrap()),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{make_app_with_state, seal_sample_decision};
    use axum_test::TestServer;
    use serde_json::{Value, json};

    #[tokio::test]
    async fn outcome_without_boundary_confirmation_returns_422() {
        // AC-O1
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/outcome")
            .json(&json!({
                "status": "TARGET",
                "observation_boundary_passed": false,
                "exit_timestamp": "2026-08-20T10:15:00Z",
                "exit_price": 1234.50
            }))
            .await;
        resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
        let body: Value = resp.json();
        assert!(body["error"]
            .as_str()
            .unwrap()
            .contains("observation boundary"));
    }

    #[tokio::test]
    async fn outcome_with_boundary_confirmation_returns_200() {
        // AC-O2
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/outcome")
            .json(&json!({
                "status": "TARGET",
                "observation_boundary_passed": true,
                "exit_timestamp": "2026-08-20T10:15:00Z",
                "exit_price": 1234.50
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["outcome"]["status"], "TARGET");
    }

    #[tokio::test]
    async fn unknown_decision_returns_404() {
        // AC-O3
        let (app, _state) = make_app_with_state().await;
        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/nonexistent/outcome")
            .json(&json!({
                "status": "TARGET",
                "observation_boundary_passed": true
            }))
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn outcome_does_not_alter_certification() {
        // AC-O4
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/outcome")
            .json(&json!({
                "status": "TARGET",
                "observation_boundary_passed": true,
                "exit_timestamp": "2026-08-20T10:15:00Z"
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["certification"]["status"], "CERTIFIED");
        assert!(!body["decision"]["certification"]["policy_artifact_hash"]
            .as_str()
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn outcome_does_not_alter_decision_core() {
        // AC-O5
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/outcome")
            .json(&json!({
                "status": "REFERENCE_RISK",
                "observation_boundary_passed": true,
                "exit_timestamp": "2026-08-20T10:15:00Z",
                "exit_price": 1180.25
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["decision"]["direction"], "LONG");
        assert_eq!(body["decision"]["decision"]["trend"], "Bullish");
        assert_eq!(body["decision"]["decision"]["momentum"], "Positive");
    }

    #[tokio::test]
    async fn outcome_does_not_inject_evidence() {
        // AC-O6
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/outcome")
            .json(&json!({
                "status": "TARGET",
                "observation_boundary_passed": true,
                "exit_timestamp": "2026-08-20T10:15:00Z"
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        // Evidence fields must remain null — outcome does not populate them.
        assert!(body["decision"]["evidence"]["similar_decisions_count"].is_null());
        assert!(body["decision"]["evidence"]["historical_target_rate"].is_null());
    }

    #[tokio::test]
    async fn reference_risk_outcome_is_recorded() {
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/outcome")
            .json(&json!({
                "status": "REFERENCE_RISK",
                "observation_boundary_passed": true,
                "exit_timestamp": "2026-08-20T10:15:00Z",
                "exit_price": 1180.25
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["outcome"]["status"], "REFERENCE_RISK");
        assert_eq!(body["decision"]["outcome"]["exit_price"], 1180.25);
    }

    #[tokio::test]
    async fn horizon_outcome_is_recorded() {
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/outcome")
            .json(&json!({
                "status": "HORIZON",
                "observation_boundary_passed": true,
                "exit_timestamp": "2026-08-24T10:15:00Z"
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["outcome"]["status"], "HORIZON");
    }
}