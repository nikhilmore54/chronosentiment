//! LIVE-004 — Freshness + Provenance Certification
//!
//! Pure certification boundary. Reads LIVE-001 snapshot + LIVE-002 state +
//! LIVE-003 recommendation artifacts and produces a certification status.
//!
//! ## Invariants
//!
//!   - NO data fetching
//!   - NO recalculation
//!   - NO modification of any recommendation
//!   - Deterministic: same artifacts → same certification always
//!
//! ## Certification gates (all must pass for CERTIFIED)
//!
//!   1. Freshness        — snapshot age <= threshold (configurable, default 30 min)
//!   2. Snapshot coherence — all C3-002 inputs from same LIVE-001 boundary
//!   3. Completeness     — required inputs exist for evaluated instruments
//!   4. Recommendation inputs — no required engine input silently substituted
//!   5. Reproducibility  — LIVE-003 source_state_id matches LIVE-002 state_id
//!   6. Frozen artifacts — C3-002 hash + engine version match frozen identities
//!
//! ## Statuses (priority order)
//!
//!   CERTIFIED   — all gates pass
//!   DEGRADED    — ≥1 required input substituted (e.g. relative_volume_20 = 1.0)
//!   STALE       — snapshot age > freshness threshold
//!   INCOMPLETE  — C3-002 inputs not from same coherent boundary, or completeness failure
//!
//! ## input_integrity vs degradation_level
//!
//!   These are SEPARATE concepts and must never be conflated:
//!
//!   degradation_level (from LIVE-003 RecommendationRecordV1):
//!     Exact / RelaxVolume / RelaxBoth / StateOnly / Insufficient
//!     → answers: "How strong was the historical analogue evidence?"
//!
//!   input_integrity (produced by LIVE-004):
//!     AVAILABLE / SUBSTITUTED / MISSING per input
//!     → answers: "How faithfully did the live pipeline reproduce the engine inputs?"
//!
//! ## Usage
//!
//!   live004_certify \
//!     --snapshot    live_capture/snapshots/latest.json \
//!     --state       live_capture/evaluations/latest.json \
//!     --recommend   live_capture/recommendations/latest.json \
//!     --output      live_capture/certifications/ \
//!     --freshness   30

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// ─── Frozen identity constants ─────────────────────────────────────────────────

/// Frozen C3-002 artifact hash — must match what LIVE-002 and LIVE-003 recorded.
const FROZEN_C3_002_HASH: &str =
    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121";

/// Frozen recommendation engine version.
const FROZEN_ENGINE_VERSION: &str = "v1";

/// Default freshness threshold in minutes.
const DEFAULT_FRESHNESS_MINUTES: i64 = 30;

/// Neutral substitute value used when relative_volume_20 is unavailable.
const RELATIVE_VOLUME_NEUTRAL: f64 = 1.0;

// ─── LIVE-004 producer identity ───────────────────────────────────────────────

const PRODUCER: &str = "live004_certify.v1";

// ─── Input artifact schemas (read-only) ───────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SnapshotInstrument {
    ticker: String,
    #[allow(dead_code)]
    trend: Option<String>,
    #[allow(dead_code)]
    momentum: Option<String>,
    #[allow(dead_code)]
    volatility: Option<String>,
    tmv_complete: bool,
    completeness_status: String,
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
    instruments: Vec<SnapshotInstrument>,
}

#[derive(Debug, Deserialize)]
struct StateInstrument {
    ticker: String,
    eligibility: String,
    #[allow(dead_code)]
    c3_002_direction: Option<String>,
    #[allow(dead_code)]
    trend: Option<String>,
    #[allow(dead_code)]
    momentum: Option<String>,
    #[allow(dead_code)]
    volatility: Option<String>,
    #[allow(dead_code)]
    reference_price: Option<f64>,
    #[allow(dead_code)]
    atr_14: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Live002StateArtifact {
    state_id: String,
    evaluated_at: String,
    c3_002_artifact_hash: String,
    source_snapshot_id: String,
    source_snapshot_timestamp: String,
    source_type: String,
    n_input: usize,
    n_evaluated: usize,
    n_excluded_incomplete: usize,
    n_excluded_error: usize,
    n_long: usize,
    n_short: usize,
    n_no_trade: usize,
    instruments: Vec<StateInstrument>,
}

#[derive(Debug, Deserialize)]
struct RecommendRecord {
    #[allow(dead_code)]
    instrument: String,
    #[allow(dead_code)]
    direction: String,
    #[allow(dead_code)]
    action: String,
    degradation_level: String,
    #[allow(dead_code)]
    rank_score: f64,
    #[allow(dead_code)]
    recommendation_policy_version: String,
    #[allow(dead_code)]
    vol_regime: String,
    volume_regime: String,
}

#[derive(Debug, Deserialize)]
struct Live003RecommendArtifact {
    recommendation_id: String,
    recommended_at: String,
    source_state_id: String,
    source_snapshot_id: String,
    source_snapshot_timestamp: String,
    source_type: String,
    c3_002_artifact_hash: String,
    engine_version: String,
    evidence_store_dir: String,
    evidence_store_n_files: usize,
    n_input_from_live002: usize,
    n_evaluated_from_live002: usize,
    n_excluded_incomplete_from_live002: usize,
    n_excluded_error_from_live002: usize,
    n_long_from_live002: usize,
    n_short_from_live002: usize,
    n_no_trade_from_live002: usize,
    n_recommended: usize,
    n_skipped_no_trade: usize,
    n_skipped_excluded: usize,
    n_buy: usize,
    n_sell: usize,
    n_watch: usize,
    n_no_trade_evidence: usize,
    recommendations: Vec<RecommendRecord>,
}

// ─── Output artifact schema ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct CertificationArtifact {
    certification_id: String,
    certified_at: String,
    producer: String,

    // Provenance chain
    source_snapshot_id: String,
    source_snapshot_timestamp: String,
    source_state_id: String,
    source_state_evaluated_at: String,
    source_recommendation_id: String,
    source_recommended_at: String,
    source_type: String,
    c3_002_artifact_hash: String,
    engine_version: String,
    evidence_store_dir: String,
    evidence_store_n_files: usize,

    // Overall certification status
    certification_status: String,

    // Gate results (all six)
    gates: CertificationGates,

    // Input integrity — per-input substitution tracking (SEPARATE from degradation_level)
    input_integrity: InputIntegrity,

    // Snapshot freshness detail
    snapshot_age_minutes: i64,
    freshness_threshold_minutes: i64,

    // Pass-through counts from LIVE-003 (not recalculated)
    n_recommended: usize,
    n_skipped_no_trade: usize,
    n_skipped_excluded: usize,
    n_buy: usize,
    n_sell: usize,
    n_watch: usize,
    n_no_trade_evidence: usize,

    // Pass-through counts from LIVE-002 (not recalculated)
    n_input_from_live002: usize,
    n_evaluated_from_live002: usize,
    n_excluded_incomplete_from_live002: usize,
    n_excluded_error_from_live002: usize,
    n_long_from_live002: usize,
    n_short_from_live002: usize,
    n_no_trade_from_live002: usize,

    // Provenance chain narrative
    provenance_chain: ProvenanceChain,
}

#[derive(Debug, Serialize)]
struct CertificationGates {
    freshness: GateResult,
    snapshot_coherence: GateResult,
    completeness: GateResult,
    recommendation_inputs: GateResult,
    reproducibility: GateResult,
    frozen_artifacts: GateResult,
}

#[derive(Debug, Serialize)]
struct GateResult {
    pass: bool,
    detail: String,
}

/// Input integrity — SEPARATE from degradation_level.
///
/// degradation_level answers: "How strong was the historical analogue evidence?"
/// input_integrity answers:   "How faithfully did the live pipeline reproduce engine inputs?"
#[derive(Debug, Serialize)]
struct InputIntegrity {
    relative_volume_20: InputField,
}

#[derive(Debug, Serialize)]
struct InputField {
    /// AVAILABLE | SUBSTITUTED | MISSING
    status: String,
    /// The value used (neutral substitute if SUBSTITUTED)
    value: f64,
    /// Why this status was assigned
    reason: String,
}

#[derive(Debug, Serialize)]
struct ProvenanceChain {
    step1_market_snapshot: String,
    step2_c3_002_state: String,
    step3_c3_002_artifact: String,
    step4_recommendation_engine: String,
    step5_evidence_store: String,
    step6_recommendation: String,
    step7_certification: String,
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    snapshot: PathBuf,
    state: PathBuf,
    recommend: PathBuf,
    output: PathBuf,
    freshness_minutes: i64,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut snapshot = PathBuf::from("live_capture/snapshots/latest.json");
    let mut state = PathBuf::from("live_capture/evaluations/latest.json");
    let mut recommend = PathBuf::from("live_capture/recommendations/latest.json");
    let mut output = PathBuf::from("live_capture/certifications");
    let mut freshness_minutes = DEFAULT_FRESHNESS_MINUTES;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--snapshot" => { i += 1; snapshot = PathBuf::from(&args[i]); }
            "--state"    => { i += 1; state = PathBuf::from(&args[i]); }
            "--recommend"=> { i += 1; recommend = PathBuf::from(&args[i]); }
            "--output"   => { i += 1; output = PathBuf::from(&args[i]); }
            "--freshness"=> {
                i += 1;
                freshness_minutes = args[i].parse::<i64>()
                    .map_err(|_| format!("--freshness must be an integer, got '{}'", args[i]))?;
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
        i += 1;
    }

    Ok(Args { snapshot, state, recommend, output, freshness_minutes })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn read_json<T: for<'de> Deserialize<'de>>(path: &PathBuf) -> Result<T, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("cannot parse {}: {e}", path.display()))
        .map_err(|e| e.into())
}

fn parse_utc(s: &str) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("cannot parse timestamp '{}': {e}", s).into())
}

// ─── Gate evaluators ──────────────────────────────────────────────────────────

/// Gate 1: Freshness — snapshot age <= threshold.
fn gate_freshness(
    snapshot: &LiveSnapshot,
    now: &DateTime<Utc>,
    threshold_minutes: i64,
) -> (GateResult, i64) {
    let captured = match parse_utc(&snapshot.snapshot_timestamp) {
        Ok(t) => t,
        Err(e) => {
            return (GateResult {
                pass: false,
                detail: format!("cannot parse snapshot_timestamp: {e}"),
            }, i64::MAX);
        }
    };
    let age_minutes = now.signed_duration_since(captured).num_minutes();
    let pass = age_minutes <= threshold_minutes;
    let detail = if pass {
        format!(
            "snapshot age {}min <= threshold {}min — FRESH",
            age_minutes, threshold_minutes
        )
    } else {
        format!(
            "snapshot age {}min > threshold {}min — STALE",
            age_minutes, threshold_minutes
        )
    };
    (GateResult { pass, detail }, age_minutes)
}

/// Gate 2: Snapshot coherence — all three artifacts reference the same LIVE-001 snapshot_id.
fn gate_snapshot_coherence(
    snapshot: &LiveSnapshot,
    state: &Live002StateArtifact,
    recommend: &Live003RecommendArtifact,
) -> GateResult {
    let snap_id = &snapshot.snapshot_id;
    let state_ref = &state.source_snapshot_id;
    let rec_ref = &recommend.source_snapshot_id;

    let pass = snap_id == state_ref && snap_id == rec_ref;
    let detail = if pass {
        format!(
            "all artifacts reference snapshot_id={} — COHERENT",
            snap_id
        )
    } else {
        format!(
            "snapshot_id mismatch: snapshot={} state.source={} recommend.source={} — INCOHERENT",
            snap_id, state_ref, rec_ref
        )
    };
    GateResult { pass, detail }
}

/// Gate 3: Completeness — every tmv_complete instrument in snapshot appears in state.
fn gate_completeness(
    snapshot: &LiveSnapshot,
    state: &Live002StateArtifact,
) -> GateResult {
    let state_tickers: HashSet<&str> = state
        .instruments
        .iter()
        .map(|i| i.ticker.as_str())
        .collect();

    let missing: Vec<&str> = snapshot
        .instruments
        .iter()
        .filter(|i| i.tmv_complete && i.completeness_status != "ERROR")
        .map(|i| i.ticker.as_str())
        .filter(|t| !state_tickers.contains(t))
        .collect();

    let pass = missing.is_empty();
    let detail = if pass {
        format!(
            "all {} tmv_complete snapshot instruments present in state artifact — COMPLETE",
            snapshot.n_complete
        )
    } else {
        format!(
            "{} tmv_complete instruments missing from state artifact: {:?} — INCOMPLETE",
            missing.len(), missing
        )
    };
    GateResult { pass, detail }
}

/// Gate 4: Recommendation inputs — detect silent substitution of required engine inputs.
///
/// relative_volume_20 is the only input currently substituted (neutral value 1.0).
/// This is SEPARATE from degradation_level (which measures evidence quality).
fn gate_recommendation_inputs(
    recommend: &Live003RecommendArtifact,
) -> (GateResult, InputIntegrity) {
    // Detect whether all recommendations used the neutral volume substitute.
    // VolumeRegime::Normal corresponds to relative_volume_20 = 1.0.
    // If every recommendation has volume_regime = "Normal", the neutral was used.
    let all_normal_volume = recommend
        .recommendations
        .iter()
        .all(|r| r.volume_regime == "Normal");

    let (status, reason) = if all_normal_volume {
        (
            "SUBSTITUTED".to_string(),
            "relative_volume_20 not available in LIVE-002 state artifact; neutral value 1.0 used for all instruments".to_string(),
        )
    } else {
        (
            "AVAILABLE".to_string(),
            "live relative_volume_20 values present in recommendation inputs".to_string(),
        )
    };

    let input_integrity = InputIntegrity {
        relative_volume_20: InputField {
            status: status.clone(),
            value: RELATIVE_VOLUME_NEUTRAL,
            reason: reason.clone(),
        },
    };

    let pass = status == "AVAILABLE";
    let detail = if pass {
        "all required RecommendationEngine inputs AVAILABLE — no substitution detected".to_string()
    } else {
        format!(
            "relative_volume_20 SUBSTITUTED with neutral value {} for all {} recommendations — engine input not fully available from live pipeline",
            RELATIVE_VOLUME_NEUTRAL,
            recommend.recommendations.len()
        )
    };

    (GateResult { pass, detail }, input_integrity)
}

/// Gate 5: Reproducibility — LIVE-003 source_state_id matches LIVE-002 state_id.
fn gate_reproducibility(
    state: &Live002StateArtifact,
    recommend: &Live003RecommendArtifact,
) -> GateResult {
    let state_id = &state.state_id;
    let rec_state_ref = &recommend.source_state_id;

    let pass = state_id == rec_state_ref;
    let detail = if pass {
        format!(
            "LIVE-003 source_state_id={} matches LIVE-002 state_id — REPRODUCIBLE",
            state_id
        )
    } else {
        format!(
            "state_id mismatch: state.state_id={} recommend.source_state_id={} — NOT REPRODUCIBLE",
            state_id, rec_state_ref
        )
    };
    GateResult { pass, detail }
}

/// Gate 6: Frozen artifacts — C3-002 hash + engine version match frozen identities.
fn gate_frozen_artifacts(
    state: &Live002StateArtifact,
    recommend: &Live003RecommendArtifact,
) -> GateResult {
    let state_hash_ok = state.c3_002_artifact_hash == FROZEN_C3_002_HASH;
    let rec_hash_ok = recommend.c3_002_artifact_hash == FROZEN_C3_002_HASH;
    let engine_ok = recommend.engine_version == FROZEN_ENGINE_VERSION;

    let pass = state_hash_ok && rec_hash_ok && engine_ok;
    let detail = if pass {
        format!(
            "C3-002 hash={} (MATCH) engine_version={} (MATCH) — FROZEN IDENTITIES VERIFIED",
            FROZEN_C3_002_HASH, FROZEN_ENGINE_VERSION
        )
    } else {
        let mut issues = Vec::new();
        if !state_hash_ok {
            issues.push(format!(
                "state.c3_002_artifact_hash={} expected={}",
                state.c3_002_artifact_hash, FROZEN_C3_002_HASH
            ));
        }
        if !rec_hash_ok {
            issues.push(format!(
                "recommend.c3_002_artifact_hash={} expected={}",
                recommend.c3_002_artifact_hash, FROZEN_C3_002_HASH
            ));
        }
        if !engine_ok {
            issues.push(format!(
                "engine_version={} expected={}",
                recommend.engine_version, FROZEN_ENGINE_VERSION
            ));
        }
        format!("FROZEN IDENTITY MISMATCH: {}", issues.join("; "))
    };
    GateResult { pass, detail }
}

// ─── Certification status derivation ──────────────────────────────────────────
//
// Priority order: INCOMPLETE > STALE > DEGRADED > CERTIFIED
//
// INCOMPLETE takes priority because a coherence or completeness failure means
// the pipeline itself is broken — freshness and degradation are secondary.
//
// STALE takes priority over DEGRADED because a stale recommendation should
// not be acted upon regardless of input quality.
//
// DEGRADED means the pipeline ran correctly but with a documented substitution.

fn derive_status(gates: &CertificationGates, input_integrity: &InputIntegrity) -> String {
    if !gates.snapshot_coherence.pass || !gates.completeness.pass || !gates.frozen_artifacts.pass {
        return "INCOMPLETE".to_string();
    }
    if !gates.freshness.pass {
        return "STALE".to_string();
    }
    if !gates.reproducibility.pass {
        return "INCOMPLETE".to_string();
    }
    if input_integrity.relative_volume_20.status == "SUBSTITUTED" {
        return "DEGRADED".to_string();
    }
    "CERTIFIED".to_string()
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    println!("[live004] LIVE-004 — Freshness + Provenance Certification");
    println!("[live004] ================================================");
    println!("[live004] snapshot:    {}", args.snapshot.display());
    println!("[live004] state:       {}", args.state.display());
    println!("[live004] recommend:   {}", args.recommend.display());
    println!("[live004] freshness:   {}min threshold", args.freshness_minutes);

    // ── Read all three input artifacts (read-only — no modification) ──────────
    let snapshot: LiveSnapshot = read_json(&args.snapshot)?;
    let state: Live002StateArtifact = read_json(&args.state)?;
    let recommend: Live003RecommendArtifact = read_json(&args.recommend)?;

    println!("[live004] snapshot_id={}", snapshot.snapshot_id);
    println!("[live004] state_id={}", state.state_id);
    println!("[live004] recommendation_id={}", recommend.recommendation_id);

    let now: DateTime<Utc> = Utc::now();
    let certified_at = now.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let certification_id = format!(
        "LIVE-004-{}",
        recommend.recommendation_id.trim_start_matches("LIVE-003-")
    );

    // ── Evaluate all six gates ────────────────────────────────────────────────
    println!("[live004]");
    println!("[live004] Evaluating certification gates...");

    let (gate_freshness, age_minutes) = gate_freshness(&snapshot, &now, args.freshness_minutes);
    println!(
        "[live004]   [{}] Gate 1 Freshness: {}",
        if gate_freshness.pass { "PASS" } else { "FAIL" },
        gate_freshness.detail
    );

    let gate_coherence = gate_snapshot_coherence(&snapshot, &state, &recommend);
    println!(
        "[live004]   [{}] Gate 2 Snapshot coherence: {}",
        if gate_coherence.pass { "PASS" } else { "FAIL" },
        gate_coherence.detail
    );

    let gate_complete = gate_completeness(&snapshot, &state);
    println!(
        "[live004]   [{}] Gate 3 Completeness: {}",
        if gate_complete.pass { "PASS" } else { "FAIL" },
        gate_complete.detail
    );

    let (gate_inputs, input_integrity) = gate_recommendation_inputs(&recommend);
    println!(
        "[live004]   [{}] Gate 4 Recommendation inputs: {}",
        if gate_inputs.pass { "PASS" } else { "FAIL" },
        gate_inputs.detail
    );
    println!(
        "[live004]        input_integrity.relative_volume_20: status={} value={} reason={}",
        input_integrity.relative_volume_20.status,
        input_integrity.relative_volume_20.value,
        input_integrity.relative_volume_20.reason
    );

    let gate_repro = gate_reproducibility(&state, &recommend);
    println!(
        "[live004]   [{}] Gate 5 Reproducibility: {}",
        if gate_repro.pass { "PASS" } else { "FAIL" },
        gate_repro.detail
    );

    let gate_frozen = gate_frozen_artifacts(&state, &recommend);
    println!(
        "[live004]   [{}] Gate 6 Frozen artifacts: {}",
        if gate_frozen.pass { "PASS" } else { "FAIL" },
        gate_frozen.detail
    );

    let gates = CertificationGates {
        freshness: gate_freshness,
        snapshot_coherence: gate_coherence,
        completeness: gate_complete,
        recommendation_inputs: gate_inputs,
        reproducibility: gate_repro,
        frozen_artifacts: gate_frozen,
    };

    let certification_status = derive_status(&gates, &input_integrity);

    println!("[live004]");
    println!("[live004] certification_status={certification_status}");

    // ── Build provenance chain narrative ──────────────────────────────────────
    let provenance_chain = ProvenanceChain {
        step1_market_snapshot: format!(
            "snapshot_id={} captured_at={} source_type={}",
            snapshot.snapshot_id, snapshot.snapshot_timestamp, snapshot.source_type
        ),
        step2_c3_002_state: format!(
            "state_id={} evaluated_at={} source_snapshot_id={}",
            state.state_id, state.evaluated_at, state.source_snapshot_id
        ),
        step3_c3_002_artifact: format!(
            "c3_002_artifact_hash={} (frozen={})",
            state.c3_002_artifact_hash, FROZEN_C3_002_HASH
        ),
        step4_recommendation_engine: format!(
            "engine_version={} (frozen={})",
            recommend.engine_version, FROZEN_ENGINE_VERSION
        ),
        step5_evidence_store: format!(
            "evidence_store_dir={} evidence_store_n_files={}",
            recommend.evidence_store_dir, recommend.evidence_store_n_files
        ),
        step6_recommendation: format!(
            "recommendation_id={} recommended_at={} n_recommended={} n_buy={} n_sell={} n_watch={} n_no_trade_evidence={}",
            recommend.recommendation_id, recommend.recommended_at,
            recommend.n_recommended, recommend.n_buy, recommend.n_sell,
            recommend.n_watch, recommend.n_no_trade_evidence
        ),
        step7_certification: format!(
            "certification_id={} certified_at={} status={}",
            certification_id, certified_at, certification_status
        ),
    };

    // ── Build output artifact ─────────────────────────────────────────────────
    let artifact = CertificationArtifact {
        certification_id: certification_id.clone(),
        certified_at: certified_at.clone(),
        producer: PRODUCER.to_string(),
        source_snapshot_id: snapshot.snapshot_id.clone(),
        source_snapshot_timestamp: snapshot.snapshot_timestamp.clone(),
        source_state_id: state.state_id.clone(),
        source_state_evaluated_at: state.evaluated_at.clone(),
        source_recommendation_id: recommend.recommendation_id.clone(),
        source_recommended_at: recommend.recommended_at.clone(),
        source_type: snapshot.source_type.clone(),
        c3_002_artifact_hash: state.c3_002_artifact_hash.clone(),
        engine_version: recommend.engine_version.clone(),
        evidence_store_dir: recommend.evidence_store_dir.clone(),
        evidence_store_n_files: recommend.evidence_store_n_files,
        certification_status: certification_status.clone(),
        gates,
        input_integrity,
        snapshot_age_minutes: age_minutes,
        freshness_threshold_minutes: args.freshness_minutes,
        n_recommended: recommend.n_recommended,
        n_skipped_no_trade: recommend.n_skipped_no_trade,
        n_skipped_excluded: recommend.n_skipped_excluded,
        n_buy: recommend.n_buy,
        n_sell: recommend.n_sell,
        n_watch: recommend.n_watch,
        n_no_trade_evidence: recommend.n_no_trade_evidence,
        n_input_from_live002: recommend.n_input_from_live002,
        n_evaluated_from_live002: recommend.n_evaluated_from_live002,
        n_excluded_incomplete_from_live002: recommend.n_excluded_incomplete_from_live002,
        n_excluded_error_from_live002: recommend.n_excluded_error_from_live002,
        n_long_from_live002: recommend.n_long_from_live002,
        n_short_from_live002: recommend.n_short_from_live002,
        n_no_trade_from_live002: recommend.n_no_trade_from_live002,
        provenance_chain,
    };

    // ── Write output artifact ─────────────────────────────────────────────────
    fs::create_dir_all(&args.output)?;
    let artifact_filename = format!("{certification_id}.json");
    let artifact_path = args.output.join(&artifact_filename);
    let latest_path = args.output.join("latest.json");

    let json = serde_json::to_string_pretty(&artifact)?;
    fs::write(&artifact_path, &json)?;
    fs::write(&latest_path, &json)?;

    println!("[live004]");
    println!("[live004] result=OK");
    println!("[live004] certification_id={certification_id}");
    println!("[live004] certification_status={certification_status}");
    println!("[live004] snapshot_age_minutes={age_minutes}");
    println!("[live004] freshness_threshold_minutes={}", args.freshness_minutes);
    println!("[live004] artifact={}", artifact_path.display());
    println!("[live004] latest={}", latest_path.display());

    Ok(())
}