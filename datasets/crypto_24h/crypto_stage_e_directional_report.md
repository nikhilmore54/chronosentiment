# Stage E — Directional Decomposition of H-VA2b

- **State:** LOW_PERSISTENCE (<= 0.1129)
- **Horizon:** 300m
- **Prospective Dataset N:** 23308

## 1. Signed Directional Behaviour
| Metric | VA_HIGH | VA_NORMAL | Δ |
|---|---|---|---|
| N | 4021 | 19287 | - |
| Positive-Return Freq (Hit Rate) | 61.2% | 52.3% | 8.8% |
| Negative-Return Freq | 38.8% | 47.6% | -8.8% |
| Expected Signed Value (Mean) | 21.6 bps | 7.3 bps | **14.3 bps** |
| LONG-Direction Expected Return | 66.2 bps | 63.6 bps | 2.6 bps |
| SHORT-Direction Expected Loss | -48.6 bps | -54.6 bps | 6.0 bps |

## 2. Magnitude (Absolute Expansion) Behaviour
| Metric | VA_HIGH | VA_NORMAL | Δ |
|---|---|---|---|
| Mean Absolute Return | 59.4 bps | 59.3 bps | 0.0 bps |
| Median Absolute Return | 37.0 bps | 38.2 bps | -1.2 bps |

## 3. Signed Return Distribution
| Quantile | VA_HIGH | VA_NORMAL | Δ |
|---|---|---|---|
| Q10 | -66.0 bps | -84.9 bps | 18.9 bps |
| Q25 | -21.6 bps | -37.1 bps | 15.5 bps |
| Q50 | 10.6 bps | 2.9 bps | 7.7 bps |
| Q75 | 52.0 bps | 39.3 bps | 12.7 bps |
| Q90 | 106.2 bps | 100.2 bps | 6.1 bps |

## 4. 1% Absolute-Return Tail Sensitivity
Removing the largest 1% of absolute returns from each respective subgroup to test extreme tail reliance.

| Metric | VA_HIGH | VA_NORMAL | Δ |
|---|---|---|---|
| N Removed | 41 | 193 | - |
| Raw Mean Return | 21.6 bps | 7.3 bps | 14.3 bps |
| Trimmed Mean Return | 16.5 bps | 3.0 bps | 13.5 bps |
| Loss to Trimming (Raw - Trim) | 5.1 bps | 4.3 bps | - |
