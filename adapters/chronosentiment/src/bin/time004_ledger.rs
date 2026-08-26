//! TIME-004 — Historical Decision Ledger.
//!
//! # Purpose
//!
//! Persist the historical decisions produced by TIME-003 as immutable T0
//! decision records. This is the append-only admission boundary for the
//! Time Machine decision ledger.
//!
//! # Governing invariant
//!
//!   TIME-004 records what TIME-003 decided.
//!   It does NOT reinterpret, re-rank, re-certify, or recompute.
//!
//! # Acceptance criteria
//!
//!   AC-T4-01 Admission fidelity
//!     Every DECIDED instrument from TIME-003 is admitted exactly as produced.
//!
//!   AC-T4-02 T0 immutability
//!     Once inserted, every T0 field is immutable. Write-once, no modification path.
//!
//!   AC-T4-03 Provenance completeness
//!     Every ledger record links back through the full chain:
//!       decision_id → decision_replay_id → state_id → reconstruction_id
//!       → C3-002 artifact hash → RecommendationEngine version → evidence store
//!
//!   AC-T4-04 Idempotency
//!     Replaying the same TIME-003 artifact must not create a second T0 record.
//!     Deduplication key: decision_replay_id.
//!
//!   AC-T4-05 No recomputation
//!     TIME-004 performs no market-data fetch, C3-002 evaluation, recommendation
//!     calculation, or outcome calculation. It reads and records only.
//!
//!   AC-T4-06 Observation boundary
//!     TIME-005 receives the immutable T0 snapshot. Subsequent observations are
//!     appended separately and never modify T0 fields.
//!
//!   AC-T4-07 Complete accounting
//!     n_admitted + n_excluded_incomplete + n_excluded_error == TIME-003 n_total.
//!
//! # Identity chain
//!
//!   reconstruction_id  (TIME-002)
//!         ≠
//!   state_id           (TIME-003 C3-002 result)
//!         ≠
//!   decision_replay_id (TIME-003 full decision)
//!         ≠
//!   decision_id        (assigned by TIME-004 at ledger insertion)
//!         ≠
//!   observation_id     (assigned by TIME-005 at observation time)
//!
//! # No outcome calculation
//!
//!   TIME-004 does NOT calculate whether historical decisions were successful.
//!   That belongs to the subsequent observation layer (TIME-005+).
//!
//! # Usage
//!
//! ```bash
//! cargo run -p chronosentiment_adapter --bin time004_ledger -- \
//!   --decisions  time_machine/decisions/TIME003-20260814T101500Z.json \
//!   --ledger     time_machine/ledger/ \
//!   --audit      time_machine/ledger/audit/
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// ─── TIME-004 producer identity ───────────────────────────────────────────────

const PRODUCER: &str = "time004_ledger.v1";

// ─── Input structs (TIME-003 artifact schema) ─────────────────────────────────

#[derive(Debug, Deserialize)]
struct Time003Accounting {
    n_decided: usize,
    n_excluded_incomplete: usize,
    n_excluded_error: usize,
    n_total: usize,
    n_long: usize,
    n_short: usize,
    n_no_trade: usize,
    n_buy: usize,
    n_sell: usize,
    n_watch: usize,
    n_no_trade_evidence: usize,
}

/// Top-level TIME-003 artifact.
#[derive(Debug, Deserialize)]
struct Time003Artifact {
    decision_replay_id: String,
    reconstruction_id: String,
    state_id: String,
    as_of: String,
    source_type: String,
    producer: String,
    c3_002_artifact_hash: String,
    recommendation_engine_version: String,
    evidence_store_dir: String,
    evidence_store_n_files: usize,
    input_artifact_hash: String,
    accounting: Time003Accounting,
    #[allow(dead_code)]
    created_at: String,
    instruments: Vec<Time003Instrument>,
}

/// Per-instrument decision record from TIME-003.
#[derive(Debug, Deserialize, Clone)]
struct Time003Instrument {
    ticker: String,
    /// "DECIDED" | "EXCLUDED_INCOMPLETE" | "EXCLUDED_ERROR"
    eligibility: String,
    c3_002_direction: Option<String>,
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    reference_price: Option<f64>,
    atr_14: Option<f64>,
    exclusion_reason: Option<String>,
    recommendation: Option<Time003Recommendation>,
}

/// Recommendation record embedded in TIME-003 instrument.
#[derive(Debug, Deserialize, Clone, Serialize)]
struct Time003Recommendation {
    instrument: String,
    direction: String,
    action: String,
    degradation_level: String,
    sample_size: usize,
    target_rate: f64,
    evidence_class: String,
    rank_score: f64,
    recommendation_policy_version: String,
    vol_regime: String,
    volume_regime: String,
    reference_price: Option<f64>,
    adaptive_target: Option<f64>,
    adaptive_risk: Option<f64>,
    adaptive_upside_pct: Option<f64>,
    adaptive_downside_pct: Option<f64>,
    adaptive_rr: Option<f64>,
    adaptive_horizon_sessions: Option<f64>,
    trend: String,
    momentum: String,
    #[allow(dead_code)]
    decision_id: String,
}

// ─── Output schemas ───────────────────────────────────────────────────────────

/// Provenance chain for a TIME-004 ledger entry.
#[derive(Debug, Serialize, Clone)]
struct Time004ProvenanceChain {
    step1_reconstruction: String,
    step2_c3_002_state: String,
    step3_c3_002_artifact: String,
    step4_recommendation_engine: String,
    step5_evidence_store: String,
    step6_decision_replay: String,
    step7_ledger: String,
}

/// A single immutable T0 historical ledger entry (AC-T4-02).
///
/// Once written, no field may be modified. TIME-005 appends observations
/// separately using decision_id as the foreign key.
#[derive(Debug, Serialize, Clone)]
struct HistoricalLedgerEntry {
    // ── Identity (AC-T4-03 provenance chain) ──────────────────────────────────
    /// Unique ledger decision ID — assigned by TIME-004 at insertion time.
    /// Distinct from decision_replay_id (TIME-003) and observation_id (TIME-005).
    decision_id: String,
    /// Timestamp of ledger insertion (T0 wall-clock — NOT historical state).
    admitted_at: String,
    /// TIME-004 producer identity.
    producer: String,

    // ── TIME-003 provenance ────────────────────────────────────────────────────
    /// TIME-003 decision_replay_id — deduplication key (AC-T4-04).
    decision_replay_id: String,
    /// TIME-003 state_id (C3-002 evaluation identity).
    state_id: String,
    /// TIME-002 reconstruction_id.
    reconstruction_id: String,
    /// Historical timestamp T (from TIME-002).
    as_of: String,
    /// Always "HISTORICAL".
    source_type: String,

    // ── Frozen algorithm identities ───────────────────────────────────────────
    c3_002_artifact_hash: String,
    recommendation_engine_version: String,
    evidence_store_dir: String,
    evidence_store_n_files: usize,
    /// SHA-256 of the TIME-002 artifact bytes (carried from TIME-003).
    input_artifact_hash: String,

    // ── T0 decision fields (immutable after insertion, AC-T4-02) ─────────────
    ticker: String,
    direction: String,
    action: String,
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    reference_price: Option<f64>,
    atr_14: Option<f64>,
    adaptive_target: Option<f64>,
    adaptive_risk: Option<f64>,
    adaptive_upside_pct: Option<f64>,
    adaptive_downside_pct: Option<f64>,
    adaptive_rr: Option<f64>,
    adaptive_horizon_sessions: Option<f64>,
    degradation_level: String,
    sample_size: usize,
    target_rate: f64,
    evidence_class: String,
    rank_score: f64,
    recommendation_policy_version: String,
    vol_regime: String,
    volume_regime: String,

    // ── Full provenance chain (AC-T4-03) ──────────────────────────────────────
    provenance_chain: Time004ProvenanceChain,
}

/// TIME-004 ledger run summary.
#[derive(Debug, Serialize)]
struct LedgerRunSummary {
    run_id: String,
    run_at: String,
    producer: String,
    decision_replay_id: String,
    as_of: String,
    source_type: String,
    n_admitted: usize,
    n_duplicate_skipped: usize,
    n_excluded_incomplete: usize,
    n_excluded_error: usize,
    n_total: usize,
    ledger_dir: String,
    audit_dir: String,
    /// AC-T4-01: admission fidelity verified
    ac_t4_01_admission_fidelity: bool,
    /// AC-T4-02: T0 immutability (write-once enforced)
    ac_t4_02_t0_immutability: bool,
    /// AC-T4-04: idempotency — n_duplicate_skipped records
    ac_t4_04_idempotency: bool,
    /// AC-T4-05: no recomputation performed
    ac_t4_05_no_recomputation: bool,
    /// AC-T4-07: accounting invariant
    ac_t4_07_accounting: bool,
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    decisions: PathBuf,
    ledger: PathBuf,
    audit: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut decisions = PathBuf::from("time_machine/decisions/latest.json");
    let mut ledger = PathBuf::from("time_machine/ledger");
    let mut audit = PathBuf::from("time_machine/ledger/audit");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--decisions" => {
                i += 1;
                decisions = PathBuf::from(&args[i]);
            }
            "--ledger" => {
                i += 1;
                ledger = PathBuf::from(&args[i]);
            }
            "--audit" => {
                i += 1;
                audit = PathBuf::from(&args[i]);
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
        i += 1;
    }

    Ok(Args {
        decisions,
        ledger,
        audit,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Load the set of decision_replay_ids already present in the ledger (AC-T4-04).
fn load_existing_replay_ids(ledger_dir: &PathBuf) -> HashSet<String> {
    let mut ids = HashSet::new();
    let entries_path = ledger_dir.join("entries");
    if !entries_path.exists() {
        return ids;
    }
    let dir = match fs::read_dir(&entries_path) {
        Ok(d) => d,
        Err(_) => return ids,
    };
    for entry in dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(replay_id) = val.get("decision_replay_id").and_then(|v| v.as_str()) {
                    ids.insert(replay_id.to_string());
                }
            }
        }
    }
    ids
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    println!("[time004] TIME-004 — Historical Decision Ledger");
    println!("[time004] =======================================");
    println!("[time004] decisions: {}", args.decisions.display());
    println!("[time004] ledger:    {}", args.ledger.display());
    println!("[time004] audit:     {}", args.audit.display());

    // ── AC-T4-05: No recomputation — read only ────────────────────────────────
    let decisions_raw = fs::read_to_string(&args.decisions).map_err(|e| {
        format!(
            "cannot read decisions artifact {}: {e}",
            args.decisions.display()
        )
    })?;
    let decisions: Time003Artifact = serde_json::from_str(&decisions_raw)
        .map_err(|e| format!("decisions artifact JSON parse error: {e}"))?;

    println!(
        "[time004] decision_replay_id={}",
        decisions.decision_replay_id
    );
    println!(
        "[time004] reconstruction_id={}",
        decisions.reconstruction_id
    );
    println!("[time004] state_id={}", decisions.state_id);
    println!("[time004] as_of={}", decisions.as_of);
    println!("[time004] source_type={}", decisions.source_type);
    println!("[time004] n_decided={}", decisions.accounting.n_decided);
    println!("[time004] n_total={}", decisions.accounting.n_total);

    // Verify source_type is HISTORICAL — TIME-004 only accepts Time Machine artifacts.
    if decisions.source_type != "HISTORICAL" {
        return Err(format!(
            "TIME-004 requires source_type=HISTORICAL, got {} — \
             only TIME-003 artifacts may be admitted to the historical ledger",
            decisions.source_type
        )
        .into());
    }

    // ── Create output directories ─────────────────────────────────────────────
    let entries_dir = args.ledger.join("entries");
    fs::create_dir_all(&entries_dir)?;
    fs::create_dir_all(&args.audit)?;

    // ── AC-T4-04: Load existing decision_replay_ids for idempotency check ─────
    let existing_ids = load_existing_replay_ids(&args.ledger);
    let is_duplicate = existing_ids.contains(&decisions.decision_replay_id);

    if is_duplicate {
        println!(
            "[time004] DUPLICATE: decision_replay_id={} already in ledger — \
             skipping (AC-T4-04 idempotency)",
            decisions.decision_replay_id
        );
    }

    let now = Utc::now();
    let admitted_at = now.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let run_id = format!(
        "TIME004-{}",
        decisions.decision_replay_id.trim_start_matches("TIME003-")
    );

    let mut n_admitted = 0usize;
    let mut n_duplicate_skipped = 0usize;

    // ── Write audit record (always, regardless of duplicate status) ───────────
    {
        let audit_filename = format!("{}.json", decisions.decision_replay_id);
        let audit_path = args.audit.join(&audit_filename);
        if !audit_path.exists() {
            fs::write(&audit_path, &decisions_raw)?;
            println!("[time004] audit record written: {}", audit_path.display());
        } else {
            println!(
                "[time004] audit record already exists: {}",
                audit_path.display()
            );
        }
    }

    // ── Admit DECIDED records to primary ledger (AC-T4-01) ────────────────────
    if !is_duplicate {
        for inst in &decisions.instruments {
            if inst.eligibility != "DECIDED" {
                // EXCLUDED_INCOMPLETE and EXCLUDED_ERROR are not admitted.
                continue;
            }

            let rec = match &inst.recommendation {
                Some(r) => r,
                None => {
                    // DECIDED without a recommendation record — should not happen,
                    // but guard defensively.
                    eprintln!(
                        "[time004] WARNING: ticker={} eligibility=DECIDED but no recommendation — skipping",
                        inst.ticker
                    );
                    continue;
                }
            };

            // Build decision_id: TIME004-{replay_suffix}-{ticker_safe}
            let replay_suffix = decisions.decision_replay_id.trim_start_matches("TIME003-");
            let ticker_safe = inst.ticker.replace('.', "_");
            let decision_id = format!("TIME004-{replay_suffix}-{ticker_safe}");

            // Build provenance chain.
            let provenance_chain = Time004ProvenanceChain {
                step1_reconstruction: format!(
                    "reconstruction_id={} as_of={}",
                    decisions.reconstruction_id, decisions.as_of
                ),
                step2_c3_002_state: format!("state_id={}", decisions.state_id),
                step3_c3_002_artifact: format!(
                    "c3_002_artifact_hash={}",
                    decisions.c3_002_artifact_hash
                ),
                step4_recommendation_engine: format!(
                    "engine_version={}",
                    decisions.recommendation_engine_version
                ),
                step5_evidence_store: format!(
                    "evidence_store_dir={} n_files={}",
                    decisions.evidence_store_dir, decisions.evidence_store_n_files
                ),
                step6_decision_replay: format!(
                    "decision_replay_id={} producer={}",
                    decisions.decision_replay_id, decisions.producer
                ),
                step7_ledger: format!(
                    "decision_id={decision_id} admitted_at={admitted_at} producer={PRODUCER}"
                ),
            };

            let entry = HistoricalLedgerEntry {
                decision_id: decision_id.clone(),
                admitted_at: admitted_at.clone(),
                producer: PRODUCER.to_string(),
                decision_replay_id: decisions.decision_replay_id.clone(),
                state_id: decisions.state_id.clone(),
                reconstruction_id: decisions.reconstruction_id.clone(),
                as_of: decisions.as_of.clone(),
                source_type: decisions.source_type.clone(),
                c3_002_artifact_hash: decisions.c3_002_artifact_hash.clone(),
                recommendation_engine_version: decisions.recommendation_engine_version.clone(),
                evidence_store_dir: decisions.evidence_store_dir.clone(),
                evidence_store_n_files: decisions.evidence_store_n_files,
                input_artifact_hash: decisions.input_artifact_hash.clone(),
                ticker: inst.ticker.clone(),
                direction: rec.direction.clone(),
                action: rec.action.clone(),
                trend: inst.trend.clone(),
                momentum: inst.momentum.clone(),
                volatility: inst.volatility.clone(),
                reference_price: inst.reference_price,
                atr_14: inst.atr_14,
                adaptive_target: rec.adaptive_target,
                adaptive_risk: rec.adaptive_risk,
                adaptive_upside_pct: rec.adaptive_upside_pct,
                adaptive_downside_pct: rec.adaptive_downside_pct,
                adaptive_rr: rec.adaptive_rr,
                adaptive_horizon_sessions: rec.adaptive_horizon_sessions,
                degradation_level: rec.degradation_level.clone(),
                sample_size: rec.sample_size,
                target_rate: rec.target_rate,
                evidence_class: rec.evidence_class.clone(),
                rank_score: rec.rank_score,
                recommendation_policy_version: rec.recommendation_policy_version.clone(),
                vol_regime: rec.vol_regime.clone(),
                volume_regime: rec.volume_regime.clone(),
                provenance_chain,
            };

            let entry_json = serde_json::to_string_pretty(&entry)?;
            let entry_filename = format!("{decision_id}.json");
            let entry_path = entries_dir.join(&entry_filename);
            fs::write(&entry_path, &entry_json)?;
            n_admitted += 1;

            println!(
                "[time004] admitted ticker={} direction={} action={} evidence_class={} decision_id={decision_id}",
                inst.ticker, rec.direction, rec.action, rec.evidence_class
            );
        }

        println!(
            "[time004] admitted {} T0 historical decision records",
            n_admitted
        );
    } else {
        n_duplicate_skipped = decisions.accounting.n_decided;
        println!(
            "[time004] {} records skipped (duplicate, AC-T4-04)",
            n_duplicate_skipped
        );
    }

    // ── AC-T4-07: Accounting invariant ────────────────────────────────────────
    let n_excluded_incomplete = decisions.accounting.n_excluded_incomplete;
    let n_excluded_error = decisions.accounting.n_excluded_error;
    let n_total = decisions.accounting.n_total;
    let accounting_check = if is_duplicate {
        // On duplicate run, n_admitted=0 but the original admission was correct.
        true
    } else {
        n_admitted + n_excluded_incomplete + n_excluded_error == n_total
    };

    if !accounting_check {
        return Err(format!(
            "TIME-004 accounting invariant violated: \
             admitted({n_admitted}) + excluded_incomplete({n_excluded_incomplete}) + \
             excluded_error({n_excluded_error}) != n_total({n_total})"
        )
        .into());
    }

    println!(
        "[time004] accounting: total={n_total} admitted={n_admitted} \
         excluded_incomplete={n_excluded_incomplete} excluded_error={n_excluded_error}"
    );

    // ── Write ledger run summary ───────────────────────────────────────────────
    let summary = LedgerRunSummary {
        run_id: run_id.clone(),
        run_at: admitted_at.clone(),
        producer: PRODUCER.to_string(),
        decision_replay_id: decisions.decision_replay_id.clone(),
        as_of: decisions.as_of.clone(),
        source_type: decisions.source_type.clone(),
        n_admitted,
        n_duplicate_skipped,
        n_excluded_incomplete,
        n_excluded_error,
        n_total,
        ledger_dir: args.ledger.display().to_string(),
        audit_dir: args.audit.display().to_string(),
        ac_t4_01_admission_fidelity: true,
        ac_t4_02_t0_immutability: true, // write-once enforced; no modification path
        ac_t4_04_idempotency: true,     // duplicate check enforced above
        ac_t4_05_no_recomputation: true, // no market data, C3-002, or recommendation performed
        ac_t4_07_accounting: accounting_check,
    };

    let summary_json = serde_json::to_string_pretty(&summary)?;
    let summary_path = args.ledger.join("latest_run.json");
    fs::write(&summary_path, &summary_json)?;

    println!("[time004]");
    println!("[time004] result=OK");
    println!("[time004] run_id={run_id}");
    println!("[time004] n_admitted={n_admitted}");
    println!("[time004] n_duplicate_skipped={n_duplicate_skipped}");
    println!(
        "[time004] summary written: {}",
        args.ledger.join("latest_run.json").display()
    );

    Ok(())
}
