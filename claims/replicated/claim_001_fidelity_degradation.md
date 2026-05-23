# Claim 001: Aggregation Erases Peak Occupancy Geometry

| Field | Description |
| --- | --- |
| **Claim** | Aggregation-induced occupancy erasure is NOT mechanically guaranteed. Under bounded replay configurations, extreme temporal compression (e.g., 217k ticks → 61 ticks) can occur with zero peak occupancy deformation if the underlying geometric structure remains perfectly synchronized. |
| **Evidence** | `scripts/phase2a_degradation_study.py` running against the `2024_etf_approval_1h` universe (Tick vs 1m). Both Tier 0 and Tier 1 emitted exactly 0.0 peak occupancy. |
| **Boundary** | Claim applies specifically to Oscillatory Topology (`osc_50_1.0`) configurations with Rolling Bounds (`rolling_50`) cognition. Validated on BTCUSDT during the January 8 2024 volatility expansion. |
| **Falsification** | This claim is falsified if we find an identical configuration where the temporal compression mechanically forces a massive divergence in occupancy that was structurally impossible in the native tick trace. |

**Status:** Provisional (Null Result)
**Date:** 2026-05-23
**Notes:** Initial degradation study confirms that smoothing chronology does not automatically break topology. The null result is structurally significant; the observatory accurately registered insensitivity rather than manufacturing false geometric divergence.
