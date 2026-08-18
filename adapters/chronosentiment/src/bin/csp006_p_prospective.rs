//! CS-P-006-P prospective C3-002 paper clock.
//!
//! Current Yahoo daily bars → certified TMV at latest session ≤ now → C3-002
//! → seal. Does not attach outcomes. Does not evolve. Does not start C.3-G.
//!
//! Optional: `--emit-url http://localhost:3001` — POST each newly-sealed
//! decision to the Coralys Decision Server so it appears in chrono-ui.
//!
//! Optional: `--universe datasets/universes/coralys_102_v1.json` — load the
//! instrument universe from a canonical JSON file instead of RESEARCH_UNIVERSE.
//! The JSON must have an `"instruments"` array of Yahoo ticker strings.
//! Without this flag the original 7-stock RESEARCH_UNIVERSE is used.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR, RESEARCH_UNIVERSE,
};
use chronosentiment_adapter::decision_support::enrichment_certify::metrics_from_bars_at_t;
use chronosentiment_adapter::decision_support::forward_tick::instrument_id_for;
use chronosentiment_adapter::decision_support::observatory_prospective::{
    empty_prospective_ledger, generate_prospective_decision,
    latest_session_at_or_before, seal_prospective,
    PROSPECTIVE_NOT_CSP003_VALIDATION,
};
use chronosentiment_adapter::decision_support::observatory_slice::{
    render_product_html, ObservatoryLedger, SealedDecisionRecord,
};
use chronosentiment_adapter::decision_support::policy_artifact::PolicyArtifact;
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::YahooProvider;
use chronosentiment_adapter::instrument::Instrument;

// ─── Canonical hashes (mirrors coralys-decision/src/adapter.rs) ──────────────

const C3_002_POLICY_ARTIFACT_HASH: &str =
    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121";
const CORALYS_EXEC_ARTIFACT_HASH: &str =
    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f";

// ─── Session helpers ──────────────────────────────────────────────────────────

/// Return the next NSE trading session date (YYYY-MM-DD) after `t`.
///
/// Simple rule: skip Saturday (6) and Sunday (7). Does not account for
/// NSE holidays — a future enhancement can add a holiday calendar.
fn next_trading_session(t: DateTime<Utc>) -> String {
    use chrono::{Datelike, Duration, Weekday};
    let mut d = t.date_naive() + Duration::days(1);
    loop {
        match d.weekday() {
            Weekday::Sat | Weekday::Sun => d += Duration::days(1),
            _ => break,
        }
    }
    d.format("%Y-%m-%d").to_string()
}

// ─── Decision Server emit ─────────────────────────────────────────────────────

/// POST a newly-sealed decision to the Coralys Decision Server.
///
/// Maps `SealedDecisionRecord` → the `IngestRequest` JSON body accepted by
/// `POST /decisions`.
///
/// - `target_price` and `reference_risk_boundary_price` are `None` — they
///   require `entry_price` from the next session open (E), not from T.
/// - `atr_14` and `reference_price` (previous close at T) are included so
///   the UI can display them and the server can compute execution params at E.
/// - `effective_session` is the next trading session date (YYYY-MM-DD).
#[allow(clippy::too_many_arguments)]
async fn emit_to_server(
    client: &reqwest::Client,
    base_url: &str,
    decision: &SealedDecisionRecord,
    atr_14: Option<f64>,
    reference_price: Option<f64>,
) -> Result<(), String> {
    let direction = match decision.action {
        DecisionAction::Long => "LONG",
        DecisionAction::Short => "SHORT",
        DecisionAction::NoTrade => "NO_TRADE",
    };
    let decision_ts: DateTime<Utc> = decision
        .decision_time
        .parse()
        .map_err(|e| format!("bad decision_time: {e}"))?;
    let data_snapshot_id = format!(
        "yahoo-daily-{}",
        decision_ts.format("%Y%m%dT%H%M%SZ")
    );
    let effective_session = next_trading_session(decision_ts);
    let body = serde_json::json!({
        "decision_id": decision.decision_id,
        "instrument": decision.instrument,
        "decision_timestamp": decision_ts,
        "direction": direction,
        "trend": decision.state.trend,
        "momentum": decision.state.momentum,
        "volatility": decision.state.volatility,
        "target_price": null,
        "policy_artifact_hash": C3_002_POLICY_ARTIFACT_HASH,
        "execution_artifact_hash": CORALYS_EXEC_ARTIFACT_HASH,
        "decision_pipeline": "C3-002",
        "data_snapshot_id": data_snapshot_id,
        "certified_timestamp": decision_ts,
        "reference_risk_boundary_price": null,
        "reference_risk_boundary_type": "CORALYS_V0_ATR_TMV",
        "atr_14": atr_14,
        "reference_price": reference_price,
        "effective_session": effective_session
    });
    let url = format!("{base_url}/decisions");
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("POST {url} failed: {e}"))?;
    let status = resp.status();
    if status.is_success() || status.as_u16() == 409 {
        // 409 = already exists — idempotent, not an error
        Ok(())
    } else {
        let text = resp.text().await.unwrap_or_default();
        Err(format!("POST {url} returned {status}: {text}"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (search_two, historical_dir, output, now, emit_url, universe_path) = parse_args()?;

    // Load instrument universe: from --universe JSON file or fall back to RESEARCH_UNIVERSE.
    let universe: Vec<String> = if let Some(ref path) = universe_path {
        let raw = fs::read_to_string(path)
            .map_err(|e| format!("cannot read --universe {}: {e}", path.display()))?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| format!("--universe JSON parse error: {e}"))?;
        let tickers = parsed["instruments"]
            .as_array()
            .ok_or("--universe JSON must have an 'instruments' array")?
            .iter()
            .map(|v| v.as_str().ok_or("instrument must be a string").map(|s| s.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        println!("universe=custom path={} count={}", path.display(), tickers.len());
        tickers
    } else {
        let tickers: Vec<String> = RESEARCH_UNIVERSE.iter().map(|s| s.to_string()).collect();
        println!("universe=RESEARCH_UNIVERSE count={}", tickers.len());
        tickers
    };
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

    let http_client = reqwest::Client::new();
    let mut sealed = 0u32;
    let mut already = 0u32;
    let mut emitted = 0u32;
    let mut emit_errors = 0u32;
    let mut skipped = 0u32;
    for ticker in &universe {
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: uuid::Uuid::nil(),
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: now,
        };
        let bars = match yahoo
            .fetch_historical(&instrument, TimeRange::FiveYears)
            .await
        {
            Ok(b) => b,
            Err(e) => {
                eprintln!("skip ticker={ticker} reason=yahoo_error error={e}");
                skipped += 1;
                continue;
            }
        };
        let decision = match generate_prospective_decision(&artifact, ticker, &bars, now) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skip ticker={ticker} reason=decision_error error={e}");
                skipped += 1;
                continue;
            }
        };
        let is_new = seal_prospective(&mut ledger, decision.clone())?;
        if is_new {
            sealed += 1;
            println!(
                "seal ticker={ticker} time={} action={:?} status=OBSERVING id={}",
                decision.decision_time, decision.action, decision.decision_id
            );
            // Emit to Decision Server if --emit-url was supplied.
            if let Some(ref url) = emit_url {
                // Extract ATR-14 and reference price (previous close at T) from bars.
                let t = latest_session_at_or_before(&bars, now)
                    .unwrap_or(now);
                let instrument_id = instrument_id_for(ticker);
                let metrics = metrics_from_bars_at_t(&bars, t, instrument_id);
                let atr_14 = metrics.get_float("atr_14");
                let reference_price = bars
                    .iter()
                    .filter(|b| b.timestamp <= t.timestamp())
                    .last()
                    .and_then(|b| if b.close > 0.0 { Some(b.close) } else { None });
                match emit_to_server(&http_client, url, &decision, atr_14, reference_price).await {
                    Ok(()) => {
                        emitted += 1;
                        println!("emit ticker={ticker} url={url} atr_14={atr_14:?} ref={reference_price:?} status=OK");
                    }
                    Err(e) => {
                        emit_errors += 1;
                        eprintln!("emit ticker={ticker} url={url} error={e}");
                    }
                }
            }
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
    println!("skipped_no_data={skipped}");
    println!("observing={}", ledger.decisions.len());
    println!("observed={}", ledger.observations.len());
    println!("csp003_validation={}", !PROSPECTIVE_NOT_CSP003_VALIDATION);
    println!("search_three_authorized={}", ledger.search_three_authorized);
    println!("output={}", output.display());
    if emit_url.is_some() {
        println!("emitted={emitted}");
        println!("emit_errors={emit_errors}");
    }
    if emit_errors > 0 {
        return Err(format!("{emit_errors} decision(s) failed to emit to Decision Server").into());
    }
    Ok(())
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf, DateTime<Utc>, Option<String>, Option<PathBuf>), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut search_two = None;
    let mut historical = None;
    let mut output = None;
    let mut now_raw = None;
    let mut emit_url = None;
    let mut universe_path = None;
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
            "--emit-url" => emit_url = Some(args.next().ok_or("missing --emit-url")?),
            "--universe" => {
                universe_path = Some(PathBuf::from(args.next().ok_or("missing --universe")?))
            }
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
        emit_url,
        universe_path,
    ))
}
