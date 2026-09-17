//! Full-universe Deferred Live replay + loss audit.
//!
//! Frozen path only:
//! cached 1m → MarketObservation → DeferredLiveRuntime.ingest → paper fill
//! → TARGET / STOP / HORIZON → ledger. Does not enable reassessment or INVERT.
//! Does not mutate the Deferred Live driver.
//!
//! ```text
//! cargo run -p chronosentiment_adapter --bin deferred_live_universe_audit -- \
//!   --source cached --speed instant
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use chronosentiment_adapter::product::{
    audit_rows_from_ledger, cached_session_tape, ingest_tape, load_intraday_briefs,
    select_session_briefs, summarize_universe_audit, DeferredLiveAuditRow, DeferredLiveConfig,
    DeferredLiveRuntime, UniverseAuditSummary,
};
use chronosentiment_adapter::product::deferred_live_loop::{
    brief_ticker_from_cache_symbol, cache_symbol_from_brief_ticker, load_cached_1m_observations,
};
use chronosentiment_adapter::product::live_observation::unix_ist_date;
use serde::Serialize;

#[derive(Serialize)]
struct UniverseAuditReport {
    summary: UniverseAuditSummary,
    coverage: Coverage,
    rows: Vec<DeferredLiveAuditRow>,
    losses: Vec<DeferredLiveAuditRow>,
}

#[derive(Serialize)]
struct Coverage {
    cache_dates: Vec<String>,
    brief_dates: Vec<String>,
    sessions_run: Vec<String>,
    cache_only_dates: Vec<String>,
    brief_only_dates: Vec<String>,
    cache_symbols: usize,
    act_with_cache: usize,
    act_without_cache: usize,
    skipped_not_act: usize,
}

struct Args {
    cache_dir: PathBuf,
    dataset: String,
    profiles: PathBuf,
    out: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut cache_dir = PathBuf::from("intraday_capture/yahoo_cache_1m");
    let mut dataset = std::env::var("INTRADAY_DATASET_PATH")
        .unwrap_or_else(|_| "datasets/p4_opportunity_dataset.json".into());
    let mut profiles = PathBuf::from("datasets/symbol_movement_profiles.csv");
    let mut out = PathBuf::from("datasets/deferred_live_universe_loss_audit.json");
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--cache-dir" => {
                i += 1;
                cache_dir = PathBuf::from(raw.get(i).cloned().ok_or("--cache-dir needs a value")?);
            }
            "--dataset" => {
                i += 1;
                dataset = raw.get(i).cloned().ok_or("--dataset needs a value")?;
            }
            "--profiles" => {
                i += 1;
                profiles = PathBuf::from(raw.get(i).cloned().ok_or("--profiles needs a value")?);
            }
            "--out" => {
                i += 1;
                out = PathBuf::from(raw.get(i).cloned().ok_or("--out needs a value")?);
            }
            "--source" | "--speed" => {
                i += 1; // accepted for CLI compatibility; universe is always cached/instant
            }
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    Ok(Args {
        cache_dir,
        dataset,
        profiles,
        out,
    })
}

fn list_cache_tickers(cache_dir: &Path) -> Result<Vec<String>, String> {
    let mut tickers = Vec::new();
    let entries = std::fs::read_dir(cache_dir)
        .map_err(|e| format!("read {}: {e}", cache_dir.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(stem) = name.strip_suffix(".json") {
            tickers.push(brief_ticker_from_cache_symbol(stem));
        }
    }
    tickers.sort();
    tickers.dedup();
    Ok(tickers)
}

fn cache_ist_dates(cache_dir: &Path, tickers: &[String]) -> BTreeSet<String> {
    let mut dates = BTreeSet::new();
    for ticker in tickers {
        match load_cached_1m_observations(cache_dir, &cache_symbol_from_brief_ticker(ticker)) {
            Ok(obs) => {
                for o in obs {
                    let d = unix_ist_date(o.unix);
                    if !d.is_empty() {
                        dates.insert(d);
                    }
                }
            }
            Err(e) => eprintln!("[universe-audit] cache skip {ticker}: {e}"),
        }
    }
    dates
}

fn load_profiles(path: &Path) -> BTreeMap<(String, String), String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    let mut lines = text.lines();
    let header = match lines.next() {
        Some(h) => h,
        None => return out,
    };
    let cols: Vec<&str> = header.split(',').collect();
    let idx = |name: &str| cols.iter().position(|c| *c == name);
    let ticker_i = idx("ticker");
    let dir_i = idx("direction");
    let n_i = idx("n_obs");
    let win_i = idx("win_rate_h300");
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        let get = |i: Option<usize>| i.and_then(|n| fields.get(n)).copied().unwrap_or("");
        let ticker = get(ticker_i).trim();
        let direction = get(dir_i).trim().to_ascii_uppercase();
        if ticker.is_empty() || direction.is_empty() {
            continue;
        }
        let n = get(n_i);
        let win = get(win_i);
        out.insert(
            (ticker.to_string(), direction),
            format!("n_obs={n} h300_win={win}"),
        );
    }
    out
}

fn fmt_pct(v: Option<f64>) -> String {
    v.map(|x| format!("{:+.2}%", x * 100.0))
        .unwrap_or_else(|| "n/a".into())
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.4}")).unwrap_or_else(|| "—".into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args().map_err(|e| e)?;
    let briefs = load_intraday_briefs(&args.dataset)?;
    let cache_tickers = list_cache_tickers(&args.cache_dir)?;
    let cache_dates = cache_ist_dates(&args.cache_dir, &cache_tickers);
    let brief_dates: BTreeSet<String> = briefs.iter().map(|b| b.date.clone()).collect();
    let sessions_run: Vec<String> = cache_dates
        .iter()
        .filter(|d| brief_dates.contains(*d))
        .cloned()
        .collect();
    let cache_only: Vec<String> = cache_dates
        .iter()
        .filter(|d| !brief_dates.contains(*d))
        .cloned()
        .collect();
    let brief_only: Vec<String> = brief_dates
        .iter()
        .filter(|d| !cache_dates.contains(*d))
        .cloned()
        .collect();
    let cache_set: BTreeSet<&str> = cache_tickers.iter().map(|s| s.as_str()).collect();

    let mut act_decisions = 0usize;
    let mut act_with_cache = 0usize;
    let mut act_without_cache = 0usize;
    let mut skipped_not_act = 0usize;
    for date in &sessions_run {
        let on_date: Vec<_> = briefs.iter().filter(|b| b.date == *date).collect();
        skipped_not_act += on_date
            .iter()
            .filter(|b| !b.entry_action.eq_ignore_ascii_case("ACT"))
            .count();
        let selected = select_session_briefs(&briefs, date);
        act_decisions += selected.len();
        for b in selected {
            if cache_set.contains(b.ticker.as_str()) {
                act_with_cache += 1;
            } else {
                act_without_cache += 1;
            }
        }
    }

    let profiles = load_profiles(&args.profiles);
    let mut all_rows = Vec::new();

    println!(
        "[universe-audit] cached symbols={} cache_dates={} sessions_with_briefs={} instant CACHED_1M reassess=off invert=false",
        cache_tickers.len(),
        cache_dates.len(),
        sessions_run.len()
    );

    for date in &sessions_run {
        let mut runtime = DeferredLiveRuntime::new(DeferredLiveConfig::default());
        let armed = runtime.begin_session(&briefs, date);
        println!(
            "[universe-audit] session {date} auto-arm {} ACT (driver unchanged)",
            armed.len()
        );
        if armed.is_empty() {
            println!("[universe-audit] session {date} no ACT — skip ingest");
            continue;
        }
        let tape = cached_session_tape(&args.cache_dir, &cache_tickers, date, 0.0)?;
        println!(
            "[universe-audit] session {date} bars={} source={}",
            tape.len(),
            tape.source().freshness()
        );
        ingest_tape(&mut runtime, tape);
        let rows = audit_rows_from_ledger(runtime.ledger(), &briefs, &profiles);
        println!(
            "[universe-audit] session {date} entered={}",
            rows.len()
        );
        all_rows.extend(rows);
    }

    let summary = summarize_universe_audit(
        &all_rows,
        sessions_run.len(),
        cache_tickers.len(),
        act_decisions,
    );
    let losses: Vec<DeferredLiveAuditRow> = all_rows
        .iter()
        .filter(|r| r.realized_pct.is_some_and(|x| x < 0.0))
        .cloned()
        .collect();

    println!();
    println!("ALL SYMBOLS");
    println!("───────────");
    println!("Sessions          {}", summary.sessions);
    println!("Symbols cached    {}", summary.symbols_cached);
    println!("Symbols entered   {}", summary.symbols_entered);
    println!("ACT decisions     {}", summary.act_decisions);
    println!("Paper entries     {}", summary.paper_entries);
    println!("TARGET            {}", summary.n_target);
    println!("STOP              {}", summary.n_stop);
    println!("HORIZON           {}", summary.n_horizon);
    println!("OPEN              {}", summary.n_open);
    println!("Wins              {}", summary.n_win);
    println!("Losses            {}", summary.n_loss);
    println!("Mean return       {}", fmt_pct(summary.mean_return));
    println!("Total return      {}", fmt_pct(summary.total_return));
    println!("Mean closed       {}", fmt_pct(summary.mean_closed_return));
    println!("Mean open mark    {}", fmt_pct(summary.mean_open_mark));
    println!("Mean fill vs brief{}", fmt_pct(summary.mean_fill_vs_decision));
    println!();
    println!("LOSSES");
    println!("──────");
    println!("loss count        {}", summary.losses.loss_count);
    println!("closed count      {}", summary.losses.closed_count);
    println!(
        "loss rate         {}",
        summary
            .losses
            .loss_rate
            .map(|x| format!("{:.1}%", x * 100.0))
            .unwrap_or_else(|| "n/a".into())
    );
    println!("open underwater   {}", summary.losses.open_underwater);
    println!(
        "worst symbols     {}",
        summary
            .losses
            .worst_symbols
            .iter()
            .map(|k| format!("{} ({})", k.key, k.n))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "worst directions  {}",
        summary
            .losses
            .worst_directions
            .iter()
            .map(|k| format!("{} ({})", k.key, k.n))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "worst exits       {}",
        summary
            .losses
            .worst_exit_reasons
            .iter()
            .map(|k| format!("{} ({})", k.key, k.n))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!();
    println!(
        "{:<12} {:<12} {:<6} {:>4} {:>10} {:>10} {:<8} {:>10} {:>9} {:>9} {:>9} {:>9} {:<8}",
        "SYMBOL",
        "DATE",
        "DIR",
        "OQS",
        "DECISION",
        "FILL",
        "EXIT",
        "EXIT_PX",
        "REALIZED",
        "MAE",
        "MFE",
        "MARK",
        "STATE"
    );
    for row in &all_rows {
        println!(
            "{:<12} {:<12} {:<6} {:>4} {:>10} {:>10.2} {:<8} {:>10} {:>9} {:>9} {:>9} {:>9} {:<8}",
            row.symbol,
            row.date,
            row.direction,
            row.oqs.map(|v| v.to_string()).unwrap_or_else(|| "—".into()),
            fmt_opt(row.decision_price),
            row.paper_fill,
            row.exit_reason,
            row.exit_price
                .map(|p| format!("{p:.2}"))
                .unwrap_or_else(|| "—".into()),
            row.realized_pct
                .map(|x| format!("{:+.2}%", x * 100.0))
                .unwrap_or_else(|| "—".into()),
            format!("{:+.2}%", row.mae * 100.0),
            format!("{:+.2}%", row.mfe * 100.0),
            row.mark_at_tape_end
                .map(|x| format!("{:+.2}%", x * 100.0))
                .unwrap_or_else(|| "—".into()),
            row.session_state
        );
    }

    let report = UniverseAuditReport {
        summary,
        coverage: Coverage {
            cache_dates: cache_dates.into_iter().collect(),
            brief_dates: brief_dates.into_iter().collect(),
            sessions_run: sessions_run.clone(),
            cache_only_dates: cache_only,
            brief_only_dates: brief_only,
            cache_symbols: cache_tickers.len(),
            act_with_cache,
            act_without_cache,
            skipped_not_act,
        },
        rows: all_rows,
        losses,
    };
    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&args.out, serde_json::to_string_pretty(&report)?)?;
    println!();
    println!("[universe-audit] wrote {}", args.out.display());
    println!("[universe-audit] {}", report.summary.note);
    Ok(())
}
