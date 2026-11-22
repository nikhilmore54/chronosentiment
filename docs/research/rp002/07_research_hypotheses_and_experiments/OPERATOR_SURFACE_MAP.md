# RP-002 — Coralys Operator Surface Map

**Status:** Draft 2026-11-16  
**Authority:** 0%  
**Purpose:** Identify where FRRMAB-style adaptive operator selection (AOS) can attach without embedding ML training in the MOGA core.

---

## 1. Platform vs adapter ownership

| Layer | What exists today | AOS role |
| ----- | ----------------- | -------- |
| `coralys-moga` | Single `MutationOperator` + single `CrossoverOperator`; fixed `mutation_rate` / `crossover_rate` | Host **selection policy interface** (arm choose + credit update); no domain operators |
| Domain adapters | Concrete variation / destroy / repair operators | Own **arm portfolio** + domain FIR definition |
| Learning Intelligence (future) | Not implemented | May later supply priors / contextual features; not required for E0 |

Hard boundary (RP-002 / ARCH-001-LI): FRRMAB credit + UCB selection are **Optimization control**, not supervised training inside MOGA.

---

## 2. `coralys-moga` EvolutionEngine surface

**Code anchors**

- Traits: `coralys-moga/src/traits/evaluator.rs` — `MutationOperator`, `CrossoverOperator`
- Loop: `coralys-moga/src/engine.rs` — Bernoulli apply of one mutator / one crossover; forced mutate if neither fires
- Stub: `TransformationOperator` in `runtime/optimization/operator.rs` (empty marker)

**Current limitation for FRRMAB**

There is **no operator pool** and **no per-application credit telemetry**. Rates are static config. AOS cannot be reproduced until the engine (or an adapter harness) exposes:

1. A finite set of named arms `K ≥ 2`
2. Per-application: `(arm_id, parent_scalar, child_scalar)` → FIR
3. A pluggable `OperatorSelectionPolicy` (fixed / PM / UCB / FRRMAB / …)

**Proposed platform contract (design only — not implemented)**

```text
OperatorArmId: u8 | String
CreditEvent { arm, fir, generation, eval_budget_used }
OperatorSelectionPolicy {
  select(rng) -> OperatorArmId
  observe(CreditEvent)
}
```

MOGA remains deterministic given seed + policy state; stochastic ML training stays out of this path.

---

## 3. Ready portfolio — ROADEF LNS (RP-404 / RP-405)

**Best near-term FRRMAB attachment point.** Already has five complementary destroy arms and a closed weight-bandit baseline.

| Arm | Name | Source | Role |
| --- | ---- | ------ | ---- |
| 0 | `destroy_random` | RP-404A | Diversification |
| 1 | `destroy_congestion` | RP-404B | Congestion-local |
| 2 | `destroy_highcost` | RP-404B-HC | Cost exposure |
| 3 | `destroy_bottleneck_link` | RP-404C | Saturated edge |
| 4 | `destroy_ecmp_conflict` | RP-404D | ECMP conflict |

**Baseline policy (RP-405):** roulette on weights; ×1.5 on improvement; ×0.9 every 5 iters; bounds `[0.1, 10]`.  
**Binary:** `adapters/roadef/src/bin/rp405_adaptive.rs`  
**Verdict:** Partially supported complementarity; did **not** beat fixed random on aggregate Δobj (−3.53 vs −5.36).

**FIR for E0 (proposed):** on each destroy→repair cycle,

```text
FIR = (parent_obj − child_obj) / max(|parent_obj|, ε)   # minimize
```

Only accept non-empty destroy sets; log zeros/negatives into the sliding window as in FRRMAB (no improvement → FIR ≤ 0).

This portfolio is the **recommended E0 testbed** (operational combinatorial path already instrumented).

---

## 4. UltraCrew mutation surface (secondary)

`adapters/ultracrew/src/optimization.rs` exposes distinct moves (e.g. random reassignment, swap, workload-balanced swap) but EvolutionEngine typically sees **one** `MutationOperator` façade.

| Candidate arms (if unwrapped) | Notes |
| ----------------------------- | ----- |
| Random reassignment | Exploration |
| Swap | Local exchange |
| Workload-balanced swap | Feasibility / load specialist |

**E0 posture:** map only; do **not** use as first reproduce harness until ROADEF FRRMAB vs RP-405 weight bandit is complete.

---

## 5. Continuous MO benchmarks (ZDT / DTLZ / WFG)

Register recommends these to separate general MO learning from routing. Coralys does **not** yet ship a first-class MOEA/D + DE mutation pool matching Li et al.

| Option | Pros | Cons |
| ------ | ---- | ---- |
| Port FRRMAB+MOEA/D reference first | Closest paper reproduce | New infrastructure; less Coralys-native |
| ROADEF destroy arms first | Existing arms + RP-405 control | Not MOEA/D; FIR on scalar LNS obj |
| UltraCrew first | Product-relevant | Arms not yet exposed as pool |

**Recommendation:** E0 = ROADEF FRRMAB vs RP-405 / fixed; E1 = continuous MOEA/D reproduce if E0 shows within-run learning value.

---

## 6. ChronoSentiment / RP-608

**Out of scope for E0.** Gate 5 / OF ±0.30 / OFX freezes unchanged. No Estimand A operator AOS from RP-002.

---

## 7. Mapping summary → FRRMAB

| Coralys surface | Arm set ready? | Scalar FIR ready? | Recommended use |
| --------------- | -------------- | ----------------- | ---------------- |
| EvolutionEngine core | No | No | Add policy interface later |
| ROADEF destroy portfolio | **Yes (5)** | **Yes (obj)** | **E0 primary** |
| UltraCrew mutations | Partial (hidden) | Partial | Later |
| MOEA/D + DE (Li paper) | No | Via subproblem g | E1 paper-faithful reproduce |
| RP-608 trading | N/A | N/A | Forbidden for E0 |

---

*Operator surface map v0.1 | 2026-11-16 | Design only*
