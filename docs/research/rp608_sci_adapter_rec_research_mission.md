# RP-608-SCI-ADAPTER-REC — Research mission statement

**Date:** 2026-11-27 Asia/Calcutta  
**Owner:** Anuj (outline) · worker executes only after outline + package pick  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys no-touch · books never pooled · no Gate 5 from research alone  
**Status of v1:** S1=b side-generator selection is **not working** (hostile soak; modest P1 lift; stack-confounded coverage). Design freeze until Anuj outlines + picks.

---

## Mission (one paragraph)

Determine, from **peer-reviewed journals / transactions / top proceedings only**, which **adapter-owned recommendation architecture** can produce durable **after-cost expectancy** on a **single crypto book** (BTC 15m first; never pooled) when Coralys remains a **sealed white-box oracle**, authority stays **0%**, exits/OF floors stay **Gate-5 locked**, and evaluation is **W/L-first per book** with payoff + friction net required — then name the **smallest sealed package** that tests that architecture without remine, cut-ratchet, or Coralys edits.

---

## Problem the research must solve

Live and offline evidence say the current kernel fails as a product:

1. **S1=b side generation** competes with (rather than filters) Coralys — conflicts with meta-labeling practice (primary side + secondary act/pass).
2. **Coverage is confounded** by inherited Gate2E stack (DI_SKEW / PATH_INVALID), so the selective classifier does not own the risk–coverage curve.
3. **P1 holdout WR lift is small** near base rate → fragile under multiple-testing discipline; not Gate 5 fuel.
4. **Baselines also hostile** (adapter_select, Gate2E B) → heuristic and scientific-v1 paths share the expectancy problem; “tighter Wilson” is not the answer.

The outline must therefore choose **architecture**, not another soft-score.

---

## Success criteria for the outline (what “done” looks like)

Anuj’s outline is complete when it delivers, for each candidate architecture:

| # | Deliverable |
| --- | --- |
| A | **Primary question** in one sentence (falsifiable) |
| B | **Cited venues** (journal / transactions / A\* proceedings) with the claim each citation justifies |
| C | **Role split:** who owns side · who owns act/pass · who owns size (if any) |
| D | **Feature orthogonality rule** (what secondary may see; what is forbidden as re-threshold of primary) |
| E | **Label geometry** (barrier / live plan outcomes vs fixed-horizon; decision-time only) |
| F | **Economic objective** (after-cost selective risk — not 0–1 accuracy) |
| G | **Eval protocol** (chronological firewall, burned holdout, W/L + payoff + net + coverage; HLZ-class hurdles) |
| H | **Negative controls** (Gate2E B, adapter_select, SCI-v1 generator as baseline books — never pooled) |
| I | **Hard non-goals** (Coralys edits, OF ±0.30, traj15, soak cut-ratchet, deep HF models, Gate 5) |
| J | **Recommended next sealed package id** (e.g. ML-1 / SCI-clean / HOLD) with go/no-go gates |

---

## In-scope research themes (outline may reorder / cut)

1. **Meta-labeling** — primary high-recall side (Coralys) + secondary act/pass (adapter); JFDS / JPM line.  
2. **Selective classification / reject option** — who owns abstention; coverage–risk reporting (JMLR / ICAIF).  
3. **Calibration + sparse-proportion inference** — isotonic/Platt; Wilson/Agresti for small n (not soak retunes).  
4. **Multiple testing / false discovery** — HLZ RFS/JF; why modest WR lift ≠ promote.  
5. **Cost-sensitive / switching-cost learning** — after-friction objectives; implicit hold regularization.  
6. **Regime-orthogonal meta features** — vol/session/spread as *meta covariates*, not silent PRIMARY vetoes on the same estimand.  
7. **Labeling** — triple-barrier / live exit geometry vs naive horizon signs.  
8. **Purged CV / PBO** — inside-train selection vs sealed holdout (do not conflate).

---

## Out of scope (do not outline into design)

- Editing `coralys-*` or reinterpreting baseline OF as aggression (OFX boundary).  
- Gate 5 changes to OF ±0.30, stops/targets/holds, Estimand A floors.  
- Pooling BTC/ETH/NSE or pooling books for a “universal” filter.  
- Soft-score remine of Gate2E B / adapter_select mid-soak.  
- Deep nets / GBDT as default HF crypto kernels.  
- Promoting on open MTM / morning marks.

---

## Constraints the outline must preserve

| Constraint | Binding source |
| --- | --- |
| Deterministic, event-driven; no randomness in strategy logic | Core §2 |
| W/L-first **per book**; payoff + net required | Core §3b · Gate 2 v2 |
| Coralys sealed; adapter-owned research surface | Charter · SCI-ADAPTER-REC-001 |
| Chronological train→purge→val→burned holdout | S5 · AFML/PBO hygiene |
| Authority 0% until explicit Gate path | Ops freeze |
| Books never share one corrective | Core §3c |

---

## Mission north star (one line)

**Find the peer-reviewed, after-cost, adapter-owned act/pass (or clean selective) architecture that can beat frozen negative controls on a sealed BTC book — without touching Coralys or Gate 5 — and specify the smallest package that can falsify it.**

---

## Handoff

1. **Anuj** produces the research outline against this mission (sections A–J).  
2. Worker executes literature fill / package draft **only** after outline + explicit package pick (HOLD / ML-1 / ML-2 / DEEPER / other named).  
3. No implementation, no cut change, no twin remine until then.

*End — research mission for `RP-608-SCI-ADAPTER-REC-001` · 2026-11-27 Asia/Calcutta.*
