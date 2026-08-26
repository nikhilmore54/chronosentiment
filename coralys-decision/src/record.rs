//! MVP-001 — DecisionRecord schema (frozen).
//!
//! This module defines the canonical `DecisionRecord` and all sub-types that
//! constitute a certified Coralys decision.
//!
//! **Invariants (from CORALYS_DECISION_INTELLIGENCE_MVP_V01.md):**
//! - A `DecisionRecord` is immutable once certified.
//! - No field inside `DecisionCore` or `Certification` may be derived from
//!   information that post-dates `decision_timestamp`.
//! - `ExecutionRecord` and `OutcomeRecord` are lifecycle appendages; they do
//!   not modify the original decision.
//! - `EvidenceRecord` fields are null until validated research datasets exist.
//! - The product must never manufacture `confidence`, `probability_of_success`,
//!   `expected_return`, `ranking_score`, or `quality_score`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Identity ────────────────────────────────────────────────────────────────

/// Immutable identity of a certified decision.
///
/// `decision_id` is deterministic and suitable for joining all lifecycle events.
/// Format: `coralys-{INSTRUMENT}-{TIMESTAMP_UTC}-{SEQ}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionIdentity {
    /// Unique, immutable, deterministic decision identifier.
    pub decision_id: String,
    /// Instrument symbol (e.g. `"ADANIENT.NS"`).
    pub instrument: String,
    /// Authoritative temporal boundary — no post-T information may enter the
    /// certified decision.
    pub decision_timestamp: DateTime<Utc>,
}

// ─── Certification ───────────────────────────────────────────────────────────

/// Certification status of a decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CertificationStatus {
    /// Decision has been certified — provenance is complete and verifiable.
    Certified,
    /// Decision is pending certification (transient; must not be displayed).
    Pending,
    /// Certification failed — provenance could not be established.
    Failed,
}

/// Provenance record that allows independent reproduction of the decision.
///
/// A reviewer must be able to answer:
/// > "Exactly which policy, execution artifact, data snapshot, and pipeline
/// > version produced this decision?"
///
/// The four provenance fields are **distinct** objects:
/// - `policy_artifact_hash` — the frozen C3-002 policy artifact (Search #2).
/// - `execution_artifact_hash` — the Coralys execution/risk model artifact
///   (e.g. `CORALYS_V0_ATR_TMV`). `None` when no execution model was applied.
/// - `data_snapshot_id` — identity of the market data snapshot available at T.
/// - `decision_pipeline` — the pipeline version that performed the translation
///   (e.g. `"C3-002"`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Certification {
    pub status: CertificationStatus,
    /// SHA-256 hex digest of the frozen policy artifact (C3-002 / Search #2).
    pub policy_artifact_hash: String,
    /// SHA-256 hex digest of the Coralys execution/risk model artifact.
    /// `None` when no execution model was applied at decision time.
    pub execution_artifact_hash: Option<String>,
    /// Decision pipeline identifier (e.g. `"C3-002"`).
    pub decision_pipeline: String,
    /// Timestamp at which certification was recorded (≥ `decision_timestamp`).
    pub certified_timestamp: DateTime<Utc>,
    /// Identity of the market data snapshot available at decision time.
    pub data_snapshot_id: String,
}

// ─── Decision core ───────────────────────────────────────────────────────────

/// Direction of the certified decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    Long,
    Short,
    NoTrade,
}

/// The output of the canonical Coralys decision pipeline, persisted verbatim.
///
/// **Do not add fields that Coralys does not currently produce.**
/// In particular, do not add `confidence`, `probability_of_success`,
/// `expected_return`, `ranking_score`, or `quality_score`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionCore {
    pub direction: Direction,
    /// Trend label from certified TMV state (e.g. `"Bullish"`, `"Bearish"`).
    pub trend: String,
    /// Momentum label from certified TMV state (e.g. `"Positive"`, `"Negative"`).
    pub momentum: String,
    /// Volatility presence from certified TMV state (e.g. `"present"`, `"absent"`).
    pub volatility: String,
    /// Target price sealed at decision time. `None` when not yet determined.
    pub target_price: Option<f64>,
    /// ATR-14 in price units at decision time T (certified from bars ≤ T).
    /// Used by the execution model at E to compute target/risk from actual fill.
    /// `None` when not available from the data snapshot.
    #[serde(default)]
    pub atr_14: Option<f64>,
    /// Last traded price / previous close at decision time T.
    /// This is the reference price for the recommendation — NOT the execution price.
    /// Label as "LTP / Reference" in the UI until actual execution is recorded.
    #[serde(default)]
    pub reference_price: Option<f64>,
    /// Next NSE trading session date (YYYY-MM-DD) this decision applies to.
    /// Derived as the next weekday after `decision_timestamp`.
    #[serde(default)]
    pub effective_session: Option<String>,
}

// ─── Reference Risk ──────────────────────────────────────────────────────────

/// Status of the reference risk boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReferenceRiskStatus {
    /// This is a reference boundary, not a claim of optimality.
    Reference,
}

/// The current Coralys execution boundary, exposed as a **Reference Risk
/// Boundary** — not an optimal stop.
///
/// Future research may replace `boundary_type` without changing this schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReferenceRisk {
    /// Price level of the reference risk boundary at decision time.
    pub boundary_price: Option<f64>,
    /// Identifier of the boundary algorithm (e.g. `"CORALYS_V0_ATR_TMV"`).
    pub boundary_type: String,
    pub status: ReferenceRiskStatus,
}

// ─── Execution ───────────────────────────────────────────────────────────────

/// User execution status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionStatus {
    /// No execution information has been recorded.
    NotRecorded,
    /// The user chose not to act on this decision.
    UserIgnored,
    /// The user executed this decision.
    UserExecuted,
    /// The user cancelled a previously recorded execution.
    UserCancelled,
}

/// User execution record — appended after the decision is certified.
///
/// **Critical invariant:** Coralys must never infer `quantity` or `allocation`
/// from universe size, recommendation rank, conviction, available capital,
/// historical return, or signal density.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub status: ExecutionStatus,
    /// Timestamp at which the user recorded the execution. `None` when not executed.
    pub execution_timestamp: Option<DateTime<Utc>>,
    /// Quantity traded. Populated only when the user supplies it explicitly.
    pub quantity: Option<f64>,
    /// Execution price. Populated only when the user supplies it explicitly.
    pub execution_price: Option<f64>,
    /// Source of the execution record (always `"USER"` for manual entries).
    pub execution_source: Option<String>,
}

impl Default for ExecutionRecord {
    fn default() -> Self {
        Self {
            status: ExecutionStatus::NotRecorded,
            execution_timestamp: None,
            quantity: None,
            execution_price: None,
            execution_source: None,
        }
    }
}

// ─── Outcome ─────────────────────────────────────────────────────────────────

/// Outcome status of a decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutcomeStatus {
    /// Observation window has not yet closed.
    Open,
    /// Target price was reached within the observation window.
    Target,
    /// Reference risk boundary was reached within the observation window.
    ReferenceRisk,
    /// Observation horizon elapsed without target or risk boundary being reached.
    Horizon,
    /// User closed the position before the horizon.
    UserClosed,
}

/// Outcome record — appended only after the observation boundary has passed.
///
/// **Temporal invariant:** An outcome must never be present on the original
/// certified decision object before its observation boundary has passed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub status: OutcomeStatus,
    pub exit_reason: Option<String>,
    pub exit_timestamp: Option<DateTime<Utc>>,
    pub exit_price: Option<f64>,
    pub realized_pnl: Option<f64>,
}

impl Default for OutcomeRecord {
    fn default() -> Self {
        Self {
            status: OutcomeStatus::Open,
            exit_reason: None,
            exit_timestamp: None,
            exit_price: None,
            realized_pnl: None,
        }
    }
}

// ─── Evidence ────────────────────────────────────────────────────────────────

/// Evidence enrichment — populated only when validated research datasets exist.
///
/// All fields are `None` in MVP v0.1. The stop research dataset is an
/// evidence/research asset, not a source of new decision-time rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EvidenceRecord {
    pub similar_decisions_count: Option<u32>,
    pub historical_target_rate: Option<f64>,
    pub median_mae_pct: Option<f64>,
    pub p90_mae_pct: Option<f64>,
    pub median_mfe_pct: Option<f64>,
    pub median_time_to_target_sessions: Option<f64>,
}

// ─── DecisionRecord ──────────────────────────────────────────────────────────

/// The canonical, immutable product object.
///
/// Structure:
/// ```text
/// DecisionRecord
/// │
/// ├── identity      — who, what, when (immutable)
/// ├── certification — provenance (immutable)
/// ├── decision      — Coralys output (immutable)
/// ├── reference_risk — current risk boundary (immutable at seal time)
/// ├── execution     — user action (append-only lifecycle)
/// ├── outcome       — market result (append-only, post-horizon only)
/// └── evidence      — research enrichment (null until v0.2+)
/// ```
///
/// The `identity`, `certification`, `decision`, and `reference_risk` sections
/// are sealed at certification time and must never be mutated afterward.
/// `execution`, `outcome`, and `evidence` are lifecycle appendages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub identity: DecisionIdentity,
    pub certification: Certification,
    pub decision: DecisionCore,
    pub reference_risk: ReferenceRisk,
    pub execution: ExecutionRecord,
    pub outcome: OutcomeRecord,
    pub evidence: EvidenceRecord,
}

impl DecisionRecord {
    /// Returns `true` if the decision is certified and the temporal firewall
    /// invariant can be checked: all decision-time fields must have timestamps
    /// ≤ `identity.decision_timestamp`.
    pub fn is_certified(&self) -> bool {
        self.certification.status == CertificationStatus::Certified
    }

    /// Returns `true` if the outcome observation window is still open.
    pub fn is_open(&self) -> bool {
        self.outcome.status == OutcomeStatus::Open
    }

    /// Reproducibility tuple — the minimum set of fields required to
    /// reconstruct this decision from its provenance.
    pub fn reproducibility_tuple(&self) -> ReproducibilityTuple {
        ReproducibilityTuple {
            policy_artifact_hash: self.certification.policy_artifact_hash.clone(),
            execution_artifact_hash: self.certification.execution_artifact_hash.clone(),
            data_snapshot_id: self.certification.data_snapshot_id.clone(),
            decision_timestamp: self.identity.decision_timestamp,
            instrument: self.identity.instrument.clone(),
            decision_pipeline: self.certification.decision_pipeline.clone(),
        }
    }
}

/// Minimum tuple required to reproduce a certified decision (AC-06).
///
/// Contains all four provenance fields needed to answer:
/// - Which policy generated this decision?
/// - Which execution/risk artifact accompanied it?
/// - Which data snapshot was available?
/// - Which pipeline version performed the translation?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproducibilityTuple {
    pub policy_artifact_hash: String,
    pub execution_artifact_hash: Option<String>,
    pub data_snapshot_id: String,
    pub decision_timestamp: DateTime<Utc>,
    pub instrument: String,
    pub decision_pipeline: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_record() -> DecisionRecord {
        let ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 15, 0).unwrap();
        DecisionRecord {
            identity: DecisionIdentity {
                decision_id: "coralys-ADANIENT-20260817T101500Z-001".into(),
                instrument: "ADANIENT.NS".into(),
                decision_timestamp: ts,
            },
            certification: Certification {
                status: CertificationStatus::Certified,
                policy_artifact_hash:
                    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121".into(),
                execution_artifact_hash: Some(
                    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f".into(),
                ),
                decision_pipeline: "C3-002".into(),
                certified_timestamp: ts,
                data_snapshot_id: "snapshot-20260817T101500Z".into(),
            },
            decision: DecisionCore {
                direction: Direction::Long,
                trend: "Bullish".into(),
                momentum: "Positive".into(),
                volatility: "present".into(),
                target_price: Some(1234.50),
                atr_14: None,
                reference_price: None,
                effective_session: None,
            },
            reference_risk: ReferenceRisk {
                boundary_price: Some(1180.25),
                boundary_type: "CORALYS_V0_ATR_TMV".into(),
                status: ReferenceRiskStatus::Reference,
            },
            execution: ExecutionRecord::default(),
            outcome: OutcomeRecord::default(),
            evidence: EvidenceRecord::default(),
        }
    }

    #[test]
    fn certified_record_is_certified() {
        let r = sample_record();
        assert!(r.is_certified());
    }

    #[test]
    fn new_record_is_open() {
        let r = sample_record();
        assert!(r.is_open());
    }

    #[test]
    fn execution_default_is_not_recorded() {
        let exec = ExecutionRecord::default();
        assert_eq!(exec.status, ExecutionStatus::NotRecorded);
        assert!(exec.quantity.is_none(), "no inferred allocation");
        assert!(exec.execution_price.is_none());
    }

    #[test]
    fn evidence_default_is_all_null() {
        let ev = EvidenceRecord::default();
        assert!(ev.similar_decisions_count.is_none());
        assert!(ev.median_mae_pct.is_none());
        assert!(ev.p90_mae_pct.is_none());
    }

    #[test]
    fn reproducibility_tuple_contains_required_fields() {
        let r = sample_record();
        let t = r.reproducibility_tuple();
        assert!(!t.policy_artifact_hash.is_empty());
        assert!(t.execution_artifact_hash.is_some());
        assert!(!t.data_snapshot_id.is_empty());
        assert!(!t.instrument.is_empty());
        assert!(!t.decision_pipeline.is_empty());
    }

    #[test]
    fn record_round_trips_json() {
        let r = sample_record();
        let json = serde_json::to_string(&r).expect("serialize");
        let back: DecisionRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r, back);
    }

    #[test]
    fn no_confidence_field_in_decision_core() {
        // AC-07: the product must not manufacture confidence scores.
        let json = serde_json::to_string(&sample_record().decision).unwrap();
        assert!(!json.contains("confidence"));
        assert!(!json.contains("probability"));
        assert!(!json.contains("ranking"));
    }

    #[test]
    fn no_allocation_field_in_execution() {
        // AC-08: no capital allocation.
        let json = serde_json::to_string(&sample_record().execution).unwrap();
        assert!(!json.contains("allocation"));
        assert!(!json.contains("capital"));
        assert!(!json.contains("portfolio_rank"));
    }
}
