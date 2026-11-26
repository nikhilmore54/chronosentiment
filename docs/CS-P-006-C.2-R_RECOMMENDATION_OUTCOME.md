# CS-P-006-C.2-R — Search #1 recommendation outcome

**Document type:** Bounded analysis of an immutable search  
**Status:** Complete — Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.2-P  
**Does not:** run Search #2, change Search #1, change fitness/seed/universe/horizon, choose a volatility encoding, amend `csp006a.policy_artifact.1`, feed evaluation to Coralys, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy deterministically; outcomes do not re-enter discovery.

---

## What this freeze is

Apply the immutable Search #1 `PolicyArtifact` to every certified historical decision point and record recommendation vs realized 20-day outcome.

```text
PolicyArtifact 9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0
        ↓
273 certified states
        ↓
recommendation-vs-outcome matrix
```

This is **not** Search #2. Coralys is not re-run. Evaluation numbers are ChronoSentiment holdout diagnosis.

Sidecar: `product_validation/CS-P-006/discovery/20260814T195327Z/recommendations/`

---

## Semantics

| Recommendation | Correct | Incorrect | Otherwise |
|----------------|---------|-----------|-----------|
| LONG | actual 20D return `> 0` | actual 20D return `< 0` | flat / missing |
| SHORT | actual 20D return `< 0` | actual 20D return `> 0` | flat / missing |
| NO_TRADE | never | never | opportunity vs LONG/SHORT alternatives |

NO_TRADE is standing aside. It is not “correct because the market later fell.”

Two means are reported:

* **Recommendation-level mean** — average of signed 20D returns on the LONG/SHORT rows themselves.
* **Protocol mean** — Search #1 fitness: mean of seven per-instrument means; untraded name = 0. This is the +1.6325% / +1.9938% / −0.0229% number.

They are not the same statistic.

---

## Scorecard

```text
Recommendations generated:       273
LONG recommendations:            121
SHORT recommendations:             0
NO_TRADE recommendations:        152

LONG correct:                     69
LONG incorrect:                   52
SHORT correct:                     0
SHORT incorrect:                   0

Overall LONG directional accuracy: 57.0%
```

| Slice | n | LONG | SHORT | NO_TRADE | LONG correct | LONG incorrect | LONG accuracy | Rec-level LONG mean | Protocol mean |
|-------|---|------|-------|----------|--------------|----------------|---------------|--------------------:|--------------:|
| all | 273 | 121 | 0 | 152 | 69 | 52 | 57.0% | +0.636% | +0.825% |
| development | 91 | 49 | 0 | 42 | 29 | 20 | 59.2% | +1.404% | +1.633% |
| selection | 91 | 39 | 0 | 52 | 23 | 16 | 59.0% | +1.569% | +1.994% |
| evaluation | 91 | 33 | 0 | 58 | 17 | 16 | 51.5% | −1.606% | **−0.023%** |

```text
20D protocol mean
Development     +1.63%
Selection       +1.99%
Evaluation      -0.02%

Generalization: FAIL
```

SHORT is zero by construction of the sealed rule.

---

## The −0.02% evaluation mean is not “every call was wrong”

Evaluation LONG distribution (33 rows):

| | Signed 20D |
|--|--:|
| min | −30.82% |
| p25 | −5.81% |
| median | **+0.71%** |
| p75 | +3.81% |
| max | +17.88% |
| mean | −1.61% |

Directionally the holdout is a coin flip (17 / 16). The mean is pulled by a left tail, not by uniform small losses.

Worst evaluation LONGs: IDEA 2024-08-31 −30.8%, 2024-09-30 −18.0%, 2024-10-31 −14.9%.  
Best: IDEA 2024-12-31 +17.9%, TCS 2024-06-30 +10.1%, INFY 2024-05-31 +8.9%.

All 33 evaluation LONGs fall in 2024. Incorrect evaluation LONGs concentrate on IDEA (5), MAHABANK (4), and RELIANCE (4).

All 121 LONG signed returns:

| Bin | Count |
|-----|------:|
| < −10% | 8 |
| −10% to −5% | 12 |
| −5% to 0 | 32 |
| 0 to +5% | 45 |
| +5% to +10% | 16 |
| ≥ +10% | 8 |

---

## NO_TRADE opportunity

NO_TRADE is not scored as correct.

| Slice | n | Market up after | Market down after | Mean raw 20D | Mean if LONG instead | Mean if SHORT instead |
|-------|---|----------------:|------------------:|-------------:|---------------------:|----------------------:|
| all | 152 | 74 | 78 | +0.26% | +0.26% | −0.26% |
| development | 42 | 22 | 20 | −0.31% | −0.31% | +0.31% |
| selection | 52 | 28 | 24 | +1.21% | +1.21% | −1.21% |
| evaluation | 58 | 24 | 34 | −0.17% | −0.17% | +0.17% |

On selection, standing aside left a subsequent +1.21% average move on the table (Bullish names kept rising). On development and evaluation, the average raw move after NO_TRADE was slightly negative.

---

## By instrument (all slices)

| Instrument | LONG | Correct | Incorrect | NO_TRADE | Mean signed when traded |
|------------|-----:|--------:|----------:|---------:|------------------------:|
| HDFCBANK.NS | 18 | 11 | 7 | 21 | +1.56% |
| ICICIBANK.NS | 13 | 8 | 5 | 26 | +2.11% |
| INFY.NS | 17 | 12 | 5 | 22 | +1.65% |
| RELIANCE.NS | 19 | 10 | 9 | 20 | −0.30% |
| TCS.NS | 18 | 12 | 6 | 21 | +1.28% |
| IDEA.NS | 22 | 8 | 14 | 17 | −2.08% |
| MAHABANK.NS | 14 | 8 | 6 | 25 | +1.55% |

IDEA is the only name that is both net-wrong on direction and net-negative on traded return. That is diagnosis, not a reason to drop the ticker after seeing the holdout.

---

## Example rows

| Date | Instrument | State | Recommendation | Actual 20D | Call |
|------|------------|-------|----------------|-----------:|------|
| 2021-10-31 | HDFCBANK.NS | Bullish | NO_TRADE | −4.26% | — |
| 2021-11-30 | HDFCBANK.NS | Bearish | LONG | −3.46% | incorrect |
| 2021-12-31 | HDFCBANK.NS | Bearish | LONG | +2.85% | correct |
| 2022-01-31 | HDFCBANK.NS | Bullish | NO_TRADE | +2.45% | — |

The full 273-row matrix is `recommendations/recommendations.json`.

---

## What this does not authorize

* C.3 / Search #2
* Retuning from the 17/16 holdout split or the IDEA tail
* Calling NO_TRADE “correct”
* A new policy or volatility encoding
* Feeding this scorecard back to Coralys

The product question — “are these recommendations useful?” — is now answerable at row level. The research answer for this artifact remains: **not on the holdout.**

---

## Code

| Piece | Location |
|-------|----------|
| Analysis | `adapters/chronosentiment/src/decision_support/recommendation_outcome.rs` |
| Binary | `src/bin/csp006_recommendation_outcome.rs` |
| Runner | `run_csp006_recommendation_outcome.sh` |
| Sidecar | `product_validation/CS-P-006/discovery/20260814T195327Z/recommendations/` |
| Tests | `adapters/chronosentiment/tests/csp006c2r_recommendation_outcome_tests.rs` |

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
