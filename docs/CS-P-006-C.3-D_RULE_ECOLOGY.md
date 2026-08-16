# CS-P-006-C.3-D — Search #2 live-rule ecology

**Document type:** Sealed-artifact rule diagnostic  
**Status:** Complete — Search #2 is a candidate research artifact; C.3-E measures persistence; Search #3 is not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.3-C  
**Does not:** run Search #3, retune Search #2, add indicators, drop IDEA or MAHABANK, rewrite the seven rules, promote a strategy, freeze Decision Engine v1.0, use unique-best as fitness  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy; Coralys does not receive holdout feedback; measure the learned surface before changing the information set.

---

## What this freeze is

Search #1 remains the control (`9a887827…`). Search #2 remains a **candidate research artifact — promising but not validated for promotion**. It is not Strategy v2, not a production policy, and not Decision Engine v1.0.

This document decomposes the three live first-match rules. It does not make the surface bigger.

```text
Search #2 sealed
        │
        ▼
C.3-D live-rule ecology     ← this document
        │
        ✕  no Search #3
        ✕  no new indicators
```

Sidecar: `product_validation/CS-P-006/discovery/20260815T051900Z_c3/rule_ecology/`

Search #2: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`

---

## What `otherwise` actually means

Do not change the rule. On the certified 273 rows, `Bullish otherwise → SHORT` fired on **exactly one** state:

```text
Bullish ∧ Negative Momentum ∧ Volatility present     n = 43
```

There is no Neutral Momentum and no unavailable Momentum in this fired set. Under the certified TMV encoding, the live surface is:

```text
Bearish                         → LONG     (121)
Bullish ∧ Positive Momentum     → LONG     (109)
Bullish ∧ Negative Momentum     → SHORT    ( 43)
```

Momentum is a discriminator **inside Bullish**. That is a measured hypothesis, not a handwritten product rule.

---

## Live-rule headline

| Live state | Action | n | Mean V | Eval V | Unique-best |
|---|---|---:|---:|---:|---:|
| Bearish | LONG | 121 | +0.636% | **−1.606%** | 69 (57.0%) |
| Bullish ∧ Positive Momentum | LONG | 109 | +0.558% | **+0.190%** | 57 (52.3%) |
| Bullish ∧ Negative Momentum | SHORT | 43 | +0.489% | **+1.127%** | 26 (60.5%) |

Share of total signed value (`n × mean V`): Bearish LONG **48.5%**, Bullish+Positive LONG **38.3%**, Bullish+Negative SHORT **13.3%**. No single new rule created the entire aggregate. The **evaluation** sign split is sharper: the inherited Bearish → LONG block is still negative on holdout; the two Bullish rules are non-negative.

NO_TRADE = 0 is not treated as a failure. The question is whether the directional decisions have persistent value.

---

## 1. Bearish → LONG (inherited Search #1 surface)

| Slice | n | Mean V |
|---|---:|---:|
| development | 49 | +1.404% |
| selection | 39 | +1.569% |
| evaluation | 33 | **−1.606%** |

V: median +0.96%, P25 −3.03%, P75 +4.39%. Regret mean 4.72%. Alternatives: LONG +0.64% / SHORT −0.64% / NO_TRADE 0.

Momentum inside this rule is mixed (84 Negative, 37 Positive). The rule does **not** use Momentum. Volatility is present on all 121.

Evaluation by instrument (this rule only): HDFC / ICICI / INFY / TCS positive; RELIANCE −2.58%; IDEA **−8.06%** (7 rows); MAHABANK −6.02%. This is the Search #1 holdout failure, still present.

Years: 2021 +5.01% (n=9); 2022 +0.24%; 2023 +2.13%; 2024 = evaluation −1.61%.

---

## 2. Bullish ∧ Positive Momentum → LONG (new)

| Slice | n | Mean V |
|---|---:|---:|
| development | 30 | −0.556% |
| selection | 37 | +1.878% |
| evaluation | 42 | +0.190% |

One fired state only: Bullish / Positive / present. Unique-best 57/109. Median V +0.32%; evaluation median **−0.32%** with mean +0.19% — the holdout centre is not cleanly positive.

Evaluation by instrument: MAHABANK +2.87% (6); INFY +1.17%; TCS +0.31%; ICICI +0.21%; RELIANCE +0.09%; HDFC **−1.79%** (7); IDEA **−2.93%** (3). Heterogeneous. Development of this rule is negative; selection is strongly positive.

Years: 2021 −4.37% (n=8); 2022 +0.43%; 2023 +2.13%; 2024 +0.18%.

---

## 3. Bullish ∧ Negative Momentum → SHORT (new; the former “otherwise”)

| Slice | n | Mean V |
|---|---:|---:|
| development | 12 | −0.312% |
| selection | 15 | +0.451% |
| evaluation | 16 | **+1.127%** |

n = 43 is small. Unique-best 26/43. Alternatives confirm the action: mean LONG on these rows is **−0.49%**.

Evaluation by instrument (small cells): six of seven names non-negative; MAHABANK −0.55% (2 rows). IDEA evaluation on this rule is +0.43% (3 rows) — the IDEA damage is not coming from SHORT.

Years: 2021 −1.63% (n=4); 2022 ~0; 2023 +0.72%; 2024 +1.13%.

---

## Did one rule generate the aggregate improvement?

No.

- The inherited Bearish → LONG block still dominates count and still fails evaluation.
- The two Bullish rules are the former NO_TRADE region. Together they are the part of the surface that is non-negative on holdout.
- Bullish+Negative SHORT has the best evaluation mean and the smallest sample.
- Bullish+Positive LONG has the largest new sample and only a thin evaluation mean.

That is consistent with C.3-C: richer surface, heterogeneous instruments, not a solved policy.

---

## What this does not authorize

* Search #3
* Hand-writing `Bullish ∧ Negative Momentum → SHORT` into ChronoSentiment
* New indicators
* Promotion of Search #2

Engine version remains **`unfrozen-dev`**. No real capital.
