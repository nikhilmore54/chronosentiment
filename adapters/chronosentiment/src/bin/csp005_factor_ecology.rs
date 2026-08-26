//! CS-P-005 Factor Ecology Analysis v0.1.
//!
//! Read-only restore of the certified enrichment snapshot.
//! Does not implement a candidate policy. Does not tune thresholds. Not B5 / G-GATE / v1.0.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use chronosentiment_adapter::decision_support::enrichment_certify::{
    load_yahoo_cache_dir, metrics_from_bars_at_t,
};
use chronosentiment_adapter::decision_support::factor_ecology::{
    analyze, render_ecology, row_from_profile,
};
use chronosentiment_adapter::reasoning::assessment::AssessmentProfile;
use sqlx::Row;
use uuid::Uuid;

const EXPECT_DUMP_SHA: &str = "e7685d936bdfaf53d7055ca683a87b4ca85149dd0eb89402dfaa93facfd8616f";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (output, cache_dir) = parse_args()?;
    let url = env::var("DATABASE_URL")?;
    let pool = sqlx::PgPool::connect(&url).await?;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await?;
    if matches!(database.as_str(), "chrono_b3_test" | "chrono_b4_test") {
        return Err(format!("refusing certified database {database}").into());
    }

    let cache = load_yahoo_cache_dir(&cache_dir)?;
    let instrument_rows = sqlx::query("SELECT id, display_symbol FROM instruments")
        .fetch_all(&pool)
        .await?;
    let mut labels = BTreeMap::new();
    for row in instrument_rows {
        let id: Uuid = row.try_get("id")?;
        let symbol: String = row.try_get("display_symbol")?;
        labels.insert(id, symbol);
    }

    let assess_rows = sqlx::query(
        r#"
        SELECT instrument_id, evaluation_timestamp, profile_json
        FROM knowledge_assessments
        ORDER BY evaluation_timestamp ASC, instrument_id ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let outcome_rows = sqlx::query(
        r#"
        SELECT d.instrument_id, d.evaluation_timestamp, o.horizon, o.outcome_return
        FROM knowledge_outcomes o
        JOIN knowledge_decisions d ON o.decision_id = d.id
        "#,
    )
    .fetch_all(&pool)
    .await?;
    let mut outcomes: BTreeMap<(Uuid, DateTime<Utc>, String), f64> = BTreeMap::new();
    for row in outcome_rows {
        let id: Uuid = row.try_get("instrument_id")?;
        let ts: DateTime<Utc> = row.try_get("evaluation_timestamp")?;
        let horizon: String = row.try_get("horizon")?;
        let ret: f64 = row.try_get("outcome_return")?;
        outcomes.insert((id, ts, horizon), ret);
    }

    let mut rows = Vec::new();
    for row in assess_rows {
        let instrument_id: Uuid = row.try_get("instrument_id")?;
        let ts: DateTime<Utc> = row.try_get("evaluation_timestamp")?;
        let profile_json: serde_json::Value = row.try_get("profile_json")?;
        let profile: AssessmentProfile = serde_json::from_value(profile_json)?;
        let symbol = labels
            .get(&instrument_id)
            .cloned()
            .unwrap_or_else(|| instrument_id.to_string());
        let (roc_20, atr_14) = match cache.get(&symbol) {
            Some(bars) => {
                let metrics = metrics_from_bars_at_t(bars, ts, instrument_id);
                (metrics.get_float("roc_20"), metrics.get_float("atr_14"))
            }
            None => (None, None),
        };
        let mut ecology = row_from_profile(&profile, symbol, roc_20, atr_14);
        ecology.outcome_5d = outcomes.get(&(instrument_id, ts, "5D".into())).copied();
        ecology.outcome_10d = outcomes.get(&(instrument_id, ts, "10D".into())).copied();
        ecology.outcome_20d = outcomes.get(&(instrument_id, ts, "20D".into())).copied();
        ecology.outcome_60d = outcomes.get(&(instrument_id, ts, "60D".into())).copied();
        rows.push(ecology);
    }

    let report = analyze(&rows);
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("ecology.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    fs::write(output.join("rows.json"), serde_json::to_vec_pretty(&rows)?)?;
    fs::write(output.join("FACTOR_ECOLOGY.md"), render_ecology(&report))?;
    fs::write(
        output.join("DESIGN_CONSTRAINTS.md"),
        render_constraints(&report),
    )?;
    println!("n_rows={}", report.n_rows);
    println!("snapshot_expect_sha={EXPECT_DUMP_SHA}");
    Ok(())
}

fn render_constraints(
    report: &chronosentiment_adapter::decision_support::factor_ecology::FactorEcologyReport,
) -> String {
    let mut md = String::from("# Candidate-policy design constraints (not a policy)\n\n");
    for c in &report.design_constraints {
        md.push_str(&format!("- {c}\n"));
    }
    md.push_str("\nDo not search thresholds against the attached 60D outcomes.\n");
    md
}

fn parse_args() -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut output = None;
    let mut cache = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output = Some(PathBuf::from(
                    args.next().ok_or("--output requires a path")?,
                ));
            }
            "--yahoo-cache" => {
                cache = Some(PathBuf::from(
                    args.next().ok_or("--yahoo-cache requires a path")?,
                ));
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        output.ok_or("usage: csp005_factor_ecology --output DIR --yahoo-cache DIR")?,
        cache.ok_or("usage: csp005_factor_ecology --output DIR --yahoo-cache DIR")?,
    ))
}
