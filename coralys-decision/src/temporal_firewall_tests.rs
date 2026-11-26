//! OBS-000 — Temporal Firewall Test Suite
//!
//! Proves that the observation machinery cannot leak future information into
//! certified T0 decisions. All 7 formal invariants from INDEX.md v1.76 are
//! exercised here, plus the future-poison test.
//!
//! **Invariants under test:**
//!
//! 1. T0 immutability — no observation process can modify any certified T0 field.
//! 2. Future-data exclusion — observation data with timestamp > T0 cannot
//!    participate in T0 creation or reconstruction.
//! 3. Boundary correctness — TARGET/REFERENCE_RISK/HORIZON cannot be classified
//!    using data that precedes T0 or wasn't available at observation time.
//! 4. Direction symmetry — LONG: target above T0 / risk below; SHORT: target
//!    below T0 / risk above.
//! 5. Append-only outcomes — outcome appended, never rewrites original decision.
//! 6. Replay determinism — same T0 snapshot + subsequent market observations →
//!    deterministic result.
//! 7. No accidental research feedback — outcome cannot flow back into T0 decision.
//!
//! **Future-poison test:** A T0 decision produced with future-poison data must
//! either reject the future data or produce an identical certified T0 result.

#[cfg(test)]
mod obs_000_temporal_firewall {
    use chrono::{TimeZone, Utc};

    use crate::{
        ledger::{DecisionLedger, LedgerError},
        record::{
            Certification, CertificationStatus, DecisionCore, DecisionIdentity, DecisionRecord,
            Direction, EvidenceRecord, ExecutionRecord, ExecutionStatus, OutcomeRecord,
            OutcomeStatus, ReferenceRisk, ReferenceRiskStatus,
        },
    };

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn ts(y: i32, mo: u32, d: u32, h: u32, m: u32, s: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, m, s).unwrap()
    }

    /// Build a minimal certified LONG decision at the given timestamp.
    fn long_decision(
        id: &str,
        instrument: &str,
        decision_ts: chrono::DateTime<Utc>,
        reference_price: f64,
        target_price: f64,
        risk_boundary: f64,
    ) -> DecisionRecord {
        DecisionRecord {
            identity: DecisionIdentity {
                decision_id: id.to_string(),
                instrument: instrument.to_string(),
                decision_timestamp: decision_ts,
            },
            certification: Certification {
                status: CertificationStatus::Certified,
                policy_artifact_hash:
                    "5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121".into(),
                execution_artifact_hash: Some(
                    "3876ffa232f75068636aa058c6775671ac2f935ad2751c1253edd49e0770883f".into(),
                ),
                decision_pipeline: "C3-002".into(),
                certified_timestamp: decision_ts,
                data_snapshot_id: "snapshot-20260817T101500Z".into(),
            },
            decision: DecisionCore {
                direction: Direction::Long,
                trend: "Bullish".into(),
                momentum: "Positive".into(),
                volatility: "present".into(),
                target_price: Some(target_price),
                atr_14: Some(10.0),
                reference_price: Some(reference_price),
                effective_session: Some("2026-08-18".into()),
            },
            reference_risk: ReferenceRisk {
                boundary_price: Some(risk_boundary),
                boundary_type: "CORALYS_V0_ATR_TMV".into(),
                status: ReferenceRiskStatus::Reference,
            },
            execution: ExecutionRecord::default(),
            outcome: OutcomeRecord::default(),
            evidence: EvidenceRecord::default(),
        }
    }

    /// Build a minimal certified SHORT decision at the given timestamp.
    fn short_decision(
        id: &str,
        instrument: &str,
        decision_ts: chrono::DateTime<Utc>,
        reference_price: f64,
        target_price: f64,
        risk_boundary: f64,
    ) -> DecisionRecord {
        let mut d = long_decision(
            id,
            instrument,
            decision_ts,
            reference_price,
            target_price,
            risk_boundary,
        );
        d.decision.direction = Direction::Short;
        d.decision.trend = "Bearish".into();
        d
    }

    // ── Invariant 1: T0 Immutability ─────────────────────────────────────────

    /// After sealing, the identity, certification, decision core, and reference
    /// risk fields must be identical to what was sealed — no observation process
    /// may alter them.
    #[test]
    fn inv1_t0_fields_unchanged_after_execution_recorded() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision(
            "obs000-inv1-exec",
            "ADANIENT.NS",
            t0,
            1200.0,
            1260.0,
            1160.0,
        );

        // Capture the T0 snapshot before any lifecycle events.
        let sealed_identity = record.identity.clone();
        let sealed_certification = record.certification.clone();
        let sealed_decision = record.decision.clone();
        let sealed_risk = record.reference_risk.clone();

        ledger.seal_decision(record).unwrap();

        // Record an execution event.
        let exec_ts = ts(2026, 8, 17, 10, 20, 0);
        ledger
            .record_execution(
                "obs000-inv1-exec",
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(exec_ts),
                    quantity: None,
                    execution_price: Some(1201.50),
                    execution_source: Some("USER".into()),
                },
                exec_ts,
            )
            .unwrap();

        let after = ledger.get_decision("obs000-inv1-exec").unwrap();

        // T0 fields must be byte-identical to what was sealed.
        assert_eq!(
            after.identity, sealed_identity,
            "identity mutated after execution"
        );
        assert_eq!(
            after.certification, sealed_certification,
            "certification mutated after execution"
        );
        assert_eq!(
            after.decision, sealed_decision,
            "decision core mutated after execution"
        );
        assert_eq!(
            after.reference_risk, sealed_risk,
            "reference_risk mutated after execution"
        );
    }

    /// After sealing, the T0 fields must be unchanged after an outcome is appended.
    #[test]
    fn inv1_t0_fields_unchanged_after_outcome_recorded() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv1-out", "BPCL.NS", t0, 500.0, 525.0, 485.0);

        let sealed_identity = record.identity.clone();
        let sealed_certification = record.certification.clone();
        let sealed_decision = record.decision.clone();
        let sealed_risk = record.reference_risk.clone();

        ledger.seal_decision(record).unwrap();

        let outcome_ts = ts(2026, 8, 20, 10, 15, 0);
        ledger
            .record_outcome(
                "obs000-inv1-out",
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: Some("TARGET".into()),
                    exit_timestamp: Some(outcome_ts),
                    exit_price: Some(525.0),
                    realized_pnl: Some(25.0),
                },
                outcome_ts,
                true,
            )
            .unwrap();

        let after = ledger.get_decision("obs000-inv1-out").unwrap();
        assert_eq!(
            after.identity, sealed_identity,
            "identity mutated after outcome"
        );
        assert_eq!(
            after.certification, sealed_certification,
            "certification mutated after outcome"
        );
        assert_eq!(
            after.decision, sealed_decision,
            "decision core mutated after outcome"
        );
        assert_eq!(
            after.reference_risk, sealed_risk,
            "reference_risk mutated after outcome"
        );
    }

    // ── Invariant 2: Future-Data Exclusion ───────────────────────────────────

    /// An execution event with a timestamp before T0 must be rejected.
    #[test]
    fn inv2_execution_before_t0_is_rejected() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv2-exec", "VEDL.NS", t0, 264.0, 274.0, 258.0);
        ledger.seal_decision(record).unwrap();

        // Attempt to record an execution that happened BEFORE T0.
        let past_ts = ts(2026, 8, 16, 9, 0, 0);
        let err = ledger
            .record_execution(
                "obs000-inv2-exec",
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

        assert!(
            matches!(err, LedgerError::TemporalFirewallViolation { .. }),
            "expected TemporalFirewallViolation, got: {err:?}"
        );
    }

    /// An outcome event with a timestamp before T0 must be rejected.
    #[test]
    fn inv2_outcome_before_t0_is_rejected() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv2-out", "TRENT.NS", t0, 2942.0, 3047.0, 2889.0);
        ledger.seal_decision(record).unwrap();

        let past_ts = ts(2026, 8, 16, 9, 0, 0);
        let err = ledger
            .record_outcome(
                "obs000-inv2-out",
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: None,
                    exit_timestamp: Some(past_ts),
                    exit_price: Some(3047.0),
                    realized_pnl: None,
                },
                past_ts,
                true,
            )
            .unwrap_err();

        assert!(
            matches!(err, LedgerError::TemporalFirewallViolation { .. }),
            "expected TemporalFirewallViolation, got: {err:?}"
        );
    }

    /// Evidence enrichment with a timestamp before T0 must be rejected.
    #[test]
    fn inv2_evidence_before_t0_is_rejected() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv2-ev", "SRF.NS", t0, 2630.0, 2682.0, 2603.0);
        ledger.seal_decision(record).unwrap();

        let past_ts = ts(2026, 8, 16, 9, 0, 0);
        let err = ledger
            .append_evidence(
                "obs000-inv2-ev",
                EvidenceRecord {
                    similar_decisions_count: Some(42),
                    historical_target_rate: Some(0.55),
                    median_mae_pct: Some(0.012),
                    p90_mae_pct: Some(0.025),
                    median_mfe_pct: Some(0.038),
                    median_time_to_target_sessions: Some(3.5),
                },
                past_ts,
            )
            .unwrap_err();

        assert!(
            matches!(err, LedgerError::TemporalFirewallViolation { .. }),
            "expected TemporalFirewallViolation, got: {err:?}"
        );
    }

    // ── Invariant 3: Boundary Correctness ────────────────────────────────────

    /// Outcome without boundary confirmation must be rejected (AC-O1).
    #[test]
    fn inv3_outcome_without_boundary_confirmation_rejected() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv3", "VBL.NS", t0, 439.0, 448.0, 434.0);
        ledger.seal_decision(record).unwrap();

        let obs_ts = ts(2026, 8, 20, 10, 15, 0);
        let err = ledger
            .record_outcome(
                "obs000-inv3",
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: None,
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(448.0),
                    realized_pnl: None,
                },
                obs_ts,
                false, // boundary NOT confirmed
            )
            .unwrap_err();

        assert!(
            matches!(err, LedgerError::ObservationBoundaryNotPassed(_)),
            "expected ObservationBoundaryNotPassed, got: {err:?}"
        );
    }

    /// Outcome with boundary confirmation and valid timestamp is accepted.
    #[test]
    fn inv3_outcome_with_boundary_confirmation_accepted() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv3-ok", "TVSMOTOR.NS", t0, 4356.0, 4529.0, 4270.0);
        ledger.seal_decision(record).unwrap();

        let obs_ts = ts(2026, 8, 20, 10, 15, 0);
        ledger
            .record_outcome(
                "obs000-inv3-ok",
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: Some("TARGET".into()),
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(4529.0),
                    realized_pnl: None,
                },
                obs_ts,
                true,
            )
            .unwrap();

        let after = ledger.get_decision("obs000-inv3-ok").unwrap();
        assert_eq!(after.outcome.status, OutcomeStatus::Target);
    }

    // ── Invariant 4: Direction Symmetry ──────────────────────────────────────

    /// LONG: target must be above reference price; risk must be below.
    #[test]
    fn inv4_long_target_above_reference_risk_below() {
        let reference = 1200.0_f64;
        let target = 1260.0_f64;
        let risk = 1160.0_f64;

        assert!(
            target > reference,
            "LONG target ({target}) must be above reference ({reference})"
        );
        assert!(
            risk < reference,
            "LONG risk ({risk}) must be below reference ({reference})"
        );

        // Verify the sealed record preserves this geometry.
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision(
            "obs000-inv4-long",
            "ADANIENT.NS",
            t0,
            reference,
            target,
            risk,
        );
        ledger.seal_decision(record).unwrap();

        let d = ledger.get_decision("obs000-inv4-long").unwrap();
        let ref_price = d.decision.reference_price.unwrap();
        let target_price = d.decision.target_price.unwrap();
        let risk_price = d.reference_risk.boundary_price.unwrap();

        assert_eq!(d.decision.direction, Direction::Long);
        assert!(
            target_price > ref_price,
            "LONG target must be above reference after seal"
        );
        assert!(
            risk_price < ref_price,
            "LONG risk must be below reference after seal"
        );
    }

    /// SHORT: target must be below reference price; risk must be above.
    #[test]
    fn inv4_short_target_below_reference_risk_above() {
        let reference = 4971.0_f64;
        let target = 4792.0_f64; // below reference for SHORT
        let risk = 5060.0_f64; // above reference for SHORT

        assert!(
            target < reference,
            "SHORT target ({target}) must be below reference ({reference})"
        );
        assert!(
            risk > reference,
            "SHORT risk ({risk}) must be above reference ({reference})"
        );

        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = short_decision(
            "obs000-inv4-short",
            "TORNTPHARM.NS",
            t0,
            reference,
            target,
            risk,
        );
        ledger.seal_decision(record).unwrap();

        let d = ledger.get_decision("obs000-inv4-short").unwrap();
        let ref_price = d.decision.reference_price.unwrap();
        let target_price = d.decision.target_price.unwrap();
        let risk_price = d.reference_risk.boundary_price.unwrap();

        assert_eq!(d.decision.direction, Direction::Short);
        assert!(
            target_price < ref_price,
            "SHORT target must be below reference after seal"
        );
        assert!(
            risk_price > ref_price,
            "SHORT risk must be above reference after seal"
        );
    }

    // ── Invariant 5: Append-Only Outcomes ────────────────────────────────────

    /// The original decision fields must be unchanged after an outcome is appended.
    /// The outcome is a new field on the record, not a rewrite of the decision.
    #[test]
    fn inv5_outcome_appended_not_rewritten() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv5", "POLYCAB.NS", t0, 6000.0, 6300.0, 5850.0);

        let original_target = record.decision.target_price;
        let original_direction = record.decision.direction.clone();
        let original_trend = record.decision.trend.clone();
        let original_risk = record.reference_risk.boundary_price;

        ledger.seal_decision(record).unwrap();

        // Append outcome.
        let obs_ts = ts(2026, 8, 22, 10, 15, 0);
        ledger
            .record_outcome(
                "obs000-inv5",
                OutcomeRecord {
                    status: OutcomeStatus::ReferenceRisk,
                    exit_reason: Some("REFERENCE_RISK".into()),
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(5850.0),
                    realized_pnl: Some(-150.0),
                },
                obs_ts,
                true,
            )
            .unwrap();

        let after = ledger.get_decision("obs000-inv5").unwrap();

        // Outcome is appended.
        assert_eq!(after.outcome.status, OutcomeStatus::ReferenceRisk);
        assert_eq!(after.outcome.exit_price, Some(5850.0));

        // Original decision fields are unchanged.
        assert_eq!(
            after.decision.target_price, original_target,
            "target_price was rewritten"
        );
        assert_eq!(
            after.decision.direction, original_direction,
            "direction was rewritten"
        );
        assert_eq!(after.decision.trend, original_trend, "trend was rewritten");
        assert_eq!(
            after.reference_risk.boundary_price, original_risk,
            "risk boundary was rewritten"
        );
    }

    /// HORIZON outcome is appended correctly without modifying T0 fields.
    #[test]
    fn inv5_horizon_outcome_appended() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv5-horizon", "INFY.NS", t0, 1800.0, 1890.0, 1755.0);
        let original_decision = record.decision.clone();
        ledger.seal_decision(record).unwrap();

        let obs_ts = ts(2026, 8, 24, 10, 15, 0);
        ledger
            .record_outcome(
                "obs000-inv5-horizon",
                OutcomeRecord {
                    status: OutcomeStatus::Horizon,
                    exit_reason: Some("HORIZON".into()),
                    exit_timestamp: Some(obs_ts),
                    exit_price: None,
                    realized_pnl: None,
                },
                obs_ts,
                true,
            )
            .unwrap();

        let after = ledger.get_decision("obs000-inv5-horizon").unwrap();
        assert_eq!(after.outcome.status, OutcomeStatus::Horizon);
        assert_eq!(
            after.decision, original_decision,
            "decision core mutated by horizon outcome"
        );
    }

    // ── Invariant 6: Replay Determinism ──────────────────────────────────────

    /// Sealing the same T0 snapshot twice (in two separate ledgers) must produce
    /// identical certified decisions. The ledger is deterministic given the same input.
    #[test]
    fn inv6_replay_determinism_same_snapshot_same_result() {
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record_a = long_decision("obs000-inv6", "WIPRO.NS", t0, 550.0, 577.0, 536.0);
        let record_b = record_a.clone();

        let mut ledger_a = DecisionLedger::new();
        let mut ledger_b = DecisionLedger::new();

        ledger_a.seal_decision(record_a).unwrap();
        ledger_b.seal_decision(record_b).unwrap();

        let a = ledger_a.get_decision("obs000-inv6").unwrap();
        let b = ledger_b.get_decision("obs000-inv6").unwrap();

        assert_eq!(a.identity, b.identity, "identity differs between replays");
        assert_eq!(
            a.certification, b.certification,
            "certification differs between replays"
        );
        assert_eq!(
            a.decision, b.decision,
            "decision core differs between replays"
        );
        assert_eq!(
            a.reference_risk, b.reference_risk,
            "reference_risk differs between replays"
        );
    }

    /// Appending the same outcome to two identical ledgers produces identical results.
    #[test]
    fn inv6_replay_determinism_same_outcome_same_result() {
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let obs_ts = ts(2026, 8, 20, 10, 15, 0);

        let make_outcome = || OutcomeRecord {
            status: OutcomeStatus::Target,
            exit_reason: Some("TARGET".into()),
            exit_timestamp: Some(obs_ts),
            exit_price: Some(577.0),
            realized_pnl: Some(27.0),
        };

        let mut ledger_a = DecisionLedger::new();
        let mut ledger_b = DecisionLedger::new();

        ledger_a
            .seal_decision(long_decision(
                "obs000-inv6-out",
                "WIPRO.NS",
                t0,
                550.0,
                577.0,
                536.0,
            ))
            .unwrap();
        ledger_b
            .seal_decision(long_decision(
                "obs000-inv6-out",
                "WIPRO.NS",
                t0,
                550.0,
                577.0,
                536.0,
            ))
            .unwrap();

        ledger_a
            .record_outcome("obs000-inv6-out", make_outcome(), obs_ts, true)
            .unwrap();
        ledger_b
            .record_outcome("obs000-inv6-out", make_outcome(), obs_ts, true)
            .unwrap();

        let a = ledger_a.get_decision("obs000-inv6-out").unwrap();
        let b = ledger_b.get_decision("obs000-inv6-out").unwrap();

        assert_eq!(a.outcome, b.outcome, "outcome differs between replays");
        assert_eq!(
            a.decision, b.decision,
            "decision core differs between replays after outcome"
        );
    }

    // ── Invariant 7: No Accidental Research Feedback ─────────────────────────

    /// Evidence enrichment must not alter the decision core, certification,
    /// identity, or reference risk fields.
    #[test]
    fn inv7_evidence_does_not_alter_decision_core() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv7-ev", "HDFCBANK.NS", t0, 1700.0, 1785.0, 1658.0);

        let original_decision = record.decision.clone();
        let original_certification = record.certification.clone();
        let original_identity = record.identity.clone();
        let original_risk = record.reference_risk.clone();

        ledger.seal_decision(record).unwrap();

        let ev_ts = ts(2026, 8, 18, 10, 15, 0);
        ledger
            .append_evidence(
                "obs000-inv7-ev",
                EvidenceRecord {
                    similar_decisions_count: Some(87),
                    historical_target_rate: Some(0.62),
                    median_mae_pct: Some(0.009),
                    p90_mae_pct: Some(0.021),
                    median_mfe_pct: Some(0.045),
                    median_time_to_target_sessions: Some(2.8),
                },
                ev_ts,
            )
            .unwrap();

        let after = ledger.get_decision("obs000-inv7-ev").unwrap();

        // Evidence is populated.
        assert_eq!(after.evidence.similar_decisions_count, Some(87));
        assert_eq!(after.evidence.historical_target_rate, Some(0.62));

        // T0 fields are unchanged — evidence cannot feed back into the decision.
        assert_eq!(
            after.decision, original_decision,
            "decision core altered by evidence"
        );
        assert_eq!(
            after.certification, original_certification,
            "certification altered by evidence"
        );
        assert_eq!(
            after.identity, original_identity,
            "identity altered by evidence"
        );
        assert_eq!(
            after.reference_risk, original_risk,
            "reference_risk altered by evidence"
        );
    }

    /// Outcome must not alter the decision core — the outcome is a separate
    /// lifecycle appendage, not a research feedback loop.
    #[test]
    fn inv7_outcome_does_not_feed_back_into_decision() {
        let mut ledger = DecisionLedger::new();
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let record = long_decision("obs000-inv7-out", "RELIANCE.NS", t0, 2900.0, 3045.0, 2827.0);

        let original_decision = record.decision.clone();
        ledger.seal_decision(record).unwrap();

        let obs_ts = ts(2026, 8, 21, 10, 15, 0);
        ledger
            .record_outcome(
                "obs000-inv7-out",
                OutcomeRecord {
                    status: OutcomeStatus::UserClosed,
                    exit_reason: Some("USER_CLOSED".into()),
                    exit_timestamp: Some(obs_ts),
                    exit_price: Some(2980.0),
                    realized_pnl: Some(80.0),
                },
                obs_ts,
                true,
            )
            .unwrap();

        let after = ledger.get_decision("obs000-inv7-out").unwrap();

        // Outcome is recorded.
        assert_eq!(after.outcome.status, OutcomeStatus::UserClosed);

        // Decision core is unchanged — outcome cannot alter T0.
        assert_eq!(
            after.decision, original_decision,
            "decision core was altered by outcome (research feedback violation)"
        );
    }

    // ── Future-Poison Test ────────────────────────────────────────────────────

    /// A T0 decision sealed with a legitimate snapshot must have its T0 fields
    /// unchanged after any number of future observations are appended.
    /// Future data (evidence, execution, outcome) must only appear in lifecycle
    /// appendages — never in the sealed T0 fields.
    #[test]
    fn future_poison_subsequent_observations_cannot_alter_t0() {
        let t0 = ts(2026, 8, 17, 10, 15, 0);

        // Seal a legitimate T0 decision.
        let legitimate = long_decision("obs000-poison", "ICICIBANK.NS", t0, 1100.0, 1155.0, 1072.0);
        let sealed_decision = legitimate.decision.clone();
        let sealed_certification = legitimate.certification.clone();

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(legitimate).unwrap();

        // Now simulate "future" observations arriving after T0.
        // These are legitimate post-T0 events — they must not alter T0.
        let future_ts_1 = ts(2026, 8, 18, 10, 15, 0);
        let future_ts_2 = ts(2026, 8, 19, 10, 15, 0);
        let future_ts_3 = ts(2026, 8, 20, 10, 15, 0);

        // Future evidence (post-T0 research enrichment) — must not alter T0.
        ledger
            .append_evidence(
                "obs000-poison",
                EvidenceRecord {
                    similar_decisions_count: Some(55),
                    historical_target_rate: Some(0.58),
                    median_mae_pct: Some(0.011),
                    p90_mae_pct: Some(0.022),
                    median_mfe_pct: Some(0.041),
                    median_time_to_target_sessions: Some(3.1),
                },
                future_ts_1,
            )
            .unwrap();

        // Future execution (post-T0 user action) — must not alter T0.
        ledger
            .record_execution(
                "obs000-poison",
                ExecutionRecord {
                    status: ExecutionStatus::UserExecuted,
                    execution_timestamp: Some(future_ts_2),
                    quantity: None,
                    execution_price: Some(1102.0),
                    execution_source: Some("USER".into()),
                },
                future_ts_2,
            )
            .unwrap();

        // Future outcome (post-T0 market result) — must not alter T0.
        ledger
            .record_outcome(
                "obs000-poison",
                OutcomeRecord {
                    status: OutcomeStatus::Target,
                    exit_reason: Some("TARGET".into()),
                    exit_timestamp: Some(future_ts_3),
                    exit_price: Some(1155.0),
                    realized_pnl: Some(55.0),
                },
                future_ts_3,
                true,
            )
            .unwrap();

        // After all future observations, T0 fields must be identical to what was sealed.
        let after = ledger.get_decision("obs000-poison").unwrap();

        assert_eq!(
            after.decision, sealed_decision,
            "future observations altered the T0 decision core (future-poison violation)"
        );
        assert_eq!(
            after.certification, sealed_certification,
            "future observations altered the T0 certification (future-poison violation)"
        );

        // Confirm future data IS present in the lifecycle appendages.
        assert_eq!(
            after.evidence.similar_decisions_count,
            Some(55),
            "evidence not recorded"
        );
        assert_eq!(
            after.execution.status,
            ExecutionStatus::UserExecuted,
            "execution not recorded"
        );
        assert_eq!(
            after.outcome.status,
            OutcomeStatus::Target,
            "outcome not recorded"
        );
    }

    /// Attempting to seal a second decision with the same ID after the first is
    /// sealed must be rejected — even if the second has different T0 fields.
    /// This proves the ledger cannot be "poisoned" by a duplicate seal.
    #[test]
    fn future_poison_duplicate_seal_rejected() {
        let t0 = ts(2026, 8, 17, 10, 15, 0);
        let original = long_decision(
            "obs000-poison-dup",
            "MARUTI.NS",
            t0,
            12000.0,
            12600.0,
            11700.0,
        );

        let mut ledger = DecisionLedger::new();
        ledger.seal_decision(original.clone()).unwrap();

        // Attempt to overwrite with a "poisoned" version that has different prices.
        let mut poisoned = original.clone();
        poisoned.decision.target_price = Some(99999.0); // future-inflated target
        poisoned.decision.reference_price = Some(99999.0);

        let err = ledger.seal_decision(poisoned).unwrap_err();
        assert!(
            matches!(err, LedgerError::DecisionAlreadyExists(_)),
            "duplicate seal with poisoned data was not rejected: {err:?}"
        );

        // Original T0 fields must be unchanged.
        let after = ledger.get_decision("obs000-poison-dup").unwrap();
        assert_eq!(
            after.decision.target_price,
            Some(12600.0),
            "T0 target_price was overwritten by poisoned duplicate seal"
        );
        assert_eq!(
            after.decision.reference_price,
            Some(12000.0),
            "T0 reference_price was overwritten by poisoned duplicate seal"
        );
    }
}
