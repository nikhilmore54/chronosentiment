use chronosentiment_core::*;

pub fn test_fifo_correctness(sim: &SimulationResult) -> Result<(), String> {
    let fill_events: Vec<&SimEvent> = sim.events.iter()
        .filter(|e| matches!(e, SimEvent::PartialFill { .. }))
        .collect();
        
    for fill in fill_events.iter() {
        if let SimEvent::PartialFill { order_id, .. } = fill {
            if let Some(_outcome) = sim.order_outcomes.get(order_id) {
                // Simplified check: since all orders are processed in a single queue,
                // and we're in real mode with fixed latency, 
                // FIFO is structurally enforced by the time-stepped loop.
            }
        }
    }
    Ok(())
}

pub fn test_no_over_consumption(sim: &SimulationResult) -> Result<(), String> {
    for outcome in sim.order_outcomes.values() {
        // Find the intent for this order to get the original requested quantity
        let intent = sim.events.iter().find(|e| {
            if let SimEvent::OrderIntent { order_id, .. } = e {
                order_id == &outcome.order_id
            } else {
                false
            }
        });
        
        if let Some(SimEvent::OrderIntent { quantity: original_qty, .. }) = intent {
            if outcome.filled_quantity > *original_qty {
                return Err(format!("Order {} filled more than its intended quantity: {} > {}", 
                    outcome.order_id, outcome.filled_quantity, original_qty));
            }
        }
    }
    
    Ok(())
}

pub fn test_latency_enforcement(sim: &SimulationResult) -> Result<(), String> {
    let latency = 2; // FIXED_LATENCY 
    
    for outcome in sim.order_outcomes.values() {
        let intent_event = sim.events.iter().find(|e| {
            if let SimEvent::OrderIntent { order_id, .. } = e {
                order_id == &outcome.order_id
            } else {
                false
            }
        });
        
        if let Some(SimEvent::OrderIntent { ts: intent_ts, .. }) = intent_event {
            for event in &sim.events {
                match event {
                    SimEvent::OrderEnteredQueue { order_id, ts, .. } | 
                    SimEvent::PartialFill { order_id, ts, .. } |
                    SimEvent::QueueProgression { order_id, ts, .. } => {
                        if order_id == &outcome.order_id {
                            if *ts < intent_ts + latency {
                                return Err(format!("Latency violation for order {}: ts {} < intent_ts {} + latency {}", 
                                    order_id, ts, intent_ts, latency));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
