# CS-P-006-C.2-S — Selection and decision-value review

**Document type:** Bounded protocol review of an immutable search  
**Status:** Complete — Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.2-P, CS-P-006-C.2-R  
**Does not:** run Search #2, change Search #1, change fitness/seed/universe/horizon, choose a volatility encoding, freeze a borderline cutoff, amend `csp006a.policy_artifact.1`, feed evaluation to Coralys, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: diagnose the sealed experiment; do not invent methodology; accuracy is not the discovery objective.

---

## What this freeze is

A review of **how Search #1 converted a rich population into one sealed policy**, and whether the frozen fitness/selection notion represents decision value.

```text
C.2-O archive + sealed PolicyArtifact 9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0
        ↓
selection bottleneck + continuous outcome distributions
```

This is **not** Search #2. No borderline threshold is invented from these returns. Accuracy is a descriptive share of positive traded outcomes, not the headline KPI and not the Coralys objective.

Sidecar: `product_validation/CS-P-006/discovery/20260814T195327Z/selection_review/`

---

## Answers

### 1. Why were only two generation-best genomes passed into selection?

Because `evolve_on_development` builds the selection pool from `generation_history` plus `global_best`, then deduplicates. Coralys MOGA did not return the living population of 32. CS-P-006-C already recorded that limitation.

```text
384 evaluated slots
        ↓
generation-best archive (12 entries, 2 unique identities)
        ↓
selection compares 2 genomes
        ↓
one sealed PolicyArtifact
```

Identities: selected `d8363a93…` (generation 0–1 best) and development-best `9eb80355…` (generation 2–11 best).

### 2. Was that an intentional frozen protocol decision?

**Partly.** CS-P-006-B requires: search on development, **select one** sealed candidate on selection, never use evaluation for learning. It speaks of a “candidate population,” not “generation-best only.”

The two-genome pool is therefore an **implementation bottleneck**, not a B-protocol rule. Search #1 stays immutable. Widening the pool would be a later instrument change, not a silent rewrite of this run.

### 3. How many near-best candidates could have materially changed selection?

C.2-O archived **43** unique genomes (winner + 42 others) with serialized rules. Each was scored on the **selection** slice with the frozen protocol mean.

```text
Near-best / archived alternatives that beat d8363a93… on selection:  0
Momentum-rich among those:                                           0
```

Among the development plateau we can now see, **expanding selection to those 42 genomes would not have changed Search #1’s winner.** The bottleneck is real. For this archive, it was not decisive.

### 4. What happens to Momentum-rich candidates between development and selection?

They are present on development and lose on selection.

| Genome | Momentum | Development protocol | Selection protocol |
|--------|----------|---------------------:|-------------------:|
| Selected `d8363a93…` | no | +0.016325 | **+0.019938** |
| Development-best `9eb80355…` | yes | **+0.017399** | +0.009064 |

The richer genome trades less (34 vs 49 development rows; 26 vs 39 selection rows) and is more selective. Selection still prefers the simpler Bearish → LONG rule. That is not “Momentum was never tried.” It is “Momentum-containing elites did not win the frozen selection score.”

### 5. How sensitive is selection to individual instruments?

Leave-one-out on the selection slice (protocol mean, MAHABANK removed):

| Genome | Selection (7 names) | Selection without MAHABANK |
|--------|--------------------:|---------------------------:|
| Selected | +0.019938 | +0.009747 |
| Development-best | +0.009064 | −0.005361 |

MAHABANK inflates the selected score. The selected genome **still beats** the development-best without it. The preference for the simple rule is not only a four-bar MAHABANK artifact. C.1’s visibility finding stands; this is not a ticker drop.

### 6. Full outcome distribution of the two protocol elites

Headline metric is the **distribution**, not a correct/incorrect count. Share-positive is descriptive.

**Selected `d8363a93…` (Bearish → LONG)**

| Slice | n traded | Stood aside | Protocol mean | Row mean | Median | P25 | P75 | Best | Worst | + / − | Sum simple | Compounded | Max DD |
|-------|---------:|------------:|--------------:|---------:|-------:|----:|----:|-----:|------:|------:|-----------:|-----------:|-------:|
| development | 49 | 42 | +1.63% | +1.40% | +1.78% | −3.57% | +4.69% | +26.2% | −12.0% | 29 / 20 | +0.688 | +79.5% | 46.2% |
| selection | 39 | 52 | +1.99% | +1.57% | +1.17% | −1.41% | +3.98% | +18.5% | −14.0% | 23 / 16 | +0.612 | +72.2% | 23.4% |
| evaluation | 33 | 58 | −0.02% | −1.61% | **+0.71%** | −5.81% | +3.81% | +17.9% | −30.8% | 17 / 16 | −0.530 | −49.7% | 71.3% |

**Development-best `9eb80355…` (TMV, can emit SHORT)**

| Slice | n traded | Stood aside | Protocol mean | Row mean | Median | P25 | P75 | Best | Worst | + / − | Sum simple | Compounded | Max DD |
|-------|---------:|------------:|--------------:|---------:|-------:|----:|----:|-----:|------:|------:|-----------:|-----------:|-------:|
| development | 34 | 57 | +1.74% | +1.44% | +2.04% | −3.57% | +4.86% | +12.5% | −9.0% | 21 / 13 | +0.491 | +55.3% | 31.3% |
| selection | 26 | 65 | +0.91% | +0.71% | +0.77% | −2.31% | +3.20% | +18.5% | −14.0% | 13 / 13 | +0.184 | +13.7% | 29.0% |
| evaluation | 24 | 67 | −1.56% | −2.64% | **+0.71%** | −6.63% | +2.65% | +17.9% | −30.8% | 12 / 12 | −0.634 | −53.6% | 66.4% |

Evaluation of the richer elite is holdout diagnosis only. It was **not** used to choose or retune. Both elites fail to generalize. The richer one is not a hidden holdout rescue.

### 7. Does the current fitness function reflect a useful trading decision?

It reflects **mean signed 20-day traded return, equal-weighted across seven names**, with NO_TRADE standing aside and an untraded name contributing 0.

It does **not** include: transaction costs, drawdown, duration, intra-horizon path, or a consistency penalty. Accuracy is not the objective (that part is right). Economic value in the CS-P-006-V sense is not the objective either.

A policy of many small gains can beat a policy of rare large gains, or the reverse, depending only on the mean. Search #1 therefore cannot be read as “Coralys optimized decision quality.” It optimized that scalar.

### 8. Are borderline outcomes discarded?

**Not by fitness.** Every traded magnitude enters the mean. No `+0.2% = correct` rule is in Coralys.

C.2-R’s correct/incorrect overlay was too crude for this programme and is **not** reused as a classifier here. A borderline band is **not frozen**. If one is ever wanted, it must come from a separately frozen methodology or an explicit certified cost — not from peeking at these returns.

### 9. Is NO_TRADE evaluated as opportunity cost?

In fitness: **standing aside**, not a zero-return trade. Those rows are excluded from the traded mean. That is protocol-correct and is not “NO_TRADE = 0%.”

Opportunity cost is visible only in diagnosis:

| Genome | Selection NO_TRADE n | Mean raw 20D after standing aside |
|--------|--------------------:|----------------------------------:|
| Selected | 52 | +1.21% |
| Development-best | 65 | +1.62% |

Fitness does not reward avoiding a later loss or penalize missing a later gain except by omitting the row. That is weaker than a true opportunity-cost objective.

### 10. Does the 20-day objective represent the decision problem we want?

It is a legitimate **first discovery** objective: observation-path, not the 60D lake, opposite sign for SHORT, seven-name aggregation.

It is **not** yet the ChronoSentiment co-pilot objective. It ignores costs, risk, holding period, and path. CS-P-006-V already said price direction is not economic value. C.2-S does not replace the 20-day horizon. It records that the horizon is a research choice, not a product complete.

---

## What this changes about Search #1

We can now say, together with C.2-P:

> Search #1 did not fail because Momentum or SHORT were excluded from the search space.

And we can add:

> Among archived near-bests, Search #1 also did not fail because selection never saw the richer genomes. Those genomes lost on the selection score. Both the simple winner and the richer development-best fail on evaluation. The live question is the **objective and the regime**, not missing TMV inputs.

```text
Need more factors?          not supported by this archive
Need volatility encoding?   still an open information question; not justified as a Search #2 rescue
Need to understand selection + decision value?   yes — this freeze
Need Search #2?             not authorized
```

---

## What C.2-S does not authorize

* C.3 / Search #2
* Changing the selection pool, fitness, seed, horizon, or universe
* Teaching Coralys a borderline boundary
* A cost or drawdown term invented from these numbers
* Promoting `9eb80355…` after seeing evaluation
* A new PolicyArtifact schema or volatility encoding

Whether a later search should present a wider candidate pool, or optimize a different decision-value scalar, is a **later freeze**. Neither is this document.

---

## Code

| Piece | Location |
|-------|----------|
| Analysis | `adapters/chronosentiment/src/decision_support/selection_decision_value.rs` |
| Binary | `src/bin/csp006_selection_review.rs` |
| Runner | `run_csp006_selection_review.sh` |
| Sidecar | `product_validation/CS-P-006/discovery/20260814T195327Z/selection_review/` |
| Tests | `adapters/chronosentiment/tests/csp006c2s_selection_review_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
