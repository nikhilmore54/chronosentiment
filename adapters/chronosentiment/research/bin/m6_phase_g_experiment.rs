use chrono::{TimeZone, Utc};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::env;
use uuid::Uuid;

use chronosentiment_adapter::reasoning::strategy::Horizon;
use chronosentiment_adapter::research::dataset::{ArtifactPopulation, DateRange, ResearchDataset};
use chronosentiment_adapter::research::experiment::ResearchExperiment;
use chronosentiment_adapter::research::predictive_value::PredictiveValueExperiment;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://nikhil@localhost:5432/chronosentiment".to_string());
    println!("Connecting to database: {}", db_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Connected! Executing PredictiveValueExperiment...");

    let experiment = PredictiveValueExperiment::new(pool.clone());

    let dataset = ResearchDataset::new(
        "Production Phase G Dataset".to_string(),
        "v1.0".to_string(),
        serde_json::json!("Nifty50"),
        DateRange {
            start: Utc.with_ymd_and_hms(2010, 1, 1, 0, 0, 0).unwrap(),
            end: Utc.with_ymd_and_hms(2025, 12, 31, 23, 59, 59).unwrap(),
        },
        vec![
            Horizon::Intraday,
            Horizon::Swing,
            Horizon::Position,
            Horizon::Investment,
            Horizon::Strategic,
        ],
        serde_json::json!([]),
        serde_json::json!([]),
        ArtifactPopulation {
            artifact_types: vec!["Outcome".to_string()],
            population_rules: serde_json::json!({}),
        },
    );

    let measurements = experiment.execute(&dataset).await?;

    println!("\n=== POPULATION ACCOUNTING ===");
    println!("| Signature | 5D N | 10D N | 20D N | 60D N | 5D Entry | 10D Entry | 20D Entry | 60D Entry |");
    println!("|-----------|-----:|------:|------:|------:|---------:|----------:|----------:|----------:|");

    let pop_accounting = &measurements.findings[2]["data"];
    if let Some(arr) = pop_accounting.as_array() {
        for row in arr {
            println!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                row["signature"].as_str().unwrap_or(""),
                row["5D_N"],
                row["10D_N"],
                row["20D_N"],
                row["60D_N"],
                row["5D_Entry"],
                row["10D_Entry"],
                row["20D_Entry"],
                row["60D_Entry"]
            );
        }
    }

    println!("\n=== AGGREGATE MATRIX ===");
    println!("| Assessment Signature | Horizon | N | Entry % | Target % | Stop % | Mean Return | Median Return | Median MFE | Median MAE | Median DD |");
    println!("|----------------------|--------:|--:|--------:|---------:|-------:|------------:|--------------:|-----------:|-----------:|----------:|");

    let aggregate_matrix = &measurements.findings[0]["data"];
    if let Some(arr) = aggregate_matrix.as_array() {
        for row in arr {
            println!("| {} | {} | {} | {:.2}% | {:.2}% | {:.2}% | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |", 
                row["signature"].as_str().unwrap_or(""),
                row["horizon"].as_str().unwrap_or(""),
                row["N"],
                row["entry_pct"].as_f64().unwrap_or(0.0) * 100.0,
                row["target_pct"].as_f64().unwrap_or(0.0) * 100.0,
                row["stop_pct"].as_f64().unwrap_or(0.0) * 100.0,
                row["mean_return"].as_f64().unwrap_or(0.0),
                row["median_return"].as_f64().unwrap_or(0.0),
                row["median_mfe"].as_f64().unwrap_or(0.0),
                row["median_mae"].as_f64().unwrap_or(0.0),
                row["median_drawdown"].as_f64().unwrap_or(0.0)
            );
        }
    }

    println!("\n=== RAW EVIDENCE LEDGER (First 5 records) ===");
    let raw_ledger = &measurements.findings[1]["data"];
    if let Some(arr) = raw_ledger.as_array() {
        let count = std::cmp::min(5, arr.len());
        for i in 0..count {
            let r = &arr[i];
            println!("Record {}:", i + 1);
            println!("  Assessment ID: {}", r["assessment_id"]);
            println!("  Decision ID:   {}", r["decision_id"]);
            println!("  Strategy ID:   {}", r["strategy_id"]);
            println!("  Outcome ID:    {}", r["outcome_id"]);
            println!("  Signature:     {}", r["signature"]);
            println!("  Horizon:       {}", r["horizon"]);
            println!("  Exit Reason:   {}", r["exit_reason"]);
            println!(
                "  Return:        {:.4}",
                r["outcome_return"].as_f64().unwrap_or(0.0)
            );
            println!("");
        }
        println!("Total Records: {}", arr.len());
    }

    Ok(())
}
