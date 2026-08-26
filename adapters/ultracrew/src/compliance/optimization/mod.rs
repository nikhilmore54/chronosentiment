/// Optimization policy packs — soft objectives and preference policies.
///
/// Unlike regulatory and company packs (which are mostly hard constraints),
/// optimization packs contribute soft penalties and rewards that shape the
/// objective function: fairness, fatigue minimization, disruption recovery
/// preference, schedule robustness, etc.
///
/// These are loaded last so their soft penalties are layered on top of the
/// hard constraint landscape established by regulatory and company packs.
///
/// # Planned packs
/// - `fairness`    — Minimize variance in hours across crew members
/// - `fatigue`     — Penalise schedules that accumulate high fatigue scores
/// - `disruption`  — Prefer assignments that minimize cascading disruption
/// - `robustness`  — Reward schedules with buffer time and swap flexibility
///
/// # Adding a new optimization pack
/// 1. Create `optimization/<policy>/mod.rs` implementing [`CompliancePack`].
/// 2. Declare the sub-module below.
/// 3. Load the pack after all hard-constraint packs at optimizer startup.
///
/// [`CompliancePack`]: crate::compliance::traits::CompliancePack

// pub mod fairness;
// pub mod fatigue;
// pub mod disruption;
// pub mod robustness;

/// Marker — no optimization packs implemented yet.
pub struct NoOptimizationPacksYet;
