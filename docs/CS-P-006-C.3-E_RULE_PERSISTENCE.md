# CS-P-006-C.3-E — Search #2 discovered-rule persistence

**Document type:** Sealed-artifact persistence diagnostic  
**Status:** Complete — Search #2 is a candidate research artifact; C.3-F measures the state landscape; Search #3 is not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.3-D  
**Does not:** run Search #3, retune Search #2, add indicators, drop IDEA or MAHABANK, rewrite the seven rules, promote a strategy, freeze Decision Engine v1.0, introduce a pass/fail threshold, use unique-best as fitness  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy; Coralys does not receive holdout feedback; persistence is measured, not gated.

---

## What this freeze is

Search #1 remains the control (`9a887827…`). Search #2 remains a **candidate research artifact — promising but not validated for promotion**. It is not Strategy v2, not a production policy, and not Decision Engine v1.0.

C.3-D exposed three live first-match relationships. This document asks whether those relationships persist across time, instruments, and exact certified states. **No threshold decides whether a rule "passes."**

```text
C.3-D live-rule ecology
        │
        ▼
C.3-E discovered-rule persistence     ← this document
        │
        ✕  no Search #3
        ✕  no pass/fail threshold
        ✕  no new indicators
```

Sidecar: `product_validation/CS-P-006/discovery/20260815T051900Z_c3/rule_persistence/`

Search #2: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`

Calendar windows **2021–22 / 2022–23 / 2023–24 overlap** on the shared year. They are persistence views, not a second partition. The frozen slices remain development / selection / evaluation.

Unique-best shares are diagnostics over this fixed sample. They are not probabilities and not confidence.

---

## Sample size and contribution

| Live state | Action | n | Eval n | Mean V | Eval V | Signed sum | Eval signed sum |
|---|---|---:|---:|---:|---:|---:|---:|
| Bearish | LONG | 121 | 33 | +0.636% | **−1.606%** | +0.770 | **−0.530** |
| Bullish ∧ Positive Momentum | LONG | 109 | 42 | +0.558% | +0.190% | +0.608 | +0.080 |
| Bullish ∧ Negative Momentum | SHORT | 43 | 16 | +0.489% | +1.127% | +0.210 | +0.180 |

Share of total signed value (`n × mean V`): 48.5% / 38.3% / 13.3%. Evaluation signed sums are the useful holdout decomposition: the inherited Bearish block is the entire evaluation drag; the two Bullish rules are the entire evaluation offset. Share-of-a-negative-total is not a ranking and is not used here.

NO_TRADE = 0 is not treated as a failure. The question is whether the directional decisions persist.

---

## Temporal persistence

| Rule | 2021–22 | 2022–23 | 2023–24 | Development | Selection | Evaluation |
|---|---:|---:|---:|---:|---:|---:|
| Bearish → LONG | +1.05% (53) | +1.07% (79) | +0.32% (68) | +1.40% (49) | +1.57% (39) | **−1.61% (33)** |
| Bullish+PosMom → LONG | **−0.58% (38)** | +1.35% (66) | +1.17% (71) | −0.56% (30) | +1.88% (37) | +0.19% (42) |
| Bullish+NegMom → SHORT | **−0.46% (14)** | +0.41% (23) | +0.95% (29) | −0.31% (12) | +0.45% (15) | +1.13% (16) |

Bearish → LONG is positive in the first two calendar windows and does not persist into evaluation. The two Bullish rules are negative in 2021–22 / development and non-negative afterward. That is a measured pattern, not a pass.

---

## 1. Bearish → LONG (inherited)

Action advantage equals LONG, as required: all +0.636% / eval −1.606%. SHORT alternative is the sign flip. Unique-best 69/121 (57.0%) is diagnostic.

### Exact certified states

| Trend | Momentum | Volatility | n | Mean V | Eval n | Eval V |
|---|---|---|---:|---:|---:|---:|
| Bearish | Negative | present | 84 | +0.05% | 24 | **−2.64%** |
| Bearish | Positive | present | 37 | +1.97% | 9 | +1.15% |

The inherited block is not one relationship. Momentum is mixed inside Bearish. Coralys did **not** gate this rule on Momentum. Do not rewrite the rule. The holdout failure sits in Bearish ∧ Negative Momentum.

### Instruments (not required to be uniformly positive)

| Instrument | n | Mean V | Eval n | Eval V | +/− |
|---|---:|---:|---:|---:|---:|
| HDFCBANK.NS | 18 | +1.56% | 4 | +3.34% | 11/7 |
| ICICIBANK.NS | 13 | +2.11% | 1 | +7.54% | 8/5 |
| INFY.NS | 17 | +1.65% | 5 | +2.70% | 12/5 |
| RELIANCE.NS | 19 | −0.30% | 6 | −2.58% | 10/9 |
| TCS.NS | 18 | +1.28% | 5 | +2.92% | 12/6 |
| IDEA.NS | 22 | −2.08% | 7 | **−8.06%** | 8/14 |
| MAHABANK.NS | 14 | +1.55% | 5 | −6.02% | 8/6 |

### Failure cluster

52 losing rows. Loss sum −2.856. IDEA is 42.3% of that sum (14 rows). 2024 / evaluation is 48.5%. This is the Search #1 holdout failure, still present.

---

## 2. Bullish ∧ Positive Momentum → LONG (new)

One fired state only: Bullish / Positive / present (109). Unique-best 57/109 (52.3%) is diagnostic.

Action advantage equals LONG: all +0.558% / eval +0.190%. Evaluation median is **−0.36%** with 19 positive / 23 negative — the holdout centre is not cleanly positive.

### Instruments

| Instrument | n | Mean V | Eval n | Eval V | +/− |
|---|---:|---:|---:|---:|---:|
| HDFCBANK.NS | 13 | −0.28% | 7 | −1.79% | 8/5 |
| ICICIBANK.NS | 19 | −0.67% | 9 | +0.21% | 8/11 |
| INFY.NS | 17 | +0.17% | 7 | +1.17% | 9/8 |
| RELIANCE.NS | 16 | +0.19% | 5 | +0.09% | 8/8 |
| TCS.NS | 13 | +0.10% | 5 | +0.31% | 6/7 |
| IDEA.NS | 12 | +0.78% | 3 | −2.93% | 7/5 |
| MAHABANK.NS | 19 | +3.18% | 6 | +2.87% | 11/8 |

### Failure cluster

52 losing rows. Loss sum −2.688. No single instrument dominates: MAHABANK 22.1%, IDEA 22.0%, INFY 15.8%. Years are spread (2022 29.5%, 2023 28.6%, 2024 23.7%, 2021 18.2%). Development holds 39.2% of the loss sum.

---

## 3. Bullish ∧ Negative Momentum → SHORT (new)

One fired state only: Bullish / Negative / present (43). This is what C.3-D's `otherwise` actually meant. Unique-best 26/43 (60.5%) is diagnostic. n = 43 and evaluation n = 16 remain small.

Action advantage confirms the action: LONG alternative is −0.489% overall and **−1.127%** on evaluation. NO_TRADE is 0.

### Instruments

| Instrument | n | Mean V | Eval n | Eval V | +/− |
|---|---:|---:|---:|---:|---:|
| HDFCBANK.NS | 8 | +2.25% | 2 | +3.37% | 5/3 |
| ICICIBANK.NS | 7 | −1.32% | 3 | +1.19% | 2/5 |
| INFY.NS | 5 | +3.53% | 1 | +1.22% | 5/0 |
| RELIANCE.NS | 4 | +2.14% | 2 | +1.11% | 4/0 |
| TCS.NS | 8 | −0.17% | 3 | +1.37% | 4/4 |
| IDEA.NS | 5 | −1.71% | 3 | +0.43% | 3/2 |
| MAHABANK.NS | 6 | −0.66% | 2 | −0.55% | 3/3 |

IDEA evaluation on this rule is +0.43% (3 rows). The IDEA damage is not coming from SHORT.

### Failure cluster

17 losing rows. Loss sum −0.769. MAHABANK 27.0%, TCS 24.2%, IDEA 23.4%, ICICI 21.0%. Evaluation is only 18.2% of the loss sum (5 rows). The largest loss year is 2023, not 2024.

---

## Recorded observation (not a verdict)

No rule is marked pass or fail.

- **Bearish → LONG** does not persist into evaluation. Losses cluster in IDEA and 2024. Inside the block, Bearish ∧ Negative Momentum is the holdout-negative state; Bearish ∧ Positive Momentum is non-negative on the 9 evaluation rows. Coralys did not use that split. Do not hand-write it.
- **Bullish ∧ Positive Momentum → LONG** is later-window positive and early-window negative. Evaluation mean is thin; evaluation median is negative. Instrument persistence is mixed.
- **Bullish ∧ Negative Momentum → SHORT** is the most coherent later-window and evaluation relationship, and it has the smallest sample. The LONG alternative is negative on the same rows. IDEA is not driving it.
- Same TMV information. Search #1's limitation was not simply "TMV has no information." TMV sufficiency is still not established.

That is consistent with C.3-C and C.3-D: richer surface, heterogeneous instruments, not a solved policy.

---

## What this does not authorize

* Search #3
* A pass/fail gate on any live rule
* Hand-writing either Bullish Momentum split, or a Bearish Momentum split, into ChronoSentiment
* New indicators
* Promotion of Search #2

Engine version remains **`unfrozen-dev`**. No real capital.
