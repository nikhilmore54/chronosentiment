use chronosentiment_core::*;
use crate::ApiError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SimulateInput {
    pub mode: String,
    pub dataset: Option<String>,
    pub seed: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SimulateOutput {
    pub pnl: i64,
    pub trade_count: u64,
    pub events: Vec<SimEvent>,
    pub state_hash: String,
    #[serde(skip)]
    pub original_result: SimulationResult, // Internal use
}

pub fn handle_simulate(input: SimulateInput) -> Result<SimulateOutput, ApiError> {
    let mode = match input.mode.as_str() {
        "real" => ExecutionMode::Real,
        "ideal" => ExecutionMode::Ideal,
        _ => return Err(ApiError::InvalidInput("mode must be 'real' or 'ideal'".to_string())),
    };

    // Baseline Validation 1: Determinism Check
    let res1 = run_simulation(mode);
    let res2 = run_simulation(mode);

    if res1.pnl != res2.pnl || res1.trades != res2.trades || res1.events.len() != res2.events.len() {
        return Err(ApiError::InternalError("Determinism violation detected".to_string()));
    }

    // Baseline Validation 2: Event Identity
    for i in 0..res1.events.len() {
        if res1.events[i] != res2.events[i] {
            return Err(ApiError::InternalError(format!("Event mismatch at sequence {}", i)));
        }
    }

    let state_hash = format!("{:x}", res1.pnl.abs()); // Simple deterministic hash for MVP

    Ok(SimulateOutput {
        pnl: res1.pnl,
        trade_count: res1.trades,
        events: res1.events.clone(),
        state_hash,
        original_result: res1,
    })
}
