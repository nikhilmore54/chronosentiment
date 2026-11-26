# Phase 5A: Environmental Response Surfaces

We mapped the `event_reset` execution trace persistence against the continuous geometry of the environment to determine how replay behavior changes across the geometry.

## 1. Linear vs. Non-Linear (Quadratic) Fits
| Predictor | Linear $R^2$ | Linear p-value | Quadratic $R^2$ | Quad Term p-value | AIC Improvement (Quad over Lin) |
|-----------|-------------|----------------|----------------|-------------------|---------------------------------|
| ecological_position | 0.1244 | 1.48e-08 | 0.1283 | 2.97e-01 | -0.9 |
| realized_volatility | 0.2878 | 1.39e-19 | 0.2884 | 6.60e-01 | -1.8 |
| trend_strength | 0.0066 | 2.05e-01 | 0.0231 | 4.52e-02 | 2.1 |
| session_range_pct | 0.1593 | 9.51e-11 | 0.1620 | 3.80e-01 | -1.2 |

## 2. Key Findings
- **Ecological Position**: We projected the 4D state `[volatility, trend, range, return]` onto the Ward separation vector. The response surface shows highly significant coupling.
- **Dominant Driver**: `realized_volatility` explains the most variance in execution persistence.
- **Non-Linearities (Saturation/Thresholds)**: We detected statistically significant non-linear behavior in the following dimensions:
  - `trend_strength` (Quad term p = 4.52e-02)
This implies a saturation point or threshold effect in the execution response surface.
