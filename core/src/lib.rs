use std::collections::HashMap;
use serde::{Serialize, Deserialize};

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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CreateOrder {
    pub order_id: String,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub ts: u64,
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
pub enum SimEvent {
    MarketEvent {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        subtype: MarketEventType,
        price: u64,
        quantity: u64,
        ts: u64,
    },
    OrderIntent {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        side: Side,
        price: u64,
        quantity: u64,
        ts: u64,
    },
    OrderEnteredQueue {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        ts: u64,
        price: u64,
        quantity_ahead: u64,
    },
    PartialFill {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        ts: u64,
        filled_qty: u64,
        price: u64,
    },
    QueueProgression {
        sequence_id: u64,
        parent_sequence_id: Option<u64>,
        order_id: String,
        ts: u64,
        new_quantity_ahead: u64,
    },
}

impl SimEvent {
    pub fn timestamp(&self) -> u64 {
        match self {
            SimEvent::MarketEvent { ts, .. } => *ts,
            SimEvent::OrderIntent { ts, .. } => *ts,
            SimEvent::OrderEnteredQueue { ts, .. } => *ts,
            SimEvent::PartialFill { ts, .. } => *ts,
            SimEvent::QueueProgression { ts, .. } => *ts,
        }
    }

    pub fn sequence_id(&self) -> u64 {
        match self {
            SimEvent::MarketEvent { sequence_id, .. } => *sequence_id,
            SimEvent::OrderIntent { sequence_id, .. } => *sequence_id,
            SimEvent::OrderEnteredQueue { sequence_id, .. } => *sequence_id,
            SimEvent::PartialFill { sequence_id, .. } => *sequence_id,
            SimEvent::QueueProgression { sequence_id, .. } => *sequence_id,
        }
    }

    pub fn parent_sequence_id(&self) -> Option<u64> {
        match self {
            SimEvent::MarketEvent { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::OrderIntent { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::OrderEnteredQueue { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::PartialFill { parent_sequence_id, .. } => *parent_sequence_id,
            SimEvent::QueueProgression { parent_sequence_id, .. } => *parent_sequence_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillEvent {
    pub ts: u64,
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

pub mod ese;
pub mod kernel;
pub mod replay;
pub mod inspector;

pub use ese::*;
pub use kernel::*;
pub use replay::*;
pub use inspector::*;
