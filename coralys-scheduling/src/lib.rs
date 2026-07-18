//! # coralys-scheduling
//!
//! Airline crew scheduling domain model for the Coralys platform.
//!
//! ## Architecture
//!
//! The crate is organised in layers.  Only Layer 1 (this crate) is
//! implemented; Layers 2–5 will be added in subsequent milestones.
//!
//! | Layer | Concern | Status |
//! |-------|---------|--------|
//! | 1 | Domain model (this crate) | ✅ Implemented |
//! | 2 | Operational correctness (legality rules) | 🔲 Stub only |
//! | 3 | Planner productivity (interactive evaluation) | 🔲 Planned |
//! | 4 | Optimisation (multi-objective scheduling) | 🔲 Planned |
//! | 5 | Operational resilience (disruption recovery) | 🔲 Planned |
//!
//! ## Domain hierarchy
//!
//! ```text
//! FlightLeg  ← atomic unit: one operated flight
//!     └── Duty        ← contiguous work block (ordered legs)
//!             └── Pairing     ← base-to-base trip (ordered duties + rests)
//!                     └── Rotation    ← crew member's assigned pairings
//!                             └── Roster  ← complete schedule for all crew
//! ```
//!
//! [`CrewMember`](domain::CrewMember) is orthogonal to the leg hierarchy —
//! it carries identity and qualification data referenced by the legality layer.
//!
//! ## Design principles
//!
//! - **Behavior-light entities**: domain structs carry data and enforce
//!   structural invariants only.  No legality or optimisation logic here.
//! - **Strong typing**: newtypes (`FlightNumber`, `AirportCode`, `CrewId`, …)
//!   prevent accidental misuse of raw strings.
//! - **Immutable by default**: all fields are `pub` for read access; mutation
//!   goes through constructors that enforce invariants.
//! - **Framework independence**: this crate has no dependency on
//!   `coralys-eval` or any other Coralys framework crate.

pub mod domain;
pub mod legality;
pub mod optimization;
pub mod planner;

// ── Top-level re-exports ──────────────────────────────────────────────────────
//
// Re-export the most commonly used types at the crate root so that downstream
// crates can write `use coralys_scheduling::FlightLeg` instead of the full
// path.

pub use domain::{
    AircraftType, AirportCode, CrewId, CrewMember, CrewRole, Duty, DutyError, DutyId, FlightLeg,
    FlightLegId, FlightNumber, Pairing, PairingError, PairingId, PlanningPeriod, Qualification,
    Roster, RosterError, RosterId, Rotation, RotationError, RotationId,
};

pub use legality::{
    EntityRef, LegalityChecker, LegalityRule, LegalityViolation, ViolationSeverity,
};