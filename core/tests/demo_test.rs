// tests/demo_test.rs

// This file contains tests for the MVP Demonstration Layer.
// These tests verify the core objectives:
// 1. Execution constraints change outcomes.
// 2. Execution constraints change GA evolution.

#[cfg(test)]
mod demo_tests {
    use chronosentiment_core::{run_simulation, ExecutionMode};

    // Test 1 — Determinism: same seed → identical output
    // NOTE: run_ga stub (kernel.rs) removed in Phase 2 authority consolidation.
    // GA determinism is now covered by evaluation_service::test_run_ga_api_determinism.
    #[test]
    fn test_determinism() {
        let result1_sim = run_simulation(ExecutionMode::Real);
        let result2_sim = run_simulation(ExecutionMode::Real);
        assert_eq!(result1_sim.pnl, result2_sim.pnl);
        assert_eq!(result1_sim.trades, result2_sim.trades);
    }

    // Test 2 — Mode Difference: ideal_result != real_result
    #[test]
    fn test_mode_difference() {
        let ideal_result = run_simulation(ExecutionMode::Ideal);
        let real_result = run_simulation(ExecutionMode::Real);

        // We expect PnL and trades to be different due to execution constraints
        assert_ne!(ideal_result.pnl, real_result.pnl);
        assert_ne!(ideal_result.trades, real_result.trades);
    }

    // Test 3 — Partial Fill: O1 must NOT fully fill in real mode
    #[test]
    fn test_partial_fill_real_mode() {
        let real_result = run_simulation(ExecutionMode::Real);
        // O1 quantity is 600
        if let Some(o1_outcome) = real_result.order_outcomes.get("O1") {
            assert_ne!(o1_outcome.filled_quantity, 600);
            assert_ne!(o1_outcome.remaining_quantity, 0);
            assert!(o1_outcome.filled_quantity < 600);
        } else {
            panic!("Order O1 not found in real mode outcomes");
        }
    }

    // Test 4 — Full Fill in Ideal: O1 must fully fill in ideal mode
    #[test]
    fn test_full_fill_ideal_mode() {
        let ideal_result = run_simulation(ExecutionMode::Ideal);
        // O1 quantity is 600
        if let Some(o1_outcome) = ideal_result.order_outcomes.get("O1") {
            assert_eq!(o1_outcome.filled_quantity, 600);
            assert_eq!(o1_outcome.remaining_quantity, 0);
        } else {
            panic!("Order O1 not found in ideal mode outcomes");
        }
    }

    // Test 5 — GA Divergence: removed in Phase 2 authority consolidation.
    // kernel::run_ga() was a stub returning hardcoded strings, not the real GA.
    // Real GA mode-divergence is exercised via evaluation_service and pipeline tests.
}
