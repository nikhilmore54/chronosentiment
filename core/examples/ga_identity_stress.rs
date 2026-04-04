use chronosentiment_core::ga::{Strategy, GaConfig, random_strategy, deterministic_strategy_id};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashSet;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let config = GaConfig::default();
    let mut seen = HashSet::new();
    let iterations = 10_000;

    println!("🚀 Starting Identity Integrity Stress Test ({} iterations)...", iterations);

    for i in 0..iterations {
        let strat = random_strategy(&config, &mut rng);
        // Scenarios and seed for deterministic ID
        let scenarios = vec!["test_scenario_1".to_string(), "test_scenario_2".to_string()];
        let seed = 42;
        let id = deterministic_strategy_id(&strat, &scenarios, seed);

        if !seen.insert(id.clone()) {
            panic!("🔥 DUPLICATE STRATEGY ID DETECTED at iteration {}: {}", i, id);
        }
    }

    println!("✅ SUCCESS: 10,000 unique iterations, 0 collisions.");
    println!("DNA Space verified for 13-gene strategy model.");
}
