//! Roster — the complete crew schedule for a planning period.
//!
//! A [`Roster`] is the top-level aggregate of the scheduling domain.  It
//! collects all [`Rotation`]s (one per crew member) and the full set of
//! [`FlightLeg`]s that must be covered for a given planning period.
//!
//! This module defines **structure only**.  Coverage checks (every leg must
//! be assigned to exactly the required crew complement), legality checks, and
//! optimisation objectives all belong to Layers 2–4.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::crew::CrewId;
use super::flight::{FlightLeg, FlightLegId};
use super::rotation::Rotation;

// ── Identifier ────────────────────────────────────────────────────────────────

/// Opaque identifier for a [`Roster`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RosterId(String);

impl RosterId {
    /// Create a new [`RosterId`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RosterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Planning period ───────────────────────────────────────────────────────────

/// A half-open planning period `[start, end)`.
///
/// All times are UTC.  The period is used to scope the roster and to validate
/// that all legs and rotations fall within the declared window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanningPeriod {
    /// Inclusive start of the planning period (UTC).
    pub start: DateTime<Utc>,
    /// Exclusive end of the planning period (UTC).
    pub end: DateTime<Utc>,
}

impl PlanningPeriod {
    /// Create a new [`PlanningPeriod`].
    ///
    /// # Panics
    /// Panics if `end` is not strictly after `start`.
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        assert!(end > start, "PlanningPeriod end must be after start");
        Self { start, end }
    }

    /// Returns `true` if `t` falls within `[start, end)`.
    pub fn contains(&self, t: DateTime<Utc>) -> bool {
        t >= self.start && t < self.end
    }
}

// ── Core entity ───────────────────────────────────────────────────────────────

/// The complete crew schedule for a planning period.
///
/// A `Roster` holds:
/// - The set of [`FlightLeg`]s that must be covered (indexed by [`FlightLegId`]).
/// - One [`Rotation`] per crew member (indexed by [`CrewId`]).
/// - The [`PlanningPeriod`] this roster covers.
///
/// # Invariants (enforced at construction)
/// - No duplicate leg IDs.
/// - No duplicate crew IDs (at most one rotation per crew member).
///
/// Coverage and legality invariants (every leg covered, no over-assignment,
/// etc.) are **not** enforced here — they belong to the legality layer
/// (Layer 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roster {
    /// Unique identifier.
    pub id: RosterId,
    /// The planning period this roster covers.
    pub period: PlanningPeriod,
    /// All flight legs that must be covered, keyed by leg ID.
    legs: HashMap<FlightLegId, FlightLeg>,
    /// One rotation per crew member, keyed by crew ID.
    rotations: HashMap<CrewId, Rotation>,
}

impl Roster {
    /// Construct a new [`Roster`].
    ///
    /// # Errors
    /// Returns [`RosterError`] if:
    /// - `legs` contains duplicate [`FlightLegId`]s.
    /// - `rotations` contains duplicate [`CrewId`]s (two rotations for the
    ///   same crew member).
    pub fn new(
        id: RosterId,
        period: PlanningPeriod,
        legs: Vec<FlightLeg>,
        rotations: Vec<Rotation>,
    ) -> Result<Self, RosterError> {
        // Index legs, checking for duplicates.
        let mut leg_map: HashMap<FlightLegId, FlightLeg> = HashMap::new();
        for leg in legs {
            if leg_map.contains_key(&leg.id) {
                return Err(RosterError::DuplicateLeg { id: leg.id });
            }
            leg_map.insert(leg.id.clone(), leg);
        }

        // Index rotations, checking for duplicates.
        let mut rotation_map: HashMap<CrewId, Rotation> = HashMap::new();
        for rotation in rotations {
            if rotation_map.contains_key(&rotation.crew_id) {
                return Err(RosterError::DuplicateCrewRotation {
                    crew_id: rotation.crew_id,
                });
            }
            rotation_map.insert(rotation.crew_id.clone(), rotation);
        }

        Ok(Self {
            id,
            period,
            legs: leg_map,
            rotations: rotation_map,
        })
    }

    // ── Leg accessors ─────────────────────────────────────────────────────────

    /// All flight legs in this roster.
    pub fn legs(&self) -> impl Iterator<Item = &FlightLeg> {
        self.legs.values()
    }

    /// Look up a leg by ID.
    pub fn leg(&self, id: &FlightLegId) -> Option<&FlightLeg> {
        self.legs.get(id)
    }

    /// Number of flight legs in this roster.
    pub fn leg_count(&self) -> usize {
        self.legs.len()
    }

    // ── Rotation accessors ────────────────────────────────────────────────────

    /// All rotations in this roster.
    pub fn rotations(&self) -> impl Iterator<Item = &Rotation> {
        self.rotations.values()
    }

    /// Look up the rotation for a crew member.
    pub fn rotation_for(&self, crew_id: &CrewId) -> Option<&Rotation> {
        self.rotations.get(crew_id)
    }

    /// Number of crew members with rotations in this roster.
    pub fn crew_count(&self) -> usize {
        self.rotations.len()
    }

    /// IDs of all crew members with rotations in this roster.
    pub fn crew_ids(&self) -> impl Iterator<Item = &CrewId> {
        self.rotations.keys()
    }

    // ── Aggregate metrics ─────────────────────────────────────────────────────

    /// Total number of flight legs across all rotations.
    ///
    /// Note: this counts *assigned* legs (legs appearing in rotations), which
    /// may differ from [`leg_count`](Self::leg_count) if coverage is incomplete.
    pub fn total_assigned_leg_count(&self) -> usize {
        self.rotations.values().map(|r| r.total_leg_count()).sum()
    }
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors that can occur when constructing a [`Roster`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RosterError {
    /// Two legs share the same [`FlightLegId`].
    #[error("duplicate flight leg ID: {id}")]
    DuplicateLeg { id: FlightLegId },

    /// Two rotations are assigned to the same crew member.
    #[error("crew member {crew_id} has more than one rotation")]
    DuplicateCrewRotation { crew_id: CrewId },
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::crew::CrewId;
    use crate::domain::duty::{Duty, DutyId};
    use crate::domain::flight::{AircraftType, AirportCode, FlightNumber};
    use crate::domain::pairing::{Pairing, PairingId};
    use crate::domain::rotation::{Rotation, RotationId};
    use chrono::TimeZone;

    fn make_leg(id: &str, origin: &str, dest: &str, dep_h: u32, arr_h: u32) -> FlightLeg {
        let base = Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap();
        FlightLeg::new(
            FlightLegId::new(id),
            FlightNumber::new(format!("XX{id}")),
            AirportCode::new(origin),
            AirportCode::new(dest),
            base + chrono::Duration::hours(dep_h as i64),
            base + chrono::Duration::hours(arr_h as i64),
            AircraftType::new("B738"),
        )
    }

    fn make_rotation(crew_id: &str) -> Rotation {
        let leg = make_leg("L1", "LHR", "CDG", 8, 10);
        let leg2 = make_leg("L2", "CDG", "LHR", 14, 16);
        let d1 = Duty::new(DutyId::new("D1"), vec![leg]).unwrap();
        let d2 = Duty::new(DutyId::new("D2"), vec![leg2]).unwrap();
        let p = Pairing::new(PairingId::new("P1"), AirportCode::new("LHR"), vec![d1, d2]).unwrap();
        Rotation::new(
            RotationId::new(format!("R-{crew_id}")),
            CrewId::new(crew_id),
            vec![p],
        )
        .unwrap()
    }

    fn sample_period() -> PlanningPeriod {
        PlanningPeriod::new(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 31, 23, 59, 59).unwrap(),
        )
    }

    #[test]
    fn empty_roster_is_valid() {
        let roster = Roster::new(
            RosterId::new("R1"),
            sample_period(),
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(roster.leg_count(), 0);
        assert_eq!(roster.crew_count(), 0);
    }

    #[test]
    fn roster_with_legs_and_rotations() {
        let legs = vec![
            make_leg("L1", "LHR", "CDG", 8, 10),
            make_leg("L2", "CDG", "LHR", 14, 16),
        ];
        let rotations = vec![make_rotation("C1"), make_rotation("C2")];
        let roster = Roster::new(RosterId::new("R1"), sample_period(), legs, rotations).unwrap();
        assert_eq!(roster.leg_count(), 2);
        assert_eq!(roster.crew_count(), 2);
    }

    #[test]
    fn leg_lookup_by_id() {
        let legs = vec![make_leg("L1", "LHR", "CDG", 8, 10)];
        let roster =
            Roster::new(RosterId::new("R1"), sample_period(), legs, vec![]).unwrap();
        assert!(roster.leg(&FlightLegId::new("L1")).is_some());
        assert!(roster.leg(&FlightLegId::new("MISSING")).is_none());
    }

    #[test]
    fn rotation_lookup_by_crew_id() {
        let rotations = vec![make_rotation("C1")];
        let roster =
            Roster::new(RosterId::new("R1"), sample_period(), vec![], rotations).unwrap();
        assert!(roster.rotation_for(&CrewId::new("C1")).is_some());
        assert!(roster.rotation_for(&CrewId::new("C99")).is_none());
    }

    #[test]
    fn rejects_duplicate_leg_ids() {
        let legs = vec![
            make_leg("L1", "LHR", "CDG", 8, 10),
            make_leg("L1", "CDG", "LHR", 14, 16), // same ID
        ];
        let err = Roster::new(RosterId::new("R1"), sample_period(), legs, vec![]).unwrap_err();
        assert!(matches!(err, RosterError::DuplicateLeg { .. }));
    }

    #[test]
    fn rejects_duplicate_crew_rotations() {
        let rotations = vec![make_rotation("C1"), make_rotation("C1")]; // same crew
        let err =
            Roster::new(RosterId::new("R1"), sample_period(), vec![], rotations).unwrap_err();
        assert!(matches!(err, RosterError::DuplicateCrewRotation { .. }));
    }

    #[test]
    fn planning_period_contains() {
        let period = sample_period();
        let inside = Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).unwrap();
        let before = Utc.with_ymd_and_hms(2026, 6, 30, 23, 59, 59).unwrap();
        assert!(period.contains(inside));
        assert!(!period.contains(before));
    }

    #[test]
    fn serde_round_trip() {
        let legs = vec![
            make_leg("L1", "LHR", "CDG", 8, 10),
            make_leg("L2", "CDG", "LHR", 14, 16),
        ];
        let rotations = vec![make_rotation("C1")];
        let roster =
            Roster::new(RosterId::new("R1"), sample_period(), legs, rotations).unwrap();
        let json = serde_json::to_string(&roster).unwrap();
        let restored: Roster = serde_json::from_str(&json).unwrap();
        assert_eq!(roster, restored);
    }
}