# TIME-008 — Discrimination Analysis Report

**Generated:** 2026-08-20T11:46:16.722688+00:00
**Input:** `time_machine/cohorts/aggregate_evidence.csv`
**Total rows:** 612  |  **Eligible for primary comparison:** 345
**Cohorts:** T1, T2, T3, T4, T5, T6

---

## Governing constraints

- Analysis-only. No upstream pipeline changes permitted.
- Consistency threshold: >=4/6 cohorts for 'consistent', >=3/6 reversals for 'inconsistent'.
- Primary outcomes: `target_reached`, `risk_reached`, `realized_return` (eligible rows only).
- Secondary outcomes: `actual_mfe`, `actual_mae`, `exit_reason` (full population).
- `eligible_for_primary_comparison` is a filter, not an outcome.
- R:R terciles computed once over all 612 rows; fixed thereafter.

---

## Q1 — Evidence-class discrimination

**Question:** Does evidence_class correspond to materially different forward outcomes?

### Pooled results (primary: eligible rows only)

| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Favourable | 84 | 84 | 39.3% | 16.7% | 0.0079 |
| Mixed | 261 | 261 | 28.4% | 12.3% | 0.0064 |
| Unfavourable | 261 | 0 | N/A | N/A | N/A |
| Insufficient | 6 | 0 | N/A | N/A | N/A |

### Secondary outcomes (full population)

| Evidence Class | N total | MFE mean | MAE mean |
|---|---|---|---|
| Favourable | 84 | 0.0305 | -0.0153 |
| Mixed | 261 | 0.0243 | -0.0159 |
| Unfavourable | 261 | 0.0204 | -0.0221 |
| Insufficient | 6 | 0.0246 | -0.0157 |

### Consistency verdicts (>=4/6 cohorts = consistent)

- **target_reached_Fav_gt_Mix_gt_Unf**: 0/6 cohorts → **inconsistent**
  - T1: False
  - T2: False
  - T3: False
  - T4: False
  - T5: False
  - T6: False
- **risk_reached_Unf_gt_Mix_gt_Fav**: 0/6 cohorts → **inconsistent**
  - T1: False
  - T2: False
  - T3: False
  - T4: False
  - T5: False
  - T6: False
- **realized_return_Fav_gt_Mix_gt_Unf**: 0/6 cohorts → **inconsistent**
  - T1: False
  - T2: False
  - T3: False
  - T4: False
  - T5: False
  - T6: False

## Q2 — Direction asymmetry

**Question:** Does discrimination differ between LONG and SHORT decisions?

Direction counts: LONG=318, SHORT=294

### LONG — Pooled by evidence class

| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Favourable | 67 | 67 | 46.3% | 13.4% | 0.0111 |
| Mixed | 159 | 159 | 37.7% | 11.3% | 0.0100 |
| Unfavourable | 89 | 0 | N/A | N/A | N/A |
| Insufficient | 3 | 0 | N/A | N/A | N/A |

**Consistency verdicts (LONG):**
- target_reached_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**
- risk_reached_Unf_gt_Mix_gt_Fav: 0/6 → **inconsistent**
- realized_return_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**

### SHORT — Pooled by evidence class

| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Favourable | 17 | 17 | 11.8% | 29.4% | -0.0046 |
| Mixed | 102 | 102 | 13.7% | 13.7% | 0.0009 |
| Unfavourable | 172 | 0 | N/A | N/A | N/A |
| Insufficient | 3 | 0 | N/A | N/A | N/A |

**Consistency verdicts (SHORT):**
- target_reached_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**
- risk_reached_Unf_gt_Mix_gt_Fav: 0/6 → **inconsistent**
- realized_return_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**

## Q3 — Action vs underlying decision

**Question:** Does action contain predictive information beyond evidence_class?

Action counts (all 612): Buy=74, NoTrade=267, Sell=17, Watch=254

### Pooled: within Favourable by action

| Action | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Buy | 67 | 67 | 46.3% | 13.4% | 0.0111 |
| Sell | 17 | 17 | 11.8% | 29.4% | -0.0046 |

### Pooled: within Mixed by action

| Action | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Buy | 7 | 7 | 28.6% | 28.6% | 0.0166 |
| Watch | 254 | 254 | 28.3% | 11.8% | 0.0062 |

### Pooled: within Unfavourable by action

| Action | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| NoTrade | 261 | 0 | N/A | N/A | N/A |

### Pooled: within Insufficient by action

| Action | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| NoTrade | 6 | 0 | N/A | N/A | N/A |

### Specific comparisons (pre-specified)

- **Favourable_Buy_gt_Watch_target**: Buy > Watch within Favourable (target_reached_rate): 0/0 cohorts → **insufficient_data**
- **Mixed_Watch_gt_NoTrade_target**: Watch > NoTrade within Mixed (target_reached_rate): 0/0 cohorts → **insufficient_data**
- **Favourable_Buy_gt_Watch_return**: Buy > Watch within Favourable (realized_return_mean): 0/0 cohorts → **insufficient_data**
- **Mixed_Watch_gt_NoTrade_return**: Watch > NoTrade within Mixed (realized_return_mean): 0/0 cohorts → **insufficient_data**

## Q4 — R:R interaction

**Question:** Does the relationship between T0 evidence and outcome depend on adaptive_rr?

R:R tercile boundaries (global, 612 rows): low<=0.9529, medium<=1.1453, high>1.1453

### Low R:R tercile (n=203)

| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Favourable | 37 | 37 | 40.5% | 13.5% | 0.0062 |
| Mixed | 96 | 96 | 36.5% | 11.5% | 0.0061 |
| Unfavourable | 70 | 0 | N/A | N/A | N/A |

**Consistency verdicts (low R:R tercile):**
- target_reached_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**
- risk_reached_Unf_gt_Mix_gt_Fav: 0/6 → **inconsistent**
- realized_return_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**

### Medium R:R tercile (n=202)

| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Favourable | 32 | 32 | 43.8% | 18.8% | 0.0114 |
| Mixed | 93 | 93 | 26.9% | 6.5% | 0.0103 |
| Unfavourable | 77 | 0 | N/A | N/A | N/A |

**Consistency verdicts (medium R:R tercile):**
- target_reached_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**
- risk_reached_Unf_gt_Mix_gt_Fav: 0/6 → **inconsistent**
- realized_return_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**

### High R:R tercile (n=201)

| Evidence Class | N total | N eligible | Target Reached | Risk Reached | Realized Return |
|---|---|---|---|---|---|
| Favourable | 15 | 15 | 26.7% | 20.0% | 0.0045 |
| Mixed | 72 | 72 | 19.4% | 20.8% | 0.0019 |
| Unfavourable | 114 | 0 | N/A | N/A | N/A |

**Consistency verdicts (high R:R tercile):**
- target_reached_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**
- risk_reached_Unf_gt_Mix_gt_Fav: 0/6 → **inconsistent**
- realized_return_Fav_gt_Mix_gt_Unf: 0/6 → **inconsistent**

## Cohort consistency summary

| Question | Check | Verdict |
|---|---|---|
| Q1 | target_reached_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q1 | risk_reached_Unf_gt_Mix_gt_Fav | **inconsistent** |
| Q1 | realized_return_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q2_LONG | target_reached_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q2_LONG | risk_reached_Unf_gt_Mix_gt_Fav | **inconsistent** |
| Q2_LONG | realized_return_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q2_SHORT | target_reached_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q2_SHORT | risk_reached_Unf_gt_Mix_gt_Fav | **inconsistent** |
| Q2_SHORT | realized_return_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q3_specific | Favourable_Buy_gt_Watch_target | **insufficient_data** |
| Q3_specific | Mixed_Watch_gt_NoTrade_target | **insufficient_data** |
| Q3_specific | Favourable_Buy_gt_Watch_return | **insufficient_data** |
| Q3_specific | Mixed_Watch_gt_NoTrade_return | **insufficient_data** |
| Q4_low | target_reached_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q4_low | risk_reached_Unf_gt_Mix_gt_Fav | **inconsistent** |
| Q4_low | realized_return_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q4_medium | target_reached_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q4_medium | risk_reached_Unf_gt_Mix_gt_Fav | **inconsistent** |
| Q4_medium | realized_return_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q4_high | target_reached_Fav_gt_Mix_gt_Unf | **inconsistent** |
| Q4_high | risk_reached_Unf_gt_Mix_gt_Fav | **inconsistent** |
| Q4_high | realized_return_Fav_gt_Mix_gt_Unf | **inconsistent** |

---

## Bounded conclusion

TIME-008 establishes descriptive evidence about discrimination—or lack thereof—within the frozen historical experiment. It does not establish predictive validity, economic utility, or deployment readiness.

TIME-008 may NOT conclude:
- That Coralys has proven predictive power
- That the system is economically useful
- That any threshold should be changed
- That any algorithm should be modified
- That the evidence is sufficient for live deployment
