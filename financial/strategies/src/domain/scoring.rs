use serde::{Serialize, Deserialize};

use super::signatures::SignalType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionDirective {
    ExecuteImmediate,
    QueueForEntry,
    ProbeLiquidity,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationStatus {
    Pending,
    Executed,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConvictionOutcome {
    pub conviction_score: f64,
    pub edge_weight: f64,
    pub norm_momentum: f64,
    pub norm_volume: f64,
    pub is_valid: bool,
    pub selection_threshold: f64,
    pub raw_q_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeRecommendation {
    pub signal_type: SignalType,
    pub status: RecommendationStatus,
    pub price: f64,
    pub confidence: f64,
    pub size: f64,
    pub reason: String,
    pub tp_target: Option<f64>,
    pub sl_target: Option<f64>,
}
