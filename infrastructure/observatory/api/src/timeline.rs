use crate::ApiError;
use chronosentiment_core::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct TimelineEvent {
    pub timestamp: u64,
    pub sequence_id: u64,
    pub description: String,
}

pub fn handle_timeline(sim: &SimulationResult) -> Result<Vec<TimelineEvent>, ApiError> {
    let mut timeline = Vec::new();
    for event in &sim.events {
        let desc = match event {
            SimEvent::MarketEvent { subtype, .. } => format!("MarketEvent {:?}", subtype),
            SimEvent::OrderIntent { order_id, .. } => format!("OrderIntent {} created", order_id),
            SimEvent::OrderEnteredQueue { order_id, .. } => {
                format!("Order {} entered queue", order_id)
            }
            SimEvent::PartialFill {
                order_id,
                filled_qty,
                ..
            } => format!("Order {} partial fill qty={}", order_id, filled_qty),
            SimEvent::QueueProgression {
                order_id,
                queue_ahead,
                ..
            } => format!("Order {} queue progression: q={}", order_id, queue_ahead),
            SimEvent::OrderFilled { order_id, .. } => format!("Order {} filled", order_id),
        };

        timeline.push(TimelineEvent {
            timestamp: event.timestamp(),
            sequence_id: event.sequence_id(),
            description: desc,
        });
    }

    // Baseline validation: check timeline ordering
    for i in 0..timeline.len() - 1 {
        let e1 = &timeline[i];
        let e2 = &timeline[i + 1];
        if e1.timestamp == e2.timestamp {
            if e1.sequence_id >= e2.sequence_id {
                return Err(ApiError::InternalError(
                    "Timeline sequence_id ordering violation".to_string(),
                ));
            }
        } else if e1.timestamp > e2.timestamp {
            return Err(ApiError::InternalError(
                "Timeline timestamp ordering violation".to_string(),
            ));
        }
    }

    Ok(timeline)
}
