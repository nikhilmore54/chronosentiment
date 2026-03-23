use chronosentiment_core::*;
use crate::ApiError;

pub fn handle_inspect(order_id: String, sim: &SimulationResult) -> Result<TradeInspection, ApiError> {
    if !sim.order_outcomes.contains_key(&order_id) {
        return Err(ApiError::InvalidInput(format!("Order {} not found", order_id)));
    }

    Ok(inspect_trade(&order_id, sim))
}
