//! MVP-006 — Decision Detail API: `GET /decisions/{id}`
//!
//! Returns the complete certified `DecisionRecord` for a given `decision_id`,
//! retrieved directly from the immutable ledger.
//!
//! **The API does not reconstruct the decision from C3-002 at request time.**
//! It exposes the already-certified ledger record.
//!
//! **Acceptance criteria:**
//! - AC-D1: Returns 200 + full record for a known decision_id.
//! - AC-D2: Returns 404 for an unknown decision_id.
//! - AC-D3: All four provenance fields are present in the response.
//! - AC-D4: decision_timestamp matches the sealed record.
//! - AC-D5: certification_status is CERTIFIED.
//! - AC-D6: execution defaults to NOT_RECORDED.
//! - AC-D7: outcome defaults to OPEN.
//! - AC-D8: evidence fields are null until enriched.
//! - AC-D9: No confidence, probability, ranking, or allocation fields.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use super::DecisionResponse;
use crate::AppState;

// ─── Detail response ──────────────────────────────────────────────────────────

/// Response envelope for `GET /decisions/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailResponse {
    pub decision: DecisionResponse,
}

/// Error response for 404.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotFoundResponse {
    pub error: String,
    pub decision_id: String,
}

// ─── Handler ──────────────────────────────────────────────────────────────────

/// `GET /decisions/{id}`
///
/// Retrieves the complete certified `DecisionRecord` from the ledger.
/// Returns 404 if the decision_id is not found.
pub async fn get_decision_by_id(
    State(state): State<AppState>,
    Path(decision_id): Path<String>,
) -> impl IntoResponse {
    let ledger = state.ledger.read().await;

    match ledger.get_decision(&decision_id) {
        Ok(record) => {
            let response = DetailResponse {
                decision: DecisionResponse::from(record),
            };
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
        }
        Err(_) => {
            let not_found = NotFoundResponse {
                error: "decision not found".to_string(),
                decision_id: decision_id.clone(),
            };
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::to_value(not_found).unwrap()),
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
    use serde_json::Value;

    #[tokio::test]
    async fn known_decision_returns_200_with_full_record() {
        // AC-D1
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        resp.assert_status_ok();
        let body: Value = resp.json();
        assert_eq!(
            body["decision"]["identity"]["decision_id"],
            "coralys-ADANIENT-20260817T101500Z-001"
        );
        assert_eq!(body["decision"]["identity"]["instrument"], "ADANIENT.NS");
    }

    #[tokio::test]
    async fn unknown_decision_returns_404() {
        // AC-D2
        let (app, _state) = make_app_with_state().await;
        let server = TestServer::new(app);
        let resp = server.get("/decisions/nonexistent-id").await;
        resp.assert_status(StatusCode::NOT_FOUND);
        let body: Value = resp.json();
        assert_eq!(body["error"], "decision not found");
        assert_eq!(body["decision_id"], "nonexistent-id");
    }

    #[tokio::test]
    async fn all_four_provenance_fields_are_present() {
        // AC-D3
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        let body: Value = resp.json();
        let cert = &body["decision"]["certification"];
        assert!(!cert["policy_artifact_hash"].as_str().unwrap_or("").is_empty());
        assert!(!cert["decision_pipeline"].as_str().unwrap_or("").is_empty());
        assert!(!cert["data_snapshot_id"].as_str().unwrap_or("").is_empty());
        // execution_artifact_hash may be null (optional) but key must exist.
        assert!(cert.get("execution_artifact_hash").is_some());
    }

    #[tokio::test]
    async fn certification_status_is_certified() {
        // AC-D5
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        let body: Value = resp.json();
        assert_eq!(
            body["decision"]["certification"]["status"],
            "CERTIFIED"
        );
    }

    #[tokio::test]
    async fn execution_defaults_to_not_recorded() {
        // AC-D6
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        let body: Value = resp.json();
        assert_eq!(body["decision"]["execution"]["status"], "NOT_RECORDED");
        assert!(body["decision"]["execution"]["quantity"].is_null());
    }

    #[tokio::test]
    async fn outcome_defaults_to_open() {
        // AC-D7
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        let body: Value = resp.json();
        assert_eq!(body["decision"]["outcome"]["status"], "OPEN");
    }

    #[tokio::test]
    async fn evidence_fields_are_null_before_enrichment() {
        // AC-D8
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        let body: Value = resp.json();
        let ev = &body["decision"]["evidence"];
        assert!(ev["similar_decisions_count"].is_null());
        assert!(ev["median_mae_pct"].is_null());
        assert!(ev["p90_mae_pct"].is_null());
    }

    #[tokio::test]
    async fn no_confidence_or_allocation_in_detail_response() {
        // AC-D9
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        let raw: Value = resp.json();
        let text = serde_json::to_string(&raw).unwrap();
        assert!(!text.contains("confidence"));
        assert!(!text.contains("probability"));
        assert!(!text.contains("allocation"));
        assert!(!text.contains("capital"));
        assert!(!text.contains("ranking"));
    }

    #[tokio::test]
    async fn decision_timestamp_matches_sealed_record() {
        // AC-D4
        let (app, state) = make_app_with_state().await;
        seal_sample_decision(&state, "coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS").await;

        let server = TestServer::new(app);
        let resp = server
            .get("/decisions/coralys-ADANIENT-20260817T101500Z-001")
            .await;
        let body: Value = resp.json();
        let ts = body["decision"]["identity"]["decision_timestamp"]
            .as_str()
            .unwrap();
        assert!(ts.contains("2026-08-17"));
    }
}