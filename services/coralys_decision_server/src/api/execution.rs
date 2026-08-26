//! MVP-007 — User Execution Recording: `POST /decisions/{id}/execution`
//!
//! Records what the user actually did after receiving a certified decision.
//! This is a user-reported action — Coralys does not infer, calculate, or
//! recommend quantity, allocation, or capital deployment.
//!
//! **Acceptance criteria:**
//! - AC-E1: POST execution for valid decision → 200 with updated record.
//! - AC-E2: Unknown decision → 404.
//! - AC-E3: `quantity: null` accepted (no quantity inference).
//! - AC-E4: No quantity/allocation inference in any code path.
//! - AC-E5: Execution timestamp before decision timestamp → 422.
//! - AC-E6: Execution lifecycle event appears in ledger.
//! - AC-E7: Original certified decision fields remain unchanged.
//! - AC-E8: USER_IGNORED is a valid execution status.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use coralys_decision::record::{ExecutionRecord, ExecutionStatus};

use super::DecisionResponse;
use crate::AppState;

// ─── Request ──────────────────────────────────────────────────────────────────

/// Request body for `POST /decisions/{id}/execution`.
///
/// The user tells Coralys what they did. Coralys records it verbatim.
///
/// **No fields are inferred.** `quantity` and `execution_price` are only
/// recorded when the user explicitly supplies them.
#[derive(Debug, Clone, Deserialize)]
pub struct RecordExecutionRequest {
    /// `USER_EXECUTED` or `USER_IGNORED` or `USER_CANCELLED`.
    pub status: ExecutionStatusRequest,
    /// When the user acted. Required for USER_EXECUTED; optional for others.
    pub execution_timestamp: Option<DateTime<Utc>>,
    /// Quantity traded. Only present when the user explicitly supplies it.
    /// Coralys never infers this from capital, rank, or signal strength.
    pub quantity: Option<f64>,
    /// Execution price. Only present when the user explicitly supplies it.
    pub execution_price: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionStatusRequest {
    UserExecuted,
    UserIgnored,
    UserCancelled,
}

impl From<ExecutionStatusRequest> for ExecutionStatus {
    fn from(s: ExecutionStatusRequest) -> Self {
        match s {
            ExecutionStatusRequest::UserExecuted => ExecutionStatus::UserExecuted,
            ExecutionStatusRequest::UserIgnored => ExecutionStatus::UserIgnored,
            ExecutionStatusRequest::UserCancelled => ExecutionStatus::UserCancelled,
        }
    }
}

// ─── Response ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionRecordedResponse {
    pub decision: DecisionResponse,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub decision_id: String,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// `POST /decisions/{id}/execution`
///
/// Records a user execution action for a certified decision.
/// The original certified decision is never modified.
pub async fn record_execution(
    State(state): State<AppState>,
    Path(decision_id): Path<String>,
    Json(body): Json<RecordExecutionRequest>,
) -> impl IntoResponse {
    let mut ledger = state.ledger.write().await;

    // Resolve the event timestamp: use supplied timestamp, or fall back to the
    // decision timestamp itself (the earliest valid event time). Using Utc::now()
    // would fail the temporal firewall when the server clock is behind the
    // sealed decision timestamp (e.g. in tests or when recording USER_IGNORED
    // retroactively).
    let event_ts = match body.execution_timestamp {
        Some(ts) => ts,
        None => {
            // Look up the decision timestamp as the floor.
            match ledger.get_decision(&decision_id) {
                Ok(r) => r.identity.decision_timestamp,
                Err(_) => Utc::now(),
            }
        }
    };

    let execution = ExecutionRecord {
        status: body.status.into(),
        execution_timestamp: body.execution_timestamp,
        quantity: body.quantity,
        execution_price: body.execution_price,
        execution_source: Some("USER".to_string()),
    };

    match ledger.record_execution(&decision_id, execution, event_ts) {
        Ok(()) => {
            let record = ledger.get_decision(&decision_id).unwrap();
            let response = ExecutionRecordedResponse {
                decision: DecisionResponse::from(record),
                message: "Execution recorded.".to_string(),
            };
            (
                StatusCode::OK,
                Json(serde_json::to_value(response).unwrap()),
            )
                .into_response()
        }
        Err(coralys_decision::ledger::LedgerError::DecisionNotFound(_)) => {
            let err = ErrorResponse {
                error: "decision not found".to_string(),
                decision_id: decision_id.clone(),
            };
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::to_value(err).unwrap()),
            )
                .into_response()
        }
        Err(coralys_decision::ledger::LedgerError::TemporalFirewallViolation { .. }) => {
            let err = ErrorResponse {
                error: "execution timestamp is before the decision timestamp".to_string(),
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
    async fn post_execution_for_valid_decision_returns_200() {
        // AC-E1
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/execution")
            .json(&json!({
                "status": "USER_EXECUTED",
                "execution_timestamp": "2026-08-17T10:20:00Z"
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["execution"]["status"], "USER_EXECUTED");
    }

    #[tokio::test]
    async fn unknown_decision_returns_404() {
        // AC-E2
        let (app, _state) = make_app_with_state().await;
        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/nonexistent/execution")
            .json(&json!({ "status": "USER_EXECUTED" }))
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn null_quantity_is_accepted() {
        // AC-E3: no quantity inference
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/execution")
            .json(&json!({
                "status": "USER_EXECUTED",
                "execution_timestamp": "2026-08-17T10:20:00Z",
                "quantity": null
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert!(body["decision"]["execution"]["quantity"].is_null());
    }

    #[tokio::test]
    async fn execution_timestamp_before_decision_returns_422() {
        // AC-E5: temporal firewall
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/execution")
            .json(&json!({
                "status": "USER_EXECUTED",
                // Before the decision timestamp of 2026-08-17T10:15:00Z
                "execution_timestamp": "2026-08-17T09:00:00Z"
            }))
            .await;
        resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn user_ignored_is_valid_execution_status() {
        // AC-E8
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/execution")
            .json(&json!({ "status": "USER_IGNORED" }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["execution"]["status"], "USER_IGNORED");
    }

    #[tokio::test]
    async fn original_certified_decision_fields_unchanged() {
        // AC-E7: immutability of original decision
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/execution")
            .json(&json!({
                "status": "USER_EXECUTED",
                "execution_timestamp": "2026-08-17T10:20:00Z"
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();

        // Original decision fields must be unchanged.
        assert_eq!(body["decision"]["identity"]["instrument"], "ADANIENT.NS");
        assert_eq!(body["decision"]["certification"]["status"], "CERTIFIED");
        assert_eq!(body["decision"]["decision"]["direction"], "LONG");
        assert_eq!(body["decision"]["decision"]["trend"], "Bullish");
    }

    #[tokio::test]
    async fn execution_source_is_always_user() {
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(
            &state,
            "coralys-ADANIENT-20260817T101500Z-001",
            "ADANIENT.NS",
        )
        .await;

        let server = TestServer::new(app);
        let resp = server
            .post("/decisions/coralys-ADANIENT-20260817T101500Z-001/execution")
            .json(&json!({
                "status": "USER_EXECUTED",
                "execution_timestamp": "2026-08-17T10:20:00Z"
            }))
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["decision"]["execution"]["execution_source"], "USER");
    }
}
