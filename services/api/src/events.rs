use chronosentiment_core::*;
use crate::{ApiError, EventStreamResponse};

pub fn handle_events(sim: &SimulationResult, from: Option<u64>, to: Option<u64>) -> Result<EventStreamResponse, ApiError> {
    let from = from.unwrap_or(0);
    let to = to.unwrap_or(u64::MAX);

    if from > to {
        return Err(ApiError::InvalidInput("from must be <= to".to_string()));
    }

    let subset: Vec<SimEvent> = sim.events.iter()
        .filter(|e| {
            let seq = e.sequence_id();
            seq >= from && seq <= to
        })
        .cloned()
        .collect();

    Ok(EventStreamResponse { events: subset })
}
