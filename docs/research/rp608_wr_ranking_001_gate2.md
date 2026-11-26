# RP-608-WR-RANKING-001 — Gate 2 Assessment

**Date:** 2026-11-06  
**Book:** NSE Estimand A  
**Report:** `artifacts/rp608_econ_edge_diag/rp608_wr_ranking_001.md`  
**Scope:** `docs/research/rp608_wr_ranking_001_scoped.md`  
**Authority:** 0% · baseline frozen · criteria unchanged

## Formal disposition

**Gate 2: DO NOT ADVANCE**  
**Gate 3:** unauthorized

## What this unit established

1. **Oct 28 distinction (locked):** Full eligible WR = **58.55% (161/114)**. The **83.8% (31/6)** figure is the ENTRY-DISCOVERY-002 selected slice (`p_hat ≥ 0.50`), not the session population. Gap to 85% on that slice = **−1.22 pp**.
2. That selected slice is **entirely** `MOM_UP ∩ LOC_MID`. Same thr on weaker sessions does **not** reproduce near-85% (Oct 30: 59.3% n=27; Nov 2: 33.3% n=6).
3. Within-Oct28 ranking AUC ≈ **0.56** — score does not strongly separate winners from losers inside the strong session.
4. Ranking transfer remains weak: discovery AUC **0.63** → holdout **0.51** → holdout ex-Oct28 **0.48** → Nov6 forward **0.47**.
5. §F genuine ranking improvement: **not demonstrated** (F1 AUC transfer fail; F2 predeclared-band forward lift fail).
6. No later untouched sessions in the current join for prospective confirmation.

## Non-claims

- Does **not** abandon the aspirational 85% target.  
- Does **not** treat 70% @ 8.6% or Oct 28’s 83.8% slice as a validated policy.  
- Does **not** authorize threshold re-scans on the contaminated holdout.  
- Does **not** authorize feature mining, OF/stop/target/horizon changes, or live authority > 0%.

## Next (research only)

Accumulate untouched post-holdout sessions; improve ranking transfer and coverage under the frozen §F protocol — not narrower threshold slices.
