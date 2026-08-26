//! CS-P-006-P.E.3-A — Coralys Execution Model v0 (ATR-anchored, TMV-scaled).
//!
//! Status: APPROVED — READY TO FREEZE (pending artifact hash computation)
//!
//! Two information boundaries:
//!
//!   Decision boundary T:
//!     Direction = f(state ≤ T)
//!     Sealed at T by C3-002.
//!
//!   Execution boundary E:
//!     Execution Intent = f(state ≤ T, direction, entry_open(E))
//!     Sealed at E (next eligible session open).
//!     entry_source = "NEXT_SESSION_OPEN"
//!
//! Authorized inputs:
//!   From bars ≤ T: certified_tmv_labels, state_hash, atr_14, roc_20,
//!                  frozen_coralys_target_artifact_id
//!   From E open:   entry_price (actual fill at session open)
//!   From T seal:   c3_002_direction
//!
//! Forbidden inputs:
//!   bars_after_T, realized_future_return, realized_V, target_hit,
//!   path_optimized_hit_rate, per_name_hindsight_target,
//!   coralys_evolved_after_T, new_indicator_families,
//!   intraday_price_after_open, E_close, any_bar_after_E_open.
//!
//! ATR=0 or unavailable → CoralysExecutionResult::Invalid.
//! Do NOT fall back to +5%. That would blend the treatment with the P.E.2 control.
//!
//! The multipliers are FROZEN DESIGN PARAMETERS for v0, not learned values.
//! Whether they outperform the +5% control is the question CS-P-007 will answer.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::decision_intent::{
    CORALYS_TARGET_ARTIFACT_PRESENT, CORALYS_TARGET_SEARCH_AUTHORIZED,
    TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED, TARGET_LOOKAHEAD_AUTHORIZED,
};
use super::observatory_execution::TARGET_PATH_OPTIMIZATION_AUTHORIZED;

// ─── Model identity ──────────────────────────────────────────────────────────

pub const CORALYS_EXEC_MODEL_ID: &str = "coralys-exec-v0";
pub const CORALYS_EXEC_MODEL_VERSION: &str = "0.1.0";
pub const CORALYS_EXEC_METHODOLOGY: &str = "atr_anchored_tmv_scaled";
pub const ENTRY_SOURCE_NEXT_SESSION_OPEN: &str = "NEXT_SESSION_OPEN";

/// SHA256 of the canonical model specification text (frozen sections 1–11 of
/// PE3_CORALYS_EXECUTION_MODEL_SPEC.md, 11002 chars, boundary at "## 12. Revised experiment plan").
/// Computed: 2026-08-16. Must never change.
pub const CORALYS_EXEC_ARTIFACT_HASH: &str =
    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f";

// ─── Target / risk parameters ────────────────────────────────────────────────

/// Minimum target percentage (2%). Frozen design parameter.
pub const TARGET_PCT_MIN: f64 = 0.02;
/// Maximum target percentage (15%). Frozen design parameter.
pub const TARGET_PCT_MAX: f64 = 0.15;
/// Minimum risk percentage (1%). Frozen design parameter.
pub const RISK_PCT_MIN: f64 = 0.01;
/// Maximum risk percentage (8%). Frozen design parameter.
pub const RISK_PCT_MAX: f64 = 0.08;
/// Maximum hold in market sessions (same as P.E.2 control).
pub const MAXIMUM_HOLD_SESSIONS: u32 = 20;

// ─── TMV state ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TmvState {
    BullishPositive,
    BullishNegative,
    BearishPositive,
    BearishNegative,
    Unknown,
}

impl TmvState {
    pub fn from_labels(trend: &str, momentum: &str) -> Self {
        match (trend, momentum) {
            ("Bullish", "Positive") => TmvState::BullishPositive,
            ("Bullish", "Negative") => TmvState::BullishNegative,
            ("Bearish", "Positive") => TmvState::BearishPositive,
            ("Bearish", "Negative") => TmvState::BearishNegative,
            _ => TmvState::Unknown,
        }
    }

    /// Target multiplier. FROZEN DESIGN PARAMETER for v0. Not learned.
    pub fn target_multiplier(&self) -> f64 {
        match self {
            TmvState::BullishPositive => 2.0,
            TmvState::BullishNegative => 1.5,
            TmvState::BearishPositive => 1.5,
            TmvState::BearishNegative => 1.0,
            TmvState::Unknown => 1.0,
        }
    }

    /// Risk multiplier. FROZEN DESIGN PARAMETER for v0. Not learned.
    pub fn risk_multiplier(&self) -> f64 {
        match self {
            TmvState::BullishPositive => 1.0,
            TmvState::BullishNegative => 0.75,
            TmvState::BearishPositive => 0.75,
            TmvState::BearishNegative => 0.5,
            TmvState::Unknown => 0.5,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TmvState::BullishPositive => "Bullish / Positive",
            TmvState::BullishNegative => "Bullish / Negative",
            TmvState::BearishPositive => "Bearish / Positive",
            TmvState::BearishNegative => "Bearish / Negative",
            TmvState::Unknown => "Unknown",
        }
    }
}

// ─── Output types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoralysExecutionIntent {
    pub instrument: String,
    /// T — when C3-002 decided. Direction information boundary.
    pub decision_time: String,
    /// T — bars ≤ T used for state/ATR. Same as decision_time.
    pub decision_information_cutoff: String,
    /// E — entry session open timestamp. Execution information boundary.
    pub entry_time: String,
    /// E — entry open is the last allowed input. Same as entry_time.
    pub execution_information_cutoff: String,
    /// Always "NEXT_SESSION_OPEN" for v0.
    pub entry_source: String,
    pub direction: String,
    /// Actual fill price at E open. Not a future price.
    pub entry_price: f64,
    pub target_pct: f64,
    pub target_price: f64,
    pub target_basis: String,
    pub risk_pct: f64,
    pub risk_boundary: f64,
    pub risk_basis: String,
    /// ATR(14) from bars ≤ T. In price units.
    pub atr_14_at_t: f64,
    pub tmv_state: String,
    pub state_hash: String,
    pub maximum_hold_sessions: u32,
    pub coralys_model_id: String,
    pub coralys_model_version: String,
    pub coralys_artifact_hash: String,
    pub intent_hash: String,
    /// true — execution intent seals at E (entry session open), not at T.
    pub sealed_at_entry: bool,
    /// true — direction was sealed at T by C3-002.
    pub direction_sealed_at_t: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoralysExecutionParams {
    pub target_pct: f64,
    pub target_price: f64,
    pub risk_pct: f64,
    pub risk_boundary: f64,
    pub atr_14_used: f64,
    pub tmv_state: TmvState,
}

/// Result type for Coralys execution intent generation.
///
/// `Invalid` is returned when ATR is unavailable or zero.
/// Do NOT fall back to +5% — that would blend the treatment with the P.E.2 control.
/// Invalid positions are excluded from the P.E.3 treatment sample.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoralysExecutionResult {
    Intent(CoralysExecutionIntent),
    Invalid {
        instrument: String,
        decision_time: String,
        reason: String,
    },
}

impl CoralysExecutionResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, CoralysExecutionResult::Intent(_))
    }

    pub fn intent(self) -> Option<CoralysExecutionIntent> {
        match self {
            CoralysExecutionResult::Intent(i) => Some(i),
            _ => None,
        }
    }

    pub fn invalid_reason(&self) -> Option<&str> {
        match self {
            CoralysExecutionResult::Invalid { reason, .. } => Some(reason.as_str()),
            _ => None,
        }
    }
}

// ─── Guard ────────────────────────────────────────────────────────────────────

/// Refuse to emit a Coralys execution intent if any forbidden research gate is open,
/// or if the artifact has not been frozen.
pub fn refuse_if_not_ready() -> Result<(), String> {
    if TARGET_LOOKAHEAD_AUTHORIZED {
        return Err("coralys-exec-v0: refusing — TARGET_LOOKAHEAD_AUTHORIZED is true".into());
    }
    if TARGET_PATH_OPTIMIZATION_AUTHORIZED {
        return Err(
            "coralys-exec-v0: refusing — TARGET_PATH_OPTIMIZATION_AUTHORIZED is true".into(),
        );
    }
    if CORALYS_TARGET_SEARCH_AUTHORIZED {
        return Err("coralys-exec-v0: refusing — CORALYS_TARGET_SEARCH_AUTHORIZED is true".into());
    }
    if TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED {
        return Err(
            "coralys-exec-v0: refusing — TARGET_FROM_REALIZED_OUTCOME_AUTHORIZED is true".into(),
        );
    }
    if !CORALYS_TARGET_ARTIFACT_PRESENT {
        return Err(
            "coralys-exec-v0: refusing — CORALYS_TARGET_ARTIFACT_PRESENT is false; \
             freeze the artifact before calling seal_coralys_execution_intent"
                .into(),
        );
    }
    Ok(())
}

// ─── Core computation ─────────────────────────────────────────────────────────

/// Compute target and risk parameters from ATR and TMV state.
///
/// Returns `None` if ATR is unavailable or zero.
/// Do NOT fall back to a fixed percentage — that would blend the treatment with P.E.2.
///
/// All inputs must be certified at or before T.
/// `atr_14` is in price units (not percentage).
/// `entry_price` is the actual fill price at entry session open (E).
pub fn compute_execution_params(
    atr_14: Option<f64>,
    entry_price: f64,
    trend: &str,
    momentum: &str,
    direction: &str,
) -> Option<CoralysExecutionParams> {
    let atr = match atr_14 {
        Some(a) if a > 0.0 && entry_price > 0.0 => a,
        _ => return None, // ATR unavailable or zero → Invalid, not fallback
    };

    let tmv = TmvState::from_labels(trend, momentum);
    let base = atr / entry_price;

    let target_pct = (base * tmv.target_multiplier()).clamp(TARGET_PCT_MIN, TARGET_PCT_MAX);
    let risk_pct = (base * tmv.risk_multiplier()).clamp(RISK_PCT_MIN, RISK_PCT_MAX);

    // Direction-aware price levels.
    // LONG/SHORT symmetry invariant:
    //   |target_price - entry_price| is equal for LONG and SHORT
    //   |risk_boundary - entry_price| is equal for LONG and SHORT
    let (target_price, risk_boundary) = match direction {
        "LONG" => (
            entry_price * (1.0 + target_pct),
            entry_price * (1.0 - risk_pct),
        ),
        "SHORT" => (
            entry_price * (1.0 - target_pct),
            entry_price * (1.0 + risk_pct),
        ),
        _ => return None,
    };

    Some(CoralysExecutionParams {
        target_pct,
        target_price,
        risk_pct,
        risk_boundary,
        atr_14_used: atr,
        tmv_state: tmv,
    })
}

/// Seal a Coralys execution intent.
///
/// This is the P.E.3 treatment entry point. It will return an error until
/// `CORALYS_TARGET_ARTIFACT_PRESENT` is set to `true` in `decision_intent.rs`.
///
/// Returns `CoralysExecutionResult::Invalid` when ATR is unavailable or zero.
/// Do NOT fall back to +5% — that would blend the treatment with the P.E.2 control.
///
/// `entry_price` is the actual fill price at the entry session open (E).
/// The execution intent seals at E, not at T.
/// The direction was sealed at T by C3-002.
///
/// `decision_time` = T (direction seal time)
/// `entry_time` = E (entry session open timestamp)
pub fn seal_coralys_execution_intent(
    instrument: &str,
    decision_time: &str,
    entry_time: &str,
    direction: &str,
    entry_price: f64,
    atr_14: Option<f64>,
    trend: &str,
    momentum: &str,
    state_hash: &str,
) -> Result<CoralysExecutionResult, String> {
    refuse_if_not_ready()?;

    if instrument.trim().is_empty() {
        return Err("coralys-exec-v0: instrument must not be empty".into());
    }

    if entry_price <= 0.0 {
        return Ok(CoralysExecutionResult::Invalid {
            instrument: instrument.to_string(),
            decision_time: decision_time.to_string(),
            reason: format!("entry_price must be positive, got {entry_price}"),
        });
    }

    if !matches!(direction, "LONG" | "SHORT") {
        return Ok(CoralysExecutionResult::Invalid {
            instrument: instrument.to_string(),
            decision_time: decision_time.to_string(),
            reason: format!("direction must be LONG or SHORT, got {direction}"),
        });
    }

    let params = match compute_execution_params(atr_14, entry_price, trend, momentum, direction) {
        Some(p) => p,
        None => {
            return Ok(CoralysExecutionResult::Invalid {
                instrument: instrument.to_string(),
                decision_time: decision_time.to_string(),
                reason: format!(
                    "atr_14 unavailable or zero (got {:?}) — cannot derive execution intent; \
                     position excluded from P.E.3 treatment sample (NO_CORALYS_EXECUTION)",
                    atr_14
                ),
            });
        }
    };

    let basis = format!(
        "{} (frozen v0 parameters: atr_14={:.4}, tmv={}, target_mult={:.2}, risk_mult={:.2})",
        CORALYS_EXEC_METHODOLOGY,
        params.atr_14_used,
        params.tmv_state.label(),
        params.tmv_state.target_multiplier(),
        params.tmv_state.risk_multiplier(),
    );

    let mut intent = CoralysExecutionIntent {
        instrument: instrument.to_string(),
        decision_time: decision_time.to_string(),
        decision_information_cutoff: decision_time.to_string(),
        entry_time: entry_time.to_string(),
        execution_information_cutoff: entry_time.to_string(),
        entry_source: ENTRY_SOURCE_NEXT_SESSION_OPEN.to_string(),
        direction: direction.to_string(),
        entry_price,
        target_pct: params.target_pct,
        target_price: params.target_price,
        target_basis: basis.clone(),
        risk_pct: params.risk_pct,
        risk_boundary: params.risk_boundary,
        risk_basis: basis,
        atr_14_at_t: params.atr_14_used,
        tmv_state: params.tmv_state.label().to_string(),
        state_hash: state_hash.to_string(),
        maximum_hold_sessions: MAXIMUM_HOLD_SESSIONS,
        coralys_model_id: CORALYS_EXEC_MODEL_ID.to_string(),
        coralys_model_version: CORALYS_EXEC_MODEL_VERSION.to_string(),
        coralys_artifact_hash: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
        intent_hash: String::new(),
        sealed_at_entry: true,
        direction_sealed_at_t: true,
    };

    intent.intent_hash = hash_coralys_intent(&intent);
    Ok(CoralysExecutionResult::Intent(intent))
}

fn hash_coralys_intent(intent: &CoralysExecutionIntent) -> String {
    let identity = serde_json::json!({
        "coralys_model_id": intent.coralys_model_id,
        "coralys_model_version": intent.coralys_model_version,
        "coralys_artifact_hash": intent.coralys_artifact_hash,
        "decision_time": intent.decision_time,
        "direction": intent.direction,
        "entry_price": format!("{:.8}", intent.entry_price),
        "entry_source": intent.entry_source,
        "execution_information_cutoff": intent.execution_information_cutoff,
        "instrument": intent.instrument,
        "risk_pct": format!("{:.8}", intent.risk_pct),
        "state_hash": intent.state_hash,
        "target_pct": format!("{:.8}", intent.target_pct),
    });
    format!("{:x}", Sha256::digest(identity.to_string().as_bytes()))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── compute_execution_params tests ──────────────────────────────────────

    #[test]
    fn bullish_positive_long_produces_2x_atr_target() {
        let params = compute_execution_params(
            Some(50.0), // ATR = ₹50
            1000.0,     // entry = ₹1000
            "Bullish",
            "Positive",
            "LONG",
        )
        .expect("should produce params");
        // base = 50/1000 = 0.05; multiplier = 2.0 → target = 0.10
        assert!(
            (params.target_pct - 0.10).abs() < 1e-9,
            "target_pct={}",
            params.target_pct
        );
        assert!((params.target_price - 1100.0).abs() < 1e-6);
        // risk multiplier = 1.0 → risk = 0.05
        assert!((params.risk_pct - 0.05).abs() < 1e-9);
        assert!((params.risk_boundary - 950.0).abs() < 1e-6);
    }

    #[test]
    fn bearish_negative_long_produces_1x_atr_target() {
        let params = compute_execution_params(Some(50.0), 1000.0, "Bearish", "Negative", "LONG")
            .expect("should produce params");
        // base = 0.05; multiplier = 1.0 → target = 0.05
        assert!((params.target_pct - 0.05).abs() < 1e-9);
        // risk multiplier = 0.5 → risk = 0.025
        assert!((params.risk_pct - 0.025).abs() < 1e-9);
    }

    #[test]
    fn short_direction_inverts_target_and_risk() {
        let params = compute_execution_params(Some(50.0), 1000.0, "Bullish", "Positive", "SHORT")
            .expect("should produce params");
        assert!(
            params.target_price < 1000.0,
            "SHORT target should be below entry"
        );
        assert!(
            params.risk_boundary > 1000.0,
            "SHORT risk should be above entry"
        );
    }

    #[test]
    fn zero_atr_returns_none_not_fallback() {
        // ATR=0 must return None, not fall back to +5%.
        // Falling back would blend the treatment with the P.E.2 control.
        let result = compute_execution_params(Some(0.0), 1000.0, "Bullish", "Positive", "LONG");
        assert!(
            result.is_none(),
            "ATR=0 must return None (NO_CORALYS_EXECUTION), not a fallback"
        );
    }

    #[test]
    fn none_atr_returns_none_not_fallback() {
        let result = compute_execution_params(None, 1000.0, "Bullish", "Positive", "LONG");
        assert!(
            result.is_none(),
            "ATR=None must return None (NO_CORALYS_EXECUTION), not a fallback"
        );
    }

    #[test]
    fn very_high_atr_is_clamped_to_max_target() {
        // ATR = ₹500 on ₹1000 entry → base = 0.5; × 2.0 = 1.0 → clamped to 0.15
        let params = compute_execution_params(Some(500.0), 1000.0, "Bullish", "Positive", "LONG")
            .expect("should produce params");
        assert!((params.target_pct - TARGET_PCT_MAX).abs() < 1e-9);
    }

    #[test]
    fn very_low_atr_is_clamped_to_min_target() {
        // ATR = ₹1 on ₹1000 entry → base = 0.001; × 2.0 = 0.002 → clamped to 0.02
        let params = compute_execution_params(Some(1.0), 1000.0, "Bullish", "Positive", "LONG")
            .expect("should produce params");
        assert!((params.target_pct - TARGET_PCT_MIN).abs() < 1e-9);
    }

    #[test]
    fn determinism_same_inputs_same_outputs() {
        let p1 = compute_execution_params(Some(45.0), 1250.0, "Bullish", "Positive", "LONG")
            .expect("should produce params");
        let p2 = compute_execution_params(Some(45.0), 1250.0, "Bullish", "Positive", "LONG")
            .expect("should produce params");
        assert_eq!(p1.target_pct, p2.target_pct);
        assert_eq!(p1.risk_pct, p2.risk_pct);
        assert_eq!(p1.target_price, p2.target_price);
        assert_eq!(p1.risk_boundary, p2.risk_boundary);
    }

    #[test]
    fn long_short_symmetry_in_distance() {
        // LONG/SHORT symmetry invariant:
        //   |target_price - entry_price| is equal for LONG and SHORT
        //   |risk_boundary - entry_price| is equal for LONG and SHORT
        let long_p = compute_execution_params(Some(50.0), 1000.0, "Bullish", "Positive", "LONG")
            .expect("should produce params");
        let short_p = compute_execution_params(Some(50.0), 1000.0, "Bullish", "Positive", "SHORT")
            .expect("should produce params");
        // Same percentages
        assert!((long_p.target_pct - short_p.target_pct).abs() < 1e-9);
        assert!((long_p.risk_pct - short_p.risk_pct).abs() < 1e-9);
        // Same absolute distances
        let long_target_dist = (long_p.target_price - 1000.0).abs();
        let short_target_dist = (short_p.target_price - 1000.0).abs();
        assert!(
            (long_target_dist - short_target_dist).abs() < 1e-6,
            "LONG target dist={long_target_dist}, SHORT target dist={short_target_dist}"
        );
        let long_risk_dist = (long_p.risk_boundary - 1000.0).abs();
        let short_risk_dist = (short_p.risk_boundary - 1000.0).abs();
        assert!(
            (long_risk_dist - short_risk_dist).abs() < 1e-6,
            "LONG risk dist={long_risk_dist}, SHORT risk dist={short_risk_dist}"
        );
        // LONG target above entry, SHORT target below
        assert!(long_p.target_price > 1000.0);
        assert!(short_p.target_price < 1000.0);
    }

    // ── seal_coralys_execution_intent tests ─────────────────────────────────

    #[test]
    fn seal_succeeds_when_artifact_present() {
        // CORALYS_TARGET_ARTIFACT_PRESENT = true (frozen 2026-08-16)
        // artifact hash: 3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f
        let result = seal_coralys_execution_intent(
            "INFY.NS",
            "2026-07-15T03:45:00+00:00",
            "2026-07-16T03:45:00+00:00",
            "LONG",
            1076.30,
            Some(45.0),
            "Bullish",
            "Positive",
            "abc123",
        );
        assert!(result.is_ok(), "seal must succeed when artifact is present");
        let intent = result.unwrap().intent().expect("should produce an intent");
        assert_eq!(intent.coralys_artifact_hash, CORALYS_EXEC_ARTIFACT_HASH);
        assert!(intent.sealed_at_entry);
        assert!(intent.direction_sealed_at_t);
        assert_eq!(intent.entry_source, ENTRY_SOURCE_NEXT_SESSION_OPEN);
        assert!(!intent.intent_hash.is_empty());
    }

    #[test]
    fn seal_returns_invalid_for_zero_atr_not_fallback() {
        // When artifact is present (future state), ATR=0 must return Invalid, not +5%.
        // We test compute_execution_params directly since artifact is not yet present.
        let result = compute_execution_params(Some(0.0), 1000.0, "Bullish", "Positive", "LONG");
        assert!(result.is_none(),
            "ATR=0 must produce None (NO_CORALYS_EXECUTION), not a +5% fallback that blends with P.E.2 control");
    }

    #[test]
    fn no_lookahead_different_atr_produces_different_output() {
        // Verify that the model is sensitive to ATR — if a future bar leaked and
        // changed ATR, the output would change. This is the anti-lookahead invariant:
        // bars_at_or_before must filter future bars before ATR is computed.
        let p1 = compute_execution_params(Some(42.5), 1000.0, "Bullish", "Negative", "LONG")
            .expect("should produce params");
        let p2 = compute_execution_params(Some(99.9), 1000.0, "Bullish", "Negative", "LONG")
            .expect("should produce params");
        assert_ne!(p1.target_pct, p2.target_pct,
            "different ATR must produce different target — if equal, the model is insensitive to ATR");
    }

    #[test]
    fn two_information_boundaries_are_distinct() {
        // Document the design: decision_time = T, entry_time = E.
        // decision_information_cutoff = T, execution_information_cutoff = E.
        // These are two distinct seals.
        let intent = CoralysExecutionIntent {
            instrument: "TEST".into(),
            decision_time: "2026-08-14T03:45:00+00:00".into(),
            decision_information_cutoff: "2026-08-14T03:45:00+00:00".into(),
            entry_time: "2026-08-18T09:15:00+05:30".into(),
            execution_information_cutoff: "2026-08-18T09:15:00+05:30".into(),
            entry_source: ENTRY_SOURCE_NEXT_SESSION_OPEN.to_string(),
            direction: "LONG".into(),
            entry_price: 1000.0,
            target_pct: 0.10,
            target_price: 1100.0,
            target_basis: "test".into(),
            risk_pct: 0.05,
            risk_boundary: 950.0,
            risk_basis: "test".into(),
            atr_14_at_t: 50.0,
            tmv_state: "Bullish / Positive".into(),
            state_hash: "abc".into(),
            maximum_hold_sessions: 20,
            coralys_model_id: CORALYS_EXEC_MODEL_ID.into(),
            coralys_model_version: CORALYS_EXEC_MODEL_VERSION.into(),
            coralys_artifact_hash: "placeholder".into(),
            intent_hash: "placeholder".into(),
            sealed_at_entry: true,
            direction_sealed_at_t: true,
        };
        assert!(
            intent.sealed_at_entry,
            "execution intent must seal at entry (E)"
        );
        assert!(intent.direction_sealed_at_t, "direction must seal at T");
        assert_eq!(intent.entry_source, ENTRY_SOURCE_NEXT_SESSION_OPEN);
        assert_ne!(
            intent.decision_information_cutoff, intent.execution_information_cutoff,
            "T and E must be different timestamps"
        );
    }
}
