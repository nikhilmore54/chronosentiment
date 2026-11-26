//! Crew member identity and qualification.
//!
//! [`CrewMember`] carries identity, role, and qualification information.
//! It does **not** contain scheduling state (assigned pairings, rosters, etc.)
//! — those associations live in [`Rotation`](super::rotation::Rotation) and
//! [`Roster`](super::roster::Roster).

use serde::{Deserialize, Serialize};
use std::fmt;

use super::flight::{AircraftType, AirportCode};

// ── Identifier ────────────────────────────────────────────────────────────────

/// Opaque identifier for a [`CrewMember`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrewId(String);

impl CrewId {
    /// Create a new [`CrewId`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CrewId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Role ─────────────────────────────────────────────────────────────────────

/// The operational role of a crew member.
///
/// Roles determine which positions on a flight a crew member may fill and
/// which legality rules apply to them.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrewRole {
    /// Pilot in command.
    Captain,
    /// Second-in-command / co-pilot.
    FirstOfficer,
    /// Senior cabin crew / purser.
    CabinCrewSenior,
    /// Standard cabin crew.
    CabinCrew,
    /// Relief pilot for long-haul operations.
    ReliefPilot,
}

impl fmt::Display for CrewRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CrewRole::Captain => "Captain",
            CrewRole::FirstOfficer => "First Officer",
            CrewRole::CabinCrewSenior => "Senior Cabin Crew",
            CrewRole::CabinCrew => "Cabin Crew",
            CrewRole::ReliefPilot => "Relief Pilot",
        };
        f.write_str(s)
    }
}

// ── Qualification ─────────────────────────────────────────────────────────────

/// A type rating or qualification held by a crew member.
///
/// Qualifications are additive — a crew member may hold multiple.
/// The legality layer (Layer 2) is responsible for checking that a crew
/// member's qualifications are sufficient for a given flight leg.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Qualification {
    /// The aircraft type this qualification covers.
    pub aircraft_type: AircraftType,
}

impl Qualification {
    /// Create a new [`Qualification`] for the given aircraft type.
    pub fn new(aircraft_type: AircraftType) -> Self {
        Self { aircraft_type }
    }
}

// ── Core entity ───────────────────────────────────────────────────────────────

/// A crew member — pilot or cabin crew.
///
/// `CrewMember` is an identity and qualification record.  Scheduling
/// assignments (which pairings, rotations, or rosters a crew member is
/// assigned to) are represented in the higher-level domain entities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewMember {
    /// Unique identifier.
    pub id: CrewId,
    /// Display name.
    pub name: String,
    /// Operational role.
    pub role: CrewRole,
    /// Type ratings and qualifications held.
    pub qualifications: Vec<Qualification>,
    /// Home base airport code.
    pub base: AirportCode,
}

impl CrewMember {
    /// Construct a new [`CrewMember`].
    pub fn new(
        id: CrewId,
        name: impl Into<String>,
        role: CrewRole,
        qualifications: Vec<Qualification>,
        base: AirportCode,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            role,
            qualifications,
            base,
        }
    }

    /// Returns `true` if this crew member holds a qualification for the given
    /// aircraft type.
    ///
    /// This is a **data query**, not a legality check — the legality layer
    /// decides whether the qualification is sufficient for a specific operation.
    pub fn is_qualified_for(&self, aircraft_type: &AircraftType) -> bool {
        self.qualifications
            .iter()
            .any(|q| &q.aircraft_type == aircraft_type)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_crew() -> CrewMember {
        CrewMember::new(
            CrewId::new("C001"),
            "Alice Smith",
            CrewRole::Captain,
            vec![
                Qualification::new(AircraftType::new("B738")),
                Qualification::new(AircraftType::new("A320")),
            ],
            AirportCode::new("LHR"),
        )
    }

    #[test]
    fn qualified_for_held_type() {
        let crew = sample_crew();
        assert!(crew.is_qualified_for(&AircraftType::new("B738")));
        assert!(crew.is_qualified_for(&AircraftType::new("A320")));
    }

    #[test]
    fn not_qualified_for_unrated_type() {
        let crew = sample_crew();
        assert!(!crew.is_qualified_for(&AircraftType::new("B744")));
    }

    #[test]
    fn crew_id_display() {
        let id = CrewId::new("C001");
        assert_eq!(id.to_string(), "C001");
    }

    #[test]
    fn crew_role_display() {
        assert_eq!(CrewRole::Captain.to_string(), "Captain");
        assert_eq!(CrewRole::FirstOfficer.to_string(), "First Officer");
        assert_eq!(CrewRole::CabinCrewSenior.to_string(), "Senior Cabin Crew");
        assert_eq!(CrewRole::CabinCrew.to_string(), "Cabin Crew");
        assert_eq!(CrewRole::ReliefPilot.to_string(), "Relief Pilot");
    }

    #[test]
    fn empty_qualifications() {
        let crew = CrewMember::new(
            CrewId::new("C002"),
            "Bob Jones",
            CrewRole::CabinCrew,
            vec![],
            AirportCode::new("CDG"),
        );
        assert!(!crew.is_qualified_for(&AircraftType::new("A320")));
    }

    #[test]
    fn serde_round_trip() {
        let crew = sample_crew();
        let json = serde_json::to_string(&crew).unwrap();
        let restored: CrewMember = serde_json::from_str(&json).unwrap();
        assert_eq!(crew, restored);
    }
}
