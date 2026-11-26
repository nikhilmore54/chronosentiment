use crate::exit::ExitReason;
use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIDENCE_FLOOR: f64 = 0.30;
pub const DEFAULT_SCORE_FLOOR: f64 = 0.40;

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
