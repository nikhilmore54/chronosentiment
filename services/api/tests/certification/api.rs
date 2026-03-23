use chronosentiment_core::*;
use chronosentiment_api::*;

pub fn test_stateless_api() -> Result<(), String> {
    // 1. call /simulate twice → identical output
    let sim1 = run_simulation(ExecutionMode::Real);
    let sim2 = run_simulation(ExecutionMode::Real);
    
    if sim1.pnl != sim2.pnl || sim1.events.len() != sim2.events.len() {
        return Err("API is stateful: successive calls differ".to_string());
    }
    
    Ok(())
}

pub fn test_read_endpoints_do_not_mutate(sim: &SimulationResult) -> Result<(), String> {
    // Call: /events, /inspect, /timeline
    let _events = handle_events(&sim, Some(0), Some(u64::MAX)).map_err(|e| format!("{:?}", e))?;
    let _inspection = handle_inspect("order_1".to_string(), &sim).map_err(|e| format!("{:?}", e)); 
    let _timeline = handle_timeline(&sim).map_err(|e| format!("{:?}", e))?;
    
    // Rerun simulation → results must match
    let sim2 = run_simulation(ExecutionMode::Real);
    
    if sim.pnl != sim2.pnl || sim.events.len() != sim2.events.len() {
        return Err("Read endpoints mutated state".to_string());
    }
    
    Ok(())
}
