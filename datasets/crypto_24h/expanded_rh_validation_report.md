# Stage C: Validation of Frozen Expanded Hypotheses

Evaluation of H1, H2, and H3 on the strict chronological 20% Validation split, using only thresholds and normalizations derived from the Train split.

## H1: LOW_VOL_UP / R1 / 300m
| Split | Spearman | N | Blocks | PosBlocks | Min Obs/Blk | Med Obs/Blk |
|---|---|---|---|---|---|---|
| Train | 0.3263 | 6427 | 24 | 63.6% | 22 | 325 |
| Validation | 0.1833 | 3818 | 12 | 75.0% | 81 | 324 |

## H2: LOW_VOL_EXPANDING / R1 / 300m
| Split | Spearman | N | Blocks | PosBlocks | Min Obs/Blk | Med Obs/Blk |
|---|---|---|---|---|---|---|
| Train | 0.3075 | 5607 | 21 | 61.1% | 17 | 268 |
| Validation | 0.1135 | 2790 | 9 | 55.6% | 65 | 308 |

## H3: CHOP_DOWN / R2 / 300m
| Split | Spearman | N | Blocks | PosBlocks | Min Obs/Blk | Med Obs/Blk |
|---|---|---|---|---|---|---|
| Train | 0.2398 | 6211 | 36 | 67.6% | 1 | 171 |
| Validation | -0.0998 | 1863 | 12 | 45.5% | 2 | 174 |

