# CVD-001 HC3 Semantics — Research Note v1.0

**Date:** 2026-07-16  
**Sprint:** 9, Experiment 2 (Step 3 — Source Audit)  
**Status:** EVIDENCE CHAIN COMPLETE — hypothesis not yet confirmed  
**Author:** Coralys / Lyzo

---

## Purpose

This note documents the complete evidence chain for the HC3 semantics investigation.
It distinguishes observations, evidence, hypotheses, and open questions.
It is the prerequisite for Experiment 3.

---

## 1. Observations (directly measured)

### O1 — HC3 as currently implemented

In [`adapters/ultracrew/src/constraint_engine.rs`](../../adapters/ultracrew/src/constraint_engine.rs):

```rust
const DEFAULT_WEEKLY_MAX_HOURS: u64 = 40;
let hc3_limit = self.context.scenario
    .as_ref()
    .and_then(|s| s.max_hours_per_worker)
    .map(|h| h as u64)
    .unwrap_or(DEFAULT_WEEKLY_MAX_HOURS);
if hours > hc3_limit {
    fitness -= 500.0;
    hc3_violations += 1;
}
```

HC3 is a **per-worker** check: `historical_hours + assigned_hours > 40h`.

### O2 — Per-worker audit result (Run 2, all 33 crew)

Script: [`scripts/hc3_audit.py`](../../scripts/hc3_audit.py)

| Worker | Hist(h) | Asgn(h) | Total(h) | Viol? |
|--------|--------:|--------:|---------:|-------|
| EMP001 |   51.83 |   56.48 |   108.32 | YES   |
| EMP002 |   82.20 |   55.15 |   137.35 | YES   |
| EMP003 |   81.25 |   58.08 |   139.33 | YES   |
| EMP004 |   79.03 |   58.48 |   137.52 | YES   |
| EMP005 |   61.17 |   60.42 |   121.58 | YES   |
| EMP006 |   67.70 |   56.23 |   123.93 | YES   |
| EMP007 |   81.53 |   56.22 |   137.75 | YES   |
| EMP008 |   64.42 |   53.85 |   118.27 | YES   |
| EMP009 |   84.90 |   59.23 |   144.13 | YES   |
| EMP010 |   77.45 |   37.20 |   114.65 | YES   |
| EMP011 |   82.92 |   59.38 |   142.30 | YES   |
| EMP012 |   41.65 |   53.57 |    95.22 | YES   |
| EMP013 |   53.73 |   58.17 |   111.90 | YES   |
| EMP014 |   76.78 |   57.33 |   134.12 | YES   |
| EMP015 |   79.28 |   57.35 |   136.63 | YES   |
| EMP016 |   77.45 |   60.42 |   137.87 | YES   |
| EMP017 |   48.62 |   61.08 |   109.70 | YES   |
| EMP018 |   59.72 |   60.40 |   120.12 | YES   |
| EMP019 |   70.60 |   57.25 |   127.85 | YES   |
| EMP020 |   83.40 |   54.25 |   137.65 | YES   |
| EMP021 |   74.28 |   56.00 |   130.28 | YES   |
| EMP022 |   76.67 |   59.52 |   136.18 | YES   |
| EMP023 |   80.47 |   57.28 |   137.75 | YES   |
| EMP024 |   80.67 |   57.08 |   137.75 | YES   |
| EMP025 |   65.87 |   58.35 |   124.22 | YES   |
| EMP026 |   66.62 |   58.58 |   125.20 | YES   |
| EMP027 |   23.67 |   57.73 |    81.40 | YES   |
| EMP028 |   72.77 |   55.83 |   128.60 | YES   |
| EMP029 |   75.65 |   58.62 |   134.27 | YES   |
| EMP030 |   33.55 |   56.85 |    90.40 | YES   |
| EMP031 |   83.68 |   58.05 |   141.73 | YES   |
| EMP032 |   65.48 |   57.48 |   122.97 | YES   |
| EMP033 |   63.60 |   56.57 |   120.17 | YES   |
| **TOTAL** | **2268.60** | **1878.50** | | **33/33** |

**33/33 workers violate HC3 as currently defined.**

### O3 — Historical workloads are bid-period totals, not weekly

Historical hours range: 23.67h (EMP027) to 84.90h (EMP009).  
Mean: 68.75h. These are 31-day bid-period totals, not weekly figures.  
30 of 33 workers already exceed 40h from historical hours alone.

### O4 — Shift duration stats

- 1013 shifts assigned, 0 zero-duration
- Duration range: 0.617h – 3.267h (37–196 min)
- Mean: 1.854h per shift
- Total assigned: 1878.50h

### O5 — API metrics block (Run 2)

```
hard_violations: 0.0
rest_violations: 0.0
fatigue_penalty: 3818.30
fairness_penalty: 135.61
fitness: -9953.91
total_credited_hours: (field absent)
```

`hard_violations = 0.0` despite 33/33 per-worker violations. This confirms the engine's internal HC3 counter and the API's `hard_violations` field measure different things, or the engine's HC3 is not wired to `hard_violations`.

---

## 2. Evidence (from benchmark source files)

### E1 — `creditedHours` is reference solution output

File: [`instance1/creditedHours`](instance1/creditedHours)

The file contains per-worker metrics from `solution_0` (the reference schedule):
- `credited hours`: total flying time for that worker in the reference solution
- `schedule cost`: optimizer cost
- `number of vacations`: rest days

This is **descriptive output** of the reference solution, not a constraint specification.

### E2 — `credit_constrains.csv` is derived from `creditedHours` with 3% slack

File: [`credit_constraints.cpp`](credit_constraints.cpp) (generator, not evaluator)

The generator:
1. Reads `creditedHours` → sums per base
2. Subtracts briefing/debriefing credit (1h per duty day per worker)
3. Adds 3% slack
4. Writes `credit_constrains.csv`

Computed values:

| Base | Workers | Raw credited h | ×1.03 | − briefing | **CSV cap** |
|------|---------|---------------|-------|------------|-------------|
| BASE1 | 7 crew  | 381.38h | 392.82h | −65.92h | **326.91h** |
| BASE2 | 20 crew | 1467.08h | 1511.10h | −231.75h | **1279.35h** |
| BASE3 | 6 crew  | 420.13h | 432.74h | −49.44h | **383.30h** |
| TOTAL | 33 crew | 2268.60h | 2336.66h | −347.11h | **1989.55h** |

### E3 — `credit_constrains.cpp` is a preprocessing tool, not the evaluator

`params.txt` lists it alongside `preferredVacations.cpp` and `EmployeeLegPreferences.cpp` — all are **instance generators**, not the benchmark solver. The benchmark solver that reads `credit_constrains.csv` during optimization is the original GERAD C++ solver, which is **not present in this repository**.

### E4 — Our own inventory describes the values as "targets"

[`DATASET-INVENTORY-v1.0.md`](DATASET-INVENTORY-v1.0.md) line 39:
> `credit_constrains.csv` — "Per-base credit hour **targets**"

Line 115:
> "Credit hours represent the **total flying time assigned to crew at each base** over the planning period."

The word "targets" was chosen deliberately and is consistent with either hard constraints or soft objectives.

### E5 — The benchmark evaluator source is not available locally

The GERAD distribution (`G1422-DataSets.zip`) contains generators only. The solver that enforces or penalizes the credit constraint is not in this repo. The `params.txt` references `../instances/FINAL/` paths that do not exist locally.

---

## 3. Conclusions (established facts)

**C1.** The current Coralys HC3 (`hist + assigned > 40h` per worker) does not match the CVD-001 benchmark. This is proven by O2 and O3: the constraint is structurally unsatisfiable for this dataset.

**C2.** `creditedHours` is reference data, not a constraint specification. It is the output of the prior period's schedule.

**C3.** `credit_constrains.csv` contains per-base aggregate credit hour values derived from the reference solution with 3% slack and briefing credit subtracted.

**C4.** The benchmark evaluator source that reads `credit_constrains.csv` is not available in this repository.

---

## 4. Hypothesis (not yet confirmed)

**H1.** The CVD-001 benchmark enforces `credit_constrains.csv` as a **hard constraint**: the total credited hours assigned to all crew at base B must not exceed `cap(B)`.

**H1-alt-A.** The values are enforced as a **soft penalty** (objective term), not a hard constraint.

**H1-alt-B.** The values are used as **optimization targets** (balance objectives), not enforced at all.

**H1-alt-C.** The values are used for **post-hoc validation** only (reporting metric).

---

## 5. Missing evidence

**M1.** The benchmark solver source code that reads `credit_constrains.csv` and either:
- compares with `>` (hard constraint), or
- adds a penalty term (soft constraint), or
- uses as an objective target, or
- reports only.

**M2.** The CVD-001 benchmark paper or README.pdf (referenced in DATASET-INVENTORY but not extracted locally) may contain the formal constraint specification.

**M3.** The `README.pdf` from `G1422-DataSets.zip` — if it defines HC3 formally, it would resolve H1 vs H1-alt-*.

---

## 6. Recommended next steps (in order)

1. **Extract and read `README.pdf`** from `instance1.zip` or the original archive. This is the most direct path to the formal constraint specification.

2. **Search for the benchmark solver source** — the GERAD group may have published it separately (e.g., on their website or in a companion paper).

3. **If neither is available**: treat H1 as the working hypothesis (per-base hard constraint) and implement Experiment 3 with that assumption, clearly documented as unconfirmed. The 3% slack added to the caps is consistent with hard constraint semantics (slack ensures feasibility), which weakly supports H1 over H1-alt-*.

---

## 7. Evidence chain status

```
Observation: HC3 = 33/33 violations under 40h/worker rule
    ↓ proven
Evidence: 40h/worker rule is structurally unsatisfiable for CVD-001
    ↓ proven
Evidence: credit_constrains.csv contains per-base aggregate caps
    ↓ proven
Evidence: caps are derived from reference solution + 3% slack
    ↓ proven
Hypothesis: caps are enforced as hard constraints in the benchmark
    ↓ NOT YET PROVEN — evaluator source unavailable
Implementation: rewrite HC3 as per-base aggregate check
    ↓ BLOCKED pending hypothesis confirmation
```

---

## 8. Evidence Level Summary

| Evidence level | Status | Source |
|---|---|---|
| Dataset evidence (files, structure, values) | **Complete** | `creditedHours`, `credit_constrains.csv`, `credit_constraints.cpp`, `params.txt` |
| Reference implementation evidence (how caps are enforced) | **Incomplete** | Benchmark evaluator source not in repo; `README.pdf` not on local filesystem |
| Constraint semantics (hard / soft / objective / reporting) | **Hypothesis only** | H1 not confirmed; H1-alt-A/B/C not ruled out |

This table makes immediately clear to future readers why Experiment 3 was paused: the dataset evidence is complete, but the reference implementation evidence needed to determine enforcement semantics is unavailable locally.

---

## 9. Disposition

This note is **FROZEN** pending resolution of M1 or M2.  
Experiment 3 is **BLOCKED** until H1 is confirmed or the working-hypothesis decision is explicitly made.

The decision to proceed under H1 as a working hypothesis must be made explicitly and documented before any code changes to HC3.

**To unblock:**
1. Obtain `README.pdf` from the GERAD archive (`G1422-DataSets.zip`) — the primary source for the formal constraint specification.
2. Locate the benchmark solver source (Quesnel et al., Polytechnique Montréal / GERAD).
3. Or: make an explicit working-hypothesis decision (H1) with documented uncertainty, and proceed with Experiment 3 under that assumption.