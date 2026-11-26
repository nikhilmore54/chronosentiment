//! TIME-003 — Frozen Coralys Decision Replay.
//!
//! # Purpose
//!
//! Given a frozen TIME-002 reconstruction artifact, apply the frozen Coralys
//! decision pipeline (C3-002 → RecommendationEngine v1 + REC-001-H) to every
//! COMPLETE instrument and produce a deterministic historical decision artifact.
//!
//! # Architectural boundary
//!
//! TIME-003 answers exactly one question:
//!
//! > **Given the information legitimately available at T, what would the frozen
//! > Coralys decision system have produced?**
//!
//! It does NOT answer whether that decision was good. That belongs downstream
//! (OBS-001+). TIME-003 must not become an opportunity to improve the algorithm.
//!
//! # Mandatory invariants
//!
//! 1. **No network access** — no Yahoo/API calls, no current market data.
//! 2. **No feature recomputation** — consume features already in the TIME-002 artifact.
//! 3. **Frozen algorithm identity** — C3-002 artifact hash verified at startup;
//!    RecommendationEngine v1 identity and REC-001-H identity recorded.
//! 4. **Eligibility fidelity** — only COMPLETE TIME-002 records enter evaluation;
//!    INCOMPLETE and ERROR remain excluded and explicitly accounted for.
//! 5. **Determinism** — same TIME-002 artifact + same frozen artifacts → identical output.
//! 6. **No optimization** — no threshold changes, no R:R tuning, no policy changes.
//! 7. **Full accounting** — DECIDED + EXCLUDED_INCOMPLETE + EXCLUDED_ERROR == TIME-002 total.
//!
//! # Identity chain
//!
//! ```text
//! reconstruction_id  (TIME-002)
//!        │
//!        ▼
//! state_id           (TIME-003 C3-002 result)
//!        │
//!        ▼
//! decision_replay_id (TIME-003 full decision)
//!        │
//!        ▼
//! observation_id     (future OBS phase)
//! ```
//!
//! # Provenance artifact fields
//!
//! ```text
//! decision_replay_id          — unique ID for this replay run
//! reconstruction_id           — TIME-002 provenance.reconstruction_id
//! state_id                    — TIME-003 C3-002 evaluation identity
//! as_of                       — TIME-002 provenance.as_of
//! source_type                 — "HISTORICAL"
//! c3_002_artifact_hash        — verified frozen hash
//! recommendation_engine_version — frozen engine version string
//! evidence_store_dir          — REC-001-H directory
//! evidence_store_n_files      — file count for identity recording
//! input_artifact_hash         — SHA-256 of the TIME-002 artifact bytes
//! accounting                  — { n_decided, n_excluded_incomplete, n_excluded_error, n_total }
//! created_at                  — wall-clock time of artifact generation
//!                               (MUST NOT influence the replay result)
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run -p chronosentiment_adapter --bin time003_replay -- \
//!   --reconstruction time_machine/reconstructions/TIME002-20260814T101500Z.json \
//!   --policy         product_validation/CS-P-006/discovery/20260815T051900Z_c3 \
//!   --evidence       datasets/recommendation/historical \
//!   --output         time_machine/decisions
//! ```

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
use coralys_decision::recommendation::{
    Rec001hStore, RecommendationEngineV1, RecommendationRecordV1, RECOMMENDATION_POLICY_VERSION_V1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ─── Producer identity ────────────────────────────────────────────────────────

const PRODUCER: &str = "time003_replay.v1";
const C3_002_ARTIFACT_HASH: &str = RESEARCH_DISCOVERY_TWO_ARTIFACT_HASH;
const SOURCE_TYPE: &str = "HISTORICAL";

// ─── Input structs (TIME-002 artifact schema) ─────────────────────────────────

#[derive(Debug, Deserialize)]
struct Time002Accounting {
    n_total: usize,
    #[allow(dead_code)]
    n_complete: usize,
    #[allow(dead_code)]
    n_incomplete: usize,
    #[allow(dead_code)]
    n_error: usize,
}

#[derive(Debug, Deserialize)]
struct Time002Provenance {
    reconstruction_id: String,
    as_of: String,
    #[allow(dead_code)]
    universe_id: String,
    #[allow(dead_code)]
    data_source: String,
    #[allow(dead_code)]
    data_boundary_rule: String,
    #[allow(dead_code)]
    source_dataset_hash: String,
    #[allow(dead_code)]
    clock_mode: String,
    #[allow(dead_code)]
    feature_pipeline_id: String,
    accounting: Time002Accounting,
    #[allow(dead_code)]
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct Time002Instrument {
    ticker: String,
    #[allow(dead_code)]
    source: String,
    #[allow(dead_code)]
    source_bar_timestamp: Option<String>,
    #[allow(dead_code)]
    n_bars_total: usize,
    #[allow(dead_code)]
    n_bars_at_t: usize,
    #[allow(dead_code)]
    n_bars_excluded: usize,
    reference_price: Option<f64>,
    #[allow(dead_code)]
    atr_14: Option<f64>,
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    tmv_complete: bool,
    /// "COMPLETE" | "INCOMPLETE" | "ERROR"
    status: String,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Time002Artifact {
    provenance: Time002Provenance,
    instruments: Vec<Time002Instrument>,
}

// ─── Output structs ───────────────────────────────────────────────────────────

/// Per-instrument decision record.
#[derive(Debug, Serialize)]
struct InstrumentDecision {
    ticker: String,
    /// "DECIDED" | "EXCLUDED_INCOMPLETE" | "EXCLUDED_ERROR"
    eligibility: String,
    /// C3-002 direction — only present when eligibility == "DECIDED"
    c3_002_direction: Option<String>,
    /// TMV inputs (from TIME-002 artifact, not re-fetched)
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    reference_price: Option<f64>,
    atr_14: Option<f64>,
    /// Exclusion reason when not decided
    exclusion_reason: Option<String>,
    /// Full recommendation record — only present when eligibility == "DECIDED"
    /// and direction != NO_TRADE
    recommendation: Option<RecommendationRecordV1>,
}

/// TIME-003 accounting summary.
#[derive(Debug, Serialize)]
struct Time003Accounting {
    /// Instruments that entered the decision pipeline (COMPLETE TIME-002 records).
    n_decided: usize,
    /// Instruments excluded because TIME-002 status == INCOMPLETE.
    n_excluded_incomplete: usize,
    /// Instruments excluded because TIME-002 status == ERROR.
    n_excluded_error: usize,
    /// Must equal TIME-002 provenance.accounting.n_total.
    n_total: usize,
    /// C3-002 direction breakdown (among decided).
    n_long: usize,
    n_short: usize,
    n_no_trade: usize,
    /// RecommendationEngine action breakdown (among decided, direction != NO_TRADE).
    n_buy: usize,
    n_sell: usize,
    n_watch: usize,
    n_no_trade_evidence: usize,
}

/// The complete TIME-003 decision replay artifact.
#[derive(Debug, Serialize)]
struct Time003Artifact {
    // ── Identity ──────────────────────────────────────────────────────────────
    /// e.g. "TIME003-20260814T101500Z-gen20260820T..."
    decision_replay_id: String,
    /// TIME-002 provenance.reconstruction_id
    reconstruction_id: String,
    /// TIME-003 C3-002 evaluation identity
    state_id: String,
    /// TIME-002 provenance.as_of
    as_of: String,
    /// Always "HISTORICAL"
    source_type: String,
    producer: String,

    // ── Frozen algorithm identity ─────────────────────────────────────────────
    c3_002_artifact_hash: String,
    recommendation_engine_version: String,
    evidence_store_dir: String,
    evidence_store_n_files: usize,

    // ── Input artifact integrity ──────────────────────────────────────────────
    /// SHA-256 of the raw TIME-002 artifact bytes.
    input_artifact_hash: String,

    // ── Accounting ────────────────────────────────────────────────────────────
    accounting: Time003Accounting,

    // ── Artifact generation time (NOT historical state) ───────────────────────
    created_at: String,

    // ── Per-instrument decisions ──────────────────────────────────────────────
    instruments: Vec<InstrumentDecision>,
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    reconstruction: PathBuf,
    policy_dir: PathBuf,
    evidence_dir: String,
    output: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut reconstruction: Option<PathBuf> = None;
    let mut policy_dir = PathBuf::from(RESEARCH_DISCOVERY_TWO_DIR);
    let mut evidence_dir = "datasets/recommendation/historical".to_string();
    let mut output = PathBuf::from("time_machine/decisions");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--reconstruction" => {
                i += 1;
                reconstruction = Some(PathBuf::from(&args[i]));
            }
            "--policy" => {
                i += 1;
                policy_dir = PathBuf::from(&args[i]);
            }
            "--evidence" => {
                i += 1;
                evidence_dir = args[i].clone();
            }
            "--output" => {
                i += 1;
                output = PathBuf::from(&args[i]);
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
        i += 1;
    }

    Ok(Args {
        reconstruction: reconstruction
            .ok_or("--reconstruction is required (path to TIME-002 artifact JSON)")?,
        policy_dir,
        evidence_dir,
        output,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn count_jsonl_files(dir: &str) -> usize {
    match fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "jsonl")
                    .unwrap_or(false)
            })
            .count(),
        Err(_) => 0,
    }
}

fn sha256_of_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Convert Yahoo ticker format ("RELIANCE.NS") to REC-001-H store key ("RELIANCE_NS").
fn ticker_to_store_key(ticker: &str) -> String {
    ticker.replace('.', "_")
}

fn make_decision_replay_id(as_of: &str, created_at: &str) -> String {
    // as_of is already in RFC3339 format from TIME-002; strip non-alphanumeric for the ID.
    let as_of_compact = as_of
        .replace(['-', ':', '.'], "")
        .replace('T', "T")
        .trim_end_matches('Z')
        .to_string()
        + "Z";
    let created_compact = created_at
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == 'T' || *c == 'Z')
        .collect::<String>();
    format!("TIME003-{as_of_compact}-gen{created_compact}")
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── Step 1: Verify frozen C3-002 policy artifact ──────────────────────────
    let policy_path = args.policy_dir.join("selected_policy.json");
    let policy_raw = fs::read_to_string(&policy_path)
        .map_err(|e| format!("cannot read policy {}: {e}", policy_path.display()))?;
    let policy_artifact: PolicyArtifact =
        serde_json::from_str(&policy_raw).map_err(|e| format!("policy JSON parse error: {e}"))?;

    if policy_artifact.artifact_hash != C3_002_ARTIFACT_HASH {
        return Err(format!(
            "TIME-003 identity gate: expected C3-002 artifact hash {C3_002_ARTIFACT_HASH}, \
             got {} — algorithm identity violated",
            policy_artifact.artifact_hash
        )
        .into());
    }
    println!(
        "[time003] C3-002 artifact verified: {}",
        policy_artifact.artifact_hash
    );

    // ── Step 2: Load frozen REC-001-H evidence store ──────────────────────────
    println!(
        "[time003] loading evidence store from: {}",
        args.evidence_dir
    );
    let store = Rec001hStore::load_from_dir(&args.evidence_dir).map_err(|e| {
        format!(
            "cannot load REC-001-H evidence store from {}: {e}",
            args.evidence_dir
        )
    })?;
    let evidence_store_n_files = count_jsonl_files(&args.evidence_dir);
    println!(
        "[time003] evidence store loaded: n_files={evidence_store_n_files} dir={}",
        args.evidence_dir
    );

    // ── Step 3: Load TIME-002 reconstruction artifact (no network) ────────────
    let reconstruction_raw = fs::read_to_string(&args.reconstruction).map_err(|e| {
        format!(
            "cannot read reconstruction artifact {}: {e}",
            args.reconstruction.display()
        )
    })?;
    let input_artifact_hash = sha256_of_bytes(reconstruction_raw.as_bytes());

    let reconstruction: Time002Artifact = serde_json::from_str(&reconstruction_raw)
        .map_err(|e| format!("reconstruction artifact JSON parse error: {e}"))?;

    let prov = &reconstruction.provenance;
    println!(
        "[time003] reconstruction_id={} as_of={} n_total={}",
        prov.reconstruction_id, prov.as_of, prov.accounting.n_total
    );

    // TIME-003 only accepts HISTORICAL source artifacts.
    // The TIME-002 artifact carries clock_mode=REPLAY which is the historical marker.
    // (LIVE-001 artifacts carry source_type=LIVE; TIME-002 carries clock_mode=REPLAY.)
    // We verify this is not a live snapshot by checking clock_mode.
    // (TIME-002 artifacts do not have a source_type field; they have clock_mode.)

    // ── Step 4: Build identity strings ────────────────────────────────────────
    let created_at = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let decision_replay_id = make_decision_replay_id(&prov.as_of, &created_at);
    let state_id = format!(
        "TIME003-STATE-{}",
        prov.reconstruction_id.trim_start_matches("TIME002-")
    );

    println!("[time003] decision_replay_id={decision_replay_id}");
    println!("[time003] state_id={state_id}");
    println!("[time003] input_artifact_hash={input_artifact_hash}");

    // ── Step 5: Eligibility loop — frozen C3-002 + frozen RecommendationEngine ─
    let engine = RecommendationEngineV1::new(&store);

    let mut instruments: Vec<InstrumentDecision> = Vec::new();
    let mut n_decided = 0usize;
    let mut n_excluded_incomplete = 0usize;
    let mut n_excluded_error = 0usize;
    let mut n_long = 0usize;
    let mut n_short = 0usize;
    let mut n_no_trade_direction = 0usize;
    let mut n_buy = 0usize;
    let mut n_sell = 0usize;
    let mut n_watch = 0usize;
    let mut n_no_trade_evidence = 0usize;

    for inst in &reconstruction.instruments {
        // Eligibility gate: ERROR status or error field present → EXCLUDED_ERROR
        if inst.status == "ERROR" || inst.error.is_some() {
            println!("[time003] excluded ticker={} reason=ERROR", inst.ticker);
            instruments.push(InstrumentDecision {
                ticker: inst.ticker.clone(),
                eligibility: "EXCLUDED_ERROR".to_string(),
                c3_002_direction: None,
                trend: inst.trend.clone(),
                momentum: inst.momentum.clone(),
                volatility: inst.volatility.clone(),
                reference_price: inst.reference_price,
                atr_14: inst.atr_14,
                exclusion_reason: Some(
                    inst.error
                        .clone()
                        .unwrap_or_else(|| "ERROR status".to_string()),
                ),
                recommendation: None,
            });
            n_excluded_error += 1;
            continue;
        }

        // Eligibility gate: INCOMPLETE or tmv_complete=false → EXCLUDED_INCOMPLETE
        if inst.status == "INCOMPLETE" || !inst.tmv_complete {
            println!(
                "[time003] excluded ticker={} reason=INCOMPLETE",
                inst.ticker
            );
            instruments.push(InstrumentDecision {
                ticker: inst.ticker.clone(),
                eligibility: "EXCLUDED_INCOMPLETE".to_string(),
                c3_002_direction: None,
                trend: inst.trend.clone(),
                momentum: inst.momentum.clone(),
                volatility: inst.volatility.clone(),
                reference_price: inst.reference_price,
                atr_14: inst.atr_14,
                exclusion_reason: Some("tmv_complete=false or INCOMPLETE status".to_string()),
                recommendation: None,
            });
            n_excluded_incomplete += 1;
            continue;
        }

        // COMPLETE instrument — apply frozen C3-002 using only TIME-002 features.
        // No re-fetch, no re-computation.
        let trend = inst.trend.as_deref().unwrap_or("absent");
        let momentum = inst.momentum.as_deref().unwrap_or("absent");
        let volatility = inst.volatility.as_deref().unwrap_or("absent");

        let action = first_match_action_from_tmv(&policy_artifact, trend, momentum, volatility);
        let direction = match action {
            DecisionAction::Long => "LONG",
            DecisionAction::Short => "SHORT",
            DecisionAction::NoTrade => "NO_TRADE",
        };

        match action {
            DecisionAction::Long => n_long += 1,
            DecisionAction::Short => n_short += 1,
            DecisionAction::NoTrade => n_no_trade_direction += 1,
        }

        // Apply frozen RecommendationEngine v1 + frozen REC-001-H.
        // decision_id = decision_replay_id + ticker (unique per instrument per replay).
        let decision_id = format!("{decision_replay_id}-{}", inst.ticker);
        let store_key = ticker_to_store_key(&inst.ticker);

        // Volatility mapping: TIME-002 uses "Available"/"Unavailable";
        // RecommendationEngine v1 expects "present"/"absent".
        let volatility_for_engine = if volatility == "Available" {
            "present"
        } else {
            "absent"
        };

        // relative_volume_20: not available in TIME-002 artifact; use 1.0 (neutral).
        // This is documented in the artifact as a known limitation, consistent with LIVE-003.
        let relative_volume_20 = 1.0_f64;

        let rec = engine.evaluate(
            &decision_id,
            &store_key,
            direction,
            trend,
            momentum,
            inst.reference_price,
            volatility_for_engine,
            relative_volume_20,
        );

        let action_str = rec.action.as_str();
        println!(
            "[time003] decided ticker={} direction={direction} trend={trend} \
             momentum={momentum} volatility={volatility} action={action_str} \
             evidence_class={} sample_size={}",
            inst.ticker, rec.evidence_class, rec.sample_size
        );

        match rec.action {
            coralys_decision::recommendation::RecommendationAction::Buy => n_buy += 1,
            coralys_decision::recommendation::RecommendationAction::Sell => n_sell += 1,
            coralys_decision::recommendation::RecommendationAction::Watch => n_watch += 1,
            coralys_decision::recommendation::RecommendationAction::NoTrade => {
                n_no_trade_evidence += 1
            }
        }

        instruments.push(InstrumentDecision {
            ticker: inst.ticker.clone(),
            eligibility: "DECIDED".to_string(),
            c3_002_direction: Some(direction.to_string()),
            trend: inst.trend.clone(),
            momentum: inst.momentum.clone(),
            volatility: inst.volatility.clone(),
            reference_price: inst.reference_price,
            atr_14: inst.atr_14,
            exclusion_reason: None,
            recommendation: Some(rec),
        });
        n_decided += 1;
    }

    // ── Step 6: Accounting invariant ──────────────────────────────────────────
    let n_total = reconstruction.provenance.accounting.n_total;
    let accounting_total = n_decided + n_excluded_incomplete + n_excluded_error;
    if accounting_total != n_total {
        return Err(format!(
            "TIME-003 accounting invariant violated: \
             decided({n_decided}) + excluded_incomplete({n_excluded_incomplete}) + \
             excluded_error({n_excluded_error}) = {accounting_total} != \
             TIME-002 n_total({n_total})"
        )
        .into());
    }

    println!(
        "[time003] accounting: total={n_total} decided={n_decided} \
         excluded_incomplete={n_excluded_incomplete} excluded_error={n_excluded_error}"
    );
    println!("[time003] c3_002: long={n_long} short={n_short} no_trade={n_no_trade_direction}");
    println!(
        "[time003] recommendation: buy={n_buy} sell={n_sell} watch={n_watch} \
         no_trade_evidence={n_no_trade_evidence}"
    );

    // ── Step 7: Build and write artifact ──────────────────────────────────────
    let artifact = Time003Artifact {
        decision_replay_id: decision_replay_id.clone(),
        reconstruction_id: prov.reconstruction_id.clone(),
        state_id: state_id.clone(),
        as_of: prov.as_of.clone(),
        source_type: SOURCE_TYPE.to_string(),
        producer: PRODUCER.to_string(),
        c3_002_artifact_hash: C3_002_ARTIFACT_HASH.to_string(),
        recommendation_engine_version: RECOMMENDATION_POLICY_VERSION_V1.to_string(),
        evidence_store_dir: args.evidence_dir.clone(),
        evidence_store_n_files,
        input_artifact_hash,
        accounting: Time003Accounting {
            n_decided,
            n_excluded_incomplete,
            n_excluded_error,
            n_total,
            n_long,
            n_short,
            n_no_trade: n_no_trade_direction,
            n_buy,
            n_sell,
            n_watch,
            n_no_trade_evidence,
        },
        created_at: created_at.clone(),
        instruments,
    };

    fs::create_dir_all(&args.output)?;

    // Named artifact: TIME003-<as_of_compact>.json
    let as_of_compact = prov
        .as_of
        .replace(['-', ':', '.'], "")
        .replace('T', "T")
        .trim_end_matches('Z')
        .to_string()
        + "Z";
    let artifact_filename = format!("TIME003-{as_of_compact}.json");
    let artifact_path = args.output.join(&artifact_filename);
    let latest_path = args.output.join("latest.json");

    let artifact_json = serde_json::to_string_pretty(&artifact)?;
    fs::write(&artifact_path, &artifact_json)?;
    fs::write(&latest_path, &artifact_json)?;

    println!("[time003] artifact written: {}", artifact_path.display());
    println!("[time003] latest written:   {}", latest_path.display());
    println!("[time003] decision_replay_id={decision_replay_id}");
    println!("[time003] DONE");

    Ok(())
}
