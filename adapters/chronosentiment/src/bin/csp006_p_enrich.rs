//! CS-P-006-P Enrichment Emitter — `csp006_p_enrich`
//!
//! Reads the 101 sealed decisions from the prospective `ledger.json` and
//! emits each one to the Coralys Decision Server with the market context
//! (reference_price, atr_14) required by RecommendationEngine v1.
//!
//! **Architecture:**
//! ```text
//!   IMMUTABLE ledger.json (sealed decisions)
//!         +
//!   Yahoo market data (incremental cache)
//!         ↓
//!   Evaluation Context (reference_price, atr_14)
//!         ↓
//!   POST /decisions  (existing decision_id — idempotent, 409 = already exists)
//!         ↓
//!   Coralys Decision Server → RecommendationEngine v1
//! ```
//!
//! **Governance invariants:**
//! - Does NOT reseal or mutate the ledger.
//! - Does NOT create new decision IDs.
//! - Uses the existing sealed `decision_id` from `ledger.json`.
//! - 409 Conflict from the server = decision already present = success (idempotent).
//!
//! **Usage:**
//! ```bash
//! cargo run -p chronosentiment_adapter --bin csp006_p_enrich -- \
//!   --ledger product_validation/CS-P-006/observatory/prospective/ledger.json \
//!   --emit-url http://localhost:3001
//! ```

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use chronosentiment_adapter::decision_support::enrichment_certify::metrics_from_bars_at_t;
use chronosentiment_adapter::decision_support::forward_tick::instrument_id_for;
use chronosentiment_adapter::decision_support::observatory_prospective::latest_session_at_or_before;
use chronosentiment_adapter::decision_support::observatory_slice::{
    ObservatoryLedger, SealedDecisionRecord,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use chronosentiment_adapter::ingestion::provider::{MarketDataProvider, TimeRange};
use chronosentiment_adapter::ingestion::yahoo::YahooProvider;
use chronosentiment_adapter::instrument::Instrument;

// ─── Canonical hashes (mirrors csp006_p_prospective.rs) ──────────────────────

const C3_002_POLICY_ARTIFACT_HASH: &str =
    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121";
const CORALYS_EXEC_ARTIFACT_HASH: &str =
    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f";

// ─── Session helper ───────────────────────────────────────────────────────────

fn next_trading_session(t: DateTime<Utc>) -> String {
    let mut d = t.date_naive() + Duration::days(1);
    loop {
        match d.weekday() {
            Weekday::Sat | Weekday::Sun => d += Duration::days(1),
            _ => break,
        }
    }
    d.format("%Y-%m-%d").to_string()
}

// ─── Emit ─────────────────────────────────────────────────────────────────────

/// POST a sealed decision + market context to the Coralys Decision Server.
///
/// Uses the **existing `decision_id`** from the sealed ledger — does not
/// create a new ID. 409 = already exists = idempotent success.
async fn emit_to_server(
    client: &reqwest::Client,
    base_url: &str,
    decision: &SealedDecisionRecord,
    atr_14: Option<f64>,
    reference_price: Option<f64>,
) -> Result<bool, String> {
    let direction = match decision.action {
        DecisionAction::Long => "LONG",
        DecisionAction::Short => "SHORT",
        DecisionAction::NoTrade => "NO_TRADE",
    };
    let decision_ts: DateTime<Utc> = decision
        .decision_time
        .parse()
        .map_err(|e| format!("bad decision_time '{}': {e}", decision.decision_time))?;
    let data_snapshot_id = format!("yahoo-daily-{}", decision_ts.format("%Y%m%dT%H%M%SZ"));
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
    if status.as_u16() == 409 {
        // Already exists — idempotent, treat as success but flag as existing
        return Ok(false); // false = already existed
    }
    if status.is_success() {
        return Ok(true); // true = newly created
    }
    let text = resp.text().await.unwrap_or_default();
    Err(format!("POST {url} returned {status}: {text}"))
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (ledger_path, emit_url, now) = parse_args()?;

    // Load the sealed prospective ledger (immutable — we do not write it back).
    let raw = fs::read_to_string(&ledger_path)
        .map_err(|e| format!("cannot read ledger {}: {e}", ledger_path.display()))?;
    let ledger: ObservatoryLedger =
        serde_json::from_str(&raw).map_err(|e| format!("ledger JSON parse error: {e}"))?;

    let decisions: Vec<SealedDecisionRecord> = ledger.decisions;
    println!(
        "ledger={} decisions={} observations={}",
        ledger_path.display(),
        decisions.len(),
        ledger.observations.len()
    );

    if decisions.is_empty() {
        println!("result=SKIP reason=no_decisions");
        return Ok(());
    }

    let yahoo = YahooProvider::new();
    let http_client = reqwest::Client::new();

    let mut emitted_new = 0u32;
    let mut already = 0u32;
    let mut emit_errors = 0u32;
    let mut skipped_no_data = 0u32;

    for decision in &decisions {
        let ticker = &decision.instrument;

        // Build a minimal Instrument for the Yahoo provider.
        let mut provider_ids = HashMap::new();
        provider_ids.insert("yahoo".to_string(), ticker.to_string());
        let instrument = Instrument {
            id: uuid::Uuid::nil(),
            exchange: "NSE".to_string(),
            display_symbol: ticker.to_string(),
            provider_ids,
            created_at: now,
        };

        // Fetch bars (incremental — uses cache if warm).
        let bars = match yahoo
            .fetch_historical(&instrument, TimeRange::FiveYears)
            .await
        {
            Ok(b) => b,
            Err(e) => {
                eprintln!("skip ticker={ticker} reason=yahoo_error error={e}");
                skipped_no_data += 1;
                continue;
            }
        };

        // Compute market context at decision time T.
        let decision_ts: DateTime<Utc> = match decision.decision_time.parse() {
            Ok(ts) => ts,
            Err(e) => {
                eprintln!("skip ticker={ticker} reason=bad_decision_time error={e}");
                skipped_no_data += 1;
                continue;
            }
        };
        let t = latest_session_at_or_before(&bars, decision_ts).unwrap_or(decision_ts);
        let instrument_id = instrument_id_for(ticker);
        let metrics = metrics_from_bars_at_t(&bars, t, instrument_id);
        let atr_14 = metrics.get_float("atr_14");
        let reference_price = bars
            .iter()
            .filter(|b| b.timestamp <= t.timestamp())
            .last()
            .and_then(|b| if b.close > 0.0 { Some(b.close) } else { None });

        // Emit to Decision Server.
        match emit_to_server(&http_client, &emit_url, decision, atr_14, reference_price).await {
            Ok(true) => {
                emitted_new += 1;
                println!(
                    "emit ticker={ticker} id={} atr_14={atr_14:?} ref={reference_price:?} status=NEW",
                    decision.decision_id
                );
            }
            Ok(false) => {
                already += 1;
                println!(
                    "emit ticker={ticker} id={} status=ALREADY_EXISTS",
                    decision.decision_id
                );
            }
            Err(e) => {
                emit_errors += 1;
                eprintln!("emit ticker={ticker} id={} error={e}", decision.decision_id);
            }
        }
    }

    println!("result=PASS");
    println!("emitted_new={emitted_new}");
    println!("already_existed={already}");
    println!("skipped_no_data={skipped_no_data}");
    println!("emit_errors={emit_errors}");

    if emit_errors > 0 {
        return Err(format!("{emit_errors} decision(s) failed to emit to Decision Server").into());
    }
    Ok(())
}

// ─── Arg parsing ─────────────────────────────────────────────────────────────

fn parse_args() -> Result<(PathBuf, String, DateTime<Utc>), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let mut ledger_path = None;
    let mut emit_url = None;
    let mut now_raw = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ledger" => ledger_path = Some(PathBuf::from(args.next().ok_or("missing --ledger")?)),
            "--emit-url" => emit_url = Some(args.next().ok_or("missing --emit-url")?),
            "--now" => now_raw = Some(args.next().ok_or("missing --now")?),
            other => return Err(format!("unknown argument {other}").into()),
        }
    }
    let ledger = ledger_path.unwrap_or_else(|| {
        PathBuf::from("product_validation/CS-P-006/observatory/prospective/ledger.json")
    });
    let url = emit_url.ok_or("--emit-url is required")?;
    let now = match now_raw {
        Some(s) => s
            .parse()
            .map_err(|e| format!("--now must be RFC3339: {e}"))?,
        None => Utc::now(),
    };
    Ok((ledger, url, now))
}
