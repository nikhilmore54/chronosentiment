use crate::models::{Shift, Worker};
/// Core trait contracts for the UltraCrew Compliance Framework.
///
/// This module defines the *contracts* only — no infrastructure.
/// Infrastructure (registry, pack loading) lives in [`super::registry`].
///
/// # Contracts defined here
/// - [`ConstraintRule`]  — a single evaluable rule
/// - [`CompliancePack`]  — a named bundle of rules from one source
/// - [`RuleContext`]     — the schedule view passed to each rule
/// - [`RuleOutcome`]     — the result of evaluating one rule
/// - [`Severity`]        — Hard (schedule-invalidating) vs Soft (penalised)
use std::collections::HashMap;

// ── Identifiers ──────────────────────────────────────────────────────────────

/// Stable, dot-namespaced rule identifier.
/// Convention: `<domain>.<category>.<rule>`, e.g. `generic.workforce.minimum_rest`.
pub type RuleId = &'static str;

// ── Severity ─────────────────────────────────────────────────────────────────

/// Severity of a constraint violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Schedule is invalid if any hard rule is violated.
    /// The optimizer must drive hard violations to zero before optimising soft costs.
    Hard,
    /// Violation incurs a fitness penalty but does not invalidate the schedule.
    Soft,
}

// ── Outcome ───────────────────────────────────────────────────────────────────

/// Structured explainability payload attached to every [`RuleOutcome::Violated`].
///
/// Every field is mandatory so that downstream consumers (portal UI, audit logs,
/// dispatcher reports) can always render a complete, actionable explanation.
#[derive(Debug, Clone)]
pub struct ViolationExplanation {
    /// Stable dot-namespaced rule identifier, e.g. `"generic.workforce.minimum_rest"`.
    pub rule_id: &'static str,
    /// Canonical regulatory citation, e.g. `"WTD §6.1"`.
    pub regulatory_ref: &'static str,
    /// Severity of this violation.
    pub severity: Severity,
    /// Human-readable description of what was observed and why it violates the rule.
    pub message: String,
    /// Actionable remediation guidance for the planner or schedule manager.
    pub remediation: String,
}

/// The result of evaluating a single [`ConstraintRule`] for one worker or shift.
#[derive(Debug, Clone)]
pub enum RuleOutcome {
    Satisfied,
    Violated(ViolationExplanation),
}

impl RuleOutcome {
    /// Convenience constructor — builds a `Violated` outcome with all explainability fields.
    pub fn violated(
        rule_id: &'static str,
        regulatory_ref: &'static str,
        severity: Severity,
        message: String,
        remediation: String,
    ) -> Self {
        RuleOutcome::Violated(ViolationExplanation {
            rule_id,
            regulatory_ref,
            severity,
            message,
            remediation,
        })
    }

    pub fn is_satisfied(&self) -> bool {
        matches!(self, RuleOutcome::Satisfied)
    }

    pub fn is_hard_violation(&self) -> bool {
        matches!(
            self,
            RuleOutcome::Violated(ViolationExplanation {
                severity: Severity::Hard,
                ..
            })
        )
    }

    pub fn is_soft_violation(&self) -> bool {
        matches!(
            self,
            RuleOutcome::Violated(ViolationExplanation {
                severity: Severity::Soft,
                ..
            })
        )
    }

    /// Extract the explanation if this is a violation; `None` if satisfied.
    pub fn explanation(&self) -> Option<&ViolationExplanation> {
        match self {
            RuleOutcome::Violated(e) => Some(e),
            RuleOutcome::Satisfied => None,
        }
    }
}

// ── Context ──────────────────────────────────────────────────────────────────

/// Thin, read-only view of the schedule passed to each rule during evaluation.
///
/// `worker_shifts` maps worker id → the shifts assigned to that worker,
/// sorted ascending by `start_hour`.  Rules must not reach into GA internals.
pub struct RuleContext<'a> {
    pub workers: &'a [Worker],
    pub all_shifts: &'a [Shift],
    /// worker_id → sorted list of shifts assigned to that worker
    pub worker_shifts: HashMap<u64, Vec<&'a Shift>>,
}

impl<'a> RuleContext<'a> {
    /// Build a `RuleContext` from a flat assignment map `shift_id → worker_id`.
    pub fn from_assignments(
        workers: &'a [Worker],
        all_shifts: &'a [Shift],
        assignments: &HashMap<u64, u64>,
    ) -> Self {
        let mut worker_shifts: HashMap<u64, Vec<&'a Shift>> = HashMap::new();
        for shift in all_shifts {
            if let Some(&worker_id) = assignments.get(&shift.id) {
                worker_shifts.entry(worker_id).or_default().push(shift);
            }
        }
        // Sort each worker's shifts by start_hour for sequential analysis.
        for shifts in worker_shifts.values_mut() {
            shifts.sort_by_key(|s| s.start_hour);
        }
        RuleContext {
            workers,
            all_shifts,
            worker_shifts,
        }
    }
}

// ── ConstraintRule ────────────────────────────────────────────────────────────

/// A single schedulable constraint or policy rule.
///
/// Implementors live in `regulatory/`, `company/`, `agreements/`, or
/// `optimization/`.  The optimizer never imports a concrete implementor.
pub trait ConstraintRule: Send + Sync {
    /// Stable identifier, e.g. `"generic.workforce.minimum_rest"`.
    fn id(&self) -> RuleId;

    /// Human-readable description of what this rule enforces.
    fn description(&self) -> &str;

    /// Severity of a violation.
    fn severity(&self) -> Severity;

    /// Evaluate the rule against the full schedule context.
    /// Returns one outcome per worker (or per shift) that violates the rule.
    /// An empty `Vec` means the rule is fully satisfied.
    fn evaluate(&self, ctx: &RuleContext<'_>) -> Vec<RuleOutcome>;
}

// ── CompliancePack ────────────────────────────────────────────────────────────

/// A named collection of rules from one regulatory, company, agreement, or
/// optimization source.
///
/// The name `CompliancePack` is intentionally broader than `RulePack` because
/// not everything loaded is a "rule" — optimization objectives are policies,
/// not regulations.
///
/// Implementors call `registry.register(...)` for each rule they own inside
/// `load_into()`.  The optimizer calls `pack.load_into(registry)` once at startup.
///
/// # Naming convention
/// Pack structs should be named `<Authority>CompliancePack`, e.g.:
/// `EuwtdCompliancePack`, `AcmeCorpCompanyPack`.
pub trait CompliancePack {
    /// Provenance descriptor for this pack.
    /// Used by the registry to report which packs are active and their versions.
    fn descriptor(&self) -> crate::compliance::metadata::ComplianceDescriptor;

    /// Load all rules in this pack into the registry.
    fn load_into(&self, registry: &mut crate::compliance::registry::ComplianceRegistry);
}
