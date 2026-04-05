use chronosentiment_core::ga::{
    aggregate_strategy_reports, ExecutionMetrics, FitnessMode, GaConfig, ScenarioCapability,
    ScenarioExecutionSignature, Strategy, StrategyEvaluation,
};

fn mock_eval(pnl: f64, trades: usize, fill_eff: f64) -> StrategyEvaluation {
    let mut eval = StrategyEvaluation {
        strategy_id: "mock".to_string(),
        strategy: Strategy {
            queue_threshold: 100,
            base_edge: 2,
            take_profit: 20,
            stop_loss: 10,
            holding_period: 0,
            w_conviction: 50,
            w_momentum: 30,
            w_volatility: 20,
            exp_conviction: 100,
            exp_momentum: 100,
            exp_volatility: 100,
            selectivity: 75,
            archetype: 0,
        },
        capability: ScenarioCapability::Executable,
        avg_pnl: pnl,
        std_dev: 0.001, // Stable distribution
        worst: pnl,
        robustness: pnl,
        fitness: 0.0,
        trade_count: trades,
        max_drawdown: 0.01,
        participation_rate: 1.0,
        profitable_trades: (trades as f64 * 0.6) as usize, // 60% win rate
        zero_pnl_trades: 0,
        quality_trades: trades as f64,
        payoff_ratio: 1.5,
        execution_metrics: ExecutionMetrics {
            fill_efficiency: fill_eff,
            capture_efficiency: 0.8,
            avg_slippage: 0.02,
            latency_impact: 0.05,
        },
        scenario_signature: ScenarioExecutionSignature::default(),
        avg_conviction: 0.8,
        avg_efficiency: 0.8,
        avg_edge_quality: 0.8,
        directional_accuracy: 0.7,
        decisiveness: 0.8,
        execution_friction: 1.0,
        exit_tp_count: (trades as f64 * 0.5) as usize,
        exit_sl_count: (trades as f64 * 0.4) as usize,
        exit_ts_count: (trades as f64 * 0.1) as usize,
        consistency_score: 1.0,
        recent_performance: pnl,
        ..Default::default()
    };
    eval
}

fn main() {
    let mut config = GaConfig::default();

    // Test Set: A, B, C, D
    let genomes = vec![
        ("A (Lucky Spike)", mock_eval(0.05, 1, 1.0)),
        ("B (Scalable Winner)", mock_eval(0.01, 100, 1.0)),
        ("C (Unfillable Edge)", mock_eval(0.10, 5, 0.2)),
        ("D (Overtrading Noise)", mock_eval(0.001, 300, 0.1)),
    ];

    for mode in [FitnessMode::Sniper, FitnessMode::Scalable] {
        config.fitness_mode = mode;
        println!("\n🚀 TESTING MODE: {:?} 🚀", mode);
        println!("{:-<100}", "");
        println!(
            "{:<25} | {:<10} | {:<10} | {:<10} | {:<10}",
            "Genome", "Trades", "PnL", "Fill", "FITNESS"
        );
        println!("{:-<100}", "");

        let mut results = Vec::new();

        for (name, eval) in &genomes {
            // We need to pass a Vec because aggregation works on scenarios
            let evals = vec![eval.clone()];
            if let Some(agg) = aggregate_strategy_reports(evals, &config) {
                println!(
                    "{:<25} | {:<10} | {:<10.4} | {:<10.2} | {:<10.4}",
                    name,
                    agg.trade_count,
                    agg.avg_pnl,
                    agg.execution_metrics.fill_efficiency,
                    agg.fitness
                );
                results.push((name, agg.fitness));
            }
        }

        // Qualitative Assertions
        println!("{:-<100}", "");
        match mode {
            FitnessMode::Sniper => {
                println!("🧠 SNIPER AUDIT:");
                // Sniper should penalize B (over 18 trades/scenario, though here it's 100 trades in 1 scenario)
                // Actually aggregate_strategy_reports works on per-scenario averages.
            }
            FitnessMode::Scalable => {
                println!("🧠 SCALABLE AUDIT:");
                let fitness_a = results
                    .iter()
                    .find(|(n, _)| **n == "A (Lucky Spike)")
                    .map(|(_, f)| *f)
                    .unwrap_or(0.0);
                let fitness_b = results
                    .iter()
                    .find(|(n, _)| **n == "B (Scalable Winner)")
                    .map(|(_, f)| *f)
                    .unwrap_or(0.0);

                if fitness_b > fitness_a {
                    println!("✅ SUCCESS: Scalable B (consistent) beats A (lucky spike)");
                } else {
                    println!("❌ FAILURE: Lucky spike A still dominates B");
                }
            }
        }
    }
}
