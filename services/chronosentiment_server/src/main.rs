use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use chronosentiment_adapter::evidence::{EvidenceItem, EvidenceSourceType};
use chronosentiment_adapter::hypothesis::InvestmentThesis;
use chronosentiment_adapter::observation::Observation;
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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("ChronoSentiment Research Sessions API running on http://0.0.0.0:3000");
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
    Json(observation): Json<Observation>,
) -> Result<StatusCode, StatusCode> {
    let mut store = state.workspaces.write().await;
    if let Some(ws) = store.get_mut(&id) {
        // Map observation to evidence
        let evidence = EvidenceItem::new(
            observation.id.to_string(),
            id.clone(),
            format!("Observation: {}", observation.observation_type),
            EvidenceSourceType::MarketData,
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
