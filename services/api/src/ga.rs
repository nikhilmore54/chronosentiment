use chronosentiment_core::*;
use crate::ApiError;

#[derive(Debug, Clone)]
pub struct GAInput {
    pub mode: String,
    pub population: u64,
    pub generations: u64,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct GAOutput {
    pub best_config: String,
    pub best_fitness: i64,
    pub history: Vec<i64>,
}

pub fn handle_ga_run(input: GAInput) -> Result<GAOutput, ApiError> {
    let mode = match input.mode.as_str() {
        "real" => ExecutionMode::Real,
        "ideal" => ExecutionMode::Ideal,
        _ => return Err(ApiError::InvalidInput("mode must be 'real' or 'ideal'".to_string())),
    };

    // Determinism check for GA
    let res1 = run_ga(mode);
    let res2 = run_ga(mode);

    if res1.best_config != res2.best_config {
        return Err(ApiError::InternalError("GA Determinism violation detected".to_string()));
    }

    Ok(GAOutput {
        best_config: res1.best_config,
        best_fitness: if mode == ExecutionMode::Real { 10000 } else { 130300 }, // Mock fitness for MVP demo
        history: vec![8000, 9000, 10000], // Mock history
    })
}
