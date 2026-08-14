# G-Extension Methodology v1.1

**Predictive-Value Experiment Design — Executable Freeze**

**Status:** FROZEN for G-GATE execution  
**Supersedes:** `G_Extension_Methodology.md` (v1.0, hash-authenticated, scientifically non-executable)  
**Companion:** `G_Extension_Methodology_v1.1_TrainTestSplit.md`  
**Dataset:** B3 only (`chrono_b3_test` / `r3_evidence/20260813T035739Z_B3/`)  
**Deterministic seed:** `20260813`

v1.0 is preserved unchanged as a historical artifact. v1.1 does not overwrite it.

This version inherits v1.0 scientific structure and records **explicit freeze decisions** for the five v1.0 omissions that blocked execution. Those decisions are **new methodological choices**, not recovered authorial intent.

---

## 0. Freeze-gate completion

| Parameter | v1.0 | v1.1 freeze |
|-----------|------|-------------|
| Positive-outcome definition `Y_h` | Required, unspecified | §3 |
| Candidate model / features | Required, unspecified | §5 |
| CI / resampling procedure | Preferred bootstrap; n, unit, blocks unspecified | §9 |
| Train / validation / test windows | Example only; B2 referenced | Companion v1.1 split document |
| Minimum evaluation N | Required, unspecified | §16 |

---

## 1. Purpose and Scope

The experiment answers:

> Does the information available at decision time contain statistically demonstrable predictive information for the predefined horizon-specific outcome, beyond the frozen baseline, under a temporally valid out-of-sample evaluation?

Horizons: `5D`, `10D`, `20D`, `60D`. All four are primary. No horizon may be selected after seeing results.

This does **not** establish investment profitability, deployability, causal market impact, regime robustness, or live trading performance.

---

## 2. Unit of Analysis

Primary unit: one Strategy and its four horizon-specific Outcomes, linked through:

```
Assessment → Decision → Strategy → Outcome (5D / 10D / 20D / 60D)
```

Join path (frozen):

```sql
knowledge_strategies s
JOIN knowledge_decisions d ON s.decision_id = d.id
JOIN knowledge_assessments a ON d.assessment_id = a.id
JOIN knowledge_outcomes o ON o.strategy_id = s.id
```

A strategy may not be split across folds. All four outcomes move together.

Feature cutoff: only artifacts and fields whose evaluation timestamp is `<=` the decision `evaluation_timestamp`.

---

## 3. Prediction Targets

For each horizon `h ∈ {5D, 10D, 20D, 60D}`:

```
Y_h = 1  if  knowledge_outcomes.outcome_return > 0
Y_h = 0  otherwise
```

**Source of the field:** v1.0 TrainTestSplit §1 names `outcome_return` as the quantity “used only for the primary endpoint.”

**Source of the threshold:** explicit v1.1 decision. The mapping is the sign of realized return. The threshold `0` is pre-specified and must not be tuned on validation or test data.

Ties at exactly `0` are negative (`Y_h = 0`).

`target_hit`, `stop_hit`, `entry_reached`, and `exit_reason` are **not** the primary endpoint. Continuous `outcome_return` may be reported as a secondary descriptive statistic only. It must not replace `Y_h`.

---

## 4. Decision-Time Information Set

```
X_t = certified Assessment and Decision artifacts with evaluation_timestamp ≤ t
```

**Permitted feature (frozen, exhaustive):**

- `knowledge_assessments.signature_hash` of the assessment linked by `knowledge_decisions.assessment_id`

**Forbidden:** future prices; future observations; future outcomes; future strategy results; any field derived from observations after `t`; validation or test labels; `outcome_return`, `target_hit`, `stop_hit`, `mfe`, `mae`, `drawdown`, `exit_reason` as features.

The feature cutoff timestamp for every observation is the decision `evaluation_timestamp`.

No additional features may be added without a new methodology version.

---

## 5. Candidate Model

**Explicit v1.1 decision.** Not recovered from v1.0.

For each horizon `h` independently, the candidate is a **train-only empirical lookup**:

```
p_hat_h(signature_hash) =
    n_pos_train(signature_hash, h) / n_train(signature_hash, h)
```

If `signature_hash` is unseen in the training fold for that horizon, or if `n_train(signature_hash, h) = 0`:

```
p_hat_h = p_train_h
```

where `p_train_h` is the training-set positive-class prevalence for horizon `h` (identical to the baseline probability).

No hyperparameters. No validation-set model selection. No test-set fitting. No regularization search.

Validation is used only for leakage audit and descriptive reporting. It must not affect `p_hat`.

---

## 6. Baseline / Null Model

Inherited from v1.0, unchanged:

```
p_baseline_h = training-set positive-class prevalence for horizon h
```

Constant for every evaluation observation of that horizon. Contains no decision-specific information.

A constant score has ROC-AUC `0.5` when AUC is defined. Therefore:

```
AUC_baseline_h = 0.5
ΔAUC_h = AUC_candidate_h − 0.5
```

No secondary baseline is used.

---

## 7. Evaluation Design

Chronological, strategy-contiguous, B3-only. Exact windows: companion v1.1 split document.

```
TRAIN       = 55 strategies  (ranks 1–55)
VALIDATION  = 27 strategies  (ranks 56–82)
TEST        = 28 strategies  (ranks 83–110)
```

Ordering: `evaluation_timestamp ASC`, tie-break `strategy_id ASC`.

The test fold is the sole confirmatory evaluation set.

Random shuffling across time is prohibited.

Overlapping horizons (same strategy, 5D/10D/20D/60D) are dependent. Calendar overlap between fold outcome-expiry windows is documented in the split companion and in the leakage audit. It does not alter the frozen ranks.

---

## 8. Primary Metric

ROC-AUC on the **test** fold, per horizon, using `p_hat_h` vs `Y_h`.

Computation (frozen):

1. Sort test observations by `p_hat` ascending; ties by `strategy_id` ascending.
2. Mann–Whitney / Wilcoxon form:

```
AUC = (n_pos_neg_pairs_correct + 0.5 * n_pos_neg_pairs_tied) / (n_pos * n_neg)
```

If `n_pos = 0` or `n_neg = 0`, AUC is undefined and that horizon is **INCONCLUSIVE**.

`AUC > 0.5` alone is not sufficient for `PREDICTIVE_VALUE_DETECTED`.

---

## 9. Confidence-Interval and P-Value Method

**Explicit v1.1 decision.**

| Item | Freeze |
|------|--------|
| Method | Deterministic moving-block bootstrap |
| Seed | `20260813` |
| PRNG | 32-bit LCG: `state := (1103515245 * state + 12345) mod 2^31`; initial `state = 20260813`; draws `state / 2^31` |
| Resamples `B` | `10000` |
| Evaluation unit (per horizon) | One observation per test strategy |
| Block construction | Chronologically ordered test strategies (same order as the split) |
| Block length `L` | `5` strategies |
| Wrap | Non-wrapping; the last incomplete block is kept as a shorter block |
| Draw | Sample blocks with replacement until `n_test` observations are obtained; truncate to `n_test` |
| Pairing | The same resampled strategy index sequence is used for candidate scores and labels (baseline AUC remains 0.5) |
| AUC CI | Percentile 95% interval: empirical 2.5th and 97.5th percentiles of the `B` bootstrap AUCs |
| ΔAUC CI | The same percentiles applied to `AUC* − 0.5` |
| P-value (one-sided) | `p = (1 + #{b : ΔAUC^(b) ≤ 0}) / (B + 1)` |
| Multiple testing | Holm, `α = 0.05`, family = the four horizons |
| Report | Unadjusted `p` and Holm-adjusted `p` for every horizon |

If `n_test < L`, set `L = n_test` (full-sample block; reduces to ordinary bootstrap of size `n_test`).

This block bootstrap is the frozen treatment of temporal dependence among chronologically adjacent strategies. Horizon overlap within a strategy is removed by evaluating each horizon separately (one row per strategy per horizon).

---

## 10. Secondary Metrics

For every horizon, on the **test** fold:

- `N`, `n_pos`, `n_neg`
- Brier score: `mean((p_hat − Y)^2)`
- Brier baseline: `mean((p_baseline − Y)^2)`
- ΔBrier: `Brier_baseline − Brier_candidate` (positive = improvement)
- Calibration intercept `a` and slope `b` from OLS `Y = a + b * p_hat` on test observations
- Observed event rate: `mean(Y)`
- Predicted event rate: `mean(p_hat)`
- Reliability table: five equal-count bins by `p_hat` (ties broken by `strategy_id`). If fewer than five distinct `p_hat` values, one bin per distinct value.

If `Var(p_hat) = 0` on the test fold, calibration slope is undefined and that horizon is **INCONCLUSIVE**.

---

## 11. Classification Rules

Inherited from v1.0; operationalized as follows.

**PREDICTIVE_VALUE_DETECTED** requires **all** of:

1. All four horizons execute successfully (metrics defined).
2. Leakage audit PASS (§14).
3. For every horizon: `ΔAUC_h > 0`.
4. For every horizon: lower bound of the 95% ΔAUC CI `> 0`.
5. For every horizon: Holm-adjusted `p_h < 0.05`.

A single attractive horizon is insufficient.

**PREDICTIVE_VALUE_NOT_DETECTED** requires complete execution, all required metrics available, leakage PASS, and the detection criterion not satisfied.

**INCONCLUSIVE** if any of: missing required data; a required horizon cannot be evaluated; a required metric cannot be computed; `N`, `n_pos`, or `n_neg` fails §16; leakage FAIL; implementation does not match this document; any required parameter is still unfrozen.

No predictive-value claim may be made from an inconclusive run.

---

## 12. Determinism Requirements

The run must record:

- B3 dataset SHA-256
- methodology manifest SHA-256 (v1.1)
- per-file methodology SHA-256
- experiment binary SHA-256
- configuration (this protocol, seed `20260813`, `B=10000`, `L=5`)
- evaluation-period definition (companion split)
- model specification (§5)
- output SHA-256
- execution timestamp
- final classification

Independent executions against the same frozen dataset, binary, configuration, and seed must produce identical semantic results. Floating-point values in reports are rounded to 10 decimal places using round-half-away-from-zero.

---

## 13. Required Horizon Report

| Horizon | N | Positive | Negative | AUC | 95% CI | ΔAUC | ΔAUC 95% CI | Brier | Cal. intercept | Cal. slope | p-value | Holm-adjusted p | Classification contribution |
|---------|---|----------|----------|-----|--------|------|-------------|-------|----------------|------------|---------|-----------------|------------------------------|
| 5D | | | | | | | | | | | | | |
| 10D | | | | | | | | | | | | | |
| 20D | | | | | | | | | | | | | |
| 60D | | | | | | | | | | | | | |

Blank values must never be interpreted as zero.

---

## 14. Leakage Audit

Produce `leakage_audit.md` covering:

1. Feature timestamps `<=` decision `evaluation_timestamp`.
2. Labels (`outcome_return` / `Y_h`) not used as features.
3. Lookup fitted on TRAIN only.
4. VALIDATION not used for fitting, selection, or thresholding.
5. TEST unused until final evaluation.
6. No duplicate `strategy_id` in a fold.
7. Cluster constraint: all four horizons of a strategy in the same fold.
8. Calendar overlap of outcome-expiry windows across folds (document; do not re-split).
9. No scaler, encoder, or prevalence estimated from validation or test.

Any FAIL forces **INCONCLUSIVE**.

---

## 15. Evidence Bundle

```
G_EXTENSION/
├── methodology.md          (copy of this v1.1 file)
├── train_test_split.md     (copy of v1.1 split companion)
├── configuration.json
├── dataset.sha256
├── binary.sha256
├── methodology_manifest.sha256
├── output.txt
├── results.md
├── witness.json
└── leakage_audit.md
```

Place under the B3 evidence tree without modifying the frozen B3 dump or v1.0 provenance files.

---

## 16. Minimum Evaluation Sample

**Explicit v1.1 decision.**

For each horizon on the **test** fold, all of the following are required to compute metrics:

- `N >= 20`
- `n_pos >= 1`
- `n_neg >= 1`

Otherwise that horizon is **INCONCLUSIVE**, and the overall classification is **INCONCLUSIVE**.

Validation fold sample size is not a detection criterion.

---

## 17. Scientific Claim Boundary

Unchanged from v1.0 §17. A detected result is evidence of decision-time association with the frozen binary endpoint on this B3 chronological test fold. It is not a trading mandate.

---

## 18. Implementation Freeze Gate (closed)

| Item | Frozen by |
|------|-----------|
| Target definition | §3 |
| Positive / negative rule | §3 |
| Feature cutoff / feature set | §4 |
| Train / validation / test | v1.1 split companion |
| Primary baseline | §6 |
| Candidate model | §5 |
| Primary metric | §8 |
| Secondary metrics | §10 |
| CI method | §9 |
| P-value method | §9 |
| Significance level | Holm `α = 0.05` |
| Multiple-testing correction | Holm over 4 horizons |
| Random seed | `20260813` |
| Minimum evaluation N | §16 |
| Classification rules | §11 |

Any later change requires v1.2+.

---

## 19. Status

**FROZEN — v1.1**

v1.0 remains the historically recovered, hash-authenticated draft. G-GATE execution must use **v1.1** only.
