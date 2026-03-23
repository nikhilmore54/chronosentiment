use chronosentiment_core::*;
use chronosentiment_api::*;
use serde_json::Value;

pub fn test_api_contract_lock() -> Result<(), String> {
    let sim = run_simulation(ExecutionMode::Real);
    
    // 1. /simulate endpoint contract
    let sim_output = handle_simulate(SimulateInput {
        mode: "real".to_string(),
        dataset: None,
        seed: 42,
    }).map_err(|e| format!("{:?}", e))?;
    let sim_val = serde_json::to_value(&sim_output).map_err(|e| e.to_string())?;
    verify_keys(&sim_val, &["pnl", "trade_count", "events", "state_hash"], "/simulate")?;
    
    // 2. /events endpoint contract
    let events_output = handle_events(&sim, Some(0), Some(u64::MAX)).map_err(|e| format!("{:?}", e))?;
    let events_val = serde_json::to_value(&events_output).map_err(|e| e.to_string())?;
    verify_keys(&events_val, &["events"], "/events")?;
    
    // 3. /inspect endpoint contract
    let inspect_output = handle_inspect("O1".to_string(), &sim).map_err(|e| format!("{:?}", e))?;
    let inspect_val = serde_json::to_value(&inspect_output).map_err(|e| e.to_string())?;
    verify_keys(&inspect_val, &["decision", "execution", "outcome"], "/inspect")?;
    
    // 4. /timeline endpoint contract
    let timeline_output = handle_timeline(&sim).map_err(|e| format!("{:?}", e))?;
    let timeline_val = serde_json::to_value(&timeline_output).map_err(|e| e.to_string())?;
    // timeline is a list of events in our MVP
    if !timeline_val.is_array() {
        return Err("/timeline output is not an array".to_string());
    }
    
    // 5. /certify endpoint contract
    let certify_output = handle_certify(&sim).map_err(|e| format!("{:?}", e))?;
    let certify_val = serde_json::to_value(&certify_output).map_err(|e| e.to_string())?;
    verify_keys(&certify_val, &["status", "hash_1", "hash_2", "divergence_point", "fingerprint"], "/certify")?;
    
    Ok(())
}

fn verify_keys(val: &Value, expected_keys: &[&str], endpoint: &str) -> Result<(), String> {
    if let Value::Object(map) = val {
        for key in expected_keys {
            if !map.contains_key(*key) {
                return Err(format!("Endpoint {} missing key '{}' in response", endpoint, key));
            }
        }
        Ok(())
    } else {
        Err(format!("Endpoint {} output is not an object", endpoint))
    }
}
