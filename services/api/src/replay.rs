use chronosentiment_core::*;
use crate::ApiError;
use crate::timeline::TimelineEvent;

pub fn handle_replay(sim: &SimulationResult, from: u64, to: u64) -> Result<Vec<TimelineEvent>, ApiError> {
    if from > to {
        return Err(ApiError::InvalidInput("from must be <= to".to_string()));
    }

    let mut subset = Vec::new();
    for event in &sim.events {
        let ts = event.timestamp();
        if ts >= from && ts <= to {
            let desc = match event {
                SimEvent::MarketEvent { subtype, .. } => format!("MarketEvent {:?}", subtype),
                SimEvent::OrderIntent { order_id, .. } => format!("OrderIntent {} created", order_id),
                SimEvent::OrderEnteredQueue { order_id, .. } => format!("Order {} entered queue", order_id),
                SimEvent::PartialFill { order_id, filled_qty, .. } => format!("Order {} partial fill qty={}", order_id, filled_qty),
                SimEvent::QueueProgression { order_id, new_quantity_ahead, .. } => format!("Order {} queue progression: q={}", order_id, new_quantity_ahead),
            };
            
            subset.push(TimelineEvent {
                ts: event.timestamp(),
                sequence_id: event.sequence_id(),
                description: desc,
            });
        }
    }

    Ok(subset)
}
