//! Deferred Live observation producer — controlled CACHED_1M / YAHOO_1M clock.
//!
//! Does not rewrite the Deferred Live Driver. Emits `MarketObservation` then
//! `DeferredLiveRuntime.ingest`. Not paper_trader_v2 replay. Not a broker.
//!
//! ```text
//! cargo run -p chronosentiment_adapter --bin deferred_live -- \
//!   --ticker JUBLFOOD_NS --source cached --date 2026-09-07 --speed 60x
//!
//! cargo run -p chronosentiment_adapter --bin deferred_live -- \
//!   --scenario target --speed instant
//!
//! cargo run -p chronosentiment_adapter --bin deferred_live -- \
//!   --session 2026-09-07 --source cached --speed instant --reassess-experiment
//! ```

use std::path::PathBuf;
use std::time::Duration;

use chronosentiment_adapter::product::{
    cached_1m_tape, cached_1m_through_unix, cached_session_tape, lifecycle_scenario_tape,
    load_intraday_briefs, select_session_briefs, yahoo_1m_tape, CachedOhlcBar, DecisionBrief,
    DeferredLiveConfig, DeferredLiveRuntime, LifecycleScenario, ObservationProducer, parse_speed,
};
use serde_json::Value;

struct Args {
    ticker: String,
    source: String,
    date: Option<String>,
    speed: f64,
    cache_dir: PathBuf,
    dataset: String,
    decision_id: Option<String>,
    scenario: Option<LifecycleScenario>,
    through_horizon: bool,
    session_date: Option<String>,
    reassess_experiment: bool,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut ticker = "JUBLFOOD_NS".to_string();
    let mut source = "cached".to_string();
    let mut date = Some("2026-09-11".to_string());
    let mut speed = 60.0;
    let mut cache_dir = PathBuf::from("intraday_capture/yahoo_cache_1m");
    let mut dataset = std::env::var("INTRADAY_DATASET_PATH")
        .unwrap_or_else(|_| "datasets/p4_opportunity_dataset.json".into());
    let mut decision_id = None;
    let mut scenario = None;
    let mut through_horizon = false;
    let mut session_date = None;
    let mut reassess_experiment = std::env::var("DEFERRED_LIVE_REASSESS_EXPERIMENT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        .unwrap_or(false);
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--ticker" => {
                i += 1;
                ticker = raw.get(i).cloned().ok_or("--ticker needs a value")?;
            }
            "--source" => {
                i += 1;
                source = raw.get(i).cloned().ok_or("--source needs a value")?;
            }
            "--date" => {
                i += 1;
                date = Some(raw.get(i).cloned().ok_or("--date needs a value")?);
            }
            "--speed" => {
                i += 1;
                speed = parse_speed(raw.get(i).ok_or("--speed needs a value")?)?;
            }
            "--cache-dir" => {
                i += 1;
                cache_dir = PathBuf::from(raw.get(i).ok_or("--cache-dir needs a value")?);
            }
            "--dataset" => {
                i += 1;
                dataset = raw.get(i).cloned().ok_or("--dataset needs a value")?;
            }
            "--decision-id" => {
                i += 1;
                decision_id = Some(raw.get(i).cloned().ok_or("--decision-id needs a value")?);
            }
            "--scenario" => {
                i += 1;
                scenario = Some(LifecycleScenario::parse(
                    raw.get(i).ok_or("--scenario needs target|stop|horizon")?,
                )?);
                speed = 0.0;
            }
            "--through-horizon" => {
                through_horizon = true;
            }
            "--session" => {
                i += 1;
                session_date = Some(raw.get(i).cloned().ok_or("--session needs YYYY-MM-DD")?);
            }
            "--reassess-experiment" => {
                reassess_experiment = true;
            }
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    Ok(Args {
        ticker,
        source,
        date,
        speed,
        cache_dir,
        dataset,
        decision_id,
        scenario,
        through_horizon,
        session_date,
        reassess_experiment,
    })
}

fn pick_brief(briefs: &[DecisionBrief], args: &Args) -> Result<DecisionBrief, String> {
    if let Some(id) = &args.decision_id {
        return briefs
            .iter()
            .find(|b| b.id == *id)
            .cloned()
            .ok_or_else(|| format!("unknown decision_id {id}"));
    }
    let mut hits: Vec<&DecisionBrief> = briefs
        .iter()
        .filter(|b| b.ticker == args.ticker)
        .collect();
    if let Some(date) = &args.date {
        let dated: Vec<&DecisionBrief> = hits.iter().copied().filter(|b| b.date == *date).collect();
        if !dated.is_empty() {
            hits = dated;
        }
    }
    hits.sort_by(|a, b| b.date.cmp(&a.date).then(b.oqs.cmp(&a.oqs)));
    hits.first()
        .copied()
        .cloned()
        .ok_or_else(|| format!("no DecisionBrief for ticker {}", args.ticker))
}

async fn fetch_yahoo_1m(symbol: &str) -> Result<Vec<CachedOhlcBar>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?interval=1m&range=5d"
    );
    let response: Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("yahoo fetch: {e}"))?
        .json()
        .await
        .map_err(|e| format!("yahoo json: {e}"))?;
    let result = &response["chart"]["result"][0];
    if result.is_null() {
        return Err("Yahoo returned no 1m chart result".into());
    }
    let timestamps = result["timestamp"]
        .as_array()
        .ok_or("Yahoo chart missing timestamp")?;
    let quote = &result["indicators"]["quote"][0];
    let highs = quote["high"].as_array().ok_or("Yahoo chart missing high")?;
    let lows = quote["low"].as_array().ok_or("Yahoo chart missing low")?;
    let closes = quote["close"].as_array().ok_or("Yahoo chart missing close")?;
    let mut bars = Vec::new();
    for i in 0..timestamps.len() {
        let ts = timestamps[i].as_i64().unwrap_or(0);
        let close = closes.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
        if !close.is_finite() || close <= 0.0 {
            continue;
        }
        bars.push(CachedOhlcBar {
            timestamp: ts,
            high: highs.get(i).and_then(|v| v.as_f64()).unwrap_or(close),
            low: lows.get(i).and_then(|v| v.as_f64()).unwrap_or(close),
            close,
        });
    }
    Ok(bars)
}

async fn play_tape(
    runtime: &mut DeferredLiveRuntime,
    tape: chronosentiment_adapter::product::ControlledObservationTape,
) -> Result<(), Box<dyn std::error::Error>> {
    runtime.set_feed_snapshot(tape.snapshot());
    let mut producer = ObservationProducer::tape(tape);
    loop {
        let wait = producer.wait_millis_for_next();
        let batch = producer.poll(&[])?;
        if batch.is_empty() {
            break;
        }
        if wait > 0 {
            tokio::time::sleep(Duration::from_millis(wait)).await;
        }
        for sourced in batch {
            let src = sourced.source;
            let out = runtime.ingest_sourced(sourced);
            if out.opened {
                if let Some(pos) = runtime.ledger().positions.last() {
                    println!(
                        "[deferred-live] PAPER_ENTER {} @ {} source={}",
                        pos.paper.ticker,
                        pos.paper_entry_price(),
                        src.freshness()
                    );
                }
            }
            for ev in &out.events {
                if ev.kind != "HOLD" {
                    println!(
                        "[deferred-live] {} {} {} @ {}",
                        ev.kind,
                        runtime
                            .ledger()
                            .positions
                            .last()
                            .map(|p| p.paper.ticker.as_str())
                            .unwrap_or(""),
                        ev.note.as_deref().unwrap_or(""),
                        ev.price
                    );
                }
            }
        }
    }
    Ok(())
}

async fn run_session(args: &Args, date: &str) -> Result<(), Box<dyn std::error::Error>> {
    let briefs = load_intraday_briefs(&args.dataset)?;
    let mut runtime = DeferredLiveRuntime::new(DeferredLiveConfig::default());
    if args.reassess_experiment {
        runtime.enable_reassess_experiment();
        println!(
            "[deferred-live] reassess experiment ON (path-shape + adverse-mark gate, INVERT=false). Frozen book and Stage C marks unchanged."
        );
    }
    let armed = runtime.begin_session(&briefs, date);
    let tickers: Vec<String> = select_session_briefs(&briefs, date)
        .into_iter()
        .map(|b| b.ticker)
        .collect();
    println!(
        "[deferred-live] session {date} auto-arm {} ACT tickers (no POST /arm, driver unchanged)",
        armed.len()
    );
    if let Some(s) = runtime.session() {
        println!(
            "[deferred-live] on_date={} eligible={} skipped_not_act={} skipped_geometry={}",
            s.on_date, s.eligible, s.skipped_not_act, s.skipped_missing_geometry
        );
        println!("[deferred-live] {}", s.note);
    }
    let tape = cached_session_tape(&args.cache_dir, &tickers, date, args.speed)?;
    println!(
        "[deferred-live] mode=DEFERRED_LIVE observation_source={} bars={} speed={}x yahoo_authorized=false",
        tape.source().freshness(),
        tape.len(),
        tape.speed()
    );
    play_tape(&mut runtime, tape).await?;
    let mut n_open = 0;
    let mut n_target = 0;
    let mut n_stop = 0;
    let mut n_horizon = 0;
    for pos in &runtime.ledger().positions {
        match pos.exit_reason() {
            None => n_open += 1,
            Some("TARGET") => n_target += 1,
            Some("STOP") => n_stop += 1,
            Some("HORIZON") => n_horizon += 1,
            Some(_) => {}
        }
    }
    println!(
        "[deferred-live] session done positions={} OPEN={} TARGET={} STOP={} HORIZON={} source={} yahoo_authorized=false",
        runtime.ledger().positions.len(),
        n_open,
        n_target,
        n_stop,
        n_horizon,
        runtime.feed().observation_source.freshness()
    );
    let perf = runtime.performance(&briefs);
    println!(
        "[deferred-live] performance entered={} closed_win={} mean_closed={} mean_open_mark={} mean_fill_vs_decision={}",
        perf.n_entered,
        perf.n_win_closed,
        fmt_opt_pct(perf.mean_closed_return),
        fmt_opt_pct(perf.mean_open_mark),
        fmt_opt_pct(perf.mean_fill_vs_decision)
    );
    println!("[deferred-live] {}", perf.note);
    for row in &perf.rows {
        println!(
            "[deferred-live] {:<14} {:<5} {:<8} paper={:.2} brief={} last={:.2} {} {}={:+.2}% bars={}",
            row.ticker.replace("_NS", ""),
            row.direction,
            row.outcome,
            row.paper_entry_price,
            row.decision_entry_price
                .map(|p| format!("{p:.2}"))
                .unwrap_or_else(|| "—".into()),
            row.last_or_exit_price,
            row.ret_kind,
            row.outcome,
            row.ret * 100.0,
            row.bars_held
        );
    }
    print_reassess_experiment(&runtime);
    Ok(())
}

fn print_reassess_experiment(runtime: &DeferredLiveRuntime) {
    let rep = runtime.reassess_experiment_report();
    if !rep.enabled {
        return;
    }
    println!(
        "[deferred-live] reassess-experiment rule={} invert=false",
        rep.rule
    );
    println!(
        "[deferred-live] baseline v0.2       triggered={} helped={} hurt={} mean_delta={} sum_delta={:+.2}%",
        rep.baseline.n_triggered,
        rep.baseline.n_helped,
        rep.baseline.n_hurt,
        fmt_opt_pct(rep.baseline.mean_delta_vs_frozen),
        rep.baseline.sum_delta_vs_frozen * 100.0
    );
    println!(
        "[deferred-live] gated mark-adverse  triggered={} helped={} hurt={} suppressed={} mean_delta={} sum_delta={:+.2}%",
        rep.n_triggered,
        rep.n_helped,
        rep.n_hurt,
        rep.n_suppressed_by_mark_gate,
        fmt_opt_pct(rep.mean_delta_vs_frozen),
        rep.sum_delta_vs_frozen * 100.0
    );
    println!("[deferred-live] {}", rep.note);
    for row in &rep.rows {
        println!(
            "[deferred-live] experiment {:<14} frozen={} {}={:+.2}% v02={} v02_delta={} gated={} exp={} delta={} gate={}",
            row.ticker.replace("_NS", ""),
            row.frozen_outcome,
            row.frozen_ret_kind,
            row.frozen_ret * 100.0,
            if row.v02_triggered { "YES" } else { "no" },
            row.v02_delta_vs_frozen
                .map(|x| format!("{:+.2}%", x * 100.0))
                .unwrap_or_else(|| "—".into()),
            if row.reassess_triggered { "YES" } else { "no" },
            row.experiment_ret
                .map(|x| format!("{:+.2}%", x * 100.0))
                .unwrap_or_else(|| "—".into()),
            row.delta_vs_frozen
                .map(|x| format!("{:+.2}%", x * 100.0))
                .unwrap_or_else(|| "—".into()),
            if row.suppressed_by_mark_gate {
                "held (favorable mark)"
            } else if row.reassess_triggered {
                "exit"
            } else {
                "no signal"
            },
        );
    }
}

fn fmt_opt_pct(v: Option<f64>) -> String {
    v.map(|x| format!("{:+.2}%", x * 100.0))
        .unwrap_or_else(|| "n/a".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args().map_err(|e| e)?;
    if let Some(date) = args.session_date.clone() {
        return run_session(&args, &date).await;
    }
    let (brief, tape, config) = if let Some(kind) = args.scenario {
        println!(
            "[deferred-live] lifecycle scenario={} (producer tape only; driver unchanged)",
            kind.as_str()
        );
        lifecycle_scenario_tape(kind, args.speed)
    } else {
        let briefs = load_intraday_briefs(&args.dataset)?;
        let brief = pick_brief(&briefs, &args)?;
        let date = args
            .date
            .clone()
            .unwrap_or_else(|| brief.date.clone());
        let tape = match args.source.to_ascii_lowercase().as_str() {
            "cached" | "cached_1m" if args.through_horizon => {
                let snap = brief
                    .execution
                    .snap_unix
                    .ok_or("snap_unix required for --through-horizon")?;
                let until = snap + 300 * 60;
                cached_1m_through_unix(&args.cache_dir, &args.ticker, &date, until, args.speed)?
            }
            "cached" | "cached_1m" => {
                cached_1m_tape(&args.cache_dir, &args.ticker, &date, args.speed)?
            }
            "yahoo" | "yahoo_1m" => {
                let symbol = if args.ticker.ends_with(".NS") {
                    args.ticker.clone()
                } else {
                    args.ticker.replace("_NS", ".NS")
                };
                let bars = fetch_yahoo_1m(&symbol).await?;
                yahoo_1m_tape(&args.ticker, &bars, args.date.as_deref(), args.speed)?
            }
            other => return Err(format!("unknown --source {other}").into()),
        };
        (brief, tape, DeferredLiveConfig::default())
    };

    println!(
        "[deferred-live] mode=DEFERRED_LIVE observation_source={} ticker={} decision_id={} bars={} speed={}x",
        tape.source().freshness(),
        brief.ticker,
        brief.id,
        tape.len(),
        tape.speed()
    );
    println!(
        "[deferred-live] paper_entry_price will be the first observation price, not DecisionBrief.entry_price={:?}",
        brief.entry_price
    );
    let decision_entry = brief.entry_price;

    let mut runtime = DeferredLiveRuntime::new(config);
    if args.reassess_experiment {
        runtime.enable_reassess_experiment();
        println!(
            "[deferred-live] reassess experiment ON (path-shape + adverse-mark gate, INVERT=false). Frozen book and Stage C marks unchanged."
        );
    }
    runtime.arm(brief);
    play_tape(&mut runtime, tape).await?;

    let n = runtime.ledger().positions.len();
    if let Some(pos) = runtime.ledger().positions.first() {
        println!(
            "[deferred-live] final status={:?} exit={} paper_entry={} decision_entry={:?} last={} source={} yahoo_authorized=false",
            pos.status(),
            pos.exit_reason().unwrap_or("OPEN"),
            pos.paper_entry_price(),
            decision_entry,
            pos.current_price,
            runtime.feed().observation_source.freshness()
        );
    } else {
        println!(
            "[deferred-live] done positions={n} observation_source={} yahoo_authorized=false",
            runtime.feed().observation_source.freshness()
        );
    }
    print_reassess_experiment(&runtime);
    Ok(())
}
