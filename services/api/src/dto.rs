use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub use chronosentiment_core::{Strategy, pipeline::UnifiedGaResponse, pipeline::UnifiedStrategyEvaluation as StrategyEvaluationDto};

// ─── Canonical Schema Types ────────────────────────────────────────────────
// These types conform to the JSON schemas in schemas/canonical/.
// Every field maps 1:1 to a schema field. No derived or synthesized values.

/// Authority layer that emitted an event. Maps to `event.schema.json#/properties/source_layer`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceLayer {
    Kernel,
    Sequencer,
    LatencyLayer,
    Ese,
    PortfolioEngine,
    Governor,
    GaOptimizer,
}

/// Replay Engine certification state. Maps to `replay_response.schema.json#/properties/certification_state`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CertificationState {
    Certified,
    Degraded,
    Partial,
    Invalid,
}

/// Narrative block group. Maps to `decision_trace.schema.json#/properties/narrative_blocks/items/properties/group`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NarrativeGroup {
    Intent,
    Queue,
    Execution,
    Settlement,
    Governance,
}

/// Narrative block type. Maps to `decision_trace.schema.json#/properties/narrative_blocks/items/properties/block_type`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NarrativeBlockType {
    Primary,
    Derived,
    CausalLink,
    DivergenceMarker,
}

/// A single backend-certified narrative block. Replaces client-side `groupAndNarrateEvents()`.
/// Maps to `decision_trace.schema.json#/properties/narrative_blocks/items`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NarrativeBlock {
    pub block_id: Uuid,
    pub group: NarrativeGroup,
    pub sequence_id: u64,
    pub narrative: String,
    pub block_type: NarrativeBlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_block_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divergence_score: Option<f64>,
}

/// Canonical conformant response for `POST /inspect_strategy`.
/// Conforms to both `replay_response.schema.json` and `decision_trace.schema.json`.
#[derive(Debug, Serialize)]
pub struct CanonicalInspectResponse {
    // replay_response fields
    pub session_id: Uuid,
    pub strategy_id: String,
    pub requested_sequence_id: u64,
    pub certification_state: CertificationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certification_reason: Option<String>,
    pub reconstructed_at_ns: u64,
    pub event_window: CanonicalEventWindow,
    pub portfolio_state: CanonicalPortfolioState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causal_chain: Option<Vec<u64>>,
    pub replay_signature: String,

    // decision_trace fields (embedded — one trace per inspect session)
    pub trace_id: Uuid,
    pub narrative_blocks: Vec<NarrativeBlock>,
    pub causal_ancestry: Vec<u64>,
    pub trace_signature: String,

    // legacy fields preserved for prototype UI compatibility during transition
    pub decision_trace: Vec<EventWrapper>,
    pub execution_trace: Vec<EventWrapper>,
    pub metrics: StrategyEvaluationDto,
    pub event_sequence: Vec<EventWrapper>,
}

/// Canonical event window. Maps to `replay_response.schema.json#/properties/event_window`.
#[derive(Debug, Serialize)]
pub struct CanonicalEventWindow {
    pub first_sequence_id: u64,
    pub last_sequence_id: u64,
    pub event_count: usize,
    pub events: Vec<CanonicalEvent>,
}

/// Canonical portfolio state. Maps to `replay_response.schema.json#/properties/portfolio_state`.
#[derive(Debug, Serialize)]
pub struct CanonicalPortfolioState {
    pub positions: Vec<CanonicalPosition>,
    pub cash_balance: f64,
    pub total_equity: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_trades: u64,
}

/// A single position in the portfolio state.
#[derive(Debug, Serialize)]
pub struct CanonicalPosition {
    pub symbol: String,
    pub quantity: f64,
    pub avg_entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
}

/// Canonical event conforming to `event.schema.json`.
/// Extends `EventWrapper` with `source_layer` and `kernel_signature`.
#[derive(Debug, Serialize, Clone)]
pub struct CanonicalEvent {
    pub sequence_id: u64,
    pub timestamp_ns: u64,
    #[serde(rename = "event_type")]
    pub event_type: String,
    pub source_layer: SourceLayer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_sequence_id: Option<u64>,
    pub payload: serde_json::Value,
    pub kernel_signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_session_id: Option<Uuid>,
}

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
    // Added for canonical conformance — maps to event.schema.json#/properties/source_layer
    pub source_layer: SourceLayer,
    // Added for canonical conformance — maps to event.schema.json#/properties/kernel_signature
    pub kernel_signature: String,
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
    pub reject_reason: Option<String>,
    pub expected_holding_time: String,
    pub current_pnl: f64,
    pub peak_pnl: f64,
    pub exit_reason: Option<chronosentiment_core::pipeline::ExitReason>,
    pub is_open: bool,
    pub strategy_id: String,
    pub reason: String,
    pub rank_score: f64,
    pub rank_position: Option<u32>,
    pub allocated_capital: Option<PriceDto>,
    pub quantity: Option<u64>,
    pub status: String,
    pub porosity: String,
    pub porosity_trend: f64,
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
            reject_reason: s.reject_reason,
            expected_holding_time: s.expected_holding_time,
            current_pnl: s.current_pnl,
            peak_pnl: s.peak_pnl,
            exit_reason: s.exit_reason,
            is_open: s.is_open,
            strategy_id: s.strategy_id,
            reason: s.reason,
            rank_score: s.rank_score,
            rank_position: s.rank_position,
            allocated_capital: s.allocated_capital.map(PriceDto::from_scaled),
            quantity: s.quantity,
            status: format!("{:?}", s.status),
            porosity: format!("{:?}", s.porosity),
            porosity_trend: s.porosity_trend,
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
