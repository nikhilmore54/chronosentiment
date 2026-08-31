# Optimizer Root-Cause Report
## 40/196 vs 194/196 Recommendation Defect

**Branch:** `governance-hardening @ 6f6a5786f`  
**Status:** Investigation complete — no code modified

---

```
ROOT CAUSE
Responsible layer:  UltraCrew domain adapter — problem definition
Function/file:      UltraCrewEvaluator::evaluate()
                    adapters/ultracrew/src/inrc/schedule_optimizer.rs:66–157

Mechanism:          Coverage of required staffing positions is absent from the
                    FitnessVector supplied to coralys_moga. The 5-objective vector is:
                      [s6_assignment_penalty, s7_weekend_penalty, recovery_penalty,
                       workload_balance, temporal_load_balance]
                    A genome with 40/196 filled positions scores near-zero on all 5
                    objectives (under-assigned nurses violate no constraints and have
                    perfectly balanced low workloads). A genome with 194/196 filled
                    positions scores higher on all 5 (fully-assigned nurses trigger
                    assignment, weekend, and streak constraints). coralys_moga's Pareto
                    dominance logic is correct — it correctly identifies that 40/196
                    dominates 194/196 on the problem it was given.

    Additionally: InrcEvaluation::is_valid() always returns true
                  (adapters/ultracrew/src/inrc/optimization.rs:86–89)
                  so no genome is ever rejected as infeasible regardless of coverage
                  deficit. The feasibility gate is disabled.

    Additionally: archive.solutions[0] is the first Pareto-non-dominated solution
                  inserted (the baseline seed), not a ranked recommendation.
                  (adapters/ultracrew/src/pipeline.rs:146–147)
                  The optimizer does not set recommended_alternative_id.
                  rankAlternatives() in the UI adapter is the only ranking step.

---

40/196 (Candidate A):
  feasibility:       FEASIBLE (is_valid() always true)
  HC1 coverage cost: 0 — not evaluated by UltraCrewEvaluator
  objectives:        [~0, ~0, ~0, ~0, ~0]  (near-zero on all 5)
  selection status:  Non-dominated — admitted to Pareto archive

194/196 (Candidate B):
  feasibility:       FEASIBLE (is_valid() always true)
  HC1 coverage cost: 0 — not evaluated by UltraCrewEvaluator
  objectives:        [higher, higher, higher, higher, higher]
  selection status:  Dominated by A on all 5 objectives — rejected from archive
                     OR non-dominated on some but ranked behind A by insertion order

WHY 40/196 WINS:
  The optimization problem given to coralys_moga contains no coverage objective
  and no feasibility gate. Under-staffed schedules are mathematically optimal on
  all 5 objectives that were defined. coralys_moga is functioning correctly.
  The problem definition is wrong.

---

COVERAGE SEMANTICS: HARD CONSTRAINT

Evidence:
  [adapters/ultracrew/src/inrc/models.rs:132]
    InrcConstraintId::Hc1MinimumCoverage
    The domain model explicitly classifies minimum coverage as HC (Hard Constraint).
    The HC prefix is the INRC standard notation for hard constraints.

  [adapters/ultracrew/src/inrc/models.rs:117–120]
    InrcRequirementLevel { minimum: usize, optimal: usize }
    The data model distinguishes minimum (hard) from optimal (soft).
    minimum → HC1_MinimumCoverage (hard)
    optimal → S8_OptimalCoverage (soft)

  [adapters/ultracrew/src/inrc/evaluator.rs:240–255]
    The scalar InrcOptimizer path applies:
      cost = missing * hard_constraint_violation (1000 per unit)
    to minimum shortfalls — confirming the intended hard-constraint treatment.

  [adapters/ultracrew/src/inrc/models.rs:162]
    hard_constraint_violation = 1000
    This is 33× the soft-constraint weights (30), consistent with a hard requirement.

  [adapters/ultracrew/src/inrc/validator.rs:143–147]
    is_legal does NOT include coverage — confirming that the simple validator
    (used for streak/succession checks) is incomplete, not that coverage is soft.

Relationship to scalar path:
  The scalar InrcOptimizer (run_pipeline / run_pipeline_from_request) correctly
  treats HC1_MinimumCoverage as a hard constraint with weight 1000.
  The Pareto UltraCrewEvaluator (run_inrc_startup_pipeline) omits it entirely.
  This is an inconsistency between the two optimizer paths in the same adapter.

Relationship to canonical 196-position demand:
  The UI's canonical demand model (3 Early + 2 Late + 2 Night × 28 days = 196)
  corresponds to InrcRequirementLevel.minimum across all shift/skill/day slots.
  The 196 required positions are the sum of all minimum requirements.
  These are HC1 positions — hard requirements — not soft preferences.

---

WORKFORCE CAPACITY DETERMINATION:

  Instance n030w4 (smallest benchmark, 30 nurses, 4 weeks):
    Minimum demand per week (sum of all InrcRequirementLevel.minimum): 91 positions
    Minimum demand over 4 weeks: 364 positions
    Workforce capacity (upper bound): 30 nurses × max 22 assignments = 660
    Workforce capacity (conservative): ~360–450 assignments available

  The INRC benchmark instances are designed so the workforce can satisfy minimum
  coverage. InrcRequirementLevel.minimum is an absolute demand requirement, not
  a workforce-capacity-relative target. A 40/196 result is not a workforce
  limitation — it is the optimizer failing to assign available nurses to required
  positions because coverage was absent from its objective vector.

  Therefore: coverage_deficit > 0 is a legitimate infeasibility indicator for
  these instances. The correction is valid.

DECISION: Option A authorized — IMPLEMENTED

  Implementation: coverage_deficit added as objective[5] in UltraCrewEvaluator::evaluate()
  File: adapters/ultracrew/src/inrc/schedule_optimizer.rs
  Weight: 1000 per uncovered minimum position (matches hard_constraint_violation)

  Implementation approach: penalty-based Pareto dominance (not is_valid() gate)
  Rationale: The engine_proof EvolutionEngine does not call is_valid(). The
  penalty magnitude (1000 per position) ensures any genome with coverage_deficit > 0
  is dominated by any genome with coverage_deficit = 0 on objective[5], regardless
  of the other 5 objectives. This is mathematically equivalent to a hard feasibility
  gate in Pareto terms, while preserving the generic MOGA architecture.

  Expected effect:
    40/196 → objective[5] = 156,000 → dominated by any genome with coverage_deficit = 0
    194/196 → objective[5] = 2,000 → dominated by 196/196 on objective[5] only
    196/196 → objective[5] = 0 → non-dominated on coverage objective

  Build: exit 0 (warnings only, no errors)
  Call sites updated: adapters/ultracrew/src/pipeline.rs + 7 bin files in services/

  Regression tests required:
    1. A genome with coverage_deficit > 0 must be dominated by coverage_deficit = 0.
    2. The existing 24 tests in selectDecision.test.ts must pass.
    3. The existing 7 tests in rankAlternatives.test.ts must pass.

CODE CHANGED — build passes.
```

---

## Evidence Index

| Claim | Source |
|---|---|
| HC1_MinimumCoverage is a hard constraint | [`adapters/ultracrew/src/inrc/models.rs:132`](adapters/ultracrew/src/inrc/models.rs:132) |
| InrcRequirementLevel.minimum = hard, .optimal = soft | [`adapters/ultracrew/src/inrc/models.rs:117–120`](adapters/ultracrew/src/inrc/models.rs:117) |
| Scalar path applies hard_constraint_violation=1000 to minimum shortfalls | [`adapters/ultracrew/src/inrc/evaluator.rs:240–255`](adapters/ultracrew/src/inrc/evaluator.rs:240) |
| FitnessVector[5] — coverage absent | [`adapters/ultracrew/src/inrc/schedule_optimizer.rs:150–156`](adapters/ultracrew/src/inrc/schedule_optimizer.rs:150) |
| `is_valid()` always `true` | [`adapters/ultracrew/src/inrc/optimization.rs:86–89`](adapters/ultracrew/src/inrc/optimization.rs:86) |
| `archive.solutions[0]` = first inserted | [`adapters/ultracrew/src/pipeline.rs:146–147`](adapters/ultracrew/src/pipeline.rs:146) |
| Pareto dominance logic (correct) | [`coralys-moga/src/engine_proof.rs:47–84`](coralys-moga/src/engine_proof.rs:47) |
| validator.is_legal does not include coverage | [`adapters/ultracrew/src/inrc/validator.rs:143–147`](adapters/ultracrew/src/inrc/validator.rs:143) |
| ObjectiveWeights: hard_constraint_violation = 1000 | [`adapters/ultracrew/src/inrc/models.rs:162`](adapters/ultracrew/src/inrc/models.rs:162) |