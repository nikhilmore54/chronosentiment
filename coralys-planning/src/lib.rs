//! `coralys-planning` — generic workforce planning capability.
//!
//! This crate defines the minimal execution contract that any planning domain
//! must satisfy to participate in the Coralys platform.  It sits between
//! `coralys-core` (primitive platform traits) and domain-specific Solution
//! Adapters (e.g. `adapters/ultracrew`, `adapters/airline`).
//!
//! Dependency direction:
//!   Application → Solution Adapter → coralys-planning → coralys-core
//!
//! # Interface
//!
//! The five traits defined here correspond directly to the Phase 1 domain
//! comparison (see `docs/PHASE1_DOMAIN_COMPARISON.md` §6):
//!
//! | Trait             | INRC2 mapping                        | Airline mapping        |
//! |-------------------|--------------------------------------|------------------------|
//! | `Worker`          | `InrcNurse`                          | `CrewMember`           |
//! | `PlanningUnit`    | shift assignment `(nurse, shift, day)`| `Pairing`              |
//! | `CoverageDemand`  | `InrcRequirement` (shift×skill×day)  | `FlightLeg` coverage   |
//! | `PlanningSolution`| schedule matrix                      | crew roster            |
//! | `PlanningScenario`| `InrcScenario`                       | airline scenario        |

/// A worker who can be assigned to planning units.
///
/// Workers have an identity (`id`) and may carry domain-specific attributes
/// (skills, qualifications, contract type) that are opaque to this crate.
pub trait Worker: Send + Sync + 'static {
    /// Stable, unique identifier for this worker within the scenario.
    fn id(&self) -> &str;
}

/// The atomic unit of work that is assigned to a [`Worker`].
///
/// In INRC2 this is a shift assignment `(nurse_id, shift_type_id, day_index)`.
/// In airline crew scheduling this is a `Pairing` (base-to-base trip).
///
/// A `PlanningUnit` is indivisible: it is either assigned or not.
pub trait PlanningUnit: Send + Sync + 'static {
    /// Stable, unique identifier for this planning unit within the scenario.
    fn id(&self) -> &str;
}

/// A coverage requirement: how many workers of what kind are needed.
///
/// In INRC2 this is an `InrcRequirement` (shift_type × skill × day →
/// minimum and optimal headcount).  In airline scheduling this is a
/// flight-leg coverage requirement.
pub trait CoverageDemand: Send + Sync + 'static {
    /// Minimum number of workers required to satisfy this demand.
    fn minimum(&self) -> usize;

    /// Optimal (target) number of workers for this demand.
    fn optimal(&self) -> usize;
}

/// A complete assignment of planning units to workers.
///
/// A `PlanningSolution` is the output of a planning run.  It must be
/// cloneable so that the platform can store and compare candidate solutions.
pub trait PlanningSolution: Clone + Send + Sync + 'static {
    type W: Worker;
    type U: PlanningUnit;

    /// Iterate over all `(worker, planning_unit)` assignments in this solution.
    fn assignments(&self) -> impl Iterator<Item = (&Self::W, &Self::U)>;

    /// Return `true` if the solution satisfies all hard constraints.
    fn is_feasible(&self) -> bool;
}

/// The complete description of a planning problem.
///
/// A `PlanningScenario` is the entry point for any domain that participates
/// in the Coralys planning capability.  It provides read-only access to the
/// workers, planning units, and coverage demands that define the problem.
///
/// Implementing this trait for a domain type (e.g. `InrcScenario`) is the
/// primary integration point between a Solution Adapter and this crate.
///
/// # Principle 10 (Domain Fidelity)
///
/// If a faithful domain implementation cannot implement this trait without
/// semantic distortion, the trait contract is wrong — not the domain.
/// Raise an architectural question (OQ-10, …) rather than forcing the domain
/// to misrepresent itself.
pub trait PlanningScenario: Send + Sync + 'static {
    type W: Worker;
    type U: PlanningUnit;
    type D: CoverageDemand;
    type S: PlanningSolution<W = Self::W, U = Self::U>;

    /// Stable identifier for this scenario instance.
    fn id(&self) -> &str;

    /// All workers available for assignment in this scenario.
    fn workers(&self) -> &[Self::W];

    /// All planning units that may be assigned in this scenario.
    fn planning_units(&self) -> &[Self::U];

    /// All coverage demands that the solution must satisfy.
    fn coverage_demands(&self) -> &[Self::D];
}
