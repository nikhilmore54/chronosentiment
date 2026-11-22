# RP-002-E0 — FRRMAB Within-Run AOS Protocol

**Protocol ID:** `RP-002-E0`  
**Title:** FRRMAB vs fixed / weight-bandit operator selection on ROADEF destroy portfolio  
**Disposition:** **DESIGN ONLY · NOT AUTHORISED TO RUN**  
**Date:** 2026-11-16  
**Authority:** **0%**  
**Depends on:** [`BATCH1_WITHIN_RUN_LEARNING_NOTES.md`](BATCH1_WITHIN_RUN_LEARNING_NOTES.md), [`OPERATOR_SURFACE_MAP.md`](OPERATOR_SURFACE_MAP.md), RP-405 closed baseline  

---

## 0. Intent

Test whether **FRRMAB** (Li et al., 2014) improves **within-run** search on Coralys’s existing five-arm ROADEF destroy portfolio relative to:

1. Fixed equal probabilities  
2. Fixed single best historical arm (`destroy_random`, per RP-404/405)  
3. RP-405 weight-based bandit (closed control)

This is the first experimental application of RP-002’s broader question (acquire / evaluate / preserve / reuse optimisation knowledge)—scoped to **within-run adaptation only**.

---

## 1. Hard boundaries

| Boundary | Rule |
| -------- | ---- |
| Architecture | **Unfrozen** — results may revise CLA assumptions |
| MOGA core | No ML training embedded; AOS is selection + credit only |
| RP-405 | Control baseline; do not rewrite historical RP-405 claims |
| ChronoSentiment / Gate 5 / OF / OFX | **Untouched** |
| Execution | **Not authorised** until separate run authorization |

---

## 2. Hypotheses

**H0 (null):** FRRMAB does not improve aggregate objective Δ vs the best control (fixed random or RP-405) under matched budget.

**H1 (primary):** FRRMAB improves mean/median objective Δ (or matches Δ at lower successful-iteration count) vs RP-405 weight bandit on the same instance set and seed schedule.

**H2 (secondary):** FRRMAB retains higher **operator entropy** late in search than winner-take-all weight bandit when multiple arms remain useful (complementary-knowledge probe; links F-02 / RD-3).

**H3 (overhead):** FRRMAB wall-clock overhead ≤ 5% vs RP-405 under identical eval budgets (else fail cost gate even if quality improves).

---

## 3. Arms and FIR

**Arms:** identical to RP-405 indices 0–4 (random, congestion, highcost, bottleneck, ecmp-conflict).

**FIR (minimisation):**

```text
FIR_t = (obj_parent − obj_child) / max(|obj_parent|, ε)
```

- Improvement → FIR > 0  
- No improvement / worse → FIR ≤ 0 (still recorded in window)  
- Empty destroy → skip observe + select again (do not credit)

**FRRMAB defaults (from paper; sensitivity later):**

| Param | Draft default | Notes |
| ----- | ------------- | ----- |
| `W` | 50 | Sliding window length (applications) |
| `D` | 1.0 → also try 0.5 | Rank decay; paper shows sensitivity |
| `C` | 1.0 → also try {0.5, 2.0} | UCB exploration |
| Warm-start | Each arm ≥ 1 use before FRRMAB | Per paper |

---

## 4. Treatments

| ID | Policy | Description |
| -- | ------ | ----------- |
| T0 | FixedUniform | `p_i = 1/5` always |
| T1 | FixedRandom | Always arm 0 |
| T2 | RP405Weights | Existing weight bandit (replay params) |
| T3 | FRRMAB | Credit = FIR+rank+decay; select = UCB on FRR |

All share: RP-401C repair, `k=10`, `iters=50`, `timeout=120s`, seed schedule matching RP-405 where possible.

---

## 5. Instance set and metrics

**Instances:** RP-405 Dataset A cohort (20 instances), same exclusions/notes for setA-17 infeasibility.

**Primary metrics (pre-declared):**

1. Aggregate objective Δ vs RP-403 baseline (same definition as RP-405 report)  
2. Count of instances with finite improvement  
3. Mean wall-clock / instance  

**Secondary:**

4. Final arm-use histogram + Shannon entropy of arm use in last 20% of iterations  
5. Sliding-window FRR traces (diagnostic only)  

**Success (advance):** H1 supported **and** H3 pass, on pre-registered metrics—no post-hoc metric shopping.

**Partial:** H2 without H1 → evidence for complementary preservation research, not E0 pass.

---

## 6. Ablations (only if H1 supported)

| Ablation | Question |
| -------- | -------- |
| A1 Window-only UCB (no rank decay) | Is FIR normalisation enough? |
| A2 Page-Hinkley reset + UCB (DaCosta) | Change-detect vs window |
| A3 FRRMAB without negative FIR logging | Does ignoring failures bias arms? |

---

## 7. Implementation sketch (not started)

1. Extract shared destroy/repair loop from `rp405_adaptive.rs` into a harness with injectable `OperatorSelectionPolicy`.  
2. Implement `FixedUniform`, `FixedRandom`, `Rp405Weights`, `FrrmabPolicy`.  
3. Emit JSONL: `{instance, seed, treatment, iter, arm, fir, obj, t_ms}`.  
4. Analysis notebook/script compares treatments; no production path wiring.

**Forbidden in E0:** changing repair; adding new destroy arms; UltraCrew; ChronoSentiment; deep RL.

---

## 8. Authorization checklist (must all be true before run)

- [ ] This protocol reviewed  
- [ ] Seed list + instance list frozen in an appendix artifact  
- [ ] Harness behind a research binary (not default EvolutionEngine path)  
- [ ] Explicit **run authorization** recorded in RP-002 changelog  
- [ ] No Gate 5 / OF / OFX contact  

Until then: **DESIGN ONLY**.

---

## 9. Relationship to later experiments

| Next | Scope |
| ---- | ----- |
| E1 | Paper-faithful MOEA/D+DE FRRMAB on ZDT/DTLZ/WFG (if E0 warrants) |
| E2 | Multi-criteria credit (Lin-style / Schaffer vector) — RD-1/RD-3 |
| Batch 2 | Across-run / meta selection (Tornede et al.) — after within-run baseline |

---

## Change log

| Version | Date | Notes |
| ------- | ---- | ----- |
| 0.1 | 2026-11-16 | Initial design-only protocol; ROADEF 5-arm portfolio; not authorised to run |

---

*RP-002-E0 v0.1 | Design only · Authority 0%*
