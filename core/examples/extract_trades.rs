use chronosentiment_core::pipeline::{generate_latest_signals_with_thresholds, SignalAction};
use std::env;

fn main() {
    let assets = vec!["TATAMOTORS".to_string(), "RELIANCE".to_string()];
    let conf = env::var("CONFIDENCE_FLOOR")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let score = env::var("SCORE_FLOOR")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let snapshot = generate_latest_signals_with_thresholds(assets.clone(), 0.5, conf, score);

    let mut trades: Vec<_> = snapshot
        .signals
        .into_iter()
        .filter(|s| matches!(s.action, SignalAction::BUY | SignalAction::SELL))
        .collect();

    trades.sort_by(|a, b| {
        a.asset.cmp(&b.asset).then_with(|| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    println!("assets={:?}", assets);
    println!("confidence_floor={:?}", conf);
    println!("score_floor={:?}", score);
    println!("timestamp={}", snapshot.timestamp);
    println!("trade_count={}", trades.len());
    println!("asset,action,regime,confidence,composite_score,expected_edge,position_size,reason,strategy_id");
    for t in trades {
        let action = match t.action {
            SignalAction::BUY => "BUY",
            SignalAction::SELL => "SELL",
            SignalAction::HOLD => "HOLD",
        };
        println!(
            "{},{},{},{:.4},{:.6},{:.6},{:.4},\"{}\",{}",
            t.asset,
            action,
            t.regime,
            t.confidence,
            t.composite_score,
            t.expected_edge,
            t.position_size,
            t.reason.replace('\"', "'"),
            t.strategy_id
        );
    }
}
