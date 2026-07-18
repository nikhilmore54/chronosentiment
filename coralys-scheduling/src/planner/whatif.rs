//! What-if analysis.
//!
//! [`WhatIfAnalysis`] compares the legality violations of two roster states —
//! a *baseline* and a *candidate* — and identifies:
//!
//! - **Introduced violations**: present in the candidate but not the baseline.
//! - **Resolved violations**: present in the baseline but not the candidate.
//! - **Persisting violations**: present in both.
//!
//! This allows planners to evaluate the impact of a proposed change before
//! committing it, without running a full re-evaluation manually.
//!
//! # Example
//!
//! ```rust,ignore
//! let baseline_violations = checker.check(&original_roster);
//! let candidate_violations = checker.check(&modified_roster);
//! let analysis = WhatIfAnalysis::compare(baseline_violations, candidate_violations);
//! println!("Introduced: {}", analysis.introduced_count());
//! println!("Resolved:   {}", analysis.resolved_count());
//! ```
//!
//! # Matching semantics
//!
//! Two violations are considered the **same** if they share the same
//! `rule_id`, `entity` (as a string), `observed` (within 0.001), and
//! `threshold` (within 0.001).  This is intentionally loose to handle
//! floating-point representation differences across roster states.

use crate::legality::LegalityViolation;

// ── Violation identity ────────────────────────────────────────────────────────

/// A stable key used to match violations across roster states.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ViolationKey {
    rule_id: String,
    entity: String,
    /// Observed value rounded to 3 decimal places for stable comparison.
    observed_rounded: i64,
    /// Threshold rounded to 3 decimal places for stable comparison.
    threshold_rounded: i64,
}

impl ViolationKey {
    fn from(v: &LegalityViolation) -> Self {
        Self {
            rule_id: v.rule_id.clone(),
            entity: format!("{}", v.entity),
            observed_rounded: (v.observed * 1000.0).round() as i64,
            threshold_rounded: (v.threshold * 1000.0).round() as i64,
        }
    }
}

// ── Analysis ──────────────────────────────────────────────────────────────────

/// The result of comparing two roster states' violations.
///
/// Constructed via [`WhatIfAnalysis::compare`].
#[derive(Debug, Clone)]
pub struct WhatIfAnalysis {
    /// Violations present in the candidate but not the baseline.
    introduced: Vec<LegalityViolation>,
    /// Violations present in the baseline but not the candidate.
    resolved: Vec<LegalityViolation>,
    /// Violations present in both baseline and candidate.
    persisting: Vec<LegalityViolation>,
}

impl WhatIfAnalysis {
    /// Compare baseline and candidate violation sets.
    ///
    /// `baseline` — violations from the original roster.
    /// `candidate` — violations from the proposed modified roster.
    pub fn compare(
        baseline: Vec<LegalityViolation>,
        candidate: Vec<LegalityViolation>,
    ) -> Self {
        use std::collections::HashMap;

        // Index baseline violations by key, counting occurrences.
        let mut baseline_counts: HashMap<ViolationKey, usize> = HashMap::new();
        let mut baseline_by_key: HashMap<ViolationKey, Vec<LegalityViolation>> = HashMap::new();
        for v in baseline {
            let key = ViolationKey::from(&v);
            *baseline_counts.entry(key.clone()).or_insert(0) += 1;
            baseline_by_key.entry(key).or_default().push(v);
        }

        // Index candidate violations by key.
        let mut candidate_counts: HashMap<ViolationKey, usize> = HashMap::new();
        let mut candidate_by_key: HashMap<ViolationKey, Vec<LegalityViolation>> = HashMap::new();
        for v in candidate {
            let key = ViolationKey::from(&v);
            *candidate_counts.entry(key.clone()).or_insert(0) += 1;
            candidate_by_key.entry(key).or_default().push(v);
        }

        let mut introduced = Vec::new();
        let mut resolved = Vec::new();
        let mut persisting = Vec::new();

        // Collect all keys from both sets.
        let mut all_keys: std::collections::HashSet<ViolationKey> = std::collections::HashSet::new();
        all_keys.extend(baseline_counts.keys().cloned());
        all_keys.extend(candidate_counts.keys().cloned());

        for key in all_keys {
            let b_count = baseline_counts.get(&key).copied().unwrap_or(0);
            let c_count = candidate_counts.get(&key).copied().unwrap_or(0);

            if b_count == 0 {
                // All candidate occurrences are new.
                if let Some(vs) = candidate_by_key.remove(&key) {
                    introduced.extend(vs);
                }
            } else if c_count == 0 {
                // All baseline occurrences are resolved.
                if let Some(vs) = baseline_by_key.remove(&key) {
                    resolved.extend(vs);
                }
            } else {
                // Some persist, some may be introduced or resolved.
                let persist_count = b_count.min(c_count);
                let extra_candidate = c_count.saturating_sub(b_count);
                let extra_baseline = b_count.saturating_sub(c_count);

                if let Some(vs) = candidate_by_key.remove(&key) {
                    let mut iter = vs.into_iter();
                    for _ in 0..persist_count {
                        if let Some(v) = iter.next() {
                            persisting.push(v);
                        }
                    }
                    for v in iter.take(extra_candidate) {
                        introduced.push(v);
                    }
                }
                if let Some(vs) = baseline_by_key.remove(&key) {
                    for v in vs.into_iter().take(extra_baseline) {
                        resolved.push(v);
                    }
                }
            }
        }

        Self { introduced, resolved, persisting }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    /// Violations newly introduced by the candidate change.
    pub fn introduced(&self) -> &[LegalityViolation] {
        &self.introduced
    }

    /// Violations resolved by the candidate change.
    pub fn resolved(&self) -> &[LegalityViolation] {
        &self.resolved
    }

    /// Violations that persist in both baseline and candidate.
    pub fn persisting(&self) -> &[LegalityViolation] {
        &self.persisting
    }

    /// Number of newly introduced violations.
    pub fn introduced_count(&self) -> usize {
        self.introduced.len()
    }

    /// Number of resolved violations.
    pub fn resolved_count(&self) -> usize {
        self.resolved.len()
    }

    /// Number of persisting violations.
    pub fn persisting_count(&self) -> usize {
        self.persisting.len()
    }

    /// Returns `true` if the candidate introduces no new violations.
    pub fn is_improvement_or_neutral(&self) -> bool {
        self.introduced.is_empty()
    }

    /// Net change in violation count: negative means improvement.
    pub fn net_change(&self) -> i64 {
        self.introduced_count() as i64 - self.resolved_count() as i64
    }

    /// Returns `true` if the candidate is strictly better (fewer violations,
    /// none introduced).
    pub fn is_strict_improvement(&self) -> bool {
        self.introduced.is_empty() && !self.resolved.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legality::{EntityRef, LegalityViolation};

    fn err(rule: &str, entity: &str, obs: f64, thr: f64) -> LegalityViolation {
        LegalityViolation::error(rule, EntityRef::Duty(entity.into()), obs, thr, "")
    }

    // ── No change ─────────────────────────────────────────────────────────────

    #[test]
    fn identical_violations_all_persist() {
        let v = err("max_duty_time", "D1", 900.0, 840.0);
        let analysis = WhatIfAnalysis::compare(vec![v.clone()], vec![v]);
        assert_eq!(analysis.persisting_count(), 1);
        assert_eq!(analysis.introduced_count(), 0);
        assert_eq!(analysis.resolved_count(), 0);
        assert_eq!(analysis.net_change(), 0);
    }

    // ── All resolved ──────────────────────────────────────────────────────────

    #[test]
    fn all_violations_resolved() {
        let v = err("max_duty_time", "D1", 900.0, 840.0);
        let analysis = WhatIfAnalysis::compare(vec![v], vec![]);
        assert_eq!(analysis.resolved_count(), 1);
        assert_eq!(analysis.introduced_count(), 0);
        assert_eq!(analysis.persisting_count(), 0);
        assert!(analysis.is_strict_improvement());
        assert_eq!(analysis.net_change(), -1);
    }

    // ── All introduced ────────────────────────────────────────────────────────

    #[test]
    fn all_violations_introduced() {
        let v = err("max_duty_time", "D1", 900.0, 840.0);
        let analysis = WhatIfAnalysis::compare(vec![], vec![v]);
        assert_eq!(analysis.introduced_count(), 1);
        assert_eq!(analysis.resolved_count(), 0);
        assert_eq!(analysis.persisting_count(), 0);
        assert!(!analysis.is_improvement_or_neutral());
        assert_eq!(analysis.net_change(), 1);
    }

    // ── Mixed: some resolved, some introduced, some persist ───────────────────

    #[test]
    fn mixed_change() {
        let v1 = err("max_duty_time", "D1", 900.0, 840.0); // baseline only → resolved
        let v2 = err("minimum_rest", "P1", 300.0, 600.0);  // both → persists
        let v3 = err("coverage", "L1", 0.0, 1.0);          // candidate only → introduced

        let baseline = vec![v1, v2.clone()];
        let candidate = vec![v2, v3];
        let analysis = WhatIfAnalysis::compare(baseline, candidate);

        assert_eq!(analysis.resolved_count(), 1);
        assert_eq!(analysis.persisting_count(), 1);
        assert_eq!(analysis.introduced_count(), 1);
        assert_eq!(analysis.net_change(), 0);
        assert!(!analysis.is_strict_improvement());
        assert!(!analysis.is_improvement_or_neutral());
    }

    // ── Empty baseline and candidate ──────────────────────────────────────────

    #[test]
    fn both_empty_no_change() {
        let analysis = WhatIfAnalysis::compare(vec![], vec![]);
        assert_eq!(analysis.total_violations(), 0);
        assert!(analysis.is_improvement_or_neutral());
    }

    // ── Neutral: same violations, different order ─────────────────────────────

    #[test]
    fn same_violations_different_order_all_persist() {
        let v1 = err("r1", "D1", 10.0, 5.0);
        let v2 = err("r2", "D2", 20.0, 10.0);
        let analysis = WhatIfAnalysis::compare(vec![v1.clone(), v2.clone()], vec![v2, v1]);
        assert_eq!(analysis.persisting_count(), 2);
        assert_eq!(analysis.introduced_count(), 0);
        assert_eq!(analysis.resolved_count(), 0);
    }

    // ── Improvement or neutral ────────────────────────────────────────────────

    #[test]
    fn no_new_violations_is_improvement_or_neutral() {
        let v = err("r", "D1", 10.0, 5.0);
        let analysis = WhatIfAnalysis::compare(vec![v], vec![]);
        assert!(analysis.is_improvement_or_neutral());
    }
}

impl WhatIfAnalysis {
    /// Total violations in the candidate (introduced + persisting).
    pub fn total_violations(&self) -> usize {
        self.introduced.len() + self.persisting.len()
    }
}