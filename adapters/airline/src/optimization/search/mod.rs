//! Search strategies.
//!
//! | Module | Strategy |
//! |--------|---------|
//! | [`greedy`] | Greedy constructive scheduler — assigns pairings to rotations in cost order |
//! | [`local_search`] | Hill-climbing local search using Layer 2 as feasibility oracle |

pub mod greedy;
pub mod local_search;