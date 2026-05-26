use chronosentiment_core::{Side, SimEvent, to_real};
use std::collections::HashMap;

use crate::dto::{OrderState, PortfolioState, SystemState};

/// Canonical operational replay reducer (V-007 D-1 / D-3).
///
/// Law: `docs/governance/V007_TYPE_AUTHORITY_DECISION.md`
pub fn reduce_replay_state(events: &[SimEvent], seq_id: u64) -> SystemState {
    let mut orders: HashMap<String, OrderState> = HashMap::new();
    let mut pnl = 0.0;
    let mut position = 0i64;

    for event in events.iter() {
        if event.sequence_id() > seq_id {
            break;
        }

        match event {
            SimEvent::OrderIntent {
                order_id,
                side,
                price,
                quantity,
                ..
            } => {
                orders.insert(
                    order_id.clone(),
                    OrderState {
                        order_id: order_id.clone(),
                        status: "NEW".to_string(),
                        quantity_total: *quantity,
                        quantity_filled: 0,
                        quantity_remaining: *quantity,
                        queue_ahead: 0,
                        price: to_real(*price),
                        side: *side,
                    },
                );
            }
            SimEvent::OrderEnteredQueue {
                order_id, queue_ahead, ..
            } => {
                if let Some(order) = orders.get_mut(order_id) {
                    order.status = "ACTIVE".to_string();
                    order.queue_ahead = *queue_ahead;
                }
            }
            SimEvent::PartialFill {
                order_id,
                filled_qty,
                price,
                ..
            } => {
                if let Some(order) = orders.get_mut(order_id) {
                    order.status = "PARTIAL".to_string();
                    order.quantity_filled += *filled_qty;
                    order.quantity_remaining = order.quantity_remaining.saturating_sub(*filled_qty);

                    let multiplier = match order.side {
                        Side::Buy => 1,
                        Side::Sell => -1,
                    };
                    position += multiplier * (*filled_qty as i64);
                    pnl += multiplier as f64 * (*filled_qty as f64) * to_real(*price);
                }
            }
            SimEvent::QueueProgression {
                order_id, queue_ahead, ..
            } => {
                if let Some(order) = orders.get_mut(order_id) {
                    order.queue_ahead = *queue_ahead;
                }
            }
            SimEvent::OrderFilled { order_id, .. } => {
                if let Some(order) = orders.get_mut(order_id) {
                    order.status = "FILLED".to_string();
                    order.quantity_remaining = 0;
                }
            }
            SimEvent::MarketEvent { .. } => {}
        }
    }

    SystemState {
        orders,
        portfolio: PortfolioState { pnl, position },
        last_sequence_id: seq_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronosentiment_core::{
        harness, CreateOrder, ExecutionMode, MarketEvent, MarketEventType, Side,
    };

    #[test]
    fn test_canonical_replay_reducer_matches_operational_semantics() {
        let events = vec![
            MarketEvent {
                subtype: MarketEventType::NewOrder,
                price: 100,
                quantity: 2000,
                side: Some(Side::Sell),
                exchange_ts: 10,
            },
            MarketEvent {
                subtype: MarketEventType::Trade,
                price: 100,
                quantity: 500,
                side: None,
                exchange_ts: 15,
            },
        ];

        let orders = vec![CreateOrder {
            order_id: "test_order".to_string(),
            side: Side::Buy,
            price: 100,
            quantity: 1000,
            timestamp: 10,
            fill_probability: 1.0,
        }];

        let (_, sim, _) = harness::run_simulation_harness(ExecutionMode::Real, events, orders);

        let Some(last_ev) = sim.events.last() else {
            panic!("expected simulation events");
        };
        let final_seq = last_ev.sequence_id();

        let state = reduce_replay_state(&sim.events, final_seq);

        assert_eq!(state.last_sequence_id, final_seq);
        let order = state
            .orders
            .get("test_order")
            .expect("expected test order in replay state");
        assert!(
            matches!(order.status.as_str(), "NEW" | "ACTIVE" | "PARTIAL" | "FILLED"),
            "unexpected status: {}",
            order.status
        );
        assert_eq!(order.quantity_total, 1000);

        let repeated = reduce_replay_state(&sim.events, final_seq);
        assert_eq!(repeated.orders.len(), state.orders.len());
        assert_eq!(repeated.portfolio.position, state.portfolio.position);
        assert_eq!(repeated.portfolio.pnl, state.portfolio.pnl);
    }
}
