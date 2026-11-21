//! RP-608 position decide / manage / plan-lifecycle CLI — Rust shadow cutover.
//!
//! Modes:
//! - **One-shot:** Gate2X snapshot JSON → decision JSON
//! - **`--manage-loop`:** Gate2X JSONL snapshots → decisions (in-process)
//! - **`--plan-lifecycle-loop`:** plan+quote JSONL → full lifecycle tick
//!   (Adaptive TP + SL/TP + Gate 2X) — Python shadows become I/O only
//!
//! Deterministic · coralys-* frozen (PVP-003) · books never pooled.

use std::env;
use std::io::{self, BufRead, Read, Write};
use std::process;

use chronosentiment_adapter::product::{
    Gate2xBookKind, Gate2xFeatureSnapshot, Gate2xPositionManager, OpenPlanState,
    PlanLifecycleConfig, PlanLifecycleManager, PlanQuote, PLAN_LIFECYCLE_PROTOCOL,
};
use serde::Deserialize;

const VERSION: &str = "RP-608-POSITION-DECIDE-001";
const MANAGE_VERSION: &str = "RP-608-GATE2X-POSITION-MANAGE-001";

fn detect_book(evidence_dir: &str, explicit: Option<&str>) -> Gate2xBookKind {
    if let Some(b) = explicit {
        return match b.to_ascii_lowercase().as_str() {
            "nse" => Gate2xBookKind::Nse,
            _ => Gate2xBookKind::Crypto,
        };
    }
    let lower = evidence_dir.to_ascii_lowercase();
    if lower.contains("historical_crypto") || lower.ends_with("crypto") {
        Gate2xBookKind::Crypto
    } else {
        Gate2xBookKind::Nse
    }
}

#[derive(Debug, Deserialize)]
struct PlanLifecycleRequest {
    plan: OpenPlanState,
    quote: PlanQuote,
    #[serde(default)]
    config: Option<PlanLifecycleConfig>,
    /// When true, reset Gate2X latch (new position).
    #[serde(default)]
    reset_latch: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut evidence_dir = "datasets/recommendation/historical_crypto".to_string();
    let mut manage_loop = false;
    let mut plan_lifecycle_loop = false;
    let mut book_arg: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--evidence-dir" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--evidence-dir requires a path");
                    process::exit(2);
                }
                evidence_dir = args[i].clone();
            }
            "--book" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--book requires crypto|nse");
                    process::exit(2);
                }
                book_arg = Some(args[i].clone());
            }
            "--manage-loop" => {
                manage_loop = true;
            }
            "--plan-lifecycle-loop" => {
                plan_lifecycle_loop = true;
            }
            "--version" => {
                println!("{VERSION}");
                process::exit(0);
            }
            "-h" | "--help" => {
                eprintln!(
                    "rp608_position_decide ({VERSION} / {MANAGE_VERSION} / {PLAN_LIFECYCLE_PROTOCOL})\n\
                     One-shot:              JSON stdin snapshot → JSON stdout decision\n\
                     Gate2X manage:         --manage-loop  JSONL → JSONL\n\
                     Plan lifecycle manage: --plan-lifecycle-loop  JSONL plan+quote → JSONL result\n\
                     --evidence-dir DIR   REC001h JSONL directory (crypto or NSE)\n\
                     --book crypto|nse    force book (else inferred from evidence-dir)\n\
                     --version            print protocol id\n\
                     Env: RP608_POSITION_DECIDE_BIN / RP608_GATE2X_MANAGE_BIN / RP608_PLAN_LIFECYCLE_BIN"
                );
                process::exit(0);
            }
            other => {
                eprintln!("unknown arg: {other}");
                process::exit(2);
            }
        }
        i += 1;
    }

    let book = detect_book(&evidence_dir, book_arg.as_deref());

    if plan_lifecycle_loop {
        let mut mgr = match PlanLifecycleManager::load(
            &evidence_dir,
            book,
            PlanLifecycleConfig::default(),
        ) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };
        eprintln!(
            "[{PLAN_LIFECYCLE_PROTOCOL}] plan-lifecycle-loop ready book={book:?} evidence={evidence_dir}"
        );
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("stdin read: {e}");
                    process::exit(1);
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            let req: PlanLifecycleRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("parse plan-lifecycle request: {e}");
                    process::exit(1);
                }
            };
            if req.reset_latch {
                mgr.reset_position_latch();
            }
            // Optional per-request config override is accepted but manager keeps
            // process-default for latch continuity; apply via new fields on plan.
            let _ = req.config;
            let result = mgr.manage_tick(req.plan, req.quote);
            let out = match serde_json::to_string(&result) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("serialize: {e}");
                    process::exit(1);
                }
            };
            if writeln!(stdout, "{out}").is_err() {
                process::exit(1);
            }
            let _ = stdout.flush();
        }
        return;
    }

    let mut mgr = match Gate2xPositionManager::load(&evidence_dir, book) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    if manage_loop {
        eprintln!(
            "[{MANAGE_VERSION}] manage-loop ready book={book:?} evidence={evidence_dir}",
        );
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("stdin read: {e}");
                    process::exit(1);
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            let snap: Gate2xFeatureSnapshot = match serde_json::from_str(&line) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("parse snapshot: {e}");
                    process::exit(1);
                }
            };
            let decision = mgr.manage_tick(snap);
            let out = match serde_json::to_string(&decision) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("serialize: {e}");
                    process::exit(1);
                }
            };
            if writeln!(stdout, "{out}").is_err() {
                process::exit(1);
            }
            let _ = stdout.flush();
        }
        return;
    }

    // One-shot
    let mut buf = String::new();
    if io::stdin().read_to_string(&mut buf).is_err() {
        process::exit(1);
    }
    let snap: Gate2xFeatureSnapshot = match serde_json::from_str(buf.trim()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("parse snapshot: {e}");
            process::exit(1);
        }
    };
    let decision = mgr.manage_tick(snap);
    match serde_json::to_string(&decision) {
        Ok(s) => {
            println!("{s}");
        }
        Err(e) => {
            eprintln!("serialize: {e}");
            process::exit(1);
        }
    }
}
