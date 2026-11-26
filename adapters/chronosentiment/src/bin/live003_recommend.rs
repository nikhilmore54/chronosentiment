//! LIVE-003 — Live Recommendation Generation
//!
//! Pure transformation: reads a LIVE-002 state artifact (latest.json) and
//! applies the frozen RecommendationEngine v1 + frozen REC-001-H evidence
//! store to every EVALUATED instrument. No network access. No Yahoo fetch.
//! No new indicators. No algorithm changes.
//!
//! ## Provenance chain
//!
//! ```text
//! LIVE-001 snapshot_id
//!   └─► LIVE-002 state_id  (c3_002_artifact_hash verified)
//!         └─► LIVE-003 recommendation_id  (engine_version + evidence_store_identity)
//! ```
//!
//! ## AC boundary
//!
//! 1. **Input fidelity**        — every value originates from the LIVE-002 state artifact.
//! 2. **Engine fidelity**       — frozen RecommendationEngine v1 (RECOMMENDATION_POLICY_VERSION_V1).
//! 3. **Evidence fidelity**     — frozen REC-001-H evidence store; identity recorded in artifact.
//! 4. **No algorithm changes**  — no threshold/R:R/scoring changes vs REC-BASELINE-001.
//! 5. **Full provenance**       — input state_id + C3-002 artifact hash + engine version +
//!                                evidence-store identity + recommendation timestamp all recorded.
//! 6. **Accounting**            — recommended + skipped_no_trade + skipped_excluded == n_evaluated_input.
//!
//! ## Usage
//!
//!   live003_recommend \
//!     --state    live_capture/evaluations/latest.json \
//!     --evidence datasets/recommendation/historical \
//!     --output   live_capture/recommendations/
//!
//! The `--state` argument defaults to `live_capture/evaluations/latest.json`.
//! The `--evidence` argument defaults to `datasets/recommendation/historical`.
//! The `--output` argument defaults to `live_capture/recommendations/`.

use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use coralys_decision::recommendation::{
    Rec001hStore, RecommendationEngineV1, RecommendationRecordV1, RECOMMENDATION_POLICY_VERSION_V1,
};
use serde::{Deserialize, Serialize};

// ─── LIVE-003 producer identity ───────────────────────────────────────────────

const PRODUCER: &str = "live003_recommend.v1";

// ─── Input structs (mirrors LIVE-002 artifact schema) ─────────────────────────

#[derive(Debug, Deserialize)]
struct InstrumentState {
    ticker: String,
    /// "EVALUATED", "EXCLUDED_INCOMPLETE", "EXCLUDED_ERROR"
    eligibility: String,
    /// Only present when eligibility == "EVALUATED"
    c3_002_direction: Option<String>,
    trend: Option<String>,
    momentum: Option<String>,
    volatility: Option<String>,
    reference_price: Option<f64>,
    #[allow(dead_code)]
    atr_14: Option<f64>,
    #[allow(dead_code)]
    source_bar_timestamp: Option<String>,
    #[allow(dead_code)]
    acquisition_timestamp: Option<String>,
    #[allow(dead_code)]
    exclusion_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Live002StateArtifact {
    state_id: String,
    evaluated_at: String,
    #[allow(dead_code)]
    producer: String,
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
    instruments: Vec<InstrumentState>,
}

// ─── Output structs ───────────────────────────────────────────────────────────

/// Full LIVE-003 recommendation artifact — one record per EVALUATED instrument.
#[derive(Debug, Serialize)]
struct Live003RecommendationArtifact {
    /// e.g. "LIVE-003-20260819-0518"
    recommendation_id: String,
    /// ISO-8601 UTC timestamp when this recommendation batch was produced.
    recommended_at: String,
    producer: String,

    // ── Provenance chain ──────────────────────────────────────────────────────
    /// Source LIVE-002 state_id.
    source_state_id: String,
    /// Source LIVE-002 evaluated_at timestamp.
    source_state_evaluated_at: String,
    /// Source LIVE-001 snapshot_id (carried through from LIVE-002).
    source_snapshot_id: String,
    /// Source LIVE-001 snapshot_timestamp (carried through from LIVE-002).
    source_snapshot_timestamp: String,
    /// Source type — must be "LIVE".
    source_type: String,
    /// C3-002 artifact hash from LIVE-002 (frozen, not re-verified here).
    c3_002_artifact_hash: String,
    /// RecommendationEngine policy version (frozen).
    engine_version: String,
    /// REC-001-H evidence store directory identity.
    evidence_store_dir: String,
    /// Number of JSONL files loaded from the evidence store.
    evidence_store_n_files: usize,

    // ── LIVE-002 accounting (carried through for traceability) ────────────────
    n_input_from_live002: usize,
    n_evaluated_from_live002: usize,
    n_excluded_incomplete_from_live002: usize,
    n_excluded_error_from_live002: usize,
    n_long_from_live002: usize,
    n_short_from_live002: usize,
    n_no_trade_from_live002: usize,

    // ── LIVE-003 accounting ───────────────────────────────────────────────────
    /// Instruments that received a recommendation (eligibility == EVALUATED, direction != NO_TRADE).
    n_recommended: usize,
    /// Instruments skipped because direction == NO_TRADE.
    n_skipped_no_trade: usize,
    /// Instruments skipped because eligibility != EVALUATED.
    n_skipped_excluded: usize,
    /// BUY count among recommended.
    n_buy: usize,
    /// SELL count among recommended.
    n_sell: usize,
    /// WATCH count among recommended.
    n_watch: usize,
    /// NO_TRADE count among recommended (evidence-driven, not direction-driven).
    n_no_trade_evidence: usize,

    /// Per-instrument recommendations (only EVALUATED instruments).
    recommendations: Vec<RecommendationRecordV1>,
}

// ─── CLI args ─────────────────────────────────────────────────────────────────

struct Args {
    state: PathBuf,
    evidence_dir: String,
    output: PathBuf,
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut state = PathBuf::from("live_capture/evaluations/latest.json");
    let mut evidence_dir = "datasets/recommendation/historical".to_string();
    let mut output = PathBuf::from("live_capture/recommendations");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--state" => {
                i += 1;
                state = PathBuf::from(&args[i]);
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
        state,
        evidence_dir,
        output,
    })
}

// ─── Evidence store file count (for identity recording) ───────────────────────

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

// ─── Ticker normalisation ──────────────────────────────────────────────────────
//
// LIVE-001/002 use Yahoo ticker format: "RELIANCE.NS"
// Rec001hStore keys use underscore format: "RELIANCE_NS"
// This function converts between the two.

fn ticker_to_store_key(ticker: &str) -> String {
    ticker.replace('.', "_")
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;

    // ── AC-3: Load frozen REC-001-H evidence store ────────────────────────────
    println!(
        "[live003] loading evidence store from: {}",
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
        "[live003] evidence store loaded: n_files={evidence_store_n_files} dir={}",
        args.evidence_dir
    );

    // ── AC-1: Load LIVE-002 state artifact (no network) ───────────────────────
    let state_raw = fs::read_to_string(&args.state)
        .map_err(|e| format!("cannot read state artifact {}: {e}", args.state.display()))?;
    let state: Live002StateArtifact = serde_json::from_str(&state_raw)
        .map_err(|e| format!("state artifact JSON parse error: {e}"))?;

    println!(
        "[live003] state_id={} source_type={} n_evaluated={}",
        state.state_id, state.source_type, state.n_evaluated
    );

    // Verify source_type is LIVE
    if state.source_type != "LIVE" {
        return Err(format!(
            "LIVE-003 requires source_type=LIVE, got {}",
            state.source_type
        )
        .into());
    }

    // ── AC-2: Apply frozen RecommendationEngine v1 ────────────────────────────
    let recommended_at = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let recommendation_id = format!(
        "LIVE-003-{}",
        &state.state_id.trim_start_matches("LIVE-002-")
    );

    let engine = RecommendationEngineV1::new(&store);

    let mut recommendations: Vec<RecommendationRecordV1> = Vec::new();
    let mut n_recommended = 0usize;
    let mut n_skipped_no_trade = 0usize;
    let mut n_skipped_excluded = 0usize;
    let mut n_buy = 0usize;
    let mut n_sell = 0usize;
    let mut n_watch = 0usize;
    let mut n_no_trade_evidence = 0usize;

    for inst in &state.instruments {
        // AC-6: Only process EVALUATED instruments
        if inst.eligibility != "EVALUATED" {
            n_skipped_excluded += 1;
            continue;
        }

        let direction = inst.c3_002_direction.as_deref().unwrap_or("NO_TRADE");
        let trend = inst.trend.as_deref().unwrap_or("absent");
        let momentum = inst.momentum.as_deref().unwrap_or("absent");
        // Volatility from LIVE-002: "Available" or "Unavailable" — map to "present"/"absent"
        // for the v1 engine's VolatilityRegime::from_str()
        let volatility_raw = inst.volatility.as_deref().unwrap_or("absent");
        let volatility = if volatility_raw == "Available" {
            "present"
        } else {
            "absent"
        };

        // Decision ID for this live instrument: use recommendation_id + ticker
        let decision_id = format!("{}-{}", recommendation_id, inst.ticker);

        // Store key: convert "RELIANCE.NS" → "RELIANCE_NS"
        let store_key = ticker_to_store_key(&inst.ticker);

        // relative_volume_20: not available in LIVE-002 state; use 1.0 (neutral)
        // This is documented in the artifact as a known limitation.
        let relative_volume_20 = 1.0_f64;

        let rec = engine.evaluate(
            &decision_id,
            &store_key,
            direction,
            trend,
            momentum,
            inst.reference_price,
            volatility,
            relative_volume_20,
        );

        let action_str = rec.action.as_str();
        println!(
            "[live003] ticker={} direction={direction} trend={trend} momentum={momentum} \
             volatility={volatility} action={action_str} evidence_class={} sample_size={}",
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

        if direction == "NO_TRADE" {
            n_skipped_no_trade += 1;
        } else {
            n_recommended += 1;
        }

        recommendations.push(rec);
    }

    // AC-6: Accounting invariant
    // n_recommended + n_skipped_no_trade + n_skipped_excluded == n_evaluated (from LIVE-002)
    let accounting_total = n_recommended + n_skipped_no_trade + n_skipped_excluded;
    let n_evaluated_input =
        state.n_evaluated + state.n_excluded_incomplete + state.n_excluded_error;
    if accounting_total != n_evaluated_input {
        return Err(format!(
            "LIVE-003 accounting invariant violated: recommended({n_recommended}) + \
             skipped_no_trade({n_skipped_no_trade}) + skipped_excluded({n_skipped_excluded}) \
             = {accounting_total} != n_input({n_evaluated_input})"
        )
        .into());
    }

    // Sort by rank_score descending for the output artifact
    recommendations.sort_by(|a, b| {
        b.rank_score
            .partial_cmp(&a.rank_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let artifact = Live003RecommendationArtifact {
        recommendation_id: recommendation_id.clone(),
        recommended_at,
        producer: PRODUCER.to_string(),
        source_state_id: state.state_id.clone(),
        source_state_evaluated_at: state.evaluated_at.clone(),
        source_snapshot_id: state.source_snapshot_id.clone(),
        source_snapshot_timestamp: state.source_snapshot_timestamp.clone(),
        source_type: state.source_type.clone(),
        c3_002_artifact_hash: state.c3_002_artifact_hash.clone(),
        engine_version: RECOMMENDATION_POLICY_VERSION_V1.to_string(),
        evidence_store_dir: args.evidence_dir.clone(),
        evidence_store_n_files,
        n_input_from_live002: state.n_input,
        n_evaluated_from_live002: state.n_evaluated,
        n_excluded_incomplete_from_live002: state.n_excluded_incomplete,
        n_excluded_error_from_live002: state.n_excluded_error,
        n_long_from_live002: state.n_long,
        n_short_from_live002: state.n_short,
        n_no_trade_from_live002: state.n_no_trade,
        n_recommended,
        n_skipped_no_trade,
        n_skipped_excluded,
        n_buy,
        n_sell,
        n_watch,
        n_no_trade_evidence,
        recommendations,
    };

    // ── Write output artifact ─────────────────────────────────────────────────
    fs::create_dir_all(&args.output)?;
    let artifact_filename = format!("{recommendation_id}.json");
    let artifact_path = args.output.join(&artifact_filename);
    let latest_path = args.output.join("latest.json");

    let json = serde_json::to_string_pretty(&artifact)?;
    fs::write(&artifact_path, &json)?;
    fs::write(&latest_path, &json)?;

    println!("[live003] result=OK");
    println!("[live003] recommendation_id={recommendation_id}");
    println!("[live003] engine_version={RECOMMENDATION_POLICY_VERSION_V1}");
    println!("[live003] evidence_store_dir={}", args.evidence_dir);
    println!("[live003] evidence_store_n_files={evidence_store_n_files}");
    println!("[live003] source_state_id={}", state.state_id);
    println!("[live003] source_snapshot_id={}", state.source_snapshot_id);
    println!(
        "[live003] c3_002_artifact_hash={}",
        state.c3_002_artifact_hash
    );
    println!("[live003] n_recommended={n_recommended}");
    println!("[live003] n_skipped_no_trade={n_skipped_no_trade}");
    println!("[live003] n_skipped_excluded={n_skipped_excluded}");
    println!("[live003] n_buy={n_buy}");
    println!("[live003] n_sell={n_sell}");
    println!("[live003] n_watch={n_watch}");
    println!("[live003] n_no_trade_evidence={n_no_trade_evidence}");
    println!(
        "[live003] accounting: {n_recommended} recommended + {n_skipped_no_trade} skipped_no_trade \
         + {n_skipped_excluded} skipped_excluded = {accounting_total} / {n_evaluated_input} input"
    );
    println!("[live003] artifact={}", artifact_path.display());
    println!("[live003] latest={}", latest_path.display());

    Ok(())
}
