# CS-P-001-E — Snap-to-fill clock (Deferred Live)

**Document type:** Product observation  
**Status:** Frozen — clock hypothesis rejected for this book; do not amend  
**Date:** 2026-09-15  
**Parent:** CS-P-001-D (frozen; not amended)  
**Does not mutate:** `deferred_live.rs`, T0 geometry, TARGET / STOP / HORIZON, INVERT, reassessment  
**Does not claim:** G-GATE predictive value  

`.cursor/rules/chronosentiment-core.mdc`: deterministic as-of events; no invented prices.

Question: is adverse paper fill a consequence of the DecisionBrief going stale between `snap_unix` and the first Deferred Live fill?

---

## 1. What was measured

Join only. No re-walk of the lifecycle.

```text
DecisionBrief.execution.snap_unix
        vs
first cached 1m bar on the session date
        vs
paper_entry_price (CS-P-001-D ledger)
```

Fill time is reconstructed as the first IST-session 1m close. That reconstruction matches all 34 ledger fills exactly. The universe-audit JSON did not store `opened_at`; the cache close is the observed fill.

`lag_secs = fill_unix − snap_unix`. Negative means the fill occurred before the recorded snap.

---

## 2. Result

| Fact | Value |
| --- | --- |
| Names | **34 / 34** |
| First-bar close = paper fill | **34 / 34** |
| Fill clock | **09:15 IST** (session open) |
| Snap clock | **15:30 IST** (`10:00:00Z` — LIVE-005 `1000` is UTC, not IST) |
| Order | **34 FILL_BEFORE_SNAP, 0 FILL_AFTER_SNAP** |
| Lag | **−6.25 h on every name** |
| Unique lags | **1** |
| Lag → displacement dose response | **Not identifiable** |

Adverse and favorable fills share the same lag. TCS STOP −2.18% and IDEA OPEN +1.49% both wait −6.25 h. Staleness-after-snap cannot explain why some fills are severe and some are not.

LIVE-005 decision ids labeled `…-1000` are **10:00 UTC / 15:30 IST cash close**, not 10:00 IST.

---

## 3. Where the distortion enters

```text
LIVE-005 snap  10:00 UTC = 15:30 IST  (close)
DecisionBrief.entry_price / adaptive_risk   (close-as-of)
        vs
Deferred Live session auto-arm
first CACHED_1M bar  09:15 IST  same cohort date
paper_entry_price = 09:15 close
```

This is **clock inversion on the cohort date**, not a decision that aged while waiting for a later fill.

That also explains the CS-P-001-D horizon fact: horizon unix is `snap_unix + 300 minutes` = 20:30 IST, after the 1m tape ends at 15:15.

---

## 4. Verdict

1. CS-P-001-D remains frozen. Do not retune stops from this clock measurement.
2. Adverse fill is **not** measured here as post-snap staleness. Lag has no variance.
3. Distortion enters because session auto-arm fills at **IST open** against a brief snapped at **IST close**.
4. Do not shift `snap_unix`, do not delay PAPER_ENTER to snap, and do not rebase risk. Those would be product-clock changes, not this observation.

Artifact: `datasets/deferred_live_snap_fill.json`.
