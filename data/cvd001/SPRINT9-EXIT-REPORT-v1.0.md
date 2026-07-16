# Sprint 9 Exit Report

**Document:** SPRINT9-EXIT-REPORT-v1.0.md  
**Date:** 2026-07-16  
**Status:** CLOSED  
**Branch:** governance-hardening  
**Final commit:** 2b6d62c2 (Scenario contract + Experiment 2 backward-compatibility run)

---

## Executive Summary

Sprint 9 completed the first end-to-end execution of the CVD-001 benchmark using the Strategy A (Flight Leg → Shift) representation. The Coralys platform and UltraCrew adapter were successfully validated against a real industrial dataset. A generic Scenario contract was introduced to externalize constraint semantics. The only unresolved issue concerns the authoritative interpretation of the benchmark's workload constraints, which has been isolated as a research question rather than an implementation defect.

---

## Exit Criterion

> Coralys successfully imports, transforms, and optimizes the CVD-001 benchmark under the documented Strategy A representation. The platform architecture has been validated. Remaining uncertainty concerns benchmark evaluation semantics rather than platform capability.

**This criterion is met.**

---

## Metrics Summary

| Metric | Result |
|---|---|
| Workers | 33 |
| Flight legs | 1013 |
| Assigned shifts | 1013 / 1013 |
| Worker coverage | 33 / 33 |
| HC1 violations | 0 |
| HC2 violations | 0 |
| Rest violations | 0 |
| Runtime (Run 1) | 101.64 s |
| Platform defects found | 0 |
| Benchmark semantic questions remaining | 1 |

---

## Sprint 9 Milestone Summary

| Milestone | Commit | Outcome |
|---|---|---|
| M1: Dataset inventory | — | CVD-001 instance1 fully catalogued; 33 crew, 23 airports, 31 days, 1013 legs |
| M2: Adapter pipeline | 86d3c7df | Strategy A pipeline: day CSV → payload → API |
| M3: Full 31-day MOGA run | f1f8ccd5 | HTTP 200, 101.64s, 1013/1013 shifts assigned |
| M4: Structured evaluation | e09c0fa5 | HC3 root cause identified; credited-hours discrepancy resolved |
| M5: Scenario contract + Experiment 2 | 2b6d62c2 | Backward compatibility confirmed; HC3=32 in both runs |
| Experiment 2 Step 3: HC3 semantics audit | b3e8e5e | Evidence chain complete; H1 not confirmed; Experiment 3 blocked |

---

## Engineering Outcomes

### 1. Coralys is now scenario-aware

The `Scenario` struct (`planning_horizon_hours: Option<f64>`, `max_hours_per_worker: Option<f64>`) is a domain-independent contract between adapters and the optimization engine. Constraint semantics are externalized rather than hardcoded. This generalizes beyond CVD-001 to any scheduling domain.

Files modified:
- [`adapters/ultracrew/src/public_contracts.rs`](../../adapters/ultracrew/src/public_contracts.rs) — `Scenario` struct, `ScheduleRequest.scenario`
- [`adapters/ultracrew/src/optimization.rs`](../../adapters/ultracrew/src/optimization.rs) — `ScheduleContext.scenario`
- [`adapters/ultracrew/src/constraint_engine.rs`](../../adapters/ultracrew/src/constraint_engine.rs) — HC3 reads `scenario.max_hours_per_worker.unwrap_or(40)`

### 2. UltraCrew successfully ingests a real airline benchmark

The CVD-001 adapter demonstrates:
- Industrial dataset ingestion (31 day CSV files, 1013 flight legs)
- Automatic transformation to Coralys domain model
- Provenance preservation (leg IDs, crew IDs, base assignments)
- Successful MOGA optimization (1013/1013 shifts, 33/33 workers)
- Reproducible execution (two independent runs, HC3=32 in both)

### 3. Backward compatibility preserved

The Scenario contract is additive. Existing scheduling domains require no changes. The `scenario: None` default preserves all prior behavior.

---

## Research Outcomes

### 1. Scheduling representation evidence

Strategy A (Flight Leg → Shift) is sufficient to produce a complete, feasible schedule for CVD-001 instance1. HC1 (shift coverage), HC2 (worker qualification), and rest constraints are all satisfied. This validates the representation choice.

### 2. HC3 semantics investigation

The per-worker HC3 audit revealed that the current 40h/worker rule is structurally unsatisfiable for CVD-001: 33/33 workers exceed 40h from historical workloads alone (range 23.7h–84.9h, mean 68.75h). This is a bid-period dataset, not a weekly dataset.

### 3. Benchmark reproducibility finding

> **Published CVD-001 benchmark artifacts do not, by themselves, provide sufficient information to uniquely reconstruct the intended workload constraint semantics.**

`credit_constrains.csv` contains per-base aggregate credit hour caps (BASE1=326.9h, BASE2=1279.4h, BASE3=383.3h) derived from the reference solution with 3% slack. The benchmark evaluator source that reads these caps during optimization is not in the available artifact set. Three interpretations remain plausible: hard feasibility constraint, soft optimization objective, or reporting metric.

This finding is documented in [`CVD-001-HC3-SEMANTICS-RESEARCH-NOTE-v1.0.md`](CVD-001-HC3-SEMANTICS-RESEARCH-NOTE-v1.0.md).

---

## Sprint 9 Exit Assessment

| Area | Status | Confidence |
|---|---|---|
| CVD-001 dataset understanding | ✅ Complete | High |
| Schema mapping | ✅ Complete | High |
| Adapter implementation | ✅ Complete | High |
| End-to-end execution | ✅ Complete | High |
| Scenario architecture | ✅ Complete | High |
| UltraCrew integration | ✅ Complete | High |
| HC3 implementation in Coralys | ✅ Complete | High |
| HC3 semantics in the benchmark | ⏸ Hypothesis only | Moderate |
| Experiment 3 | 🚫 Blocked | Appropriate |

---

## What Remains Open

### HC3 semantics (H1 hypothesis)

Three interpretations of `credit_constrains.csv` remain plausible:

1. **H1:** Hard feasibility constraint — total credited hours per base must not exceed cap
2. **H1-alt-A:** Soft optimization objective — penalty for exceeding cap
3. **H1-alt-B:** Reporting/reference metric — no enforcement during optimization

The benchmark evaluator source (GERAD C++ solver) is not in this repository. `README.pdf` is not on the local filesystem.

### Experiment 3 (blocked)

Experiment 3 may proceed only if one of the following occurs:

1. **Authoritative benchmark evidence** — original evaluator source, official README, or documentation from the benchmark authors (Quesnel et al., Polytechnique Montréal / GERAD)
2. **Author clarification** — direct communication with benchmark maintainers
3. **Declared research assumption** — explicit statement that Experiment 3 proceeds under Working Hypothesis H1, with all results labeled as hypothesis-driven rather than benchmark-equivalent

---

## Sprint 10 Roadmap

**Objective:** Recover or justify the authoritative workload-constraint semantics required for faithful reproduction of the CVD-001 benchmark.

| Milestone | Description |
|---|---|
| M1 | Acquire authoritative benchmark documentation (README.pdf, evaluator source, or author materials) |
| M2 | Reconstruct benchmark evaluation semantics from source or documentation |
| M3 | Compare Coralys evaluation with benchmark semantics |
| M4 | Decide HC3 implementation path: generic / configurable / airline solution layer |

---

## Key Files Produced in Sprint 9

| File | Purpose |
|---|---|
| [`scripts/cvd001_adapter.py`](../../scripts/cvd001_adapter.py) | Strategy A pipeline: CVD-001 → Coralys API |
| [`scripts/hc3_audit.py`](../../scripts/hc3_audit.py) | Per-worker HC3 audit |
| [`data/cvd001/CVD-001-INSTANCE1-RESULT-v1.0.json`](CVD-001-INSTANCE1-RESULT-v1.0.json) | Run 1 result |
| [`data/cvd001/CVD-001-INSTANCE1-RESULT-v2.0.json`](CVD-001-INSTANCE1-RESULT-v2.0.json) | Run 2 result (Scenario contract) |
| [`data/cvd001/CVD-001-EXPERIMENT2-SCENARIO-EVALUATION-v1.0.md`](CVD-001-EXPERIMENT2-SCENARIO-EVALUATION-v1.0.md) | Run 1 vs Run 2 comparison |
| [`data/cvd001/CVD-001-HC3-SEMANTICS-RESEARCH-NOTE-v1.0.md`](CVD-001-HC3-SEMANTICS-RESEARCH-NOTE-v1.0.md) | HC3 semantics evidence chain |
| [`data/cvd001/DATASET-INVENTORY-v1.0.md`](DATASET-INVENTORY-v1.0.md) | CVD-001 instance1 catalogue |
| [`data/cvd001/instance1/`](instance1/) | Benchmark instance data (31 day CSV + solution files) |
| [`data/cvd001/credit_constraints.cpp`](credit_constraints.cpp) | Benchmark credit constraint generator source |

---

## Sprint 9 Decision

**Status: CLOSED**

Engineering work planned for Experiment 3 is deferred until benchmark semantics are either:

- verified from authoritative sources, or
- explicitly adopted as a documented research hypothesis.

This concludes Sprint 9.