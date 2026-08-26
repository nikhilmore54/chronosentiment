//! Roster-level credit and cost aggregation — Phases 4 & 5.
//!
//! This module aggregates per-duty [`DutyCredit`] and [`DutyCost`] values
//! into roster-level summaries used by the optimisation objectives.
//!
//! # Architecture (UC-ARCH-001)
//!
//! ```text
//! [DutyCredit]  ──►  aggregate_roster_credits  ──►  RosterMetrics
//! [DutyCost]    ──►  (same call)
//! ```
//!
//! [`RosterMetrics`] is the single source of truth for the fairness objective
//! (balance `total_credited_hours` across crew) and the cost objective
//! (minimise `total_credit_cost`).
//!
//! # BaseCreditFloor (Phase 5)
//!
//! [`BaseCreditFloor`] encodes the minimum credited-hours guarantee per base
//! that appears in the GERAD instance file.  It is loaded once at startup and
//! passed to the compliance engine; it is not part of the credit formula.

use serde::{Deserialize, Serialize};

use super::cost::DutyCost;
use super::credit::DutyCredit;
use super::flight::AirportCode;

// ── RosterMetrics ─────────────────────────────────────────────────────────────

/// Aggregated credit and cost metrics for a single crew member's roster.
///
/// Computed by [`aggregate_roster_credits`] from the per-duty [`DutyCredit`]
/// and [`DutyCost`] slices produced by the credit and cost engines.
///
/// # Optimisation use
///
/// | Objective | Field |
/// |---|---|
/// | Fairness (balance workload) | `total_credited_hours` |
/// | Cost minimisation | `total_credit_cost` |
/// | Compliance (block-hour limits) | `total_block_hours` |
/// | Compliance (flight-time limits) | `total_flight_hours` |
/// | Compliance (FDP limits) | `total_fdp_hours` |
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RosterMetrics {
    /// Sum of `DutyCredit::credited_hours` across all duties.
    pub total_credited_hours: f64,
    /// Sum of `DutyCost::credit_cost` across all duties.
    pub total_credit_cost: f64,
    /// Sum of operated block hours (non-deadhead) across all duties.
    pub total_block_hours: f64,
    /// Sum of flight hours (operated block time) across all duties.
    pub total_flight_hours: f64,
    /// Sum of FDP (duty_duration) hours across all duties.
    pub total_fdp_hours: f64,
    /// Number of duties that contain at least one deadhead leg.
    pub deadhead_duty_count: u32,
    /// Number of duties that include an overnight layover.
    pub layover_duty_count: u32,
    /// Total number of duties in this roster.
    pub duty_count: u32,
}

impl RosterMetrics {
    /// Return a zeroed [`RosterMetrics`] (no duties assigned).
    pub fn zero() -> Self {
        Self {
            total_credited_hours: 0.0,
            total_credit_cost: 0.0,
            total_block_hours: 0.0,
            total_flight_hours: 0.0,
            total_fdp_hours: 0.0,
            deadhead_duty_count: 0,
            layover_duty_count: 0,
            duty_count: 0,
        }
    }
}

// ── Aggregation helper ────────────────────────────────────────────────────────

/// Per-duty input record for [`aggregate_roster_credits`].
///
/// Bundles the credit, cost, and raw time metrics for a single duty so that
/// the aggregation function has a single, coherent input type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DutyRecord {
    /// Contractual credit for this duty.
    pub credit: DutyCredit,
    /// Monetary cost for this duty.
    pub cost: DutyCost,
    /// Operated block hours (non-deadhead) for this duty.
    pub block_hours: f64,
    /// Flight hours (operated block time) for this duty.
    pub flight_hours: f64,
    /// FDP hours (duty_duration) for this duty.
    pub fdp_hours: f64,
    /// Whether this duty contains a deadhead leg.
    pub contains_deadhead: bool,
    /// Whether this duty includes an overnight layover.
    pub contains_layover: bool,
}

/// Aggregate a slice of per-duty records into a single [`RosterMetrics`].
///
/// This is a pure function: it performs no I/O and has no side effects.
///
/// # Example
///
/// ```rust,ignore
/// use coralys_airline::domain::roster_metrics::{DutyRecord, aggregate_roster_credits};
/// use coralys_airline::domain::credit::{CreditComponents, DutyCredit};
/// use coralys_airline::domain::cost::DutyCost;
///
/// let records = vec![
///     DutyRecord {
///         credit: DutyCredit {
///             credited_hours: 4.0,
///             credit_cost: 400.0,
///             components: CreditComponents {
///                 block_credit: 4.0,
///                 deadhead_credit: 0.0,
///                 layover_credit: 0.0,
///                 premium_credit: 0.0,
///                 minimum_guarantee_applied: false,
///             },
///         },
///         cost: DutyCost { credit_cost: 400.0, pay_rate: 100.0 },
///         block_hours: 4.0,
///         flight_hours: 4.0,
///         fdp_hours: 5.5,
///         contains_deadhead: false,
///         contains_layover: false,
///     },
/// ];
/// let metrics = aggregate_roster_credits(&records);
/// assert_eq!(metrics.total_credited_hours, 4.0);
/// assert_eq!(metrics.duty_count, 1);
/// ```
pub fn aggregate_roster_credits(records: &[DutyRecord]) -> RosterMetrics {
    let mut m = RosterMetrics::zero();
    for r in records {
        m.total_credited_hours += r.credit.credited_hours;
        m.total_credit_cost += r.cost.credit_cost;
        m.total_block_hours += r.block_hours;
        m.total_flight_hours += r.flight_hours;
        m.total_fdp_hours += r.fdp_hours;
        if r.contains_deadhead {
            m.deadhead_duty_count += 1;
        }
        if r.contains_layover {
            m.layover_duty_count += 1;
        }
        m.duty_count += 1;
    }
    m
}

// ── BaseCreditFloor ───────────────────────────────────────────────────────────

/// Minimum credited-hours guarantee per crew base (Phase 5).
///
/// Loaded from the GERAD instance file at startup.  The compliance engine
/// uses this to check that each crew member's roster meets the contractual
/// minimum.
///
/// # GERAD note
///
/// The `creditedHours` input file in GERAD G-2014-22 encodes the minimum
/// guaranteed credit per crew member per planning period.  The generator
/// subtracts 2 h per duty from this value before deriving base constraints
/// (data-cleaning step, not part of the contractual formula).
///
/// [`BaseCreditFloor`] stores the **raw** value from the instance file.
/// The 2 h/duty adjustment, if needed, is applied by the compliance engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseCreditFloor {
    /// The crew base to which this floor applies.
    pub base: AirportCode,
    /// Minimum credited hours guaranteed per planning period.
    pub minimum_credited_hours: f64,
    /// Slack fraction: how far below the minimum the optimiser may go before
    /// a soft penalty is applied.  `0.0` = hard floor; `0.05` = 5% slack.
    pub slack_fraction: f64,
}

impl BaseCreditFloor {
    /// Return `true` if `actual_hours` satisfies the floor (within slack).
    pub fn is_satisfied(&self, actual_hours: f64) -> bool {
        let effective_floor = self.minimum_credited_hours * (1.0 - self.slack_fraction);
        actual_hours >= effective_floor
    }

    /// Return the shortfall below the floor, or `0.0` if satisfied.
    pub fn shortfall(&self, actual_hours: f64) -> f64 {
        let effective_floor = self.minimum_credited_hours * (1.0 - self.slack_fraction);
        (effective_floor - actual_hours).max(0.0)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::cost::DutyCost;
    use super::super::credit::{CreditComponents, DutyCredit};
    use super::*;

    fn make_record(credited: f64, cost: f64, block: f64, fdp: f64) -> DutyRecord {
        DutyRecord {
            credit: DutyCredit {
                credited_hours: credited,
                credit_cost: cost,
                components: CreditComponents {
                    block_credit: block,
                    deadhead_credit: 0.0,
                    layover_credit: 0.0,
                    premium_credit: 0.0,
                    minimum_guarantee_applied: false,
                },
            },
            cost: DutyCost {
                credit_cost: cost,
                pay_rate: 100.0,
            },
            block_hours: block,
            flight_hours: block,
            fdp_hours: fdp,
            contains_deadhead: false,
            contains_layover: false,
        }
    }

    #[test]
    fn empty_records_gives_zero_metrics() {
        let m = aggregate_roster_credits(&[]);
        assert_eq!(m.total_credited_hours, 0.0);
        assert_eq!(m.duty_count, 0);
    }

    #[test]
    fn single_record_aggregates_correctly() {
        let records = vec![make_record(4.0, 400.0, 4.0, 5.5)];
        let m = aggregate_roster_credits(&records);
        assert_eq!(m.total_credited_hours, 4.0);
        assert_eq!(m.total_credit_cost, 400.0);
        assert_eq!(m.total_block_hours, 4.0);
        assert_eq!(m.total_fdp_hours, 5.5);
        assert_eq!(m.duty_count, 1);
    }

    #[test]
    fn multiple_records_sum_correctly() {
        let records = vec![
            make_record(3.0, 300.0, 3.0, 4.5),
            make_record(5.0, 500.0, 5.0, 6.5),
        ];
        let m = aggregate_roster_credits(&records);
        assert!((m.total_credited_hours - 8.0).abs() < 1e-9);
        assert!((m.total_credit_cost - 800.0).abs() < 1e-9);
        assert_eq!(m.duty_count, 2);
    }

    #[test]
    fn deadhead_and_layover_counts() {
        let mut r1 = make_record(3.0, 300.0, 3.0, 4.5);
        r1.contains_deadhead = true;
        let mut r2 = make_record(4.0, 400.0, 4.0, 5.5);
        r2.contains_layover = true;
        let mut r3 = make_record(2.0, 200.0, 2.0, 3.5);
        r3.contains_deadhead = true;
        r3.contains_layover = true;

        let m = aggregate_roster_credits(&[r1, r2, r3]);
        assert_eq!(m.deadhead_duty_count, 2);
        assert_eq!(m.layover_duty_count, 2);
        assert_eq!(m.duty_count, 3);
    }

    // ── BaseCreditFloor tests ─────────────────────────────────────────────────

    #[test]
    fn floor_satisfied_when_at_minimum() {
        let floor = BaseCreditFloor {
            base: AirportCode::new("YUL"),
            minimum_credited_hours: 80.0,
            slack_fraction: 0.0,
        };
        assert!(floor.is_satisfied(80.0));
        assert!(floor.is_satisfied(90.0));
        assert!(!floor.is_satisfied(79.9));
    }

    #[test]
    fn floor_shortfall_is_correct() {
        let floor = BaseCreditFloor {
            base: AirportCode::new("YUL"),
            minimum_credited_hours: 80.0,
            slack_fraction: 0.0,
        };
        assert!((floor.shortfall(75.0) - 5.0).abs() < 1e-9);
        assert_eq!(floor.shortfall(80.0), 0.0);
        assert_eq!(floor.shortfall(90.0), 0.0);
    }

    #[test]
    fn floor_with_slack_allows_below_minimum() {
        let floor = BaseCreditFloor {
            base: AirportCode::new("YUL"),
            minimum_credited_hours: 80.0,
            slack_fraction: 0.05, // 5% slack → effective floor = 76.0
        };
        assert!(floor.is_satisfied(76.0));
        assert!(!floor.is_satisfied(75.9));
    }

    #[test]
    fn floor_shortfall_respects_slack() {
        let floor = BaseCreditFloor {
            base: AirportCode::new("YUL"),
            minimum_credited_hours: 80.0,
            slack_fraction: 0.05, // effective floor = 76.0
        };
        // 74.0 is 2.0 below effective floor of 76.0
        assert!((floor.shortfall(74.0) - 2.0).abs() < 1e-9);
    }
}
