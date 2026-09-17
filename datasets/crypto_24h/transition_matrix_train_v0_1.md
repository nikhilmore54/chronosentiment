# Crypto 24H State Transition Research v0.1
## Stage A: Predeclared Transition/Outcome Matrix (Train Split)

This matrix evaluates structural deviations from the baseline probability for each behavioural target. `Block_Stability` represents the percentage of 12h chronological blocks where the conditional mean correctly deviated from the baseline mean in the direction of the aggregate deviation.

### Target: Reversal
| Horizon | Transition_Type | Transition | N | Baseline_% | Cond_% | Diff | Block_Stability |
|---|---|---|---|---|---|---|---|
| 60m | Volatility | LOW -> HIGH | 1566 | 51.8% | 58.4% | +6.6% | 75.0% |
| 60m | Volatility | HIGH -> LOW | 1334 | 51.8% | 41.8% | -9.9% | 70.0% |
| 60m | Volatility | MAINTAIN_HIGH | 10785 | 51.8% | 53.3% | +1.5% | 54.5% |
| 60m | Volatility | MAINTAIN_LOW | 11017 | 51.8% | 50.5% | -1.2% | 52.2% |
| 60m | Direction | UP -> DOWN | 2567 | 51.8% | 47.3% | -4.5% | 58.8% |
| 60m | Direction | DOWN -> UP | 2567 | 51.8% | 51.1% | -0.7% | 37.5% |
| 60m | Direction | MAINTAIN_UP | 11152 | 51.8% | 51.3% | -0.5% | 56.0% |
| 60m | Direction | MAINTAIN_DOWN | 8412 | 51.8% | 54.0% | +2.2% | 60.9% |
| 120m | Volatility | LOW -> HIGH | 1566 | 54.7% | 66.9% | +12.2% | 75.0% |
| 120m | Volatility | HIGH -> LOW | 1334 | 54.7% | 48.0% | -6.8% | 70.0% |
| 120m | Volatility | MAINTAIN_HIGH | 10785 | 54.7% | 54.3% | -0.5% | 54.5% |
| 120m | Volatility | MAINTAIN_LOW | 11017 | 54.7% | 54.3% | -0.5% | 47.8% |
| 120m | Direction | UP -> DOWN | 2567 | 54.7% | 58.4% | +3.7% | 52.9% |
| 120m | Direction | DOWN -> UP | 2567 | 54.7% | 50.0% | -4.7% | 62.5% |
| 120m | Direction | MAINTAIN_UP | 11152 | 54.7% | 53.6% | -1.1% | 52.0% |
| 120m | Direction | MAINTAIN_DOWN | 8412 | 54.7% | 56.6% | +1.8% | 60.9% |
| 300m | Volatility | LOW -> HIGH | 1566 | 54.2% | 74.6% | +20.5% | 62.5% |
| 300m | Volatility | HIGH -> LOW | 1334 | 54.2% | 60.0% | +5.8% | 60.0% |
| 300m | Volatility | MAINTAIN_HIGH | 10785 | 54.2% | 47.9% | -6.2% | 59.1% |
| 300m | Volatility | MAINTAIN_LOW | 11017 | 54.2% | 56.6% | +2.5% | 56.5% |
| 300m | Direction | UP -> DOWN | 2567 | 54.2% | 58.4% | +4.3% | 64.7% |
| 300m | Direction | DOWN -> UP | 2567 | 54.2% | 50.6% | -3.5% | 43.8% |
| 300m | Direction | MAINTAIN_UP | 11152 | 54.2% | 53.1% | -1.0% | 44.0% |
| 300m | Direction | MAINTAIN_DOWN | 8412 | 54.2% | 55.3% | +1.2% | 73.9% |

### Target: Vol_expansion
| Horizon | Transition_Type | Transition | N | Baseline_% | Cond_% | Diff | Block_Stability |
|---|---|---|---|---|---|---|---|
| 60m | Volatility | LOW -> HIGH | 1566 | 34.3% | 35.1% | +0.8% | 50.0% |
| 60m | Volatility | HIGH -> LOW | 1334 | 34.3% | 32.9% | -1.4% | 50.0% |
| 60m | Volatility | MAINTAIN_HIGH | 10785 | 34.3% | 26.6% | -7.7% | 63.6% |
| 60m | Volatility | MAINTAIN_LOW | 11017 | 34.3% | 41.8% | +7.5% | 69.6% |
| 60m | Direction | UP -> DOWN | 2567 | 34.3% | 38.3% | +4.1% | 52.9% |
| 60m | Direction | DOWN -> UP | 2567 | 34.3% | 41.1% | +6.8% | 56.2% |
| 60m | Direction | MAINTAIN_UP | 11152 | 34.3% | 36.0% | +1.7% | 48.0% |
| 60m | Direction | MAINTAIN_DOWN | 8412 | 34.3% | 28.7% | -5.6% | 56.5% |
| 120m | Volatility | LOW -> HIGH | 1566 | 36.8% | 34.3% | -2.5% | 62.5% |
| 120m | Volatility | HIGH -> LOW | 1334 | 36.8% | 31.9% | -4.9% | 50.0% |
| 120m | Volatility | MAINTAIN_HIGH | 10785 | 36.8% | 28.0% | -8.8% | 54.5% |
| 120m | Volatility | MAINTAIN_LOW | 11017 | 36.8% | 46.4% | +9.6% | 65.2% |
| 120m | Direction | UP -> DOWN | 2567 | 36.8% | 42.6% | +5.8% | 58.8% |
| 120m | Direction | DOWN -> UP | 2567 | 36.8% | 44.6% | +7.8% | 62.5% |
| 120m | Direction | MAINTAIN_UP | 11152 | 36.8% | 37.5% | +0.7% | 48.0% |
| 120m | Direction | MAINTAIN_DOWN | 8412 | 36.8% | 31.8% | -5.0% | 47.8% |
| 300m | Volatility | LOW -> HIGH | 1566 | 39.1% | 36.5% | -2.7% | 62.5% |
| 300m | Volatility | HIGH -> LOW | 1334 | 39.1% | 21.7% | -17.4% | 80.0% |
| 300m | Volatility | MAINTAIN_HIGH | 10785 | 39.1% | 28.2% | -11.0% | 72.7% |
| 300m | Volatility | MAINTAIN_LOW | 11017 | 39.1% | 52.4% | +13.2% | 60.9% |
| 300m | Direction | UP -> DOWN | 2567 | 39.1% | 50.4% | +11.3% | 52.9% |
| 300m | Direction | DOWN -> UP | 2567 | 39.1% | 40.2% | +1.1% | 37.5% |
| 300m | Direction | MAINTAIN_UP | 11152 | 39.1% | 42.2% | +3.0% | 48.0% |
| 300m | Direction | MAINTAIN_DOWN | 8412 | 39.1% | 31.3% | -7.8% | 56.5% |

### Target: Excursion_exceeded
| Horizon | Transition_Type | Transition | N | Baseline_% | Cond_% | Diff | Block_Stability |
|---|---|---|---|---|---|---|---|
| 60m | Volatility | LOW -> HIGH | 1566 | 1.4% | 3.8% | +2.4% | 12.5% |
| 60m | Volatility | HIGH -> LOW | 1334 | 1.4% | 0.4% | -1.0% | 90.0% |
| 60m | Volatility | MAINTAIN_HIGH | 10785 | 1.4% | 0.6% | -0.8% | 90.9% |
| 60m | Volatility | MAINTAIN_LOW | 11017 | 1.4% | 2.0% | +0.6% | 30.4% |
| 60m | Direction | UP -> DOWN | 2567 | 1.4% | 6.1% | +4.7% | 29.4% |
| 60m | Direction | DOWN -> UP | 2567 | 1.4% | 1.4% | -0.0% | 93.8% |
| 60m | Direction | MAINTAIN_UP | 11152 | 1.4% | 0.3% | -1.1% | 92.0% |
| 60m | Direction | MAINTAIN_DOWN | 8412 | 1.4% | 1.4% | +0.0% | 21.7% |
| 120m | Volatility | LOW -> HIGH | 1566 | 4.1% | 4.8% | +0.7% | 12.5% |
| 120m | Volatility | HIGH -> LOW | 1334 | 4.1% | 1.4% | -2.7% | 90.0% |
| 120m | Volatility | MAINTAIN_HIGH | 10785 | 4.1% | 2.0% | -2.2% | 86.4% |
| 120m | Volatility | MAINTAIN_LOW | 11017 | 4.1% | 6.5% | +2.3% | 34.8% |
| 120m | Direction | UP -> DOWN | 2567 | 4.1% | 10.0% | +5.9% | 29.4% |
| 120m | Direction | DOWN -> UP | 2567 | 4.1% | 6.9% | +2.8% | 18.8% |
| 120m | Direction | MAINTAIN_UP | 11152 | 4.1% | 1.6% | -2.5% | 88.0% |
| 120m | Direction | MAINTAIN_DOWN | 8412 | 4.1% | 4.8% | +0.7% | 43.5% |
| 300m | Volatility | LOW -> HIGH | 1566 | 12.7% | 13.1% | +0.4% | 25.0% |
| 300m | Volatility | HIGH -> LOW | 1334 | 12.7% | 1.7% | -11.0% | 90.0% |
| 300m | Volatility | MAINTAIN_HIGH | 10785 | 12.7% | 5.2% | -7.5% | 81.8% |
| 300m | Volatility | MAINTAIN_LOW | 11017 | 12.7% | 21.4% | +8.6% | 47.8% |
| 300m | Direction | UP -> DOWN | 2567 | 12.7% | 33.6% | +20.9% | 52.9% |
| 300m | Direction | DOWN -> UP | 2567 | 12.7% | 21.3% | +8.6% | 25.0% |
| 300m | Direction | MAINTAIN_UP | 11152 | 12.7% | 4.4% | -8.3% | 88.0% |
| 300m | Direction | MAINTAIN_DOWN | 8412 | 12.7% | 14.7% | +2.0% | 43.5% |

