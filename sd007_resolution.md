# SD-007 Resolution — Discovery Failure Root Cause Investigation

**Status:** OPEN (Sprint 3.10 evidence narrows but does not close)  
**Sprint:** 3.10 (updated from 3.9 interim)  
**Seed:** 61  
**Instance:** n050w4  
**Opened:** Sprint 3.9  
**Last updated:** Sprint 3.10  

---

## 1. Problem Statement

SD-005 established that the evaluator never returned `feasible=true` across 5000 generations
(Discovery Failure). SD-007 investigates the root cause of that Discovery Failure.

Four candidate root causes were defined in `sd007_sprint39_charter.md`:

| ID   | Hypothesis                                                                 |
|------|----------------------------------------------------------------------------|
| RC-1 | Operator Incapacity — mutation cannot reduce HC violations                 |
| RC-2 | Proxy Misalignment — selection pressure removes HC-improving offspring     |
| RC-3 | Initialization Depth — initial population too far from feasibility         |
| RC-4 | Evaluator Anomaly — evaluator misclassifies feasible genomes as infeasible |

---

## 2. Evidence Summary

### Sprint 3.9 — HC_Total Distribution Probe (archive-level)

Run: seed=61, 5000 gens. Archive sampled every 100 gens.

| Observation | Value |
|---|---|
| Min HC_Total (gen 100) | 33,000 (33 actual violations) |
| Min HC_Total (gen 5000) | 34,000 |
| Mean HC_Total (gen 100) | ~41,205 |
| Mean HC_Total (gen 5000) | ~53,954 |
| HC=0 (feasible) at any checkpoint | 0 |
| HC≤5 at any checkpoint | 0 |
| HC≤50 at any checkpoint | 0 |

**Interpretation:** Archive HC quality does not improve over 5000 generations; it worsens.
This probe measures archive trajectory after selection — it cannot distinguish operator
incapacity (RC-1) from selection suppression (RC-2).

### Sprint 3.10 — ΔHC Offspring Probe (pre-selection, offspring-level)

Run: seed=61, 5000 gens. Every offspring scored before `archive.add()`.

| Metric | Value |
|---|---|
| Total offspring evaluated | 5,000 |
| HC-improving (child_hc < parent_hc) | 1,438 (28.76%) |
| HC-neutral (child_hc == parent_hc) | 976 (19.52%) |
| HC-worsening (child_hc > parent_hc) | 2,586 (51.72%) |
| HC-improving AND admitted | 333 |
| HC-worsening AND admitted | 664 |
| P(child_hc < parent_hc) | 0.2876 |
| P(admitted \| improving) | 0.2316 (23.16%) |
| P(admitted \| worsening) | 0.2568 (25.68%) |
| Mean ΔHC per offspring | +600.2 (penalty-weighted) |

**Critical observation:** 76.84% of HC-improving offspring are rejected by the archive.
The raw counts (664 worsening admitted vs 333 improving admitted) suggest a 2× ratio,
but the conditional admission rates are much closer: 23.16% vs 25.68%. The archive does
not explicitly prefer worsening offspring — it evaluates orthogonal objectives and HC
improvement is simply not rewarded. This is a subtler mechanism than anti-feasibility bias.

---

## 3. Root Cause Classification

### RC-4 (Evaluator Anomaly) — FALSIFIED

Code audit (`sd005_instrumentation_audit.md`) confirmed: evaluator correctly sets
`feasible = hard == 0` with no short-circuit. `O1 = -(hard as f64)`. No anomaly.

### RC-1 (Operator Incapacity) — FALSIFIED (strong form)

**Evidence:** `P(child_hc < parent_hc) = 28.76%`

Nearly 1 in 3 offspring reduces HC_Total. An incapable operator would produce
`P ≈ 0`. The mutation operator (`UltraCrewMutator`) demonstrably can generate
HC-improving moves. The strong form of RC-1 ("operator cannot reduce HC") is false.

**Residual:** The operator produces more worsening steps (51.72%) than improving steps
(28.76%), and mean ΔHC = +600 (net drift away from feasibility). Operator capacity
is real but asymmetric — worsening moves are more frequent and/or larger than
improving moves. This is a weak form of RC-1 (operator bias, not incapacity).

### RC-2 (Proxy Misalignment / Selection Suppression) — STRONGLY INDICATED

**Evidence:** Of 1,438 HC-improving offspring, only 333 (23.16%) were admitted.
The archive rejected 76.84% of all HC-improving offspring. The conditional admission
rates are 23.16% (improving) vs 25.68% (worsening) — close in absolute terms, but
the gap is directionally consistent: HC improvement is not rewarded by the archive.

The proxy objectives (O1–O5) are not aligned with HC reduction as a path to
feasibility. The archive evaluates orthogonal objectives; HC-improving offspring
that worsen O3 (coverage ratio) are dominated and evicted regardless of their
HC progress.

**Connection to SD-006:** The gen-69 domination event (Sprint 3.10 run) shows:
- Victim official_total = 44,910; Dominator official_total = 60,755
- ΔOfficialTotal = +15,845 (dominator is 15,845 points worse externally)
- ΔO3 = −330 (dominator improves O3 by 330 units)

The archive traded 15,845 penalty points of external quality for 330 O3 units.
This is a more severe proxy geometry failure than the gen-283 event from Sprint 3.7
(which showed ΔOfficialTotal = +2,590 for ΔO3 = −1,755).

### RC-3 (Initialization Depth) — PARTIAL

Baseline HC_Total = 39,000 (19+2+0+18 × 1000 penalty-weighted = 39 actual violations).
The HC floor at gen 100 is 33,000 (33 violations). Initialization is not shallow —
the search starts 33+ violations from feasibility. RC-3 contributes to the difficulty
but is not the primary cause.

---

## 4. Current Classification

**SD-007 Status: OPEN**

| Hypothesis | Confidence | Evidence |
|---|---|---|
| RC-1 Operator Incapacity (strong form) | **Falsified** | P(improving) = 28.76% — operator CAN reduce HC |
| RC-1 Operator Bias (weak form) | **Confirmed** | 51.72% worsening, mean ΔHC = +600; systematic positive drift |
| RC-2 Proxy Misalignment | **Strongly Indicated** | P(admit\|improving) = 23.16% vs P(admit\|worsening) = 25.68% |
| Mutation step-size asymmetry | **Unresolved** | E[ΔHC\|improving] and E[ΔHC\|worsening] not yet measured |
| O3 as dominant attractor | **Unresolved** | Gen-69 event: +15,845 penalty traded for 330 O3 units |
| RC-3 partial contributor | Medium | HC floor = 33k; initialization is deep |
| RC-4 | **Falsified** | Code audit |

**Most likely mechanism:**

```
Mutation generates HC-improving moves at 28.76% rate
    ↓
But worsening moves are more frequent (51.72%) and likely larger in magnitude
    ↓
Mean ΔHC = +600 (net drift away from feasibility)
    ↓
Of the 28.76% improving offspring, 76.84% are rejected by archive selection
    ↓
Archive admits 2× more HC-worsening offspring than HC-improving offspring
    ↓
HC floor stabilises at 33–34k; never approaches 0
    ↓
Discovery Failure
```

---

## 5. What Sprint 3.10 Proved and Did Not Prove

### Proved

1. Mutation operator CAN reduce HC_Total (P = 28.76%). **RC-1 Operator Incapacity falsified.**
2. Operator has systematic positive drift: 51.72% worsening, mean ΔHC = +600. **RC-1 Operator Bias confirmed.**
3. 76.84% of HC-improving offspring are rejected by archive selection.
4. Conditional admission rates: P(admit|improving) = 23.16% vs P(admit|worsening) = 25.68%.
5. Archive does not explicitly prefer worsening offspring; it evaluates orthogonal objectives.

### Not Yet Proved

1. **Magnitude asymmetry:** E[ΔHC | improving] and E[ΔHC | worsening] are not measured.
   Positive mean ΔHC = +600 could arise from many small improvements + few huge regressions,
   or from slightly more regressions than improvements, or from selection feedback loops.
2. **Rejection mechanism:** Are HC-improving offspring rejected because they worsen O3
   specifically (Sub-B), or because they are weak across all proxy objectives (Sub-A)?
3. **O3 attractor magnitude:** The gen-69 event (ΔOfficialTotal = +15,845 for ΔO3 = −330)
   suggests O3 may act as a dominant attractor, not merely another objective.
4. **Counterfactual:** If selection were neutral (random admission), would HC
   accumulate toward feasibility?

---

## 6. Residual Uncertainty and Next Experiment

The critical open question is:

> Why do 76.84% of HC-improving offspring fail to survive selection?

Two sub-hypotheses:

**Sub-A (Step-size asymmetry):** Improving moves are too small to matter. The
offspring improves HC by 1,000 but worsens soft objectives, making it non-competitive
in the archive. Selection is not hostile — the offspring is simply weak.

**Sub-B (O3 proxy pressure):** HC-improving offspring tend to worsen O3 (coverage
ratio), which is the dominant proxy objective. The archive preferentially retains
O3-improving offspring even when they worsen HC. This directly extends SD-006.

**Required experiment (Sprint 3.11):** For every HC-improving offspring that is
rejected, record:
- `delta_hc` (magnitude of HC improvement)
- `delta_o3` (change in O3 coverage proxy)
- Whether the rejecting dominator has better O3

This would separate Sub-A from Sub-B and either close SD-007 or open SD-008
(Selection vs HC Improvement Attribution).

---

## 7. Scientific Debt Ledger (updated Sprint 3.10)

| ID     | Status | Classification |
|--------|--------|----------------|
| SD-003 | CLOSED | Champion Retention Error — O3 proxy domination (Sprint 3.6/3.7) |
| SD-005 | CLOSED | Discovery Failure — evaluator never returned feasible=true (Sprint 3.8) |
| SD-006 | CLOSED | O3 proxy pressure evicts best-ever champions (Sprint 3.7) |
| SD-007 | OPEN   | Discovery Failure root cause — Operator Bias confirmed; O3 attractor mechanism unresolved |

**Defensible conclusion (Sprint 3.10):**

> Discovery Failure is not caused by inability to generate HC-improving offspring.
> Discovery Failure arises because HC-improving offspring fail to accumulate,
> through some combination of operator positive drift and archive selection pressure
> that does not reward HC reduction.

---

## 8. Instrumentation Scope Audit

The ΔHC probe fires at [`inrc_archive_forensics.rs:506`](services/ultracrew_server/src/bin/inrc_archive_forensics.rs:506):

```rust
delta_hc_probe.record(parent_hc, child_hc_for_probe, was_inserted);
```

- `parent_hc` is computed from `score_inrc_official(&parent.genome, ...)` at line 457–461,
  before `engine.archive.add()` at line 465.
- `child_hc_for_probe` is computed from `child_score` at line 460–461, where `child_score`
  is set at line 443 before `archive.add()`.
- `was_inserted` is the return value of `engine.archive.add()` at line 465.
- The probe fires at line 506, after `was_inserted` is known.

**Scope:** All 5,000 evaluated offspring. Not filtered by admission. Covers the full
pre-selection offspring distribution.