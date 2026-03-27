use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use chronosentiment_core::{Strategy, pipeline::UnifiedGaResponse, pipeline::UnifiedStrategyEvaluation as StrategyEvaluationDto};

// --- Request DTOs ---

#[derive(Debug, Deserialize)]
pub struct EvaluateStrategyRequest {
    pub strategy_config: Strategy,
    #[serde(default)]
    pub scenarios: Vec<String>,
    pub seed: u64,
}

#[derive(Debug, Deserialize)]
pub struct CompareStrategiesRequest {
    pub strategies: Vec<StrategyConfigWrapper>,
    #[serde(default)]
    pub scenarios: Vec<String>,
    pub seed: u64,
}

#[derive(Debug, Deserialize)]
pub struct StrategyConfigWrapper {
    pub strategy_config: Strategy,
}


#[derive(Debug, Deserialize)]
pub struct InspectStrategyRequest {
    pub strategy_id: Option<String>,
    pub strategy_config: Option<Strategy>,
    #[serde(default)]
    pub scenarios: Vec<String>,
    pub seed: u64,
}

// --- Response DTOs ---

#[derive(Debug, Serialize, PartialEq)]
pub struct EvaluateStrategyResponse {
    pub strategy_evaluation: StrategyEvaluationDto,
}


#[derive(Debug, Serialize)]
pub struct CompareStrategiesResponse {
    pub ranking: Vec<StrategyEvaluationDto>,
    pub comparison_summary: ComparisonSummary,
}


#[derive(Debug, Serialize)]
pub struct ComparisonSummary {
    pub best_strategy: String,
    pub reason: String, // This will be a derived summary from the comparison logic
}

#[derive(Debug, Serialize)]
pub struct InspectStrategyResponse {
    pub strategy_id: String,
    pub decision_trace: Vec<EventWrapper>, 
    pub execution_trace: Vec<EventWrapper>, 
    pub metrics: StrategyEvaluationDto,
    pub event_sequence: Vec<EventWrapper>,
}

#[derive(Debug, Serialize)]
pub struct RunGaResponse {
    pub results: Vec<StrategyEvaluationDto>,
    pub generation_history: Vec<StrategyEvaluationDto>,
    pub best_per_regime: HashMap<String, StrategyEvaluationDto>,
    pub global_best: StrategyEvaluationDto,
    pub global_best_generation: usize,
    pub generation_found: usize,
    pub final_generation_best: StrategyEvaluationDto,
    pub final_gen_best: StrategyEvaluationDto,
}

impl From<UnifiedGaResponse> for RunGaResponse {
    fn from(res: UnifiedGaResponse) -> Self {
        Self {
            results: vec![res.global_best.clone(), res.final_generation_best.clone()],
            generation_history: res.generation_history,
            best_per_regime: res.best_per_regime,
            global_best: res.global_best.clone(),
            global_best_generation: res.global_best_generation,
            generation_found: res.global_best_generation,
            final_generation_best: res.final_generation_best.clone(),
            final_gen_best: res.final_generation_best,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct EventWrapper {
    pub sequence_id: u64,
    pub timestamp: u64,
    #[serde(rename = "type")]
    pub event_type: String,
    pub parent_sequence_id: Option<u64>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct TimelineResponse {
    pub events: Vec<EventWrapper>,
}

/// BUY/SELL only — excludes `HOLD` from the latest signal run (see `/signals/latest` for full snapshot).
#[derive(Debug, Serialize)]
pub struct TradeSuggestionsResponse {
    pub asset: String,
    pub timestamp: u64,
    pub suggestions: Vec<chronosentiment_core::strategy_ranking::RankedStrategy>,
    pub count: usize,
    pub debug: chronosentiment_core::strategy_ranking::SuggestionDebug,
}

#[derive(Debug, Serialize, Clone)]
pub struct TopStrategySnapshot {
    pub strategy_id: String,
    pub action: String,
    pub live_score: f64,
    pub expected_edge: f64,
    pub execution_score: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReplaySuggestionPoint {
    pub ts: u64,
    pub decision_ts: u64,
    pub execution_ts: u64,
    pub suggestion_count: usize,
    pub prev_strategy: Option<String>,
    pub flip_occurred: bool,
    pub top_strategy: Option<TopStrategySnapshot>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReplaySuggestionsResponse {
    pub asset: String,
    pub metrics: chronosentiment_core::replay_evaluator::ReplayMetrics,
    pub timeline: Vec<ReplaySuggestionPoint>,
    pub pnl: Option<chronosentiment_core::pnl_overlay::PnLMetrics>,
}

#[derive(Debug, Serialize)]
pub struct TradeInspectorResponse {
    pub order_id: String,
    pub decision: chronosentiment_core::DecisionLayer,
    pub execution: Vec<serde_json::Value>, // Simplified for now as steps
    pub outcome: chronosentiment_core::OutcomeLayer,
    pub causal_chain: Option<Vec<EventWrapper>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderState {
    pub order_id: String,
    pub status: String, // 'NEW' | 'ACTIVE' | 'PARTIAL' | 'FILLED' | 'REJECTED'
    pub quantity_total: u64,
    pub quantity_filled: u64,
    pub quantity_remaining: u64,
    pub queue_ahead: u64,
    pub price: f64, // Currency units
    pub side: chronosentiment_core::Side,
}

#[derive(Debug, Serialize, Clone)]
pub struct PortfolioState {
    pub pnl: f64,
    pub position: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct SystemState {
    pub orders: HashMap<String, OrderState>,
    pub portfolio: PortfolioState,
    pub last_sequence_id: u64,
}
