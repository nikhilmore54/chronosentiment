# Stage E: Behavioural Phenotype Extraction

## Phenotype Definitions (Descriptive)
- **Persistent Build**: `ret_30 > ret_15 AND ret_60 > ret_30 AND ret_120 > ret_60`
- **Early Excursion & Decay**: `ret_60 > 0 AND ret_120 < ret_60 * 0.5`
- **Adverse Excursion & Recovery**: `time_to_mae_60 < 30 AND mae_60 < 0 AND max_recovery_after_mae_60 > 0`
- **Flat / Bound**: `mfe_60 < 0.0053 AND mae_60 > -0.0025` (empirical medians)

## Unconditional Frequency
| Phenotype | N | % of Population |
|---|---|---|
| Persistent_Build | 585 | 22.66% |
| Early_Excursion_Decay | 402 | 15.57% |
| Adverse_Excursion_Recovery | 1709 | 66.19% |
| Flat_Bound | 360 | 13.94% |

## Overlap Matrix
| Phenotype | Persistent_Build | Early_Excursion_Decay | Adverse_Excursion_Recovery | Flat_Bound |
|---|---|---|---|---|
| Persistent_Build | 585 | 0 | 538 | 77 |
| Early_Excursion_Decay | 0 | 402 | 292 | 47 |
| Adverse_Excursion_Recovery | 538 | 292 | 1709 | 264 |
| Flat_Bound | 77 | 47 | 264 | 360 |

## Phenotype Profiles (Medians)
### Persistent_Build (N=585)
**VA × Persistence Distribution:**
- NORMAL / LOW: 305 (52.1%)
- NORMAL / HIGH: 111 (19.0%)
- HIGH / LOW: 111 (19.0%)
- HIGH / HIGH: 58 (9.9%)

**Returns:**
| 15m | 30m | 60m | 120m | 300m |
|---|---|---|---|---|
| 0.0002 | 0.0024 | 0.0056 | 0.0126 | 0.0241 |

**Excursion & Path:**
| MFE | MAE | Time-to-MFE | Time-to-MAE | Recovery (after MAE) | Giveback (after MFE) |
|---|---|---|---|---|---|
| 0.0077 | -0.0017 | 52.0 | 9.0 | 0.0089 | 0.0031 |

### Early_Excursion_Decay (N=402)
**VA × Persistence Distribution:**
- NORMAL / LOW: 207 (51.5%)
- NORMAL / HIGH: 111 (27.6%)
- HIGH / HIGH: 58 (14.4%)
- HIGH / LOW: 26 (6.5%)

**Returns:**
| 15m | 30m | 60m | 120m | 300m |
|---|---|---|---|---|
| 0.0010 | 0.0011 | 0.0033 | -0.0008 | 0.0056 |

**Excursion & Path:**
| MFE | MAE | Time-to-MFE | Time-to-MAE | Recovery (after MAE) | Giveback (after MFE) |
|---|---|---|---|---|---|
| 0.0067 | -0.0021 | 41.0 | 15.0 | 0.0087 | 0.0051 |

### Adverse_Excursion_Recovery (N=1709)
**VA × Persistence Distribution:**
- NORMAL / LOW: 871 (51.0%)
- NORMAL / HIGH: 452 (26.4%)
- HIGH / LOW: 220 (12.9%)
- HIGH / HIGH: 166 (9.7%)

**Returns:**
| 15m | 30m | 60m | 120m | 300m |
|---|---|---|---|---|
| 0.0007 | 0.0021 | 0.0040 | 0.0061 | 0.0141 |

**Excursion & Path:**
| MFE | MAE | Time-to-MFE | Time-to-MAE | Recovery (after MAE) | Giveback (after MFE) |
|---|---|---|---|---|---|
| 0.0067 | -0.0018 | 45.0 | 8.0 | 0.0086 | 0.0051 |

### Flat_Bound (N=360)
**VA × Persistence Distribution:**
- NORMAL / LOW: 281 (78.1%)
- NORMAL / HIGH: 67 (18.6%)
- HIGH / LOW: 11 (3.1%)
- HIGH / HIGH: 1 (0.3%)

**Returns:**
| 15m | 30m | 60m | 120m | 300m |
|---|---|---|---|---|
| 0.0012 | 0.0017 | 0.0025 | 0.0042 | 0.0200 |

**Excursion & Path:**
| MFE | MAE | Time-to-MFE | Time-to-MAE | Recovery (after MAE) | Giveback (after MFE) |
|---|---|---|---|---|---|
| 0.0042 | -0.0015 | 45.0 | 10.5 | 0.0050 | 0.0025 |

