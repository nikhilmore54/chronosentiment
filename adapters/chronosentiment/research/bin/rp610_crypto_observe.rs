//! RP-610 crypto observation CLI (Rust port of scripts/rp610_crypto_observe.py).

use chrono::{TimeZone, Utc};
use chronosentiment_adapter::research::multi_horizon_status::workspace_from_manifest;
use chronosentiment_adapter::research::rp610_crypto_multi_horizon::{
    admit_candidate, bootstrap_demo_candidates, build_candidate, default_artifact_journals,
    summarize_books, write_daily_observation, Rp610Store, DEFAULT_NOTIONAL_USDT,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage() -> ! {
    eprintln!(
        "Usage:
  rp610_crypto_observe bootstrap [--with-demo] [--price P]
  rp610_crypto_observe admit --horizon H1_MULTI_DAY|H2_MULTI_WEEK|H3_LONG_TERM --price P [--symbol S]
  rp610_crypto_observe eod --price P [--symbol S]
  rp610_crypto_observe report"
    );
    std::process::exit(2);
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == name)
        .map(|w| w[1].clone())
}

fn has(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let workspace = workspace_from_manifest();
    let journals = env::var("RP610_JOURNALS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_artifact_journals(&workspace));
    let store = Rp610Store::new(journals);

    let result = (|| -> Result<(), String> {
        match args[0].as_str() {
            "bootstrap" => {
                let path = store.write_horizon_registry()?;
                let mut demo_admitted = 0;
                if has(&args, "--with-demo") {
                    let price: f64 = flag(&args, "--price")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(78_000.0);
                    let out = bootstrap_demo_candidates(&store, "BTCUSDT", price, Utc::now())?;
                    demo_admitted = out.len();
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "registry": path,
                        "protocol_id": chronosentiment_adapter::research::rp610_crypto_multi_horizon::PROTOCOL_ID,
                        "live_order_authority": 0.0,
                        "report": summarize_books(&store)?,
                        "demo_admitted": demo_admitted,
                    }))
                    .unwrap()
                );
            }
            "admit" => {
                let hid = flag(&args, "--horizon").ok_or("missing --horizon")?;
                let price: f64 = flag(&args, "--price")
                    .ok_or("missing --price")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                let symbol = flag(&args, "--symbol").unwrap_or_else(|| "BTCUSDT".into());
                let c = build_candidate(
                    &symbol,
                    "LONG",
                    price,
                    Utc::now(),
                    &hid,
                    "cli_admit",
                    "CLI",
                    DEFAULT_NOTIONAL_USDT,
                    None,
                    None,
                )?;
                let admitted = admit_candidate(&store, &c)?;
                println!("{}", serde_json::to_string_pretty(&admitted).unwrap());
            }
            "eod" => {
                let price: f64 = flag(&args, "--price")
                    .ok_or("missing --price")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                let symbol = flag(&args, "--symbol").unwrap_or_else(|| "BTCUSDT".into());
                let mut marks = BTreeMap::new();
                marks.insert(symbol, price);
                let day = flag(&args, "--session-date").and_then(|s| {
                    let parts: Vec<_> = s.split('-').collect();
                    if parts.len() == 3 {
                        let y: i32 = parts[0].parse().ok()?;
                        let m: u32 = parts[1].parse().ok()?;
                        let d: u32 = parts[2].parse().ok()?;
                        Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).single()
                    } else {
                        None
                    }
                });
                let summary =
                    write_daily_observation(&store, day.unwrap_or_else(Utc::now), &marks, "")?;
                println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            }
            "report" => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summarize_books(&store)?).unwrap()
                );
            }
            _ => usage(),
        }
        Ok(())
    })();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
