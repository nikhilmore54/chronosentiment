//! CS-P-003 forward/paper session.
//!
//! `tick` — current Yahoo daily bars → decide_at(latest session ≤ now).
//! `measure` — score matured journal rows from stored prices.
//! No brokerage. Engine remains unfrozen-dev. Not a B4 replay.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, TimeZone, Utc};
use chronosentiment_adapter::decision_support::forward::{write_progress_report, ForwardJournal};
use chronosentiment_adapter::decision_support::forward_tick::{
    decide_latest_session, instrument_id_for, latest_as_of, price_bars_for, DailyBar, DEFAULT_TICKERS,
};
use chronosentiment_adapter::decision_support::policy::BaselineTrendMappingPolicy;
use chronosentiment_adapter::decision_support::replay::UNFROZEN_ENGINE_VERSION;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::YahooProvider;
use chronosentiment_adapter::instrument::Instrument;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err(
            "usage: csp003_forward_session <tick|measure> --session DIR [--now RFC3339]".into(),
        );
    }
    let cmd = args.remove(0);
    let mut session = None;
    let mut now_raw = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--session" => {
                i += 1;
                session = args.get(i).cloned();
            }
            "--now" => {
                i += 1;
                now_raw = args.get(i).cloned();
            }
            "--prices" => {
                i += 1;
            }
            other => return Err(format!("unknown argument {other}").into()),
        }
        i += 1;
    }
    let session = session.ok_or("missing --session DIR")?;
    let now: DateTime<Utc> = match now_raw {
        Some(s) => s.parse().map_err(|e| format!("--now must be RFC3339: {e}"))?,
        None => Utc::now(),
    };

    match cmd.as_str() {
        "tick" => run_tick(&session, now).await,
        "measure" => run_measure(&session, now),
        other => Err(format!("unknown command {other}").into()),
    }
}

async fn run_tick(session: &str, now: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
    let journal = ForwardJournal::open(PathBuf::from(session))?;
    fs::write(
        journal.root.join("universe.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "source": "YahooFinance",
            "exchange": "NSE",
            "cadence": "once_per_trading_day",
            "tickers": DEFAULT_TICKERS,
            "engine_version": UNFROZEN_ENGINE_VERSION,
            "not": ["b4_replay", "broker", "g_gate"]
        }))?,
    )?;

    let yahoo = YahooProvider::new();
    let mut new_decisions = 0u32;
    for ticker in DEFAULT_TICKERS {
        let instrument_id = instrument_id_for(ticker);
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: instrument_id,
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: now,
        };
        let raw = yahoo
            .fetch_historical(&instrument, TimeRange::FiveYears)
            .await?;
        let bars: Vec<DailyBar> = raw
            .into_iter()
            .filter_map(|b| {
                let ts = Utc.timestamp_opt(b.timestamp, 0).single()?;
                if b.adj_close.is_finite() && b.adj_close > 0.0 {
                    Some(DailyBar {
                        timestamp: ts,
                        close: b.adj_close,
                    })
                } else {
                    None
                }
            })
            .collect();
        let Some(as_of) = latest_as_of(&bars, now) else {
            continue;
        };
        let prices = price_bars_for(ticker, &bars, now);
        journal.persist_prices(&prices)?;
        let decision = decide_latest_session(ticker, &bars, now, &BaselineTrendMappingPolicy)?;
        let before = journal.load_ledger()?.records.len();
        let record = journal.persist(decision)?;
        let is_new = journal.load_ledger()?.records.len() > before;
        if is_new {
            new_decisions += 1;
        }
        journal.append_tick_line(&serde_json::to_string(&serde_json::json!({
            "ticker": ticker,
            "instrument_id": instrument_id,
            "as_of": as_of,
            "decision_id": record.decision_id,
            "action": record.action,
            "new": is_new,
            "engine_version": UNFROZEN_ENGINE_VERSION,
        }))?)?;
        println!(
            "tick ticker={ticker} as_of={} action={:?} new={is_new} decision_id={}",
            as_of.to_rfc3339(),
            record.action,
            record.decision_id
        );
    }

    let prices = journal.load_prices()?;
    let report = journal.performance(&prices, now)?;
    let out = PathBuf::from(session).join("reports");
    fs::create_dir_all(&out)?;
    fs::write(out.join("performance.json"), serde_json::to_vec_pretty(&report)?)?;
    write_progress_report(&out.join("PROGRESS.md"), &report)?;
    println!("new_decisions={new_decisions}");
    println!("n_decisions={}", report.behavior.n_records);
    println!("performance_content_hash={}", report.content_hash);
    Ok(())
}

fn run_measure(session: &str, now: DateTime<Utc>) -> Result<(), Box<dyn std::error::Error>> {
    let journal = ForwardJournal::open(PathBuf::from(session))?;
    let prices = journal.load_prices()?;
    let report = journal.performance(&prices, now)?;
    let out = PathBuf::from(session).join("reports");
    fs::create_dir_all(&out)?;
    fs::write(out.join("performance.json"), serde_json::to_vec_pretty(&report)?)?;
    write_progress_report(&out.join("PROGRESS.md"), &report)?;
    println!(
        "n_decisions={}",
        report.behavior.n_records
    );
    println!(
        "long={} short={} no_trade={}",
        report.behavior.counts.long, report.behavior.counts.short, report.behavior.counts.no_trade
    );
    println!("performance_content_hash={}", report.content_hash);
    Ok(())
}
