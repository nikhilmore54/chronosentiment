# CS-P-006-C.2-D — Decision Value Landscape

**Document type:** Bounded measurement contract and landscape of an immutable search  
**Status:** Complete — Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.2-R, CS-P-006-C.2-S  
**Does not:** run Search #2, change Search #1, change fitness/seed/universe/horizon, invent a ±X% borderline band, turn `advantage_vs_*` or `unique_best` into Coralys fitness, treat “acted better than NO_TRADE” as an accuracy KPI, amend `csp006a.policy_artifact.1`, feed evaluation to Coralys, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy deterministically; outcomes do not re-enter discovery.

---

## What this freeze is

Specify **what decision-value measurement means**, then apply that contract to the existing 273 Search #1 recommendations. Bands are not invented from the observed returns. Advantage is observational, not a new evolutionary objective.

```text
Search #1 PolicyArtifact 9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0
        ↓
273 C.2-R recommendations + realized 20D outcomes
        ↓
decision-value landscape (continuous magnitudes)
```

This is **not** Search #2. Coralys is not re-run. No new winner is selected. Evaluation numbers are ChronoSentiment holdout diagnosis.

Sidecar: `product_validation/CS-P-006/discovery/20260814T195327Z/decision_value/`

---

## Measurement contract

Frozen before looking at new cutoffs. Costs are not certified, so they are not subtracted.

For realized raw 20-day close-to-close return `R`:

```text
value(LONG)     =  R
value(SHORT)    = −R
value(NO_TRADE) =  0
```

`NO_TRADE = 0` is the standing-aside counterfactual (“did not take the market”). It is **not** a traded 0% inside Search #1 fitness.

Then:

```text
recommended_value = value(recommended action)
best_action       = argmax { LONG, SHORT, NO_TRADE }
best_value        = max of those three
regret            = best_value − recommended_value   (≥ 0)
advantage vs each alternative = recommended_value − value(alternative)
```

Identities that follow, without inventing a band:

* If `R > 0`, LONG is uniquely best; regret of LONG is 0; regret of SHORT is `2R`; regret of NO_TRADE is `R`.
* If `R < 0`, SHORT is uniquely best; regret of LONG is `2|R|`; regret of NO_TRADE is `|R|`.
* If `R = 0`, all three actions are equal; no unique best.
* NO_TRADE is **never** uniquely best unless the market is exactly flat. Standing aside always forgoes `|R|` and always avoids `|R|`. Those two quantities are the same number. That is not “NO_TRADE was correct.”

No `±X%` borderline classifier is frozen. Quantiles of recommended value and regret are the landscape.

`advantage_vs_*` and `recommended_is_unique_best` are ChronoSentiment measurements. They are **not** Coralys fitness. CS-P-006-M is the protocol document that keeps that distinction.

Do **not** encode:

```text
unique_best = true  → fitness 1
unique_best = false → fitness 0
```

That would collapse a continuous decision-value problem back into binary classification. Do **not** invent `advantage > 1%` or `regret > 5% = bad` from this dataset.

A wrong LONG (or a wrong SHORT) has regret `2|R|`. Standing aside on the same move has regret `|R|`. Acting the losing direction therefore carries **twice** the regret of NO_TRADE. That is an identity of the three-action set, not a reason to prefer SHORT, and not a fitted threshold.

---

## Research questions this landscape poses

These are the questions C.2-D is for. They are not accuracy KPIs.

### R1. Can the state at T identify an action with materially better decision value than the alternatives?

Evaluation unique-best is **17/91 = 18.7%**. In the majority of holdout states the recommended action was not uniquely best. “Materially better” is still **OPEN** (CS-P-006-M question 9) and is not read from these quantiles.

### R2. How often is the recommended action uniquely best rather than merely positive?

| | Evaluation | All 273 |
|--|--:|--:|
| Merely positive vs NO_TRADE (acted rows) | 17/33 = 51.5% | 69/121 = 57.0% |
| **Uniquely best of LONG / SHORT / NO_TRADE** | **17/91 = 18.7%** | **69/273 = 25.3%** |

For this LONG-only sealed rule, “acted better than NO_TRADE” equals the directional hit rate. That is a **mathematical consequence** of `value(LONG)=R` when the only active action is LONG. It must **not** become another accuracy metric.

`unique_best` is the decision-intelligence share: was the recommended action better than **both** alternatives, not merely better than standing aside?

### R3. Can Coralys learn policies that reduce regret while preserving beneficial borderline opportunities, without imposing a hand-written decision boundary?

**Not answered here.** Mean evaluation regret **5.62%** and maximum regret **61.64%** are first-class **diagnostics**. They are not a fitness function. Borderline positive cases must not be discarded by a cutoff fitted to this landscape. The first specification of a later objective is CS-P-006-M.1 (expected `V`, not `−regret`). That is not Search #2.

---

## Diagnostic overlay (kept separate from R1–R3)

These restate C.2-R so the three diagnostics are not collapsed into one number.

### D1. Directional validity

Did the direction agree? Already answered by C.2-R. Evaluation: **17/33 = 51.5%**. Descriptive only. Not the headline of this freeze.

### D2. Economic value when acted

How much was gained or lost when the policy acted?

| Slice | Acted (all LONG) | Mean signed 20D |
|-------|-----------------:|----------------:|
| Development | 49 | **+1.40%** |
| Selection | 39 | **+1.57%** |
| Evaluation | 33 | **−1.61%** |

Same magnitudes as C.2-R. The holdout mean is a left tail, not a uniform small loss (median of the 33 evaluation LONGs remains **+0.71%**).

### D3. Decision value versus alternatives

Was the recommended action better than LONG / SHORT / NO_TRADE, and by how much?

| Slice | n | Acted | Stood aside | Mean recommended value (NO_TRADE=0) | Mean regret vs best of 3 | Acted better than NO_TRADE | Unique best of 3 |
|-------|---|------:|------------:|------------------------------------:|-------------------------:|---------------------------:|-----------------:|
| all | 273 | 121 | 152 | +0.28% | 4.92% | 69/121 = 57.0% | 69/273 = 25.3% |
| development | 91 | 49 | 42 | +0.76% | 4.78% | 29/49 = 59.2% | 29/91 = 31.9% |
| selection | 91 | 39 | 52 | +0.67% | 4.38% | 23/39 = 59.0% | 23/91 = 25.3% |
| **evaluation** | **91** | **33** | **58** | **−0.58%** | **5.62%** | **17/33 = 51.5%** | **17/91 = 18.7%** |

“Acted better than NO_TRADE” is shown only as an identity check on this LONG-only policy. It is not an accuracy KPI. **Unique best of three** is the stricter share: NO_TRADE is never uniquely best, so 152 stand-asides contribute zero. Evaluation unique-best is 17/91, not 17/33.

Regret of a wrong LONG is `2|R|`. Evaluation maximum regret is **61.64%** — twice the IDEA 2024-08-31 −30.82% move. That is the asymmetric-magnitude problem stated as missed alternative, not as “incorrect.”

---

## Continuous landscape (no bands)

**Recommended value** (all rows; standing aside contributes 0):

| Slice | Mean | Median | P25 | P75 | Min | Max |
|-------|-----:|-------:|----:|----:|----:|----:|
| all | +0.28% | 0.00% | 0.00% | +0.05% | −30.82% | +26.24% |
| development | +0.76% | 0.00% | 0.00% | +2.24% | −11.98% | +26.24% |
| selection | +0.67% | 0.00% | 0.00% | +0.72% | −14.05% | +18.51% |
| evaluation | −0.58% | 0.00% | 0.00% | 0.00% | −30.82% | +17.88% |

**Regret versus the best of LONG / SHORT / NO_TRADE:**

| Slice | Mean | Median | P25 | P75 | Min | Max |
|-------|-----:|-------:|----:|----:|----:|----:|
| all | 4.92% | 2.76% | 0.00% | 6.67% | 0.00% | 61.64% |
| development | 4.78% | 2.73% | 0.00% | 6.93% | 0.00% | 41.87% |
| selection | 4.38% | 2.81% | 0.00% | 6.36% | 0.00% | 28.10% |
| evaluation | 5.62% | 2.72% | 0.30% | 6.97% | 0.00% | 61.64% |

**Acted advantage versus standing aside** (the 121 LONGs):

| Slice | Mean | Median | P25 | P75 | Min | Max |
|-------|-----:|-------:|----:|----:|----:|----:|
| all | +0.64% | +0.96% | −3.03% | +4.39% | −30.82% | +26.24% |
| development | +1.40% | +1.79% | −3.57% | +4.69% | −11.98% | +26.24% |
| selection | +1.57% | +1.17% | −1.41% | +3.75% | −14.05% | +18.51% |
| evaluation | −1.61% | **+0.71%** | −5.81% | +3.81% | −30.82% | +17.88% |

**NO_TRADE opportunity cost** (`|R|` on the 152 stand-asides; also the loss avoided):

| Slice | Mean | Median | P25 | P75 | Min | Max |
|-------|-----:|-------:|----:|----:|----:|----:|
| all | 5.09% | 3.64% | 1.67% | 6.67% | 0.06% | 41.87% |
| development | 5.93% | 4.35% | 1.78% | 6.57% | 0.32% | 41.87% |
| selection | 5.57% | 4.59% | 2.27% | 7.24% | 0.06% | 19.89% |
| evaluation | 4.04% | 2.79% | 1.22% | 6.23% | 0.11% | 16.49% |

On selection, standing aside missed a +1.21% average signed move (C.2-R). The decision-value statement is stronger: the typical absolute move stood aside from was **5.57%**. Standing aside was not “approximately nothing happened.”

These quantiles are the landscape. They are **not** cutoffs for a later search.

---

## What this does not authorize

* C.3 / Search #2
* Teaching Coralys a borderline boundary
* Using `advantage_vs_no_trade`, regret, or `unique_best` as a fitness function, including `unique_best → {0,1}`
* Inventing `advantage > 1%` or `regret > 5% = bad` from these returns
* A cost or drawdown term invented from these numbers
* Selecting a new PolicyArtifact
* Feeding evaluation regret back into the same search
* Calling NO_TRADE “correct”
* Adding indicators because the holdout failed

Search #1 stays frozen. CS-P-003 stays independent. The protocol questions that must be answered before any later search are CS-P-006-M.

---

## Code

| Piece | Location |
|-------|----------|
| Analysis | `adapters/chronosentiment/src/decision_support/decision_value_landscape.rs` |
| Binary | `src/bin/csp006_decision_value.rs` |
| Runner | `run_csp006_decision_value.sh` |
| Sidecar | `product_validation/CS-P-006/discovery/20260814T195327Z/decision_value/` |
| Tests | `adapters/chronosentiment/tests/csp006c2d_decision_value_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
