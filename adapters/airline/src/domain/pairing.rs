//! Pairing — a sequence of duties forming a self-contained crew trip.
//!
//! A [`Pairing`] begins and ends at the same base airport and covers one or
//! more consecutive [`Duty`] periods separated by rest periods.  It is the
//! unit that crew planning systems typically optimise over.
//!
//! This module defines **structure and relationships only**.  Legality checks
//! (minimum rest between duties, maximum pairing length, etc.) belong to
//! Layer 2.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::duty::{Duty, DutyId};
use super::flight::AirportCode;

// ── Identifier ────────────────────────────────────────────────────────────────

/// Opaque identifier for a [`Pairing`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PairingId(String);

impl PairingId {
    /// Create a new [`PairingId`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PairingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Core entity ───────────────────────────────────────────────────────────────

/// An ordered, non-empty sequence of duties forming a crew pairing.
///
/// # Invariants (enforced at construction)
/// - Contains at least one duty.
/// - Duties are ordered by start time (no duty starts before the previous ends).
/// - The pairing starts and ends at the same airport (`base`).
///
/// Rest periods between duties are implicit (the gap between one duty's end
/// and the next duty's start).  The legality layer checks that these gaps
/// satisfy minimum rest requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pairing {
    /// Unique identifier.
    pub id: PairingId,
    /// The base airport where this pairing starts and ends.
    pub base: AirportCode,
    /// Ordered sequence of duties.
    duties: Vec<Duty>,
}

impl Pairing {
    /// Construct a new [`Pairing`].
    ///
    /// # Errors
    /// Returns [`PairingError`] if:
    /// - `duties` is empty.
    /// - A duty starts before the previous duty ends.
    /// - The pairing does not start at `base`.
    /// - The pairing does not end at `base`.
    pub fn new(
        id: PairingId,
        base: AirportCode,
        duties: Vec<Duty>,
    ) -> Result<Self, PairingError> {
        if duties.is_empty() {
            return Err(PairingError::Empty);
        }

        // Check chronological ordering.
        for i in 1..duties.len() {
            if duties[i].start() < duties[i - 1].end() {
                return Err(PairingError::OutOfOrder {
                    earlier: duties[i - 1].id.clone(),
                    later: duties[i].id.clone(),
                });
            }
        }

        // Check that the pairing starts at base.
        let first_station = duties[0].report_station().clone();
        if first_station != base {
            return Err(PairingError::DoesNotStartAtBase {
                base: base.clone(),
                actual: first_station,
            });
        }

        // Check that the pairing ends at base.
        let last_station = duties[duties.len() - 1].release_station().clone();
        if last_station != base {
            return Err(PairingError::DoesNotEndAtBase {
                base: base.clone(),
                actual: last_station,
            });
        }

        Ok(Self { id, base, duties })
    }

    /// The ordered duties in this pairing.
    pub fn duties(&self) -> &[Duty] {
        &self.duties
    }

    /// Number of duties in this pairing.
    pub fn duty_count(&self) -> usize {
        self.duties.len()
    }

    /// Start of the pairing (start of the first duty).
    pub fn start(&self) -> DateTime<Utc> {
        self.duties[0].start()
    }

    /// End of the pairing (end of the last duty).
    pub fn end(&self) -> DateTime<Utc> {
        self.duties[self.duties.len() - 1].end()
    }

    /// Total elapsed time from start of first duty to end of last duty.
    pub fn elapsed_time(&self) -> Duration {
        self.end() - self.start()
    }

    /// Sum of block times across all duties.
    pub fn total_block_time(&self) -> Duration {
        self.duties.iter().map(|d| d.total_block_time()).sum()
    }

    /// Total number of flight legs across all duties.
    pub fn total_leg_count(&self) -> usize {
        self.duties.iter().map(|d| d.leg_count()).sum()
    }

    /// Rest periods between consecutive duties.
    ///
    /// Returns a `Vec` of `(duty_index, rest_duration)` pairs where
    /// `duty_index` is the index of the duty *after* the rest period.
    /// The legality layer uses these to check minimum rest requirements.
    pub fn rest_periods(&self) -> Vec<(usize, Duration)> {
        (1..self.duties.len())
            .map(|i| {
                let rest = self.duties[i].start() - self.duties[i - 1].end();
                (i, rest)
            })
            .collect()
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur when constructing a [`Pairing`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PairingError {
    /// The duty sequence was empty.
    #[error("a pairing must contain at least one duty")]
    Empty,

    /// A duty starts before the previous duty ends.
    #[error("duty {later} starts before duty {earlier} ends")]
    OutOfOrder { earlier: DutyId, later: DutyId },

    /// The first duty does not start at the declared base.
    #[error("pairing base is {base} but first duty starts at {actual}")]
    DoesNotStartAtBase { base: AirportCode, actual: AirportCode },

    /// The last duty does not end at the declared base.
    #[error("pairing base is {base} but last duty ends at {actual}")]
    DoesNotEndAtBase { base: AirportCode, actual: AirportCode },
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::duty::DutyId;
    use crate::domain::flight::{AircraftType, FlightLeg, FlightLegId, FlightNumber};
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

    fn make_duty(id: &str, legs: Vec<FlightLeg>) -> Duty {
        Duty::new(DutyId::new(id), legs).unwrap()
    }

    /// LHR→CDG (8–10), rest, CDG→LHR (14–16)
    fn two_duty_pairing() -> Pairing {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 14, 16)]);
        Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d1, d2]).unwrap()
    }

    #[test]
    fn single_duty_pairing_is_valid() {
        let d = make_duty("D1", vec![make_leg("L1", "LHR", "LHR", 8, 10)]);
        let p = Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d]).unwrap();
        assert_eq!(p.duty_count(), 1);
        assert_eq!(p.total_leg_count(), 1);
    }

    #[test]
    fn two_duty_pairing_metrics() {
        let p = two_duty_pairing();
        assert_eq!(p.duty_count(), 2);
        assert_eq!(p.total_leg_count(), 2);
        // elapsed = 16h - 8h = 8h
        assert_eq!(p.elapsed_time(), Duration::hours(8));
        // block = 2h + 2h = 4h
        assert_eq!(p.total_block_time(), Duration::hours(4));
    }

    #[test]
    fn rest_periods_are_correct() {
        let p = two_duty_pairing();
        let rests = p.rest_periods();
        assert_eq!(rests.len(), 1);
        let (idx, rest) = rests[0];
        assert_eq!(idx, 1);
        // D1 ends at 10h, D2 starts at 14h → 4h rest
        assert_eq!(rest, Duration::hours(4));
    }

    #[test]
    fn rejects_empty_duties() {
        let err =
            Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![]).unwrap_err();
        assert_eq!(err, PairingError::Empty);
    }

    #[test]
    fn rejects_out_of_order_duties() {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 14, 16)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 8, 10)]); // before d1
        let err =
            Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d1, d2]).unwrap_err();
        assert!(matches!(err, PairingError::OutOfOrder { .. }));
    }

    #[test]
    fn rejects_wrong_start_base() {
        let d = make_duty("D1", vec![make_leg("L1", "CDG", "LHR", 8, 10)]);
        let err =
            Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d]).unwrap_err();
        assert!(matches!(err, PairingError::DoesNotStartAtBase { .. }));
    }

    #[test]
    fn rejects_wrong_end_base() {
        let d = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let err =
            Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d]).unwrap_err();
        assert!(matches!(err, PairingError::DoesNotEndAtBase { .. }));
    }

    #[test]
    fn serde_round_trip() {
        let p = two_duty_pairing();
        let json = serde_json::to_string(&p).unwrap();
        let restored: Pairing = serde_json::from_str(&json).unwrap();
        assert_eq!(p, restored);
    }
}