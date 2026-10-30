//! RP-609 observation CLI (Rust port of scripts/rp609_observe.py).
//!
//! live_order_authority = 0.0. CNC paper books only.

use chronosentiment_adapter::research::multi_horizon_status::workspace_from_manifest;
use chronosentiment_adapter::research::rp609_multi_horizon::{
    default_artifact_journals, MultiHorizonEngine,
};
use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage() -> ! {
    eprintln!(
        "Usage:
  rp609_observe bootstrap
  rp609_observe swing-open --symbol SYM --price P --qty Q --thesis T [--max-sessions N]
  rp609_observe portfolio-plan --plan-id ID --capital C
  rp609_observe long-open --symbol SYM --price P --qty Q --thesis T [--version V]
  rp609_observe eod [--session-date YYYY-MM-DD]
  rp609_observe report
  rp609_observe review --hold-id ID [--decision HOLD_CONTINUE|THESIS_INVALIDATED]"
    );
    std::process::exit(2);
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == name)
        .map(|w| w[1].clone())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let workspace = workspace_from_manifest();
    let journals = env::var("RP609_JOURNALS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_artifact_journals(&workspace));
    let nse_dir = env::var("RP609_NSE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace
                .join("live_capture")
                .join("shadow_validation")
                .join("nse")
        });

    let cmd = args[0].as_str();
    let result = (|| -> Result<(), String> {
        match cmd {
            "bootstrap" => {
                let eng = MultiHorizonEngine::new(journals, nse_dir)?;
                eng.persist()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "registry": eng.journals.join("RP609_horizon_registry.json"),
                        "report": eng.report(),
                    }))
                    .unwrap()
                );
            }
            "swing-open" => {
                let symbol = flag(&args, "--symbol").ok_or("missing --symbol")?;
                let price: f64 = flag(&args, "--price")
                    .ok_or("missing --price")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                let qty: f64 = flag(&args, "--qty")
                    .ok_or("missing --qty")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                let thesis = flag(&args, "--thesis").ok_or("missing --thesis")?;
                let max_sessions: i64 = flag(&args, "--max-sessions")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                let stop = flag(&args, "--stop")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(price * 0.96);
                let target = flag(&args, "--target")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(price * 1.06);
                let mut eng = MultiHorizonEngine::new(journals, nse_dir)?;
                let row = eng.register_swing_candidate(
                    &symbol,
                    &thesis,
                    price,
                    stop,
                    target,
                    qty,
                    max_sessions,
                    None,
                    None,
                    true,
                    flag(&args, "--session-date").as_deref(),
                )?;
                println!("{}", serde_json::to_string_pretty(&row).unwrap());
            }
            "portfolio-plan" => {
                let plan_id = flag(&args, "--plan-id").ok_or("missing --plan-id")?;
                let capital: f64 = flag(&args, "--capital")
                    .ok_or("missing --capital")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                let legs = vec![
                    json!({"symbol": "RELIANCE.NS", "sector": "ENERGY", "target_weight_pct": 8.0}),
                    json!({"symbol": "TCS.NS", "sector": "IT", "target_weight_pct": 8.0}),
                    json!({"symbol": "HDFCBANK.NS", "sector": "BANKING", "target_weight_pct": 8.0}),
                    json!({"symbol": "ITC.NS", "sector": "FMCG", "target_weight_pct": 6.0}),
                    json!({"symbol": "LT.NS", "sector": "INDUSTRIALS", "target_weight_pct": 6.0}),
                ];
                let mut eng = MultiHorizonEngine::new(journals, nse_dir)?;
                let row = eng.register_portfolio_plan(
                    &plan_id,
                    capital,
                    &legs,
                    4,
                    10.0,
                    30.0,
                    flag(&args, "--session-date").as_deref(),
                )?;
                println!("{}", serde_json::to_string_pretty(&row).unwrap());
            }
            "long-open" => {
                let symbol = flag(&args, "--symbol").ok_or("missing --symbol")?;
                let price: f64 = flag(&args, "--price")
                    .ok_or("missing --price")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                let qty: f64 = flag(&args, "--qty")
                    .ok_or("missing --qty")?
                    .parse()
                    .map_err(|e: std::num::ParseFloatError| e.to_string())?;
                let thesis = flag(&args, "--thesis").ok_or("missing --thesis")?;
                let version = flag(&args, "--version").unwrap_or_else(|| "v1".into());
                let mut eng = MultiHorizonEngine::new(journals, nse_dir)?;
                let row = eng.open_long_term(
                    &symbol,
                    &thesis,
                    &version,
                    price,
                    qty,
                    None,
                    60,
                    flag(&args, "--session-date").as_deref(),
                )?;
                println!("{}", serde_json::to_string_pretty(&row).unwrap());
            }
            "eod" => {
                let mut eng = MultiHorizonEngine::new(journals, nse_dir)?;
                let summary =
                    eng.run_eod(flag(&args, "--session-date").as_deref(), None)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "eod": summary,
                        "report": eng.report(),
                    }))
                    .unwrap()
                );
            }
            "report" => {
                let eng = MultiHorizonEngine::new(journals, nse_dir)?;
                println!("{}", serde_json::to_string_pretty(&eng.report()).unwrap());
            }
            "review" => {
                let hold_id = flag(&args, "--hold-id").ok_or("missing --hold-id")?;
                let decision =
                    flag(&args, "--decision").unwrap_or_else(|| "HOLD_CONTINUE".into());
                let mut eng = MultiHorizonEngine::new(journals, nse_dir)?;
                let row = eng.complete_long_term_review(&hold_id, &decision)?;
                println!("{}", serde_json::to_string_pretty(&row).unwrap());
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
