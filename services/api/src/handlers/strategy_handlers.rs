use axum::{extract::{State, Path}, Json};

use crate::{dto::{CompareStrategiesRequest, CompareStrategiesResponse, EvaluateStrategyRequest, EvaluateStrategyResponse, InspectStrategyResponse, InspectStrategyRequest, RunGaResponse, TimelineResponse, SystemState, TradeInspectorResponse, EventWrapper},
    errors::ApiError,
    services::evaluation_service::EvaluationService,
    strategy_id_parse::parse_strategy_id_full,
};
use serde_json;


pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

pub async fn timeline_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<TimelineResponse>, ApiError> {
    println!("Request received: timeline");
    let events = service.get_timeline()?;
    Ok(Json(TimelineResponse { events }))
}

pub async fn events_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<Vec<EventWrapper>>, ApiError> {
    println!("Request received: events");
    let events = service.get_timeline()?;
    Ok(Json(events))
}

pub async fn replay_handler(
    State(service): State<EvaluationService>,
    Path(seq_id): Path<u64>,
) -> Result<Json<SystemState>, ApiError> {
    println!("Request received: replay/{}", seq_id);
    let state = service.get_replay(seq_id)?;
    Ok(Json(state))
}

#[derive(serde::Deserialize)]
pub struct OrderInspectionParams {
    #[serde(default)]
    pub include_chain: bool,
}

pub async fn order_inspection_handler(
    State(service): State<EvaluationService>,
    Path(order_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<OrderInspectionParams>,
) -> Result<Json<TradeInspectorResponse>, ApiError> {
    println!("Request received: order/{}", order_id);
    let response = service.get_order_inspection(order_id, params.include_chain)?;
    Ok(Json(response))
}

pub async fn evaluate_strategy_handler(
    State(service): State<EvaluationService>,
    Json(request): Json<EvaluateStrategyRequest>,
) -> Result<Json<EvaluateStrategyResponse>, ApiError> {
    println!("Request received: evaluate_strategy");
    let response = service.evaluate_strategy(
        request.strategy_config,
        request.scenarios,
        request.seed,
    )?;
    println!("Evaluation completed for strategy_id: {}", response.strategy_evaluation.strategy_id);
    Ok(Json(response))
}

pub async fn compare_strategies_handler(
    State(service): State<EvaluationService>,
    Json(request): Json<CompareStrategiesRequest>,
) -> Result<Json<CompareStrategiesResponse>, ApiError> {
    println!("Request received: compare_strategies");
    let strategies: Vec<chronosentiment_core::Strategy> = request
        .strategies
        .into_iter()
        .map(|w| w.strategy_config)
        .collect();

    let response = service.compare_strategies(
        strategies,
        request.scenarios,
        request.seed,
    )?;
    println!("Comparison completed");
    Ok(Json(response))
}

pub async fn inspect_strategy_handler(
    State(service): State<EvaluationService>,
    Json(request): Json<InspectStrategyRequest>,
) -> Result<Json<InspectStrategyResponse>, ApiError> {
    println!("Request received: inspect_strategy");

    let (strategy_config, scenario_from_id) = if let Some(cfg) = request.strategy_config {
        (cfg, None)
    } else if let Some(id) = request.strategy_id.as_deref() {
        parse_strategy_id_full(id).map_err(ApiError::ValidationError)?
    } else {
        return Err(ApiError::ValidationError(
            "inspect_strategy requires either strategy_config or strategy_id".to_string(),
        ));
    };

    // Prefer explicit scenarios[]; else scenario embedded in strategy_id; else first benchmark name (lexicographic)
    let scenario = if let Some(s) = request.scenarios.into_iter().next() {
        s
    } else if let Some(s) = scenario_from_id {
        s
    } else {
        service.load_all_real_scenarios()
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| ApiError::EngineError("No real market scenarios available in test_assets".to_string()))?
    };

    let response = service.inspect_strategy(
        strategy_config,
        scenario,
        request.seed,
    )?;
    println!("Inspection completed for strategy_id: {}", response.strategy_id);
    Ok(Json(response))
}

pub async fn test_determinism_handler(
    State(service): State<EvaluationService>,
    Json(request): Json<EvaluateStrategyRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    println!("Request received: test_determinism");
    let is_deterministic = service.test_determinism(
        request.strategy_config,
        request.scenarios,
        request.seed,
    )?;

    Ok(Json(serde_json::json!({ "deterministic": is_deterministic })))
}

pub async fn run_ga_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<RunGaResponse>, ApiError> {
    println!("RUN_GA_ENDPOINT_HIT");
    let response = service.run_ga()?;
    println!("GA run completed");
    Ok(Json(response))
}

pub async fn get_global_ranking_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<Vec<crate::dto::StrategyEvaluationDto>>, ApiError> {
    println!("Request received: get_global_ranking");
    let ranking = service.get_global_ranking()?;
    Ok(Json(ranking))
}

/// Latest on-disk `PersistedStrategyStore` JSON (reloads from disk each request).
pub async fn get_strategy_store_handler(
    State(_service): State<EvaluationService>,
) -> Json<serde_json::Value> {
    println!("Request received: ga/strategy-store");
    let path = EvaluationService::STRATEGY_STORE_PATH;
    let store = match chronosentiment_core::pipeline::load_strategy_store(path) {
        Ok(s) => serde_json::to_value(&s).unwrap_or(serde_json::Value::Null),
        Err(_) => serde_json::Value::Null,
    };
    Json(serde_json::json!({
        "path": path,
        "store": store,
    }))
}

pub async fn latest_signals_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<crate::dto::SignalsSnapshotDto>, ApiError> {
    println!("Request received: signals/latest");
    let snapshot = service.get_latest_signals()?;
    Ok(Json(snapshot))
}

pub async fn trade_suggestions_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<crate::dto::TradeSuggestionsResponse>, ApiError> {
    println!("Request received: signals/trade-suggestions");
    let response = service.get_trade_suggestions()?;
    Ok(Json(response))
}

#[derive(serde::Deserialize)]
pub struct ReplaySuggestionsParams {
    #[serde(default)]
    pub mode: Option<String>, // summary | sampled | full
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub sample_rate: Option<usize>,
    #[serde(default)]
    pub include_full: Option<bool>,
}

pub async fn replay_suggestions_handler(
    State(service): State<EvaluationService>,
    axum::extract::Query(params): axum::extract::Query<ReplaySuggestionsParams>,
) -> Result<Json<crate::dto::ReplaySuggestionsResponse>, ApiError> {
    println!("Request received: signals/replay-suggestions");
    let mode = params.mode.unwrap_or_else(|| "summary".to_string());
    let limit = params.limit.unwrap_or(1000);
    let sample_rate = params.sample_rate.unwrap_or(10);
    let include_full = params.include_full.unwrap_or(false);
    let response = service.get_replay_suggestions(mode, limit, sample_rate, include_full)?;
    Ok(Json(response))
}
