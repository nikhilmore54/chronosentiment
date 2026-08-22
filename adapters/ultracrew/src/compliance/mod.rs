/// UltraCrew Compliance Framework
///
/// The Compliance Framework is the single extension point for all scheduling
/// rules and policies.  The optimizer (`constraint_engine`) is
/// jurisdiction-agnostic: it knows only about [`traits::ConstraintRule`],
/// [`registry::ComplianceRegistry`], and [`traits::CompliancePack`].
///
/// Every rule — whether from a regional labor authority, an organization's internal
/// union agreement, or an optimization objective — implements `ConstraintRule`.
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
///   regulatory/        — Regional labor authority packs
///     eu_wtd/          — EU Working Time Directive          ← planned
///     osha/            — OSHA (USA)                         ← planned
///
///   company/           — Corporate rules or localized agreements
///     acme_corp/       — Acme Corp Enterprise Agreement      ← planned
///
///   agreements/        — General union or collective bargaining agreements
///     general_union/   — General Workers Union contract      ← planned
///   optimization/      — Soft objectives and preference policies
///     fairness/        — Hour-variance minimization          ← planned
///     fatigue/         — Fatigue score minimization          ← planned
///     disruption/      — Disruption-resilient scheduling     ← planned
///     robustness/      — Schedule robustness scoring         ← planned
/// ```
///
/// # Usage example — INRC
///
/// ```rust
/// use ultracrew::compliance::{ComplianceRegistry, CompliancePack};
///
/// let mut registry = ComplianceRegistry::new();
/// // InrcRulePack::default().load_into(&mut registry);
/// ```
///
/// # Cross-industry applicability
///
/// The same framework applies to any workforce scheduling domain.
/// Only the compliance packs change; the optimizer and constraint engine are identical.
///
/// Hospitals     → regulatory/ (Nursing Council packs)
/// Manufacturing → regulatory/ (Factory Act packs)
/// Retail        → regulatory/ (Labor Law packs)
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
