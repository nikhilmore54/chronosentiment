//! Neighborhood move generators.
//!
//! Each move is a pure function `&Roster → Option<Roster>`.  Moves do not
//! evaluate objectives or check legality — those responsibilities belong to
//! the search layer.
//!
//! | Module | Move |
//! |--------|------|
//! | [`swap`] | Swap a pairing between two rotations |
//! | [`relocate`] | Move a pairing from one rotation to another |

pub mod relocate;
pub mod swap;
