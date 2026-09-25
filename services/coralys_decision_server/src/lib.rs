//! Coralys Decision Server Library interface.

pub mod api;
pub mod protection;

use std::sync::Arc;
use axum::{Router, routing::{get, post}};
use coralys_decision::DecisionLedger;
use coralys_decision::recommendation::{EvidenceStore, Rec001hStore};
use tokio::sync::RwLock;

pub use protection::{
    GeneratorError, PositionTrajectoryState, ProtectionAction, ProtectionDecision,
    ProtectionShadowEvent, ProtectionShadowLedger, ProtectionStrategyGenerator,
};

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub ledger: Arc<RwLock<DecisionLedger>>,
    pub evidence_store: Option<Arc<EvidenceStore>>,
    pub rec001h_store: Option<Arc<Rec001hStore>>,
    pub shadow_ledger: Arc<ProtectionShadowLedger>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ledger: Arc::new(RwLock::new(DecisionLedger::new())),
            evidence_store: None,
            rec001h_store: None,
            shadow_ledger: Arc::new(ProtectionShadowLedger::new()),
        }
    }

    pub fn with_evidence(evidence_store: EvidenceStore) -> Self {
        Self {
            ledger: Arc::new(RwLock::new(DecisionLedger::new())),
            evidence_store: Some(Arc::new(evidence_store)),
            rec001h_store: None,
            shadow_ledger: Arc::new(ProtectionShadowLedger::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the Axum router with all routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/decisions",
            get(api::feed::get_decisions).post(api::ingest::ingest_decision),
        )
        .route("/decisions/{id}", get(api::detail::get_decision_by_id))
        .route(
            "/decisions/{id}/execution",
            post(api::execution::record_execution),
        )
        .route("/decisions/{id}/outcome", post(api::outcome::record_outcome))
        .route(
            "/recommendations/latest",
            get(api::recommendations::get_recommendations_latest),
        )
        .route(
            "/recommendations/v1/latest",
            get(api::recommendations_v1::get_recommendations_v1_latest),
        )
        .route(
            "/recommendations/v1/history",
            get(api::recommendations_v1::get_recommendations_v1_history),
        )
        .with_state(state)
}

// ─── Test helpers ─────────────────────────────────────────────────────────────

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use chrono::{TimeZone, Utc};
    use coralys_decision::{
        C3_002_POLICY_ARTIFACT_HASH, CORALYS_EXEC_ARTIFACT_HASH, DecisionRecordBuilder,
        SealedDecisionInput,
    };

    pub async fn make_app() -> Router {
        let state = AppState::new();
        build_router(state)
    }

    pub async fn make_app_with_state() -> (Router, AppState) {
        let state = AppState::new();
        let app = build_router(state.clone());
        (app, state)
    }

    pub async fn make_app_with_rec001h() -> Router {
        let rec001h_dir = std::env::var("REC001H_DIR")
            .unwrap_or_else(|_| "datasets/recommendation/historical".to_string());
        let rec001h_store = Rec001hStore::load_from_dir(&rec001h_dir)
            .ok()
            .map(|s| std::sync::Arc::new(s));
        let state = AppState {
            ledger: std::sync::Arc::new(tokio::sync::RwLock::new(
                coralys_decision::DecisionLedger::new(),
            )),
            evidence_store: None,
            rec001h_store,
            shadow_ledger: std::sync::Arc::new(ProtectionShadowLedger::new()),
        };
        build_router(state)
    }

    pub async fn seal_sample_decision(state: &AppState, decision_id: &str, instrument: &str) {
        let minute = if instrument.contains("ADANIENT") {
            15
        } else if instrument.contains("BPCL") {
            30
        } else {
            45
        };
        let decision_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, minute, 0).unwrap();

        let input = SealedDecisionInput {
            decision_id: decision_id.to_string(),
            instrument: instrument.to_string(),
            decision_timestamp: decision_ts,
            direction: "LONG".to_string(),
            trend: "Bullish".to_string(),
            momentum: "Positive".to_string(),
            volatility: "present".to_string(),
            target_price: Some(1234.50),
            policy_artifact_hash: C3_002_POLICY_ARTIFACT_HASH.to_string(),
            execution_artifact_hash: Some(CORALYS_EXEC_ARTIFACT_HASH.to_string()),
            decision_pipeline: "C3-002".to_string(),
            data_snapshot_id: format!("snapshot-20260817T10{minute:02}00Z"),
            certified_timestamp: decision_ts,
            reference_risk_boundary_price: Some(1180.25),
            reference_risk_boundary_type: "CORALYS_V0_ATR_TMV".to_string(),
            atr_14: None,
            reference_price: None,
            effective_session: None,
        };

        let record = DecisionRecordBuilder::build(input).unwrap();
        let mut ledger = state.ledger.write().await;
        ledger.seal_decision(record).unwrap();
    }
}
