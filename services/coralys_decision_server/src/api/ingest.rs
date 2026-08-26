//! MVP-003 ingest — `POST /decisions`
//!
//! Accepts a `SealedDecisionInput` (JSON), builds a `DecisionRecord` via
//! `DecisionRecordBuilder`, and seals it into the shared `DecisionLedger`.
//!
//! This is the only path by which new certified decisions enter the server.
//! The C3-002 pipeline (csp006_p_emit) calls this endpoint after generating
//! today's decisions.
//!
//! **Invariants preserved:**
//! - AC-01: all certification fields must be present.
//! - AC-02: temporal firewall — certified_timestamp ≥ decision_timestamp.
//! - AC-03: once sealed, the decision is immutable.
//! - MVP-004: provenance hashes are verified against canonical C3-002 values.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use coralys_decision::{DecisionRecordBuilder, SealedDecisionInput};
use serde::{Deserialize, Serialize};

use crate::AppState;

// ─── Request body ─────────────────────────────────────────────────────────────

/// JSON body for `POST /decisions`.
///
/// Maps 1-to-1 onto `SealedDecisionInput`. All fields that are `Option` in
/// the input struct are also optional here.
#[derive(Debug, Clone, Deserialize)]
pub struct IngestRequest {
    pub decision_id: String,
    pub instrument: String,
    pub decision_timestamp: DateTime<Utc>,
    pub direction: String,
    pub trend: String,
    pub momentum: String,
    pub volatility: String,
    pub target_price: Option<f64>,
    pub policy_artifact_hash: String,
    pub execution_artifact_hash: Option<String>,
    pub decision_pipeline: String,
    pub data_snapshot_id: String,
    pub certified_timestamp: DateTime<Utc>,
    pub reference_risk_boundary_price: Option<f64>,
    pub reference_risk_boundary_type: String,
    /// ATR-14 in price units at decision time T (from bar data, certified ≤ T).
    #[serde(default)]
    pub atr_14: Option<f64>,
    /// Previous close / LTP at decision time T — the reference price for the recommendation.
    #[serde(default)]
    pub reference_price: Option<f64>,
    /// Next NSE trading session date (YYYY-MM-DD) this decision applies to.
    #[serde(default)]
    pub effective_session: Option<String>,
}

// ─── Response bodies ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct IngestOkResponse {
    pub decision_id: String,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct IngestErrorResponse {
    pub error: String,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// `POST /decisions` — seal a new certified decision into the ledger.
///
/// Returns 201 Created on success.
/// Returns 409 Conflict if the decision_id already exists.
/// Returns 422 Unprocessable Entity on provenance or temporal firewall failure.
pub async fn ingest_decision(
    State(state): State<AppState>,
    Json(body): Json<IngestRequest>,
) -> impl IntoResponse {
    let decision_id = body.decision_id.clone();

    let input = SealedDecisionInput {
        decision_id: body.decision_id,
        instrument: body.instrument,
        decision_timestamp: body.decision_timestamp,
        direction: body.direction,
        trend: body.trend,
        momentum: body.momentum,
        volatility: body.volatility,
        target_price: body.target_price,
        policy_artifact_hash: body.policy_artifact_hash,
        execution_artifact_hash: body.execution_artifact_hash,
        decision_pipeline: body.decision_pipeline,
        data_snapshot_id: body.data_snapshot_id,
        certified_timestamp: body.certified_timestamp,
        reference_risk_boundary_price: body.reference_risk_boundary_price,
        reference_risk_boundary_type: body.reference_risk_boundary_type,
        atr_14: body.atr_14,
        reference_price: body.reference_price,
        effective_session: body.effective_session,
    };

    // Build the DecisionRecord — enforces AC-01, AC-02, MVP-004.
    let record = match DecisionRecordBuilder::build(input) {
        Ok(r) => r,
        Err(e) => {
            let err = IngestErrorResponse {
                error: e.to_string(),
            };
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::to_value(err).unwrap()),
            )
                .into_response();
        }
    };

    // Seal into the ledger — enforces AC-03 (duplicate rejection).
    let mut ledger = state.ledger.write().await;
    match ledger.seal_decision(record) {
        Ok(()) => {
            let ok = IngestOkResponse {
                decision_id,
                status: "SEALED",
            };
            (StatusCode::CREATED, Json(serde_json::to_value(ok).unwrap())).into_response()
        }
        Err(coralys_decision::LedgerError::DecisionAlreadyExists(_)) => {
            let err = IngestErrorResponse {
                error: format!("decision '{decision_id}' already exists in ledger"),
            };
            (
                StatusCode::CONFLICT,
                Json(serde_json::to_value(err).unwrap()),
            )
                .into_response()
        }
        Err(e) => {
            let err = IngestErrorResponse {
                error: e.to_string(),
            };
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::to_value(err).unwrap()),
            )
                .into_response()
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use coralys_decision::{C3_002_POLICY_ARTIFACT_HASH, CORALYS_EXEC_ARTIFACT_HASH};
    use serde_json::{Value, json};

    fn canonical_body() -> Value {
        json!({
            "decision_id": "coralys-ADANIENT-20260817T101500Z-001",
            "instrument": "ADANIENT.NS",
            "decision_timestamp": "2026-08-17T10:15:00Z",
            "direction": "LONG",
            "trend": "Bullish",
            "momentum": "Positive",
            "volatility": "present",
            "target_price": 1234.50,
            "policy_artifact_hash": C3_002_POLICY_ARTIFACT_HASH,
            "execution_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
            "decision_pipeline": "C3-002",
            "data_snapshot_id": "snapshot-20260817T101500Z",
            "certified_timestamp": "2026-08-17T10:15:00Z",
            "reference_risk_boundary_price": 1180.25,
            "reference_risk_boundary_type": "CORALYS_V0_ATR_TMV"
        })
    }

    #[tokio::test]
    async fn ingest_valid_decision_returns_201() {
        let server = TestServer::new(crate::test_helpers::make_app().await);
        let resp = server.post("/decisions").json(&canonical_body()).await;
        resp.assert_status(StatusCode::CREATED);
        let body: Value = resp.json();
        assert_eq!(body["status"], "SEALED");
        assert_eq!(body["decision_id"], "coralys-ADANIENT-20260817T101500Z-001");
    }

    #[tokio::test]
    async fn ingested_decision_appears_in_feed() {
        let server = TestServer::new(crate::test_helpers::make_app().await);
        server.post("/decisions").json(&canonical_body()).await;
        let resp = server.get("/decisions").await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(body["total"], 1);
    }

    #[tokio::test]
    async fn duplicate_decision_returns_409() {
        let server = TestServer::new(crate::test_helpers::make_app().await);
        server.post("/decisions").json(&canonical_body()).await;
        let resp = server.post("/decisions").json(&canonical_body()).await;
        resp.assert_status(StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn wrong_policy_hash_returns_422() {
        let server = TestServer::new(crate::test_helpers::make_app().await);
        let mut body = canonical_body();
        body["policy_artifact_hash"] = json!("wrong-hash");
        let resp = server.post("/decisions").json(&body).await;
        resp.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}
