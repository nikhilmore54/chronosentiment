# Stage D — Prospective Validation of H-VA2

## Provenance Record
- **First Prospective Timestamp:** 2026-08-17 00:00:00 UTC
- **Last Available Timestamp:** 2026-09-17 20:01:00 UTC
- **Total Complete Observations in Window:** 45842
- **Excluded (Incomplete 120m Horizon):** 120
- **Excluded (Incomplete 300m Horizon):** 300
- **FROZEN VA_HIGH Threshold:** 1.4638
- **FROZEN Persistence Threshold:** 0.1129

### H-VA2a: High Persistence (120m) -> Expected LOWER Return
| Metric | VA_HIGH | VA_NORMAL | Δ (High - Norm) |
|---|---|---|---|
| N | 6088 | 16221 | - |
| Mean Ret | 8.2 bps | 5.0 bps | **3.2 bps** |
| Med Ret | 2.9 bps | 1.4 bps | 1.5 bps |
| Hit Rate | 53.3% | 51.5% | 1.8% |
| 95% CI of Δ | - | - | [1.4, 5.1] bps |

**Block Stability Analysis**
- **Expected Sign:** Negative
- **Median Block Δ:** 5.9 bps
- **Sign Stability:** 20%
- **10 Block Δ Sequence:** -9.0, 13.3, 8.2, 0.4, 15.7, 6.8, 1.2, 5.3, 6.5, -7.7

### H-VA2b: Low Persistence (300m) -> Expected HIGHER Return
| Metric | VA_HIGH | VA_NORMAL | Δ (High - Norm) |
|---|---|---|---|
| N | 4021 | 19282 | - |
| Mean Ret | 21.6 bps | 7.3 bps | **14.3 bps** |
| Med Ret | 10.6 bps | 2.8 bps | 7.7 bps |
| Hit Rate | 61.2% | 52.3% | 8.8% |
| 95% CI of Δ | - | - | [11.2, 17.5] bps |

**Block Stability Analysis**
- **Expected Sign:** Positive
- **Median Block Δ:** 15.5 bps
- **Sign Stability:** 90%
- **10 Block Δ Sequence:** 36.2, 27.7, 30.7, 7.3, 14.2, -4.0, 1.0, 15.3, 15.7, 26.8

