// tests/demo_test.rs

// This file contains tests for the MVP Demonstration Layer.
// These tests verify the core objectives:
// 1. Execution constraints change outcomes.
// 2. Execution constraints change GA evolution.

#[cfg(test)]
mod demo_tests {
    use chronosentiment_mvp_demo::{
        ExecutionMode,
        SimulationResult,
        GAResult,
        OrderOutcome,
        run_simulation,
        run_ga,
    };

    // Test 1 — Determinism: same seed → identical output
    // This test will require a way to set a seed for the simulation/GA
    // For now, we'll assume `run_simulation` and `run_ga` are deterministic.
    #[test]
    fn test_determinism() {
        // Assuming a mechanism to set a seed or that the system is inherently deterministic
        // with the same inputs. Since we're not modifying the core engine, this should hold.
        let result1_sim = run_simulation(ExecutionMode::Real);
        let result2_sim = run_simulation(ExecutionMode::Real);
        assert_eq!(result1_sim.pnl, result2_sim.pnl);
        assert_eq!(result1_sim.trades, result2_sim.trades);

        let result1_ga = run_ga(ExecutionMode::Real);
        let result2_ga = run_ga(ExecutionMode::Real);
        assert_eq!(result1_ga.best_config, result2_ga.best_config);
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

    // Test 5 — GA Divergence: best_config_ideal != best_config_real
    #[test]
    fn test_ga_divergence() {
        let ideal_ga_result = run_ga(ExecutionMode::Ideal);
        let real_ga_result = run_ga(ExecutionMode::Real);
        assert_ne!(ideal_ga_result.best_config, real_ga_result.best_config);
    }
}
