//! CVD-001 benchmark adapter for Coralys.
//!
//! Translates CVD-001 benchmark data (GERAD Technical Report G-2014-22,
//! Kasirzadeh, Saddoune, Soumis) into Coralys evaluation concepts.
//!
//! # Architecture
//!
//! This crate is a **Coralys benchmark adapter**. It does not own evaluation
//! abstractions — Coralys does. The adapter translates benchmark-specific data
//! (FlightLeg, Duty, CrewMember) into evaluation results (workloads, constraint
//! violations, objective) that Coralys can consume uniformly across benchmarks.
//!
//! # Mathematical basis
//!
//! Mathematical reconstruction is documented in:
//! - `data/cvd001/WP-M2.1` through `WP-M2.6` (reconstruction work packages)
//! - `data/cvd001/BENCHMARK-SEMANTICS-v1.0.md` (semantic definitions, frozen 781499c4)
//! - `docs/BENCHMARK-REFERENCE-SPECIFICATION-v1.0.md` (this crate's engineering spec)
//!
//! # Public API
//!
//! - [`types`]: domain types (`FlightLeg`, `Duty`, `CrewMember`, `Solution`,
//!              `ConstraintViolation`, `EvaluationResult`)
//! - [`evaluator::evaluate`]: top-level entry point
//! - [`credit::duty_credit`]: single-duty credit (R1 component)
//! - [`workload::credited_workload`]: per-crew-member workload W_n (R1 aggregate)
//! - [`objective::objective`]: benchmark objective Z = Σ_n Δ_n (R2)
//! - [`hc3::hc3_violations`]: HC3 constraint violation collection (R3)
//! - [`hc3::hc3_feasible`]: HC3 feasibility predicate (R3)

pub mod types;
pub mod credit;
pub mod workload;
pub mod objective;
pub mod hc3;
pub mod evaluator;
pub mod framework_adapter;

// Convenience re-exports for the most common use case
pub use types::{FlightLeg, Duty, CrewMember, Solution, ConstraintViolation, EvaluationResult};
pub use evaluator::evaluate;
pub use framework_adapter::Cvd001FrameworkAdapter;