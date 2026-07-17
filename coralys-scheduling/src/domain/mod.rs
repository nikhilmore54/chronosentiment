//! Scheduling domain model — Layer 1.
//!
//! This module re-exports all domain entities and their associated types.
//! The hierarchy is:
//!
//! ```text
//! FlightLeg  (atomic unit)
//!     └── Duty        (ordered sequence of legs, single work block)
//!             └── Pairing     (ordered sequence of duties, base-to-base trip)
//!                     └── Rotation    (crew member's assigned pairings)
//!                             └── Roster  (complete schedule, all crew)
//! ```
//!
//! [`CrewMember`] is orthogonal to the leg hierarchy — it carries identity
//! and qualification data that the legality layer (Layer 2) uses when
//! validating assignments.

pub mod crew;
pub mod duty;
pub mod flight;
pub mod pairing;
pub mod roster;
pub mod rotation;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use crew::{CrewId, CrewMember, CrewRole, Qualification};
pub use duty::{Duty, DutyError, DutyId};
pub use flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};
pub use pairing::{Pairing, PairingError, PairingId};
pub use roster::{PlanningPeriod, Roster, RosterError, RosterId};
pub use rotation::{Rotation, RotationError, RotationId};