use crate::ese::{run_simulation_with_data, FIXED_LATENCY};
use crate::{CreateOrder, ExecutionMode, MarketEvent, SimEvent, SimulationResult, OrderOutcome, MarketEventType, Side};
use serde_json;
use blake3;
use std::collections::HashMap;

// A simpler, more controlled simulation runner for the harness
pub fn run_simulation_harness(
    mode: ExecutionMode,
    market_events: Vec<MarketEvent>,
    create_orders: Vec<CreateOrder>,
) -> (Vec<SimEvent>, SimulationResult, String) {
    let result = run_simulation_with_data(mode, market_events, create_orders);
    
    let events_output = result.events.clone();

    // Serialize the SimulationResult (excluding events, which are explicitly part of the event_log) for hashing
    // We need to create a temporary struct or manually serialize to exclude events from the state hash, as the problem statement 
    // implies event_log is separate from final_state for hashing purposes.
    #[derive(serde::Serialize)]
    struct StateForHashing {
        pnl: i64,
        trades: u64,
        order_outcomes: Vec<(String, OrderOutcome)>,
    }

    let mut sorted_order_outcomes: Vec<(String, OrderOutcome)> = result.order_outcomes.clone().into_iter().collect();
    sorted_order_outcomes.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    let state_for_hashing = StateForHashing {
        pnl: result.pnl,
        trades: result.trades,
        order_outcomes: sorted_order_outcomes,
    };

    let serialized_state = serde_json::to_string(&state_for_hashing).expect("Failed to serialize state for hashing");
    let state_hash = blake3::hash(serialized_state.as_bytes()).to_string();

    (events_output, result, state_hash)
}

/// Deterministic market + order set used by integration tests (`run_simulation`, determinism harness).
pub fn deterministic_demo_fixture() -> (Vec<MarketEvent>, Vec<CreateOrder>) {
    let market_events = vec![
        MarketEvent {
            subtype: MarketEventType::NewOrder,
            price: 100,
            quantity: 500,
            side: Some(Side::Sell),
            exchange_ts: 1,
        },
        MarketEvent {
            subtype: MarketEventType::Trade,
            price: 100,
            quantity: 200,
            side: None,
            exchange_ts: 4,
        },
        MarketEvent {
            subtype: MarketEventType::NewOrder,
            price: 101,
            quantity: 800,
            side: Some(Side::Sell),
            exchange_ts: 9,
        },
        MarketEvent {
            subtype: MarketEventType::Trade,
            price: 101,
            quantity: 300,
            side: None,
            exchange_ts: 10,
        },
    ];

    let create_orders = vec![
        CreateOrder {
            order_id: "O1".to_string(),
            side: Side::Buy,
            price: 100,
            quantity: 600,
            timestamp: 2,
            fill_probability: 1.0,
        },
        CreateOrder {
            order_id: "O2".to_string(),
            side: Side::Buy,
            price: 100,
            quantity: 400,
            timestamp: 5,
            fill_probability: 1.0,
        },
        CreateOrder {
            order_id: "O3".to_string(),
            side: Side::Buy,
            price: 101,
            quantity: 300,
            timestamp: 9,
            fill_probability: 1.0,
        },
    ];
    (market_events, create_orders)
}

/// Same inputs as [`deterministic_demo_fixture`]; used by demo/determinism/explainability integration tests.
pub fn run_simulation(mode: ExecutionMode) -> SimulationResult {
    let (market_events, create_orders) = deterministic_demo_fixture();
    let (_, result, _) = run_simulation_harness(mode, market_events, create_orders);
    result
}

// Replay engine that reconstructs the final state and computes its hash
pub fn replay_harness(event_log: Vec<SimEvent>) -> (SimulationResult, String) {
    let mut pnl = 0i64;
    let mut total_trades = 0u64;
    let mut order_outcomes: HashMap<String, OrderOutcome> = HashMap::new();
    // Simplified market state for replay, tracking available liquidity at price levels
    let mut market_liquidity: HashMap<u64, u64> = HashMap::new(); 

    for event in event_log.iter() {
        match event {
            SimEvent::MarketEvent { subtype, price, quantity, .. } => {
                match subtype {
                    MarketEventType::NewOrder => { *market_liquidity.entry(*price).or_insert(0) += quantity; },
                    MarketEventType::Trade => { *market_liquidity.entry(*price).or_insert(0) = market_liquidity.get(price).unwrap_or(&0).saturating_sub(*quantity); },
                    MarketEventType::Cancel => { *market_liquidity.entry(*price).or_insert(0) = market_liquidity.get(price).unwrap_or(&0).saturating_sub(*quantity); },
                }
            },
            SimEvent::OrderIntent { order_id, side: _side, price: _price, quantity, timestamp, .. } => {
                order_outcomes.insert(
                    order_id.clone(),
                    OrderOutcome {
                        order_id: order_id.clone(),
                        filled_quantity: 0,
                        remaining_quantity: *quantity,
                        arrival_time: *timestamp + FIXED_LATENCY, // Apply fixed latency as in simulation
                        queue_ahead: 0, // This is explicitly set by OrderEnteredQueue
                    },
                );
            },
            SimEvent::OrderEnteredQueue { order_id, timestamp, queue_ahead, .. } => {
                if let Some(outcome) = order_outcomes.get_mut(order_id) {
                    outcome.arrival_time = *timestamp;
                    outcome.queue_ahead = *queue_ahead;
                }
            },
            SimEvent::PartialFill { order_id, filled_qty, price, .. } => {
                if let Some(outcome) = order_outcomes.get_mut(order_id) {
                    outcome.filled_quantity += filled_qty;
                    outcome.remaining_quantity -= filled_qty;
                    pnl += (filled_qty * price) as i64;
                    total_trades += 1;
                }
            },
            SimEvent::OrderFilled { .. } => {
                // For determinism harness, OrderFilled event simply marks the order as complete.
                // All quantity updates are handled by PartialFill.
            }
            SimEvent::QueueProgression { order_id, queue_ahead, .. } => {
                if let Some(outcome) = order_outcomes.get_mut(order_id) {
                    outcome.queue_ahead = *queue_ahead;
                }
            }
        }
    }

    let reconstructed_result = SimulationResult {
        pnl,
        trades: total_trades,
        order_outcomes: order_outcomes.clone(),
        events: event_log, // This is the original event log, not a reconstructed one
    };
    
    #[derive(serde::Serialize)]
    struct StateForHashing {
        pnl: i64,
        trades: u64,
        order_outcomes: Vec<(String, OrderOutcome)>,
    }

    let mut sorted_order_outcomes: Vec<(String, OrderOutcome)> = order_outcomes.into_iter().collect();
    sorted_order_outcomes.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

    let state_for_hashing = StateForHashing {
        pnl: reconstructed_result.pnl,
        trades: reconstructed_result.trades,
        order_outcomes: sorted_order_outcomes,
    };

    let serialized_state = serde_json::to_string(&state_for_hashing).expect("Failed to serialize reconstructed state for hashing");
    let reconstructed_hash = blake3::hash(serialized_state.as_bytes()).to_string();

    (reconstructed_result, reconstructed_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_deterministic_inputs() -> (Vec<MarketEvent>, Vec<CreateOrder>) {
        deterministic_demo_fixture()
    }

    #[test]
    fn test_determinism_certification_harness() {
        let (market_events, create_orders) = get_deterministic_inputs();

        println!("\n--- Running Determinism Certification Harness ---");

        // 3. Determinism Test
        println!("\n- - Determinism Test - -");
        let (events1, result1, hash1) = run_simulation_harness(ExecutionMode::Real, market_events.clone(), create_orders.clone());
        let (events2, result2, hash2) = run_simulation_harness(ExecutionMode::Real, market_events.clone(), create_orders.clone());

        assert_eq!(events1, events2, "Event logs diverged");
        assert_eq!(result1.pnl, result2.pnl, "PNL diverged");
        assert_eq!(result1.trades, result2.trades, "Trades diverged");
        assert_eq!(result1.order_outcomes, result2.order_outcomes, "Order outcomes diverged");
        assert_eq!(hash1, hash2, "State hashes diverged");
        println!("  ✅ Simulation runs produced identical event logs, states, and hashes.");

        // 4. Replay Validation
        println!("\n- - Replay Validation - -");
        let (reconstructed_result, reconstructed_hash) = replay_harness(events1.clone());

        assert_eq!(reconstructed_result.pnl, result1.pnl, "Replayed PNL diverged");
        assert_eq!(reconstructed_result.trades, result1.trades, "Replayed trades diverged");
        assert_eq!(reconstructed_result.order_outcomes, result1.order_outcomes, "Replayed order outcomes diverged");
        assert_eq!(reconstructed_hash, hash1, "Replayed hash diverged");
        println!("  ✅ Replay produced identical state and hash.");

        // 5. Cross-run Safety (N=10)
        println!("\n- - Cross-run Safety (N=10) - -");
        let mut hashes = Vec::new();
        for i in 0..10 {
            let (_, _, current_hash) = run_simulation_harness(ExecutionMode::Real, market_events.clone(), create_orders.clone());
            hashes.push(current_hash);
            println!("  Run {}: {}", i + 1, hashes[i]);
        }

        let first_hash = &hashes[0];
        assert!(hashes.iter().all(|h| h == first_hash), "Cross-run hashes diverged");
        println!("  ✅ All 10 simulation runs produced identical hashes.");

        // 6. Output
        println!("\n--- Verification Results ---");
        println!("Event Log (first 5 events): {:#?}", &events1[0..5]);
        println!("Final State Hash: {}", hash1);
        println!("Verification Result: SUCCESS");
    }

    #[test]
    fn determinism_multiple_runs_ci() {
        let (market_events, create_orders) = get_deterministic_inputs();
        let mut hashes = Vec::new();

        for i in 0..5 {
            let (_, _, current_hash) = run_simulation_harness(ExecutionMode::Real, market_events.clone(), create_orders.clone());
            hashes.push(current_hash);
            println!("  CI Run {}: {}", i + 1, hashes[i]);
        }

        let first_hash = &hashes[0];
        assert!(hashes.iter().all(|h| h == first_hash), "CI multiple runs hashes diverged");
        println!("  ✅ All 5 CI simulation runs produced identical hashes.");
    }
}
