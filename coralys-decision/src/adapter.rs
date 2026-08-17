//! MVP-003/004 — C3-002 → DecisionLedger adapter + provenance binding.
//!
//! This module is the **only** boundary between the existing Coralys/C3-002
//! decision pipeline and the canonical `DecisionRecord` / `DecisionLedger`.
//!
//! Architecture:
//! ```text
//! adapters/chronosentiment (C3-002 output)
//!         │
//!         │  SealedDecisionInput (plain data, no crate dependency)
//!         ▼
//! DecisionRecordBuilder  ← this module
//!         │
//!         ▼
//! DecisionRecord  →  DecisionLedger::seal_decision()
//! ```
//!
//! **Design rules:**
//! - `coralys-decision` does NOT depend on `chronosentiment-adapter`.
//! - The caller (chronosentiment adapter or orchestration layer) constructs a
//!   `SealedDecisionInput` from the C3-002 output and passes it here.
//! - This module does not modify C3-002. It only translates its output.
//! - No stop-research dataset is connected here (MVP-003/004 scope only).
//! - No confidence, probability, ranking, or allocation fields are added.
//!
//! **Provenance chain (MVP-004):**
//! The four provenance fields are distinct objects:
//! - `policy_artifact_hash` — frozen C3-002 policy artifact (Search #2).
//! - `execution_artifact_hash` — Coralys execution/risk model artifact.
//! - `data_snapshot_id` — identity of the market data snapshot at T.
//! - `decision_pipeline` — pipeline version that performed the translation.
//!
//! **Temporal invariant (AC-02):**
//! All fields in the resulting `DecisionRecord` must derive from information
//! available at or before `decision_timestamp`. The builder enforces this by
//! accepting only decision-time inputs and producing null evidence/execution.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::record::{
    Certification, CertificationStatus, DecisionCore, DecisionIdentity, DecisionRecord, Direction,
    EvidenceRecord, ExecutionRecord, OutcomeRecord, ReferenceRisk, ReferenceRiskStatus,
};

// ─── Known canonical hashes ───────────────────────────────────────────────────

/// SHA-256 of the frozen C3-002 policy artifact (Search #2).
/// Source: `adapters/chronosentiment/src/decision_support/csp006_protocol.rs`
pub const C3_002_POLICY_ARTIFACT_HASH: &str =
    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121";

/// SHA-256 of the Coralys execution/risk model artifact (CORALYS_V0_ATR_TMV).
/// Source: `adapters/chronosentiment/src/decision_support/coralys_execution_model.rs`
pub const CORALYS_EXEC_ARTIFACT_HASH: &str =
    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f";

// ─── Adapter errors ───────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdapterError {
    #[error("instrument is required")]
    MissingInstrument,

    #[error("policy_artifact_hash is required")]
    MissingPolicyArtifactHash,

    #[error("data_snapshot_id is required")]
    MissingDataSnapshotId,

    #[error("decision_pipeline is required")]
    MissingDecisionPipeline,

    #[error("invalid direction: '{0}' — expected LONG, SHORT, or NO_TRADE")]
    InvalidDirection(String),

    #[error("temporal firewall violation: certified_timestamp {certified_ts} is before decision_timestamp {decision_ts}")]
    CertifiedBeforeDecision {
        decision_ts: DateTime<Utc>,
        certified_ts: DateTime<Utc>,
    },

    #[error("provenance binding failure: policy_artifact_hash '{supplied}' does not match the known C3-002 hash '{expected}'")]
    PolicyHashMismatch { supplied: String, expected: String },

    #[error("provenance binding failure: execution_artifact_hash '{supplied}' does not match the known Coralys execution artifact hash '{expected}'")]
    ExecutionHashMismatch { supplied: String, expected: String },
}

// ─── Input types ─────────────────────────────────────────────────────────────

/// Plain-data input from the C3-002 decision pipeline.
///
/// The caller (chronosentiment adapter) populates this from a
/// `SealedDecisionRecord` without creating a crate dependency on
/// `chronosentiment-adapter`.
///
/// All fields reflect information available **at or before** `decision_timestamp`.
///
/// **Provenance fields (MVP-004):**
/// - `policy_artifact_hash` — SHA-256 of the C3-002 policy artifact.
/// - `execution_artifact_hash` — SHA-256 of the Coralys execution/risk model
///   artifact. `None` when no execution model was applied.
/// - `data_snapshot_id` — identity of the market data snapshot at T.
/// - `decision_pipeline` — pipeline version (e.g. `"C3-002"`).
#[derive(Debug, Clone)]
pub struct SealedDecisionInput {
    /// Unique decision identifier from C3-002.
    pub decision_id: String,
    /// Instrument symbol (e.g. `"ADANIENT.NS"`).
    pub instrument: String,
    /// Decision timestamp — the authoritative temporal boundary.
    pub decision_timestamp: DateTime<Utc>,
    /// Direction from C3-002: `"LONG"`, `"SHORT"`, or `"NO_TRADE"`.
    pub direction: String,
    /// Trend label from certified TMV state.
    pub trend: String,
    /// Momentum label from certified TMV state.
    pub momentum: String,
    /// Volatility label from certified TMV state.
    pub volatility: String,
    /// Target price sealed at decision time. `None` when not yet determined.
    pub target_price: Option<f64>,
    /// SHA-256 hex digest of the frozen policy artifact (C3-002 / Search #2).
    pub policy_artifact_hash: String,
    /// SHA-256 hex digest of the Coralys execution/risk model artifact.
    /// `None` when no execution model was applied at decision time.
    pub execution_artifact_hash: Option<String>,
    /// Decision pipeline identifier (e.g. `"C3-002"`).
    pub decision_pipeline: String,
    /// Identity of the market data snapshot available at decision time.
    pub data_snapshot_id: String,
    /// Timestamp at which certification was recorded (must be ≥ `decision_timestamp`).
    pub certified_timestamp: DateTime<Utc>,
    /// Reference risk boundary price at decision time. `None` when not available.
    pub reference_risk_boundary_price: Option<f64>,
    /// Boundary type identifier (e.g. `"CORALYS_V0_ATR_TMV"`).
    pub reference_risk_boundary_type: String,
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Translates a `SealedDecisionInput` (C3-002 output) into a canonical
/// `DecisionRecord` ready for `DecisionLedger::seal_decision`.
///
/// The builder enforces:
/// - AC-01: all certification fields are present.
/// - AC-02: `certified_timestamp` ≥ `decision_timestamp`.
/// - MVP-004: provenance binding — policy and execution artifact hashes are
///   verified against the known canonical values when `strict_provenance` is
///   `true` (default for production; `false` for test fixtures).
/// - AC-07: no confidence/probability fields are added.
/// - AC-08: no allocation/quantity fields are added.
/// - AC-09: evidence is null (populated later by research enrichment).
pub struct DecisionRecordBuilder;

impl DecisionRecordBuilder {
    /// Build a `DecisionRecord` from a `SealedDecisionInput`.
    ///
    /// Provenance hashes are verified against the canonical C3-002 and Coralys
    /// execution artifact hashes. Pass `strict_provenance = false` only in
    /// test fixtures that use synthetic hashes.
    pub fn build(input: SealedDecisionInput) -> Result<DecisionRecord, AdapterError> {
        Self::build_inner(input, true)
    }

    /// Build without strict provenance hash verification (test use only).
    #[cfg(test)]
    pub fn build_unchecked(input: SealedDecisionInput) -> Result<DecisionRecord, AdapterError> {
        Self::build_inner(input, false)
    }

    fn build_inner(
        input: SealedDecisionInput,
        strict_provenance: bool,
    ) -> Result<DecisionRecord, AdapterError> {
        // Validate required fields.
        if input.instrument.is_empty() {
            return Err(AdapterError::MissingInstrument);
        }
        if input.policy_artifact_hash.is_empty() {
            return Err(AdapterError::MissingPolicyArtifactHash);
        }
        if input.data_snapshot_id.is_empty() {
            return Err(AdapterError::MissingDataSnapshotId);
        }
        if input.decision_pipeline.is_empty() {
            return Err(AdapterError::MissingDecisionPipeline);
        }

        // AC-02: temporal firewall.
        if input.certified_timestamp < input.decision_timestamp {
            return Err(AdapterError::CertifiedBeforeDecision {
                decision_ts: input.decision_timestamp,
                certified_ts: input.certified_timestamp,
            });
        }

        // MVP-004: provenance binding.
        if strict_provenance {
            if input.policy_artifact_hash != C3_002_POLICY_ARTIFACT_HASH {
                return Err(AdapterError::PolicyHashMismatch {
                    supplied: input.policy_artifact_hash.clone(),
                    expected: C3_002_POLICY_ARTIFACT_HASH.to_string(),
                });
            }
            if let Some(ref exec_hash) = input.execution_artifact_hash {
                if exec_hash != CORALYS_EXEC_ARTIFACT_HASH {
                    return Err(AdapterError::ExecutionHashMismatch {
                        supplied: exec_hash.clone(),
                        expected: CORALYS_EXEC_ARTIFACT_HASH.to_string(),
                    });
                }
            }
        }

        // Translate direction.
        let direction = Self::parse_direction(&input.direction)?;

        Ok(DecisionRecord {
            identity: DecisionIdentity {
                decision_id: input.decision_id,
                instrument: input.instrument,
                decision_timestamp: input.decision_timestamp,
            },
            certification: Certification {
                status: CertificationStatus::Certified,
                policy_artifact_hash: input.policy_artifact_hash,
                execution_artifact_hash: input.execution_artifact_hash,
                decision_pipeline: input.decision_pipeline,
                certified_timestamp: input.certified_timestamp,
                data_snapshot_id: input.data_snapshot_id,
            },
            decision: DecisionCore {
                direction,
                trend: input.trend,
                momentum: input.momentum,
                volatility: input.volatility,
                target_price: input.target_price,
            },
            reference_risk: ReferenceRisk {
                boundary_price: input.reference_risk_boundary_price,
                boundary_type: input.reference_risk_boundary_type,
                status: ReferenceRiskStatus::Reference,
            },
            // AC-04/AC-08: execution starts as NOT_RECORDED with no inferred quantity.
            execution: ExecutionRecord::default(),
            // Outcome starts OPEN — observation boundary has not passed.
            outcome: OutcomeRecord::default(),
            // AC-09: evidence is null until validated research datasets exist.
            evidence: EvidenceRecord::default(),
        })
    }

    fn parse_direction(s: &str) -> Result<Direction, AdapterError> {
        match s.to_uppercase().as_str() {
            "LONG" => Ok(Direction::Long),
            "SHORT" => Ok(Direction::Short),
            "NO_TRADE" | "NOTRADE" => Ok(Direction::NoTrade),
            other => Err(AdapterError::InvalidDirection(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(y: i32, mo: u32, d: u32, h: u32, m: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, m, s).unwrap()
    }

    /// Production-realistic input using the actual canonical hashes.
    fn canonical_input() -> SealedDecisionInput {
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        SealedDecisionInput {
            decision_id: "coralys-ADANIENT-20260817T101500Z-001".into(),
            instrument: "ADANIENT.NS".into(),
            decision_timestamp: decision_ts,
            direction: "LONG".into(),
            trend: "Bullish".into(),
            momentum: "Positive".into(),
            volatility: "present".into(),
            target_price: Some(1234.50),
            policy_artifact_hash: C3_002_POLICY_ARTIFACT_HASH.into(),
            execution_artifact_hash: Some(CORALYS_EXEC_ARTIFACT_HASH.into()),
            decision_pipeline: "C3-002".into(),
            data_snapshot_id: "snapshot-20260817T101500Z".into(),
            certified_timestamp: decision_ts,
            reference_risk_boundary_price: Some(1180.25),
            reference_risk_boundary_type: "CORALYS_V0_ATR_TMV".into(),
        }
    }

    /// Synthetic input for tests that don't need canonical hash verification.
    fn synthetic_input() -> SealedDecisionInput {
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        SealedDecisionInput {
            decision_id: "coralys-ADANIENT-20260817T101500Z-001".into(),
            instrument: "ADANIENT.NS".into(),
            decision_timestamp: decision_ts,
            direction: "LONG".into(),
            trend: "Bullish".into(),
            momentum: "Positive".into(),
            volatility: "present".into(),
            target_price: Some(1234.50),
            policy_artifact_hash: "synthetic-policy-hash".into(),
            execution_artifact_hash: Some("synthetic-exec-hash".into()),
            decision_pipeline: "C3-002".into(),
            data_snapshot_id: "snapshot-20260817T101500Z".into(),
            certified_timestamp: decision_ts,
            reference_risk_boundary_price: Some(1180.25),
            reference_risk_boundary_type: "CORALYS_V0_ATR_TMV".into(),
        }
    }

    // ── MVP-004: provenance binding tests ────────────────────────────────────

    #[test]
    fn canonical_hashes_are_correct() {
        // Verify the constants match the values established in the codebase.
        assert_eq!(
            C3_002_POLICY_ARTIFACT_HASH,
            "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121"
        );
        assert_eq!(
            CORALYS_EXEC_ARTIFACT_HASH,
            "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f"
        );
    }

    #[test]
    fn wrong_policy_hash_is_rejected_in_strict_mode() {
        let mut input = canonical_input();
        input.policy_artifact_hash = "wrong-hash".into();
        let err = DecisionRecordBuilder::build(input).unwrap_err();
        assert!(matches!(err, AdapterError::PolicyHashMismatch { .. }));
    }

    #[test]
    fn wrong_execution_hash_is_rejected_in_strict_mode() {
        let mut input = canonical_input();
        input.execution_artifact_hash = Some("wrong-exec-hash".into());
        let err = DecisionRecordBuilder::build(input).unwrap_err();
        assert!(matches!(err, AdapterError::ExecutionHashMismatch { .. }));
    }

    #[test]
    fn no_execution_hash_is_accepted() {
        // Execution model is optional — decisions without ATR/TMV risk model are valid.
        let mut input = canonical_input();
        input.execution_artifact_hash = None;
        let record = DecisionRecordBuilder::build(input).unwrap();
        assert!(record.certification.execution_artifact_hash.is_none());
    }

    // ── MVP-003 integration test: complete temporal/provenance verification ──

    #[test]
    fn build_produces_certified_record_with_correct_provenance() {
        let input = canonical_input();
        let decision_ts = input.decision_timestamp;
        let record = DecisionRecordBuilder::build(input).unwrap();

        // AC-01: certification fields present.
        assert_eq!(
            record.certification.policy_artifact_hash,
            C3_002_POLICY_ARTIFACT_HASH
        );
        assert_eq!(
            record.certification.execution_artifact_hash.as_deref(),
            Some(CORALYS_EXEC_ARTIFACT_HASH)
        );
        assert_eq!(record.certification.decision_pipeline, "C3-002");
        assert!(!record.certification.data_snapshot_id.is_empty());
        assert_eq!(
            record.certification.status,
            crate::record::CertificationStatus::Certified
        );

        // AC-02: decision_timestamp is the authoritative boundary.
        assert_eq!(record.identity.decision_timestamp, decision_ts);
        assert!(record.certification.certified_timestamp >= decision_ts);

        // AC-03: decision_id is set.
        assert_eq!(
            record.identity.decision_id,
            "coralys-ADANIENT-20260817T101500Z-001"
        );

        // AC-04/AC-08: execution is NOT_RECORDED, no quantity.
        assert_eq!(
            record.execution.status,
            crate::record::ExecutionStatus::NotRecorded
        );
        assert!(record.execution.quantity.is_none());

        // AC-05: outcome is OPEN.
        assert_eq!(record.outcome.status, crate::record::OutcomeStatus::Open);

        // AC-07: no confidence field.
        let json = serde_json::to_string(&record.decision).unwrap();
        assert!(!json.contains("confidence"));
        assert!(!json.contains("probability"));

        // AC-08: no allocation field.
        let exec_json = serde_json::to_string(&record.execution).unwrap();
        assert!(!exec_json.contains("allocation"));
        assert!(!exec_json.contains("capital"));

        // AC-09: evidence is all null.
        assert!(record.evidence.similar_decisions_count.is_none());
        assert!(record.evidence.median_mae_pct.is_none());
    }

    #[test]
    fn direction_long_maps_correctly() {
        let record = DecisionRecordBuilder::build(canonical_input()).unwrap();
        assert_eq!(record.decision.direction, Direction::Long);
    }

    #[test]
    fn direction_short_maps_correctly() {
        let mut input = canonical_input();
        input.direction = "SHORT".into();
        let record = DecisionRecordBuilder::build(input).unwrap();
        assert_eq!(record.decision.direction, Direction::Short);
    }

    #[test]
    fn direction_no_trade_maps_correctly() {
        let mut input = canonical_input();
        input.direction = "NO_TRADE".into();
        let record = DecisionRecordBuilder::build(input).unwrap();
        assert_eq!(record.decision.direction, Direction::NoTrade);
    }

    #[test]
    fn invalid_direction_is_rejected() {
        let mut input = synthetic_input();
        input.direction = "HOLD".into();
        let err = DecisionRecordBuilder::build_unchecked(input).unwrap_err();
        assert!(matches!(err, AdapterError::InvalidDirection(_)));
    }

    #[test]
    fn certified_before_decision_is_rejected() {
        // AC-02: temporal firewall
        let mut input = synthetic_input();
        input.certified_timestamp = ts(2026, 8, 16, 9, 0, 0); // before decision_ts
        let err = DecisionRecordBuilder::build_unchecked(input).unwrap_err();
        assert!(matches!(err, AdapterError::CertifiedBeforeDecision { .. }));
    }

    #[test]
    fn missing_policy_hash_is_rejected() {
        // AC-01: certification completeness
        let mut input = synthetic_input();
        input.policy_artifact_hash = String::new();
        let err = DecisionRecordBuilder::build_unchecked(input).unwrap_err();
        assert!(matches!(err, AdapterError::MissingPolicyArtifactHash));
    }

    #[test]
    fn missing_data_snapshot_id_is_rejected() {
        let mut input = synthetic_input();
        input.data_snapshot_id = String::new();
        let err = DecisionRecordBuilder::build_unchecked(input).unwrap_err();
        assert!(matches!(err, AdapterError::MissingDataSnapshotId));
    }

    #[test]
    fn reference_risk_is_reference_not_optimal() {
        // Spec §9: must not claim "optimal stop"
        let record = DecisionRecordBuilder::build(canonical_input()).unwrap();
        assert_eq!(
            record.reference_risk.status,
            ReferenceRiskStatus::Reference
        );
        let json = serde_json::to_string(&record.reference_risk).unwrap();
        assert!(!json.contains("optimal"));
        assert!(!json.contains("best_stop"));
    }

    #[test]
    fn reproducibility_tuple_is_complete() {
        // AC-06
        let record = DecisionRecordBuilder::build(canonical_input()).unwrap();
        let t = record.reproducibility_tuple();
        assert_eq!(t.policy_artifact_hash, C3_002_POLICY_ARTIFACT_HASH);
        assert_eq!(
            t.execution_artifact_hash.as_deref(),
            Some(CORALYS_EXEC_ARTIFACT_HASH)
        );
        assert_eq!(t.decision_pipeline, "C3-002");
        assert_eq!(t.instrument, "ADANIENT.NS");
        assert!(!t.data_snapshot_id.is_empty());
    }

    #[test]
    fn builder_output_seals_into_ledger() {
        // Full integration: build → seal → retrieve
        use crate::ledger::DecisionLedger;

        let record = DecisionRecordBuilder::build(canonical_input()).unwrap();
        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        let retrieved = ledger
            .get_decision("coralys-ADANIENT-20260817T101500Z-001")
            .unwrap();
        assert!(retrieved.is_certified());
        assert!(retrieved.is_open());
        assert_eq!(retrieved.identity.instrument, "ADANIENT.NS");
        assert_eq!(retrieved.certification.decision_pipeline, "C3-002");
        assert_eq!(
            retrieved.certification.policy_artifact_hash,
            C3_002_POLICY_ARTIFACT_HASH
        );
        assert_eq!(
            retrieved.certification.execution_artifact_hash.as_deref(),
            Some(CORALYS_EXEC_ARTIFACT_HASH)
        );
    }
}