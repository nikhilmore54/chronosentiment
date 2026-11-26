/// Company policy packs — operator-specific internal scheduling rules.
///
/// These layer on top of regulatory packs and encode policies such as
/// minimum crew rest buffers beyond regulatory minimums, preferred base
/// assignments, seniority-based bidding constraints, and internal KPIs.
///
/// # Planned packs
/// - `acme_corp`  — Acme Corp Enterprise Agreement
/// - `globex`     — Globex Corporation policies
///
/// # Adding a new company pack
/// 1. Create `company/<company>/mod.rs` implementing [`CompliancePack`].
/// 2. Declare the sub-module below.
/// 3. Load the pack alongside the regulatory pack at optimizer startup.
///
/// [`CompliancePack`]: crate::compliance::traits::CompliancePack

// pub mod acme_corp;
// pub mod globex;

/// Marker — no company packs implemented yet.
pub struct NoCompanyPacksYet;
