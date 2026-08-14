# G-Extension Methodology

**Predictive-Value Experiment Design**

**Status:** Draft --- Methodology Freeze Candidate
**Phase:** G-Extension following Phase 5.1

---

1. **Purpose and Scope**

Phase 5.1 established that the certified Knowledge Lake can be populated, consumed reproducibly, and subjected to deterministic laboratory execution. It did not establish predictive value.

G-Extension defines the scientific experiment before any predictive-value implementation is modified.

The experiment evaluates the four existing horizons:

- 5D
- 10D
- 20D
- 60D

No implementation changes should begin until this methodology is reviewed and frozen.

2. **Unit of Analysis**

The primary observational unit is a persisted Decision and its linked Assessment, together with the corresponding horizon‑specific Outcome.

The canonical lineage is:

```
Assessment
    ↓
Decision
    ↓
Strategy
    ↓
Outcome (5D / 10D / 20D / 60D)
```

A decision may only contribute features that were legitimately available at its decision timestamp.

3. **Prediction Targets**

For each horizon \(h ∈ {5D, 10D, 20D, 60D}\), define a separate prediction target.

The primary target is a binary endpoint:

```
Y_h = 1 if the horizon‑h outcome satisfies the predefined positive‑outcome rule
Y_h = 0 otherwise
```

The exact positive‑outcome rule must be taken from the existing outcome semantics and frozen before implementation. No threshold may be selected after observing results.

If an existing continuous outcome quantity is available, it may be retained as a secondary analysis target, but it must not silently replace the primary endpoint.

4. **Decision‑Time Information Set**

For every decision timestamp \(t\):

```
X_t = all information legitimately available at or before t
```

Permitted inputs are the certified decision/assessment artifacts and source observations available by t.

Forbidden inputs include:
- future prices;
- future observations;
- future outcomes;
- future strategy results;
- derived features using observations after t.

The experiment must record the feature cutoff timestamp used for every evaluation observation.

5. **Baseline / Null Models**

Predictive value must be evaluated against a frozen baseline.

**Primary baseline**

Use a constant‑probability model estimated from the training sample:

```
p_baseline = training‑set positive‑class prevalence
```

This baseline contains no decision‑specific predictive information.

Any secondary historical or rule‑based baseline must be specified before implementation and must not be selected retrospectively.

6. **Evaluation Design**

The preferred evaluation design is chronological:

```
Historical observations
        │
        ├── Training period
        ├── Validation period
        └── Chronologically later test period
```

The test period must remain untouched during feature selection, model selection, threshold selection, and hyperparameter selection.

Random shuffling across time is prohibited where it can introduce temporal leakage.

Overlapping 5D, 10D, and 60D horizons must be explicitly documented because their observations may not be statistically independent.

7. **Primary Metric**

The primary discrimination metric is ROC‑AUC.

For every horizon report:

- `AUC_h`

AUC must be accompanied by an uncertainty interval and interpreted relative to the frozen baseline.

`AUC > 0.5` alone is not sufficient evidence of predictive value.

8. **Secondary Metrics**

For every horizon report:

- **Calibration** – at minimum: Brier score; calibration intercept; calibration slope; observed vs. predicted event rate. If sample size permits, provide a reliability table by probability bin.
- **Effect size** – report candidate‑versus‑baseline improvement: `ΔAUC = AUC_candidate − AUC_baseline`. Also report Brier‑score improvement where appropriate.
- **Confidence interval** – report a 95 % confidence interval for the primary metric and principal effect‑size comparison. The CI method must be frozen before implementation.
- **P‑value** – primary hypothesis:

```
H0: candidate predictive performance is no better than the predefined baseline
H1: candidate predictive performance is better than the baseline
```

The test must account for paired predictions on the same evaluation observations.

9. **Confidence‑Interval Method**

The preferred method is a deterministic bootstrap over evaluation observations, subject to an explicit treatment of temporal dependence.

If observations are materially dependent because of overlapping horizons or other time‑series structure, the implementation must use an appropriate block/resampling strategy rather than independently resampling observations.

The random seed, number of resamples, and resampling unit must be frozen.

If the selected method cannot be justified from the available data structure, the result must be classified **INCONCLUSIVE** rather than silently substituting another method.

10. **Multiple‑Horizon Testing**

The four horizons constitute a related family of hypotheses.

The primary family‑wise procedure is proposed as:

- Holm correction
- \(α = 0.05\)

Both unadjusted and adjusted p‑values must be reported.

No horizon may be selected retrospectively because it produces the strongest result.

11. **Classification Rules**

- **PREDICTIVE_VALUE_DETECTED** – requires all four horizons to execute successfully, all required metrics present, leakage checks pass, candidate performance exceeds the frozen baseline, the primary effect is directionally positive, the relevant confidence interval supports a positive effect, and the corrected statistical test satisfies the frozen significance criterion. A single attractive horizon is insufficient unless the frozen protocol explicitly defines a single‑horizon success rule.

- **PREDICTIVE_VALUE_NOT_DETECTED** – requires complete execution, all required metrics available, leakage checks pass, valid statistical analysis, and the predefined detection criterion is not satisfied. This means the experiment was capable of detecting the specified effect but did not find sufficient evidence for it. It does **not** prove predictive information is absent in every possible formulation.

- **INCONCLUSIVE** – use when required data are missing, a required horizon cannot be evaluated, required metrics cannot be computed, the evaluation sample is insufficient, temporal leakage cannot be ruled out, statistical assumptions cannot be satisfied, the implementation does not faithfully implement this frozen methodology, or any required methodological parameter remains unfrozen. No predictive‑value claim should be made from an inconclusive run.

12. **Determinism Requirements**

The experiment must capture:
- dataset SHA‑256;
- source/binary SHA‑256;
- methodology/configuration SHA‑256;
- random seed (if applicable);
- evaluation‑period definition;
- model specification;
- output SHA‑256;
- execution timestamp;
- final classification.

Independent executions against the same frozen dataset, binary, configuration and seed must produce identical semantic results.

13. **Required Horizon Report**

The final report must contain a table such as:

| Horizon | N | Positive | AUC | 95 % CI | ΔAUC | Brier | Calibration | Adjusted p‑value | Result Rate | CI | p‑value |
|---------|---|----------|-----|--------|------|-------|------------|-------------------|------------|----|--------|
| 5D      | … | …        | …   | …      | …    | …     | …          | …                 | …          | …  | …      |
| 10D     | … | …        | …   | …      | …    | …     | …          | …                 | …          | …  | …      |
| 20D     | … | …        | …   | …      | …    | …     | …          | …                 | …          | …  | …      |
| 60D     | … | …        | …   | …      | …    | …     | …          | …                 | …          | …  | …      |

Blank values must never be silently interpreted as zero or failure.

14. **Leakage Audit**

Before accepting a predictive result, produce an explicit leakage audit covering:
- feature timestamps;
- outcome timestamps;
- training/test boundaries;
- feature construction cutoff;
- duplicate/repeated observations;
- overlapping‑horizon treatment;
- transformations fitted using future data.

A failed leakage audit forces **INCONCLUSIVE**.

15. **Evidence Bundle**

The experiment must produce an immutable evidence bundle containing:
```
G_EXTENSION/
├── methodology.md
├── configuration.json
├── dataset.sha256
├── binary.sha256
├── output.txt
├── results.md
├── witness.json
└── leakage_audit.md
```
The bundle must be sufficient for an independent reviewer to reconstruct exactly what was tested.

16. **Implementation Freeze Gate**

Before implementation begins, explicitly freeze:
- target definition;
- positive/negative outcome rule;
- feature cutoff rule;
- training/validation/test periods;
- primary baseline;
- candidate model;
- primary metric;
- secondary metrics;
- CI method;
- p‑value method;
- significance level;
- multiple‑testing correction;
- random seed;
- minimum evaluation sample requirements;
- classification rules.
Any change afterward constitutes a methodology revision and requires a new versioned methodology artifact.

17. **Scientific Claim Boundary**

The experiment answers the narrow question:
> Does the information available at decision time contain statistically demonstrable predictive information for the predefined horizon‑specific outcome, beyond the frozen baseline, under a temporally valid out‑of‑sample evaluation?

It does **not** establish investment profitability, deployability, causal market impact, robustness across all market regimes, superiority over every alternative model, or real‑world trading performance. Those questions require subsequent research.

18. **Research Sequence**

```
G-Extension Methodology
        ↓
Methodology Review
        ↓
Methodology Freeze
        ↓
Implementation
        ↓
Unit + Leakage Tests
        ↓
Deterministic G-Extension Run
        ↓
Statistical Evaluation
        ↓
G-Extension Classification
```
Governance rule: implementation must not begin before the methodology is frozen.

19. **Current Status**

**DRAFT — NOT YET FROZEN**

This document deliberately does not claim predictive value and does not prescribe implementation details that depend on unavailable evidence.

The next governance decision is to review and freeze the methodology. Only then should the predictive‑value experiment be implemented.
