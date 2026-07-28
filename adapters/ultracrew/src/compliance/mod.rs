/// UltraCrew Compliance Framework
///
/// The Compliance Framework is the single extension point for all scheduling
/// rules and policies.  The optimizer (`constraint_engine`) is
/// jurisdiction-agnostic: it knows only about [`traits::ConstraintRule`],
/// [`registry::ComplianceRegistry`], and [`traits::CompliancePack`].
///
/// Every rule — whether from a civil aviation authority, an airline's internal
/// policy, a union agreement, or an optimization objective — is loaded into
/// the registry at startup via `pack.load_into(&mut registry)`.
///
/// # Five architectural principles
///
/// 1. **Constraint Engine** — generic evaluation engine (unchanged per jurisdiction).
/// 2. **Compliance Framework** — owns regulatory, company, agreement, and optimization packs.
/// 3. **Compliance Registry** — composes packs into the active rule set at runtime.
/// 4. **Compliance Metadata** — identifies authority, version, and provenance of every pack.
/// 5. **Optimizer** — consumes only evaluated constraints; never imports compliance modules directly.
///
/// # Module hierarchy
///
/// ```text
/// compliance/
///   traits.rs          — ConstraintRule, CompliancePack, RuleContext, RuleOutcome, Severity
///   registry.rs        — ComplianceRegistry (infrastructure, separate from contracts)
///   metadata.rs        — ComplianceDescriptor (reserved for future provenance tracking)
///
///   regulatory/        — Civil aviation authority packs
///     dgca/            — DGCA (India) CAR S7 J III          ← implemented
///     easa/            — EASA ORO.FTL (Europe)               ← planned
///     faa_part117/     — FAA Part 117 (USA)                  ← planned
///     transport_canada/— Transport Canada CARs Part VII      ← planned (separate from FAA)
///     casa/            — CASA CAO 48.1 (Australia)           ← planned
///     uk_caa/          — UK CAA CAP 1616                     ← planned
///     gaca/            — GACA (Saudi Arabia)                 ← planned
///     gcaa/            — GCAA (UAE)                          ← planned
///     caas/            — CAAS (Singapore)                    ← planned
///     caac/            — CAAC (China)                        ← planned
///     icao_reference/  — ICAO Annex 6 baseline               ← planned
///
///   company/           — Operator-specific internal policies
///     indigo/          — IndiGo                              ← planned
///     air_india/       — Air India                           ← planned
///     ryanair/         — Ryanair                             ← planned
///     delta/           — Delta Air Lines                     ← planned
///
///   agreements/        — Union and collective bargaining agreements
///     alpa/            — ALPA CBA                            ← planned
///     ifalpa/          — IFALPA reference provisions         ← planned
///     cabin_crew/      — Generic cabin crew CBA template     ← planned
///
///   optimization/      — Soft objectives and preference policies
///     fairness/        — Hour-variance minimization          ← planned
///     fatigue/         — Fatigue score minimization          ← planned
///     disruption/      — Disruption-resilient scheduling     ← planned
///     robustness/      — Schedule robustness scoring         ← planned
/// ```
///
/// # Usage example — Indian airline
///
/// ```rust
/// use ultracrew::compliance::{ComplianceRegistry, DgcaRulePack, CompliancePack};
///
/// let mut registry = ComplianceRegistry::new();
/// DgcaRulePack::default().load_into(&mut registry);
/// // IndiGoCompanyPack::new().load_into(&mut registry);   // when implemented
/// // FairnessOptimizationPack::new().load_into(&mut registry);
/// ```
///
/// # Cross-industry applicability
///
/// The same framework applies to any workforce scheduling domain.
/// Only the compliance packs change; the optimizer and constraint engine are identical.
///
/// ```text
/// Airlines      → regulatory/ (DGCA, FAA, EASA, Transport Canada, CASA …)
/// Hospitals     → regulatory/ (Nursing Council packs)
/// Railways      → regulatory/ (Railway Labour packs)
/// Manufacturing → regulatory/ (Factory Act packs)
/// Mining        → regulatory/ (Mine Safety packs)
/// ```

pub mod traits;
pub mod registry;
pub mod metadata;
pub mod regulatory;
pub mod company;
pub mod agreements;
pub mod optimization;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use traits::{
    ConstraintRule,
    CompliancePack,
    RuleId,
    RuleOutcome,
    RuleContext,
    Severity,
    ViolationExplanation,
};
pub use registry::ComplianceRegistry;
pub use metadata::ComplianceDescriptor;
pub use regulatory::DgcaCompliancePack;
// Backward-compatible alias
pub use regulatory::DgcaRulePack;
#[cfg(test)]
mod tests;
