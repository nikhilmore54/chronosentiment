use axum::{extract::{State, Path}, Json};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    dto::{
        CompareStrategiesRequest, CompareStrategiesResponse, EvaluateStrategyRequest,
        EvaluateStrategyResponse, InspectStrategyRequest, RunGaResponse, TimelineResponse,
        SystemState, TradeInspectorResponse, EventWrapper,
        // Canonical schema types
        CanonicalInspectResponse, CanonicalEventWindow, CanonicalPortfolioState,
        CanonicalPosition, CanonicalEvent, NarrativeBlock, NarrativeBlockType, NarrativeGroup,
        CertificationState, SourceLayer,
    },
    errors::ApiError,
    services::evaluation_service::EvaluationService,
    signatures::{
        compute_event_signature, compute_replay_signature, compute_trace_signature,
        sign_event_batch,
    },
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
) -> Result<Json<CanonicalInspectResponse>, ApiError> {
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

    let legacy = service.inspect_strategy(
        strategy_config,
        scenario,
        request.seed,
    )?;
    println!("Inspection completed for strategy_id: {}", legacy.strategy_id);

    // ── Build canonical response ──────────────────────────────────────────
    let session_id = Uuid::new_v4();
    let trace_id = Uuid::new_v4();
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    // Convert legacy EventWrappers to CanonicalEvents with BLAKE3 signatures
    let mut canonical_events: Vec<CanonicalEvent> = legacy.execution_trace.iter().map(|ew| {
        let sig = compute_event_signature(
            ew.sequence_id,
            ew.timestamp,
            &ew.event_type,
            &SourceLayer::Sequencer,
            &ew.payload,
        );
        CanonicalEvent {
            sequence_id: ew.sequence_id,
            timestamp_ns: ew.timestamp,
            event_type: ew.event_type.clone(),
            source_layer: SourceLayer::Sequencer,
            strategy_id: Some(legacy.strategy_id.clone()),
            parent_sequence_id: ew.parent_sequence_id,
            payload: ew.payload.clone(),
            kernel_signature: sig,
            replay_session_id: Some(session_id),
        }
    }).collect();

    // Sign the full batch (authoritative signing pass)
    sign_event_batch(&mut canonical_events);

    // Determine sequence bounds
    let first_seq = canonical_events.iter().map(|e| e.sequence_id).min().unwrap_or(0);
    let last_seq = canonical_events.iter().map(|e| e.sequence_id).max().unwrap_or(0);
    let event_count = canonical_events.len();

    // Build causal ancestry chain from parent_sequence_id links (backend-certified)
    let causal_ancestry: Vec<u64> = {
        let mut chain = Vec::new();
        let mut current = last_seq;
        let event_map: std::collections::HashMap<u64, Option<u64>> = canonical_events
            .iter()
            .map(|e| (e.sequence_id, e.parent_sequence_id))
            .collect();
        let mut visited = std::collections::HashSet::new();
        while visited.insert(current) {
            chain.push(current);
            match event_map.get(&current).and_then(|p| *p) {
                Some(parent) => current = parent,
                None => break,
            }
        }
        chain
    };

    // Build backend-certified narrative blocks (replaces client-side groupAndNarrateEvents)
    let narrative_blocks: Vec<NarrativeBlock> = canonical_events.iter().map(|event| {
        let (group, block_type, narrative) = match event.event_type.as_str() {
            "OrderIntent" => (
                NarrativeGroup::Intent,
                NarrativeBlockType::Primary,
                format!(
                    "Strategy decision: order placed. Seq {}.",
                    event.sequence_id
                ),
            ),
            "OrderEnteredQueue" => (
                NarrativeGroup::Queue,
                NarrativeBlockType::Derived,
                format!(
                    "Order entered execution queue at seq {}.",
                    event.sequence_id
                ),
            ),
            "QueueProgression" => (
                NarrativeGroup::Queue,
                NarrativeBlockType::Derived,
                format!(
                    "Queue position advancing at seq {}.",
                    event.sequence_id
                ),
            ),
            "PartialFill" => (
                NarrativeGroup::Execution,
                NarrativeBlockType::Primary,
                format!(
                    "Partial execution recorded at seq {}.",
                    event.sequence_id
                ),
            ),
            "OrderFilled" => (
                NarrativeGroup::Execution,
                NarrativeBlockType::Primary,
                format!(
                    "Order fully executed at seq {}.",
                    event.sequence_id
                ),
            ),
            _ => (
                NarrativeGroup::Governance,
                NarrativeBlockType::CausalLink,
                format!(
                    "Event {} at seq {}.",
                    event.event_type, event.sequence_id
                ),
            ),
        };

        // Find parent block id by matching parent_sequence_id to a block's sequence_id
        let parent_block_id: Option<Uuid> = None; // resolved in a second pass if needed

        NarrativeBlock {
            block_id: Uuid::new_v4(),
            group,
            sequence_id: event.sequence_id,
            narrative,
            block_type,
            parent_block_id,
            divergence_score: None,
        }
    }).collect();

    // Determine certification state
    let (certification_state, certification_reason) = if event_count == 0 {
        (CertificationState::Invalid, Some("No events in execution trace".to_string()))
    } else if first_seq == 0 && last_seq == 0 {
        (CertificationState::Degraded, Some("Sequence IDs could not be determined".to_string()))
    } else {
        (CertificationState::Certified, None)
    };

    // Compute replay signature (BLAKE3)
    let cert_state_str = format!("{:?}", certification_state).to_uppercase();
    let replay_signature = compute_replay_signature(
        &session_id,
        &legacy.strategy_id,
        last_seq,
        &cert_state_str,
        event_count,
    );

    // Compute trace signature (BLAKE3)
    let trace_signature = compute_trace_signature(
        &trace_id,
        last_seq,
        &legacy.strategy_id,
        "EVALUATED",
        "CERTIFIED",
    );

    // Build canonical portfolio state from legacy metrics
    let portfolio_state = CanonicalPortfolioState {
        positions: vec![],  // populated from order outcomes when available
        cash_balance: 0.0,
        total_equity: legacy.metrics.avg,
        unrealized_pnl: 0.0,
        realized_pnl: legacy.metrics.avg,
        total_trades: legacy.metrics.total_trades as u64,
    };

    let canonical = CanonicalInspectResponse {
        session_id,
        strategy_id: legacy.strategy_id.clone(),
        requested_sequence_id: last_seq,
        certification_state,
        certification_reason,
        reconstructed_at_ns: now_ns,
        event_window: CanonicalEventWindow {
            first_sequence_id: first_seq,
            last_sequence_id: last_seq,
            event_count,
            events: canonical_events,
        },
        portfolio_state,
        causal_chain: Some(causal_ancestry.clone()),
        replay_signature,
        trace_id,
        narrative_blocks,
        causal_ancestry,
        trace_signature,
        // Legacy fields preserved for prototype UI compatibility
        decision_trace: legacy.decision_trace,
        execution_trace: legacy.execution_trace,
        metrics: legacy.metrics,
        event_sequence: legacy.event_sequence,
    };

    Ok(Json(canonical))
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
/// NOTE: `load_strategy_store` is not yet implemented in core; returns null stub.
pub async fn get_strategy_store_handler(
    State(_service): State<EvaluationService>,
) -> Json<serde_json::Value> {
    println!("Request received: ga/strategy-store");
    Json(serde_json::json!({
        "path": "N/A",
        "store": null,
        "note": "strategy store not yet implemented"
    }))
}

pub async fn latest_signals_handler(
    State(service): State<EvaluationService>,
) -> Result<Json<crate::dto::SignalsSnapshotDto>, ApiError> {
    println!("Request received: signals/latest");
    let snapshot = service.get_latest_signals()?;
    Ok(Json(snapshot.into()))
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
