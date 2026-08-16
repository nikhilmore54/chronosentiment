//! CS-P-006-P prospective C3-002 paper clock.
//!
//! Current Yahoo daily bars → certified TMV at latest session ≤ now → C3-002
//! → seal. Does not attach outcomes. Does not evolve. Does not start C.3-G.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR, RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::observatory_prospective::{
    empty_prospective_ledger, generate_prospective_decision, seal_prospective,
    PROSPECTIVE_NOT_CSP003_VALIDATION,
};
use chronosentiment_adapter::decision_support::observatory_slice::{
    render_product_html, ObservatoryLedger,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::YahooProvider;
use chronosentiment_adapter::instrument::Instrument;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_two, historical_dir, output, now) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }
    if output.ends_with("selected_policy.json") {
        return Err("refusing to overwrite selected_policy.json".into());
    }

    let artifact: PolicyArtifact =
        serde_json::from_str(&fs::read_to_string(search_two.join("selected_policy.json"))?)?;
    if artifact.artifact_hash != RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH {
        return Err("refusing an artifact that is not C3-002 / Search #2".into());
    }

    let historical: ObservatoryLedger = serde_json::from_str(&fs::read_to_string(
        historical_dir.join("ledger.json"),
    )?)?;

    let yahoo = YahooProvider::new();
    let mut ledger = if output.join("ledger.json").exists() {
        serde_json::from_str(&fs::read_to_string(output.join("ledger.json"))?)?
    } else {
        empty_prospective_ledger()
    };
    if !ledger.observations.is_empty() {
        return Err("refusing to continue a prospective ledger that already has observations".into());
    }

    let mut sealed = 0u32;
    let mut already = 0u32;
    for ticker in RESEARCH_UNIVERSE {
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: uuid::Uuid::nil(),
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: now,
        };
        let bars = yahoo
            .fetch_historical(&instrument, TimeRange::FiveYears)
            .await?;
        let decision = generate_prospective_decision(&artifact, ticker, &bars, now)?;
        if seal_prospective(&mut ledger, decision.clone())? {
            sealed += 1;
            println!(
                "seal ticker={ticker} time={} action={:?} status=OBSERVING id={}",
                decision.decision_time, decision.action, decision.decision_id
            );
        } else {
            already += 1;
            println!(
                "exists ticker={ticker} time={} action={:?}",
                decision.decision_time, decision.action
            );
        }
    }

    fs::create_dir_all(&output)?;
    fs::write(output.join("ledger.json"), serde_json::to_vec_pretty(&ledger)?)?;
    fs::write(
        output.join("observatory.html"),
        render_product_html(&historical, Some(&ledger), now),
    )?;
    fs::write(
        historical_dir.join("observatory.html"),
        render_product_html(&historical, Some(&ledger), now),
    )?;

    println!("result=PASS");
    println!("path_kind={}", ledger.path_kind);
    println!("sealed_new={sealed}");
    println!("already_sealed={already}");
    println!("observing={}", ledger.decisions.len());
    println!("observed={}", ledger.observations.len());
    println!("csp003_validation={}", !PROSPECTIVE_NOT_CSP003_VALIDATION);
    println!("search_three_authorized={}", ledger.search_three_authorized);
    println!("output={}", output.display());
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf, DateTime<Utc>), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_two = None;
    let mut historical = None;
    let mut output = None;
    let mut now_raw = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--search-two-dir" => {
                search_two = Some(PathBuf::from(args.next().ok_or("missing --search-two-dir")?))
            }
            "--historical-dir" => {
                historical = Some(PathBuf::from(args.next().ok_or("missing --historical-dir")?))
            }
            "--output" => output = Some(PathBuf::from(args.next().ok_or("missing --output")?)),
            "--now" => now_raw = Some(args.next().ok_or("missing --now")?),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    let now = match now_raw {
        Some(s) => s.parse().map_err(|e| format!("--now must be RFC3339: {e}"))?,
        None => Utc::now(),
    };
    Ok((
        search_two.unwrap_or_else(|| PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR)),
        historical.unwrap_or_else(|| PathBuf::from("product_validation/CS-P-006/observatory")),
        output.unwrap_or_else(|| {
            PathBuf::from("product_validation/CS-P-006/observatory/prospective")
        }),
        now,
    ))
}
