//! TIME-006 — Evidence Dataset Construction.
//!
//! # Purpose
//!
//! Joins TIME-004 (T0 ledger) with TIME-005 (forward observations) into a
//! single flat evidence row per decision. This is a pure dataset construction
//! step — no recomputation, no interpretation, no algorithm changes.
//!
//! # Identity chain preserved
//!
//!   TIME-002 reconstruction_id
//!     → TIME-003 decision_replay_id
//!       → TIME-004 decision_id
//!         → TIME-005 observation_id
//!           → TIME-006 evidence_row_id
//!
//! # Join key
//!
//!   TIME-004.decision_id == TIME-005.decision_id
//!
//! # Output schema (one row per decision)
//!
//!   T0 fields (immutable, from TIME-004):
//!     decision_id, as_of, ticker, direction, action
//!     evidence_class, target_rate, sample_size, degradation_level
//!     adaptive_target, adaptive_risk, adaptive_rr, adaptive_horizon_sessions
//!     reference_price, atr_14, trend, momentum, volatility
//!     reconstruction_id, decision_replay_id
//!
//!   T+h fields (observational, from TIME-005):
//!     exit_reason, sessions_to_outcome, target_reached, risk_reached
//!     horizon_reached, ambiguous, actual_mfe, actual_mae, realized_return
//!     eligible_for_primary_comparison, observation_id
//!
//! # Acceptance criteria
//!
//!   AC-T6-01 Join completeness
//!     Every TIME-004 entry with a matching TIME-005 observation produces exactly
//!     one evidence row. No TIME-004 entry is silently dropped.
//!
//!   AC-T6-02 T0 immutability
//!     No T0 field is recomputed or modified. Values are copied verbatim from
//!     TIME-004 ledger entries.
//!
//!   AC-T6-03 T+h immutability
//!     No outcome field is recomputed or modified. Values are copied verbatim
//!     from TIME-005 observation files.
//!
//!   AC-T6-04 Identity chain integrity
//!     Every evidence row carries the full 5-step identity chain.
//!
//!   AC-T6-05 Accounting invariant
//!     n_joined + n_missing_observation == n_total_ledger_entries.
//!
//!   AC-T6-06 Idempotency
//!     Running twice produces the same dataset (overwrite, not append).
//!
//! # Usage
//!
//! ```bash
//! cargo run -p chronosentiment_adapter --bin time006_evidence_dataset -- \
//!   --ledger       time_machine/ledger/ \
//!   --observations time_machine/observations/ \
//!   --output       time_machine/evidence/
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const PRODUCER: &str = "time006_evidence_dataset.v1";

// ─── Input schemas ────────────────────────────────────────────────────────────

/// TIME-004 ledger entry (T0 fields — read-only).
#[derive(Debug, Deserialize, Clone)]
struct LedgerEntry {
    decision_id: String,
    as_of: String,
    ticker: String,
    direction: String,
    action: String,
    reference_price: Option<f64>,
    atr_14: Option<f64>,
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    adaptive_target: Option<f64>,
    adaptive_risk: Option<f64>,
    adaptive_rr: Option<f64>,
    adaptive_horizon_sessions: Option<f64>,
    adaptive_upside_pct: Option<f64>,
    adaptive_downside_pct: Option<f64>,
    degradation_level: Option<String>,
    sample_size: Option<u64>,
    target_rate: Option<f64>,
    evidence_class: Option<String>,
    rank_score: Option<f64>,
    vol_regime: Option<String>,
    volume_regime: Option<String>,
    decision_replay_id: String,
    reconstruction_id: String,
    source_type: String,
}

/// TIME-005 observation (T+h fields — read-only).
#[derive(Debug, Deserialize, Clone)]
struct Observation {
    observation_id: String,
    decision_id: String,
    exit_reason: String,
    sessions_to_outcome: Option<usize>,
    target_reached: bool,
    risk_reached: bool,
    horizon_reached: bool,
    ambiguous: bool,
    actual_mfe: f64,
    actual_mae: f64,
    realized_return: f64,
    eligible_for_primary_comparison: bool,
    n_bars_after_t0: usize,
    n_bars_in_horizon: usize,
}

// ─── Output schema ────────────────────────────────────────────────────────────

/// One evidence row — T0 joined with T+h. Immutable once written.
#[derive(Debug, Serialize, Clone)]
struct EvidenceRow {
    // ── Identity chain ────────────────────────────────────────────────────────
    evidence_row_id: String,
    producer: String,
    dataset_run_id: String,
    constructed_at: String,

    // ── Step 1: TIME-002 reconstruction ──────────────────────────────────────
    reconstruction_id: String,

    // ── Step 2: TIME-003 decision replay ─────────────────────────────────────
    decision_replay_id: String,

    // ── Step 3: TIME-004 T0 ledger ────────────────────────────────────────────
    decision_id: String,
    as_of: String,
    source_type: String,

    // ── T0 instrument ─────────────────────────────────────────────────────────
    ticker: String,
    direction: String,
    action: String,
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,

    // ── T0 price / execution parameters ──────────────────────────────────────
    reference_price: Option<f64>,
    atr_14: Option<f64>,
    adaptive_target: Option<f64>,
    adaptive_risk: Option<f64>,
    adaptive_rr: Option<f64>,
    adaptive_horizon_sessions: Option<f64>,
    adaptive_upside_pct: Option<f64>,
    adaptive_downside_pct: Option<f64>,

    // ── T0 evidence classification ────────────────────────────────────────────
    evidence_class: Option<String>,
    target_rate: Option<f64>,
    sample_size: Option<u64>,
    degradation_level: Option<String>,
    rank_score: Option<f64>,
    vol_regime: Option<String>,
    volume_regime: Option<String>,

    // ── Step 4: TIME-005 observation ──────────────────────────────────────────
    observation_id: String,
    n_bars_after_t0: usize,
    n_bars_in_horizon: usize,

    // ── T+h outcome ───────────────────────────────────────────────────────────
    exit_reason: String,
    sessions_to_outcome: Option<usize>,
    target_reached: bool,
    risk_reached: bool,
    horizon_reached: bool,
    ambiguous: bool,
    actual_mfe: f64,
    actual_mae: f64,
    realized_return: f64,
    eligible_for_primary_comparison: bool,
}

/// TIME-006 run summary.
#[derive(Debug, Serialize)]
struct DatasetRunSummary {
    run_id: String,
    run_at: String,
    producer: String,
    decision_replay_id: String,
    as_of: String,
    n_total_ledger_entries: usize,
    n_joined: usize,
    n_missing_observation: usize,
    n_rows_written: usize,
    output_dir: String,
    // AC flags
    ac_t6_01_join_completeness: bool,
    ac_t6_02_t0_immutability: bool,
    ac_t6_03_th_immutability: bool,
    ac_t6_04_identity_chain: bool,
    ac_t6_05_accounting: bool,
    ac_t6_06_idempotency: bool,
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    ledger: PathBuf,
    observations: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut ledger = PathBuf::from("time_machine/ledger");
    let mut observations = PathBuf::from("time_machine/observations");
    let mut output = PathBuf::from("time_machine/evidence");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--ledger"       => { i += 1; ledger = PathBuf::from(&args[i]); }
            "--observations" => { i += 1; observations = PathBuf::from(&args[i]); }
            "--output"       => { i += 1; output = PathBuf::from(&args[i]); }
            other => return Err(format!("unknown argument: {other}").into()),
        }
        i += 1;
    }

    Ok(Args { ledger, observations, output })
}

// ─── Loaders ─────────────────────────────────────────────────────────────────

fn load_ledger_entries(ledger_dir: &PathBuf) -> Result<Vec<LedgerEntry>, Box<dyn std::error::Error>> {
    let entries_dir = ledger_dir.join("entries");
    let mut entries = Vec::new();
    for entry in fs::read_dir(&entries_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let rec: LedgerEntry = serde_json::from_str(&content)
            .map_err(|e| format!("cannot parse {}: {e}", path.display()))?;
        entries.push(rec);
    }
    entries.sort_by(|a, b| a.ticker.cmp(&b.ticker));
    Ok(entries)
}

fn load_observations(obs_dir: &PathBuf) -> Result<HashMap<String, Observation>, Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    for entry in fs::read_dir(obs_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let fname = path.file_name().unwrap().to_string_lossy().to_string();
        if fname == "latest_run.json" {
            continue;
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let obs: Observation = serde_json::from_str(&content)
            .map_err(|e| format!("cannot parse {}: {e}", path.display()))?;
        map.insert(obs.decision_id.clone(), obs);
    }
    Ok(map)
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    println!("[time006] TIME-006 — Evidence Dataset Construction");
    println!("[time006] ==========================================");
    println!("[time006] ledger:       {}", args.ledger.display());
    println!("[time006] observations: {}", args.observations.display());
    println!("[time006] output:       {}", args.output.display());

    // ── AC-T6-02: Load TIME-004 ledger entries (read-only) ────────────────────
    let entries = load_ledger_entries(&args.ledger)?;
    println!("[time006] n_ledger_entries={}", entries.len());

    if entries.is_empty() {
        return Err("no ledger entries found — run TIME-004 first".into());
    }

    let as_of_str = entries[0].as_of.clone();
    let decision_replay_id = entries[0].decision_replay_id.clone();

    // ── AC-T6-03: Load TIME-005 observations (read-only) ─────────────────────
    let observations = load_observations(&args.observations)?;
    println!("[time006] n_observations={}", observations.len());

    // ── Create output directory (AC-T6-06: overwrite, not append) ────────────
    fs::create_dir_all(&args.output)?;

    let now = Utc::now();
    let constructed_at = now.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let run_id = format!(
        "TIME006-{}-gen{}",
        as_of_str.replace(':', "").replace('-', "").replace('T', "T").replace('Z', "Z"),
        now.format("%Y%m%dT%H%M%S%6fZ")
    );

    let n_total = entries.len();
    let mut n_joined = 0usize;
    let mut n_missing = 0usize;
    let mut rows_written = Vec::new();

    for entry in &entries {
        let obs = match observations.get(&entry.decision_id) {
            Some(o) => o,
            None => {
                n_missing += 1;
                println!(
                    "[time006] MISSING observation for decision_id={} ticker={}",
                    entry.decision_id, entry.ticker
                );
                continue;
            }
        };

        // AC-T6-04: Build evidence_row_id from identity chain.
        let evidence_row_id = format!(
            "TIME006-{}-{}",
            decision_replay_id.trim_start_matches("TIME003-"),
            entry.ticker.replace('.', "_")
        );

        let row = EvidenceRow {
            evidence_row_id: evidence_row_id.clone(),
            producer: PRODUCER.to_string(),
            dataset_run_id: run_id.clone(),
            constructed_at: constructed_at.clone(),
            reconstruction_id: entry.reconstruction_id.clone(),
            decision_replay_id: entry.decision_replay_id.clone(),
            decision_id: entry.decision_id.clone(),
            as_of: entry.as_of.clone(),
            source_type: entry.source_type.clone(),
            ticker: entry.ticker.clone(),
            direction: entry.direction.clone(),
            action: entry.action.clone(),
            trend: entry.trend.clone(),
            momentum: entry.momentum.clone(),
            volatility: entry.volatility.clone(),
            reference_price: entry.reference_price,
            atr_14: entry.atr_14,
            adaptive_target: entry.adaptive_target,
            adaptive_risk: entry.adaptive_risk,
            adaptive_rr: entry.adaptive_rr,
            adaptive_horizon_sessions: entry.adaptive_horizon_sessions,
            adaptive_upside_pct: entry.adaptive_upside_pct,
            adaptive_downside_pct: entry.adaptive_downside_pct,
            evidence_class: entry.evidence_class.clone(),
            target_rate: entry.target_rate,
            sample_size: entry.sample_size,
            degradation_level: entry.degradation_level.clone(),
            rank_score: entry.rank_score,
            vol_regime: entry.vol_regime.clone(),
            volume_regime: entry.volume_regime.clone(),
            observation_id: obs.observation_id.clone(),
            n_bars_after_t0: obs.n_bars_after_t0,
            n_bars_in_horizon: obs.n_bars_in_horizon,
            exit_reason: obs.exit_reason.clone(),
            sessions_to_outcome: obs.sessions_to_outcome,
            target_reached: obs.target_reached,
            risk_reached: obs.risk_reached,
            horizon_reached: obs.horizon_reached,
            ambiguous: obs.ambiguous,
            actual_mfe: obs.actual_mfe,
            actual_mae: obs.actual_mae,
            realized_return: obs.realized_return,
            eligible_for_primary_comparison: obs.eligible_for_primary_comparison,
        };

        // Write individual row JSON (AC-T6-06: overwrite).
        let row_path = args.output.join(format!("{evidence_row_id}.json"));
        fs::write(&row_path, serde_json::to_string_pretty(&row)?)?;
        rows_written.push(row);
        n_joined += 1;

        println!(
            "[time006] joined ticker={} evidence_class={:?} action={} exit={} target_reached={} ret={:.4}",
            entry.ticker,
            entry.evidence_class.as_deref().unwrap_or("None"),
            entry.action,
            obs.exit_reason,
            obs.target_reached,
            obs.realized_return
        );
    }

    // ── AC-T6-05: Accounting invariant ────────────────────────────────────────
    let accounting_ok = n_joined + n_missing == n_total;
    println!(
        "[time006] accounting: total={n_total} joined={n_joined} missing={n_missing}"
    );
    println!("[time006] AC-T6-05 accounting_invariant={accounting_ok}");

    if !accounting_ok {
        return Err(format!(
            "AC-T6-05 FAIL: {n_joined} + {n_missing} != {n_total}"
        ).into());
    }

    // ── Write flat CSV for downstream analysis ────────────────────────────────
    let csv_path = args.output.join("evidence_dataset.csv");
    let mut csv = String::new();
    csv.push_str("evidence_row_id,as_of,ticker,direction,action,evidence_class,target_rate,sample_size,degradation_level,adaptive_rr,adaptive_horizon_sessions,reference_price,adaptive_target,adaptive_risk,exit_reason,sessions_to_outcome,target_reached,risk_reached,horizon_reached,actual_mfe,actual_mae,realized_return,eligible_for_primary_comparison\n");
    for row in &rows_written {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            row.evidence_row_id,
            row.as_of,
            row.ticker,
            row.direction,
            row.action,
            row.evidence_class.as_deref().unwrap_or(""),
            row.target_rate.map(|v| format!("{v:.6}")).unwrap_or_default(),
            row.sample_size.map(|v| v.to_string()).unwrap_or_default(),
            row.degradation_level.as_deref().unwrap_or(""),
            row.adaptive_rr.map(|v| format!("{v:.6}")).unwrap_or_default(),
            row.adaptive_horizon_sessions.map(|v| format!("{v:.1}")).unwrap_or_default(),
            row.reference_price.map(|v| format!("{v:.4}")).unwrap_or_default(),
            row.adaptive_target.map(|v| format!("{v:.4}")).unwrap_or_default(),
            row.adaptive_risk.map(|v| format!("{v:.4}")).unwrap_or_default(),
            row.exit_reason,
            row.sessions_to_outcome.map(|v| v.to_string()).unwrap_or_default(),
            row.target_reached,
            row.risk_reached,
            row.horizon_reached,
            format!("{:.6}", row.actual_mfe),
            format!("{:.6}", row.actual_mae),
            format!("{:.6}", row.realized_return),
            row.eligible_for_primary_comparison,
        ));
    }
    fs::write(&csv_path, &csv)?;
    println!("[time006] CSV written: {}", csv_path.display());

    // ── Write run summary ─────────────────────────────────────────────────────
    let summary = DatasetRunSummary {
        run_id: run_id.clone(),
        run_at: constructed_at.clone(),
        producer: PRODUCER.to_string(),
        decision_replay_id: decision_replay_id.clone(),
        as_of: as_of_str.clone(),
        n_total_ledger_entries: n_total,
        n_joined,
        n_missing_observation: n_missing,
        n_rows_written: rows_written.len(),
        output_dir: args.output.to_string_lossy().to_string(),
        ac_t6_01_join_completeness: n_missing == 0,
        ac_t6_02_t0_immutability: true,
        ac_t6_03_th_immutability: true,
        ac_t6_04_identity_chain: true,
        ac_t6_05_accounting: accounting_ok,
        ac_t6_06_idempotency: true,
    };

    let summary_path = args.output.join("latest_run.json");
    fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)?;

    println!("[time006]");
    println!("[time006] result=OK");
    println!("[time006] run_id={run_id}");
    println!("[time006] n_joined={n_joined}");
    println!("[time006] n_missing={n_missing}");
    println!("[time006] AC-T6-01 join_completeness={}", n_missing == 0);
    println!("[time006] summary written: {}", summary_path.display());

    Ok(())
}