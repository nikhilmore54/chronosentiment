//! Rotation — a crew member's assigned sequence of pairings over a planning period.
//!
//! A [`Rotation`] binds a [`CrewMember`] to an ordered sequence of
//! [`Pairing`]s.  It is the unit of assignment: one rotation per crew member
//! per planning period.
//!
//! This module defines **structure only**.  Legality checks (maximum monthly
//! flying hours, minimum days off, etc.) belong to Layer 2.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::crew::CrewId;
use super::pairing::{Pairing, PairingId};

// ── Identifier ────────────────────────────────────────────────────────────────

/// Opaque identifier for a [`Rotation`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RotationId(String);

impl RotationId {
    /// Create a new [`RotationId`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RotationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Core entity ───────────────────────────────────────────────────────────────

/// A crew member's assigned sequence of pairings for a planning period.
///
/// # Invariants (enforced at construction)
/// - Contains at least one pairing.
/// - Pairings are ordered by start time (no pairing starts before the
///   previous one ends).
///
/// Legality constraints (maximum monthly flying hours, minimum days off,
/// etc.) are **not** enforced here — they belong to the legality layer
/// (Layer 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rotation {
    /// Unique identifier.
    pub id: RotationId,
    /// The crew member this rotation belongs to.
    pub crew_id: CrewId,
    /// Ordered sequence of pairings.
    pairings: Vec<Pairing>,
}

impl Rotation {
    /// Construct a new [`Rotation`].
    ///
    /// # Errors
    /// Returns [`RotationError`] if:
    /// - `pairings` is empty.
    /// - A pairing starts before the previous pairing ends.
    pub fn new(
        id: RotationId,
        crew_id: CrewId,
        pairings: Vec<Pairing>,
    ) -> Result<Self, RotationError> {
        if pairings.is_empty() {
            return Err(RotationError::Empty);
        }

        for i in 1..pairings.len() {
            if pairings[i].start() < pairings[i - 1].end() {
                return Err(RotationError::OutOfOrder {
                    earlier: pairings[i - 1].id.clone(),
                    later: pairings[i].id.clone(),
                });
            }
        }

        Ok(Self {
            id,
            crew_id,
            pairings,
        })
    }

    /// The ordered pairings in this rotation.
    pub fn pairings(&self) -> &[Pairing] {
        &self.pairings
    }

    /// Number of pairings in this rotation.
    pub fn pairing_count(&self) -> usize {
        self.pairings.len()
    }

    /// Start of the rotation (start of the first pairing).
    pub fn start(&self) -> DateTime<Utc> {
        self.pairings[0].start()
    }

    /// End of the rotation (end of the last pairing).
    pub fn end(&self) -> DateTime<Utc> {
        self.pairings[self.pairings.len() - 1].end()
    }

    /// Total elapsed time from start of first pairing to end of last pairing.
    pub fn elapsed_time(&self) -> Duration {
        self.end() - self.start()
    }

    /// Sum of block times across all pairings.
    pub fn total_block_time(&self) -> Duration {
        self.pairings.iter().map(|p| p.total_block_time()).sum()
    }

    /// Total number of duties across all pairings.
    pub fn total_duty_count(&self) -> usize {
        self.pairings.iter().map(|p| p.duty_count()).sum()
    }

    /// Total number of flight legs across all pairings.
    pub fn total_leg_count(&self) -> usize {
        self.pairings.iter().map(|p| p.total_leg_count()).sum()
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur when constructing a [`Rotation`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RotationError {
    /// The pairing sequence was empty.
    #[error("a rotation must contain at least one pairing")]
    Empty,

    /// A pairing starts before the previous pairing ends.
    #[error("pairing {later} starts before pairing {earlier} ends")]
    OutOfOrder { earlier: PairingId, later: PairingId },
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::crew::CrewId;
    use crate::domain::duty::{Duty, DutyId};
    use crate::domain::flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};
    use crate::domain::pairing::{Pairing, PairingId};
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

    fn make_pairing(id: &str, base: &str, duties: Vec<Duty>) -> Pairing {
        Pairing::new(PairingId::new(id), AirportCode::new(base), duties).unwrap()
    }

    /// P1: LHR→CDG (8–10), rest, CDG→LHR (14–16)
    fn pairing_1() -> Pairing {
        let d1 = make_duty("D1", vec![make_leg("L1", "LHR", "CDG", 8, 10)]);
        let d2 = make_duty("D2", vec![make_leg("L2", "CDG", "LHR", 14, 16)]);
        make_pairing("P1", "LHR", vec![d1, d2])
    }

    /// P2: LHR→FRA (20–22), rest, FRA→LHR (26–28)  (next day)
    fn pairing_2() -> Pairing {
        let d1 = make_duty("D3", vec![make_leg("L3", "LHR", "FRA", 20, 22)]);
        let d2 = make_duty("D4", vec![make_leg("L4", "FRA", "LHR", 26, 28)]);
        make_pairing("P2", "LHR", vec![d1, d2])
    }

    #[test]
    fn single_pairing_rotation_is_valid() {
        let r = Rotation::new(
            RotationId::new("R1"),
            CrewId::new("C1"),
            vec![pairing_1()],
        )
        .unwrap();
        assert_eq!(r.pairing_count(), 1);
        assert_eq!(r.total_duty_count(), 2);
        assert_eq!(r.total_leg_count(), 2);
    }

    #[test]
    fn two_pairing_rotation_metrics() {
        let r = Rotation::new(
            RotationId::new("R1"),
            CrewId::new("C1"),
            vec![pairing_1(), pairing_2()],
        )
        .unwrap();
        assert_eq!(r.pairing_count(), 2);
        assert_eq!(r.total_duty_count(), 4);
        assert_eq!(r.total_leg_count(), 4);
        // block = 4 × 2h = 8h
        assert_eq!(r.total_block_time(), Duration::hours(8));
        // elapsed = 28h - 8h = 20h
        assert_eq!(r.elapsed_time(), Duration::hours(20));
    }

    #[test]
    fn rejects_empty_pairings() {
        let err = Rotation::new(RotationId::new("R1"), CrewId::new("C1"), vec![]).unwrap_err();
        assert_eq!(err, RotationError::Empty);
    }

    #[test]
    fn rejects_out_of_order_pairings() {
        // pairing_2 ends at 28h, pairing_1 starts at 8h — reversed order
        let err = Rotation::new(
            RotationId::new("R1"),
            CrewId::new("C1"),
            vec![pairing_2(), pairing_1()],
        )
        .unwrap_err();
        assert!(matches!(err, RotationError::OutOfOrder { .. }));
    }

    #[test]
    fn serde_round_trip() {
        let r = Rotation::new(
            RotationId::new("R1"),
            CrewId::new("C1"),
            vec![pairing_1(), pairing_2()],
        )
        .unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let restored: Rotation = serde_json::from_str(&json).unwrap();
        assert_eq!(r, restored);
    }
}