//! Coralys Execution Feedback Ledger — learning-ready evidence stream.
//!
//! Records completed execution outcomes in a form that Coralys can eventually
//! use to learn improved execution parameters (coralys-exec-v1 and beyond).
//!
//! This module does NOT modify coralys-exec-v0. The frozen v0 artifact remains
//! immutable. This ledger accumulates the experience that a future Coralys
//! learner will consume.
//!
//! Critical temporal invariant:
//!   `feedback_available_at` must be AFTER `exit_time`.
//!   Coralys must never train on feedback from a position that is still open.
//!
//! Learning rule:
//!   For any future Coralys model trained at T_train:
//!     training_set = { feedback | feedback.feedback_available_at <= T_train }
//!
//! Timestamp canonicalization:
//!   All timestamps are stored and compared as UTC ISO-8601 strings in the form
//!   "YYYY-MM-DDTHH:MM:SSZ". Lexicographic comparison is valid only for this
//!   canonical form. The `training_set_at` and `feedback_available_after_exit`
//!   functions parse timestamps to `DateTime<Utc>` before comparing.
//!
//! Learning scope:
//!   `learning_scope = ExecutionOnly` — Coralys learns target/risk configuration,
//!   NOT direction. C3-002 direction is frozen input, not a learning target.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ─── Learning scope ───────────────────────────────────────────────────────────

/// What Coralys is allowed to learn from this feedback record.
///
/// `ExecutionOnly` means Coralys learns target/risk configuration given the
/// C3-002 direction. It does NOT learn whether LONG or SHORT was correct.
/// That boundary preserves the architectural separation between decision
/// intelligence (C3-002) and execution intelligence (Coralys).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningScope {
    /// Coralys may learn: target_pct, risk_pct, execution profile.
    /// Coralys may NOT learn: direction (LONG/SHORT).
    ExecutionOnly,
}

impl LearningScope {
    pub fn label(&self) -> &str {
        match self {
            LearningScope::ExecutionOnly => "EXECUTION_ONLY",
        }
    }
}

// ─── Exit reason ─────────────────────────────────────────────────────────────

/// Why the position was exited.
///
/// `TargetGapThrough` and `RiskGapThrough` distinguish gap-through events
/// by which boundary was crossed. This is essential for learning: a
/// `TargetGapThrough` and a `RiskGapThrough` are very different outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionExitReason {
    /// Target price reached intraday.
    Target,
    /// Risk boundary reached intraday.
    Risk,
    /// Maximum hold period elapsed without target or risk.
    Horizon,
    /// Both target and risk crossed within the same OHLC bar;
    /// intraday ordering unavailable. Excluded from primary comparison.
    Ambiguous,
    /// Gap-through: price opened beyond the TARGET boundary.
    TargetGapThrough,
    /// Gap-through: price opened beyond the RISK boundary.
    RiskGapThrough,
    /// Session close exit (e.g. forced by market close).
    SessionClose,
}

impl ExecutionExitReason {
    pub fn label(&self) -> &str {
        match self {
            ExecutionExitReason::Target => "TARGET",
            ExecutionExitReason::Risk => "RISK",
            ExecutionExitReason::Horizon => "HORIZON",
            ExecutionExitReason::Ambiguous => "AMBIGUOUS",
            ExecutionExitReason::TargetGapThrough => "TARGET_GAP_THROUGH",
            ExecutionExitReason::RiskGapThrough => "RISK_GAP_THROUGH",
            ExecutionExitReason::SessionClose => "SESSION_CLOSE",
        }
    }

    /// Whether this exit is eligible for the primary execution comparison.
    /// AMBIGUOUS exits are excluded from P.E.3-C primary analysis.
    pub fn eligible_for_primary_comparison(&self) -> bool {
        !matches!(self, ExecutionExitReason::Ambiguous)
    }

    /// Whether the target boundary was reached (including gap-through).
    pub fn target_reached(&self) -> bool {
        matches!(self, ExecutionExitReason::Target | ExecutionExitReason::TargetGapThrough)
    }

    /// Whether the risk boundary was reached (including gap-through).
    pub fn risk_reached(&self) -> bool {
        matches!(self, ExecutionExitReason::Risk | ExecutionExitReason::RiskGapThrough)
    }

    pub fn horizon_reached(&self) -> bool {
        matches!(self, ExecutionExitReason::Horizon)
    }

    pub fn is_ambiguous(&self) -> bool {
        matches!(self, ExecutionExitReason::Ambiguous)
    }
}

// ─── Execution feature snapshot ───────────────────────────────────────────────

/// The complete feature state that produced the execution intent.
/// Preserved so Coralys can learn WHY a target/risk choice worked or failed,
/// not merely what the outcome was.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFeatureSnapshot {
    pub tmv_state: String,
    pub atr_14: f64,
    /// atr_14 / entry_price — the normalized volatility input.
    pub atr_14_normalized: f64,
    pub target_pct: f64,
    pub risk_pct: f64,
    pub direction: String,
    pub entry_price: f64,
}

impl ExecutionFeatureSnapshot {
    pub fn compute_hash(&self) -> String {
        let identity = serde_json::json!({
            "atr_14": format!("{:.8}", self.atr_14),
            "atr_14_normalized": format!("{:.8}", self.atr_14_normalized),
            "direction": self.direction,
            "entry_price": format!("{:.8}", self.entry_price),
            "risk_pct": format!("{:.8}", self.risk_pct),
            "target_pct": format!("{:.8}", self.target_pct),
            "tmv_state": self.tmv_state,
        });
        format!("{:x}", Sha256::digest(identity.to_string().as_bytes()))
    }
}

// ─── Feedback record ─────────────────────────────────────────────────────────

/// A completed execution outcome, ready for Coralys to learn from.
///
/// This record is append-only. Once written, it must not be modified.
/// The `feedback_available_at` field controls when Coralys may use it.
/// It is validated to be strictly after `exit_time`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFeedbackRecord {
    // ── Identity ──────────────────────────────────────────────────────────
    pub feedback_id: String,
    pub decision_id: String,
    pub instrument: String,

    // ── Decision seal (at T) ──────────────────────────────────────────────
    pub decision_time: String,
    pub decision_information_cutoff: String,
    /// Direction from C3-002. This is the frozen input — Coralys does not
    /// learn direction from this record (see learning_scope).
    pub c3_002_direction: String,
    pub state_hash: String,
    pub tmv_state: String,

    // ── Execution seal (at E) ─────────────────────────────────────────────
    pub entry_time: String,
    pub execution_information_cutoff: String,
    pub entry_source: String,
    pub entry_price: f64,

    // ── Coralys execution parameters (from sealed intent) ─────────────────
    pub coralys_model_id: String,
    pub coralys_model_version: String,
    pub coralys_artifact_hash: String,
    pub atr_14_at_t: f64,
    pub target_pct: f64,
    pub target_price: f64,
    pub risk_pct: f64,
    pub risk_boundary: f64,
    pub maximum_hold_sessions: u32,

    // ── Feature snapshot ──────────────────────────────────────────────────
    /// Complete feature state that produced the execution intent.
    /// Preserved for Coralys learning.
    pub execution_features: ExecutionFeatureSnapshot,
    /// SHA256 of the feature snapshot. Immutable once sealed.
    pub execution_feature_hash: String,

    // ── Outcome ───────────────────────────────────────────────────────────
    pub exit_time: String,
    pub exit_price: f64,
    pub exit_reason: ExecutionExitReason,
    pub holding_sessions: u32,

    // ── Decision value ────────────────────────────────────────────────────
    /// Realized return as a fraction (e.g. 0.05 = +5%).
    /// Positive = gain for the direction taken.
    pub realized_return: f64,
    pub target_reached: bool,
    pub risk_reached: bool,
    pub horizon_reached: bool,
    pub ambiguous: bool,

    // ── Eligibility ───────────────────────────────────────────────────────
    /// Whether this record is eligible for the primary execution comparison.
    /// AMBIGUOUS exits are excluded.
    pub eligible_for_primary_comparison: bool,

    // ── Learning control ──────────────────────────────────────────────────
    /// What Coralys is allowed to learn from this record.
    /// Always `ExecutionOnly` for P.E.3 — Coralys learns target/risk, not direction.
    pub learning_scope: LearningScope,

    /// The timestamp at which Coralys is ALLOWED to learn from this outcome.
    /// Validated to be strictly after `exit_time`.
    /// A future Coralys model trained at T_train may only use records where
    /// feedback_available_at <= T_train.
    pub feedback_available_at: String,

    // ── Integrity ─────────────────────────────────────────────────────────
    pub record_hash: String,
}

impl ExecutionFeedbackRecord {
    /// Compute the record hash from the immutable fields.
    pub fn compute_hash(&self) -> String {
        let identity = serde_json::json!({
            "ambiguous": self.ambiguous,
            "atr_14_at_t": format!("{:.8}", self.atr_14_at_t),
            "c3_002_direction": self.c3_002_direction,
            "coralys_artifact_hash": self.coralys_artifact_hash,
            "coralys_model_id": self.coralys_model_id,
            "coralys_model_version": self.coralys_model_version,
            "decision_id": self.decision_id,
            "decision_time": self.decision_time,
            "entry_price": format!("{:.8}", self.entry_price),
            "entry_source": self.entry_source,
            "entry_time": self.entry_time,
            "execution_feature_hash": self.execution_feature_hash,
            "exit_price": format!("{:.8}", self.exit_price),
            "exit_reason": self.exit_reason.label(),
            "exit_time": self.exit_time,
            "feedback_available_at": self.feedback_available_at,
            "holding_sessions": self.holding_sessions,
            "instrument": self.instrument,
            "learning_scope": self.learning_scope.label(),
            "maximum_hold_sessions": self.maximum_hold_sessions,
            "realized_return": format!("{:.8}", self.realized_return),
            "risk_pct": format!("{:.8}", self.risk_pct),
            "state_hash": self.state_hash,
            "target_pct": format!("{:.8}", self.target_pct),
            "tmv_state": self.tmv_state,
        });
        format!("{:x}", Sha256::digest(identity.to_string().as_bytes()))
    }

    /// Verify the record hash matches the current fields.
    pub fn verify_integrity(&self) -> bool {
        self.record_hash == self.compute_hash()
    }
}

// ─── Timestamp helpers ────────────────────────────────────────────────────────

/// Parse an ISO-8601 timestamp string to `DateTime<Utc>`.
/// Returns an error if the string cannot be parsed.
pub fn parse_utc(ts: &str) -> Result<DateTime<Utc>, String> {
    ts.parse::<DateTime<Utc>>()
        .map_err(|e| format!("cannot parse timestamp '{ts}': {e}"))
}

/// Validate that `feedback_available_at` is strictly after `exit_time`.
/// This is the core temporal invariant of the feedback ledger.
pub fn validate_feedback_timing(exit_time: &str, feedback_available_at: &str) -> Result<(), String> {
    let exit = parse_utc(exit_time)?;
    let available = parse_utc(feedback_available_at)?;
    if available <= exit {
        return Err(format!(
            "feedback_available_at ({feedback_available_at}) must be strictly after \
             exit_time ({exit_time}); Coralys cannot learn from a position before it closes"
        ));
    }
    Ok(())
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Build and seal an execution feedback record.
///
/// Returns an error if `feedback_available_at` is not strictly after `exit_time`.
#[allow(clippy::too_many_arguments)]
pub fn seal_execution_feedback(
    feedback_id: String,
    decision_id: String,
    instrument: String,
    decision_time: String,
    c3_002_direction: String,
    state_hash: String,
    tmv_state: String,
    entry_time: String,
    entry_source: String,
    entry_price: f64,
    coralys_model_id: String,
    coralys_model_version: String,
    coralys_artifact_hash: String,
    atr_14_at_t: f64,
    target_pct: f64,
    target_price: f64,
    risk_pct: f64,
    risk_boundary: f64,
    maximum_hold_sessions: u32,
    exit_time: String,
    exit_price: f64,
    exit_reason: ExecutionExitReason,
    holding_sessions: u32,
    realized_return: f64,
    feedback_available_at: String,
) -> Result<ExecutionFeedbackRecord, String> {
    // Validate temporal invariant before sealing.
    validate_feedback_timing(&exit_time, &feedback_available_at)?;

    let target_reached = exit_reason.target_reached();
    let risk_reached = exit_reason.risk_reached();
    let horizon_reached = exit_reason.horizon_reached();
    let ambiguous = exit_reason.is_ambiguous();
    let eligible = exit_reason.eligible_for_primary_comparison();

    let atr_14_normalized = if entry_price > 0.0 { atr_14_at_t / entry_price } else { 0.0 };

    let features = ExecutionFeatureSnapshot {
        tmv_state: tmv_state.clone(),
        atr_14: atr_14_at_t,
        atr_14_normalized,
        target_pct,
        risk_pct,
        direction: c3_002_direction.clone(),
        entry_price,
    };
    let execution_feature_hash = features.compute_hash();

    let mut record = ExecutionFeedbackRecord {
        feedback_id,
        decision_id,
        instrument,
        decision_time: decision_time.clone(),
        decision_information_cutoff: decision_time,
        c3_002_direction,
        state_hash,
        tmv_state,
        entry_time: entry_time.clone(),
        execution_information_cutoff: entry_time,
        entry_source,
        entry_price,
        coralys_model_id,
        coralys_model_version,
        coralys_artifact_hash,
        atr_14_at_t,
        target_pct,
        target_price,
        risk_pct,
        risk_boundary,
        maximum_hold_sessions,
        execution_features: features,
        execution_feature_hash,
        exit_time,
        exit_price,
        exit_reason,
        holding_sessions,
        realized_return,
        target_reached,
        risk_reached,
        horizon_reached,
        ambiguous,
        eligible_for_primary_comparison: eligible,
        learning_scope: LearningScope::ExecutionOnly,
        feedback_available_at,
        record_hash: String::new(),
    };

    record.record_hash = record.compute_hash();
    Ok(record)
}

// ─── Learning dataset queries ─────────────────────────────────────────────────

/// Filter a feedback ledger to produce the training set available at T_train.
///
/// Uses `DateTime<Utc>` comparison — not lexicographic string comparison —
/// to ensure correctness regardless of timezone representation.
pub fn training_set_at<'a>(
    ledger: &'a [ExecutionFeedbackRecord],
    t_train: &str,
) -> Result<Vec<&'a ExecutionFeedbackRecord>, String> {
    let t = parse_utc(t_train)?;
    Ok(ledger
        .iter()
        .filter(|r| {
            parse_utc(&r.feedback_available_at)
                .map(|fa| fa <= t)
                .unwrap_or(false)
        })
        .collect())
}

/// Filter to only records eligible for primary execution comparison.
/// Excludes AMBIGUOUS exits.
pub fn primary_comparison_set(
    ledger: &[ExecutionFeedbackRecord],
) -> Vec<&ExecutionFeedbackRecord> {
    ledger
        .iter()
        .filter(|r| r.eligible_for_primary_comparison)
        .collect()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(
        exit_reason: ExecutionExitReason,
        exit_time: &str,
        feedback_at: &str,
    ) -> Result<ExecutionFeedbackRecord, String> {
        seal_execution_feedback(
            "fb-001".into(),
            "dec-001".into(),
            "INFY.NS".into(),
            "2026-05-15T03:45:00Z".into(),
            "LONG".into(),
            "abc123".into(),
            "Bullish / Positive".into(),
            "2026-05-16T03:45:00Z".into(),
            "NEXT_SESSION_OPEN".into(),
            1076.30,
            "coralys-exec-v0".into(),
            "0.1.0".into(),
            "placeholder".into(),
            45.0,
            0.10,
            1183.93,
            0.05,
            1022.49,
            20,
            exit_time.into(),
            1183.93,
            exit_reason,
            2,
            0.10,
            feedback_at.into(),
        )
    }

    #[test]
    fn record_hash_is_stable() {
        let r = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert!(r.verify_integrity(), "record hash must verify");
    }

    #[test]
    fn feedback_available_at_must_be_after_exit_time() {
        // feedback_available_at == exit_time → error
        let result = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-18T03:45:00Z",
        );
        assert!(result.is_err(), "feedback_available_at == exit_time must be rejected");

        // feedback_available_at before exit_time → error
        let result = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-17T00:00:00Z",
        );
        assert!(result.is_err(), "feedback_available_at before exit_time must be rejected");

        // feedback_available_at after exit_time → ok
        let result = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        );
        assert!(result.is_ok(), "feedback_available_at after exit_time must be accepted");
    }

    #[test]
    fn ambiguous_exit_is_excluded_from_primary_comparison() {
        let r = make_record(
            ExecutionExitReason::Ambiguous,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert!(!r.eligible_for_primary_comparison);
        assert!(r.ambiguous);
    }

    #[test]
    fn target_gap_through_is_target_reached() {
        let r = make_record(
            ExecutionExitReason::TargetGapThrough,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert!(r.target_reached);
        assert!(!r.risk_reached);
        assert!(r.eligible_for_primary_comparison);
    }

    #[test]
    fn risk_gap_through_is_risk_reached() {
        let r = make_record(
            ExecutionExitReason::RiskGapThrough,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert!(r.risk_reached);
        assert!(!r.target_reached);
        assert!(r.eligible_for_primary_comparison);
    }

    #[test]
    fn target_gap_through_and_risk_gap_through_produce_different_hashes() {
        let r1 = make_record(
            ExecutionExitReason::TargetGapThrough,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        let r2 = make_record(
            ExecutionExitReason::RiskGapThrough,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert_ne!(r1.record_hash, r2.record_hash,
            "TargetGapThrough and RiskGapThrough must produce different hashes");
    }

    #[test]
    fn learning_scope_is_execution_only() {
        let r = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert_eq!(r.learning_scope, LearningScope::ExecutionOnly,
            "P.E.3 feedback must be ExecutionOnly — Coralys learns target/risk, not direction");
    }

    #[test]
    fn execution_feature_hash_is_stable() {
        let r = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert_eq!(r.execution_feature_hash, r.execution_features.compute_hash());
    }

    #[test]
    fn training_set_uses_datetime_comparison_not_lexicographic() {
        // Two timestamps representing the same instant in different timezones.
        // Lexicographic comparison would give wrong results; DateTime<Utc> is correct.
        let r1 = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",  // UTC
        ).expect("should seal");
        let ledger = vec![r1];

        // T_train in +05:30 representing the same instant as 2026-05-18T18:30:00Z
        let ts = training_set_at(&ledger, "2026-05-20T00:00:00+05:30")
            .expect("should parse");
        assert_eq!(ts.len(), 1, "UTC and +05:30 timestamps for same instant must compare correctly");
    }

    #[test]
    fn training_set_respects_feedback_available_at() {
        let r1 = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        let r2 = make_record(
            ExecutionExitReason::Horizon,
            "2026-06-14T03:45:00Z",
            "2026-06-15T00:00:00Z",
        ).expect("should seal");
        let r3 = make_record(
            ExecutionExitReason::Risk,
            "2026-06-30T03:45:00Z",
            "2026-07-01T00:00:00Z",
        ).expect("should seal");
        let ledger = vec![r1, r2, r3];

        // At 2026-05-20, only r1 is available
        let ts = training_set_at(&ledger, "2026-05-20T00:00:00Z").expect("should parse");
        assert_eq!(ts.len(), 1);

        // At 2026-06-20, r1 and r2 are available
        let ts = training_set_at(&ledger, "2026-06-20T00:00:00Z").expect("should parse");
        assert_eq!(ts.len(), 2);

        // At 2026-07-02, all three are available
        let ts = training_set_at(&ledger, "2026-07-02T00:00:00Z").expect("should parse");
        assert_eq!(ts.len(), 3);
    }

    #[test]
    fn primary_comparison_set_excludes_ambiguous() {
        let r1 = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        let r2 = make_record(
            ExecutionExitReason::Ambiguous,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        let r3 = make_record(
            ExecutionExitReason::Horizon,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        let ledger = vec![r1, r2, r3];

        let primary = primary_comparison_set(&ledger);
        assert_eq!(primary.len(), 2, "AMBIGUOUS must be excluded from primary comparison");
        assert!(primary.iter().all(|r| !r.ambiguous));
    }

    #[test]
    fn no_c3_002_direction_field_named_direction() {
        // Verify the record uses c3_002_direction, not a bare `direction` field.
        // This prevents accidental learning of direction from execution feedback.
        let r = make_record(
            ExecutionExitReason::Target,
            "2026-05-18T03:45:00Z",
            "2026-05-19T00:00:00Z",
        ).expect("should seal");
        assert_eq!(r.c3_002_direction, "LONG");
        // The learning scope must be ExecutionOnly — direction is not a learning target.
        assert_eq!(r.learning_scope, LearningScope::ExecutionOnly);
    }
}