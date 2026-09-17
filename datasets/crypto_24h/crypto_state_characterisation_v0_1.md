# Crypto Expanded State v0.1: Stage B Characterisation
Evaluation of the 3 surviving distinct dimensions on the Train split.

## 1. Distribution / Regime Structure
| Feature | Q10 | Q25 | Q50 | Q75 | Q90 | % > Q90 | % < Q10 |
|---|---|---|---|---|---|---|---|
| `volume_acceleration_60m` | 0.4724 | 0.6803 | 0.9748 | 1.4638 | 2.2243 | 10.0% | 10.0% |
| `price_volume_alignment_60m` | -0.2905 | -0.1330 | 0.0192 | 0.1951 | 0.3531 | 10.0% | 10.0% |
| `wick_asymmetry_60m` | 0.3470 | 0.4217 | 0.5022 | 0.5859 | 0.6755 | 10.0% | 10.0% |

## 2. Temporal Behaviour
| Feature | ACF (1m) | ACF (15m) | ACF (60m) | ACF (240m) | > Median Episodes | > Median Duration (Mean) | > Median Duration (Med) | < Median Episodes | < Median Duration (Mean) | < Median Duration (Med) |
|---|---|---|---|---|---|---|---|---|---|---|
| `volume_acceleration_60m` | 0.9750 | 0.3547 | -0.0461 | -0.0386 | 606 | 20.7 | 14.0 | 605 | 20.7 | 15.0 |
| `price_volume_alignment_60m` | 0.9774 | 0.7208 | 0.1076 | 0.1054 | 424 | 29.6 | 10.0 | 424 | 29.6 | 11.0 |
| `wick_asymmetry_60m` | 0.9806 | 0.7178 | 0.0258 | -0.0567 | 474 | 26.4 | 7.0 | 474 | 26.4 | 6.0 |

## 3. Interaction with Existing State
### Volume Acceleration x Volatility x Trend
| Volatility | Trend Dir | Median Volume Acceleration | N |
|---|---|---|---|
| HIGH | UP | 0.9771 | 7075 |
| HIGH | DOWN | 0.9685 | 5456 |
| LOW | UP | 0.9944 | 7006 |
| LOW | DOWN | 0.9533 | 5523 |

### Wick Asymmetry x Trend Dir x Persistence
| Trend Dir | Persistence | Median Wick Asymmetry | N |
|---|---|---|---|
| UP | HIGH | 0.5226 | 7021 |
| UP | LOW | 0.5175 | 7060 |
| DOWN | HIGH | 0.4690 | 5509 |
| DOWN | LOW | 0.4871 | 5470 |

## 4. Cross-Dimensional Events
Examining the temporal clustering of predefined composite states (using Q75/Q25 bounds).

| Composite Event | Episodes | Event Freq (N) | Expected N (Independent) | Mean Duration (mins) | Median Duration (mins) |
|---|---|---|---|---|---|
| High Vol Accel (>Q75) + Strong Trend (>Q75 Pers) | 321 | 2178 | 1566.4 | 6.8 | 3.0 |
| High Vol Accel (>Q75) + Weak Trend (<Q25 Pers) | 362 | 1180 | 1566.4 | 3.3 | 2.0 |
| High Vol Accel (>Q75) + High Volatility (>Q75) | 104 | 1382 | 1566.4 | 13.3 | 9.0 |
| High Wick Asym (>Q75) + Strong Trend (>Q75 Pers) | 264 | 1895 | 1566.4 | 7.2 | 2.5 |
| High Wick Asym (>Q75) + Weak Trend (<Q25 Pers) | 402 | 1360 | 1566.4 | 3.4 | 2.0 |
