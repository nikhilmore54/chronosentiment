//! CS-P-004 historical research laboratory run.
//!
//! Replay → ledger → outcomes → laboratory reports.
//! Read-only restore. No tuning. No G-GATE. Engine remains `unfrozen-dev`.
//! Does not write chrono_b3_test / chrono_b4_test.

use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::backtest::populate_ledger_from_assessment_schedule;
use chronosentiment_adapter::decision_support::policy::BaselineTrendMappingPolicy;
use chronosentiment_adapter::decision_support::lab_context::load_decision_context;
use chronosentiment_adapter::decision_support::laboratory::{render_reports, run_laboratory, LaboratoryInput};
use chronosentiment_adapter::decision_support::outcome::OutcomeEngine;
use chronosentiment_adapter::decision_support::replay::{ReplayAdapter, UNFROZEN_ENGINE_VERSION};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = parse_output_dir()?;
    let url = env::var("DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&url).await?;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    if matches!(database.as_str(), "chrono_b3_test" | "chrono_b4_test") {
        return Err(format!("refusing certified database {database}").into());
    }

    let adapter = ReplayAdapter::new(pool.clone());
    let ledger =
        populate_ledger_from_assessment_schedule(
            &adapter,
            UNFROZEN_ENGINE_VERSION,
            &BaselineTrendMappingPolicy,
        ).await?;
    let outcomes = OutcomeEngine::new(pool.clone()).measure_ledger(&ledger).await?;
    let context = load_decision_context(&pool, &ledger).await?;
    let report = run_laboratory(LaboratoryInput {
        ledger: &ledger,
        outcomes: &outcomes,
        context: &context,
    });
    if report.decision_engine_version != UNFROZEN_ENGINE_VERSION {
        return Err("engine version is not unfrozen-dev".into());
    }

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("laboratory.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    for (name, body) in render_reports(&report) {
        fs::write(output.join(name), body)?;
    }
    println!("laboratory_content_hash={}", report.content_hash);
    println!("ledger_identity_hash={}", report.ledger_identity_hash);
    println!("n_decisions={}", report.coverage.n_records);
    Ok(())
}

fn parse_output_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output" {
            let path = args.next().ok_or("--output requires a path")?;
            return Ok(PathBuf::from(path));
        }
    }
    Err("usage: csp004_historical_lab --output DIR".into())
}
