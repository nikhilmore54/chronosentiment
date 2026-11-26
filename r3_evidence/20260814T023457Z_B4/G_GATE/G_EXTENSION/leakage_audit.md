# G-GATE Leakage Audit (v1.1)

**Result:** PASS

| # | Check | Result | Notes |
|---|-------|--------|-------|
| 1 | Feature timestamps <= decision evaluation_timestamp | PASS | All assessment timestamps are <= linked decision timestamps. Feature is signature_hash only. |
| 2 | Labels not used as features | PASS | Candidate uses only signature_hash. outcome_return is used solely to form Y_h. |
| 3 | Lookup fitted on TRAIN only | PASS | p_hat is estimated from ranks 1–55 only. |
| 4 | VALIDATION not used for fitting/selection/thresholding | PASS | Validation is unused in model construction. No threshold is selected. |
| 5 | TEST unused until final evaluation | PASS | Test labels and signatures are not used to estimate p_hat or p_baseline. |
| 6 | No duplicate strategy_id in a fold | PASS | TRAIN=55 VALIDATION=27 TEST=28 |
| 7 | Cluster constraint: four horizons share fold | PASS | Each strategy’s four outcomes inherit the strategy fold. |
| 8 | Calendar overlap of outcome-expiry windows documented | PASS | TRAIN/VAL share 2023-07-31T15:30:00Z. Some TRAIN 60D expiries fall after first VAL evaluations. Some VAL 60D expiries fall after TEST start 2024-04-30T15:30:00Z. TRAIN latest 60D expiry 2023-09-29T15:30:00Z is before TEST start. Ranks were not altered. |
| 9 | No scaler/encoder/prevalence from validation or test | PASS | Prevalence and signature rates are training-fold only. No scaler or encoder is fitted. |

Any FAIL forces INCONCLUSIVE.
