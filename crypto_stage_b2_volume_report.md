# Stage B2: Participation & Order-Flow Characterisation

## Data Audit
- **Frozen Census N**: 2582
- **Successfully Joined N**: 2582
- **Missing/Invalid Volume N**: 0

*Note: `volume_acceleration_60m` is preserved exactly as evaluated in Stage A-D.*

## Unconditional Volume Distributions
| Variable | P10 | P25 | P50 (Median) | P75 | P90 |
|---|---|---|---|---|---|
| volume_sum_60m | 729.2507 | 925.3227 | 1364.9727 | 2191.1866 | 3363.1803 |
| volume_acceleration_60m | 0.5511 | 0.7282 | 0.9513 | 1.3394 | 2.1401 |
| relative_volume_60m | 0.5112 | 0.6779 | 0.9108 | 1.3622 | 1.8827 |
| volume_concentration_60m | 0.0403 | 0.0494 | 0.0664 | 0.1076 | 0.1623 |

## Volume by VA × Persistence Cohort (Medians)
| Cohort | N | volume_sum_60m | volume_acceleration_60m | relative_volume_60m | volume_concentration_60m |
|---|---|---|---|---|---|
| HIGH / LOW | 309 | 1297.4276 | 1.9768 | 1.0945 | 0.0828 |
| HIGH / HIGH | 201 | 3104.8775 | 2.4149 | 1.8484 | 0.1253 |
| NORMAL / LOW | 1323 | 1009.4533 | 0.8952 | 0.7895 | 0.0552 |
| NORMAL / HIGH | 749 | 2351.7640 | 0.7668 | 1.1920 | 0.0693 |

## Volume by Descriptive Phenotype (Medians)
Examining whether the identified 300-bar path shapes exhibit materially different participation structures.

| Phenotype | N | volume_sum_60m | volume_acceleration_60m | relative_volume_60m | volume_concentration_60m |
|---|---|---|---|---|---|
| Persistent_Build | 585 | 1161.5240 | 1.0512 | 0.9181 | 0.0726 |
| Early_Excursion_Decay | 402 | 1416.2930 | 0.9658 | 0.9228 | 0.0692 |
| Adverse_Excursion_Recovery | 1709 | 1367.4492 | 0.9668 | 0.9630 | 0.0659 |
| Flat_Bound | 360 | 940.2509 | 0.8522 | 0.7882 | 0.0504 |
