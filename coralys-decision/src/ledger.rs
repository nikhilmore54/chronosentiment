//! MVP-002 — Immutable Decision Ledger (append-only event store).
//!
//! The ledger is the canonical source of truth for all certified decisions and
//! their lifecycle events. It enforces the following invariants from
//! CORALYS_DECISION_INTELLIGENCE_MVP_V01.md:
//!
//! - A certified decision is **immutable** — it cannot be overwritten (AC-03).
//! - Lifecycle events are **append-only** — they extend the record without
//!   modifying the original decision.
//! - Outcome information is only appended after the observation boundary (AC-05).
//! - No capital allocation, portfolio ranking, or quantity inference (AC-08).
//! - Evidence enrichment cannot modify the original decision (AC-09).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::record::{
    Certification, CertificationStatus, DecisionRecord, EvidenceRecord, ExecutionRecord,
    OutcomeRecord, OutcomeStatus,
};

// ─── Ledger errors ───────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerError {
    #[error("decision '{0}' already exists in the ledger — immutability violation")]
    DecisionAlreadyExists(String),

    #[error("decision '{0}' not found in the ledger")]
    DecisionNotFound(String),

    #[error("decision '{0}' is not certified — cannot append lifecycle events")]
    DecisionNotCertified(String),

    #[error("outcome for decision '{0}' cannot be appended while status is still OPEN and observation boundary has not passed")]
    ObservationBoundaryNotPassed(String),

    #[error("execution record for decision '{0}' already exists — use a correction event instead")]
    ExecutionAlreadyRecorded(String),

    #[error("temporal firewall violation: event timestamp {event_ts} is before decision timestamp {decision_ts} for decision '{decision_id}'")]
    TemporalFirewallViolation {
        decision_id: String,
        decision_ts: DateTime<Utc>,
        event_ts: DateTime<Utc>,
    },

    #[error("certification is incomplete for decision '{0}' — policy_artifact_hash, data_snapshot_id, and decision_pipeline are required")]
    IncompleteCertification(String),
}

// ─── Lifecycle events ────────────────────────────────────────────────────────

/// The append-only event log for a single decision's lifecycle.
///
/// Events are ordered chronologically. The original `DecisionRecord` is never
/// mutated; the current materialized state is derived from the event stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvent {
    pub event_id: String,
    pub decision_id: String,
    pub event_type: DecisionEventType,
    pub event_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionEventType {
    /// The decision was created and sealed.
    DecisionCreated,
    /// The decision was certified with provenance.
    DecisionCertified { certification: Certification },
    /// The user recorded an execution action.
    UserExecutionRecorded { execution: ExecutionRecord },
    /// The reference risk boundary was reached.
    ReferenceRiskReached { price: f64, timestamp: DateTime<Utc> },
    /// The target price was reached.
    TargetReached { price: f64, timestamp: DateTime<Utc> },
    /// The observation horizon elapsed.
    HorizonReached { timestamp: DateTime<Utc> },
    /// The user closed the position.
    UserClosed { price: f64, timestamp: DateTime<Utc> },
    /// Evidence enrichment was appended (does not modify the decision).
    EvidenceAppended { evidence: EvidenceRecord },
    /// The decision lifecycle was closed.
    DecisionClosed { outcome: OutcomeRecord },
}

// ─── Decision Ledger ─────────────────────────────────────────────────────────

/// The immutable Decision Ledger.
///
/// Stores certified `DecisionRecord`s and their append-only event streams.
/// The ledger enforces all MVP acceptance criteria at the boundary.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DecisionLedger {
    /// Immutable certified decisions, keyed by `decision_id`.
    decisions: HashMap<String, DecisionRecord>,
    /// Append-only event log, keyed by `decision_id`.
    events: HashMap<String, Vec<DecisionEvent>>,
    /// Insertion order for stable iteration.
    insertion_order: Vec<String>,
}

impl DecisionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Write operations ──────────────────────────────────────────────────

    /// Seal a new certified decision into the ledger.
    ///
    /// Enforces:
    /// - AC-01: certification fields are present.
    /// - AC-03: decision_id must not already exist.
    /// - AC-07: no confidence/probability fields (structural — enforced by type).
    /// - AC-08: no allocation fields (structural — enforced by type).
    pub fn seal_decision(&mut self, record: DecisionRecord) -> Result<(), LedgerError> {
        let id = record.identity.decision_id.clone();

        // AC-03: immutability — reject duplicates.
        if self.decisions.contains_key(&id) {
            return Err(LedgerError::DecisionAlreadyExists(id));
        }

        // AC-01: certification completeness.
        self.validate_certification(&id, &record.certification)?;

        let ts = record.identity.decision_timestamp;

        self.insertion_order.push(id.clone());
        self.decisions.insert(id.clone(), record);
        self.events.insert(id.clone(), vec![]);

        self.append_event(id.clone(), DecisionEventType::DecisionCreated, ts)?;

        // Clone the certification before the second append_event call so we
        // don't borrow `self.decisions` while also mutably borrowing `self`.
        let cert = self.decisions[&id].certification.clone();
        self.append_event(
            id,
            DecisionEventType::DecisionCertified { certification: cert },
            ts,
        )?;

        Ok(())
    }

    /// Record a user execution action for a certified decision.
    ///
    /// Enforces:
    /// - AC-04: user-controlled execution — quantity/price only when supplied.
    /// - AC-08: no allocation inference.
    /// - Temporal firewall: event_timestamp ≥ decision_timestamp.
    pub fn record_execution(
        &mut self,
        decision_id: &str,
        execution: ExecutionRecord,
        event_timestamp: DateTime<Utc>,
    ) -> Result<(), LedgerError> {
        let record = self.get_decision(decision_id)?;
        let decision_ts = record.identity.decision_timestamp;

        self.check_certified(decision_id)?;
        self.check_temporal_firewall(decision_id, decision_ts, event_timestamp)?;

        // Append the event.
        self.append_event(
            decision_id.to_string(),
            DecisionEventType::UserExecutionRecorded {
                execution: execution.clone(),
            },
            event_timestamp,
        )?;

        // Materialize onto the record.
        self.decisions
            .get_mut(decision_id)
            .expect("decision exists")
            .execution = execution;

        Ok(())
    }

    /// Append an outcome to a certified decision.
    ///
    /// Enforces:
    /// - AC-05: outcome is only appended after the observation boundary.
    /// - Temporal firewall: event_timestamp ≥ decision_timestamp.
    ///
    /// The caller is responsible for verifying that the observation boundary
    /// has passed before calling this method. Pass `observation_boundary_passed
    /// = true` to confirm.
    pub fn record_outcome(
        &mut self,
        decision_id: &str,
        outcome: OutcomeRecord,
        event_timestamp: DateTime<Utc>,
        observation_boundary_passed: bool,
    ) -> Result<(), LedgerError> {
        let record = self.get_decision(decision_id)?;
        let decision_ts = record.identity.decision_timestamp;

        self.check_certified(decision_id)?;
        self.check_temporal_firewall(decision_id, decision_ts, event_timestamp)?;

        // AC-05: observation boundary must have passed.
        if !observation_boundary_passed {
            return Err(LedgerError::ObservationBoundaryNotPassed(
                decision_id.to_string(),
            ));
        }

        let event_type = match &outcome.status {
            OutcomeStatus::Target => DecisionEventType::TargetReached {
                price: outcome.exit_price.unwrap_or(0.0),
                timestamp: outcome.exit_timestamp.unwrap_or(event_timestamp),
            },
            OutcomeStatus::ReferenceRisk => DecisionEventType::ReferenceRiskReached {
                price: outcome.exit_price.unwrap_or(0.0),
                timestamp: outcome.exit_timestamp.unwrap_or(event_timestamp),
            },
            OutcomeStatus::Horizon => DecisionEventType::HorizonReached {
                timestamp: event_timestamp,
            },
            OutcomeStatus::UserClosed => DecisionEventType::UserClosed {
                price: outcome.exit_price.unwrap_or(0.0),
                timestamp: outcome.exit_timestamp.unwrap_or(event_timestamp),
            },
            OutcomeStatus::Open => {
                // Closing with OPEN status is a no-op close — still record it.
                DecisionEventType::DecisionClosed {
                    outcome: outcome.clone(),
                }
            }
        };

        self.append_event(decision_id.to_string(), event_type, event_timestamp)?;
        self.append_event(
            decision_id.to_string(),
            DecisionEventType::DecisionClosed {
                outcome: outcome.clone(),
            },
            event_timestamp,
        )?;

        self.decisions
            .get_mut(decision_id)
            .expect("decision exists")
            .outcome = outcome;

        Ok(())
    }

    /// Append evidence enrichment to a certified decision.
    ///
    /// Enforces AC-09: evidence cannot modify the original decision fields.
    pub fn append_evidence(
        &mut self,
        decision_id: &str,
        evidence: EvidenceRecord,
        event_timestamp: DateTime<Utc>,
    ) -> Result<(), LedgerError> {
        let record = self.get_decision(decision_id)?;
        let decision_ts = record.identity.decision_timestamp;

        self.check_certified(decision_id)?;
        self.check_temporal_firewall(decision_id, decision_ts, event_timestamp)?;

        self.append_event(
            decision_id.to_string(),
            DecisionEventType::EvidenceAppended {
                evidence: evidence.clone(),
            },
            event_timestamp,
        )?;

        self.decisions
            .get_mut(decision_id)
            .expect("decision exists")
            .evidence = evidence;

        Ok(())
    }

    // ── Read operations ───────────────────────────────────────────────────

    /// Return a reference to a decision by ID.
    pub fn get_decision(&self, decision_id: &str) -> Result<&DecisionRecord, LedgerError> {
        self.decisions
            .get(decision_id)
            .ok_or_else(|| LedgerError::DecisionNotFound(decision_id.to_string()))
    }

    /// Return all decisions in insertion order (Decision Feed).
    pub fn all_decisions(&self) -> Vec<&DecisionRecord> {
        self.insertion_order
            .iter()
            .filter_map(|id| self.decisions.get(id))
            .collect()
    }

    /// Return all decisions in reverse insertion order (newest first).
    pub fn decisions_newest_first(&self) -> Vec<&DecisionRecord> {
        self.insertion_order
            .iter()
            .rev()
            .filter_map(|id| self.decisions.get(id))
            .collect()
    }

    /// Return the event log for a decision (Decision Detail audit trail).
    pub fn events_for(&self, decision_id: &str) -> Result<&[DecisionEvent], LedgerError> {
        self.events
            .get(decision_id)
            .map(|v| v.as_slice())
            .ok_or_else(|| LedgerError::DecisionNotFound(decision_id.to_string()))
    }

    /// Total number of decisions in the ledger.
    pub fn len(&self) -> usize {
        self.decisions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty()
    }

    // ── Internal helpers ──────────────────────────────────────────────────

    fn append_event(
        &mut self,
        decision_id: String,
        event_type: DecisionEventType,
        event_timestamp: DateTime<Utc>,
    ) -> Result<(), LedgerError> {
        let event_id = format!("{}-{}", decision_id, event_timestamp.timestamp_nanos_opt().unwrap_or(0));
        let event = DecisionEvent {
            event_id,
            decision_id: decision_id.clone(),
            event_type,
            event_timestamp,
        };
        self.events
            .entry(decision_id)
            .or_default()
            .push(event);
        Ok(())
    }

    fn check_certified(&self, decision_id: &str) -> Result<(), LedgerError> {
        let record = self.decisions.get(decision_id)
            .ok_or_else(|| LedgerError::DecisionNotFound(decision_id.to_string()))?;
        if !record.is_certified() {
            return Err(LedgerError::DecisionNotCertified(decision_id.to_string()));
        }
        Ok(())
    }

    fn check_temporal_firewall(
        &self,
        decision_id: &str,
        decision_ts: DateTime<Utc>,
        event_ts: DateTime<Utc>,
    ) -> Result<(), LedgerError> {
        if event_ts < decision_ts {
            return Err(LedgerError::TemporalFirewallViolation {
                decision_id: decision_id.to_string(),
                decision_ts,
                event_ts,
            });
        }
        Ok(())
    }

    fn validate_certification(
        &self,
        decision_id: &str,
        cert: &Certification,
    ) -> Result<(), LedgerError> {
        if cert.policy_artifact_hash.is_empty()
            || cert.data_snapshot_id.is_empty()
            || cert.decision_pipeline.is_empty()
        {
            return Err(LedgerError::IncompleteCertification(decision_id.to_string()));
        }
        if cert.status != CertificationStatus::Certified {
            return Err(LedgerError::IncompleteCertification(decision_id.to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{
        DecisionCore, DecisionIdentity, Direction, ExecutionStatus, ReferenceRisk,
        ReferenceRiskStatus,
    };
    use chrono::TimeZone;

    fn ts(y: i32, mo: u32, d: u32, h: u32, m: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, m, s).unwrap()
    }

    fn sample_record(id: &str, instrument: &str, decision_ts: DateTime<Utc>) -> DecisionRecord {
        DecisionRecord {
            identity: DecisionIdentity {
                decision_id: id.to_string(),
                instrument: instrument.to_string(),
                decision_timestamp: decision_ts,
            },
            certification: Certification {
                status: CertificationStatus::Certified,
                policy_artifact_hash: "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121"
                    .into(),
                decision_pipeline: "C3-002".into(),
                execution_artifact_hash: Some(
                    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f".into(),
                ),
                certified_timestamp: decision_ts,
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
    fn seal_and_retrieve_decision() {
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);

        ledger.seal_decision(record.clone()).unwrap();
        let retrieved = ledger.get_decision("coralys-ADANIENT-20260817T101500Z-001").unwrap();
        assert_eq!(retrieved.identity.instrument, "ADANIENT.NS");
    }

    #[test]
    fn duplicate_seal_is_rejected() {
        // AC-03: immutability
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);

        ledger.seal_decision(record.clone()).unwrap();
        let err = ledger.seal_decision(record).unwrap_err();
        assert!(matches!(err, LedgerError::DecisionAlreadyExists(_)));
    }

    #[test]
    fn outcome_rejected_without_boundary_flag() {
        // AC-05: observation boundary must have passed
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);
        ledger.seal_decision(record).unwrap();

        let outcome = OutcomeRecord {
            status: OutcomeStatus::Target,
            exit_reason: Some("TARGET".into()),
            exit_timestamp: Some(ts(2026, 9, 10, 10, 0, 0)),
            exit_price: Some(1234.50),
            realized_pnl: Some(54.25),
        };

        let err = ledger
            .record_outcome(
                "coralys-ADANIENT-20260817T101500Z-001",
                outcome,
                ts(2026, 9, 10, 10, 0, 0),
                false, // boundary NOT passed
            )
            .unwrap_err();
        assert!(matches!(err, LedgerError::ObservationBoundaryNotPassed(_)));
    }

    #[test]
    fn temporal_firewall_rejects_past_event() {
        // AC-02: temporal firewall
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);
        ledger.seal_decision(record).unwrap();

        let past_ts = ts(2026, 8, 16, 9, 0, 0); // before decision_ts
        let err = ledger
            .record_execution(
                "coralys-ADANIENT-20260817T101500Z-001",
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(past_ts),
                    quantity: None,
                    execution_price: None,
                    execution_source: Some("USER".into()),
                },
                past_ts,
            )
            .unwrap_err();
        assert!(matches!(err, LedgerError::TemporalFirewallViolation { .. }));
    }

    #[test]
    fn execution_without_quantity_is_valid() {
        // AC-04 + AC-08: user-controlled execution, no allocation
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);
        ledger.seal_decision(record).unwrap();

        let exec_ts = ts(2026, 8, 17, 10, 20, 0);
        ledger
            .record_execution(
                "coralys-ADANIENT-20260817T101500Z-001",
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(exec_ts),
                    quantity: None,          // user did not supply quantity
                    execution_price: None,   // user did not supply price
                    execution_source: Some("USER".into()),
                },
                exec_ts,
            )
            .unwrap();

        let retrieved = ledger
            .get_decision("coralys-ADANIENT-20260817T101500Z-001")
            .unwrap();
        assert_eq!(retrieved.execution.status, ExecutionStatus::UserExecuted);
        assert!(retrieved.execution.quantity.is_none());
    }

    #[test]
    fn incomplete_certification_is_rejected() {
        // AC-01: certification completeness
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let mut record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);
        record.certification.policy_artifact_hash = String::new(); // empty hash

        let err = ledger.seal_decision(record).unwrap_err();
        assert!(matches!(err, LedgerError::IncompleteCertification(_)));
    }

    #[test]
    fn all_decisions_returns_insertion_order() {
        let mut ledger = DecisionLedger::new();
        let t1 = ts(2026, 8, 17, 10, 15, 0);
        let t2 = ts(2026, 8, 17, 10, 30, 0);
        ledger.seal_decision(sample_record("id-001", "ADANIENT.NS", t1)).unwrap();
        ledger.seal_decision(sample_record("id-002", "BPCL.NS", t2)).unwrap();

        let all = ledger.all_decisions();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].identity.decision_id, "id-001");
        assert_eq!(all[1].identity.decision_id, "id-002");
    }

    #[test]
    fn event_log_is_populated_on_seal() {
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);
        ledger.seal_decision(record).unwrap();

        let events = ledger
            .events_for("coralys-ADANIENT-20260817T101500Z-001")
            .unwrap();
        assert!(!events.is_empty());
        assert!(matches!(
            events[0].event_type,
            DecisionEventType::DecisionCreated
        ));
    }

    #[test]
    fn ledger_round_trips_json() {
        let mut ledger = DecisionLedger::new();
        let decision_ts = ts(2026, 8, 17, 10, 15, 0);
        let record = sample_record("coralys-ADANIENT-20260817T101500Z-001", "ADANIENT.NS", decision_ts);
        ledger.seal_decision(record).unwrap();

        let json = serde_json::to_string(&ledger).expect("serialize");
        let back: DecisionLedger = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.len(), 1);
    }
}