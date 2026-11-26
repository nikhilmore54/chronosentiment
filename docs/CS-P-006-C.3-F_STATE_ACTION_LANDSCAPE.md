# CS-P-006-C.3-F — certified TMV state × action landscape

**Document type:** Sealed-universe state × action diagnostic  
**Status:** Frozen — Search #2 is a candidate research artifact; C.3-G states the next question; Search #3 is not authorized; no product claim  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.3-E  
**Does not:** run Search #3, retune Search #2, add indicators, drop IDEA or MAHABANK, rewrite the seven rules, promote a strategy, freeze Decision Engine v1.0, introduce a pass/fail threshold, authorize a product claim, copy a marketing opportunity card into the protocol  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates certified state at T; Coralys does not receive holdout feedback; decision value is measured against alternatives available at the same T.

---

## What this freeze is

The question is not whether Search #2 can be promoted.

The question is:

> For every certified TMV state that actually occurs, what was the subsequent value of LONG, SHORT, and NO_TRADE — irrespective of which action Search #2 chose?

That is the research form of decision usefulness: **was the subsequent value meaningfully better than the alternatives available at the same T?**

Search #1 remains the control (`9a887827…`). Search #2 remains a **candidate research artifact — promising but not validated for promotion**. It is not Strategy v2, not a production policy, and not Decision Engine v1.0.

```text
C.3-E discovered-rule persistence
        │
        ▼
C.3-F state × action landscape     ← this document
        │
        ✕  no Search #3
        ✕  no product claim
        ✕  no handwritten rule
```

Sidecar: `product_validation/CS-P-006/discovery/20260815T051900Z_c3/state_landscape/`

Search #2: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`

```text
value(LONG)     =  R
value(SHORT)    = −R
value(NO_TRADE) =  0
```

SHORT is the sign flip of LONG. NO_TRADE is standing aside. No threshold decides whether a state is useful.

---

## Destination versus this freeze

A product screen that shows identified opportunities and later returns is the **destination**, not this protocol.

```text
Historical discovery
        ↓
Selection
        ↓
Untouched holdout
        ↓
Persistence analysis
        ↓
Prospective paper observation
        ↓
Only then → product claim
```

This document measures the certified landscape. It does not produce an opportunity card, a benchmark-beating basket, or a hope-for-outcomes claim. Confidence, if it appears later, must come from certified evidence — not an invented probability.

---

## Occupancy

The certified AssessmentEngine encoding produced **four** occupied TMV cells on 273 rows.

| Encoding | Count |
|---|---:|
| Trend Neutral | 0 |
| Momentum Neutral | 0 |
| Volatility absent | 0 |
| Observed states | 4 |

There is no Neutral Trend, Neutral Momentum, or unavailable Volatility in this snapshot. The live cube is Trend ∈ {Bullish, Bearish} × Momentum ∈ {Positive, Negative} × Volatility present.

---

## The complete decision-value surface

| Certified state | n | Eval n | LONG V | SHORT V | NO_TRADE | Eval LONG | Eval SHORT | Search #1 | Search #2 |
|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| Bullish ∧ Positive | 109 | 42 | +0.558% | −0.558% | 0 | **+0.190%** | −0.190% | NO_TRADE | LONG |
| Bearish ∧ Negative | 84 | 24 | +0.049% | −0.049% | 0 | **−2.642%** | **+2.642%** | LONG | LONG |
| Bullish ∧ Negative | 43 | 16 | −0.489% | +0.489% | 0 | −1.127% | **+1.127%** | NO_TRADE | SHORT |
| Bearish ∧ Positive | 37 | 9 | +1.970% | −1.970% | 0 | **+1.155%** | −1.155% | LONG | LONG |

Search columns are what each sealed policy chose. They are not the object of measurement.

Evaluation up/down (sign of R): Bullish+Positive 19/23; Bearish+Negative 12/12; Bullish+Negative 5/11; Bearish+Positive 5/4.

---

## What the states contain

### Bullish ∧ Positive Momentum — borderline, not empty

Evaluation LONG +0.19%, median LONG **−0.36%**. Search #1 stood aside. Search #2 took LONG. The state is not a clean winner. Under a hard accuracy cutoff it would likely be discarded. Continuous V keeps it as a thin positive mean. That is why M.1 does not classify states as good or bad.

Slices (LONG V): development −0.56% (30); selection +1.88% (37); evaluation +0.19% (42).

### Bearish ∧ Negative Momentum — regime reversal, not a failed discovery

Search-visible evidence supported LONG:

```text
Development:  LONG +1.44%   SHORT −1.44%
Selection:    LONG +0.71%   SHORT −0.71%
Evaluation:   LONG −2.64%   SHORT +2.64%
```

Coralys choosing LONG was **not irrational**. It could not legitimately know the evaluation reversal from frozen development/selection. The relationship **changed after the selection boundary**. Do not hand-write `Bearish ∧ Negative → SHORT`. That would turn diagnosis into hindsight engineering.

### Bullish ∧ Negative Momentum — strongest directional persistence

Search-visible evidence already pointed toward SHORT on selection, and evaluation continued in that direction:

```text
Development:  LONG +0.31%   SHORT −0.31%
Selection:    LONG −0.45%   SHORT +0.45%
Evaluation:   LONG −1.13%   SHORT +1.13%
```

Search #1 stood aside. Search #2 took SHORT. Development → selection → evaluation support the same directional relationship; magnitude changes. Evaluation n = 16 remains small. This is research evidence, not validation.

### Bearish ∧ Positive Momentum — higher LONG mean, smallest cell

Evaluation LONG +1.15% (n = 9). Both searches chose LONG. Sample is small. IDEA evaluation LONG on this cell is −10.65% (1 row); the other named evaluation cells that exist are positive. Do not extract a rule from nine holdout rows.

---

## Evaluation LONG by instrument (universe retained)

IDEA and MAHABANK stay in the universe. No name is dropped because it is inconvenient.

| State | HDFC | ICICI | INFY | RELIANCE | TCS | IDEA | MAHABANK |
|---|---:|---:|---:|---:|---:|---:|---:|
| Bullish+Pos | −1.79% (7) | +0.21% (9) | +1.17% (7) | +0.09% (5) | +0.31% (5) | −2.93% (3) | +2.87% (6) |
| Bearish+Neg | +2.98% (3) | — (0) | +0.32% (3) | −2.58% (6) | +2.38% (3) | −7.63% (6) | −6.40% (3) |
| Bullish+Neg | −3.37% (2) | −1.19% (3) | −1.22% (1) | −1.11% (2) | −1.37% (3) | −0.43% (3) | +0.55% (2) |
| Bearish+Pos | +4.44% (1) | +7.54% (1) | +6.26% (2) | — (0) | +3.72% (2) | −10.65% (1) | −5.45% (2) |

On Bullish+Negative, evaluation LONG is negative for six of seven names (MAHABANK +0.55% on 2 rows). That is the same fact as evaluation SHORT being mostly positive — reported here as the LONG counterfactual, not as a product SHORT call.

---

## Recorded conclusion (frozen)

No state is marked pass or fail. No product claim is authorized. Numbers are not reopened.

| State | Development / selection | Evaluation | Reading |
|---|---|---|---|
| Bullish ∧ Positive | −0.56% / +1.88% LONG | +0.19% LONG | Weak; survives directionally; borderline |
| Bearish ∧ Negative | +1.44% / +0.71% LONG | −2.64% LONG | **Regime reversal** after the selection boundary |
| Bullish ∧ Negative | +0.31% LONG / −0.45% LONG | +1.13% SHORT | **Strongest persistence**; n_eval = 16 |
| Bearish ∧ Positive | +1.31% / +3.29% LONG | +1.15% LONG | Most stable LONG region; n_eval = 9 |

> **The TMV representation contains actionable conditional structure, but some relationships are regime-unstable. Search #2 successfully discovered at least one relationship that persisted into holdout, while another historically valid relationship reversed.**

That is stronger than “Search #2 improved the score.”

Discovery capability and regime persistence are now separate questions. Coralys can discover a state → action-value mapping from search-visible data. That mapping is **not permanently stationary**.

Bullish ∧ Positive remains borderline (eval mean +0.19%, median −0.36%, 19/23). Continuous V still prefers LONG to SHORT and to NO_TRADE. A hard `if expected return > X` cutoff would discard it. M.1 does not.

Search #3 is not authorized. The next research target, if any, is stated in CS-P-006-C.3-G. It is not “search harder.”

---

## What this does not authorize

* Search #3
* A handwritten Trend × Momentum product rule
* A product opportunity card, basket, or benchmark-beating claim
* Promotion of Search #2
* Dropping IDEA or MAHABANK
* New indicators

Engine version remains **`unfrozen-dev`**. No real capital.
