mod decisions_api;
mod portfolio_api;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use chronosentiment_adapter::evidence::{EvidenceItem, EvidenceSourceType};
use chronosentiment_adapter::hypothesis::InvestmentThesis;
use chronosentiment_adapter::workspace::InvestmentWorkspace;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// In-memory store for Phase 1 milestone.
// (Will be replaced with PostgresWorkspaceRepository).
type WorkspaceStore = Arc<RwLock<HashMap<String, InvestmentWorkspace>>>;

#[derive(Clone)]
struct AppState {
    workspaces: WorkspaceStore,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        workspaces: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/research-sessions", post(create_session))
        .route("/research-sessions/:id", get(get_session))
        .route("/research-sessions/:id/observations", post(add_observation))
        .route("/research-sessions/:id/hypotheses", post(add_hypothesis))
        // Product MVP v0.2 — Certified decisions (backend-owned intelligence)
        .route(
            "/api/v0/decisions/current",
            get(decisions_api::get_current_decisions),
        )
        // Product MVP v0.2 — Portfolio Recommendations (no decisions[] in body)
        .route(
            "/api/v0/portfolio/recommendations",
            post(portfolio_api::post_recommendations),
        )
        .with_state(state);

    // Port is configurable via CHRONOSENTIMENT_PORT env var (default: 8080).
    // Next.js UI runs on port 3000, so the backend uses 8080 by default.
    let port = std::env::var("CHRONOSENTIMENT_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("ChronoSentiment Research Sessions API running on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct CreateSessionReq {
    subject: String,
    portfolio: String,
    research_objective: String,
}

async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionReq>,
) -> (StatusCode, Json<InvestmentWorkspace>) {
    let id = Uuid::new_v4().to_string();
    let ws = InvestmentWorkspace::new(
        id.clone(),
        payload.subject,
        payload.portfolio,
        payload.research_objective,
        Utc::now().timestamp() as u64,
    );

    state.workspaces.write().await.insert(id, ws.clone());
    (StatusCode::CREATED, Json(ws))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<InvestmentWorkspace>, StatusCode> {
    let store = state.workspaces.read().await;
    match store.get(&id) {
        Some(ws) => Ok(Json(ws.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn add_observation(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let mut store = state.workspaces.write().await;
    if let Some(ws) = store.get_mut(&id) {
        let obs_id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let obs_type = payload
            .get("observation_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let evidence = EvidenceItem::new(
            obs_id,
            id.clone(),
            format!("Observation: {obs_type}"),
            EvidenceSourceType::FinancialData,
            "Mapped from Observation envelope",
            Utc::now().timestamp() as u64,
        );
        ws.add_evidence(evidence, Utc::now().timestamp() as u64);
        Ok(StatusCode::CREATED)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn add_hypothesis(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(thesis): Json<InvestmentThesis>,
) -> Result<StatusCode, StatusCode> {
    let mut store = state.workspaces.write().await;
    if let Some(ws) = store.get_mut(&id) {
        ws.add_thesis_version(thesis, Utc::now().timestamp() as u64);
        Ok(StatusCode::CREATED)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
