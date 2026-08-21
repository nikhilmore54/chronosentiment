# UC-FEAS-001: Canonical Constraint Enforcement

**Claim:** UltraCrew's optimization pipeline cannot classify a schedule containing canonical hard-constraint violations as valid.

This evidence gate ensures that the constraint engine and the GA fitness evaluator are identically aligned on schedule feasibility. Specifically:
`ConstraintEngine::evaluate` must find violations, and `ScheduleOptimizer::evaluate` must strictly set `is_valid == false`.

## Constraint Cases

- `case_001_double_booking`: Checks overlapping assignments.
- `case_002_rest_7h59`: Checks boundaries of minimum rest (< 8h fails).
- `case_003_rest_8h00`: Checks boundaries of minimum rest (>= 8h passes).
- `case_004_weekly_40h00`: Checks boundaries of weekly limits (<= 40h passes).
- `case_005_weekly_40h01`: Checks boundaries of weekly limits (> 40h fails).
- `case_006_unqualified_worker`: Checks skill/role mismatch (HC1).
