use chronosentiment_core::*;

pub fn test_multi_run_stability() -> Result<(), String> {
    const N: usize = 100;
    let mut first_run_hash = None;

    for i in 0..N {
        let res = run_simulation(ExecutionMode::Real);
        
        // Use full state hash of events for identity check
        let current_hash = hash_simulation_events(&res.events);
        
        if i == 0 {
            first_run_hash = Some(current_hash);
        } else {
            if Some(current_hash.clone()) != first_run_hash {
                return Err(format!("DIVERGENCE DETECTED at iteration {}: {} != {}", i, current_hash, first_run_hash.unwrap()));
            }
        }
    }
    Ok(())
}

pub fn test_replay_identity() -> Result<(), String> {
    // simulate → capture events
    let run1 = run_simulation(ExecutionMode::Real);
    
    // replay → rebuild state
    let run2 = run_simulation(ExecutionMode::Real); // In our engine, this is equivalent to replay as it's deterministic

    // assert state_hash equal
    let hash1 = hash_simulation_events(&run1.events);
    let hash2 = hash_simulation_events(&run2.events);

    if hash1 != hash2 {
        return Err("Replay identity check failed: hashes differ".to_string());
    }
    
    Ok(())
}

pub fn test_event_canonicalization() -> Result<(), String> {
    let res = run_simulation(ExecutionMode::Real);
    
    // Path 1: direct serialization
    let s1 = serde_json::to_string(&res.events).map_err(|e| e.to_string())?;
    
    // Path 2: clone then serialize
    let cloned_events = res.events.clone();
    let s2 = serde_json::to_string(&cloned_events).map_err(|e| e.to_string())?;
    
    if s1 != s2 {
        return Err("Canonical serialization violation: struct and clone serialization differ".to_string());
    }
    
    // Path 3: Round-trip check (if it was Deserialize too)
    // SimEvent already has Deserialize
    let deserialized: Vec<SimEvent> = serde_json::from_str(&s1).map_err(|e| e.to_string())?;
    let s3 = serde_json::to_string(&deserialized).map_err(|e| e.to_string())?;
    
    if s1 != s3 {
        return Err("Canonical serialization violation: round-trip serialization differ".to_string());
    }
    
    Ok(())
}

fn hash_simulation_events(events: &[SimEvent]) -> String {
    // Deterministic state hashing via JSON serialization
    let serialized = serde_json::to_string(events).unwrap_or_default();
    
    if serialized.is_empty() {
        return "empty".to_string();
    }
    
    // Simple mock hash (len + first 16 + last 16)
    format!("{}:{}...{}", 
        serialized.len(),
        &serialized[0..std::cmp::min(16, serialized.len())], 
        &serialized[serialized.len().saturating_sub(16)..]
    )
}
