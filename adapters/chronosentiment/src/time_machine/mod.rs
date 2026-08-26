//! TIME Machine — historical pipeline replay infrastructure.
//!
//! This module provides the building blocks for replaying the frozen
//! LIVE-001 → LIVE-005 pipeline at any historical date `T`, producing
//! a statistically meaningful evidence dataset without waiting for
//! months of real-time observations.
//!
//! # Modules
//!
//! - [`clock`] — TIME-001: `HistoricalClock` abstraction that replaces
//!   `Utc::now()` with a deterministic point-in-time timestamp.

pub mod clock;

pub use clock::{ClockMode, HistoricalClock};
