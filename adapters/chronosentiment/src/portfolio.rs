use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a point-in-time snapshot of the user's actual or simulated portfolio.
/// This allows the Replay Engine to answer: "Given my portfolio on this date, what should I have done?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub snapshot_id: String,
    pub timestamp: u64,
    pub cash: f64,
    pub available_margin: f64,
    pub positions: Vec<Position>,
    pub risk_exposure: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub instrument_id: Uuid,
    pub quantity: f64,
    pub average_cost: f64,
    pub current_value: Option<f64>,
}
