/// Company policy packs — operator-specific internal scheduling rules.
///
/// These layer on top of regulatory packs and encode policies such as
/// minimum crew rest buffers beyond regulatory minimums, preferred base
/// assignments, seniority-based bidding constraints, and internal KPIs.
///
/// # Planned packs
/// - `indigo`     — IndiGo internal scheduling policies
/// - `air_india`  — Air India internal policies
/// - `ryanair`    — Ryanair internal policies
/// - `delta`      — Delta Air Lines internal policies
///
/// # Adding a new company pack
/// 1. Create `company/<airline>/mod.rs` implementing [`CompliancePack`].
/// 2. Declare the sub-module below.
/// 3. Load the pack alongside the regulatory pack at optimizer startup.
///
/// [`CompliancePack`]: crate::compliance::traits::CompliancePack

// pub mod indigo;
// pub mod air_india;
// pub mod ryanair;
// pub mod delta;

/// Marker — no company packs implemented yet.
pub struct NoCompanyPacksYet;