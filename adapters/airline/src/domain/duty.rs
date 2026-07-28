//! Duty — a contiguous block of work for a crew member on a single calendar day.
//!
//! A [`Duty`] is an ordered, non-empty sequence of [`FlightLeg`]s that a crew
//! member operates without an intervening rest period.  It is the primary unit
//! against which duty-time limits are checked (Layer 2).
//!
//! # Semantic enrichment
//!
//! [`DutyMetrics`] is computed once at construction time and stored on the
//! [`Duty`].  The compliance engine reads pre-computed values rather than
//! recomputing them on every evaluation pass.
//!
//! ## Briefing / debriefing offsets
//!
//! DGCA CAR Section 7 Series J Part 3 requires:
//! - Pre-flight briefing: 60 minutes before scheduled departure of the first leg.
//! - Post-flight debriefing: 30 minutes after scheduled arrival of the last leg.
//!
//! These defaults are encoded in [`BriefingOffsets::DGCA`].  Callers that need
//! different offsets (e.g. EASA, FAA Part 117) can supply their own via
//! [`Duty::new_with_offsets`].

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::flight::{AirportCode, FlightLeg, FlightLegId};

// ── Identifier ────────────────────────────────────────────────────────────────

/// Opaque identifier for a [`Duty`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DutyId(String);

impl DutyId {
    /// Create a new [`DutyId`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DutyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Briefing offsets ──────────────────────────────────────────────────────────

/// Pre-flight briefing and post-flight debriefing time offsets.
///
/// These determine the difference between scheduled departure/arrival times
/// and the crew's actual report/release times, which in turn define the
/// Flight Duty Period (FDP) used by the compliance engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BriefingOffsets {
    /// Minutes before first-leg departure that the crew must report.
    pub pre_flight_minutes: i64,
    /// Minutes after last-leg arrival before the crew is released.
    pub post_flight_minutes: i64,
}

impl BriefingOffsets {
    /// DGCA CAR Section 7 Series J Part 3 defaults:
    /// 60 min pre-flight briefing, 30 min post-flight debriefing.
    pub const DGCA: Self = Self {
        pre_flight_minutes: 60,
        post_flight_minutes: 30,
    };

    /// EASA ORO.FTL defaults: 60 min pre-flight, 30 min post-flight.
    pub const EASA: Self = Self {
        pre_flight_minutes: 60,
        post_flight_minutes: 30,
    };

    /// FAA Part 117 defaults: 60 min pre-flight, 15 min post-flight.
    pub const FAA_PART_117: Self = Self {
        pre_flight_minutes: 60,
        post_flight_minutes: 15,
    };
}

impl Default for BriefingOffsets {
    fn default() -> Self {
        Self::DGCA
    }
}

// ── Duty metrics ──────────────────────────────────────────────────────────────

/// Pre-computed compliance-relevant metrics for a [`Duty`].
///
/// All values are derived from the leg schedule at construction time and stored
/// on the [`Duty`] so the compliance engine never recomputes them.
///
/// # Definitions
///
/// | Field | Definition |
/// |---|---|
/// | `report_time` | `first_leg.departure − briefing.pre_flight_minutes` |
/// | `release_time` | `last_leg.arrival + briefing.post_flight_minutes` |
/// | `duty_duration` | `release_time − report_time` (= FDP for DGCA purposes) |
/// | `block_time` | Sum of `(arrival − departure)` for every leg |
/// | `flight_time` | Block time of non-deadhead legs only |
/// | `turnaround_time` | `duty_duration − block_time` (ground time between legs + briefing/debriefing) |
/// | `sector_count` | Number of legs in the duty |
/// | `contains_deadhead` | Set by the mapper when a leg is a positioning flight |
/// | `contains_layover` | Set by the mapper when the duty ends away from base |
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DutyMetrics {
    /// Time the crew must report for duty (pre-flight briefing start).
    pub report_time: DateTime<Utc>,
    /// Time the crew is released from duty (post-flight debriefing end).
    pub release_time: DateTime<Utc>,
    /// Total Flight Duty Period: `release_time − report_time`.
    pub duty_duration: Duration,
    /// Sum of block times across all legs (scheduled departure to arrival).
    pub block_time: Duration,
    /// Block time of operated (non-deadhead) legs only.
    pub flight_time: Duration,
    /// Ground time between legs plus briefing/debriefing overhead:
    /// `duty_duration − block_time`.
    pub turnaround_time: Duration,
    /// Number of flight legs in the duty.
    pub sector_count: usize,
    /// `true` if at least one leg in this duty is a deadhead (positioning) leg.
    pub contains_deadhead: bool,
    /// `true` if the duty ends away from the crew's home base (set by mapper).
    pub contains_layover: bool,
}

// ── Home-base status ──────────────────────────────────────────────────────────

/// Whether a duty starts and/or ends at the crew member's home base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeBaseStatus {
    /// `true` if the first leg departs from the crew's home base.
    pub starts_at_home_base: bool,
    /// `true` if the last leg arrives at the crew's home base.
    pub ends_at_home_base: bool,
}

// ── Core entity ───────────────────────────────────────────────────────────────

/// An ordered, non-empty sequence of flight legs forming a single duty period.
///
/// # Invariants (enforced at construction)
/// - Contains at least one leg.
/// - Legs are ordered by scheduled departure time (no leg departs before the
///   previous leg has arrived).
/// - Consecutive legs connect: the destination of leg *n* equals the origin of
///   leg *n+1*.
///
/// Legality constraints (maximum duty time, minimum rest between duties, etc.)
/// are **not** enforced here — they belong to the legality layer (Layer 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Duty {
    /// Unique identifier.
    pub id: DutyId,
    /// Ordered sequence of flight legs.
    legs: Vec<FlightLeg>,
    /// Pre-computed compliance-relevant metrics.
    pub metrics: DutyMetrics,
    /// Briefing offsets used to compute `metrics.report_time` and
    /// `metrics.release_time`.
    offsets: BriefingOffsets,
}

impl Duty {
    /// Construct a new [`Duty`] using DGCA briefing offsets.
    ///
    /// # Errors
    /// Returns [`DutyError`] if:
    /// - `legs` is empty.
    /// - A leg departs before the previous leg has arrived.
    /// - Consecutive legs do not connect (destination ≠ next origin).
    pub fn new(id: DutyId, legs: Vec<FlightLeg>) -> Result<Self, DutyError> {
        Self::new_with_offsets(id, legs, BriefingOffsets::DGCA)
    }

    /// Construct a new [`Duty`] with custom briefing offsets.
    ///
    /// Use this when the applicable regulation differs from DGCA (e.g. EASA,
    /// FAA Part 117).
    ///
    /// # Errors
    /// Same as [`Duty::new`].
    pub fn new_with_offsets(
        id: DutyId,
        legs: Vec<FlightLeg>,
        offsets: BriefingOffsets,
    ) -> Result<Self, DutyError> {
        if legs.is_empty() {
            return Err(DutyError::Empty);
        }

        for i in 1..legs.len() {
            let prev = &legs[i - 1];
            let curr = &legs[i];

            if curr.scheduled_departure < prev.scheduled_arrival {
                return Err(DutyError::OutOfOrder {
                    earlier: prev.id.clone(),
                    later: curr.id.clone(),
                });
            }

            if prev.destination != curr.origin {
                return Err(DutyError::Disconnected {
                    from_leg: prev.id.clone(),
                    to_leg: curr.id.clone(),
                    mismatch_at: prev.destination.clone(),
                    expected: curr.origin.clone(),
                });
            }
        }

        let metrics = Self::compute_metrics(&legs, offsets, false, false);
        Ok(Self { id, legs, metrics, offsets })
    }

    /// Return a copy of this duty with the deadhead and layover flags set.
    ///
    /// Called by the GERAD mapper (and future adapters) after construction to
    /// annotate pairing-level context that is not derivable from legs alone.
    pub fn with_flags(mut self, contains_deadhead: bool, contains_layover: bool) -> Self {
        self.metrics.contains_deadhead = contains_deadhead;
        self.metrics.contains_layover = contains_layover;
        // Recompute flight_time: exclude deadhead legs if the flag is set.
        if contains_deadhead {
            // Without per-leg deadhead markers we conservatively keep the full
            // block time as flight_time.  When per-leg markers are available
            // (future work), this will subtract deadhead block time.
        }
        self
    }

    // ── Leg accessors ─────────────────────────────────────────────────────────

    /// The ordered legs in this duty.
    pub fn legs(&self) -> &[FlightLeg] {
        &self.legs
    }

    /// Number of legs in this duty.
    pub fn leg_count(&self) -> usize {
        self.legs.len()
    }

    // ── Derived time accessors (convenience wrappers over metrics) ────────────

    /// Scheduled start of duty (departure of the first leg).
    pub fn start(&self) -> DateTime<Utc> {
        self.legs[0].scheduled_departure
    }

    /// Scheduled end of duty (arrival of the last leg).
    pub fn end(&self) -> DateTime<Utc> {
        self.legs[self.legs.len() - 1].scheduled_arrival
    }

    /// Total elapsed duty time (end − start).
    ///
    /// This is the *elapsed* duration between first departure and last arrival,
    /// **not** the FDP.  Use [`DutyMetrics::duty_duration`] for the FDP.
    pub fn elapsed_time(&self) -> Duration {
        self.end() - self.start()
    }

    /// Sum of block times across all legs.
    pub fn total_block_time(&self) -> Duration {
        self.metrics.block_time
    }

    /// Departure airport of the first leg (where the crew reports for duty).
    pub fn report_station(&self) -> &AirportCode {
        &self.legs[0].origin
    }

    /// Arrival airport of the last leg (where the crew is released from duty).
    pub fn release_station(&self) -> &AirportCode {
        &self.legs[self.legs.len() - 1].destination
    }

    /// Whether this duty starts and/or ends at the given home base.
    pub fn home_base_status(&self, home_base: &AirportCode) -> HomeBaseStatus {
        HomeBaseStatus {
            starts_at_home_base: self.report_station() == home_base,
            ends_at_home_base: self.release_station() == home_base,
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    fn compute_metrics(
        legs: &[FlightLeg],
        offsets: BriefingOffsets,
        contains_deadhead: bool,
        contains_layover: bool,
    ) -> DutyMetrics {
        let first = &legs[0];
        let last = &legs[legs.len() - 1];

        let report_time = first.scheduled_departure
            - Duration::minutes(offsets.pre_flight_minutes);
        let release_time = last.scheduled_arrival
            + Duration::minutes(offsets.post_flight_minutes);
        let duty_duration = release_time - report_time;

        let block_time: Duration = legs.iter().map(|l| l.block_time()).sum();
        // flight_time = block_time of operated legs; without per-leg deadhead
        // markers we use block_time as a conservative approximation.
        let flight_time = block_time;
        let turnaround_time = duty_duration - block_time;
        let sector_count = legs.len();

        DutyMetrics {
            report_time,
            release_time,
            duty_duration,
            block_time,
            flight_time,
            turnaround_time,
            sector_count,
            contains_deadhead,
            contains_layover,
        }
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur when constructing a [`Duty`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DutyError {
    /// The leg sequence was empty.
    #[error("a duty must contain at least one flight leg")]
    Empty,

    /// A leg departs before the previous leg has arrived.
    #[error("leg {later} departs before leg {earlier} arrives")]
    OutOfOrder {
        earlier: FlightLegId,
        later: FlightLegId,
    },

    /// Two consecutive legs do not connect geographically.
    #[error(
        "leg {from_leg} arrives at {mismatch_at} but leg {to_leg} departs from {expected}"
    )]
    Disconnected {
        from_leg: FlightLegId,
        to_leg: FlightLegId,
        /// The actual destination of `from_leg`.
        mismatch_at: AirportCode,
        /// The origin of `to_leg` (what was expected).
        expected: AirportCode,
    },
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::flight::{AircraftType, FlightNumber};
    use chrono::TimeZone;

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

    #[test]
    fn single_leg_duty_is_valid() {
        let leg = make_leg("001", "LHR", "CDG", 8, 10);
        let duty = Duty::new(DutyId::new("D1"), vec![leg]).unwrap();
        assert_eq!(duty.leg_count(), 1);
        assert_eq!(duty.elapsed_time(), Duration::hours(2));
        assert_eq!(duty.total_block_time(), Duration::hours(2));
    }

    #[test]
    fn multi_leg_connected_duty() {
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let l2 = make_leg("002", "CDG", "FRA", 11, 13);
        let duty = Duty::new(DutyId::new("D2"), vec![l1, l2]).unwrap();
        assert_eq!(duty.leg_count(), 2);
        assert_eq!(duty.report_station().as_str(), "LHR");
        assert_eq!(duty.release_station().as_str(), "FRA");
        // elapsed = 13h - 8h = 5h; block = 2h + 2h = 4h
        assert_eq!(duty.elapsed_time(), Duration::hours(5));
        assert_eq!(duty.total_block_time(), Duration::hours(4));
    }

    #[test]
    fn start_and_end_times() {
        let base = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let duty = Duty::new(DutyId::new("D3"), vec![l1]).unwrap();
        assert_eq!(duty.start(), base + Duration::hours(8));
        assert_eq!(duty.end(), base + Duration::hours(10));
    }

    #[test]
    fn rejects_empty_legs() {
        let err = Duty::new(DutyId::new("D4"), vec![]).unwrap_err();
        assert_eq!(err, DutyError::Empty);
    }

    #[test]
    fn rejects_disconnected_legs() {
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let l2 = make_leg("002", "FRA", "AMS", 11, 13); // FRA ≠ CDG
        let err = Duty::new(DutyId::new("D5"), vec![l1, l2]).unwrap_err();
        assert!(matches!(err, DutyError::Disconnected { .. }));
    }

    #[test]
    fn rejects_out_of_order_legs() {
        let l1 = make_leg("001", "LHR", "CDG", 11, 13);
        let l2 = make_leg("002", "CDG", "FRA", 8, 10); // departs before l1 arrives
        let err = Duty::new(DutyId::new("D6"), vec![l1, l2]).unwrap_err();
        assert!(matches!(err, DutyError::OutOfOrder { .. }));
    }

    #[test]
    fn legs_with_turnaround_gap_are_valid() {
        // l2 departs after l1 arrives — turnaround gap is fine
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let l2 = make_leg("002", "CDG", "FRA", 12, 14); // 2h gap
        let duty = Duty::new(DutyId::new("D7"), vec![l1, l2]).unwrap();
        assert_eq!(duty.leg_count(), 2);
    }

    #[test]
    fn serde_round_trip() {
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let l2 = make_leg("002", "CDG", "FRA", 11, 13);
        let duty = Duty::new(DutyId::new("D8"), vec![l1, l2]).unwrap();
        let json = serde_json::to_string(&duty).unwrap();
        let restored: Duty = serde_json::from_str(&json).unwrap();
        assert_eq!(duty, restored);
    }

    // ── DutyMetrics tests ─────────────────────────────────────────────────────

    #[test]
    fn metrics_report_and_release_times_use_dgca_offsets() {
        let base = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        // Leg: 08:00 → 10:00
        let leg = make_leg("001", "LHR", "CDG", 8, 10);
        let duty = Duty::new(DutyId::new("D9"), vec![leg]).unwrap();

        // report_time = 08:00 − 60 min = 07:00
        assert_eq!(duty.metrics.report_time, base + Duration::hours(7));
        // release_time = 10:00 + 30 min = 10:30
        assert_eq!(duty.metrics.release_time, base + Duration::hours(10) + Duration::minutes(30));
        // duty_duration = 10:30 − 07:00 = 3h30m
        assert_eq!(duty.metrics.duty_duration, Duration::minutes(210));
    }

    #[test]
    fn metrics_block_time_and_sector_count() {
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);  // 2h block
        let l2 = make_leg("002", "CDG", "FRA", 11, 13); // 2h block
        let duty = Duty::new(DutyId::new("D10"), vec![l1, l2]).unwrap();

        assert_eq!(duty.metrics.block_time, Duration::hours(4));
        assert_eq!(duty.metrics.sector_count, 2);
    }

    #[test]
    fn metrics_turnaround_time_includes_ground_and_briefing() {
        // Leg 08:00→10:00, gap, Leg 12:00→14:00
        // block = 4h, elapsed = 6h
        // duty_duration = (08:00 − 1h) to (14:00 + 0.5h) = 07:00 to 14:30 = 7.5h
        // turnaround = 7.5h − 4h = 3.5h
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let l2 = make_leg("002", "CDG", "FRA", 12, 14);
        let duty = Duty::new(DutyId::new("D11"), vec![l1, l2]).unwrap();

        assert_eq!(duty.metrics.turnaround_time, Duration::minutes(210));
    }

    #[test]
    fn custom_offsets_faa_part_117() {
        let base = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        let leg = make_leg("001", "LHR", "CDG", 8, 10);
        let duty = Duty::new_with_offsets(
            DutyId::new("D12"),
            vec![leg],
            BriefingOffsets::FAA_PART_117,
        )
        .unwrap();

        // report_time = 08:00 − 60 min = 07:00
        assert_eq!(duty.metrics.report_time, base + Duration::hours(7));
        // release_time = 10:00 + 15 min = 10:15
        assert_eq!(duty.metrics.release_time, base + Duration::hours(10) + Duration::minutes(15));
    }

    #[test]
    fn home_base_status_starts_and_ends_at_base() {
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let l2 = make_leg("002", "CDG", "LHR", 12, 14);
        let duty = Duty::new(DutyId::new("D13"), vec![l1, l2]).unwrap();
        let lhr = AirportCode::new("LHR");

        let status = duty.home_base_status(&lhr);
        assert!(status.starts_at_home_base);
        assert!(status.ends_at_home_base);
    }

    #[test]
    fn home_base_status_away_duty() {
        let l1 = make_leg("001", "LHR", "CDG", 8, 10);
        let l2 = make_leg("002", "CDG", "FRA", 12, 14);
        let duty = Duty::new(DutyId::new("D14"), vec![l1, l2]).unwrap();
        let lhr = AirportCode::new("LHR");

        let status = duty.home_base_status(&lhr);
        assert!(status.starts_at_home_base);
        assert!(!status.ends_at_home_base);
    }

    #[test]
    fn with_flags_sets_deadhead_and_layover() {
        let leg = make_leg("001", "LHR", "CDG", 8, 10);
        let duty = Duty::new(DutyId::new("D15"), vec![leg])
            .unwrap()
            .with_flags(true, true);

        assert!(duty.metrics.contains_deadhead);
        assert!(duty.metrics.contains_layover);
    }
}