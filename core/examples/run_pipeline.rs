use chronosentiment_core::folder_source::FolderCandleSource;
use chronosentiment_core::pipeline;
use std::path::PathBuf;

fn test_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("chronosentiment_core must live under workspace root")
        .join("test_assets")
}

fn folder_sweep_assets(folder_path: String) -> Vec<String> {
    let source = FolderCandleSource { folder_path };
    let mut names: Vec<String> = source
        .load_all()
        .into_iter()
        .map(|(asset, _)| asset)
        .collect();
    names.sort();
    names.dedup();
    names
}

fn main() {
    let assets = vec![
        ("BTC".to_string(), "test_assets/btc_ohlc.csv".to_string()),
        ("BANKNIFTY".to_string(), "test_assets/BANKNIFTY_5m_Execution_Ready.csv".to_string()),
    ];
    let global_lambda = 0.5;

    let data_source = std::env::var("DATA_SOURCE")
        .unwrap_or_else(|_| "folder".to_string())
        .to_lowercase();

    let test_assets_path = test_assets_dir();
    let test_assets_str = test_assets_path.to_string_lossy().into_owned();

    let sweep_assets: Vec<String> = if data_source == "folder" {
        let names = folder_sweep_assets(test_assets_str.clone());
        println!(
            "\n>>> FOLDER MODE: dataset_count={} scrips={:?}\n>>> (full GA + sweep use these symbols; scroll up for per-asset blocks)\n",
            names.len(),
            names
        );
        names
    } else {
        vec!["BTC".to_string(), "BANKNIFTY".to_string()]
    };

    let run_mode = std::env::var("RUN_MODE")
        .unwrap_or_else(|_| "full".to_string())
        .to_lowercase();
    let strategy_store_path = std::env::var("STRATEGY_STORE_PATH")
        .unwrap_or_else(|_| format!("{}/strategy_store.json", test_assets_str));

    if run_mode == "train" {
        println!(
            "RUN_MODE=train -> persisting strategy store at {}",
            strategy_store_path
        );
        match pipeline::train_and_persist_strategies(
            sweep_assets.clone(),
            global_lambda,
            Some(strategy_store_path),
        ) {
            Ok(count) => println!("Saved strategies for {} assets", count),
            Err(err) => eprintln!("Failed to persist strategy store: {}", err),
        }
        return;
    }

    if run_mode == "recommend" {
        println!(
            "RUN_MODE=recommend -> loading strategy store from {}",
            strategy_store_path
        );
        match pipeline::generate_latest_signals_from_saved_strategies(
            sweep_assets.clone(),
            global_lambda,
            0.45,
            0.35,
            Some(strategy_store_path),
        ) {
            Ok(snapshot) => {
                println!(
                    "Fast recommendation snapshot: trades={} total={} participation={:.2}",
                    snapshot.meta.trades,
                    snapshot.meta.total_scenarios,
                    snapshot.meta.participation
                );
            }
            Err(err) => eprintln!("Failed to generate fast recommendations: {}", err),
        }
        return;
    }

    println!("Starting real-data GA evaluation pipeline... DATA_SOURCE={}", data_source);
    let ranking = pipeline::evaluate_on_real_data(assets, global_lambda);

    println!("\n=== METRICS BY SCRIPT (PnL_Asset_*) ===");
    for entry in &ranking {
        println!(
            "{}: mean={:.6}, std={:.6}, min={:.6}, max={:.6}",
            entry.metric, entry.mean, entry.std_dev, entry.min, entry.max
        );
    }

    println!("\nPipeline completed successfully.");
    println!("Total metric rows produced: {}", ranking.len());

    println!("\nDeterministic threshold sweep (confidence floor x score floor):");
    println!("sweep_assets={:?}", sweep_assets);
    let sweep = pipeline::run_threshold_sweep(
        sweep_assets,
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
