//! Flight leg — the atomic unit of airline operations.
//!
//! A [`FlightLeg`] represents a single operated flight from one airport to
//! another.  It carries identity, timing, and equipment information but
//! contains **no legality or optimisation logic** — those concerns belong to
//! Layers 2 and 4 respectively.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// ── Newtype wrappers ──────────────────────────────────────────────────────────

/// IATA or ICAO flight number, e.g. `"BA0117"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlightNumber(String);

impl FlightNumber {
    /// Create a new [`FlightNumber`].  The value is stored as-is; callers are
    /// responsible for normalisation (upper-case, trimming, etc.).
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FlightNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Three-letter IATA airport code, e.g. `"LHR"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AirportCode(String);

impl AirportCode {
    /// Create a new [`AirportCode`], normalised to upper-case.
    ///
    /// # Panics (debug builds only)
    /// Panics if `s` is not exactly three ASCII alphabetic characters.
    pub fn new(s: impl Into<String>) -> Self {
        let s = s.into();
        debug_assert!(
            s.len() == 3 && s.chars().all(|c| c.is_ascii_alphabetic()),
            "AirportCode must be exactly three ASCII letters, got {s:?}"
        );
        Self(s.to_uppercase())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AirportCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// ICAO aircraft type designator, e.g. `"B738"` for Boeing 737-800.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AircraftType(String);

impl AircraftType {
    /// Create a new [`AircraftType`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AircraftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Opaque identifier for a [`FlightLeg`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlightLegId(String);

impl FlightLegId {
    /// Create a new [`FlightLegId`].
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FlightLegId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Core entity ───────────────────────────────────────────────────────────────

/// A single operated flight leg.
///
/// `FlightLeg` is the atomic scheduling unit.  Higher-level constructs
/// ([`Duty`](super::duty::Duty), [`Pairing`](super::pairing::Pairing), …)
/// are built by composing sequences of legs.
///
/// All times are stored in UTC; timezone conversion is a presentation concern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlightLeg {
    /// Unique identifier for this leg within the schedule.
    pub id: FlightLegId,
    /// Marketed flight number.
    pub flight_number: FlightNumber,
    /// Departure airport.
    pub origin: AirportCode,
    /// Arrival airport.
    pub destination: AirportCode,
    /// Scheduled departure time (UTC).
    pub scheduled_departure: DateTime<Utc>,
    /// Scheduled arrival time (UTC).
    pub scheduled_arrival: DateTime<Utc>,
    /// Aircraft type required for this leg.
    pub aircraft_type: AircraftType,
}

impl FlightLeg {
    /// Construct a new [`FlightLeg`].
    ///
    /// # Panics
    /// Panics if `scheduled_arrival` is not strictly after `scheduled_departure`.
    pub fn new(
        id: FlightLegId,
        flight_number: FlightNumber,
        origin: AirportCode,
        destination: AirportCode,
        scheduled_departure: DateTime<Utc>,
        scheduled_arrival: DateTime<Utc>,
        aircraft_type: AircraftType,
    ) -> Self {
        assert!(
            scheduled_arrival > scheduled_departure,
            "scheduled_arrival must be after scheduled_departure"
        );
        Self {
            id,
            flight_number,
            origin,
            destination,
            scheduled_departure,
            scheduled_arrival,
            aircraft_type,
        }
    }

    /// Block time (scheduled flight duration).
    pub fn block_time(&self) -> Duration {
        self.scheduled_arrival - self.scheduled_departure
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn dep() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 1, 8, 0, 0).unwrap()
    }

    fn arr() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 1, 10, 30, 0).unwrap()
    }

    fn sample_leg() -> FlightLeg {
        FlightLeg::new(
            FlightLegId::new("FL001"),
            FlightNumber::new("BA0117"),
            AirportCode::new("LHR"),
            AirportCode::new("JFK"),
            dep(),
            arr(),
            AircraftType::new("B744"),
        )
    }

    #[test]
    fn block_time_is_correct() {
        let leg = sample_leg();
        assert_eq!(leg.block_time(), Duration::minutes(150));
    }

    #[test]
    fn airport_code_is_uppercased() {
        let code = AirportCode::new("lhr");
        assert_eq!(code.as_str(), "LHR");
    }

    #[test]
    fn flight_leg_display_fields() {
        let leg = sample_leg();
        assert_eq!(leg.flight_number.as_str(), "BA0117");
        assert_eq!(leg.origin.as_str(), "LHR");
        assert_eq!(leg.destination.as_str(), "JFK");
    }

    #[test]
    fn flight_leg_id_display() {
        let id = FlightLegId::new("FL001");
        assert_eq!(id.to_string(), "FL001");
    }

    #[test]
    #[should_panic(expected = "scheduled_arrival must be after scheduled_departure")]
    fn rejects_arrival_before_departure() {
        FlightLeg::new(
            FlightLegId::new("BAD"),
            FlightNumber::new("XX0001"),
            AirportCode::new("SYD"),
            AirportCode::new("MEL"),
            arr(), // swapped intentionally
            dep(),
            AircraftType::new("A320"),
        );
    }

    #[test]
    fn serde_round_trip() {
        let leg = sample_leg();
        let json = serde_json::to_string(&leg).unwrap();
        let restored: FlightLeg = serde_json::from_str(&json).unwrap();
        assert_eq!(leg, restored);
    }
}
