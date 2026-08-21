# TIME-010 Analysis Protocol

**Status:** FROZEN — pre-specified before any COMPLETE observations  
**Frozen at:** 2026-08-21 (day 1 of TIME-009 observation window)  
**Predecessor:** TIME-009 prospective observation protocol (docs/TIME009_PROTOCOL.md)  
**Input:** `time_machine/analysis/TIME009/prospective_evidence.csv` (COMPLETE rows only)  
**Output:** `time_machine/analysis/TIME010/analysis_report.md` + JSON artifacts  

---

## 1. Purpose

TIME-010 is the pre-specified analysis of the TIME-009 prospective observation dataset.

It evaluates whether the frozen Coralys evidence classification (`evidence_class`) discriminates forward outcomes in genuinely prospective data — i.e., data generated after the classification was frozen and the observation protocol was committed.

TIME-010 does **not** modify Coralys, re-tune thresholds, or generate trading signals. It produces a research conclusion only.

---

## 2. Inputs and eligibility

### 2.1 Dataset source

Input: `time_machine/analysis/TIME009/prospective_evidence.csv`

Only rows with `observation_status == "COMPLETE"` are included. The `time009_dataset.py` aggregator enforces this.

### 2.2 Primary eligibility criterion (pre-specified, verbatim from TIME-009 AC-T9-05)

A row is **eligible for primary comparison** if and only if:

```
certification_status IN ("CERTIFIED", "DEGRADED")
AND evidence_class IN ("Favourable", "Mixed")
AND observation_status == "COMPLETE"
AND exit_reason NOT IN ("AMBIGUOUS", "INSUFFICIENT_DATA", "NO_TRADE")
```

This is the `eligible_for_primary_comparison` field in each artifact. TIME-010 reads this field verbatim — it does **not** recompute eligibility.

### 2.3 Unfavourable decisions

`evidence_class == "Unfavourable"` decisions are excluded from the primary comparison. They are included in secondary descriptive tables only.

This mirrors the TIME-008 finding that Unfavourable decisions had 0 eligible rows for primary comparison. If the TIME-009 dataset contains eligible Unfavourable rows, they are reported in a separate descriptive table but do not enter the primary test.

### 2.4 DEGRADED inclusion

DEGRADED decisions are included in the primary comparison (AC-T9-11). A stratified secondary table separates CERTIFIED from DEGRADED.

---

## 3. Primary endpoints (pre-specified)

These two endpoints are the primary research questions. They are evaluated in order. No additional primary endpoints may be added after seeing the data.

### 3.1 Q1 — Target attainment rate by evidence class

**Metric:** `target_reached` (boolean → 0/1)  
**Comparison:** Favourable vs Mixed, eligible rows only  
**Test:** Two-proportion z-test (one-sided, H1: Favourable > Mixed)  
**Significance threshold:** p < 0.05  
**Effect size:** Absolute difference in proportions (pp)  

**Cohort consistency criterion (pre-specified):**  
The primary result is considered **consistent** if:

```
N_cohort_dates_where_Favourable_target_rate > Mixed_target_rate
  ≥ ceil(0.67 × N_cohort_dates_with_eligible_rows_in_both_classes)
```

This is the ≥4/6 criterion from TIME-008, generalised to the actual number of cohort dates accumulated. The threshold is computed from N at analysis time, not pre-set to a fixed number.

**Reporting:** Rate, N, 95% CI (Wilson), p-value, cohort consistency count/N.

### 3.2 Q2 — Realized return mean by evidence class

**Metric:** `realized_return` (continuous, signed)  
**Comparison:** Favourable vs Mixed, eligible rows only  
**Test:** Mann-Whitney U (one-sided, H1: Favourable > Mixed)  
**Significance threshold:** p < 0.05  
**Effect size:** Median difference, common language effect size (CLES)  

**Cohort consistency criterion:** Same ≥ceil(0.67×N) rule applied to median realized_return per cohort date.

**Reporting:** Mean, median, SD, N, 95% CI (bootstrap, 10,000 resamples), p-value, cohort consistency count/N.

---

## 4. Secondary endpoints (pre-specified)

Secondary endpoints are descriptive. They do not determine the primary research conclusion.

### 4.1 Q3 — MFE and MAE by evidence class

**Metrics:** `actual_mfe`, `actual_mae`  
**Comparison:** Favourable vs Mixed, eligible rows  
**Reporting:** Mean, median, SD, N per class. No significance test.

### 4.2 Q4 — Direction stratification

**Stratification:** `direction` ∈ {LONG, SHORT}  
**Metrics:** `target_reached`, `realized_return`  
**Reporting:** Descriptive table (rate/mean per direction × evidence_class). No significance test.

### 4.3 Q5 — DEGRADED vs CERTIFIED stratification

**Stratification:** `certification_status` ∈ {CERTIFIED, DEGRADED}  
**Metrics:** `target_reached`, `realized_return`  
**Reporting:** Descriptive table. No significance test.

### 4.4 Q6 — Exit reason distribution

**Metric:** `exit_reason` counts  
**Reporting:** Frequency table per evidence_class. No significance test.

### 4.5 Q7 — Cohort date breakdown

**Reporting:** Per-cohort-date table: N_total, N_eligible, N_Favourable, N_Mixed, Favourable_target_rate, Mixed_target_rate, ordering (Fav > Mix: yes/no).

---

## 5. Consistency threshold computation

The cohort consistency threshold is computed as follows:

```python
N = number of cohort dates with at least 1 eligible row in BOTH Favourable and Mixed
threshold = math.ceil(0.67 * N)
consistent = (n_cohort_dates_where_Fav_beats_Mix >= threshold)
```

If N < 3, the cohort consistency criterion is reported as **not estimable** (insufficient cohort dates). The primary endpoint result (Q1, Q2) is still reported, but the consistency sub-criterion is marked `insufficient_cohort_dates`.

---

## 6. Research conclusion classification

The TIME-010 conclusion is one of the following pre-specified categories:

| Category | Condition |
|---|---|
| **POSITIVE** | Q1 significant AND Q2 significant AND cohort consistency met for both |
| **PARTIAL** | Exactly one of Q1/Q2 significant AND cohort consistency met for that endpoint |
| **NEGATIVE** | Neither Q1 nor Q2 significant |
| **INCONCLUSIVE** | One or both significant but cohort consistency not met |
| **NOT_ESTIMABLE** | N_eligible < 20 or N_cohort_dates < 3 |

The conclusion category is written to `latest_run.json` as `research_conclusion` and is immutable once written.

---

## 7. Minimum sample size

If the eligible dataset contains fewer than 20 rows total (Favourable + Mixed combined), TIME-010 reports `research_conclusion = NOT_ESTIMABLE` and does not run significance tests.

This threshold is pre-specified and may not be changed after seeing the data.

**NOT_ESTIMABLE execution semantics:** A NOT_ESTIMABLE run writes `conclusion_frozen: false` to `latest_run.json`. This allows subsequent runs to proceed as more COMPLETE observations accumulate. The conclusion is only frozen (`conclusion_frozen: true`) when a final estimable result is produced. This prevents a premature NOT_ESTIMABLE run from consuming the immutable conclusion slot before the stopping condition is reached.

## 7a. Bootstrap determinism

The bootstrap CI for Q2 (section 3.2) uses:

```
n_resamples = 10,000
seed        = 42
```

The seed is an implementation detail, not a research parameter. It is recorded in `latest_run.json` as `bootstrap_seed` for reproducibility. The seed does not affect eligibility, endpoint definition, or conclusion classification.

---

## 8. Prohibited actions

The following are explicitly prohibited in TIME-010:

1. **Changing primary endpoints** after seeing any outcome data.
2. **Adding new primary endpoints** not listed in section 3.
3. **Changing the eligibility criterion** (section 2.2) after seeing the data.
4. **Changing the significance threshold** (p < 0.05) after seeing the data.
5. **Changing the cohort consistency threshold** formula after seeing the data.
6. **Retrospective R:R threshold selection** — no filtering by `adaptive_horizon_sessions`, `rank_score`, or any other T0 field based on observed outcomes.
7. **Retrospective cohort exclusion** — no removing cohort dates based on their outcomes.
8. **Three-class framing** — Unfavourable decisions are not included in the primary comparison.
9. **Modifying Coralys** based on TIME-010 results without a new frozen protocol.
10. **Starting TIME-011** without a new frozen protocol document.

---

## 9. Reporting template

The TIME-010 report (`analysis_report.md`) must contain the following sections in order:

1. **Header:** experiment ID, frozen_at date, dataset source, N_total, N_eligible, N_cohort_dates
2. **Eligibility accounting table:** cohort × evidence_class: N_total, N_eligible, N_ineligible, ineligibility reasons
3. **Primary results — Q1 (target_reached):** rate table, test statistic, p-value, CI, cohort consistency
4. **Primary results — Q2 (realized_return):** mean/median table, test statistic, p-value, CI, cohort consistency
5. **Secondary results — Q3 through Q7:** descriptive tables
6. **Research conclusion:** one of the pre-specified categories (section 6), with justification
7. **Limitations:** mandatory paragraph covering: prospective horizon length, cohort count, eligibility exclusions, DEGRADED inclusion, any data gaps flagged by `time009_integrity.py`
8. **Frozen conclusion paragraph:** verbatim text that may not be softened or strengthened in subsequent documents

---

## 10. Artifact schema

### 10.1 Per-run summary (`latest_run.json`)

```json
{
  "experiment_id": "TIME010",
  "run_id": "TIME010-{YYYYMMDDTHHMMSSZ}",
  "run_at": "<ISO-8601>",
  "producer": "time010_analysis.v1",
  "dataset_source": "time_machine/analysis/TIME009/prospective_evidence.csv",
  "n_total_rows": <int>,
  "n_eligible_rows": <int>,
  "n_cohort_dates": <int>,
  "n_favourable_eligible": <int>,
  "n_mixed_eligible": <int>,
  "q1_target_rate_favourable": <float>,
  "q1_target_rate_mixed": <float>,
  "q1_p_value": <float>,
  "q1_significant": <bool>,
  "q1_cohort_consistency_n": <int>,
  "q1_cohort_consistency_threshold": <int>,
  "q1_cohort_consistent": <bool>,
  "q2_return_mean_favourable": <float>,
  "q2_return_mean_mixed": <float>,
  "q2_p_value": <float>,
  "q2_significant": <bool>,
  "q2_cohort_consistency_n": <int>,
  "q2_cohort_consistency_threshold": <int>,
  "q2_cohort_consistent": <bool>,
  "research_conclusion": "<POSITIVE|PARTIAL|NEGATIVE|INCONCLUSIVE|NOT_ESTIMABLE>",
  "conclusion_frozen": true,
  "protocol_version": "TIME010-v1.0",
  "prohibited_actions_acknowledged": true
}
```

### 10.2 Analysis report

`time_machine/analysis/TIME010/analysis_report.md` — human-readable report following the template in section 9.

---

## 11. Implementation plan

The analysis script `scripts/time010_analysis.py` must be written and committed **before** the TIME-009 stopping condition is reached. It must:

1. Load `prospective_evidence.csv`
2. Apply eligibility filter (read `eligible_for_primary_comparison` verbatim)
3. Compute Q1 (two-proportion z-test)
4. Compute Q2 (Mann-Whitney U)
5. Compute cohort consistency for Q1 and Q2
6. Classify research conclusion per section 6
7. Write `latest_run.json` and `analysis_report.md`
8. Refuse to run if `n_eligible < 20` (NOT_ESTIMABLE)
9. Refuse to overwrite a frozen `latest_run.json` with `conclusion_frozen: true`

---

## 12. Relationship to TIME-008

TIME-008 was a historical retrospective analysis. TIME-010 is a prospective analysis.

The primary endpoints (Q1, Q2) and cohort consistency criterion are identical to TIME-008's pre-specified tests. This is intentional: the prospective experiment is designed to test the same hypothesis under genuinely forward conditions.

TIME-008 result (frozen): **NEGATIVE** — cohort consistency criterion not met (0/6 for all primary orderings).

TIME-010 will either:
- Replicate the negative result (most likely given TIME-008)
- Find a positive or partial result under prospective conditions

Either outcome is scientifically valid. The protocol is designed to prevent the result from being influenced by the analyst's preference.

---

## 13. Stopping rule (inherited from TIME-009)

TIME-010 is triggered when the TIME-009 stopping condition is met:

```
min(20 prospective cohort dates, 6 calendar weeks from 2026-08-20)
```

Deadline: **2026-10-01**

TIME-010 may not be triggered early based on interim outcome inspection. The stopping rule is evaluated on cohort/date existence only, never on outcomes.

---

*Protocol frozen: 2026-08-21. No modifications permitted after this date.*