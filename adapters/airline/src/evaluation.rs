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
