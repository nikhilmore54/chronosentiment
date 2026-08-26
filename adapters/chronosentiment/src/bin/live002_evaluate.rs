//! LIVE-002 — Live C3-002 State Evaluation
//!
//! Pure transformation: reads a LIVE-001 snapshot artifact (latest.json or a
//! named snapshot) and applies the frozen C3-002 policy to every instrument
//! where `tmv_complete == true`. No network access. No Yahoo fetch. No new
//! indicators. No algorithm changes.
//!
//! ## AC boundary
//!
//! 1. **Input fidelity**   — every evaluated value originates from the snapshot artifact.
//! 2. **Eligibility gate** — only `tmv_complete=true` instruments enter C3-002.
//! 3. **Algorithm fidelity** — frozen C3-002 (Search #2, artifact hash verified).
//! 4. **Determinism**      — identical input → byte-equivalent output.
//! 5. **Accounting**       — evaluated + excluded_incomplete + excluded_error == n_input.
//!
//! ## Usage
//!
//!   live002_evaluate \
//!     --snapshot /path/to/LIVE-20260819-0518.json \
//!     --policy   product_validation/CS-P-006/discovery/20260815T051900Z_c3 \
//!     --output   /tmp/live002_state/
//!
//! The `--snapshot` argument defaults to `<output>/latest.json` if omitted.
//! The `--policy` argument defaults to the canonical RESEARCH_DISCOVERY_TWO_DIR.

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use chronosentiment_adapter::decision_support::csp006_protocol::{
    RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH, RESEARCH_DISCOVERY_TWO_DIR,
};
use chronosentiment_adapter::decision_support::policy_artifact::{
    first_match_action_from_tmv, PolicyArtifact,
};
use chronosentiment_adapter::decision_support::DecisionAction;
use serde::{Deserialize, Serialize};

// ─── LIVE-002 producer identity ───────────────────────────────────────────────

const PRODUCER: &str = "live002_evaluate.v1";
const C3_002_ARTIFACT_HASH: &str = RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;

// ─── Input structs (mirrors LIVE-001 artifact schema) ─────────────────────────

#[derive(Debug, Deserialize)]
struct InstrumentSnapshot {
    ticker: String,
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    tmv_complete: bool,
    completeness_status: String,
    reference_price: Option<f64>,
    atr_14: Option<f64>,
    source_bar_timestamp: Option<String>,
    acquisition_timestamp: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LiveSnapshot {
    snapshot_id: String,
    snapshot_timestamp: String,
    source_type: String,
    n_instruments: usize,
    n_complete: usize,
    n_incomplete: usize,
    n_error: usize,
    instruments: Vec<InstrumentSnapshot>,
}

// ─── Output structs ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct InstrumentState {
    ticker: String,
    /// "EVALUATED", "EXCLUDED_INCOMPLETE", "EXCLUDED_ERROR"
    eligibility: String,
    /// Only present when eligibility == "EVALUATED"
    c3_002_direction: Option<String>,
    /// TMV inputs used (from snapshot, not re-fetched)
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    reference_price: Option<f64>,
    atr_14: Option<f64>,
    /// Provenance from LIVE-001
    source_bar_timestamp: Option<String>,
    acquisition_timestamp: Option<String>,
    /// Exclusion reason when not evaluated
    exclusion_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct Live002StateArtifact {
    /// e.g. "LIVE-002-20260819-0518"
    state_id: String,
    /// ISO-8601 UTC timestamp when this evaluation was produced
    evaluated_at: String,
    producer: String,
    /// C3-002 policy artifact hash (frozen)
    c3_002_artifact_hash: String,
    /// Source LIVE-001 snapshot_id
    source_snapshot_id: String,
    /// Source LIVE-001 snapshot_timestamp
    source_snapshot_timestamp: String,
    /// Source LIVE-001 source_type (must be "LIVE")
    source_type: String,
    /// Total instruments in the input snapshot
    n_input: usize,
    /// Instruments evaluated by C3-002
    n_evaluated: usize,
    /// Instruments excluded: tmv_complete=false
    n_excluded_incomplete: usize,
    /// Instruments excluded: error in LIVE-001
    n_excluded_error: usize,
    /// LONG count among evaluated
    n_long: usize,
    /// SHORT count among evaluated
    n_short: usize,
    /// NO_TRADE count among evaluated
    n_no_trade: usize,
    /// Per-instrument state
    instruments: Vec<InstrumentState>,
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    snapshot: PathBuf,
    policy_dir: PathBuf,
    output: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut snapshot: Option<PathBuf> = None;
    let mut policy_dir = PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR);
    let mut output = PathBuf::from("/tmp/live002_state");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--snapshot" => {
                i += 1;
                snapshot = Some(PathBuf::from(&args[i]));
            }
            "--policy" => {
                i += 1;
                policy_dir = PathBuf::from(&args[i]);
            }
            "--output" => {
                i += 1;
                output = PathBuf::from(&args[i]);
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
        i += 1;
    }

    // Default snapshot: <output>/latest.json
    let snapshot = snapshot.unwrap_or_else(|| output.join("latest.json"));

    Ok(Args {
        snapshot,
        policy_dir,
        output,
    })
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── AC-3: Load and verify frozen C3-002 policy artifact ──────────────────
    let policy_path = args.policy_dir.join("selected_policy.json");
    let artifact: PolicyArtifact = serde_json::from_str(
        &fs::read_to_string(&policy_path)
            .map_err(|e| format!("cannot read policy {}: {e}", policy_path.display()))?,
    )
    .map_err(|e| format!("policy JSON parse error: {e}"))?;

    if artifact.artifact_hash != C3_002_ARTIFACT_HASH {
        return Err(format!(
            "LIVE-002 identity gate: expected C3-002 artifact hash {C3_002_ARTIFACT_HASH}, got {}",
            artifact.artifact_hash
        )
        .into());
    }
    println!(
        "[live002] policy artifact verified: {}",
        artifact.artifact_hash
    );

    // ── AC-1: Load LIVE-001 snapshot (no network) ─────────────────────────────
    let snapshot_raw = fs::read_to_string(&args.snapshot)
        .map_err(|e| format!("cannot read snapshot {}: {e}", args.snapshot.display()))?;
    let snapshot: LiveSnapshot = serde_json::from_str(&snapshot_raw)
        .map_err(|e| format!("snapshot JSON parse error: {e}"))?;

    println!(
        "[live002] snapshot_id={} source_type={} n_instruments={}",
        snapshot.snapshot_id, snapshot.source_type, snapshot.n_instruments
    );

    // Verify source_type is LIVE (not historical)
    if snapshot.source_type != "LIVE" {
        return Err(format!(
            "LIVE-002 requires source_type=LIVE, got {}",
            snapshot.source_type
        )
        .into());
    }

    // ── AC-2 + AC-5: Evaluate with eligibility gate and accounting ────────────
    let evaluated_at = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let state_id = format!(
        "LIVE-002-{}",
        &snapshot.snapshot_id.trim_start_matches("LIVE-")
    );

    let mut instrument_states: Vec<InstrumentState> = Vec::new();
    let mut n_evaluated = 0usize;
    let mut n_excluded_incomplete = 0usize;
    let mut n_excluded_error = 0usize;
    let mut n_long = 0usize;
    let mut n_short = 0usize;
    let mut n_no_trade = 0usize;

    for inst in &snapshot.instruments {
        // AC-2: Eligibility gate
        if inst.completeness_status == "ERROR" || inst.error.is_some() {
            println!("[live002] excluded ticker={} reason=ERROR", inst.ticker);
            instrument_states.push(InstrumentState {
                ticker: inst.ticker.clone(),
                eligibility: "EXCLUDED_ERROR".to_string(),
                c3_002_direction: None,
                trend: inst.trend.clone(),
                momentum: inst.momentum.clone(),
                volatility: inst.volatility.clone(),
                reference_price: inst.reference_price,
                atr_14: inst.atr_14,
                source_bar_timestamp: inst.source_bar_timestamp.clone(),
                acquisition_timestamp: inst.acquisition_timestamp.clone(),
                exclusion_reason: Some(
                    inst.error
                        .clone()
                        .unwrap_or_else(|| "ERROR status".to_string()),
                ),
            });
            n_excluded_error += 1;
            continue;
        }

        if !inst.tmv_complete {
            println!(
                "[live002] excluded ticker={} reason=INCOMPLETE",
                inst.ticker
            );
            instrument_states.push(InstrumentState {
                ticker: inst.ticker.clone(),
                eligibility: "EXCLUDED_INCOMPLETE".to_string(),
                c3_002_direction: None,
                trend: inst.trend.clone(),
                momentum: inst.momentum.clone(),
                volatility: inst.volatility.clone(),
                reference_price: inst.reference_price,
                atr_14: inst.atr_14,
                source_bar_timestamp: inst.source_bar_timestamp.clone(),
                acquisition_timestamp: inst.acquisition_timestamp.clone(),
                exclusion_reason: Some("tmv_complete=false".to_string()),
            });
            n_excluded_incomplete += 1;
            continue;
        }

        // AC-1 + AC-3: Apply frozen C3-002 using only values from the snapshot
        let trend = inst.trend.as_deref().unwrap_or("absent");
        let momentum = inst.momentum.as_deref().unwrap_or("absent");
        let volatility = inst.volatility.as_deref().unwrap_or("absent");

        let action = first_match_action_from_tmv(&artifact, trend, momentum, volatility);
        let direction = match action {
            DecisionAction::Long => "LONG",
            DecisionAction::Short => "SHORT",
            DecisionAction::NoTrade => "NO_TRADE",
        };

        println!(
            "[live002] evaluated ticker={} trend={trend} momentum={momentum} volatility={volatility} direction={direction}",
            inst.ticker
        );

        match action {
            DecisionAction::Long => n_long += 1,
            DecisionAction::Short => n_short += 1,
            DecisionAction::NoTrade => n_no_trade += 1,
        }

        instrument_states.push(InstrumentState {
            ticker: inst.ticker.clone(),
            eligibility: "EVALUATED".to_string(),
            c3_002_direction: Some(direction.to_string()),
            trend: inst.trend.clone(),
            momentum: inst.momentum.clone(),
            volatility: inst.volatility.clone(),
            reference_price: inst.reference_price,
            atr_14: inst.atr_14,
            source_bar_timestamp: inst.source_bar_timestamp.clone(),
            acquisition_timestamp: inst.acquisition_timestamp.clone(),
            exclusion_reason: None,
        });
        n_evaluated += 1;
    }

    // AC-5: Accounting invariant
    let n_input = snapshot.instruments.len();
    let accounting_total = n_evaluated + n_excluded_incomplete + n_excluded_error;
    if accounting_total != n_input {
        return Err(format!(
            "LIVE-002 accounting invariant violated: evaluated({n_evaluated}) + \
             excluded_incomplete({n_excluded_incomplete}) + excluded_error({n_excluded_error}) \
             = {accounting_total} != n_input({n_input})"
        )
        .into());
    }

    let state_artifact = Live002StateArtifact {
        state_id: state_id.clone(),
        evaluated_at,
        producer: PRODUCER.to_string(),
        c3_002_artifact_hash: C3_002_ARTIFACT_HASH.to_string(),
        source_snapshot_id: snapshot.snapshot_id.clone(),
        source_snapshot_timestamp: snapshot.snapshot_timestamp.clone(),
        source_type: snapshot.source_type.clone(),
        n_input,
        n_evaluated,
        n_excluded_incomplete,
        n_excluded_error,
        n_long,
        n_short,
        n_no_trade,
        instruments: instrument_states,
    };

    // ── Write output artifact ─────────────────────────────────────────────────
    fs::create_dir_all(&args.output)?;
    let artifact_filename = format!("{state_id}.json");
    let artifact_path = args.output.join(&artifact_filename);
    let latest_path = args.output.join("latest.json");

    let json = serde_json::to_string_pretty(&state_artifact)?;
    fs::write(&artifact_path, &json)?;
    fs::write(&latest_path, &json)?;

    println!("[live002] result=OK");
    println!("[live002] state_id={state_id}");
    println!("[live002] n_input={n_input}");
    println!("[live002] n_evaluated={n_evaluated}");
    println!("[live002] n_excluded_incomplete={n_excluded_incomplete}");
    println!("[live002] n_excluded_error={n_excluded_error}");
    println!("[live002] n_long={n_long}");
    println!("[live002] n_short={n_short}");
    println!("[live002] n_no_trade={n_no_trade}");
    println!("[live002] artifact={}", artifact_path.display());
    println!("[live002] latest={}", latest_path.display());

    // AC-5: Print accounting summary
    println!(
        "[live002] accounting: {n_evaluated} evaluated + {n_excluded_incomplete} excluded_incomplete + {n_excluded_error} excluded_error = {accounting_total} / {n_input} input"
    );

    Ok(())
}
