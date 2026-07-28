/// Unit tests for the UltraCrew Compliance Framework explainability layer.
///
/// These tests verify that every DGCA rule violation carries:
///   - a stable `rule_id`
///   - a `regulatory_ref` citing the correct CAR section
///   - a non-empty `message` describing the observed breach
///   - a non-empty `remediation` with actionable guidance
///
/// They also verify that the `ComplianceRegistry` correctly aggregates
/// violations and exposes them via `violation_explanations()`.

#[cfg(test)]
mod compliance_explainability_tests {
    use std::collections::HashMap;
    use crate::models::{Worker, Shift, Skill};
    use crate::compliance::{
        ComplianceRegistry, DgcaCompliancePack,
        RuleContext, Severity, ViolationExplanation,
    };
    use crate::compliance::traits::ConstraintRule;
    use crate::compliance::regulatory::dgca::{
        minimum_rest::MinimumRestRule,
        maximum_fdp::MaximumFdpRule,
        flight_hours::{MaxFlightHours28DaysRule, MaxFlightHours365DaysRule},
        standby::StandbyRule,
        limits::DgcaLimits,
    };

    // ── helpers ──────────────────────────────────────────────────────────────

    fn worker(id: u64) -> Worker {
        Worker {
            id,
            skills: vec![Skill("Pilot".to_string())],
        }
    }

    fn shift(id: u64, start: u64, duration: u64, skill: &str) -> Shift {
        Shift {
            id,
            start_hour: start,
            duration_hours: duration,
            required_skill: Skill(skill.to_string()),
        }
    }

    fn ctx_from<'a>(
        workers: &'a [Worker],
        shifts: &'a [Shift],
        assignments: &HashMap<u64, u64>,
    ) -> RuleContext<'a> {
        RuleContext::from_assignments(workers, shifts, assignments)
    }

    // ── MinimumRestRule ───────────────────────────────────────────────────────

    #[test]
    fn minimum_rest_violation_carries_full_explainability() {
        let limits = DgcaLimits::regulatory_defaults(); // min_rest = 12 h
        let rule = MinimumRestRule::new(limits);

        let workers = vec![worker(1)];
        // Shift 1 ends at h10, shift 2 starts at h18 → gap = 8 h < 12 h
        let shifts = vec![
            shift(1, 0, 10, "Pilot"),   // ends h10
            shift(2, 18, 8, "Pilot"),   // starts h18, gap = 8 h
        ];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        assignments.insert(2, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let outcomes = rule.evaluate(&ctx);
        assert_eq!(outcomes.len(), 1, "expected exactly one violation");

        let exp = outcomes[0].explanation().expect("should be a violation");
        assert_eq!(exp.rule_id, "dgca.fdtl.minimum_rest");
        assert!(exp.regulatory_ref.contains("§6.1"), "regulatory_ref should cite §6.1");
        assert_eq!(exp.severity, Severity::Hard);
        assert!(!exp.message.is_empty(), "message must not be empty");
        assert!(!exp.remediation.is_empty(), "remediation must not be empty");
        assert!(exp.message.contains("8 h"), "message should mention the actual gap");
        assert!(exp.message.contains("12 h"), "message should mention the minimum");
    }

    #[test]
    fn minimum_rest_satisfied_returns_no_outcomes() {
        let limits = DgcaLimits::regulatory_defaults();
        let rule = MinimumRestRule::new(limits);

        let workers = vec![worker(1)];
        // Gap = 14 h >= 12 h → satisfied
        let shifts = vec![
            shift(1, 0, 10, "Pilot"),   // ends h10
            shift(2, 24, 8, "Pilot"),   // starts h24, gap = 14 h
        ];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        assignments.insert(2, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        assert!(rule.evaluate(&ctx).is_empty());
    }

    // ── MaximumFdpRule ────────────────────────────────────────────────────────

    #[test]
    fn maximum_fdp_day_violation_carries_full_explainability() {
        let limits = DgcaLimits::regulatory_defaults(); // max_fdp = 11 h
        let rule = MaximumFdpRule::new(limits);

        let workers = vec![worker(1)];
        // Day shift (start_hour 8, mod 24 = 8, not night) of 13 h > 11 h
        let shifts = vec![shift(1, 8, 13, "Pilot")];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let outcomes = rule.evaluate(&ctx);
        assert_eq!(outcomes.len(), 1);

        let exp = outcomes[0].explanation().unwrap();
        assert_eq!(exp.rule_id, "dgca.fdtl.maximum_fdp");
        assert!(exp.regulatory_ref.contains("§5.1"));
        assert_eq!(exp.severity, Severity::Hard);
        assert!(exp.message.contains("13 h"));
        assert!(exp.message.contains("11 h"));
        assert!(!exp.remediation.is_empty());
    }

    #[test]
    fn maximum_fdp_night_violation_uses_10h_limit() {
        let limits = DgcaLimits::regulatory_defaults();
        let rule = MaximumFdpRule::new(limits);

        let workers = vec![worker(1)];
        // Night shift (start_hour 22, mod 24 = 22) of 11 h > 10 h night limit
        let shifts = vec![shift(1, 22, 11, "Pilot")];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let outcomes = rule.evaluate(&ctx);
        assert_eq!(outcomes.len(), 1);

        let exp = outcomes[0].explanation().unwrap();
        assert!(exp.message.contains("night"), "message should identify night duty");
        assert!(exp.message.contains("10 h"), "message should cite 10 h night limit");
    }

    // ── MaxFlightHours28DaysRule ──────────────────────────────────────────────

    #[test]
    fn max_block_hours_28d_violation_carries_full_explainability() {
        let limits = DgcaLimits::regulatory_defaults(); // max_block_hours_28d = 100
        let rule = MaxFlightHours28DaysRule::new(limits);

        let workers = vec![worker(1)];
        // 3 shifts of 40 h each = 120 h > 100 h
        let shifts = vec![
            shift(1, 0,  40, "Pilot"),
            shift(2, 48, 40, "Pilot"),
            shift(3, 96, 40, "Pilot"),
        ];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        assignments.insert(2, 1);
        assignments.insert(3, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let outcomes = rule.evaluate(&ctx);
        assert_eq!(outcomes.len(), 1);

        let exp = outcomes[0].explanation().unwrap();
        assert_eq!(exp.rule_id, "dgca.fdtl.max_block_hours_28d");
        assert!(exp.regulatory_ref.contains("§7.1"));
        assert_eq!(exp.severity, Severity::Hard);
        assert!(exp.message.contains("120"), "message should mention total hours");
        assert!(exp.message.contains("100"), "message should mention limit");
        assert!(exp.message.contains("20"), "message should mention excess");
        assert!(!exp.remediation.is_empty());
    }

    // ── MaxFlightHours365DaysRule ─────────────────────────────────────────────

    #[test]
    fn max_block_hours_365d_violation_carries_full_explainability() {
        let limits = DgcaLimits::regulatory_defaults(); // max_block_hours_365d = 1000
        let rule = MaxFlightHours365DaysRule::new(limits);

        let workers = vec![worker(1)];
        // 26 shifts of 40 h = 1040 h > 1000 h
        let shifts: Vec<Shift> = (0u64..26)
            .map(|i| shift(i + 1, i * 48, 40, "Pilot"))
            .collect();
        let mut assignments = HashMap::new();
        for s in &shifts {
            assignments.insert(s.id, 1);
        }
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let outcomes = rule.evaluate(&ctx);
        assert_eq!(outcomes.len(), 1);

        let exp = outcomes[0].explanation().unwrap();
        assert_eq!(exp.rule_id, "dgca.fdtl.max_block_hours_365d");
        assert!(exp.regulatory_ref.contains("§7.2"));
        assert!(exp.message.contains("1040"));
        assert!(exp.message.contains("1000"));
        assert!(exp.message.contains("40"), "message should mention excess");
    }

    // ── StandbyRule ───────────────────────────────────────────────────────────

    #[test]
    fn standby_duration_violation_carries_full_explainability() {
        let limits = DgcaLimits::regulatory_defaults(); // max_standby = 12 h
        let rule = StandbyRule::new(limits);

        let workers = vec![worker(1)];
        // Standby shift of 14 h > 12 h
        let shifts = vec![shift(1, 0, 14, "Standby")];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let outcomes = rule.evaluate(&ctx);
        assert_eq!(outcomes.len(), 1);

        let exp = outcomes[0].explanation().unwrap();
        assert_eq!(exp.rule_id, "dgca.fdtl.standby_limits");
        assert!(exp.regulatory_ref.contains("§8.2"));
        assert_eq!(exp.severity, Severity::Hard);
        assert!(exp.message.contains("14 h"));
        assert!(exp.message.contains("12 h"));
        assert!(!exp.remediation.is_empty());
    }

    #[test]
    fn standby_callout_notice_violation_carries_full_explainability() {
        let limits = DgcaLimits::regulatory_defaults(); // min_callout_notice = 2 h
        let rule = StandbyRule::new(limits);

        let workers = vec![worker(1)];
        // Standby starts h0; FDP starts h1 → only 1 h notice < 2 h
        let shifts = vec![
            shift(1, 0, 12, "Standby"),
            shift(2, 1, 8,  "Pilot"),
        ];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        assignments.insert(2, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let outcomes = rule.evaluate(&ctx);
        assert!(!outcomes.is_empty(), "expected at least one callout-notice violation");

        let exp = outcomes.iter()
            .find_map(|o| o.explanation())
            .expect("should have at least one violation explanation");
        assert!(exp.regulatory_ref.contains("8.3"), "should cite §8.3 for callout notice");
        assert!(!exp.remediation.is_empty());
    }

    // ── ComplianceRegistry integration ───────────────────────────────────────

    #[test]
    fn registry_violation_explanations_returns_structured_data() {
        let mut registry = ComplianceRegistry::new();
        registry.install(DgcaCompliancePack::default());

        let workers = vec![worker(1)];
        // Shift of 13 h (day) violates MaximumFDP (limit 11 h)
        let shifts = vec![shift(1, 8, 13, "Pilot")];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let explanations = registry.violation_explanations(&ctx);
        assert!(!explanations.is_empty(), "expected at least one violation");

        for exp in &explanations {
            assert!(!exp.rule_id.is_empty());
            assert!(!exp.regulatory_ref.is_empty());
            assert!(!exp.message.is_empty());
            assert!(!exp.remediation.is_empty());
        }
    }

    #[test]
    fn registry_violation_messages_include_regulatory_ref_and_remediation() {
        let mut registry = ComplianceRegistry::new();
        registry.install(DgcaCompliancePack::default());

        let workers = vec![worker(1)];
        let shifts = vec![shift(1, 8, 13, "Pilot")];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        let messages = registry.violation_messages(&ctx);
        assert!(!messages.is_empty());

        for msg in &messages {
            assert!(msg.contains("HARD") || msg.contains("SOFT"), "message should have severity tag");
            assert!(msg.contains("ref:"), "message should include regulatory ref");
            assert!(msg.contains("Remediation:"), "message should include remediation");
        }
    }

    #[test]
    fn satisfied_schedule_produces_no_violations() {
        let mut registry = ComplianceRegistry::new();
        registry.install(DgcaCompliancePack::default());

        let workers = vec![worker(1)];
        // Single 8 h day shift — well within all DGCA limits
        let shifts = vec![shift(1, 8, 8, "Pilot")];
        let mut assignments = HashMap::new();
        assignments.insert(1, 1);
        let ctx = ctx_from(&workers, &shifts, &assignments);

        assert_eq!(registry.hard_violation_count(&ctx), 0);
        assert!(registry.violation_explanations(&ctx).is_empty());
    }
}