//! CS-P-006 disposable 7-instrument research snapshot.
//!
//! Not B4. Not B5. Does not write chrono_b3_test / chrono_b4_test.
//! Outcomes are not consumed during state construction.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use chronosentiment_adapter::decision_support::csp006_protocol::RESEARCH_UNIVERSE;
use chronosentiment_adapter::decision_support::csp006_snapshot::{
    build_research_snapshot, certify_research_snapshot, load_required_yahoo_cache,
    parse_enrichment_identity_file, render_snapshot_certification, repeated_identity_matches,
    write_identity_file,
};
use chronosentiment_adapter::decision_support::forward_tick::instrument_id_for;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::YahooProvider;
use chronosentiment_adapter::instrument::Instrument;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (output, cache_dir, identity_path) = parse_args()?;
    let db = env::var("DATABASE_URL").unwrap_or_default();
    if db.contains("chrono_b3_test") || db.contains("chrono_b4_test") {
        return Err("refusing certified database name in DATABASE_URL".into());
    }

    fs::create_dir_all(&cache_dir)?;
    fs::create_dir_all(&output)?;
    env::set_var("CHRONO_YAHOO_CACHE_DIR", &cache_dir);

    fetch_missing_tickers(&cache_dir).await?;
    let cache = load_required_yahoo_cache(&cache_dir).map_err(|e| e.to_string())?;
    let snapshot = build_research_snapshot(&cache).map_err(|e| e.to_string())?;
    let again = build_research_snapshot(&cache).map_err(|e| e.to_string())?;
    if !repeated_identity_matches(&snapshot, &again) {
        return Err("repeated snapshot identity differs".into());
    }

    let five = match identity_path {
        Some(path) => {
            let text = fs::read_to_string(path)?;
            Some(parse_enrichment_identity_file(&text).map_err(|e| e.to_string())?)
        }
        None => None,
    };
    let cert = certify_research_snapshot(&snapshot, &cache, five.as_ref());

    fs::write(
        output.join("snapshot.json"),
        serde_json::to_vec_pretty(&snapshot)?,
    )?;
    fs::write(output.join("identity.txt"), write_identity_file(&snapshot))?;
    fs::write(
        output.join("certification.json"),
        serde_json::to_vec_pretty(&cert)?,
    )?;
    fs::write(
        output.join("CERTIFICATION.md"),
        render_snapshot_certification(&cert),
    )?;
    fs::write(
        output.join("PROVENANCE.md"),
        format!(
            "# CS-P-006 7-instrument snapshot provenance\n\n\
             **Not B4. Not B5.**\n\n\
             - kind: `{}`\n\
             - generated_at_wall_clock: {}\n\
             - identity_hash: `{}`\n\
             - yahoo_cache: {}\n\
             - instruments: {}\n",
            snapshot.kind,
            Utc::now(),
            snapshot.identity_hash,
            cache_dir.display(),
            snapshot.instruments.join(", ")
        ),
    )?;

    println!("result={}", cert.result);
    println!("discovery_ready={}", cert.discovery_ready);
    println!("n_rows={}", snapshot.n_rows);
    println!("identity_hash={}", snapshot.identity_hash);
    if cert.result != "PASS" {
        std::process::exit(1);
    }
    Ok(())
}

async fn fetch_missing_tickers(cache_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let yahoo = YahooProvider::new();
    for ticker in RESEARCH_UNIVERSE {
        let path = cache_dir.join(format!("{ticker}.json"));
        if path.exists() {
            continue;
        }
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: instrument_id_for(ticker),
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: Utc::now(),
        };
        yahoo
            .fetch_historical(&instrument, TimeRange::FiveYears)
            .await?;
        if !path.exists() {
            return Err(format!("fetch did not write {ticker} cache").into());
        }
    }
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, Option<PathBuf>), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut output = None;
    let mut cache = None;
    let mut identity = None;
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
            "--five-instrument-identity" => {
                identity = Some(PathBuf::from(
                    args.next()
                        .ok_or("--five-instrument-identity requires a path")?,
                ));
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    Ok((
        output.ok_or("usage: csp006_research_snapshot --output DIR --yahoo-cache DIR [--five-instrument-identity FILE]")?,
        cache.ok_or("missing --yahoo-cache")?,
        identity,
    ))
}
