/// Compliance Registry — infrastructure for composing compliance packs.
///
/// The registry is the runtime container that holds all active
/// [`ConstraintRule`] implementations for a scheduling run.  It is
/// intentionally separate from [`super::traits`] because:
///
/// - `traits.rs` defines *contracts* (what a rule must do).
/// - `registry.rs` defines *infrastructure* (how packs are composed and rules are run).
///
/// # Responsibilities
/// - Register [`CompliancePack`]s and flatten their rules into the active rule set.
/// - Report which packs are installed (for audit logs and explainability).
/// - Evaluate all rules and return outcomes.
///
/// # Typical startup sequence
///
/// ```rust
/// use ultracrew::compliance::{ComplianceRegistry, DgcaCompliancePack, CompliancePack};
///
/// let mut registry = ComplianceRegistry::new();
/// registry.install(DgcaCompliancePack::default());
/// // registry.install(IndiGoCompanyPack::new());
/// // registry.install(FairnessOptimizationPack::new());
/// ```

use crate::compliance::traits::{
    ConstraintRule, CompliancePack, RuleId, RuleOutcome, RuleContext,
    Severity, ViolationExplanation,
};
use crate::compliance::metadata::ComplianceDescriptor;

/// Holds all registered constraint rules and tracks installed pack descriptors.
pub struct ComplianceRegistry {
    /// Flattened list of all active rules (loaded from installed packs).
    rules: Vec<Box<dyn ConstraintRule>>,
    /// Descriptors of all installed packs, for audit and reporting.
    installed: Vec<ComplianceDescriptor>,
}

impl ComplianceRegistry {
    pub fn new() -> Self {
        ComplianceRegistry {
            rules: Vec::new(),
            installed: Vec::new(),
        }
    }

    /// Install a compliance pack: record its descriptor and load its rules.
    ///
    /// Hard-constraint packs should be installed before soft/optimization packs
    /// so that hard violations appear first in diagnostic output.
    pub fn install(&mut self, pack: impl CompliancePack) {
        self.installed.push(pack.descriptor());
        pack.load_into(self);
    }

    /// Register a single rule directly (called by `CompliancePack::load_into`).
    pub fn register(&mut self, rule: impl ConstraintRule + 'static) {
        self.rules.push(Box::new(rule));
    }

    /// Number of active rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Number of installed packs.
    pub fn pack_count(&self) -> usize {
        self.installed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Descriptors of all installed packs, in installation order.
    pub fn installed_packs(&self) -> &[ComplianceDescriptor] {
        &self.installed
    }

    /// Evaluate every registered rule and return a flat list of all outcomes.
    /// Each entry is `(rule_id, outcome)`.
    pub fn evaluate_all(&self, ctx: &RuleContext<'_>) -> Vec<(RuleId, RuleOutcome)> {
        let mut results = Vec::new();
        for rule in &self.rules {
            for outcome in rule.evaluate(ctx) {
                results.push((rule.id(), outcome));
            }
        }
        results
    }

    /// Count hard violations across all rules.
    pub fn hard_violation_count(&self, ctx: &RuleContext<'_>) -> usize {
        self.evaluate_all(ctx)
            .iter()
            .filter(|(_, o)| o.is_hard_violation())
            .count()
    }

    /// Total fitness penalty contribution from soft violations.
    /// Each soft violation contributes `penalty_per_violation`.
    pub fn soft_penalty(&self, ctx: &RuleContext<'_>, penalty_per_violation: f64) -> f64 {
        self.evaluate_all(ctx)
            .iter()
            .filter(|(_, o)| o.is_soft_violation())
            .count() as f64
            * penalty_per_violation
    }

    /// Collect all violation messages for diagnostic output.
    ///
    /// Each message is formatted as:
    /// `[HARD|SOFT] <rule_id> (ref: <regulatory_ref>) — <message> | Remediation: <remediation>`
    pub fn violation_messages(&self, ctx: &RuleContext<'_>) -> Vec<String> {
        self.evaluate_all(ctx)
            .into_iter()
            .filter_map(|(_id, outcome)| match outcome {
                RuleOutcome::Violated(e) => {
                    let tag = match e.severity {
                        Severity::Hard => "HARD",
                        Severity::Soft => "SOFT",
                    };
                    Some(format!(
                        "[{}] {} (ref: {}) — {} | Remediation: {}",
                        tag, e.rule_id, e.regulatory_ref, e.message, e.remediation,
                    ))
                }
                RuleOutcome::Satisfied => None,
            })
            .collect()
    }

    /// Return all violation explanations as structured data.
    ///
    /// Unlike [`violation_messages`] which returns formatted strings, this method
    /// returns the full [`ViolationExplanation`] structs so callers can render
    /// them in any format (JSON API, portal UI, audit log, etc.).
    pub fn violation_explanations(&self, ctx: &RuleContext<'_>) -> Vec<ViolationExplanation> {
        self.evaluate_all(ctx)
            .into_iter()
            .filter_map(|(_, outcome)| outcome.explanation().cloned())
            .collect()
    }

    /// One-line summary of installed packs for logging.
    pub fn installed_packs_summary(&self) -> String {
        if self.installed.is_empty() {
            return "No compliance packs installed".to_string();
        }
        self.installed
            .iter()
            .map(|d| format!("{} v{}", d.id, d.version))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for ComplianceRegistry {
    fn default() -> Self {
        Self::new()
    }
}