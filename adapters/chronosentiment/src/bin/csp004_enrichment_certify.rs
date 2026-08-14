//! Assessment Enrichment v0.1 — factor availability + temporal/lineage certification.
//!
//! Information-fidelity only. Not B5. Not G-GATE. Not a trading-strategy experiment.
//! Does not write chrono_b3_test / chrono_b4_test. Does not freeze Decision Engine v1.0.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chronosentiment_adapter::decision_support::enrichment_certify::{
    certify_snapshot, load_yahoo_cache_dir, render_certification,
};
use chronosentiment_adapter::decision_support::factor_availability::render_factor_availability;
use chronosentiment_adapter::reasoning::assessment::AssessmentProfile;
use sqlx::Row;
use uuid::Uuid;

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

    let n_decisions: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_decisions")
        .fetch_one(&pool)
        .await?;
    let orphan_decisions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM knowledge_decisions WHERE assessment_id IS NULL",
    )
    .fetch_one(&pool)
    .await?;
    let assessment_after_decision: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM knowledge_decisions d
        JOIN knowledge_assessments a ON a.id = d.assessment_id
        WHERE a.evaluation_timestamp > d.evaluation_timestamp
        "#,
    )
    .fetch_one(&pool)
    .await?;

    let instrument_rows = sqlx::query("SELECT id, display_symbol FROM instruments")
        .fetch_all(&pool)
        .await?;
    let mut labels = BTreeMap::new();
    for row in instrument_rows {
        let id: Uuid = row.try_get("id")?;
        let symbol: String = row.try_get("display_symbol")?;
        labels.insert(id, symbol);
    }

    let rows = sqlx::query(
        r#"
        SELECT a.profile_json, a.signature_hash, i.display_symbol, a.evaluation_timestamp
        FROM knowledge_assessments a
        LEFT JOIN instruments i ON i.id = a.instrument_id
        ORDER BY i.display_symbol ASC, a.evaluation_timestamp ASC, a.id ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let mut profiles = Vec::new();
    let mut stored_hashes = Vec::new();
    for row in rows {
        let profile_json: serde_json::Value = row.try_get("profile_json")?;
        let profile: AssessmentProfile = serde_json::from_value(profile_json)?;
        let hash: String = row.try_get("signature_hash")?;
        stored_hashes.push(hash);
        profiles.push(profile);
    }

    let yahoo_cache = match cache_dir {
        Some(dir) => Some(load_yahoo_cache_dir(&dir)?),
        None => None,
    };

    let cert = certify_snapshot(
        &profiles,
        &labels,
        &stored_hashes,
        n_decisions,
        orphan_decisions,
        assessment_after_decision,
        yahoo_cache.as_ref(),
    );

    fs::create_dir_all(&output)?;
    fs::write(
        output.join("certification.json"),
        serde_json::to_vec_pretty(&cert)?,
    )?;
    fs::write(
        output.join("factor_availability.json"),
        serde_json::to_vec_pretty(&cert.factor_availability)?,
    )?;
    fs::write(
        output.join("FACTOR_AVAILABILITY.md"),
        render_factor_availability(&cert.factor_availability),
    )?;
    fs::write(
        output.join("CERTIFICATION.md"),
        render_certification(&cert),
    )?;

    println!("result={}", cert.result);
    println!("n_profiles={}", cert.n_profiles);
    println!("identity_hash={}", cert.identity_hash);
    if cert.result != "PASS" {
        std::process::exit(1);
    }
    Ok(())
}

fn parse_args() -> Result<(PathBuf, Option<PathBuf>), Box<dyn std::error::Error>> {
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
    let output = output.ok_or("usage: csp004_enrichment_certify --output DIR [--yahoo-cache DIR]")?;
    Ok((output, cache))
}
