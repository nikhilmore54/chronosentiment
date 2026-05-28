use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub struct MetricAggregation {
    pub metric: String,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone)]
pub struct AssetResult {
    pub asset: String,
    pub participation: f64,
    pub avg_pnl: f64,
    pub weak_executed_count: usize,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct StrategyEvaluationDto {
    pub strategy_id: String,
    pub avg: f64,
    pub std: f64,
    pub score: f64,
    pub classification: String,
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
pub struct SignalsSnapshot<T> {
    pub timestamp: u64,
    pub signals: Vec<T>,
    pub meta: SignalMeta,
    pub asset_name: String,
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
    pub fn as_str(self) -> &'static str {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThresholdSweepRow {
    pub confidence_floor: f64,
    pub score_floor: f64,
    pub participation: f64,
    pub trades: usize,
    pub total_scenarios: usize,
    pub global_avg_pnl: f64,
    pub traded_avg_pnl: f64,
    pub std_dev: f64,
}
