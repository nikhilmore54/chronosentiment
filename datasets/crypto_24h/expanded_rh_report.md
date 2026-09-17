# Expanded Regime Matrix v0.1: Train Discovery

## RH-1: Volatility State × Short-Term Direction
| Regime | IC | Horizon | N | N % | Blocks | Min Obs/Blk | Med Obs/Blk | Spearman |
|---|---|---|---|---|---|---|---|---|
| LOW_VOL_UP | R1 | 120m | 6427 | 25.8% | 24 | 22 | 325 | 0.2151 |
| LOW_VOL_UP | R1 | 300m | 6427 | 25.8% | 24 | 22 | 325 | 0.3263 |
| LOW_VOL_UP | R2 | 120m | 6427 | 25.8% | 24 | 22 | 325 | 0.0527 |
| LOW_VOL_UP | R2 | 300m | 6427 | 25.8% | 24 | 22 | 325 | 0.1205 |
| LOW_VOL_UP | R3 | 120m | 6427 | 25.8% | 24 | 22 | 325 | 0.1412 |
| LOW_VOL_UP | R3 | 300m | 6427 | 25.8% | 24 | 22 | 325 | 0.2584 |
| LOW_VOL_DOWN | R1 | 120m | 6011 | 24.2% | 25 | 1 | 255 | 0.1365 |
| LOW_VOL_DOWN | R1 | 300m | 6011 | 24.2% | 25 | 1 | 255 | 0.2191 |
| LOW_VOL_DOWN | R2 | 120m | 6011 | 24.2% | 25 | 1 | 255 | 0.1146 |
| LOW_VOL_DOWN | R2 | 300m | 6011 | 24.2% | 25 | 1 | 255 | 0.1053 |
| LOW_VOL_DOWN | R3 | 120m | 6011 | 24.2% | 25 | 1 | 255 | 0.1308 |
| LOW_VOL_DOWN | R3 | 300m | 6011 | 24.2% | 25 | 1 | 255 | 0.1934 |
| HIGH_VOL_DOWN | R1 | 120m | 6194 | 24.9% | 22 | 28 | 305 | 0.0389 |
| HIGH_VOL_DOWN | R1 | 300m | 6194 | 24.9% | 22 | 28 | 305 | 0.0459 |
| HIGH_VOL_DOWN | R2 | 120m | 6194 | 24.9% | 22 | 28 | 305 | 0.0986 |
| HIGH_VOL_DOWN | R2 | 300m | 6194 | 24.9% | 22 | 28 | 305 | 0.1980 |
| HIGH_VOL_DOWN | R3 | 120m | 6194 | 24.9% | 22 | 28 | 305 | 0.0732 |
| HIGH_VOL_DOWN | R3 | 300m | 6194 | 24.9% | 22 | 28 | 305 | 0.1419 |
| HIGH_VOL_UP | R1 | 120m | 6242 | 25.1% | 21 | 1 | 297 | 0.0938 |
| HIGH_VOL_UP | R1 | 300m | 6242 | 25.1% | 21 | 1 | 297 | 0.0164 |
| HIGH_VOL_UP | R2 | 120m | 6242 | 25.1% | 21 | 1 | 297 | 0.2328 |
| HIGH_VOL_UP | R2 | 300m | 6242 | 25.1% | 21 | 1 | 297 | 0.2329 |
| HIGH_VOL_UP | R3 | 120m | 6242 | 25.1% | 21 | 1 | 297 | 0.1549 |
| HIGH_VOL_UP | R3 | 300m | 6242 | 25.1% | 21 | 1 | 297 | 0.1348 |

## RH-2: Volatility Level × Volatility Dynamics
| Regime | IC | Horizon | N | N % | Blocks | Min Obs/Blk | Med Obs/Blk | Spearman |
|---|---|---|---|---|---|---|---|---|
| LOW_VOL_CONTRACTING | R1 | 120m | 6834 | 27.5% | 23 | 1 | 254 | 0.2033 |
| LOW_VOL_CONTRACTING | R1 | 300m | 6834 | 27.5% | 23 | 1 | 254 | 0.2418 |
| LOW_VOL_CONTRACTING | R2 | 120m | 6834 | 27.5% | 23 | 1 | 254 | 0.0687 |
| LOW_VOL_CONTRACTING | R2 | 300m | 6834 | 27.5% | 23 | 1 | 254 | 0.0447 |
| LOW_VOL_CONTRACTING | R3 | 120m | 6834 | 27.5% | 23 | 1 | 254 | 0.1487 |
| LOW_VOL_CONTRACTING | R3 | 300m | 6834 | 27.5% | 23 | 1 | 254 | 0.1736 |
| LOW_VOL_EXPANDING | R1 | 120m | 5607 | 22.5% | 21 | 17 | 268 | 0.1358 |
| LOW_VOL_EXPANDING | R1 | 300m | 5607 | 22.5% | 21 | 17 | 268 | 0.3075 |
| LOW_VOL_EXPANDING | R2 | 120m | 5607 | 22.5% | 21 | 17 | 268 | 0.1150 |
| LOW_VOL_EXPANDING | R2 | 300m | 5607 | 22.5% | 21 | 17 | 268 | 0.1970 |
| LOW_VOL_EXPANDING | R3 | 120m | 5607 | 22.5% | 21 | 17 | 268 | 0.1254 |
| LOW_VOL_EXPANDING | R3 | 300m | 5607 | 22.5% | 21 | 17 | 268 | 0.2986 |
| HIGH_VOL_EXPANDING | R1 | 120m | 4473 | 18.0% | 16 | 5 | 294 | 0.2347 |
| HIGH_VOL_EXPANDING | R1 | 300m | 4473 | 18.0% | 16 | 5 | 294 | 0.2590 |
| HIGH_VOL_EXPANDING | R2 | 120m | 4473 | 18.0% | 16 | 5 | 294 | 0.1983 |
| HIGH_VOL_EXPANDING | R2 | 300m | 4473 | 18.0% | 16 | 5 | 294 | 0.1927 |
| HIGH_VOL_EXPANDING | R3 | 120m | 4473 | 18.0% | 16 | 5 | 294 | 0.2235 |
| HIGH_VOL_EXPANDING | R3 | 300m | 4473 | 18.0% | 16 | 5 | 294 | 0.2415 |
| HIGH_VOL_CONTRACTING | R1 | 120m | 7968 | 32.0% | 22 | 28 | 321 | -0.0323 |
| HIGH_VOL_CONTRACTING | R1 | 300m | 7968 | 32.0% | 22 | 28 | 321 | -0.0918 |
| HIGH_VOL_CONTRACTING | R2 | 120m | 7968 | 32.0% | 22 | 28 | 321 | 0.1545 |
| HIGH_VOL_CONTRACTING | R2 | 300m | 7968 | 32.0% | 22 | 28 | 321 | 0.2317 |
| HIGH_VOL_CONTRACTING | R3 | 120m | 7968 | 32.0% | 22 | 28 | 321 | 0.0397 |
| HIGH_VOL_CONTRACTING | R3 | 300m | 7968 | 32.0% | 22 | 28 | 321 | 0.0690 |

## RH-3: Path Efficiency × Short-Term Direction
| Regime | IC | Horizon | N | N % | Blocks | Min Obs/Blk | Med Obs/Blk | Spearman |
|---|---|---|---|---|---|---|---|---|
| TREND_UP | R1 | 120m | 6445 | 25.9% | 35 | 42 | 168 | 0.1024 |
| TREND_UP | R1 | 300m | 6445 | 25.9% | 35 | 42 | 168 | 0.1674 |
| TREND_UP | R2 | 120m | 6445 | 25.9% | 35 | 42 | 168 | 0.1087 |
| TREND_UP | R2 | 300m | 6445 | 25.9% | 35 | 42 | 168 | 0.1342 |
| TREND_UP | R3 | 120m | 6445 | 25.9% | 35 | 42 | 168 | 0.0972 |
| TREND_UP | R3 | 300m | 6445 | 25.9% | 35 | 42 | 168 | 0.1663 |
| TREND_DOWN | R1 | 120m | 5994 | 24.1% | 35 | 29 | 169 | -0.0020 |
| TREND_DOWN | R1 | 300m | 5994 | 24.1% | 35 | 29 | 169 | 0.0715 |
| TREND_DOWN | R2 | 120m | 5994 | 24.1% | 35 | 29 | 169 | 0.1188 |
| TREND_DOWN | R2 | 300m | 5994 | 24.1% | 35 | 29 | 169 | 0.1497 |
| TREND_DOWN | R3 | 120m | 5994 | 24.1% | 35 | 29 | 169 | 0.0633 |
| TREND_DOWN | R3 | 300m | 5994 | 24.1% | 35 | 29 | 169 | 0.1464 |
| CHOP_UP | R1 | 120m | 6224 | 25.0% | 35 | 4 | 163 | 0.2101 |
| CHOP_UP | R1 | 300m | 6224 | 25.0% | 35 | 4 | 163 | 0.1349 |
| CHOP_UP | R2 | 120m | 6224 | 25.0% | 35 | 4 | 163 | 0.1684 |
| CHOP_UP | R2 | 300m | 6224 | 25.0% | 35 | 4 | 163 | 0.2233 |
| CHOP_UP | R3 | 120m | 6224 | 25.0% | 35 | 4 | 163 | 0.1988 |
| CHOP_UP | R3 | 300m | 6224 | 25.0% | 35 | 4 | 163 | 0.2009 |
| CHOP_DOWN | R1 | 120m | 6211 | 25.0% | 36 | 1 | 171 | 0.0844 |
| CHOP_DOWN | R1 | 300m | 6211 | 25.0% | 36 | 1 | 171 | 0.1083 |
| CHOP_DOWN | R2 | 120m | 6211 | 25.0% | 36 | 1 | 171 | 0.1528 |
| CHOP_DOWN | R2 | 300m | 6211 | 25.0% | 36 | 1 | 171 | 0.2398 |
| CHOP_DOWN | R3 | 120m | 6211 | 25.0% | 36 | 1 | 171 | 0.1330 |
| CHOP_DOWN | R3 | 300m | 6211 | 25.0% | 36 | 1 | 171 | 0.2002 |

