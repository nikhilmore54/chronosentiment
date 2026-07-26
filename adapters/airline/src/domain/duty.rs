//! Duty — a contiguous block of work for a crew member on a single calendar day.
//!
//! A [`Duty`] is an ordered, non-empty sequence of [`FlightLeg`]s that a crew
//! member operates without an intervening rest period.  It is the primary unit
//! against which duty-time limits are checked (Layer 2).
//!
//! This module defines the **structure** of a duty only.  No legality checks
//! (maximum duty time, minimum rest, etc.) are performed here.

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
}

impl Duty {
    /// Construct a new [`Duty`] from an ordered sequence of legs.
    ///
    /// # Errors
    /// Returns [`DutyError`] if:
    /// - `legs` is empty.
    /// - A leg departs before the previous leg has arrived.
    /// - Consecutive legs do not connect (destination ≠ next origin).
    pub fn new(id: DutyId, legs: Vec<FlightLeg>) -> Result<Self, DutyError> {
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

        Ok(Self { id, legs })
    }

    /// The ordered legs in this duty.
    pub fn legs(&self) -> &[FlightLeg] {
        &self.legs
    }

    /// Number of legs in this duty.
    pub fn leg_count(&self) -> usize {
        self.legs.len()
    }

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
    /// This is the *elapsed* duration, not the sum of block times.
    /// The legality layer uses this to check duty-time limits.
    pub fn elapsed_time(&self) -> Duration {
        self.end() - self.start()
    }

    /// Sum of block times across all legs.
    pub fn total_block_time(&self) -> Duration {
        self.legs.iter().map(|l| l.block_time()).sum()
    }

    /// Departure airport of the first leg (where the crew reports for duty).
    pub fn report_station(&self) -> &AirportCode {
        &self.legs[0].origin
    }

    /// Arrival airport of the last leg (where the crew is released from duty).
    pub fn release_station(&self) -> &AirportCode {
        &self.legs[self.legs.len() - 1].destination
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
}