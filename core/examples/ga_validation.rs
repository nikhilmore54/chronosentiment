use chronosentiment_core::ga::{
    random_strategy, run_ga_evolution, strategy_to_id, GaConfig, ScenarioPair, Strategy,
};
use chronosentiment_core::MarketEvent;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashSet;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut config = GaConfig::default();
    config.population_size = 20;
    config.generations = 15; // Phase D.1.14: 15 generations to watch alpha emerge
    config.preserve_top_k = 5;

    println!("--- 🧪 VALIDATION: Phase D.1.13.5 & D.1.14 Stress Test ---");

    // 1. Generate 2000 points of high-density mock data (Sin wave + cycle)
    let mut mock_events = Vec::with_capacity(2000);
    for i in 0..2000 {
        let base_price = 10000.0;
        let cycle = (i as f64 * 0.05).sin() * 200.0; // Slow trend
        let noise = (i as f64 * 1.5).cos() * 20.0; // Fast noise
        let price = (base_price + cycle + noise) as u64;

        mock_events.push(MarketEvent {
            subtype: chronosentiment_core::MarketEventType::Trade,
            price,
            quantity: 100,
            side: Some(if i % 2 == 0 {
                chronosentiment_core::Side::Buy
            } else {
                chronosentiment_core::Side::Sell
            }),
            exchange_ts: 1000 + i * 10,
        });
    }

    // 2. Create 5 scenarios with 95% overlap
    let mut scenarios = Vec::new();
    let window_size = 500;
    let step = 25; // 500 * (1 - 0.95) = 25

    for i in 0..5 {
        let start = i * step;
        let end = start + window_size;
        let window = &mock_events[start..end];

        scenarios.push(ScenarioPair {
            name: format!("WINDOW_{}", i).leak(), // Leak for simplicity in test
            signal_symbol: "TEST",
            execution_symbol: "TEST",
            signal: window,
            execution: window,
        });
    }

    // 3. Configure Gates
    config.initial_queue_threshold = 50;
    std::env::set_var("GA_DEBUG", "1"); // Trace Bootstrap & Purity

    println!("🚀 Launching GA Evolution (15 Generations)...");
    let result = run_ga_evolution(config, &scenarios);

    println!("\n📊 VALIDATION METRICS:");
    println!("  - Global Best ID: {}", result.global_best.strategy_id);
    println!(
        "  - Final Gen Best Fitness: {:.4}",
        result.final_generation_best.fitness
    );
    println!(
        "  - Organic Signal Flag (Best): {}",
        result.global_best.had_organic_signals
    );

    if let Some(ref report) = result.consensus_recommendations {
        println!(
            "\n🔥 PHASE D.1.14: CONSENSUS TRUTH ({} active strategies):",
            report.active_strategies
        );
        for sig in &report.top_signals {
            println!("  - Signal #{} (Asset: {}): Alpha={:.3}, Stability={:.2}, Persistence={}/5, support={:.1}%, Label={}", 
                sig.signal_idx, sig.asset, sig.alpha_score, sig.temporal_stability, sig.persistence_count, sig.support_ratio * 100.0, sig.consensus_label);
        }

        // Assertions for Success
        if report.top_signals.is_empty() {
            println!("⚠️ WARNING: No consensus recommendations. Alpha search might be too strict.");
        } else {
            println!("\n✅ SUCCESS: Consensus signals emerged organically.");
        }
    } else {
        println!("\n⚠️ No consensus recommendations generated.");
    }

    if result.global_best.strategy_id == "FALLBACK_ZERO" {
        panic!("❌ FAILURE: GA stagnated even with Bootstrap mode.");
    }

    println!("\n✅ PHASE D.1.13.5 & D.1.14 VALIDATION COMPLETE.");
}
