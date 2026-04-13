use chronosentiment_core::folder_source::FolderCandleSource;
use chronosentiment_core::ga::{run_ga_evolution, GaConfig, PortfolioCluster, SignalAlphaReport};
use chronosentiment_core::pipeline;
use chrono::NaiveDateTime;
use serde::Serialize;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

#[derive(Serialize)]
struct JsonSignal {
    idx: usize,
    score: f64,
    label: String,
}

#[derive(Serialize)]
struct JsonCluster {
    label: String,
    archetype: u8,
    center: f64,
    size: usize,
    weight: f64,
    signals: Vec<JsonSignal>,
    start_idx: usize,
    end_idx: usize,
}

#[derive(Serialize)]
struct Snapshot {
    timestamp: u64,
    price: f64,
    clusters: Vec<JsonCluster>,
}

fn main() {
    println!("🚀 Portfolio Validation Harness");

    // 1. Data load
    let source = FolderCandleSource { folder_path: "data/nse/5m".to_string() };
    let raw = source.load_all_flexible();
    
    // Fallback logic
    let target_symbol = "INFY.NS";
    let candles = match raw.iter().find(|(s, _)| s == target_symbol) {
        Some((_, c)) => c,
        None => {
            eprintln!("❌ Target data not found: {}", target_symbol);
            return;
        }
    };
    
    if candles.is_empty() {
        eprintln!("❌ Candles array is empty");
        return;
    }
    
    println!("✅ Loaded {} candles for {}", candles.len(), target_symbol);

    let signal_map = pipeline::scenario_map_for_signal_generation(
        target_symbol,
        "folder",
        Some(candles),
        "",
    );
    let scenarios = pipeline::pair_scenarios_by_index(
        target_symbol,
        target_symbol,
        &signal_map,
        &signal_map,
    );

    if scenarios.is_empty() {
        eprintln!("❌ Scenarios were empty");
        return;
    }

    // 2. config
    let mut config = GaConfig::default();
    config.generations = 50;
    config.population_size = 50;

    println!("🔥 Running Portfolio Engine Pipeline over {} scenarios...", scenarios.len());
    let ga_result = run_ga_evolution(config, &scenarios);

    let mut portfolios_by_idx: HashMap<usize, Vec<JsonCluster>> = HashMap::new();

    if let Some(mut rep) = ga_result.consensus_recommendations {
        println!("  Extracted {} global portfolio clusters.", rep.portfolio_clusters.len());
        for c in rep.portfolio_clusters {
            if c.signals.is_empty() { continue; }
            let min_idx = c.signals.iter().map(|s| s.signal_idx).min().unwrap_or(0);
            let max_idx = c.signals.iter().map(|s| s.signal_idx).max().unwrap_or(0);
            let center_idx = c.signals.iter().map(|s| s.signal_idx as f64).sum::<f64>() / c.signals.len() as f64;
            
            // Just bind the cluster visual directly to its center index
            let entry = portfolios_by_idx.entry(center_idx.round() as usize).or_insert(Vec::new());
            entry.push(JsonCluster {
                label: c.label.to_string(),
                archetype: match c.archetype {
                    chronosentiment_core::ga::Archetype::Conviction => 0,
                    chronosentiment_core::ga::Archetype::Momentum => 1,
                    chronosentiment_core::ga::Archetype::Reversion => 2,
                    chronosentiment_core::ga::Archetype::Volatility => 3,
                },
                center: center_idx,
                size: c.signals.len(),
                weight: c.total_weight,
                signals: c.signals.iter().map(|s| JsonSignal {
                    idx: s.signal_idx,
                    score: s.alpha_score,
                    label: s.consensus_label.clone()
                }).collect(),
                start_idx: min_idx,
                end_idx: max_idx,
            });
        }
    } else {
        println!("⚠️ No consensus/portfolio report generated from GA run.");
    }

    // output construction
    let mut snapshots = Vec::new();
    for (i, candle) in candles.iter().enumerate() {
        snapshots.push(Snapshot {
            timestamp: candle.timestamp as u64,
            price: candle.close as f64,
            clusters: portfolios_by_idx.remove(&i).unwrap_or(Vec::new()),
        });
    }

    let file = File::create("clusters.json").unwrap();
    serde_json::to_writer_pretty(file, &snapshots).unwrap();
    println!("✅ Exported to clusters.json");
}
