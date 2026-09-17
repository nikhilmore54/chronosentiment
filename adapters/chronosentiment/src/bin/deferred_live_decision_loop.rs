//! Decision loop operator — synthetic scenarios or a cached session tape.
//!
//! ```text
//! cargo run -p chronosentiment_adapter --bin deferred_live_decision_loop -- \
//!   --scenario tape-exhausted
//!
//! cargo run -p chronosentiment_adapter --bin deferred_live_decision_loop -- \
//!   --session 2026-09-07
//!
//! cargo run -p chronosentiment_adapter --bin deferred_live_decision_loop -- \
//!   --session 2026-09-07 --inbox /tmp/deferred_live_obs.jsonl
//! ```
//!
//! `--session` runs Increment 1 + Increment 2 against cached 1m bars.
//! `--inbox` uses the same ingest path with `EXTERNAL_LIVE` observations.
//! Does not change the Deferred Live driver or CS-P-001-H.

use std::path::PathBuf;

use chronosentiment_adapter::product::{
    lifecycle_scenario_tape, load_intraday_briefs, run_cached_session, AsOfSessionDriver,
    DecisionLoopRuntime, DeferredLiveConfig, LifecycleScenario, MarketObservation,
};
use chronosentiment_adapter::product::intraday_decision::{DecisionBrief, ExecutionFacts};
use chronosentiment_adapter::product::live_observation::parse_observation_line;

const TICKER: &str = "JUBLFOOD_NS";
const SNAP: i64 = 1_000_000;
const FILL: f64 = 480.5;

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    if let Some(date) = arg_value(&raw, "--session") {
        if let Err(e) = run_session(&date, &raw) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    let scenario = arg_value(&raw, "--scenario").unwrap_or_else(|| "tape-exhausted".into());
    match scenario.as_str() {
        "target" => play_lifecycle(LifecycleScenario::Target),
        "stop" => play_lifecycle(LifecycleScenario::Stop),
        "horizon" => play_lifecycle(LifecycleScenario::Horizon),
        "before-snap" => play_steps(&[obs(SNAP - 60, FILL)]),
        "at-snap" => play_steps(&[obs(SNAP, FILL)]),
        "after-snap" => play_steps(&[obs(SNAP + 60, FILL)]),
        "same-unix" => play_steps(&[
            obs(SNAP, FILL),
            obs(SNAP + 60, 479.0),
            MarketObservation {
                ticker: TICKER.into(),
                unix: SNAP + 60,
                price: 478.0,
                high: Some(478.5),
                low: Some(477.5),
            },
        ]),
        _ => play_steps(&[obs(SNAP, FILL)]),
    }
}

fn run_session(date: &str, raw: &[String]) -> Result<(), String> {
    let dataset = arg_value(raw, "--dataset").unwrap_or_else(|| {
        std::env::var("INTRADAY_DATASET_PATH")
            .unwrap_or_else(|_| "datasets/p4_opportunity_dataset.json".into())
    });
    let briefs = load_intraday_briefs(&dataset)?;
    if let Some(inbox) = arg_value(raw, "--inbox") {
        return run_live_inbox(&briefs, date, &inbox);
    }
    let cache = PathBuf::from(arg_value(raw, "--cache-dir").unwrap_or_else(|| {
        "intraday_capture/yahoo_cache_1m".into()
    }));
    let report = run_cached_session(&briefs, date, &cache, 0.0)?;
    println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?);
    Ok(())
}

fn run_live_inbox(
    briefs: &[DecisionBrief],
    date: &str,
    inbox: &str,
) -> Result<(), String> {
    let text = std::fs::read_to_string(inbox).map_err(|e| format!("read {inbox}: {e}"))?;
    let frozen_armed_ids: Vec<String> =
        chronosentiment_adapter::product::select_session_briefs(briefs, date)
            .into_iter()
            .map(|b| b.id)
            .collect();
    let mut driver = AsOfSessionDriver::new(DeferredLiveConfig::default());
    driver.install_session(briefs, date);
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(obs) = parse_observation_line(line) {
            driver.ingest(obs);
        }
    }
    let report = driver.snapshot_report(date, frozen_armed_ids);
    println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?);
    Ok(())
}

fn arg_value(raw: &[String], flag: &str) -> Option<String> {
    raw.iter()
        .position(|a| a == flag)
        .and_then(|i| raw.get(i + 1).cloned())
}

fn play_lifecycle(kind: LifecycleScenario) {
    let (brief, tape, cfg) = lifecycle_scenario_tape(kind, 0.0);
    let mut rt = DecisionLoopRuntime::new(cfg);
    rt.arm(brief);
    rt.play_tape(tape);
    emit(rt.surfaces());
}

fn play_steps(observations: &[MarketObservation]) {
    let mut rt = DecisionLoopRuntime::new(DeferredLiveConfig { strict_t0_admission: false,
        horizon_secs: 300 * 60,
    });
    rt.arm(act_brief());
    for obs in observations {
        rt.step(obs.clone());
    }
    if observations.len() == 1 {
        rt.mark_tape_exhausted();
    }
    emit(rt.surfaces());
}

fn emit(surfaces: &[chronosentiment_adapter::product::DecisionSurface]) {
    for surface in surfaces {
        println!("{}", serde_json::to_string(surface).expect("DecisionSurface json"));
    }
}

fn obs(unix: i64, price: f64) -> MarketObservation {
    MarketObservation {
        ticker: TICKER.into(),
        unix,
        price,
        high: Some(price),
        low: Some(price),
    }
}

fn act_brief() -> DecisionBrief {
    DecisionBrief {
        id: "LIVE-HARNESS-JUBLFOOD_NS".into(),
        ticker: TICKER.into(),
        date: "2026-09-07".into(),
        direction: "SHORT".into(),
        oqs: 53,
        h60_class: "WAIT".into(),
        reference_price: Some(475.4),
        entry_price: Some(473.0),
        execution: ExecutionFacts {
            snap_unix: Some(SNAP),
            adaptive_target: Some(450.0),
            adaptive_risk: Some(490.0),
            adaptive_horizon_sessions: Some(1.0),
            target_distance_abs: None,
            target_distance_pct: None,
            risk_distance_abs: None,
            risk_distance_pct: None,
            expected_move_pct: None,
            current_price: None,
            last_tick_unix: None,
            freshness: "STALE".into(),
            horizon_elapsed: Some(false),
        },
        entry_state: "WAIT-HIGH".into(),
        entry_action: "ACT".into(),
        entry_confidence: "HIGH".into(),
        entry_horizon: "H300".into(),
        entry_why: "frozen session brief".into(),
        entry_risk: "frozen session brief".into(),
        h120_state: "WAIT-HIGH".into(),
        h120_action: "ACT".into(),
        h120_confidence: "HIGH".into(),
        h120_horizon: "H300".into(),
        h120_why: "frozen session brief".into(),
        h120_risk: "frozen session brief".into(),
        h15_ret: None,
        h30_ret: None,
        h60_ret: None,
        h120_ret: None,
        h180_ret: None,
        h300_ret: None,
        mfe_h60: None,
        mfe_h120: None,
        outcome: None,
        pnl: None,
        hist_win: None,
        hist_pf: None,
        hist_med: None,
    }
}
