/// Collective agreement packs — union and labour agreement constraints.
///
/// These encode constraints negotiated in collective bargaining agreements
/// (CBAs) that go beyond regulatory minimums: guaranteed days off, seniority
/// bidding windows, minimum monthly hours guarantees, etc.
///
/// # Planned packs
/// - `alpa`        — ALPA (Air Line Pilots Association) CBA provisions
/// - `ifalpa`      — IFALPA reference provisions
/// - `cabin_crew`  — Generic cabin crew CBA template
///
/// # Adding a new agreement pack
/// 1. Create `agreements/<union>/mod.rs` implementing [`CompliancePack`].
/// 2. Declare the sub-module below.
/// 3. Load the pack alongside regulatory and company packs at startup.
///
/// [`CompliancePack`]: crate::compliance::traits::CompliancePack

// pub mod alpa;
// pub mod ifalpa;
// pub mod cabin_crew;

/// Marker — no agreement packs implemented yet.
pub struct NoAgreementPacksYet;