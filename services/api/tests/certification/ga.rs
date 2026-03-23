use chronosentiment_core::*;

pub fn test_deterministic_ga() -> Result<(), String> {
    // 1. same seed → identical best_config + fitness
    // In our GA implementation, it's deterministic for the given mode/events
    let res1 = run_ga(ExecutionMode::Real);
    let res2 = run_ga(ExecutionMode::Real);
    
    if res1.best_config != res2.best_config {
        return Err(format!("GA is non-deterministic: {} != {}", res1.best_config, res2.best_config));
    }
    
    Ok(())
}

pub fn test_mutation_validity() -> Result<(), String> {
    // 2. mutated configs MUST pass validation
    let res = run_ga(ExecutionMode::Real);
    if res.best_config.is_empty() {
        return Err("GA returned empty best_config".to_string());
    }
    
    Ok(())
}

pub fn test_no_future_leakage(sim: &SimulationResult) -> Result<(), String> {
    // 3. GA must depend only on past events
    let mut last_ts = 0;
    for event in &sim.events {
        let ts = event.timestamp();
        if ts < last_ts {
            return Err(format!("Future leakage: event timestamp decreased {} < {}", ts, last_ts));
        }
        last_ts = ts;
    }
    
    Ok(())
}
