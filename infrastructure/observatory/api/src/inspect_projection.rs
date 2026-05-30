//! Replay → UI projection helpers for `POST /inspect_strategy`.
//! Converts certified `SimEvent` traces into contract-facing `EventWrapper` values.

use chronosentiment_core::{
    deterministic_demo_fixture, run_simulation_harness, ExecutionMode, SimEvent, SimulationResult,
};
use chronosentiment_optimization::Candidate;
use serde_json::json;

use crate::dto::{EventWrapper, SourceLayer};
use crate::inspector::{self, EventType, MinimalEvent};

const PRIMARY_ORDER_ID: &str = "O1";

/// Deterministic inspect simulation: strategy parameters influence order sizing;
/// market substrate remains the certified demo fixture (replay-stable).
pub fn run_inspect_simulation(
    strategy: &Candidate,
    seed: u64,
) -> (SimulationResult, String) {
    let (market_events, mut orders) = deterministic_demo_fixture();

    if let Some(primary) = orders.iter_mut().find(|o| o.order_id == PRIMARY_ORDER_ID) {
        primary.quantity = 100 + (strategy.queue_threshold % 500);
        primary.timestamp = 2 + (seed % 3);
        primary.price = 100 + (strategy.base_edge % 5);
    }

    let (_events, result, _hash) =
        run_simulation_harness(ExecutionMode::Real, market_events, orders);

    (result, PRIMARY_ORDER_ID.to_string())
}

pub fn sim_event_to_wrapper(event: &SimEvent) -> EventWrapper {
    let minimal = inspector::to_minimal_event(event);
    minimal_to_wrapper(&minimal, event)
}

pub fn minimal_to_wrapper(minimal: &MinimalEvent, source: &SimEvent) -> EventWrapper {
    let event_type = event_type_name(&minimal.event_type);

    let mut payload = serde_json::Map::new();
    if let Some(q) = minimal.queue_ahead {
        payload.insert("queue_ahead".to_string(), json!(q));
    }
    if let Some(f) = minimal.filled_qty {
        payload.insert("filled_qty".to_string(), json!(f));
    }
    if let Some(p) = minimal.price {
        payload.insert("price".to_string(), json!(p));
    }
    if let Some(order_id) = source.order_id() {
        payload.insert("order_id".to_string(), json!(order_id));
    }
    if let SimEvent::OrderIntent { side, quantity, .. } = source {
        payload.insert("side".to_string(), json!(format!("{:?}", side).to_uppercase()));
        payload.insert("quantity".to_string(), json!(quantity));
    }

    EventWrapper {
        sequence_id: minimal.sequence_id,
        timestamp: minimal.timestamp,
        event_type,
        parent_sequence_id: minimal.parent_sequence_id,
        payload: serde_json::Value::Object(payload),
        source_layer: SourceLayer::Sequencer,
        kernel_signature: String::new(),
    }
}

fn event_type_name(event_type: &EventType) -> String {
    serde_json::to_value(event_type)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "UNKNOWN".to_string())
}

pub fn primary_order_id() -> &'static str {
    PRIMARY_ORDER_ID
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronosentiment_optimization::Candidate;

    #[test]
    fn inspect_simulation_is_deterministic_for_same_inputs() {
        let strategy = Candidate::default();
        let (a, _) = run_inspect_simulation(&strategy, 42);
        let (b, _) = run_inspect_simulation(&strategy, 42);
        assert_eq!(a.events, b.events);
        assert_eq!(a.pnl, b.pnl);
    }

    #[test]
    fn event_wrappers_use_screaming_snake_types() {
        let strategy = Candidate::default();
        let (sim, _) = run_inspect_simulation(&strategy, 7);
        let wrapper = sim_event_to_wrapper(&sim.events[0]);
        assert!(
            wrapper.event_type.contains('_'),
            "expected SCREAMING_SNAKE_CASE, got {}",
            wrapper.event_type
        );
    }
}
