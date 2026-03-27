pub mod ga;
pub mod harness;
pub mod inspector;
pub mod kernel;
pub mod replay;
pub mod ese;
pub mod market_adapter;
pub mod data_source;
pub mod csv_source;
pub mod live_source;
pub mod folder_source;
pub mod pipeline;
pub mod synthetic;
pub mod binance_adapter;
pub mod strategy_ranking;
pub mod tick_replay;
pub mod replay_evaluator;
pub mod pnl_overlay;
pub mod edge_decay;

pub use ga::*;
pub use harness::{run_simulation_harness};
pub use inspector::*;
pub use kernel::*;
pub use replay::*;
pub use ese::{FIXED_LATENCY};
pub use market_adapter::*;
pub use data_source::*;
pub use csv_source::*;
pub use live_source::*;
pub use folder_source::*;
pub use pipeline::*;
pub use synthetic::*;
pub use binance_adapter::*;
pub use strategy_ranking::*;
pub use tick_replay::*;
pub use replay_evaluator::*;
pub use pnl_overlay::*;

pub const PRICE_SCALE: u64 = 100; // 1 Rupee = 100 Paise (Paise Format)

/// Rounds a price in Paise to the nearest valid tick.
/// Rule:
/// - Price < ₹20 (2000 Paise): 1 Paisa tick
/// - Price >= ₹20 (2000 Paise): 5 Paisa tick
pub fn round_to_tick(price_paise: u64) -> u64 {
    if price_paise < 2000 {
        price_paise
    } else {
        ((price_paise as f64 / 5.0).round() * 5.0) as u64
    }
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ExecutionMode {
    Ideal,
    Real,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub subtype: MarketEventType,
    pub price: u64,
    pub quantity: u64,
    pub side: Option<Side>,
    pub exchange_ts: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MarketEventType {
    NewOrder,
    Trade,
    Cancel,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CreateOrder {
    pub order_id: String,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u64,
    pub fill_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub pnl: i64,
    pub trades: u64,
    pub order_outcomes: HashMap<String, OrderOutcome>,
    pub events: Vec<SimEvent>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OrderOutcome {
    pub order_id: String,
    pub filled_quantity: u64,
    pub remaining_quantity: u64,
    pub arrival_time: u64,
    pub queue_ahead: u64,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct GAResult {
    pub best_config: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SimEvent {
    MarketEvent {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        subtype: MarketEventType,
        price: u64,
        quantity: u64,
        side: Option<Side>,
        timestamp: u64,
    },
    OrderIntent {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        side: Side,
        price: u64,
        quantity: u64,
        timestamp: u64,
    },
    OrderEnteredQueue {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        timestamp: u64,
        price: u64,
        queue_ahead: u64,
    },
    PartialFill {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        timestamp: u64,
        filled_qty: u64,
        price: u64,
    },
    QueueProgression {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        timestamp: u64,
        queue_ahead: u64,
    },
    OrderFilled {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        timestamp: u64,
    },
}

impl SimEvent {
    pub fn timestamp(&self) -> u64 {
        match self {
            SimEvent::MarketEvent { timestamp, .. } => *timestamp,
            SimEvent::OrderIntent { timestamp, .. } => *timestamp,
            SimEvent::OrderEnteredQueue { timestamp, .. } => *timestamp,
            SimEvent::PartialFill { timestamp, .. } => *timestamp,
            SimEvent::QueueProgression { timestamp, .. } => *timestamp,
            SimEvent::OrderFilled { timestamp, .. } => *timestamp,
        }
    }

    pub fn sequence_id(&self) -> u64 {
        match self {
            SimEvent::MarketEvent { sequence_id, .. } => *sequence_id,
            SimEvent::OrderIntent { sequence_id, .. } => *sequence_id,
            SimEvent::OrderEnteredQueue { sequence_id, .. } => *sequence_id,
            SimEvent::PartialFill { sequence_id, .. } => *sequence_id,
            SimEvent::QueueProgression { sequence_id, .. } => *sequence_id,
            SimEvent::OrderFilled { sequence_id, .. } => *sequence_id,
        }
    }

    pub fn parent_sequence_id(&self) -> Option<u64> {
        match self {
            SimEvent::MarketEvent { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::OrderIntent { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::OrderEnteredQueue { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::PartialFill { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::QueueProgression { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::OrderFilled { parent_sequence_id, .. } => *parent_sequence_id,
        }
    }

    pub fn order_id(&self) -> Option<&String> {
        match self {
            SimEvent::OrderIntent { order_id, .. } => Some(order_id),
            SimEvent::OrderEnteredQueue { order_id, .. } => Some(order_id),
            SimEvent::PartialFill { order_id, .. } => Some(order_id),
            SimEvent::QueueProgression { order_id, .. } => Some(order_id),
            SimEvent::OrderFilled { order_id, .. } => Some(order_id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillEvent {
    pub timestamp: u64,
    pub qty: u64,
    pub price: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLayer {
    pub order_id: String,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLayer {
    pub arrival_time: u64,
    pub latency_applied: u64,
    pub queue_ahead_initial: u64,
    pub queue_progression: Vec<u64>,
    pub fills: Vec<FillEvent>,
    pub causal_chain: Vec<SimEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeLayer {
    pub filled_quantity: u64,
    pub remaining_quantity: u64,
    pub average_price: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInspection {
    pub decision: DecisionLayer,
    pub execution: ExecutionLayer,
    pub outcome: OutcomeLayer,
}

