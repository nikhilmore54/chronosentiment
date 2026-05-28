use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::exit::ExitReason;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignalAction {
    BUY,
    SELL,
    HOLD,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationStatus {
    NEW,      // Emitted by engine
    PENDING,  // Waiting for fill
    ACTIVE,   // Position open
    CLOSED,   // Completed
    REJECTED, // Filtered out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlphaPorosity {
    Dead,         // capture < 0
    Fragile,      // 0 <= capture < 0.25
    Transitional, // 0.25 <= capture < 0.6
    Live,         // capture >= 0.6
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryType {
    MARKET,
    PULLBACK,
    BREAKOUT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeSignal {
    pub asset: String,
    pub regime: String,
    pub confidence: f64,
    pub action: SignalAction,
    pub entry_type: EntryType,
    pub entry_zone: Option<(f64, f64)>,
    pub stop_loss: Option<f64>,
    pub target: Option<f64>,
    pub expected_edge: f64,
    pub scenario_pnl: f64,
    pub risk_reward: f64,
    pub position_size: f64,
    pub conviction: f64,
    pub composite_score: f64,
    pub reject_reason: Option<String>,
    pub expected_holding_time: String,
    pub strategy_id: String,
    pub reason: String,
    pub status: RecommendationStatus,
    pub porosity: AlphaPorosity,
    pub porosity_trend: f64, // Rate of change in porosity
    pub is_open: bool,
    pub current_pnl: f64,
    pub peak_pnl: f64,
    pub exit_reason: Option<ExitReason>,
    pub rank_score: f64,
    pub rank_position: Option<u32>,
    pub allocated_capital: Option<f64>,
    pub quantity: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
enum RejectReason {
    LowConfidence,
    LowScore,
    NegativeEdge,
    SidewaysRegime,
}

impl RejectReason {
    fn as_str(self) -> &'static str {
        match self {
            RejectReason::LowConfidence => "REJECT_LOW_CONF",
            RejectReason::LowScore => "REJECT_LOW_SCORE",
            RejectReason::NegativeEdge => "REJECT_NEG_EDGE",
            RejectReason::SidewaysRegime => "REJECT_SIDEWAYS",
        }
    }
}

fn edge_reason_from_gate_reject(reject_reason: Option<RejectReason>) -> EdgeLossReason {
    match reject_reason {
        None => EdgeLossReason::Accepted,
        Some(RejectReason::LowConfidence) => EdgeLossReason::LowConfidence,
        Some(RejectReason::LowScore) => EdgeLossReason::LowScore,
        Some(RejectReason::NegativeEdge) => EdgeLossReason::RiskFiltered,
        Some(RejectReason::SidewaysRegime) => EdgeLossReason::SidewaysMarket,
    }
}

#[derive(Debug, Clone, Copy)]
struct GateDecision {
    trade_allowed: bool,
    position_size: f64,
    composite_score: f64,
    reject_reason: Option<RejectReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMeta {
    pub total_assets: usize,
    pub total_scenarios: usize,
    pub trades: usize,
    pub holds: usize,
    pub participation: f64,
    pub edge_loss_breakdown: EdgeLossBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalsSnapshot {
    pub timestamp: u64,
    pub signals: Vec<TradeSignal>,
    pub meta: SignalMeta,
    pub asset_name: String, // Add asset name to snapshot for context
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeLossReason {
    NoAggregateEvaluation,
    WeakEvalSurrogate,
    WeakLowVol,
    LowConfidence,
    LowScore,
    SidewaysMarket,
    RiskFiltered,
    QueueTooDeep,
    LowLiquidity,
    HighLatencyImpact,
    Accepted,
}

impl EdgeLossReason {
    fn as_str(self) -> &'static str {
        match self {
            EdgeLossReason::NoAggregateEvaluation => "NO_AGG_EVAL",
            EdgeLossReason::WeakEvalSurrogate => "WEAK_EVAL_SURROGATE",
            EdgeLossReason::WeakLowVol => "WEAK_LOW_VOL",
            EdgeLossReason::LowConfidence => "LOW_CONFIDENCE",
            EdgeLossReason::LowScore => "LOW_SCORE",
            EdgeLossReason::SidewaysMarket => "SIDEWAYS_MARKET",
            EdgeLossReason::RiskFiltered => "RISK_FILTERED",
            EdgeLossReason::QueueTooDeep => "QUEUE_TOO_DEEP",
            EdgeLossReason::LowLiquidity => "LOW_LIQUIDITY",
            EdgeLossReason::HighLatencyImpact => "HIGH_LATENCY_IMPACT",
            EdgeLossReason::Accepted => "ACCEPTED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EdgeTransfer {
    pub eval_edge: Option<f64>,
    pub weak_eval_edge: Option<f64>,
    pub has_strong_eval: bool,
    pub signal_edge: f64,
    pub delta: f64,
    pub confidence: f64,
    pub reason: EdgeLossReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeLossBreakdown {
    pub total_scenarios: usize,
    pub total_eval_edge: f64,
    pub total_signal_edge: f64,
    pub edge_retention_ratio: f64,
    pub true_edge_retention: f64,
    pub top_loss_reason: Option<String>,
    pub loss_distribution: Vec<ReasonLossShare>,
    pub loss_by_reason: HashMap<String, f64>,
    pub count_by_reason: HashMap<String, usize>,
    pub executed_strong_edge: f64,
    pub executed_weak_edge: f64,
    pub weak_rejected_low_conf: usize,
    pub weak_rejected_low_vol: usize,
    pub weak_executed_count: usize,
    pub transfer_traces: Vec<EdgeTransfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonLossShare {
    pub reason: String,
    pub pct: f64,
}