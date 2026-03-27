use chronosentiment_core::folder_source::FolderCandleSource;
use chronosentiment_core::pipeline;
use chronosentiment_core::PRICE_SCALE;
use std::path::PathBuf;
use std::cmp::Ordering;
use std::collections::HashMap;

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

#[derive(Clone)]
struct TemplateRecommendation {
    asset: String,
    action: String,
    entry_lo: f64,
    entry_hi: f64,
    stop_loss: f64,
    target: f64,
    confirmations: usize,
    best_confidence: f64,
    best_score: f64,
    best_edge: f64,
    best_position_size: f64,
}

fn template_key(signal: &chronosentiment_core::pipeline::TradeSignal) -> Option<String> {
    let (entry_lo, entry_hi) = signal.entry_zone?;
    let sl = signal.stop_loss?;
    let tp = signal.target?;
    Some(format!(
        "{}|{:?}|{:.2}|{:.2}|{:.2}|{:.2}",
        signal.asset, signal.action, entry_lo, entry_hi, sl, tp
    ))
}

fn dedupe_recommendations(
    signals: &[chronosentiment_core::pipeline::TradeSignal],
) -> Vec<TemplateRecommendation> {
    let mut grouped: HashMap<String, TemplateRecommendation> = HashMap::new();
    for s in signals {
        if !matches!(s.action, chronosentiment_core::pipeline::SignalAction::BUY | chronosentiment_core::pipeline::SignalAction::SELL) {
            continue;
        }
        let Some(k) = template_key(s) else { continue };
        let (entry_lo, entry_hi) = s.entry_zone.unwrap_or((0.0, 0.0));
        let sl = s.stop_loss.unwrap_or(0.0);
        let tp = s.target.unwrap_or(0.0);
        let action = format!("{:?}", s.action);
        let score = s.composite_score;
        let edge = s.expected_edge;
        grouped
            .entry(k)
            .and_modify(|row| {
                row.confirmations += 1;
                if score > row.best_score
                    || ((score - row.best_score).abs() <= 1e-12 && edge > row.best_edge)
                {
                    row.best_score = score;
                    row.best_edge = edge;
                    row.best_confidence = s.confidence;
                    row.best_position_size = s.position_size;
                }
            })
            .or_insert(TemplateRecommendation {
                asset: s.asset.clone(),
                action,
                entry_lo,
                entry_hi,
                stop_loss: sl,
                target: tp,
                confirmations: 1,
                best_confidence: s.confidence,
                best_score: score,
                best_edge: edge,
                best_position_size: s.position_size,
            });
    }
    let mut out: Vec<TemplateRecommendation> = grouped.into_values().collect();
    out.sort_by(|a, b| {
        b.best_score
            .partial_cmp(&a.best_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.confirmations.cmp(&a.confirmations))
            .then_with(|| {
                b.best_edge
                    .partial_cmp(&a.best_edge)
                    .unwrap_or(Ordering::Equal)
            })
    });
    out
}

fn close_within_bps(a: f64, b: f64, bps: f64) -> bool {
    let den = a.abs().max(b.abs()).max(1e-9);
    ((a - b).abs() / den) <= (bps / 10_000.0)
}

fn cluster_recommendations(
    rows: Vec<TemplateRecommendation>,
    cluster_bps: f64,
) -> Vec<TemplateRecommendation> {
    let mut clusters: Vec<TemplateRecommendation> = Vec::new();
    for row in rows {
        let mut merged = false;
        for c in &mut clusters {
            if c.asset != row.asset || c.action != row.action {
                continue;
            }
            let similar = close_within_bps(c.entry_lo, row.entry_lo, cluster_bps)
                && close_within_bps(c.entry_hi, row.entry_hi, cluster_bps)
                && close_within_bps(c.stop_loss, row.stop_loss, cluster_bps)
                && close_within_bps(c.target, row.target, cluster_bps);
            if similar {
                c.confirmations += row.confirmations;
                c.entry_lo = (c.entry_lo + row.entry_lo) * 0.5;
                c.entry_hi = (c.entry_hi + row.entry_hi) * 0.5;
                c.stop_loss = (c.stop_loss + row.stop_loss) * 0.5;
                c.target = (c.target + row.target) * 0.5;
                if row.best_score > c.best_score
                    || ((row.best_score - c.best_score).abs() <= 1e-12 && row.best_edge > c.best_edge)
                {
                    c.best_score = row.best_score;
                    c.best_edge = row.best_edge;
                    c.best_confidence = row.best_confidence;
                    c.best_position_size = row.best_position_size;
                }
                merged = true;
                break;
            }
        }
        if !merged {
            clusters.push(row);
        }
    }
    clusters.sort_by(|a, b| {
        b.best_score
            .partial_cmp(&a.best_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| b.confirmations.cmp(&a.confirmations))
            .then_with(|| {
                b.best_edge
                    .partial_cmp(&a.best_edge)
                    .unwrap_or(Ordering::Equal)
            })
    });
    clusters
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
                let deduped = dedupe_recommendations(&snapshot.signals);
                let cluster_bps = std::env::var("TEMPLATE_CLUSTER_BPS")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(20.0);
                let clustered = cluster_recommendations(deduped.clone(), cluster_bps);
                println!(
                    "Deduped templates: {} (from {} executable signals)",
                    deduped.len(),
                    snapshot.meta.trades
                );
                println!(
                    "Clustered templates: {} (tolerance {:.1} bps)",
                    clustered.len(),
                    cluster_bps
                );
                for (idx, r) in clustered.iter().take(10).enumerate() {
                    let scale = PRICE_SCALE as f64;
                    println!(
                        "{}. {} {} | entry {:.2}-{:.2} | sl {:.2} | target {:.2} | confs {} | score {:.3} | edge {:.6} | size {:.2} | conf {:.3}",
                        idx + 1,
                        r.asset,
                        r.action,
                        r.entry_lo / scale,
                        r.entry_hi / scale,
                        r.stop_loss / scale,
                        r.target / scale,
                        r.confirmations,
                        r.best_score,
                        r.best_edge,
                        r.best_position_size,
                        r.best_confidence
                    );
                }
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
        None,
        None,
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
