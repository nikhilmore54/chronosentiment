//! Credit Engine — Layer 1a (contractual interpretation layer).
//!
//! This module sits between operational [`DutyMetrics`] and the optimisation
//! objectives.  It answers the question: *"given what the crew actually flew,
//! how many hours does the airline contractually owe them?"*
//!
//! # Architecture (UC-ARCH-001)
//!
//! ```text
//! DutyMetrics  ──►  CreditPolicy  ──►  DutyCredit
//!                        │
//!                   CreditContext
//!                   (base, date, …)
//! ```
//!
//! [`CreditPolicy`] is a **pure function** (determinism contract): identical
//! inputs always produce identical outputs.  No external state, no I/O.
//!
//! # Two notions of credit (GERAD G-2014-22)
//!
//! 1. **Contractual formula** (§1 of Quesnel et al.):
//!    `credit = scheduled_flight_time + 0.5 × deadhead_time`
//!    This is what [`GeradCreditPolicy`] implements.
//!
//! 2. **Generator pre-processing**: the `creditedHours` input file has 2 h
//!    subtracted per duty before deriving base constraints.  That is a data-
//!    cleaning step, not part of the contractual formula, and is **not**
//!    modelled here.
//!
//! # Per-duty scope
//!
//! [`CreditPolicy`] computes credit for a single duty.  Roster-level
//! adjustments (e.g. monthly minimum guarantees) belong in [`AgreementPack`]
//! or a future `RosterCreditPolicy` extension.
//!
//! [`AgreementPack`]: crate::domain::roster::PlanningPeriod

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::duty::DutyMetrics;
use super::flight::AirportCode;

// ── Provenance ────────────────────────────────────────────────────────────────

/// Provenance metadata for a [`CreditPolicy`] implementation.
///
/// Every optimisation artefact should record this alongside the schedule so
/// that results are reproducible and auditable.
///
/// # Determinism contract
///
/// A [`CreditPolicy`] implementation **must** be a pure function:
/// `compute(metrics, context)` returns the same [`DutyCredit`] for the same
/// inputs regardless of when or where it is called.  No mutable state, no
/// external I/O, no randomness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditPolicyMetadata {
    /// Machine-readable identifier (e.g. `"gerad-g2014-22-v1"`).
    pub id: &'static str,
    /// Regulatory or contractual authority (e.g. `"GERAD G-2014-22"`).
    pub authority: &'static str,
    /// Semantic version of this policy implementation (e.g. `"1.0.0"`).
    pub version: &'static str,
    /// Human-readable description of the credit formula.
    pub description: &'static str,
}

// ── Context ───────────────────────────────────────────────────────────────────

/// Contextual information supplied to [`CreditPolicy::compute`].
///
/// Carries the crew-member and scheduling context that the policy may need
/// beyond the raw [`DutyMetrics`].  Fields that are not yet used by a policy
/// implementation are ignored; they are present so that the trait signature
/// remains stable as new policies are added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreditContext {
    /// The airport at which the crew member is currently based.
    pub crew_base: AirportCode,
    /// The crew member's contractual home base (may differ from `crew_base`
    /// during temporary reassignments).
    pub home_base: AirportCode,
    /// The calendar date on which the duty starts (local time of `crew_base`).
    pub applicable_date: NaiveDate,
    // Production extensions (add without breaking existing callers):
    // pub agreement_id: Option<String>,
    // pub crew_category: Option<CrewCategory>,
    // pub is_augmented_crew: bool,
}

// ── Credit components ─────────────────────────────────────────────────────────

/// Itemised breakdown of how [`DutyCredit::credited_hours`] was computed.
///
/// Stored alongside the total so that downstream consumers (analytics,
/// compliance checks, pay-slip generation) can inspect each component without
/// re-running the policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditComponents {
    /// Credit from operated (non-deadhead) block time.
    pub block_credit: f64,
    /// Credit from deadhead (positioning) block time, after applying the
    /// policy's `deadhead_credit_factor`.
    pub deadhead_credit: f64,
    /// Fixed credit added for overnight layovers (policy-dependent; zero for
    /// GERAD which does not include a layover premium).
    pub layover_credit: f64,
    /// Any premium credit (e.g. international, night, or holiday premiums).
    pub premium_credit: f64,
    /// `true` if a minimum-pay guarantee was applied and raised the total
    /// above the raw computed value.
    pub minimum_guarantee_applied: bool,
}

// ── Duty credit ───────────────────────────────────────────────────────────────

/// The result of applying a [`CreditPolicy`] to a single duty.
///
/// `credited_hours` is the canonical value used by:
/// - The **compliance engine** (Layer 2) to check monthly credit limits.
/// - The **fairness objective** (Layer 3) to balance workload across crew.
/// - The **cost model** (Layer 1b) to compute monetary pay cost.
///
/// `credit_cost` is a convenience field: `credited_hours × pay_rate`.  It is
/// separated from the credit calculation so that the cost model can be swapped
/// independently of the credit formula.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DutyCredit {
    /// Total contractually credited hours for this duty.
    pub credited_hours: f64,
    /// Monetary cost of this duty's credit at the applicable pay rate.
    /// Computed as `credited_hours × pay_rate_per_hour`.
    pub credit_cost: f64,
    /// Itemised breakdown of `credited_hours`.
    pub components: CreditComponents,
}

// ── Policy trait ──────────────────────────────────────────────────────────────

/// A contractual credit formula for a single duty.
///
/// # Determinism contract
///
/// Implementations **must** be pure functions.  The same `(metrics, context)`
/// pair must always produce the same [`DutyCredit`].  No mutable state, no
/// external I/O, no randomness.
///
/// # Per-duty scope
///
/// This trait computes credit for one duty at a time.  Roster-level
/// adjustments (monthly minimums, annual caps) are out of scope here and
/// belong in `AgreementPack` or a future `RosterCreditPolicy`.
pub trait CreditPolicy: Send + Sync {
    /// Compute the contractual credit for `metrics` under `context`.
    ///
    /// # Determinism
    ///
    /// This method must be a pure function — identical inputs produce identical
    /// outputs.  Implementations must not read from external state or perform
    /// I/O.
    fn compute(&self, metrics: &DutyMetrics, context: &CreditContext) -> DutyCredit;

    /// Return the provenance metadata for this policy.
    ///
    /// The returned value should be recorded in every optimisation artefact
    /// alongside the schedule so that results are reproducible.
    fn metadata(&self) -> CreditPolicyMetadata;
}

// ── GERAD G-2014-22 implementation ───────────────────────────────────────────

/// Credit policy implementing the GERAD G-2014-22 contractual formula.
///
/// # Official formula (Quesnel et al. §1)
///
/// ```text
/// credit = scheduled_flight_time + deadhead_credit_factor × deadhead_time
/// ```
///
/// where:
/// - `scheduled_flight_time` = block time of operated (non-deadhead) legs.
/// - `deadhead_time` = block time of deadhead (positioning) legs.
/// - `deadhead_credit_factor` = 0.5 (GERAD official value).
///
/// Briefing and debriefing time are **excluded** from credit.  They appear in
/// [`DutyMetrics::duty_duration`] but not in the contractual credit measure.
///
/// # Note on `creditedHours` input file
///
/// The GERAD generator subtracts 2 h per duty from the `creditedHours` file
/// before deriving base constraints.  That is a data-cleaning artefact, not
/// part of the contractual formula, and is not reproduced here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeradCreditPolicy {
    /// Fraction of deadhead block time that counts as credited hours.
    /// GERAD official value: `0.5`.
    pub deadhead_credit_factor: f64,
    /// Monetary pay rate per credited hour.
    /// Calibrate from the `creditedHours` cost data in the GERAD instance.
    /// Default placeholder: `100.0`.
    pub pay_rate_per_hour: f64,
}

impl Default for GeradCreditPolicy {
    fn default() -> Self {
        Self {
            deadhead_credit_factor: 0.5, // official GERAD definition
            pay_rate_per_hour: 100.0,    // placeholder; calibrate from creditedHours
        }
    }
}

impl CreditPolicy for GeradCreditPolicy {
    fn compute(&self, metrics: &DutyMetrics, _ctx: &CreditContext) -> DutyCredit {
        // flight_time = operated (non-deadhead) block time.
        // In DutyMetrics, flight_time == block_time when no per-leg deadhead
        // markers are available (conservative approximation in Duty::with_flags).
        let flight_hours = metrics.flight_time.num_minutes() as f64 / 60.0;

        // deadhead block time = total block − operated block.
        let deadhead_minutes = (metrics.block_time - metrics.flight_time).num_minutes();
        let deadhead_hours = deadhead_minutes as f64 / 60.0;

        let deadhead_credit = deadhead_hours * self.deadhead_credit_factor;
        let credited = flight_hours + deadhead_credit;
        let credit_cost = credited * self.pay_rate_per_hour;

        DutyCredit {
            credited_hours: credited,
            credit_cost,
            components: CreditComponents {
                block_credit: flight_hours,
                deadhead_credit,
                layover_credit: 0.0, // GERAD has no layover premium
                premium_credit: 0.0,
                minimum_guarantee_applied: false,
            },
        }
    }

    fn metadata(&self) -> CreditPolicyMetadata {
        CreditPolicyMetadata {
            id: "gerad-g2014-22-v1",
            authority: "GERAD G-2014-22",
            version: "1.0.0",
            description: "credit = flight_time + 0.5 × deadhead_time \
                          (Quesnel et al. §1; briefing/debriefing excluded)",
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    use super::super::duty::{BriefingOffsets, Duty, DutyId};
    use super::super::flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_leg(id: &str, origin: &str, dest: &str, dep_h: u32, arr_h: u32) -> FlightLeg {
        let base = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        FlightLeg::new(
            FlightLegId::new(id),
            FlightNumber::new(format!("XX{id}")),
            AirportCode::new(origin),
            AirportCode::new(dest),
            base + Duration::hours(dep_h as i64),
            base + Duration::hours(arr_h as i64),
            AircraftType::new("B738"),
        )
    }

    fn make_context() -> CreditContext {
        CreditContext {
            crew_base: AirportCode::new("YUL"),
            home_base: AirportCode::new("YUL"),
            applicable_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        }
    }

    fn duty_metrics_no_deadhead(block_hours: u32) -> DutyMetrics {
        // Single leg, no deadhead, no layover.
        let leg = make_leg("001", "YUL", "YYZ", 8, 8 + block_hours);
        let duty = Duty::new(DutyId::new("D1"), vec![leg]).unwrap();
        duty.metrics
    }

    fn duty_metrics_with_deadhead(flight_hours: u32, deadhead_hours: u32) -> DutyMetrics {
        // Two legs: first operated, second deadhead.
        // We use with_flags to mark the duty as containing a deadhead.
        // Note: without per-leg markers, flight_time == block_time in the
        // current implementation, so we manually construct DutyMetrics here
        // to test the credit formula in isolation.
        let _ = (flight_hours, deadhead_hours); // suppress unused warning
        // Build a real DutyMetrics with flight_time < block_time to simulate
        // a duty that has both operated and deadhead legs.
        let base = Utc.with_ymd_and_hms(2026, 7, 1, 8, 0, 0).unwrap();
        DutyMetrics {
            report_time: base - Duration::hours(1),
            release_time: base
                + Duration::hours((flight_hours + deadhead_hours) as i64)
                + Duration::minutes(30),
            duty_duration: Duration::hours((flight_hours + deadhead_hours + 1) as i64)
                + Duration::minutes(30),
            block_time: Duration::hours((flight_hours + deadhead_hours) as i64),
            flight_time: Duration::hours(flight_hours as i64),
            turnaround_time: Duration::hours(1) + Duration::minutes(30),
            sector_count: 2,
            contains_deadhead: true,
            contains_layover: false,
        }
    }

    // ── GeradCreditPolicy tests ───────────────────────────────────────────────

    #[test]
    fn gerad_default_has_correct_factor() {
        let policy = GeradCreditPolicy::default();
        assert_eq!(policy.deadhead_credit_factor, 0.5);
    }

    #[test]
    fn gerad_no_deadhead_credit_equals_flight_time() {
        let policy = GeradCreditPolicy::default();
        let metrics = duty_metrics_no_deadhead(3); // 3h block, no deadhead
        let ctx = make_context();
        let credit = policy.compute(&metrics, &ctx);

        // flight_time == block_time (no deadhead), so credit = 3h exactly.
        assert!(
            (credit.credited_hours - 3.0).abs() < 1e-9,
            "expected 3.0 credited hours, got {}",
            credit.credited_hours
        );
        assert_eq!(credit.components.deadhead_credit, 0.0);
        assert!(!credit.components.minimum_guarantee_applied);
    }

    #[test]
    fn gerad_deadhead_credit_is_half_deadhead_time() {
        let policy = GeradCreditPolicy::default();
        // 4h flight + 2h deadhead → credit = 4 + 0.5×2 = 5.0
        let metrics = duty_metrics_with_deadhead(4, 2);
        let ctx = make_context();
        let credit = policy.compute(&metrics, &ctx);

        assert!(
            (credit.credited_hours - 5.0).abs() < 1e-9,
            "expected 5.0 credited hours, got {}",
            credit.credited_hours
        );
        assert!(
            (credit.components.block_credit - 4.0).abs() < 1e-9,
            "expected block_credit 4.0, got {}",
            credit.components.block_credit
        );
        assert!(
            (credit.components.deadhead_credit - 1.0).abs() < 1e-9,
            "expected deadhead_credit 1.0, got {}",
            credit.components.deadhead_credit
        );
    }

    #[test]
    fn gerad_credit_cost_is_hours_times_rate() {
        let policy = GeradCreditPolicy {
            deadhead_credit_factor: 0.5,
            pay_rate_per_hour: 200.0,
        };
        let metrics = duty_metrics_no_deadhead(2); // 2h block
        let ctx = make_context();
        let credit = policy.compute(&metrics, &ctx);

        assert!(
            (credit.credit_cost - 400.0).abs() < 1e-9,
            "expected credit_cost 400.0, got {}",
            credit.credit_cost
        );
    }

    #[test]
    fn gerad_layover_credit_is_zero() {
        let policy = GeradCreditPolicy::default();
        let metrics = duty_metrics_no_deadhead(4);
        let ctx = make_context();
        let credit = policy.compute(&metrics, &ctx);
        assert_eq!(credit.components.layover_credit, 0.0);
        assert_eq!(credit.components.premium_credit, 0.0);
    }

    #[test]
    fn gerad_metadata_is_correct() {
        let policy = GeradCreditPolicy::default();
        let meta = policy.metadata();
        assert_eq!(meta.id, "gerad-g2014-22-v1");
        assert_eq!(meta.authority, "GERAD G-2014-22");
        assert_eq!(meta.version, "1.0.0");
        assert!(!meta.description.is_empty());
    }

    #[test]
    fn gerad_custom_deadhead_factor() {
        // Verify the formula works with a non-default factor.
        let policy = GeradCreditPolicy {
            deadhead_credit_factor: 1.0, // full deadhead credit
            pay_rate_per_hour: 100.0,
        };
        // 3h flight + 1h deadhead → credit = 3 + 1.0×1 = 4.0
        let metrics = duty_metrics_with_deadhead(3, 1);
        let ctx = make_context();
        let credit = policy.compute(&metrics, &ctx);

        assert!(
            (credit.credited_hours - 4.0).abs() < 1e-9,
            "expected 4.0 credited hours, got {}",
            credit.credited_hours
        );
    }

    #[test]
    fn credit_policy_metadata_is_copy() {
        // CreditPolicyMetadata must be Copy so it can be embedded in artefacts.
        let meta = CreditPolicyMetadata {
            id: "test",
            authority: "test",
            version: "0.0.1",
            description: "test policy",
        };
        let _copy = meta; // moves if not Copy
        let _also = meta; // would fail to compile if not Copy
    }

    #[test]
    fn duty_with_real_legs_produces_nonzero_credit() {
        let policy = GeradCreditPolicy::default();
        let leg = make_leg("001", "YUL", "YYZ", 8, 10); // 2h block
        let duty =
            Duty::new_with_offsets(DutyId::new("D1"), vec![leg], BriefingOffsets::DGCA).unwrap();
        let ctx = make_context();
        let credit = policy.compute(&duty.metrics, &ctx);

        assert!(credit.credited_hours > 0.0);
        assert!(credit.credit_cost > 0.0);
    }
}
