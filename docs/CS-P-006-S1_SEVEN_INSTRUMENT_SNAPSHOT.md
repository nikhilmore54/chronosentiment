# CS-P-006-S1 — Seven-instrument research snapshot

**Document type:** Disposable research-universe snapshot  
**Status:** Certified PASS / discovery-ready READY  
**Date:** 2026-08-14  
**Parent:** CS-P-006, CS-P-006-B  
**Identity:** `csp006.research_snapshot.7instrument`  
**Does not:** mutate B3/B4, open B5, reopen G-GATE, start Coralys search (that is CS-P-006-C after B.1)  

`.cursor/rules/chronosentiment-core.mdc`: state at T from bars ≤ T; same inputs → same signatures; outcomes do not construct state.

---

## What this is

A **CS-P-006 research snapshot**, not a B-series evidence dump.

```text
product_validation/CS-P-006/snapshot/20260814T183851Z_7instrument/
```

| Field | Value |
|-------|--------|
| Snapshot identity hash | `c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6` |
| Manifest SHA-256 (`SHA256SUMS`) | `80e5b82fa7c089b487f99deb2b6f064de87e9173bc8b6766ffd8c03cbb04cc1d` |
| Rows | 273 (7 × 39 month-ends) |
| TMV-complete | 273 / 273 |
| Five-instrument mismatches vs CS-P-004-E1-S1 | 0 |
| Duplicate `(instrument, T)` | 0 |
| Temporal bar leaks | 0 |
| Repeated generation | identical identity |

Universe:

| Script | Source of bars |
|--------|----------------|
| HDFCBANK.NS | Copied from CS-P-004-E1-S1 cache (reproducible) |
| ICICIBANK.NS | Copied from CS-P-004-E1-S1 cache |
| INFY.NS | Copied from CS-P-004-E1-S1 cache |
| RELIANCE.NS | Copied from CS-P-004-E1-S1 cache |
| TCS.NS | Copied from CS-P-004-E1-S1 cache |
| IDEA.NS | Fetched into this snapshot cache only |
| MAHABANK.NS | Fetched into this snapshot cache only |

As-of grid: month-end 15:30 UTC, 2021-10 through 2024-12 (39 dates × 7 = 273 rows).

Yahoo cache coverage for all seven names: 2021-08-16 through 2026-08-14 (1239 daily bars each). Reconstruction at each grid T uses bars ≤ T only.

---

## Fidelity checks (all asserted)

* Trend / Momentum / Volatility available at each `(instrument, T)` — 273/273
* `evaluation_timestamp == T`
* reconstruction uses bars with `effective_from ≤ T` only; future bars do not change signatures
* deterministic factor signatures; repeated generation → identical identity hash
* duplicate `(instrument, T)` forbidden
* assessment → `TradingDecision` lineage via explicit `BaselineTrendMappingPolicy` (fixture, not a discovered policy)
* existing five signatures match CS-P-004-E1-S1 `identity_run1.txt`
* no outcomes consumed during state construction
* never writes `chrono_b3_test` / `chrono_b4_test`

`discovery_ready=READY` only because every row has Trend, Momentum, and Volatility **available**.

This certification does **not** freeze TRAIN / VALIDATION / TEST. That is CS-P-006-B.1, which must compute chronological boundaries from this 39-point seven-name coverage — not from G-GATE 55/27/28 and not from CS-P-004 year folds.

---

## Runner

```bash
./run_csp006_7instrument_snapshot.sh
```

Regeneration against the same yahoo_cache directory must reproduce identity `c21ec256133fb63656b35e68c5e1e72b72751ad2fb45f11c12f99ddb34a628c6`.

Engine version remains **`unfrozen-dev`**. No real capital.
