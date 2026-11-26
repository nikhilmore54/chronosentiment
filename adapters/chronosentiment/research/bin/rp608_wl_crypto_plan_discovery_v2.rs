//! RP-608-WL-IMPROVE-001 — Crypto plan decision-time feature sufficiency &
//! hypothesis discovery **v2**.
//!
//! Workspace entrypoint (authoritative):
//! ```text
//! cargo run -p chronosentiment_adapter --features research --release \
//!   --bin rp608_wl_crypto_plan_discovery_v2 -- \
//!   --journal live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl \
//!   --out-dir artifacts/rp608_econ_edge_diag
//! ```
//!
//! Gate 2 criteria unchanged. Baseline / OF ±0.30 frozen. Authority 0%.

use chronosentiment_adapter::research::multi_horizon_status::workspace_from_manifest;
use chronosentiment_adapter::research::wl_crypto_plan_discovery_v2::report::render_markdown;
use chronosentiment_adapter::research::wl_crypto_plan_discovery_v2::run_pipeline;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let workspace = workspace_from_manifest();
    let mut journal = workspace.join(
        "live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl",
    );
    let mut out_dir = workspace.join("artifacts/rp608_econ_edge_diag");

    let mut args = env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--journal" => {
                if let Some(v) = args.next() {
                    journal = PathBuf::from(v);
                    if journal.is_relative() {
                        journal = workspace.join(journal);
                    }
                }
            }
            "--out-dir" => {
                if let Some(v) = args.next() {
                    out_dir = PathBuf::from(v);
                    if out_dir.is_relative() {
                        out_dir = workspace.join(out_dir);
                    }
                }
            }
            "-h" | "--help" => {
                eprintln!(
                    "Usage: rp608_wl_crypto_plan_discovery_v2 [--journal PATH] [--out-dir DIR]"
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown arg: {other}");
                return ExitCode::from(2);
            }
        }
    }

    match run_pipeline(&journal) {
        Ok(report) => {
            if let Err(e) = fs::create_dir_all(&out_dir) {
                eprintln!("mkdir {}: {e}", out_dir.display());
                return ExitCode::FAILURE;
            }
            let stem = out_dir.join("wl_crypto_plan_discovery_v2");
            let json_path = stem.with_extension("json");
            let md_path = stem.with_extension("md");
            match serde_json::to_string_pretty(&report) {
                Ok(j) => {
                    if let Err(e) = fs::write(&json_path, j) {
                        eprintln!("write {}: {e}", json_path.display());
                        return ExitCode::FAILURE;
                    }
                }
                Err(e) => {
                    eprintln!("serialize: {e}");
                    return ExitCode::FAILURE;
                }
            }
            if let Err(e) = fs::write(&md_path, render_markdown(&report)) {
                eprintln!("write {}: {e}", md_path.display());
                return ExitCode::FAILURE;
            }
            println!(
                "{}",
                serde_json::json!({
                    "integrity_pass": report.integrity_pass,
                    "gate2_disposition": report.gate2_disposition,
                    "n_candidates": report.candidates.len(),
                    "n_promoted_hypotheses": report.hypotheses.len(),
                    "n_blocked_research_records": report.research_only_blocked_records.len(),
                    "wrote_md": md_path,
                    "wrote_json": json_path,
                })
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("pipeline error: {e}");
            ExitCode::FAILURE
        }
    }
}
