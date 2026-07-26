//! Planner productivity layer — Layer 3.
//!
//! This module provides tools that help planners interact with the legality
//! engine efficiently during schedule construction and adjustment.
//!
//! # Capabilities
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`summary`] | Aggregate violations by rule, severity, entity, or crew member |
//! | [`whatif`] | Compare two roster states; identify new / resolved violations |
//! | [`incremental`] | Re-evaluate only the affected rotation after an edit |
//!
//! # Design principles
//!
//! - **Read-only**: no module in this layer modifies a roster.
//! - **Composable**: each capability is independent and can be used alone.
//! - **Fast feedback**: incremental and what-if analysis avoid re-running all
//!   rules against the entire roster when only a small change has been made.

pub mod incremental;
pub mod summary;
pub mod whatif;