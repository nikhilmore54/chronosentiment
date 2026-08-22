use crate::domain::Roster;
use crate::legality::{LegalityChecker, LegalityViolation};
use serde::{Deserialize, Serialize};

/// Summary of legality violations for a specific rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleSummary {
    /// The canonical identifier of the rule.
    pub rule_id: String,
    /// All violations produced by this rule (empty if none).
    pub violations: Vec<LegalityViolation>,
}

/// A structured report of a roster's legality evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegalityReport {
    /// True if the roster has zero Error-severity violations across all rules.
    pub legal: bool,
    /// A summary of violations grouped by rule, deterministic in order of rule registration.
    pub rules: Vec<RuleSummary>,
}

/// Evaluate a roster against a configured `LegalityChecker` and produce a deterministic report.
pub fn evaluate_roster(roster: &Roster, checker: &LegalityChecker) -> LegalityReport {
    let mut rules = Vec::with_capacity(checker.rule_count());
    let mut legal = true;

    // Pre-populate summaries to guarantee all configured rules are present
    // in deterministic registration order, even if they have zero violations.
    for rule_id in checker.rule_ids() {
        rules.push(RuleSummary {
            rule_id: rule_id.to_string(),
            violations: Vec::new(),
        });
    }

    // Run the checker once to get all violations
    let all_violations = checker.check(roster);

    // Group violations into the pre-populated summaries
    for violation in all_violations {
        if violation.is_error() {
            legal = false;
        }
        
        // Find the corresponding rule summary
        if let Some(summary) = rules.iter_mut().find(|r| r.rule_id == violation.rule_id) {
            summary.violations.push(violation);
        }
    }

    LegalityReport { legal, rules }
}

use std::collections::{HashMap, HashSet};

/// Difference between two legality evaluations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegalityDelta {
    /// Rules that were illegal in the baseline and are now legal (no violations).
    pub became_legal: Vec<String>,
    /// Rules that were legal in the baseline and are now illegal (have violations).
    pub became_illegal: Vec<String>,
    /// Violations present in the candidate report that were not in the baseline.
    pub new_violations: Vec<LegalityViolation>,
    /// Violations present in the baseline report that are no longer present in the candidate.
    pub resolved_violations: Vec<LegalityViolation>,
}

/// Compare two rosters using the same `LegalityChecker` and produce a delta report.
///
/// Returns a `LegalityDelta` capturing which rules changed legality status and the
/// specific violations that were introduced or resolved.
pub fn compare_rosters(
    baseline: &Roster,
    candidate: &Roster,
    checker: &LegalityChecker,
) -> LegalityDelta {
    // Generate deterministic reports for both rosters.
    let base_report = evaluate_roster(baseline, checker);
    let cand_report = evaluate_roster(candidate, checker);

    // Helper maps from rule_id to its violations for quick lookup.
    let mut base_map: HashMap<String, Vec<LegalityViolation>> = HashMap::new();
    for r in base_report.rules.iter() {
        base_map.insert(r.rule_id.clone(), r.violations.clone());
    }
    let mut cand_map: HashMap<String, Vec<LegalityViolation>> = HashMap::new();
    for r in cand_report.rules.iter() {
        cand_map.insert(r.rule_id.clone(), r.violations.clone());
    }

    let mut became_legal = Vec::new();
    let mut became_illegal = Vec::new();
    let mut new_violations = Vec::new();
    let mut resolved_violations = Vec::new();

    // Union of all rule ids to guarantee deterministic ordering (sorted).
    let mut all_rule_ids: Vec<String> = base_map
        .keys()
        .chain(cand_map.keys())
        .cloned()
        .collect();
    all_rule_ids.sort();
    all_rule_ids.dedup();

    for rule_id in all_rule_ids {
                // Retrieve violations, defaulting to an empty slice to avoid temporary borrow issues.
        let base_viol: &[LegalityViolation] = base_map.get(&rule_id).map(|v| v.as_slice()).unwrap_or(&[]);
        let cand_viol: &[LegalityViolation] = cand_map.get(&rule_id).map(|v| v.as_slice()).unwrap_or(&[]);

        if !base_viol.is_empty() && cand_viol.is_empty() {
            became_legal.push(rule_id.clone());
        }
        if base_viol.is_empty() && !cand_viol.is_empty() {
            became_illegal.push(rule_id.clone());
        }

        // New violations: present in candidate but not in baseline.
        for v in cand_viol {
            if !base_viol.contains(v) {
                new_violations.push(v.clone());
            }
        }
        // Resolved violations: present in baseline but not in candidate.
        for v in base_viol {
            if !cand_viol.contains(v) {
                resolved_violations.push(v.clone());
            }
        }
    }

    LegalityDelta {
        became_legal,
        became_illegal,
        new_violations,
        resolved_violations,
    }
}
