use chronosentiment_core::*;

pub fn test_out_of_order_detection(sim: &SimulationResult) -> Result<(), String> {
    // 1. We create a malformed simulation result by tampering with the events vector
    let mut malformed_sim = sim.clone();
    
    if malformed_sim.events.len() >= 2 {
        // Swap two events (violates chronological ordering)
        malformed_sim.events.swap(0, 1);
        
        // Use the existing timeline check to verify it detects the violation
        use crate::certification::timeline::test_strict_ordering;
        if let Err(e) = test_strict_ordering(&malformed_sim) {
            if e.contains("Timestamp violation") || e.contains("Sequence ID violation") {
                return Ok(());
            } else {
                return Err(format!("Incorrect error from out-of-order check: {}", e));
            }
        }
        return Err("Negative test failed: Out-of-order sequence was not detected".to_string());
    }
    Ok(())
}

pub fn test_invalid_parent_chain_detection(sim: &SimulationResult) -> Result<(), String> {
    // 1. We create a malformed simulation result by tampering with the parent sequence ids
    let mut malformed_sim = sim.clone();
    
    // Find a non-root event
    let non_root_idx = malformed_sim.events.iter().position(|e| e.parent_sequence_id().is_some());
    
    if let Some(idx) = non_root_idx {
        // Break the parent reference
        let event = malformed_sim.events[idx].clone();
        let new_event = match event {
            SimEvent::PartialFill { sequence_id, order_id, ts, filled_qty, price, .. } => {
                SimEvent::PartialFill { sequence_id, parent_sequence_id: Some(999999), order_id, ts, filled_qty, price }
            },
            SimEvent::QueueProgression { sequence_id, order_id, ts, new_quantity_ahead, .. } => {
                SimEvent::QueueProgression { sequence_id, parent_sequence_id: Some(999999), order_id, ts, new_quantity_ahead }
            },
            SimEvent::OrderEnteredQueue { sequence_id, order_id, ts, price, quantity_ahead, .. } => {
                SimEvent::OrderEnteredQueue { sequence_id, parent_sequence_id: Some(999999), order_id, ts, price, quantity_ahead }
            },
            _ => event // Should not happen given search
        };
        malformed_sim.events[idx] = new_event;
        
        // Use the existing causality check to verify it detects the violation
        use crate::certification::causality::test_no_orphan_events;
        if let Err(e) = test_no_orphan_events(&malformed_sim) {
            if e.contains("invalid parent_sequence_id") {
                return Ok(());
            } else {
                return Err(format!("Incorrect error from invalid parent check: {}", e));
            }
        }
        return Err("Negative test failed: Invalid parent ID was not detected".to_string());
    }
    Ok(())
}

pub fn test_over_consumption_detection(sim: &SimulationResult) -> Result<(), String> {
    // 1. We create a malformed simulation result by tampering with fill quantities
    let mut malformed_sim = sim.clone();
    
    if let Some(id) = malformed_sim.order_outcomes.keys().next().cloned() {
        let outcome = malformed_sim.order_outcomes.get_mut(&id).unwrap();
        // Inflate fill to be more than its original quantity (assume it was a 100% fill)
        let total = outcome.filled_quantity + outcome.remaining_quantity;
        outcome.filled_quantity = total + 100;
        
        // Use the existing execution check to verify it detects the violation
        use crate::certification::execution::test_no_over_consumption;
        if let Err(e) = test_no_over_consumption(&malformed_sim) {
            if e.contains("filled more than its intended quantity") {
                return Ok(());
            } else {
                return Err(format!("Incorrect error from over-consumption check: {}", e));
            }
        }
        return Err("Negative test failed: Over-consumption was not detected".to_string());
    }
    Ok(())
}
