use crate::*;

pub fn inspect_trade(order_id_to_find: &str, simulation: &SimulationResult) -> TradeInspection {
    let mut decision = DecisionLayer {
        order_id: String::new(),
        side: Side::Buy,
        price: 0,
        quantity: 0,
        timestamp: 0,
    };
    
    let mut execution = ExecutionLayer {
        arrival_time: 0,
        latency_applied: crate::ese::FIXED_LATENCY,
        queue_ahead_initial: 0,
        queue_progression: Vec::new(),
        fills: Vec::new(),
        causal_chain: Vec::new(),
    };

    let mut last_order_event_seq = None;

    for event in &simulation.events {
        match event {
            SimEvent::OrderIntent { order_id, side, price, quantity, ts, sequence_id, .. } if order_id == order_id_to_find => {
                decision.order_id = order_id.clone();
                decision.side = *side;
                decision.price = *price;
                decision.quantity = *quantity;
                decision.timestamp = *ts;
                last_order_event_seq = Some(*sequence_id);
            }
            SimEvent::OrderEnteredQueue { order_id, ts, quantity_ahead, sequence_id, .. } if order_id == order_id_to_find => {
                execution.arrival_time = *ts;
                execution.queue_ahead_initial = *quantity_ahead;
                execution.queue_progression.push(*quantity_ahead);
                last_order_event_seq = Some(*sequence_id);
            }
            SimEvent::QueueProgression { order_id, new_quantity_ahead, sequence_id, .. } if order_id == order_id_to_find => {
                execution.queue_progression.push(*new_quantity_ahead);
                last_order_event_seq = Some(*sequence_id);
            }
            SimEvent::PartialFill { order_id, ts, filled_qty, price, sequence_id, .. } if order_id == order_id_to_find => {
                execution.fills.push(FillEvent {
                    ts: *ts,
                    qty: *filled_qty,
                    price: *price,
                });
                last_order_event_seq = Some(*sequence_id);
            }
            _ => {}
        }
    }

    if let Some(sid) = last_order_event_seq {
        execution.causal_chain = reconstruct_chain(&simulation.events, sid);
    }

    let outcome_info = simulation.order_outcomes.get(order_id_to_find).expect("Order not found in outcomes");
    let outcome = OutcomeLayer {
        filled_quantity: outcome_info.filled_quantity,
        remaining_quantity: outcome_info.remaining_quantity,
        average_price: if outcome_info.filled_quantity > 0 { decision.price } else { 0 },
    };

    TradeInspection { decision, execution, outcome }
}

pub fn reconstruct_chain(events: &Vec<SimEvent>, start_seq_id: u64) -> Vec<SimEvent> {
    let mut chain = Vec::new();
    let mut current_id = Some(start_seq_id);

    while let Some(sid) = current_id {
        if let Some(event) = events.iter().find(|e| e.sequence_id() == sid) {
            chain.push(event.clone());
            current_id = event.parent_sequence_id();
        } else {
            break;
        }
    }
    chain.reverse();
    chain
}
