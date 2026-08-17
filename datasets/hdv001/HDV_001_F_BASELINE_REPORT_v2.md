# HDV-001-F Baseline Comparison Report (v2 — Corrected Run)

**Run date:** 2026-08-17 08:06 UTC
**Status:** FAIL
**Corrections applied:** Rule A (directional geometry) + Rule B (momentum history)

---

## Study Population

| Metric | Value |
|--------|-------|
| Total decisions | 1144 |
| COMPLETE (resolved) | 728 |
| MATURING (excluded) | 416 |

---

## Results

| Baseline | TARGET_HIT | N | Rate | Margin vs Coralys | Criterion (>=5 pp) |
|----------|-----------|---|------|-------------------|-------------------|
| **Coralys** | 260 | 728 | **35.7%** | — | — |
| A — Random | 210 | 728 | 28.8% | +6.9 pp | PASS |
| B — Inverse | 164 | 728 | 22.5% | +13.2 pp | PASS |
| C — Momentum | 248 | 728 eligible | 34.1% | +1.6 pp | FAIL |

*Baseline C: 0 decisions skipped (insufficient 20-session history — Rule B, zero random fallback)*

---

## Methodology Notes

**Rule A — Directional geometry (corrected):**
For each baseline decision, target and stop prices are reconstructed from
`reference_price +/- |original_distance|` in the baseline direction. Coralys's
absolute target/stop prices are never reused for a direction-flipped baseline.

**Rule B — Momentum history (corrected):**
Baseline C uses `hdv001_baseline_history_v1` (2026-06-01 to 2026-07-13) combined
with primary cache bars as pre-decision lookback. Decisions with fewer than 20
sessions of history are skipped entirely. Zero random fallback.

**Price path evaluation:**
Bar-by-bar walk using high/low (not close-only) to detect target/stop crossing.

---

## Conclusion

Coralys does not exceed all three mechanical baselines by >= 5 percentage points. HDV-001-F criterion: FAIL.
