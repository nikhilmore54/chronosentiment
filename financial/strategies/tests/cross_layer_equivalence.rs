use chronosentiment_optimization::{Candidate, CandidateEvaluation};
use chronosentiment_strategies::evaluation::report_adapter::SemanticEvaluationReport;
use std::collections::HashMap;

#[test]
fn semantic_projection_is_lossless() {
    let mut eval = CandidateEvaluation::default();
    eval.total_pnl = 150.0;
    eval.win_rate = 0.55;
    eval.trade_count = 10;
    eval.max_drawdown = 0.1;
    eval.fitness = 0.85;
    eval.avg_pnl = 15.0;

    let mut report: SemanticEvaluationReport = eval.into();
    report.regime = Some("bull_trending".to_string());
    report.classification = "momentum_capture".to_string();

    assert_eq!(report.fitness, 0.85);
    assert_eq!(report.avg_pnl, 15.0);
    assert_eq!(report.metrics.get("total_pnl").unwrap(), &150.0);
    assert_eq!(report.metrics.get("win_rate").unwrap(), &0.55);
    assert_eq!(report.classification, "momentum_capture");
    assert_eq!(report.regime.unwrap(), "bull_trending");
}

#[test]
fn aggregation_output_hash_is_stable() {
    use chronosentiment_core::market_adapter::Candle;
    use chronosentiment_strategies::pipeline::aggregation::scenarios_from_candles;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut candles = Vec::new();
    for i in 0..150 {
        candles.push(Candle {
            open: 1000 + i * 10,
            high: 1010 + i * 10,
            low: 990 + i * 10,
            close: 1005 + i * 10,
            volume: 1000,
            timestamp: i as u64 * 60000,
        });
    }

    let scenarios = scenarios_from_candles("BTCUSDT", &candles);

    let mut hasher = DefaultHasher::new();
    let mut keys: Vec<&String> = scenarios.keys().collect();
    keys.sort();

    for key in keys {
        key.hash(&mut hasher);
        let events = scenarios.get(key).unwrap();
        for event in events {
            let s = format!("{:?}", event);
            s.hash(&mut hasher);
        }
    }

    let hash = hasher.finish();
    // Pre-calculated hash from the old.rs implementation
    assert_eq!(
        hash, 1355411665765150301,
        "Hash diverged! Semantic aggregation logic has shifted."
    );
}

#[test]
fn pipeline_output_is_seed_stable() {
    assert!(
        true,
        "Pending implementation: Ensure end-to-end pipeline outputs are stable"
    );
}
