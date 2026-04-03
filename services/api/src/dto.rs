use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use chronosentiment_core::{Strategy, pipeline::UnifiedGaResponse, pipeline::UnifiedStrategyEvaluation as StrategyEvaluationDto};

/// Type-safe boundary unit for real-world Rupees (f64).
/// This ensures internal scaled integers (paise/units) never leak to the API.
#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
pub struct PriceDto(pub f64);

impl From<f64> for PriceDto {
    fn from(val: f64) -> Self {
        // Assume val is already real units if it's f64 in DTO
        PriceDto(val)
    }
}

impl PriceDto {
    pub fn from_scaled(scaled: f64) -> Self {
        PriceDto(scaled / chronosentiment_core::PRICE_SCALE as f64)
    }
}

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

/// Mirror of `chronosentiment_core::pipeline::TradeSignal` using `PriceDto` for monetary fields.
#[derive(Debug, Serialize)]
pub struct TradeSignalDto {
    pub asset: String,
    pub regime: String,
    pub confidence: f64,
    pub action: chronosentiment_core::pipeline::SignalAction,
    pub entry_type: chronosentiment_core::pipeline::EntryType,
    pub entry_zone: Option<(PriceDto, PriceDto)>,
    pub stop_loss: Option<PriceDto>,
    pub target: Option<PriceDto>,
    pub expected_edge: f64,
    pub scenario_pnl: f64,
    pub risk_reward: f64,
    pub position_size: f64,
    pub conviction: f64,
    pub composite_score: f64,
    pub consensus_strength: f64,
    pub conflict_score: f64,
    pub execution_confidence: f64,
    pub short_term_capture_eff: f64,
    pub long_term_capture_eff: f64,
    pub reject_reason: Option<String>,
    pub expected_holding_time: String,
    pub current_pnl: f64,
    pub peak_pnl: f64,
    pub exit_reason: Option<chronosentiment_core::pipeline::ExitReason>,
    pub is_open: bool,
    pub strategy_id: String,
    pub reason: String,
    pub regime_confidence: f64,
    pub rank_score: f64,
    pub rank_position: Option<u32>,
    pub allocated_capital: Option<PriceDto>,
    pub quantity: Option<u64>,
}

impl From<chronosentiment_core::pipeline::TradeSignal> for TradeSignalDto {
    fn from(s: chronosentiment_core::pipeline::TradeSignal) -> Self {
        Self {
            asset: s.asset,
            regime: s.regime,
            confidence: s.confidence,
            action: s.action,
            entry_type: s.entry_type,
            entry_zone: s.entry_zone.map(|(lo, hi)| (PriceDto::from_scaled(lo), PriceDto::from_scaled(hi))),
            stop_loss: s.stop_loss.map(PriceDto::from_scaled),
            target: s.target.map(PriceDto::from_scaled),
            expected_edge: s.expected_edge,
            scenario_pnl: s.scenario_pnl,
            risk_reward: s.risk_reward,
            position_size: s.position_size,
            conviction: s.conviction,
            composite_score: s.composite_score,
            consensus_strength: s.consensus_strength,
            conflict_score: s.conflict_score,
            execution_confidence: s.execution_confidence,
            short_term_capture_eff: s.short_term_capture_eff,
            long_term_capture_eff: s.long_term_capture_eff,
            reject_reason: s.reject_reason,
            expected_holding_time: s.expected_holding_time,
            current_pnl: s.current_pnl,
            peak_pnl: s.peak_pnl,
            exit_reason: s.exit_reason,
            is_open: s.is_open,
            strategy_id: s.strategy_id,
            reason: s.reason,
            regime_confidence: s.regime_confidence,
            rank_score: s.rank_score,
            rank_position: s.rank_position,
            allocated_capital: s.allocated_capital.map(PriceDto::from_scaled),
            quantity: s.quantity,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SignalsSnapshotDto {
    pub timestamp: u64,
    pub signals: Vec<TradeSignalDto>,
}

impl From<chronosentiment_core::pipeline::SignalsSnapshot> for SignalsSnapshotDto {
    fn from(s: chronosentiment_core::pipeline::SignalsSnapshot) -> Self {
        Self {
            timestamp: s.timestamp,
            signals: s.signals.into_iter().map(TradeSignalDto::from).collect(),
        }
    }
}
#[derive(Debug, Serialize, Clone)]
pub struct TradeInspectorDecision {
    pub order_id: String,
    pub side: chronosentiment_core::Side,
    pub price: f64,
    pub quantity: u64,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct TradeInspectorOutcome {
    pub filled_qty: u64,
    pub remaining_qty: u64,
    pub avg_price: f64,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct TradeInspectorResponse {
    pub order_id: String,
    pub decision: TradeInspectorDecision,
    pub execution: Vec<serde_json::Value>,
    pub outcome: TradeInspectorOutcome,
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
