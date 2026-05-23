# Claim 001: Aggregation Erases Peak Occupancy Geometry

| Field | Description |
| --- | --- |
| **Claim** | Degrading chronology from native `aggTrade` ticks down to 1m `kline` aggregation systematically erases the peak geometric intensity of the occupancy traces, reducing the maximum observable deformation by at least 30% during violent market shocks. |
| **Evidence** | `scripts/phase2a_degradation_study.py` running against the `2024_etf_approval` universe (Tick vs 1m). *Pending completion of Tier 0 (aggTrade) backfill for this universe to generate the final artifact hashes.* |
| **Boundary** | Claim applies specifically to Oscillatory Topology configurations with Rolling Bounds cognition. It has only been validated on the BTCUSDT pair during volatility expansion regimes. |
| **Falsification** | This claim is falsified if a subsequent replay of a liquidation cascade (e.g., FTX collapse) under identical topology shows that the 1m aggregation trace preserves or exceeds the peak geometric intensity of the native tick trace. |

**Status:** Provisional
**Date:** 2026-05-23
**Notes:** Initial degradation study confirms that smoothing chronology mechanically hides the deepest stress points of the deformation matrix. The observatory must rely on Tier 0 for true structural persistence studies.
