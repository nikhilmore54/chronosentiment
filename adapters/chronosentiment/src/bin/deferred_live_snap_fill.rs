//! Snap-to-fill clock over the frozen CS-P-001-D ledger.
//!
//! Joins DecisionBrief.snap_unix with the first cached 1m bar on the session
//! date. Does not re-walk TARGET / STOP / HORIZON.
//!
//! ```text
//! cargo run -p chronosentiment_adapter --bin deferred_live_snap_fill -- \
//!   --from datasets/deferred_live_universe_loss_audit.json
//! ```

use std::path::PathBuf;

use chronosentiment_adapter::product::deferred_live_loop::{
    cache_symbol_from_brief_ticker, load_cached_1m_observations,
};
use chronosentiment_adapter::product::live_observation::filter_ist_date;
use chronosentiment_adapter::product::{
    load_intraday_briefs, snap_fill_lag, summarize_snap_fill, DeferredLiveAuditRow, SnapFillRow,
};
use serde::Serialize;

#[derive(serde::Deserialize)]
struct AuditFile {
    rows: Vec<DeferredLiveAuditRow>,
}

#[derive(Serialize)]
struct Report {
    summary: chronosentiment_adapter::product::SnapFillSummary,
    rows: Vec<SnapFillRow>,
}

struct Args {
    from: PathBuf,
    dataset: String,
    cache_dir: PathBuf,
    out: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut from = PathBuf::from("datasets/deferred_live_universe_loss_audit.json");
    let mut dataset = std::env::var("INTRADAY_DATASET_PATH")
        .unwrap_or_else(|_| "datasets/p4_opportunity_dataset.json".into());
    let mut cache_dir = PathBuf::from("intraday_capture/yahoo_cache_1m");
    let mut out = PathBuf::from("datasets/deferred_live_snap_fill.json");
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--from" => {
                i += 1;
                from = PathBuf::from(raw.get(i).cloned().ok_or("--from needs a value")?);
            }
            "--dataset" => {
                i += 1;
                dataset = raw.get(i).cloned().ok_or("--dataset needs a value")?;
            }
            "--cache-dir" => {
                i += 1;
                cache_dir = PathBuf::from(raw.get(i).cloned().ok_or("--cache-dir needs a value")?);
            }
            "--out" => {
                i += 1;
                out = PathBuf::from(raw.get(i).cloned().ok_or("--out needs a value")?);
            }
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    Ok(Args {
        from,
        dataset,
        cache_dir,
        out,
    })
}

fn prices_match(fill: f64, bar_close: f64) -> bool {
    if !fill.is_finite() || !bar_close.is_finite() {
        return false;
    }
    (fill - bar_close).abs() <= 1e-6 * fill.max(1.0)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args().map_err(|e| e)?;
    let audit: AuditFile = serde_json::from_str(&std::fs::read_to_string(&args.from)?)?;
    let briefs = load_intraday_briefs(&args.dataset)?;
    let mut by_id = std::collections::BTreeMap::new();
    for b in &briefs {
        by_id.insert(b.id.as_str(), b);
    }

    let mut rows = Vec::new();
    for row in &audit.rows {
        let brief = by_id
            .get(row.decision_id.as_str())
            .copied()
            .ok_or_else(|| format!("no DecisionBrief for {}", row.decision_id))?;
        let snap = brief
            .execution
            .snap_unix
            .ok_or_else(|| format!("missing snap_unix for {}", row.decision_id))?;
        let symbol = cache_symbol_from_brief_ticker(&brief.ticker);
        let all = load_cached_1m_observations(&args.cache_dir, &symbol)?;
        let day = filter_ist_date(all, &row.date);
        let first = day
            .first()
            .ok_or_else(|| format!("no cached bars for {} on {}", symbol, row.date))?;
        let matched = prices_match(row.paper_fill, first.price);
        rows.push(SnapFillRow {
            symbol: row.symbol.clone(),
            date: row.date.clone(),
            direction: row.direction.clone(),
            exit_reason: row.exit_reason.clone(),
            decision_id: row.decision_id.clone(),
            fill_vs_decision: row.fill_vs_decision,
            lag: snap_fill_lag(snap, first.unix, matched),
        });
    }

    let summary = summarize_snap_fill(&rows);
    println!(
        "[snap-fill] n={} fill_before_snap={} fill_after_snap={} first_bar_matches={} unique_lags={} dose_response={} CS-P-001-D frozen",
        summary.n,
        summary.n_fill_before_snap,
        summary.n_fill_after_snap,
        summary.n_first_bar_matches_fill,
        summary.unique_lag_secs.len(),
        summary.dose_response_identifiable
    );
    println!(
        "[snap-fill] unique_lag_secs={:?} (hours={:?})",
        summary.unique_lag_secs,
        summary
            .unique_lag_secs
            .iter()
            .map(|s| *s as f64 / 3600.0)
            .collect::<Vec<_>>()
    );
    println!();
    println!(
        "{:<12} {:<12} {:<6} {:>8} {:>8} {:>8} {:>10} {}",
        "SYMBOL", "DATE", "EXIT", "FILL_VS", "SNAP_IST", "FILL_IST", "LAG_H", "ORDER"
    );
    for row in &rows {
        println!(
            "{:<12} {:<12} {:<6} {:>7.2}% {:>8} {:>8} {:>+9.2} {}",
            row.symbol,
            row.date,
            row.exit_reason,
            row.fill_vs_decision.unwrap_or(0.0) * 100.0,
            unix_ist_hm(row.lag.snap_unix),
            unix_ist_hm(row.lag.fill_unix),
            row.lag.lag_secs as f64 / 3600.0,
            row.lag.order.as_str()
        );
    }

    let report = Report { summary, rows };
    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&args.out, serde_json::to_string_pretty(&report)?)?;
    println!();
    println!("[snap-fill] wrote {}", args.out.display());
    println!("[snap-fill] {}", report.summary.note);
    Ok(())
}

fn unix_ist_hm(unix: i64) -> String {
    use chrono::{FixedOffset, TimeZone};
    let ist = FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("IST");
    ist.timestamp_opt(unix, 0)
        .single()
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_else(|| unix.to_string())
}
