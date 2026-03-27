use axum::{extract::{State, Path}, Json};

use crate::{dto::{CompareStrategiesRequest, CompareStrategiesResponse, EvaluateStrategyRequest, EvaluateStrategyResponse, InspectStrategyResponse, InspectStrategyRequest, RunGaResponse, TimelineResponse, SystemState, TradeInspectorResponse, EventWrapper},
    errors::ApiError,
    services::evaluation_service::EvaluationService,
};
use serde_json;


fn strategy_from_id(strategy_id: &str) -> Result<chronosentiment_core::Strategy, ApiError> {
    let mut nums: Vec<u64> = Vec::new();
    for part in strategy_id.split('_').rev() {
        if let Ok(v) = part.parse::<u64>() {
            nums.push(v);
            if nums.len() == 4 {
                break;
            }
        }
    }

    if nums.len() < 4 {
        return Err(ApiError::ValidationError(format!(
            "Could not parse strategy parameters from strategy_id: {}",
            strategy_id
        )));
    }

    Ok(chronosentiment_core::Strategy {
        stop_loss: nums[0],
        take_profit: nums[1],
        base_edge: nums[2],
        queue_threshold: nums[3],
    })
}


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

    let strategy_config = if let Some(cfg) = request.strategy_config {
        cfg
    } else if let Some(id) = request.strategy_id.as_deref() {
        strategy_from_id(id)?
    } else {
        return Err(ApiError::ValidationError(
            "inspect_strategy requires either strategy_config or strategy_id".to_string(),
        ));
    };

    // Default to the first benchmark scenario if none provided
    let scenario = if let Some(s) = request.scenarios.into_iter().next() {
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

pub async fn latest_signals_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<chronosentiment_core::pipeline::SignalsSnapshot>, ApiError> {
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
