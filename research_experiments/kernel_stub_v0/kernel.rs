use crate::*;

pub fn run_ga(mode: ExecutionMode) -> GAResult {
    match mode {
        ExecutionMode::Ideal => GAResult {
            best_config: "<config_ideal_A_123>".to_string(),
        },
        ExecutionMode::Real => GAResult {
            best_config: "<config_real_diverged_B_456>".to_string(),
        },
    }
}
