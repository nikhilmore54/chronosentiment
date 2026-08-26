//! Optimization layer — Layer 4.
//!
//! This module provides multi-objective scheduling optimization built on top
//! of the legality engine (Layer 2) and planner tools (Layer 3).
//!
//! # Module structure
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`objective`] | `SchedulingObjective` trait + concrete objectives |
//! | [`cost`] | `CostVector` (multi-objective) and `CostEvaluator` |
//! | [`neighborhood`] | Move generators (swap, relocate) |
//! | [`search`] | Greedy constructive scheduler + local search |
//! | [`metrics`] | `OptimizationMetrics` (iteration counts, improvements) |
//!
//! # Design principles
//!
//! - **Feasibility oracle**: the Layer 2 [`LegalityChecker`] is the sole
//!   arbiter of feasibility.  The optimizer never embeds legality logic.
//! - **Objective independence**: each objective is a separate struct
//!   implementing [`SchedulingObjective`].  Objectives can be composed
//!   without coupling.
//! - **Neighborhood independence**: each neighborhood move is a pure
//!   function from `Roster → Option<Roster>`.  Moves do not evaluate
//!   objectives or check legality.
//! - **Search independence**: search strategies consume objectives and
//!   neighborhoods as parameters.  Swapping greedy for local search (or
//!   vice versa) requires no changes to objectives or neighborhoods.
//!
//! [`LegalityChecker`]: crate::legality::LegalityChecker

pub mod cost;
pub mod metrics;
pub mod neighborhood;
pub mod objective;
pub mod pairing_generator;
pub mod search;
