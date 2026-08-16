# CS-P-006-C.3-R — One authorized Search #2 run

**Document type:** Run authorization and sealed evidence  
**Status:** Frozen complete experiment — C.3-C review recorded; no iteration  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.3, CS-P-006-C.3-I  
**Does not:** overwrite Search #1, retune during the run, change TMV, add indicators, drop IDEA or MAHABANK, require SHORT, require a higher unique-best share, feed evaluation to Coralys, iterate from holdout, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy; Coralys discovers mappings; same input → same output; holdout does not re-enter the same search.

---

## Authorization

C.3-I PASSed. The implementation is frozen. This document authorizes **one complete, immutable experiment**, not permission to iterate.

```text
Search #1
TMV + traded-only fitness + narrow selection pool
             │
             ▼
        FAILED HOLDOUT
             │
             ▼
       C.3-I implementation PASS
             │
             ▼
Search #2     ← this document
same TMV / universe / horizon / seed / MOGA
             │
             ├── continuous V
             ├── NO_TRADE = 0
             ├── full observed candidate pool
             └── full observability
             │
             ▼
        STOP AND INSPECT
```

Scientific question:

> With the information available at T held constant, does changing the learning objective from traded-only return to continuous decision value—and removing the two-elite selection bottleneck—produce a policy that generalizes better?

---

## Five hard conditions

1. **Search #1 remains immutable.** Control artifact `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`. Search #2 has its own directory and artifact.
2. **Evaluation remains invisible to search.** Evolution and selection take only the development and selection slices. Evaluation is loaded only after seal.
3. **Do not stop early.** Seed 42, population 32, 12 generations, mutation 0.25, crossover 0.8, tournament 3, max rules 16, seven instruments, 20-day horizon, certified TMV snapshot `c21ec256…`.
4. **Do not interpret the winner during the run.** No mid-run IDEA / Momentum / SHORT interventions.
5. **Produce the full decision ecology after the seal.** Development V, selection V, evaluation V, action counts, symbol × slice matrices, mean / median / tails, regret, unique-best, opportunity cost, per-instrument value, population ecology.

Unique-best share is **not** a success criterion. SHORT is **not** required to appear.

Primary comparison: **development value → selection value → evaluation value**.

---

## Evidence

Sidecar: `product_validation/CS-P-006/discovery/20260815T051900Z_c3/`

Search #2 artifact: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`  
Search #2 methodology: `eff198957d799419035a5b86f6adceee6233bfa626f5ff2fee39d59132d99a99`  
Search #1 remains: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`

Complete configuration: seed 42, pop 32, 12 generations, 7 instruments, 20D, living pool **286**. Repeated in-process identity matched.

| Measure | Search #1 | Search #2 |
|---|---:|---:|
| Development protocol V | 0.007559 | **0.005313** |
| Selection protocol V | 0.006724 | **0.015103** |
| Evaluation protocol V | −0.005825 | **−0.002964** |
| Mean regret (evaluation) | 0.056182 | 0.053321 |
| Unique-best (evaluation) | 18.7% | 51.6% |
| Living selection candidates | 2 elites | 286 |
| Population ecology | SEARCH-SPACE EXPLORED | SEARCH-SPACE EXPLORED |

Sealed genome identity: `50709d968b90bec17e6904d6f1daf9c16ba636a8b9dfea4fa6495482fd745839` (7 rules).  
Action mix on 273 rows (diagnostic): Search #1 121 LONG / 0 SHORT / 152 NO_TRADE; Search #2 230 LONG / 43 SHORT / 0 NO_TRADE.

Unique-best and SHORT presence are diagnostics, not success criteria. Do not iterate from these numbers.

---

## After this run

Stop. Inspect the evidence before changing the information set, the representation, or Coralys.

Engine version remains **`unfrozen-dev`**. No real capital.
