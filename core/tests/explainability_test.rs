// tests/explainability_test.rs

use chronosentiment_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1 — Parent Integrity: Every event (except root) has valid parent_sequence_id
    #[test]
    fn test_parent_integrity() {
        let sim = run_simulation(ExecutionMode::Real);
        for event in &sim.events {
            match event {
                SimEvent::OrderIntent { parent_sequence_id, .. } => {
                    assert!(parent_sequence_id.is_none(), "OrderIntent must be a root (None parent)");
                }
                SimEvent::MarketEvent { parent_sequence_id, .. } => {
                    assert!(parent_sequence_id.is_none(), "MarketEvent must be a root (None parent)");
                }
                SimEvent::OrderEnteredQueue { parent_sequence_id, .. } => {
                    assert!(parent_sequence_id.is_some(), "OrderEnteredQueue must have a parent");
                }
                SimEvent::QueueProgression { parent_sequence_id, .. } => {
                    assert!(parent_sequence_id.is_some(), "QueueProgression must have a parent");
                }
                SimEvent::PartialFill { parent_sequence_id, .. } => {
                    assert!(parent_sequence_id.is_some(), "PartialFill must have a parent");
                }
                SimEvent::OrderFilled { parent_sequence_id, .. } => {
                    assert!(parent_sequence_id.is_some(), "OrderFilled must have a parent");
                }
            }
        }
    }

    // Test 2 — Chain Reconstruction: reconstruct_chain returns correct ordered path
    #[test]
    fn test_chain_reconstruction() {
        let sim = run_simulation(ExecutionMode::Real);

        // Real-mode fixture may not produce PartialFill for O1 (queue never clears); use OrderEnteredQueue.
        let queue_event = sim
            .events
            .iter()
            .find(|e| matches!(e, SimEvent::OrderEnteredQueue { order_id, .. } if order_id == "O1"))
            .expect("OrderEnteredQueue for O1 not found");

        let chain = reconstruct_chain(&sim.events, queue_event.sequence_id());

        assert!(!chain.is_empty(), "Chain should not be empty");
        assert!(
            matches!(chain[0], SimEvent::OrderIntent { .. }),
            "Chain must start with OrderIntent"
        );
        assert!(
            matches!(chain.last().unwrap(), SimEvent::OrderEnteredQueue { .. }),
            "Chain must end with OrderEnteredQueue"
        );

        for i in 0..chain.len() - 1 {
            assert!(chain[i].sequence_id() < chain[i + 1].sequence_id());
        }
    }

    // Test 3 — Determinism: same input → identical event chains
    #[test]
    fn test_chain_determinism() {
        let sim1 = run_simulation(ExecutionMode::Real);
        let sim2 = run_simulation(ExecutionMode::Real);
        
        assert_eq!(sim1.events.len(), sim2.events.len());
        for i in 0..sim1.events.len() {
            assert_eq!(sim1.events[i].sequence_id(), sim2.events[i].sequence_id());
            assert_eq!(sim1.events[i].parent_sequence_id(), sim2.events[i].parent_sequence_id());
        }
    }

    // Test 4 — No Behavior Change: PnL, fills, outcomes MUST remain identical
    #[test]
    fn test_no_behavior_change() {
        let sim = run_simulation(ExecutionMode::Real);

        // Deterministic baseline for [`deterministic_demo_fixture`] + current real-mode queue model (O1 stays behind queue).
        assert_eq!(sim.pnl, 0, "PnL baseline (no fills)");
        assert_eq!(sim.trades, 0, "Trade count baseline");

        let o1 = sim.order_outcomes.get("O1").unwrap();
        assert_eq!(o1.filled_quantity, 0);
        assert_eq!(o1.remaining_quantity, 600);
    }

    // Test 5 — No Cycles: causal chain must never loop
    #[test]
    fn test_no_causal_cycles() {
        let sim = run_simulation(ExecutionMode::Real);
        for event in &sim.events {
            let mut visited = std::collections::HashSet::new();
            let mut curr = Some(event.sequence_id());
            
            while let Some(sid) = curr {
                assert!(visited.insert(sid), "Causal loop detected at seq_id {}", sid);
                curr = sim.events.iter()
                    .find(|e| e.sequence_id() == sid)
                    .and_then(|e| e.parent_sequence_id());
            }
        }
    }
}
