# RP-608-SCI-ADAPTER-REC — Package ML-1D  
## Dual-Dimension Recommendation Intelligence

**Date:** 2026-11-27 Asia/Calcutta  
**Status:** PRIMARY RESEARCH HYPOTHESIS · **not authorized for implementation**  
**Author:** Anuj (architecture + literature) · worker formalization  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · books never pooled · no Gate 5  
**Supersedes as hypothesis:** single-dimension ML-1 (meta-label act/pass alone) — retained as nested special case / comparator, **not frozen**  
**Quantitative evidence (do not convert to cuts):** `docs/research/rp608_sci_adapter_rec_quantitative_evidence_register.md`

---

## 0. One-line claim

Improve the **use** of sealed Coralys intelligence with two **independent** adapter capabilities — **recommendation reliability** and **economic actionability** — then combine them in a deterministic ACT/PASS policy. Do **not** replace Coralys side, size, or exits.

---

## 1. Role split (C)

| Role | Owner | Authority |
| --- | --- | --- |
| Primary intelligence (side / recommendation) | **Coralys** (sealed) | Unchanged |
| Reliability intelligence | Adapter | Observational estimates only |
| Economic intelligence | Adapter | Observational estimates only |
| Decision | Adapter | **ACT / PASS only** |
| Sizing / exits / OF floors | Frozen baselines | **0% adapter authority** |

---

## 2. Two dimensions (must not collapse)

| Dimension | Measures | Primary output |
| --- | --- | --- |
| **Reliability** | Predictive correctness, calibration, uncertainty, evidence sufficiency, regime / drift sensitivity | Reliability estimate \(Q_t\) with uncertainty |
| **Economic actionability** | Expected payoff after fees, spread, slippage; liquidity; cost-sensitive utility | Conditional net utility \(U_t\) with uncertainty |

Four-quadrant cases (all first-class):

1. Reliable ∧ economically unattractive → **PASS**  
2. Uncertain ∧ economically attractive (payoff asymmetry) → policy must declare evidence rule  
3. Reliable ∧ economically attractive → **ACT** candidate  
4. Neither → **PASS**

**Anti-double-counting:** \(Q_t\) may enter the construction of \(U_t\); do **not** treat the same probability mass as two independent pieces of evidence in the ACT gate.

---

## 3. Deterministic policy (sketch — not live cuts)

\[
ACT_t =
\begin{cases}
1 & \text{if reliability evidence is sufficient (predeclared)} \\
  & \text{and } U_t > 0 \\
0 & \text{otherwise}
\end{cases}
\]

Evidence sufficiency is a **predeclared** uncertainty / coverage / sample policy — not an arbitrary soft-score threshold remine.

---

## 4. Learning target & objective (E, F)

| Item | Spec |
| --- | --- |
| Labels | Frozen-plan **realized** outcomes (live barrier / plan geometry) |
| Economic objective | After-cost expectancy (W/L-first per book + payoff + friction net) |
| Book | BTC 15m first; never pool ETH/NSE |
| Features | Decision-time only; economic features = friction/liquidity context, not a second direction model |

---

## 5. Experimental comparison (must run before claiming complementarity)

Identical eligible event population · chronological splits · frozen negatives (Gate2E B, adapter_select, SCI-v1).

| Experiment | Reliability | Actionability | Purpose |
| --- | --- | --- | --- |
| **E0** | No | No | Frozen primary baseline |
| **E1** | Yes | No | Reliability alone |
| **E2** | No | Yes | Economic alone |
| **E3** | Yes | Yes | Incremental value of **combination** vs E1/E2 (not only vs E0) |

Component ladder (dev-set first):

| ID | Technique | Gate to next |
| --- | --- | --- |
| R1 | Probability calibration | Useful calib before conformal stack |
| R2 | Adaptive conformal / drift-aware uncertainty | Extension after R1 |
| R3 | Selection-aware calibration (post-ACT subset) | Extension; validity of selected statements |
| Ec1 | Conditional net utility | Core economic |
| Ec2 | Liquidity-conditioned utility | Incremental over Ec1 |
| C1 | Dual combine | Only if components show useful evidence |

---

## 6. Hard non-goals (I)

- Coralys edits · side replacement · sizing · exit / OF ±0.30 / traj15 changes  
- Gate 5 from this package alone  
- One generic confidence score replacing both dimensions  
- Soak cut-ratchet · remine · book pooling · promote on open MTM  

---

## 7. Go / no-go before implementation (J)

| Gate | Requirement |
| --- | --- |
| G0 | Historical data sufficiency + decision-time feature availability verified |
| G1 | Strengthening literature pass complete (dependent-series conformal · conditional predictive inference · cost-aware DRO) — see part 3 |
| G2 | Explicit Anuj package **SELECT** (ML-1D) — not implied by hypothesis |
| G3 | Sealed offline package design with E0–E3 protocol; no Pi arm until offline gates |

**Current state:** Hypothesis **accepted as primary**. Implementation **blocked** pending G0–G2.

---

## 8. A–J checklist (outline complete)

| # | Status |
| --- | --- |
| A Primary question | ✓ Dual-dimension around sealed Coralys |
| B Citations | ✓ Anuj findings + part 3 strengtheners |
| C Role split | ✓ Coralys side · adapter ACT/PASS |
| D Orthogonality | ✓ Reliability ≠ re-threshold Coralys score; economic = friction/payoff, not second side model |
| E Labels | ✓ Frozen-plan realized outcomes |
| F Economic objective | ✓ After-cost \(U_t\) |
| G Eval | ✓ Chronological · E0–E3 · W/L+payoff+net · HLZ caution |
| H Negatives | ✓ B · adapter_select · SCI-v1 |
| I Non-goals | ✓ Listed |
| J Package id | ✓ **ML-1D** (await SELECT) |

---

*End — ML-1D dual-dimension package · 2026-11-27 Asia/Calcutta.*
