use chronosentiment_core::pipeline;

fn main() {
    let assets = vec![
        ("BTC".to_string(), "test_assets/btc_ohlc.csv".to_string()),
        ("BANKNIFTY".to_string(), "test_assets/BANKNIFTY_5m_Execution_Ready.csv".to_string()),
    ];
    let global_lambda = 0.5;

    println!("Starting real-data GA evaluation pipeline...");
    let ranking = pipeline::evaluate_on_real_data(assets, global_lambda);

    println!("\nPipeline completed successfully.");
    println!("Total metric rows produced: {}", ranking.len());
    for entry in ranking {
        println!(
            "{}: mean={:.6}, std={:.6}, min={:.6}, max={:.6}",
            entry.metric, entry.mean, entry.std_dev, entry.min, entry.max
        );
    }

    println!("\nDeterministic threshold sweep (confidence floor x score floor):");
    let sweep = pipeline::run_threshold_sweep(
        vec!["BTC".to_string(), "BANKNIFTY".to_string()],
        global_lambda,
        &[0.30, 0.35, 0.40, 0.45, 0.50],
        &[0.35, 0.40, 0.45, 0.50, 0.55],
    );
    println!("conf_floor | score_floor | participation | trades | total | global_avg | traded_avg | std");
    for row in sweep.iter().take(9) {
        println!(
            "{:.2} | {:.2} | {:.2} | {} | {} | {:.6} | {:.6} | {:.6}",
            row.confidence_floor,
            row.score_floor,
            row.participation,
            row.trades,
            row.total_scenarios,
            row.global_avg_pnl,
            row.traded_avg_pnl,
            row.std_dev,
        );
    }
}
