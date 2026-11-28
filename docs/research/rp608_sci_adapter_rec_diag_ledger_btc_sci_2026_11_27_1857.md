# BTC SCI diagnostic ledger snapshot — ~18:57 Asia/Calcutta

**Tool:** `scripts/research/rp608_sci_adapter_rec_diag_ledger.py`  
**Book:** stacked `CRYPTO_PLAN_SCI_ADAPTER_REC` only (never pooled)  
**Scorecard UTC:** 2026-11-27T13:27:45Z → **18:57 Asia/Calcutta**  
**Charter:** diagnose-first · `.cursor/rules/chronosentiment-core.mdc` §3b

## Stages (partial — S0/S2 incomplete)

| Stage | Result |
| --- | --- |
| S1 upstream suppress total | **825** |
| — DI_SKEW | 87 |
| — PATH_INVALID | 60 |
| — OPEN_PLAN | 678 |
| S3 PLAN_CREATED | 11 |
| S4/S5 closed W/L | **0/10** (open=1) |
| S6 gross / friction | −8.62 / **+11.99** (avg fric 1.20) |
| S7 net | **−30.18** · net/plan −2.74 |

## Ownership reading (hypothesis)

Upstream stack (DI_SKEW + PATH + OPEN_PLAN) dominates event volume vs PLAN_CREATED. Among the few accepted+closed plans, **all losses**; friction exceeds |gross|. Supports P1a (selection ownership) as first *conditional* next package — without retuning SCI cuts. S0/S2 still need decision-card join.

## P4 soak

Still **`HOLD_SOAK`** — continue frozen stacked SCI; no cut ratchet; SCI_CLEAN not armed.

---

*End — diag ledger BTC SCI · 2026-11-27 Asia/Calcutta.*
