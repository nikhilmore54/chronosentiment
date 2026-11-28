# ELR-SCI-CLEAN-OBSERVE-002 — paired new closes (2026-11-28 ~10:34 Asia/Calcutta)

**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · twins frozen · no force differential · no pooling / remine / Coralys / Gate 5  
**Board at record:** stacked SCI **1/43** net **−126.71** · SCI_CLEAN **0/6** net **−14.40** · Gate2E B 0/35 −95.44

---

## Anuj lock

Paired Δnet **−$5.62** on the interval stacked 1/41→1/43 and CLEAN 0/4→0/6.  
Interpretation: sum of the **two new closed nets** (−4.37 + −1.25), identical on both books — **not** a CLEAN−stacked treatment delta.

---

## (A) New closed plan IDs — identical events?

| plan_id | On stacked? | On CLEAN? | Identical exit / gross / fric / net? |
| --- | --- | --- | --- |
| `PLAN-317541-LONG-1795838551607` | Yes | Yes | **Yes** (all fields match) |
| `PLAN-317622-LONG-1795840715969` | Yes | Yes | **Yes** (all fields match) |

`clean_only` plan_ids: **∅** · post-arm created overlap includes these two · still synchronized population.

---

## (B) Per-close economics + failure class

| plan_id | side | exit | gross | fric | net | peak MFE | primary class |
| --- | --- | --- | --- | --- | --- | --- | --- |
| …8551607 | LONG | STOP_LOSS_HIT | −1.97 | 1.1988 | **−4.37** | 0.10 | **DIRECTIONAL_FAILURE** |
| …0715969 | LONG | GATE2X_MFE_GIVEBACK_PROTECT | −0.05 | 1.2000 | **−1.25** | 1.31 | **EXECUTION_FRICTION** (fric > \|gross\|; path MFE cleared fric+$1 — timing co-tag) |

Pair sum net = **−5.62** (both books). Between-book Δnet = **0.00**.

---

## (C) DI / regime differential

| Scope | DI_SKEW | PATH_INVALID | REGIME |
| --- | --- | --- | --- |
| Stacked post-arm | **0** | **0** | **0** |
| CLEAN lifetime | **0** | **0** | — |

**DI/regime differential: still unobserved.** Milestone unmet. Do not force.

---

## ELR table

| Field | Value |
| --- | --- |
| Experiment ID | `ELR-SCI-CLEAN-OBSERVE-002` |
| Result | **Inconclusive** (paired closes identical; no treatment contrast) |
| Treatment differentiation | **Not observed** |
| Learning | New losses continue friction + directional mix on **shared** plans; −$5.62 is same-book pair sum, not CLEAN advantage/disadvantage |
| Next observation | First valid DI/regime-alignment differential event |
| Decision | **hold** both DNA · no new sealed package · no corrective |

---

*End — ELR-SCI-CLEAN-OBSERVE-002 · 10:34 Asia/Calcutta 2026-11-28.*
