# Stage C — Conditional Behaviour of Existing Price States Under Volume Acceleration

- **Train N:** 25062
- **VA_HIGH Threshold (Q75):** 1.4638
- **HIGH_VOL Threshold (Median):** 0.0004
- **HIGH_PERS Threshold (Median):** 0.1129

### H1: Volume × Existing Trend
#### Horizon: 60m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| UP | VA_HIGH | 3527 | 47.7% | -2.5 bps | -1.1 bps | -0.0971 |
| UP | VA_NORMAL | 10554 | 50.0% | -0.3 bps | 0.0 bps | 0.0018 |
| DOWN | VA_HIGH | 2738 | 47.4% | -0.5 bps | -1.5 bps | -0.0614 |
| DOWN | VA_NORMAL | 8241 | 54.1% | 1.3 bps | 1.9 bps | -0.0468 |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| UP | -2.2 | -2.1 | 60% | 18 | -3.9, -1.8, 4.7, -2.4, 0.9, 5.1, -12.2, -7.8, -5.0, 2.7 |
| DOWN | -1.8 | -3.1 | 56% | 0 | 6.4, -91.2, 5.0, -3.1, N/A, -10.0, 3.2, -11.1, -3.3, 41.4 |

#### Horizon: 120m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| UP | VA_HIGH | 3527 | 45.7% | -6.9 bps | -3.8 bps | -0.0608 |
| UP | VA_NORMAL | 10554 | 48.7% | -1.7 bps | -1.0 bps | -0.0124 |
| DOWN | VA_HIGH | 2738 | 55.3% | 3.1 bps | 2.8 bps | 0.0358 |
| DOWN | VA_NORMAL | 8241 | 57.5% | 3.5 bps | 4.9 bps | -0.0176 |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| UP | -5.2 | -10.4 | 80% | 18 | -2.9, 7.7, -29.7, -30.6, -3.7, -30.8, -17.0, -21.5, -1.4, 1.1 |
| DOWN | -0.4 | -0.1 | 56% | 0 | -2.8, 18.7, -0.1, 7.5, N/A, -2.7, 7.2, -13.7, -2.9, 38.9 |

#### Horizon: 300m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| UP | VA_HIGH | 3527 | 49.9% | -4.3 bps | -0.1 bps | 0.0190 |
| UP | VA_NORMAL | 10554 | 47.4% | -5.3 bps | -3.4 bps | -0.0405 |
| DOWN | VA_HIGH | 2738 | 50.8% | 4.7 bps | 0.6 bps | 0.0631 |
| DOWN | VA_NORMAL | 8241 | 57.8% | 4.8 bps | 7.6 bps | -0.0007 |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| UP | 0.9 | -1.5 | 40% | 18 | 2.1, -4.4, -3.2, -31.4, -6.8, 45.2, 2.5, -2.7, 4.3, -0.3 |
| DOWN | -0.1 | 1.6 | 44% | 0 | 22.8, 17.0, -8.6, 1.6, N/A, -5.6, 12.5, 6.9, -8.6, -28.8 |

### H2: Volume × Existing Volatility
#### Horizon: 60m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| HIGH_VOL | VA_HIGH | 2956 | 49.2% | 0.2 bps | -0.6 bps | N/A |
| HIGH_VOL | VA_NORMAL | 9575 | 51.7% | 2.6 bps | 1.1 bps | N/A |
| LOW_VOL | VA_HIGH | 3310 | 46.1% | -3.2 bps | -1.9 bps | N/A |
| LOW_VOL | VA_NORMAL | 9221 | 51.9% | -1.9 bps | 0.7 bps | N/A |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| HIGH_VOL | -2.4 | -3.5 | 62% | 0 | 1.8, -4.7, 5.0, -5.9, N/A, -2.3, -10.0, -9.0, N/A, 1.6 |
| LOW_VOL | -1.3 | -2.9 | 67% | 0 | -0.4, N/A, 2.7, -2.9, 0.9, -12.6, -7.0, -9.4, -4.2, 46.3 |

#### Horizon: 120m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| HIGH_VOL | VA_HIGH | 2956 | 53.5% | 1.7 bps | 2.8 bps | N/A |
| HIGH_VOL | VA_NORMAL | 9575 | 53.5% | 3.9 bps | 3.3 bps | N/A |
| LOW_VOL | VA_HIGH | 3310 | 46.7% | -6.4 bps | -2.5 bps | N/A |
| LOW_VOL | VA_NORMAL | 9221 | 51.7% | -2.9 bps | 1.0 bps | N/A |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| HIGH_VOL | -2.2 | -2.6 | 50% | 0 | -15.2, 6.1, 2.7, -8.6, N/A, -5.2, -13.9, 1.1, N/A, 0.1 |
| LOW_VOL | -3.5 | -3.7 | 78% | 0 | -0.4, N/A, -16.9, 6.9, -3.7, -54.2, -18.5, -18.9, -1.3, 50.3 |

#### Horizon: 300m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| HIGH_VOL | VA_HIGH | 2956 | 53.0% | 7.1 bps | 3.5 bps | N/A |
| HIGH_VOL | VA_NORMAL | 9575 | 56.1% | 4.9 bps | 7.0 bps | N/A |
| LOW_VOL | VA_HIGH | 3310 | 47.9% | -7.1 bps | -1.8 bps | N/A |
| LOW_VOL | VA_NORMAL | 9221 | 47.7% | -6.8 bps | -2.4 bps | N/A |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| HIGH_VOL | 2.3 | 1.0 | 75% | 0 | 24.1, -5.0, 0.3, -10.9, N/A, 12.7, 1.7, 9.0, N/A, 0.2 |
| LOW_VOL | -0.3 | 0.1 | 44% | 0 | 8.7, N/A, -24.6, 0.1, -6.8, 16.6, -40.5, 7.2, 2.1, -0.3 |

### H3: Volume × Persistence
#### Horizon: 60m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| HIGH_PERSISTENCE | VA_HIGH | 3696 | 46.0% | -2.5 bps | -2.3 bps | N/A |
| HIGH_PERSISTENCE | VA_NORMAL | 8835 | 52.5% | 0.9 bps | 1.0 bps | N/A |
| LOW_PERSISTENCE | VA_HIGH | 2570 | 49.7% | -0.4 bps | -0.1 bps | N/A |
| LOW_PERSISTENCE | VA_NORMAL | 9961 | 51.2% | -0.1 bps | 0.8 bps | N/A |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| HIGH_PERSISTENCE | -3.3 | -4.8 | 60% | 308 | 4.7, -5.3, 0.5, -11.8, 3.7, -4.2, -12.5, -13.8, -6.8, 9.6 |
| LOW_PERSISTENCE | -0.4 | -3.6 | 70% | 174 | -3.8, -4.3, 12.1, 4.6, -4.2, -3.4, -9.3, -7.1, -1.2, 8.4 |

#### Horizon: 120m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| HIGH_PERSISTENCE | VA_HIGH | 3696 | 48.6% | -3.8 bps | -1.2 bps | N/A |
| HIGH_PERSISTENCE | VA_NORMAL | 8835 | 54.2% | 2.3 bps | 2.9 bps | N/A |
| LOW_PERSISTENCE | VA_HIGH | 2570 | 51.8% | -0.7 bps | 1.1 bps | N/A |
| LOW_PERSISTENCE | VA_NORMAL | 9961 | 51.1% | -1.0 bps | 0.9 bps | N/A |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| HIGH_PERSISTENCE | -6.1 | -5.4 | 80% | 308 | -1.6, 0.0, -5.1, -2.4, -5.6, -6.9, -22.6, -20.1, -6.3, 6.0 |
| LOW_PERSISTENCE | 0.3 | 2.4 | 50% | 174 | -4.3, 13.7, 4.9, 11.8, -2.6, -25.2, -0.1, -13.0, 7.1, 10.8 |

#### Horizon: 300m
| State | Regime | N | Hit Rate | Mean Ret | Med Ret | Spearman |
|---|---|---|---|---|---|---|
| HIGH_PERSISTENCE | VA_HIGH | 3696 | 50.5% | -2.1 bps | 0.5 bps | N/A |
| HIGH_PERSISTENCE | VA_NORMAL | 8835 | 53.2% | 0.1 bps | 3.9 bps | N/A |
| LOW_PERSISTENCE | VA_HIGH | 2570 | 50.0% | 2.1 bps | 0.0 bps | N/A |
| LOW_PERSISTENCE | VA_NORMAL | 9961 | 50.9% | -1.7 bps | 1.0 bps | N/A |

**Block Stability Analysis (Mean Ret in bps)**
| State | Overall Δ | Med Block Δ | Sign Stability | Min Block N | Block 0..9 Δs |
|---|---|---|---|---|---|
| HIGH_PERSISTENCE | -2.2 | -6.5 | 70% | 308 | 10.8, -12.2, -14.6, -4.9, -8.9, 15.8, -5.2, 5.0, -7.8, -8.3 |
| LOW_PERSISTENCE | 3.7 | 5.0 | 80% | 174 | 6.2, 3.9, 0.4, 2.9, -3.2, 7.4, 18.1, -0.5, 19.3, 6.8 |

