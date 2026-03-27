use chronosentiment_core::{*, ese::run_simulation_with_data};
use crate::{ApiError, market_adapter};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MarketDataSimulateInput {
    pub mode: String,
    pub market_data_jsonl: Vec<String>,
    pub order_intents: Vec<CreateOrder>,
}

pub fn handle_simulate_with_market_data(input: MarketDataSimulateInput) -> Result<crate::SimulateOutput, ApiError> {
    let mode = match input.mode.as_str() {
        "real" => ExecutionMode::Real,
        "ideal" => ExecutionMode::Ideal,
        _ => return Err(ApiError::InvalidInput("mode must be 'real' or 'ideal'".to_string())),
    };

    let market_events = market_adapter::parse_market_data(input.market_data_jsonl);

    // Baseline Validation 1: Determinism Check
    let res1 = run_simulation_with_data(mode, market_events.clone(), input.order_intents.clone());
    let res2 = run_simulation_with_data(mode, market_events.clone(), input.order_intents.clone());

    if res1.pnl != res2.pnl || res1.trades != res2.trades || res1.events.len() != res2.events.len() {
        return Err(ApiError::InternalError("Determinism violation detected in market data simulation".to_string()));
    }

    // Baseline Validation 2: Event Identity
    for i in 0..res1.events.len() {
        if res1.events[i] != res2.events[i] {
            return Err(ApiError::InternalError(format!("Event mismatch at sequence {} in market data simulation", i)));
        }
    }

    let state_hash = format!("{:x}", res1.pnl.abs()); 

    Ok(crate::SimulateOutput {
        pnl: res1.pnl,
        trade_count: res1.trades,
        events: res1.events.clone(),
        state_hash,
        original_result: res1,
    })
}
