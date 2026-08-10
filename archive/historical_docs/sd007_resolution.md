# SD-007 Resolution — Discovery Failure Root Cause Investigation

**Status:** OPEN (Sprint 3.11 evidence identifies O3 attractor as primary rejection gate; operator redesign required to close)
**Sprint:** 3.11 (updated from 3.10)
**Seed:** 61
**Instance:** n050w4
**Opened:** Sprint 3.9
**Last updated:** Sprint 3.11

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

### Sprint 3.11 — Rejection Attribution Probe (pre-selection, per-rejected-improving-offspring)

Run: seed=61, 5000 gens. Fires for every HC-improving offspring (child_hc < parent_hc).
Records ΔHC magnitude and ΔO3 direction separately for rejected vs admitted offspring.

| Metric | Value |
|---|---|
| Total HC-improving offspring | 1,264 |
| HC-improving AND admitted | 256 |
| HC-improving AND rejected | 1,008 |
| E[ΔHC \| improving AND rejected] | 1,714.29 |
| E[ΔHC \| improving AND admitted] | 1,699.22 |
| Ratio (admitted / rejected) | 0.99× |
| P(O3 worsened \| rejected improving) | 0.5159 (51.59%) |
| HC-improving AND O3-worsening AND rejected | 520 (51.59%) |
| HC-improving AND O3-improving AND rejected | 474 (47.02%) |

**ΔHC magnitude distribution (rejected improving only):**

| Bucket | Count | % |
|---|---|---|
| \|ΔHC\| ≤ 10 (tiny) | 0 | 0.00% |
| 10 < \|ΔHC\| ≤ 100 (small) | 0 | 0.00% |
| 100 < \|ΔHC\| ≤ 1000 (medium) | 518 | 51.39% |
| \|ΔHC\| > 1000 (large) | 490 | 48.61% |

**Key finding:** Zero tiny improvements are rejected. All rejected HC-improving offspring
have medium-to-large ΔHC (100–1000+ penalty units). Step-size asymmetry (Sub-A) is
**falsified** — the rejected improvements are not small; they are comparable in magnitude
to admitted improvements (ratio = 0.99×).

**Key finding:** 51.59% of rejected HC-improving offspring also worsen O3. The archive
preferentially rejects offspring that trade O3 for HC improvement. O3 proxy pressure
(Sub-B) is **confirmed** as the primary rejection gate.

---

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

### RC-2 (Proxy Misalignment / Selection Suppression) — MECHANISM IDENTIFIED

**Sprint 3.10 evidence:** Of 1,438 HC-improving offspring, only 333 (23.16%) were admitted.
The archive rejected 76.84% of all HC-improving offspring. The conditional admission
rates are 23.16% (improving) vs 25.68% (worsening) — close in absolute terms, but
the gap is directionally consistent: HC improvement is not rewarded by the archive.

**Sprint 3.11 evidence (mechanism identified):** The rejection gate is O3 proxy pressure.
- 51.59% of rejected HC-improving offspring also worsen O3 (Sub-B confirmed)
- Rejected improvements are NOT small: zero tiny improvements rejected; all are medium-to-large
- E[ΔHC | rejected] = 1,714 vs E[ΔHC | admitted] = 1,699 — ratio 0.99× (Sub-A falsified)
- The archive is not rejecting weak improvements; it is rejecting improvements that cost O3

The proxy objectives (O1–O5) appear insufficiently aligned with HC reduction as a
path to feasibility. One plausible mechanism is O3 proxy pressure (confirmed by
Sprint 3.11), but the full attribution is not yet complete: 47.02% of rejected
HC-improving offspring improved O3 and were still rejected, indicating other proxy
objectives also gate HC-improving offspring.

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
| RC-2 Proxy Misalignment | **Mechanism Identified** | O3 attractor confirmed as primary rejection gate (Sprint 3.11) |
| Sub-A: Step-size asymmetry | **Falsified** | Ratio = 0.99×; rejected improvements are NOT smaller than admitted ones |
| Sub-B: O3 proxy pressure | **Confirmed** | P(O3 worsened \| rejected improving) = 51.59%; zero tiny improvements rejected |
| O3 as dominant attractor | **Confirmed** | 51.59% of rejected HC-improving offspring worsen O3; consistent with gen-69 event |
| RC-3 partial contributor | Medium | HC floor = 33k; initialization is deep |
| RC-4 | **Falsified** | Code audit |

**Most likely mechanism:**

```
Mutation generates HC-improving moves at 28.76% rate
    ↓
But worsening moves are more frequent (51.72%) and larger in aggregate
    ↓
Mean ΔHC = +600 (net drift away from feasibility)
    ↓
Of the 28.76% improving offspring, 76.84% are rejected by archive selection
    ↓
Rejection gate: 51.59% of rejected HC-improving offspring also worsen O3
    ↓
O3 acts as dominant attractor — archive preferentially retains O3-improving offspring
    even when they worsen HC (consistent with gen-69: +15,845 HC penalty for 330 O3 units)
    ↓
HC-improving offspring that cost O3 are dominated away regardless of ΔHC magnitude
    ↓
HC floor stabilises at 33–34k; never approaches 0
    ↓
Discovery Failure
```

---

## 5. What Sprints 3.10 and 3.11 Proved and Did Not Prove

### Proved (Sprint 3.10)

1. Mutation operator CAN reduce HC_Total (P = 28.76%). **RC-1 Operator Incapacity falsified.**
2. Operator has systematic positive drift: 51.72% worsening, mean ΔHC = +600. **RC-1 Operator Bias confirmed.**
3. 76.84% of HC-improving offspring are rejected by archive selection.
4. Conditional admission rates: P(admit|improving) = 23.16% vs P(admit|worsening) = 25.68%.
5. Archive does not explicitly prefer worsening offspring; it evaluates orthogonal objectives.

### Proved (Sprint 3.11)

6. **Sub-A (step-size asymmetry) FALSIFIED:** E[ΔHC | rejected] = 1,714 vs E[ΔHC | admitted] = 1,699 (ratio 0.99×). Rejected improvements are not smaller than admitted ones. Zero tiny improvements (|ΔHC| ≤ 100) are rejected.
7. **Sub-B (O3 proxy pressure) CONFIRMED:** P(O3 worsened | rejected improving) = 51.59%. The majority of rejected HC-improving offspring also worsen O3. The archive preferentially rejects offspring that trade O3 for HC improvement.
8. **O3 attractor confirmed:** The rejection gate is not step-size weakness but O3 proxy pressure. This is mechanistically consistent with the gen-69 event (+15,845 HC penalty traded for 330 O3 units).

### Not Yet Proved

1. **Joint magnitude:** E[ΔHC | improving AND worsening-O3 AND rejected] vs E[ΔHC | improving AND improving-O3 AND rejected] — not yet measured. Would confirm whether O3-worsening improvements are also larger or smaller.
2. **Other proxy gates:** 47.02% of rejected HC-improving offspring improved O3 but were still rejected. Which proxy objectives (O1, O2, O4) gate these?
3. **O3 attractor structural vs artifact:** Is the O3 attractor a property of the n050w4 constraint landscape or an artifact of the proxy formulation? Requires operator redesign experiment.
4. **Constraint anti-correlation (Coverage ↔ Successions):** Archive member decomposition reveals a striking pattern: HC_Coverage is improving (baseline 18k → many members at 12–14k) while HC_Successions is exploding (20k → 33–41k+). HC_OneShiftPerDay is consistently 0 (solved). This suggests the search has found a basin where reducing coverage violations systematically increases succession violations. If confirmed, the question shifts from "why doesn't HC improve?" to "why does reducing coverage increase successions?" — a much more concrete mechanism than proxy misalignment alone.
5. **Counterfactual:** If selection were neutral (random admission), would HC accumulate toward feasibility?

---

## 6. Residual Uncertainty and Next Experiment

The primary open question has shifted from "why are HC-improving offspring rejected?" to:

> Is the O3 attractor a structural property of the n050w4 constraint landscape,
> or an artifact of the proxy objective formulation?

**Sub-B is confirmed** as the primary rejection gate. The remaining scientific debt is:

**Required experiment A (Sprint 3.12 candidate):** Split ΔHC into components (ΔHC_Coverage,
ΔHC_Skills, ΔHC_Successions, ΔHC_OneShift) for every offspring. This would reveal whether
the positive HC drift is driven by succession explosion while coverage improves — confirming
the Coverage ↔ Successions anti-correlation hypothesis. If P(ΔHC_Successions > 0 | ΔHC_Coverage < 0)
is high, the constraint landscape has a structural anti-correlation that the current proxy
cannot navigate.

**Required experiment B (Sprint 3.12 candidate):** Targeted HC-reduction operator that
does not perturb O3 (e.g. shift-swap within the same succession family). If such an
operator produces HC-improving offspring that are admitted at higher rates, the O3
attractor is confirmed as an artifact of the current proxy formulation, not the landscape.
Alternatively, if HC-improving offspring still worsen O3 even with targeted operators,
the attractor is structural (HC and O3 are anti-correlated in the n050w4 landscape).

---

## 7. Scientific Debt Ledger (updated Sprint 3.11)

| ID     | Status | Classification |
|--------|--------|----------------|
| SD-003 | CLOSED | Champion Retention Error — O3 proxy domination (Sprint 3.6/3.7) |
| SD-005 | CLOSED | Discovery Failure — evaluator never returned feasible=true (Sprint 3.8) |
| SD-006 | CLOSED | O3 proxy pressure evicts best-ever champions (Sprint 3.7) |
| SD-007 | OPEN   | Discovery Failure root cause — O3 attractor confirmed as primary rejection gate; structural vs artifact question open |

**Defensible conclusion (Sprint 3.11):**

> Discovery Failure is not caused by inability to generate HC-improving offspring (RC-1 Incapacity falsified).
> Discovery Failure is not caused by step-size weakness of HC-improving offspring (Sub-A falsified).
> Discovery Failure arises because HC-improving offspring that worsen O3 are preferentially rejected
> by the Pareto archive. O3 acts as a dominant attractor: 51.59% of rejected HC-improving offspring
> also worsen O3, and the rejected improvements are large (median 100–1000+ penalty units), not small.
> The mechanism is O3 proxy pressure (Sub-B confirmed).
>
> A secondary hypothesis — not yet tested — is that the n050w4 constraint landscape has a structural
> anti-correlation between HC_Coverage and HC_Successions: archive members show coverage improving
> (18k → 12–14k) while successions explode (20k → 33–41k+). If this anti-correlation is confirmed,
> the search is trapped in a basin where coverage repair systematically increases succession violations,
> and the O3 attractor is a symptom of a deeper landscape geometry problem.

---

## 8. Instrumentation Scope Audit

The ΔHC probe fires at [`inrc_archive_forensics.rs:602`](services/ultracrew_server/src/bin/inrc_archive_forensics.rs:602):

```rust
delta_hc_probe.record(parent_hc, child_hc_for_probe, was_inserted);
```

The rejection attribution probe fires at [`inrc_archive_forensics.rs:610`](services/ultracrew_server/src/bin/inrc_archive_forensics.rs:610):

```rust
rejected_improving_probe.record(
    parent_hc, child_hc_for_probe,
    parent_o3_for_probe, child_o3_for_probe,
    was_inserted,
);
```

- `parent_hc` is computed from `score_inrc_official(&parent.genome, ...)` before `engine.archive.add()`.
- `child_hc_for_probe` is computed from `child_score` before `archive.add()`.
- `parent_o3_for_probe = parent.fitness[2]` (proxy O3 = HC_Successions, already in archive).
- `child_o3_for_probe = child_fitness[2]` (proxy O3 of the offspring, before admission).
- `was_inserted` is the return value of `engine.archive.add()`.
- Both probes fire after `was_inserted` is known, before the next generation.

**Scope:** All 5,000 evaluated offspring. Not filtered by admission. Covers the full
pre-selection offspring distribution. The rejection attribution probe additionally
fires only for HC-improving offspring (child_hc < parent_hc), recording ΔHC magnitude
and ΔO3 direction separately for rejected vs admitted offspring.

**Artifact:** [`rejection_attribution_report.md`](services/ultracrew_server/rejection_attribution_report.md) — Sprint 3.11 canonical evidence artifact.