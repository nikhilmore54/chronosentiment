pub mod binance_adapter;
pub mod csv_source;
pub mod data_source;
pub mod edge_decay;
pub mod ensemble;
pub mod ese;
pub mod exit;
pub mod folder_source;
pub mod ga;
pub mod harness;
pub mod inspector;
pub mod kernel;
pub mod market_adapter;
pub mod paper;
pub mod pipeline;
pub mod pnl_overlay;
pub mod reco;
pub mod replay;
pub mod replay_evaluator;
pub mod selection_cap;
pub mod strategy_ranking;
pub mod synthetic;
pub mod tick_replay;

pub use binance_adapter::load_binance_events_from_jsonl;
pub use csv_source::*;
pub use data_source::CandleSource;
pub use edge_decay::*;
pub use ensemble::*;
pub use ese::*;
pub use exit::{ExitDecision, ExitEvaluator, ExitMetadata, ExitReason};
pub use folder_source::*;
pub use ga::*;
pub use harness::{deterministic_demo_fixture, run_simulation, run_simulation_harness};
pub use inspector::*;
pub use kernel::*;
pub use market_adapter::*;
pub use paper::*;
pub use pipeline::*;
pub use pnl_overlay::*;
pub use reco::*;
pub use replay::*;
pub use replay_evaluator::*;
pub use selection_cap::*;
pub use strategy_ranking::*;
pub use synthetic::*;
pub use tick_replay::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedMarketEvent {
    pub asset: String,
    pub exchange_ts: u64,
    pub price: f64,
    pub volume: f64,
    pub side: Option<Side>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub bids: Option<Vec<(f64, f64)>>,
    pub asks: Option<Vec<(f64, f64)>>,
}

impl NormalizedMarketEvent {
    pub fn to_legacy_market_event(&self) -> Option<MarketEvent> {
        Some(MarketEvent {
            subtype: MarketEventType::Trade,
            price: from_real(self.price),
            quantity: self.volume as u64,
            side: self.side,
            exchange_ts: self.exchange_ts,
        })
    }
}

/// System-wide institutional price scaling factor.
/// 1.0000 = 10,000 units (1/100th of a paisa precision).
/// This is the SINGLE SOURCE OF TRUTH for internal math.
pub const PRICE_SCALE: u64 = 10000;

/// Converts internal scaled integer to real-unit float (Rupees).
pub fn to_real(price: u64) -> f64 {
    price as f64 / PRICE_SCALE as f64
}

/// Converts real-unit float (Rupees) to internal scaled integer.
/// Performs rounding to the nearest internal unit.
pub fn from_real(price: f64) -> u64 {
    (price * PRICE_SCALE as f64).round() as u64
}

#[cfg(test)]
mod precision_tests {
    use super::*;

    #[test]
    fn test_rupee_scaling_contract() {
        let value_rs = 432.85;
        let scaled = from_real(value_rs);
        // 432.85 * 10000 = 4,328,500
        assert_eq!(scaled, 4328500, "Scaling must be exactly 10,000x");
        // Float precision safe comparison
        assert!(
            (to_real(scaled) - value_rs).abs() < 1e-6,
            "Round-trip must be lossless for 2-decimal prices"
        );
    }
}

/// Rounds a scaled price to the nearest valid institutional tick.
/// Rule:
/// - Price < 20 units: 1 Paisa tick (0.01 * PRICE_SCALE)
/// - Price >= 20 units: 5 Paisa tick (0.05 * PRICE_SCALE)
pub fn round_to_tick(price_scaled: u64) -> u64 {
    let tick_01 = (0.01 * PRICE_SCALE as f64) as u64; // 1 Paisa
    let tick_05 = (0.05 * PRICE_SCALE as f64) as u64; // 5 Paise
    let threshold = 20 * PRICE_SCALE;

    if price_scaled < threshold {
        ((price_scaled as f64 / tick_01 as f64).round() * tick_01 as f64) as u64
    } else {
        ((price_scaled as f64 / tick_05 as f64).round() * tick_05 as f64) as u64
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GaExitReason {
    TakeProfit,
    StopLoss,
    TimeStop,
    NoFill, // New: Market interaction failed (queue pressure, etc)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Hash, Deserialize)]
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
            SimEvent::MarketEvent {
                parent_sequence_id, ..
            } => *parent_sequence_id,
            SimEvent::OrderIntent {
                parent_sequence_id, ..
            } => *parent_sequence_id,
            SimEvent::OrderEnteredQueue {
                parent_sequence_id, ..
            } => *parent_sequence_id,
            SimEvent::PartialFill {
                parent_sequence_id, ..
            } => *parent_sequence_id,
            SimEvent::QueueProgression {
                parent_sequence_id, ..
            } => *parent_sequence_id,
            SimEvent::OrderFilled {
                parent_sequence_id, ..
            } => *parent_sequence_id,
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
pub fn compute_trend_deltas(market: &[MarketEvent], idx: usize, lookback: usize) -> i64 {
    if idx < lookback || idx >= market.len() {
        return 0;
    }
    let mut trend = 0i64;
    for j in (idx - lookback)..idx {
        trend += market[j + 1].price as i64 - market[j].price as i64;
    }
    trend
}
