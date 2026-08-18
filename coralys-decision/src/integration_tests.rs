//! MVP-010 — Temporal and Reproducibility Integration Tests.
//!
//! These tests prove the complete Coralys Decision Intelligence lifecycle
//! invariants at the domain layer (not the HTTP layer).
//!
//! **Final acceptance criterion:**
//! > A Coralys decision can be certified from the C3-002 artifact, exposed to
//! > the user, optionally acted upon by the user, and subsequently observed,
//! > without any post-certification operation altering the original decision,
//! > certification, provenance, or evidence.
//!
//! **Six invariant groups:**
//! 1. Certification immutability — identity/certification/decision/reference_risk
//!    unchanged after execution + outcome.
//! 2. Temporal firewall — decision_ts ≤ execution_ts ≤ observation_ts; reversed
//!    timestamps are rejected.
//! 3. Provenance survives the complete lifecycle — all four hash fields intact.
//! 4. User execution is user-controlled — no quantity inference, no allocation,
//!    no confidence, no ranking.
//! 5. Observation boundary is a hard gate — boundary_confirmed=false → error,
//!    ledger unchanged.
//! 6. Byte-level immutability — serialize the certified portion before and after
//!    the full lifecycle; assert identical.

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json;

    use crate::{
        C3_002_POLICY_ARTIFACT_HASH, CORALYS_EXEC_ARTIFACT_HASH,
        DecisionLedger, DecisionRecordBuilder, LedgerError, SealedDecisionInput,
        record::{
            ExecutionRecord, ExecutionStatus, OutcomeRecord, OutcomeStatus,
        },
    };

    // ─── Test fixture ─────────────────────────────────────────────────────────

    /// Build a canonical `SealedDecisionInput` using the real C3-002 and
    /// Coralys execution artifact hashes. This is the closest we can get to a
    /// real C3-002 artifact without running the full pipeline.
    fn canonical_input(decision_id: &str) -> SealedDecisionInput {
        let decision_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 15, 0).unwrap();
        SealedDecisionInput {
            decision_id: decision_id.to_string(),
            instrument: "ADANIENT.NS".to_string(),
            decision_timestamp: decision_ts,
            direction: "LONG".to_string(),
            trend: "Bullish".to_string(),
            momentum: "Positive".to_string(),
            volatility: "present".to_string(),
            target_price: Some(1234.50),
            policy_artifact_hash: C3_002_POLICY_ARTIFACT_HASH.to_string(),
            execution_artifact_hash: Some(CORALYS_EXEC_ARTIFACT_HASH.to_string()),
            decision_pipeline: "C3-002".to_string(),
            data_snapshot_id: "ADANIENT.NS@2026-08-17T10:15:00Z".to_string(),
            certified_timestamp: decision_ts,
            reference_risk_boundary_price: Some(1180.25),
            reference_risk_boundary_type: "CORALYS_V0_ATR_TMV".to_string(),
            atr_14: None,
            reference_price: None,
            effective_session: None,
        }
    }

    /// Serialize only the immutable certified portion of a `DecisionRecord`
    /// (identity + certification + decision_core + reference_risk + evidence).
    /// Excludes the intentionally mutable lifecycle fields (execution, outcome).
    fn serialize_certified_portion(ledger: &DecisionLedger, id: &str) -> String {
        let record = ledger.get_decision(id).unwrap();
        let certified = serde_json::json!({
            "identity": record.identity,
            "certification": record.certification,
            "decision": record.decision,
            "reference_risk": record.reference_risk,
            "evidence": record.evidence,
        });
        serde_json::to_string(&certified).unwrap()
    }

    // ─── Group 1: Certification immutability ──────────────────────────────────

    #[test]
    fn certification_immutability_after_execution_and_outcome() {
        let id = "coralys-ADANIENT-20260817T101500Z-001";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        // Capture the certified state.
        let original = ledger.get_decision(id).unwrap().clone();

        // Record execution.
        let exec_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 20, 0).unwrap();
        ledger
            .record_execution(
                id,
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(exec_ts),
                    quantity: Some(100.0),
                    execution_price: Some(1230.0),
                    execution_source: Some("USER".to_string()),
                },
                exec_ts,
            )
            .unwrap();

        // Record outcome.
        let obs_ts = Utc.with_ymd_and_hms(2026, 8, 20, 10, 15, 0).unwrap();
        ledger
            .record_outcome(
                id,
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: Some("Target reached".to_string()),
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(1234.50),
                    realized_pnl: Some(450.0),
                },
                obs_ts,
                true,
            )
            .unwrap();

        let after = ledger.get_decision(id).unwrap();

        // Identity must be unchanged.
        assert_eq!(after.identity.decision_id, original.identity.decision_id);
        assert_eq!(after.identity.instrument, original.identity.instrument);
        assert_eq!(
            after.identity.decision_timestamp,
            original.identity.decision_timestamp
        );

        // Certification must be unchanged.
        assert_eq!(
            after.certification.policy_artifact_hash,
            original.certification.policy_artifact_hash
        );
        assert_eq!(
            after.certification.execution_artifact_hash,
            original.certification.execution_artifact_hash
        );
        assert_eq!(
            after.certification.decision_pipeline,
            original.certification.decision_pipeline
        );
        assert_eq!(
            after.certification.data_snapshot_id,
            original.certification.data_snapshot_id
        );

        // Decision core must be unchanged.
        assert_eq!(after.decision.direction, original.decision.direction);
        assert_eq!(after.decision.trend, original.decision.trend);
        assert_eq!(after.decision.momentum, original.decision.momentum);
        assert_eq!(after.decision.target_price, original.decision.target_price);

        // Reference risk must be unchanged.
        assert_eq!(
            after.reference_risk.boundary_price,
            original.reference_risk.boundary_price
        );
        assert_eq!(
            after.reference_risk.boundary_type,
            original.reference_risk.boundary_type
        );

        // Evidence must be unchanged (all null).
        assert_eq!(after.evidence, original.evidence);
    }

    // ─── Group 2: Temporal firewall ───────────────────────────────────────────

    #[test]
    fn temporal_firewall_execution_before_decision_is_rejected() {
        let id = "coralys-ADANIENT-20260817T101500Z-002";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        // Attempt execution BEFORE the decision timestamp.
        let before_decision = Utc.with_ymd_and_hms(2026, 8, 17, 9, 0, 0).unwrap();
        let result = ledger.record_execution(
            id,
            ExecutionRecord {
                status: ExecutionStatus::UserExecuted,
                execution_timestamp: Some(before_decision),
                quantity: None,
                execution_price: None,
                execution_source: Some("USER".to_string()),
            },
            before_decision,
        );

        assert!(
            matches!(result, Err(LedgerError::TemporalFirewallViolation { .. })),
            "expected TemporalFirewallViolation, got {:?}",
            result
        );
    }

    #[test]
    fn temporal_firewall_outcome_before_decision_is_rejected() {
        let id = "coralys-ADANIENT-20260817T101500Z-003";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        // Attempt outcome BEFORE the decision timestamp.
        let before_decision = Utc.with_ymd_and_hms(2026, 8, 17, 9, 0, 0).unwrap();
        let result = ledger.record_outcome(
            id,
            OutcomeRecord {
                status: OutcomeStatus::Target,
                exit_reason: None,
                exit_timestamp: Some(before_decision),
                exit_price: Some(1234.50),
                realized_pnl: None,
            },
            before_decision,
            true,
        );

        assert!(
            matches!(result, Err(LedgerError::TemporalFirewallViolation { .. })),
            "expected TemporalFirewallViolation, got {:?}",
            result
        );
    }

    #[test]
    fn temporal_firewall_valid_sequence_is_accepted() {
        // decision_ts ≤ execution_ts ≤ observation_ts
        let id = "coralys-ADANIENT-20260817T101500Z-004";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        let exec_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 20, 0).unwrap();
        ledger
            .record_execution(
                id,
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(exec_ts),
                    quantity: None,
                    execution_price: None,
                    execution_source: Some("USER".to_string()),
                },
                exec_ts,
            )
            .unwrap();

        let obs_ts = Utc.with_ymd_and_hms(2026, 8, 20, 10, 15, 0).unwrap();
        ledger
            .record_outcome(
                id,
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: None,
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(1234.50),
                    realized_pnl: None,
                },
                obs_ts,
                true,
            )
            .unwrap();

        let record = ledger.get_decision(id).unwrap();
        assert_eq!(record.execution.status, ExecutionStatus::UserExecuted);
        assert_eq!(record.outcome.status, OutcomeStatus::Target);
    }

    // ─── Group 3: Provenance survives the complete lifecycle ──────────────────

    #[test]
    fn all_four_provenance_fields_intact_after_full_lifecycle() {
        let id = "coralys-ADANIENT-20260817T101500Z-005";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        let exec_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 20, 0).unwrap();
        ledger
            .record_execution(
                id,
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(exec_ts),
                    quantity: None,
                    execution_price: None,
                    execution_source: Some("USER".to_string()),
                },
                exec_ts,
            )
            .unwrap();

        let obs_ts = Utc.with_ymd_and_hms(2026, 8, 20, 10, 15, 0).unwrap();
        ledger
            .record_outcome(
                id,
                OutcomeRecord {
                    status: OutcomeStatus::ReferenceRisk,
                    exit_reason: None,
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(1180.25),
                    realized_pnl: None,
                },
                obs_ts,
                true,
            )
            .unwrap();

        let after = ledger.get_decision(id).unwrap();

        // All four provenance fields must be intact.
        assert_eq!(
            after.certification.policy_artifact_hash,
            C3_002_POLICY_ARTIFACT_HASH
        );
        assert_eq!(
            after.certification.execution_artifact_hash.as_deref(),
            Some(CORALYS_EXEC_ARTIFACT_HASH)
        );
        assert_eq!(after.certification.decision_pipeline, "C3-002");
        assert_eq!(
            after.certification.data_snapshot_id,
            "ADANIENT.NS@2026-08-17T10:15:00Z"
        );
    }

    // ─── Group 4: User execution is user-controlled ───────────────────────────

    #[test]
    fn user_executed_without_quantity_is_valid() {
        // No quantity inference — quantity must remain None.
        let id = "coralys-ADANIENT-20260817T101500Z-006";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        let exec_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 20, 0).unwrap();
        ledger
            .record_execution(
                id,
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(exec_ts),
                    quantity: None, // explicitly not supplied
                    execution_price: None,
                    execution_source: Some("USER".to_string()),
                },
                exec_ts,
            )
            .unwrap();

        let after = ledger.get_decision(id).unwrap();
        assert_eq!(after.execution.status, ExecutionStatus::UserExecuted);
        assert!(
            after.execution.quantity.is_none(),
            "quantity must not be inferred"
        );
        assert!(
            after.execution.execution_price.is_none(),
            "execution_price must not be inferred"
        );
    }

    #[test]
    fn user_ignored_is_a_valid_lifecycle_event() {
        let id = "coralys-ADANIENT-20260817T101500Z-007";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        let decision_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 15, 0).unwrap();
        ledger
            .record_execution(
                id,
                ExecutionRecord {
                    status: ExecutionStatus::UserIgnored,
                    execution_timestamp: None,
                    quantity: None,
                    execution_price: None,
                    execution_source: None,
                },
                decision_ts, // use decision_ts as floor when no timestamp supplied
            )
            .unwrap();

        let after = ledger.get_decision(id).unwrap();
        assert_eq!(after.execution.status, ExecutionStatus::UserIgnored);
        // Original decision must be unchanged.
        assert_eq!(after.decision.direction, crate::record::Direction::Long);
    }

    // ─── Group 5: Observation boundary is a hard gate ─────────────────────────

    #[test]
    fn outcome_without_boundary_confirmation_is_rejected_and_ledger_unchanged() {
        let id = "coralys-ADANIENT-20260817T101500Z-008";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        let obs_ts = Utc.with_ymd_and_hms(2026, 8, 20, 10, 15, 0).unwrap();
        let result = ledger.record_outcome(
            id,
            OutcomeRecord {
                status: OutcomeStatus::Target,
                exit_reason: None,
                exit_timestamp: Some(obs_ts),
                exit_price: Some(1234.50),
                realized_pnl: None,
            },
            obs_ts,
            false, // boundary NOT confirmed
        );

        assert!(
            matches!(result, Err(LedgerError::ObservationBoundaryNotPassed(_))),
            "expected ObservationBoundaryNotPassed, got {:?}",
            result
        );

        // Ledger must be unchanged — outcome must still be OPEN.
        let after = ledger.get_decision(id).unwrap();
        assert_eq!(after.outcome.status, OutcomeStatus::Open);
        assert!(after.outcome.exit_price.is_none());
    }

    // ─── Group 6: Byte-level immutability ─────────────────────────────────────

    #[test]
    fn certified_portion_is_byte_identical_after_full_lifecycle() {
        let id = "coralys-ADANIENT-20260817T101500Z-009";
        let input = canonical_input(id);
        let record = DecisionRecordBuilder::build(input).unwrap();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(record).unwrap();

        // Capture the certified portion immediately after sealing.
        let before_bytes = serialize_certified_portion(&ledger, id);

        // Execute.
        let exec_ts = Utc.with_ymd_and_hms(2026, 8, 17, 10, 20, 0).unwrap();
        ledger
            .record_execution(
                id,
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(exec_ts),
                    quantity: Some(50.0),
                    execution_price: Some(1231.0),
                    execution_source: Some("USER".to_string()),
                },
                exec_ts,
            )
            .unwrap();

        // Record outcome.
        let obs_ts = Utc.with_ymd_and_hms(2026, 8, 20, 10, 15, 0).unwrap();
        ledger
            .record_outcome(
                id,
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: Some("Target reached".to_string()),
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(1234.50),
                    realized_pnl: Some(175.0),
                },
                obs_ts,
                true,
            )
            .unwrap();

        // Capture the certified portion after the full lifecycle.
        let after_bytes = serialize_certified_portion(&ledger, id);

        assert_eq!(
            before_bytes, after_bytes,
            "certified portion must be byte-identical before and after the full lifecycle"
        );
    }

}