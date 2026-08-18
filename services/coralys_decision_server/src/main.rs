//! Coralys Decision Intelligence API server.
//!
//! MVP-005: `GET /decisions`                — Decision Feed
//! MVP-006: `GET /decisions/{id}`           — Decision Detail
//! MVP-009: `GET /recommendations/latest`   — Ranked Recommendation Snapshot
//!
//! Architecture:
//! ```text
//! DecisionLedger (shared, RwLock-protected)
//! EvidenceStore  (loaded once at startup from HDV-001 outcomes file)
//!         │
//!         ▼
//! Axum router
//!         │
//!         ├── GET /decisions                → feed::get_decisions
//!         ├── GET /decisions/{id}           → detail::get_decision_by_id
//!         └── GET /recommendations/latest   → recommendations::get_recommendations_latest
//! ```
//!
//! The ledger is the authoritative source. No decisions are reconstructed
//! from C3-002 at request time.

mod api;

use std::sync::Arc;

use axum::{Router, routing::{get, post}};
use coralys_decision::DecisionLedger;
use coralys_decision::recommendation::EvidenceStore;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing_subscriber::{EnvFilter, fmt};

// ─── Shared state ─────────────────────────────────────────────────────────────

/// Shared application state.
///
/// - `ledger` — the `DecisionLedger` is the single source of truth for all
///   certified decisions. `RwLock` allows concurrent reads with exclusive writes.
/// - `evidence_store` — the frozen HDV-001 analogue index, loaded once at
///   startup. `None` when the outcomes file is unavailable (server still starts
///   but `/recommendations/latest` returns 503).
#[derive(Clone)]
pub struct AppState {
    pub ledger: Arc<RwLock<DecisionLedger>>,
    pub evidence_store: Option<Arc<EvidenceStore>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            ledger: Arc::new(RwLock::new(DecisionLedger::new())),
            evidence_store: None,
        }
    }

    /// Build state with a pre-loaded `EvidenceStore`.
    pub fn with_evidence(evidence_store: EvidenceStore) -> Self {
        Self {
            ledger: Arc::new(RwLock::new(DecisionLedger::new())),
            evidence_store: Some(Arc::new(evidence_store)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Router ───────────────────────────────────────────────────────────────────

/// Build the Axum router with all routes.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/decisions",
            get(api::feed::get_decisions).post(api::ingest::ingest_decision),
        )
        .route(
            "/decisions/{id}",
            get(api::detail::get_decision_by_id),
        )
        .route(
            "/decisions/{id}/execution",
            post(api::execution::record_execution),
        )
        .route(
            "/decisions/{id}/outcome",
            post(api::outcome::record_outcome),
        )
        .route(
            "/recommendations/latest",
            get(api::recommendations::get_recommendations_latest),
        )
        .with_state(state)
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load the frozen HDV-001 evidence store.
    // Path is relative to the workspace root; the server is expected to be
    // started from the repository root (or the path overridden via env).
    let outcomes_path = std::env::var("HDV001_OUTCOMES_PATH")
        .unwrap_or_else(|_| "datasets/hdv001/hdv001_outcomes_v1.json".to_string());

    let evidence_store = match EvidenceStore::load_from_file(&outcomes_path) {
        Ok(store) => {
            tracing::info!(
                path = %outcomes_path,
                "HDV-001 evidence store loaded"
            );
            Some(Arc::new(store))
        }
        Err(e) => {
            tracing::warn!(
                path = %outcomes_path,
                error = %e,
                "HDV-001 outcomes file not found — /recommendations/latest will return 503"
            );
            None
        }
    };

    let state = AppState {
        ledger: Arc::new(RwLock::new(DecisionLedger::new())),
        evidence_store,
    };

    let app = build_router(state);

    let listener = TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("failed to bind port 3001");

    tracing::info!("Coralys Decision Intelligence API listening on :3001");
    axum::serve(listener, app)
        .await
        .expect("server error");
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

    /// Build a test app with an empty ledger.
    pub async fn make_app() -> Router {
        let state = AppState::new();
        build_router(state)
    }

    /// Build a test app and return both the router and the shared state so
    /// tests can pre-populate the ledger.
    pub async fn make_app_with_state() -> (Router, AppState) {
        let state = AppState::new();
        let app = build_router(state.clone());
        (app, state)
    }

    /// Seal a sample decision into the ledger for testing.
    ///
    /// Uses the canonical C3-002 and Coralys exec hashes so strict provenance
    /// verification passes. The `decision_id` and `instrument` are
    /// caller-supplied to allow multiple distinct decisions in one test.
    ///
    /// Timestamps are offset by `seq` minutes from 2026-08-17T10:15:00Z so
    /// that ordering tests work correctly.
    pub async fn seal_sample_decision(state: &AppState, decision_id: &str, instrument: &str) {
        // Derive a unique timestamp from the decision_id to ensure ordering.
        // ADANIENT → minute 15, BPCL → minute 30, others → minute 45.
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