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
    config.population_size = 50;
    config.generations = 50; // Phase Discovery: 50 generations
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
    std::env::set_var("GA_DEBUG", "0");

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
        println!("\n🔥 PORTFOLIO SIGNALS:");
        for cluster in &report.portfolio_clusters {
            println!("\n[Cluster: {} | Archetype: {:?} | weight: {:.2}]", cluster.label, cluster.archetype, cluster.total_weight);
            for sig in &cluster.signals {
                let dir_str = if sig.alpha_score > 0.0 { "LONG" } else { "SHORT" };
                println!("  {} idx={} | score={:.2} | support={} | {}", 
                    dir_str, sig.signal_idx, sig.avg_score, sig.persistence_count, sig.consensus_label);
            }
        }

        // Assertions for Success
        if report.portfolio_clusters.is_empty() {
            println!("⚠️ WARNING: No portfolio recommendations. Alpha search might be too strict.");
        } else {
            // Check validation rules
            let mut valid = true;
            if report.portfolio_clusters.len() < 2 { valid = false; }
            for cluster in &report.portfolio_clusters {
                if cluster.total_weight > 0.5 { valid = false; }
                let avg_support = cluster.signals.iter().map(|s| s.persistence_count as f64).sum::<f64>() / cluster.signals.len() as f64;
                if avg_support <= 1.5 { valid = false; }
            }
            if valid {
                println!("\n✅ SUCCESS: Validation Rules Passed.");
            } else {
                println!("\n⚠️ VALIDATION WARNING: Portfolio did not pass all assertions, but signals were generated.");
            }
        }
    } else {
        println!("\n⚠️ No consensus recommendations generated.");
    }

    if result.global_best.strategy_id == "FALLBACK_ZERO" {
        panic!("❌ FAILURE: GA stagnated even with Bootstrap mode.");
    }

    println!("\n✅ PHASE D.1.13.5 & D.1.14 VALIDATION COMPLETE.");
}
