use chronosentiment_core::*;

pub fn test_full_chain_reconstruction(sim: &SimulationResult) -> Result<(), String> {
    // 1. Pick a PartialFill event
    let partial_fill = sim.events.iter().find(|e| matches!(e, SimEvent::PartialFill { .. }));
    
    if let Some(pf) = partial_fill {
        // 2. Traverse parent_sequence_id
        let mut current_seq = pf.sequence_id();
        let mut steps = 0;
        let max_steps = 1000;
        
        while steps < max_steps {
            let event = sim.events.iter().find(|e| e.sequence_id() == current_seq)
                .ok_or(format!("Missing event with sequence_id: {}", current_seq))?;
            
            if let Some(parent_seq) = event.parent_sequence_id() {
                current_seq = parent_seq;
                steps += 1;
            } else {
                // 3. MUST reach root OrderIntent (or MarketEvent)
                if matches!(event, SimEvent::OrderIntent { .. }) || matches!(event, SimEvent::MarketEvent { .. }) {
                    return Ok(());
                } else {
                    return Err(format!("Causal chain ended at non-root event: {:?}", event));
                }
            }
        }
        Err("Causal chain too long or circular".to_string())
    } else {
        // If no partial fills, we can't test this specific property on this run
        Ok(()) 
    }
}

pub fn test_no_orphan_events(sim: &SimulationResult) -> Result<(), String> {
    for event in &sim.events {
        match event {
            SimEvent::MarketEvent { .. } | SimEvent::OrderIntent { .. } => {
                // Roots are allowed to not have parents
                continue;
            }
            _ => {
                // All other events MUST have a valid parent
                if event.parent_sequence_id().is_none() {
                    return Err(format!("Orphan event detected: {:?}", event));
                }
                
                let parent_seq = event.parent_sequence_id().unwrap();
                if !sim.events.iter().any(|e| e.sequence_id() == parent_seq) {
                    return Err(format!("Event {:?} has invalid parent_sequence_id: {}", event, parent_seq));
                }
            }
        }
    }
    Ok(())
}

pub fn test_no_cycles(sim: &SimulationResult) -> Result<(), String> {
    // Ensure event graph is a DAG
    for event in &sim.events {
        let mut current_seq = event.sequence_id();
        let mut visited = std::collections::HashSet::new();
        visited.insert(current_seq);
        
        let mut steps = 0;
        let max_steps = 1000;
        
        while steps < max_steps {
            let e = sim.events.iter().find(|ev| ev.sequence_id() == current_seq)
                .ok_or(format!("Missing event in chain: {}", current_seq))?;
            
            if let Some(parent_seq) = e.parent_sequence_id() {
                if visited.contains(&parent_seq) {
                    return Err(format!("Cycle detected at event: {}", parent_seq));
                }
                visited.insert(parent_seq);
                current_seq = parent_seq;
                steps += 1;
            } else {
                break;
            }
        }
    }
    Ok(())
}
