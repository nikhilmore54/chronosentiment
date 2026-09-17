//! Intraday classification — Intelligence Contract v1.
//!
//! Implements the frozen product intelligence model from nine iterations of
//! discovery on the 459-decision historical dataset (Aug 20 – Sep 7).
//!
//! See `docs/INTELLIGENCE_CONTRACT_V1.md` for the full specification.
//!
//! # Module structure
//!
//! - [`classification`] — entry-time classifier (T0 information only)
//! - [`state_machine`] — state vocabulary and H120 reassessment
//! - [`action`] — action vocabulary and state → product action mapping
//!
//! # Quick start
//!
//! ```rust,ignore
//! use intraday_classification::{
//!     classification::{classify_at_entry, Direction, EntryInput, H60Classification},
//!     state_machine::{reassess_at_h120, H120Input},
//!     action::{resolve_action, Checkpoint},
//! };
//!
//! // 1. Classify at entry (T0 information only)
//! let entry_input = EntryInput {
//!     direction: Direction::Long,
//!     opportunity_quality_score: 55,
//!     h60_classification: H60Classification::Wait,
//! };
//! let entry_state = classify_at_entry(&entry_input);
//! // → WaitMid
//!
//! // 2. Resolve product action at entry
//! let entry_action = resolve_action(Checkpoint::Entry, entry_state, None);
//! // → Action::Monitor, Confidence::Moderate
//!
//! // 3. Reassess at H120 (LONG WAIT-MID only)
//! let h120 = H120Input { h120_ret: Some(0.005), mfe_h120: Some(0.006) };
//! let h120_state = reassess_at_h120(entry_input.direction, entry_state, &h120);
//! // → EnterLate
//!
//! // 4. Resolve product action at H120
//! let h120_action = resolve_action(Checkpoint::H120, h120_state, Some(entry_state));
//! // → Action::Upgrade, Confidence::High
//! ```

pub mod action;
pub mod classification;
pub mod state_machine;

// Re-export the most commonly used types for convenience.
pub use action::{Action, Checkpoint, Confidence, TradingAction, resolve_action};
pub use classification::{
    Direction, EntryInput, H60Classification, MFE_FLOOR, FAV_ADV_THRESHOLD,
    OQS_LONG_HIGH, OQS_LONG_MID, OQS_SHORT_HIGH, ReturnBucket, bucket, classify_at_entry,
};
pub use state_machine::{H120Input, OpportunityState, reassess_at_h120};