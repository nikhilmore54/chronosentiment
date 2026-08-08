# RC Milestone Lifecycle v1.0

**Programme:** Coralys ROADEF 2026 Competition Engineering
**Version:** 1.0 — FROZEN
**Date:** 2026-08-06
**Applies to:** All RC milestones from RC-001 onwards

> **Freeze notice:** This document is frozen at v1.0. No RC milestone may modify this lifecycle.
> If the lifecycle itself must change, a new version (v2.0) must be created and all subsequent
> RC reports must reference the version under which they were conducted. Earlier RC reports
> remain valid under the version that governed them.

---

## Purpose

This document defines the mandatory lifecycle every Competition Engineering (RC) milestone must follow before its changes are integrated into the solver baseline.

The lifecycle exists to preserve experimental validity. The most common failure mode in iterative solver development is tweaking a component between implementation and benchmarking, which destroys the ability to attribute observed changes to the specific modification under test.

---

## The RC Lifecycle

Every RC milestone follows this sequence without exception:

```
Design Note
      ↓
Implementation
      ↓
Smoke Test (cargo check / cargo test)
      ↓
A/B Campaign
      ↓
RC Report
      ↓
Acceptance Decision
      ↓
Merge (if accepted)
```

Nothing skips the A/B campaign.

---

## Stage Definitions

### 1. Design Note

Before any code is written, record:

- The hypothesis being tested (one sentence)
- The EEB subsystem targeted (Construction / Repair / Selection / Variation / Evaluation)
- The primary metric (official ROADEF objective)
- The explanatory metric (IFR, COR, PE, or other EEB metric)
- The acceptance criterion (see §Acceptance Criterion below)
- The CB-000 baseline values for the primary and explanatory metrics

The design note is the contract. It cannot be changed after implementation begins.

### 2. Implementation

Implement the change. The implementation must:

- Preserve the existing behaviour as a selectable mode (e.g. `ConstructionMode::Random` for CB-000)
- Not modify any frozen baseline binary
- Compile cleanly (`cargo check` exits 0)

### 3. Smoke Test

Run `cargo check` and `cargo test` (if tests exist). Fix compilation errors. Do not run the A/B campaign yet.

**Critical rule:** Do not modify the implementation after the smoke test passes. The temptation to tweak before benchmarking destroys experimental validity.

### 4. A/B Campaign

Run the dedicated campaign binary (e.g. `campaign_rc001`) with:

- Fixed seed (same for both arms)
- Identical population size, generation limit, elite count, time budget
- Arm A: existing baseline (CB-000 or previous accepted RC)
- Arm B: the new candidate

Record all five report fields:

1. IFR (or relevant EEB metric) — explanatory
2. Official ROADEF objective — primary
3. Per-instance comparison — where it improved / regressed
4. Execution metrics — runtime, generations, n_eval
5. Verdict — one word

### 5. RC Report

The report answers exactly five questions:

**Q1. Did the primary metric improve?**  
Report mean objective for Arm A and Arm B, wins, losses, ties.

**Q2. Did the explanatory metric improve?**  
Report mean IFR (or COR, PE, etc.) for both arms. This explains *why* the objective changed.

**Q3. Where did it improve?**  
Per-instance table: improved / no change / regression.

**Q4. What happened to execution?**  
Runtime, generations executed, n_eval. Flag if Arm B consumes significantly more time.

**Q5. Is it accepted?**  
One word: ACCEPTED / ACCEPTED WITH CAUTION / REJECTED.

The report is written by the campaign binary automatically. It is frozen immediately after the campaign completes.

### 6. Acceptance Decision

**Primary gate:** Arm B wins on the official ROADEF objective on ≥ 2/3 of instances.

**Regression check:** Arm B mean runtime ≤ 2× Arm A mean runtime.

**Supporting evidence:** The explanatory EEB metric (IFR, COR, etc.) moved in the predicted direction. This is evidence, not a gate.

**Outcome:**

| Condition | Verdict |
|-----------|---------|
| Primary gate passed, no regression | ACCEPTED |
| Primary gate passed, runtime regression | ACCEPTED WITH CAUTION — investigate before integration |
| Primary gate not passed | REJECTED — retain baseline |

### 7. Merge (if accepted)

If ACCEPTED:

1. Make the new mode the default for the RC integration branch.
2. Preserve the CB-000 mode as an immutable selectable option.
3. Update `ROADEF_PROGRAMME.md`: advance the RC milestone TRL to Accepted.
4. Begin the next RC milestone from the accepted baseline.

If REJECTED:

1. Retain the existing baseline unchanged.
2. Document the falsified hypothesis in the RC report.
3. Advance to the next RC milestone.

---

## Acceptance Criterion (canonical)

```
Primary gate:   Arm B wins on official ROADEF objective on ≥ 2/3 of instances.
Regression:     Arm B mean runtime ≤ 2× Arm A mean runtime.
Evidence:       Explanatory EEB metric moved in predicted direction (not a gate).
```

The official ROADEF objective is the primary criterion because competition performance is the primary programme goal. EEB metrics explain *why* a change succeeds or fails — they are diagnostic, not gatekeeping.

---

## What is NOT permitted

- Modifying the implementation after the smoke test passes and before the A/B campaign runs.
- Running the A/B campaign with different seeds for Arm A and Arm B.
- Changing the acceptance criterion after seeing the results.
- Skipping the A/B campaign for "obviously beneficial" changes.
- Merging a REJECTED milestone without re-running the full lifecycle.

---

## RC Milestone Registry

| Milestone | Hypothesis | EEB Target | Status | Verdict |
|-----------|-----------|------------|--------|---------|
| RC-001 | GreedyLoadAware constructor raises IFR and improves objective vs CB-000 | Construction (IFR ↑) | Benchmark pending | — |
| RC-002 | Repair heuristics reduce infeasibility rate and improve objective | Repair (IFR ↑, COR ↑) | Concept | — |
| RC-003 | Large neighbourhood search improves objective via destroy/repair | Variation (PE ↑) | Concept | — |
| RC-004 | ROADEF-aware crossover improves objective vs uniform crossover | Variation (COR ↑) | Concept | — |
| RC-005 | Local search phase improves objective via hill-climbing | Selection (OSR ↑) | Concept | — |
| RC-006 | Evidence-based comparator redesign improves objective vs scalar | Selection (PE ↑) | Concept | — |
| RC-007 | Automated solver configuration improves objective vs fixed parameters | All subsystems | Concept | — |
| RC-008 | Component interaction study identifies best combination | All subsystems | Concept | — |

---

## Relationship to EEB

Every RC milestone targets one or more EEB subsystems:

```
EEB = IFR × COR × PE × OSR × APS × ACR × AOR × SDI × EEB_eval
```

The design note must identify which subsystem is targeted and predict the direction of change. The RC report must confirm or falsify that prediction.

This keeps the programme aligned: competition performance is the primary goal, EEB metrics explain the mechanism.