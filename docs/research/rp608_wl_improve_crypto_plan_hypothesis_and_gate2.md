# RP-608-WL-IMPROVE-001 — Hypothesis Specification & Gate 2 Disposition (Crypto Plan)

**Date:** 2026-11-06  
**Inputs:**  
- Phase 2 decision-time study: `artifacts/rp608_econ_edge_diag/wl_phase2_crypto_plan_entry_quality_2026-11-03_to_05.md`  
- Replication: `artifacts/rp608_econ_edge_diag/wl_phase2_crypto_plan_replication_beyond_2026-11-03_to_05.md`  
**Authority:** 0.0% · baseline frozen  

## Sequence position

1. Gate 1 — PASS  
2. Phase 2 Crypto plan (decision-time) — DONE  
3. Cross-session replication beyond 3–5 Nov — DONE  
4. Hypothesis specification — **this document**  
5. Gate 2 review — **this document**  

## Replication outcome (primary: holdout pre ex-2026-10-29)

| Association (discovery) | Holdout verdict |
| --- | --- |
| LONG WR > SHORT WR | **FAILS_REPLICATION** (gap +20.9pp → −1.3pp) |
| OF_SUP=True > False | **FAILS_REPLICATION** (gap +13.4pp → −19.2pp) |
| LONG\|OF_SUP=True > SHORT\|OF_SUP=True | **INCONCLUSIVE_UNDERPOWERED** (SHORT\|OF side n=7) |
| ABS_IMB_≥0.70 > \<0.30 | **FAILS_REPLICATION** (gap flips) |
| ATR_REL_\<20bps > 40–80bps | **FAILS_REPLICATION** (gap flips) |

**REPLICATES count: 0.**

Discovery-window patterns (including the striking LONG vs SHORT gap on 3–5 Nov) do **not** establish stable decision-time entry characteristics that reduce loss probability out of sample.

## Hypothesis specification

**No falsifiable entry-quality hypothesis is specified for Gate 2.**

Rationale: step 4 requires a candidate that survived replication. Promoting a failed or underpowered discovery association would violate the stage success criterion (defensible, *repeatable* explanation of entry win probability — not slice-specific WR).

### Explicitly rejected candidate forms

- “Prefer LONG over SHORT at entry” — failed holdout  
- “Require OF_SUP=True” — failed holdout (and would not authorize OF ±0.30 changes regardless)  
- “Require |imbalance| ≥ 0.70” — failed holdout  
- Hour-based selection — already flagged single-session dominant in Phase 2; not advanced  

## Gate 2 review

| Question | Decision |
| --- | --- |
| Does evidence support proposing an offline entry-quality experiment? | **NO** |
| Gate 3 offline experiment | **Remains unauthorized** |
| Baseline / OF ±0.30 / H0 / stops / targets | **Unchanged** |

### Gate 2 formal disposition

**Gate 2: DO NOT ADVANCE** (no experiment proposal).

This is a completed negative result for the current decision-time feature set on Crypto plan, not a license to invent filters from the discovery window alone.

## What this does *not* claim

- Does not claim Crypto plan is unimprovable forever  
- Does not reopen NSE A / NSE OF / Crypto OF Phase 2 automatically  
- Does not authorize using post-entry MFE/MAE as entry predictors  
- Does not authorize OFX Stage 2 or Step 3+ (`.cursor/rules/rp608-ofx.mdc`)

## Allowed research continuations (optional; not Gate 3)

1. Accumulate more OF-era sessions and re-test only underpowered H_DIR_OF with predeclared n floors  
2. Reconstruct look-ahead-safe trend/momentum/price-location features (currently UNAVAILABLE) and run a **new** Phase 2 — new discovery window, not re-mining 3–5 Nov  
3. Keep session W/L monitoring under frozen baseline  

## Success criterion check

Stage asked for a defensible, repeatable explanation of entry win probability.  
**Result:** current recorded decision-time fields do **not** provide one that replicates beyond 3–5 November. That is a valid research outcome.
