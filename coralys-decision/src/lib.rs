//! `coralys-decision` — Coralys Decision Intelligence core types.
//!
//! This crate implements the canonical `DecisionRecord` schema (MVP-001) and
//! the immutable append-only `DecisionLedger` (MVP-002) as specified in
//! `docs/CORALYS_DECISION_INTELLIGENCE_MVP_V01.md`.
//!
//! # Invariants
//!
//! - A certified `DecisionRecord` is **immutable** after sealing (AC-03).
//! - Lifecycle events are **append-only** (AC-05, AC-09).
//! - No capital allocation, portfolio ranking, or quantity inference (AC-08).
//! - No manufactured confidence, probability, or ranking scores (AC-07).
//! - Temporal firewall: no post-decision information enters the certified
//!   decision (AC-02).

pub mod adapter;
pub mod integration_tests;
pub mod ledger;
pub mod record;
pub mod recommendation;
pub mod temporal_firewall_tests;
pub mod traits;

pub use adapter::{
    AdapterError, DecisionRecordBuilder, SealedDecisionInput, C3_002_POLICY_ARTIFACT_HASH,
    CORALYS_EXEC_ARTIFACT_HASH,
};
pub use ledger::{DecisionEvent, DecisionEventType, DecisionLedger, LedgerError};
pub use record::{
    Certification, CertificationStatus, DecisionCore, DecisionIdentity, DecisionRecord,
    Direction, EvidenceRecord, ExecutionRecord, ExecutionStatus, OutcomeRecord, OutcomeStatus,
    ReferenceRisk, ReferenceRiskStatus, ReproducibilityTuple,
};
pub use traits::{CandidateEvaluator, DecisionMaker, DecisionPolicy};
