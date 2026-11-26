# G-GATE v1.1 Statistical Interpretation / Closure Report

**Status:** CLOSED — v1.1 G-GATE cycle complete. Research question closed under v1.1 (Decision A).  
**Protocol:** G-Extension Methodology v1.1 (frozen; not modified by this document)  
**Dataset interpreted:** B4 (`r3_evidence/20260814T023457Z_B4/`)  
**B4 dump SHA-256:** `f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6`  
**v1.1 methodology SHA-256:** `e129d7add66d7f4c12aab14811a3d552abf6b603f012eeb75c99c484e0065e66`  
**v1.1 manifest SHA-256:** `1604563a0a4516cbe983ef398ad36b6e1daacc8842b7a8daa28812e8ffee958e`  
**B4 G-GATE execution timestamp:** `2026-08-14T02:38:05.534290+00:00`  
**Classification recorded by the run:** `INCONCLUSIVE`  
**Predictive-value claim:** `NOT_ESTABLISHED`

**Authoritative conclusion:**

> Predictive value was not established under the v1.1 protocol. B4 passed leakage and lineage; the experiment remained inconclusive because required bootstrap inference was undefined for some horizons.

This document does **not** change v1.1, `Y_h`, the candidate, the split, the bootstrap, B3, or B4. It does **not** authorize B5, a v1.1 rerun, or v1.2.

---

## 1. Governance boundary (closed)

```text
B3  (frozen, unmodified)
 └─ G-GATE v1.1 → INCONCLUSIVE / LEAKAGE FAIL
                ↓
        Root cause identified (assessment stamped with Utc::now())
                ↓
        Population repair (assess_at(replay dt))
                ↓
B4
 ├─ Temporal invariant 195/195 PASS
 ├─ E-GATE v3 → PASS
 └─ G-GATE v1.1 (protocol unchanged)
       ├─ Leakage → PASS
       └─ Predictive value → INCONCLUSIVE
```

Leakage is a data-integrity question. It is closed on B4. Predictive value is a statistical-methodology question. The B4 run answered it under v1.1: **not established**, because required inference was incomplete.

v1.1’s dataset clause names B3. B4 is the repaired generator output used after E-GATE v3 PASS so that the same frozen protocol could be executed with a valid as-of column. Rank sizes remain 55 / 27 / 28. B3 UUID boundary identities do not apply to B4.

---

## 2. What the B4 run actually showed

Source: `r3_evidence/20260814T023457Z_B4/G_GATE/G_GATE_REPORT.md` and `G_GATE_WITNESS.json`.

Leakage audit: **PASS** (nine checks, including feature timestamps `<=` decision `evaluation_timestamp`).

Test fold: **N = 28** on every horizon (`N >= 20`, both classes present on the original test sample).

| Horizon | n_pos | n_neg | Point AUC | Undefined bootstrap AUCs | AUC CI / p | Calibration (OLS) | Horizon metrics_defined |
|---------|-------|-------|-----------|--------------------------|------------|-------------------|-------------------------|
| 5D | 12 | 16 | 0.50 | **2 / 10000** | undefined | defined | false |
| 10D | 12 | 16 | 0.50 | 0 / 10000 | p = 1.0; CI = [0.50, 0.50] | **undefined** (`Var(p_hat)=0`) | false |
| 20D | 7 | 21 | 0.50 | 0 / 10000 | p = 1.0; CI = [0.50, 0.50] | defined | **true** |
| 60D | 6 | 22 | 0.50 | **4 / 10000** | undefined | defined | false |

Holm-adjusted p is **undefined on all four horizons**. v1.1 §9 sets the Holm family to the four horizons. Missing unadjusted p on 5D and 60D makes the family incomplete.

Point AUCs of 0.50 and ΔAUC = 0 are **descriptive**. They are not a classification of `PREDICTIVE_VALUE_NOT_DETECTED`.

---

## 3. Why some bootstrap AUCs are undefined

v1.1 §8 (primary metric, applied to any evaluation sample used for AUC):

> If `n_pos = 0` or `n_neg = 0`, AUC is undefined and that horizon is **INCONCLUSIVE**.

v1.1 §9 draws **B = 10000** moving-block bootstrap resamples of the **test** strategies (block length `L = 5`, non-wrapping, last incomplete block kept). Each resample is a new 28-row sample. Mann–Whitney AUC on that sample is undefined whenever the resample contains only one class.

That event is possible under the frozen sampler:

- Test N = 28, L = 5 → six chronological blocks (five of length 5, one of length 3).
- Blocks are drawn with replacement until 28 observations are obtained, then truncated.
- A draw that over-represents negative (or positive) blocks can produce `n_pos = 0` or `n_neg = 0`.
- 60D has only 6 positives in the original test fold, so single-class resamples are more likely than on 10D/20D. The observed counts (2 on 5D, 4 on 60D) are small relative to B, but **non-zero**.

This is not an implementation bug relative to §8. It is a property of block-resampling a binary label with N = 28.

---

## 4. Does v1.1 specify a fallback when bootstrap AUCs are undefined?

**No permitted fallback is written.** v1.1 does not say any of the following:

- drop undefined replicates and form CI/p from `B' < B`;
- impute undefined `AUC*` as 0.5;
- retry the PRNG until B defined AUCs exist;
- reduce B, change L, or switch to ordinary (non-block) bootstrap after seeing results;
- classify from defined horizons only;
- treat incomplete Holm as `PREDICTIVE_VALUE_NOT_DETECTED`.

What v1.1 **does** specify:

| Clause | Rule that applies |
|--------|-------------------|
| §9 | AUC CI is the percentile interval of **the B bootstrap AUCs**. P-value is `(1 + #{b : ΔAUC^(b) ≤ 0}) / (B + 1)` with **B = 10000**. Holm family = **the four horizons**. |
| §10 | If `Var(p_hat) = 0` on the test fold, calibration slope is undefined and **that horizon is INCONCLUSIVE**. |
| §11 | `PREDICTIVE_VALUE_DETECTED` requires all four horizons to execute successfully (**metrics defined**), leakage PASS, every ΔAUC > 0, every ΔAUC CI lower bound > 0, and every Holm-adjusted p < 0.05. |
| §11 | `PREDICTIVE_VALUE_NOT_DETECTED` requires **complete execution, all required metrics available**, leakage PASS, and detection not satisfied. |
| §11 | **INCONCLUSIVE** if any of: a required horizon cannot be evaluated; **a required metric cannot be computed**; leakage FAIL; … |
| §11 | No predictive-value claim may be made from an inconclusive run. |
| §8 | `AUC > 0.5` alone is not sufficient for detection. |

If any bootstrap replicate has undefined AUC, the vector of B AUCs required by §9 does not exist. CI and p for that horizon cannot be computed as specified. Those are required metrics. §11 therefore forces **INCONCLUSIVE**.

10D is independently INCONCLUSIVE under §10: test `p_hat` has zero variance (one reliability bin; OLS `sxx = 0`), so calibration slope is undefined even though all 10000 bootstrap AUCs were defined.

Because Holm’s family is the four horizons, incomplete p-values also block Holm-adjusted p on 20D, the only horizon whose own metric set is complete.

The experiment binary’s classification (`INCONCLUSIVE` when `!all_defined`) is the operationalization of §11. It is not a post-hoc rule.

---

## 5. Permitted statements

**Permitted:**

> Predictive value was not established under the v1.1 protocol. The B4 dataset passed the temporal-leakage and lineage gates, but the predictive-value experiment remained inconclusive because the required bootstrap inference was undefined for some horizons.

**Also permitted, as protocol status:**

- B3 remains frozen. B3 G-GATE v1.1 remains `INCONCLUSIVE` / leakage FAIL.
- B4 E-GATE v3 PASS and B4 G-GATE leakage PASS close the timestamp-population defect.
- v1.1 is finished as an executable protocol on B4: the classification is `INCONCLUSIVE`.
- Further predictive-value evidence requires a **new methodology version (v1.2+)**, frozen before another run.

**Not permitted:**

- “ChronoSentiment has no predictive value.”
- “ChronoSentiment demonstrates predictive value.”
- Re-labelling B4 as `PREDICTIVE_VALUE_NOT_DETECTED` because point AUC = 0.50.
- Regenerating B5, mutating B4, or re-running G-GATE v1.1 to avoid undefined bootstrap AUCs.
- Changing B, L, seed, candidate, `Y_h`, split, or Holm handling inside v1.1.

AUC = 0.50 on every horizon is expected when the candidate often returns the training prevalence (unseen `signature_hash` → `p_train_h`, v1.1 §5). That description is not a negative detection claim.

---

## 6. Research decision (recorded)

**Decision A is selected.** The v1.1 research question is closed. Predictive value was not established. Work proceeds to other research or product activity.

There is no remaining engineering defect to chase in the current protocol. There is no integrity reason to regenerate B5, mutate B4, or rerun G-GATE under v1.1. Point AUC = 0.50 must not be reinterpreted as “no predictive value.”

**Decision B (v1.2) is not opened.** Opening v1.2 would require a separate, explicit authorization and a new freeze *before* any further experiment: bootstrap handling when `AUC*` is undefined; minimum effective defined resamples; candidate / features; split; minimum N; Holm family rules. Those would be new methodological choices, not recovered v1.1 intent. No such freeze exists. No additional G-GATE execution is authorized.

---

## 7. Alignment

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no invented methodology; claims must not outrun frozen evaluation. This closure records the frozen v1.1 outcome. It does not add a model, a split, or a detection rule.

`docs/PRD_v3_3.md` / evidence programme: this is technical evidence that the G-GATE protocol was executed and classified. It is not a product demonstration of trading performance (`G_Extension_Methodology_v1.1.md` §1, §17).
