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
//!
//! # Credit Engine (Layer 1a — UC-ARCH-001)
//!
//! ```text
//! DutyMetrics  ──►  CreditPolicy  ──►  DutyCredit  ──►  RosterMetrics
//!                        │                                      │
//!                   CreditContext                          CostModel
//! ```
//!
//! - [`credit`]: contractual credit formula (`CreditPolicy`, `DutyCredit`,
//!   `GeradCreditPolicy`, `CreditPolicyMetadata`).
//! - [`cost`]: monetary cost model (`CostModel`, `DutyCost`,
//!   `FlatRateCostModel`).
//! - [`roster_metrics`]: roster-level aggregation (`RosterMetrics`,
//!   `BaseCreditFloor`, `aggregate_roster_credits`).

pub mod cost;
pub mod credit;
pub mod crew;
pub mod duty;
pub mod flight;
pub mod pairing;
pub mod roster;
pub mod roster_metrics;
pub mod rotation;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use cost::{CostContext, CostModel, DutyCost, FlatRateCostModel};
pub use credit::{
    CreditComponents, CreditContext, CreditPolicy, CreditPolicyMetadata, DutyCredit,
    GeradCreditPolicy,
};
pub use crew::{CrewId, CrewMember, CrewRole, Qualification};
// RosterError now has DuplicateCrewMember variant — re-export is unchanged
pub use duty::{Duty, DutyError, DutyId};
pub use flight::{AircraftType, AirportCode, FlightLeg, FlightLegId, FlightNumber};
pub use pairing::{Pairing, PairingError, PairingId};
pub use roster::{PlanningPeriod, Roster, RosterError, RosterId};
pub use roster_metrics::{BaseCreditFloor, DutyRecord, RosterMetrics, aggregate_roster_credits};
pub use rotation::{Rotation, RotationError, RotationId};
