use chronosentiment_core::*;
use crate::ApiError;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    NEW,
    ACTIVE,
    PARTIAL,
    FILLED,
    REJECTED,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderState {
    pub order_id: String,
    pub status: OrderStatus, // NEW, ACTIVE, PARTIAL, FILLED
    pub quantity_total: i32,
    pub quantity_filled: i32,
    pub quantity_remaining: i32,
    pub queue_ahead: i32,
    pub price: i32,
    pub side: Side,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortfolioState {
    pub pnl: i64,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemState {
    pub orders: HashMap<String, OrderState>,
    pub portfolio: PortfolioState,
    pub last_sequence_id: u64,
}

/// PURE FUNCTION: Deterministically apply an event to the state
pub fn apply_event(state: &mut SystemState, event: &SimEvent) {
    state.last_sequence_id = event.sequence_id();

    match event {
        SimEvent::OrderIntent { order_id, side, price, quantity, .. } => {
            state.orders.insert(order_id.clone(), OrderState {
                order_id: order_id.clone(),
                status: OrderStatus::NEW,
                quantity_total: *quantity as i32,
                quantity_filled: 0,
                quantity_remaining: *quantity as i32,
                queue_ahead: 0,
                price: *price as i32,
                side: *side,
            });
        }
        SimEvent::OrderEnteredQueue { order_id, queue_ahead, .. } => {
            if let Some(order) = state.orders.get_mut(order_id) {
                order.status = OrderStatus::ACTIVE;
                order.queue_ahead = *queue_ahead as i32;
            }
        }
        SimEvent::QueueProgression { order_id, queue_ahead, .. } => {
            if let Some(order) = state.orders.get_mut(order_id) {
                order.queue_ahead = *queue_ahead as i32;
            }
        }
        SimEvent::PartialFill { order_id, filled_qty, price, .. } => {
            if let Some(order) = state.orders.get_mut(order_id) {
                order.quantity_filled += *filled_qty as i32;
                order.quantity_remaining -= *filled_qty as i32;
                order.status = if order.quantity_remaining == 0 {
                    OrderStatus::FILLED
                } else {
                    OrderStatus::PARTIAL
                };

                // Update Portfolio
                let multiplier = if order.side == Side::Buy { 1 } else { -1 };
                state.portfolio.position += (*filled_qty as i64) * multiplier;
                state.portfolio.pnl += (*filled_qty * price) as i64; // Simple gross value for MVP
            }
        }
        SimEvent::OrderFilled { order_id, .. } => {
            if let Some(order) = state.orders.get_mut(order_id) {
                order.status = OrderStatus::FILLED;
            }
        }
        _ => {} // MarketEvents etc do not directly change Replay SystemState in this MVP
    }
}

pub fn handle_replay(target_seq: u64, sim: &SimulationResult) -> Result<SystemState, ApiError> {
    let mut state = SystemState::default();
    
    // Deterministic reduction: state = reduce(events[0..=target_seq])
    for event in sim.events.iter() {
        if event.sequence_id() > target_seq {
            break;
        }
        apply_event(&mut state, event);
    }
    
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_consistency() {
        // 1. Run full simulation with simple harness
        let config = chronosentiment_core::GaConfig {
            population_size: 1,
            generations: 1,
            mutation_rate: 0.1,
            seed: 42,
            order_id_prefix: "test".to_string(),
            order_price: 100,
            order_quantity_for_strategy: 1000,
            order_timestamp: 10,
            lambda: 0.5,
            initial_queue_threshold: 200,
            ..chronosentiment_core::GaConfig::default()
        };
        
        let events = vec![
            chronosentiment_core::MarketEvent { subtype: chronosentiment_core::MarketEventType::NewOrder, price: 100, quantity: 2000, side: Some(chronosentiment_core::Side::Sell), exchange_ts: 10 },
            chronosentiment_core::MarketEvent { subtype: chronosentiment_core::MarketEventType::Trade, price: 100, quantity: 500, side: None, exchange_ts: 15 },
        ];
        
        let orders = vec![chronosentiment_core::CreateOrder {
            order_id: "test_order".to_string(),
            side: chronosentiment_core::Side::Buy,
            price: 100,
            quantity: 1000,
            timestamp: 10,
            fill_probability: 1.0,
        }];

        let (_, sim, _) = chronosentiment_core::harness::run_simulation_harness(
            ExecutionMode::Real,
            events,
            orders,
        );
        
        // 2. Replay all the way to the end
        if let Some(last_ev) = sim.events.last() {
            let final_seq = last_ev.sequence_id();
            let state = handle_replay(final_seq, &sim).unwrap();
            
            // 3. Verify deterministic state: state.last_sequence_id MUST match final event
            assert_eq!(state.last_sequence_id, final_seq);
            
            // 5. Verify PnL integrity
            assert_eq!(state.portfolio.pnl, sim.pnl);
        }
    }
}
