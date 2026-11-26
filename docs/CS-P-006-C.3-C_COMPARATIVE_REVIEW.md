# CS-P-006-C.3-C — Search #2 recommendation and decision-value review

**Document type:** Sealed-artifact comparative review  
**Status:** Complete — both searches frozen; C.3-D decomposes the live rules; Search #3 is not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.3-R  
**Does not:** run Search #3, retune either policy, change TMV, add indicators, drop IDEA or MAHABANK, modify the seven-rule genome, use unique-best as fitness, feed evaluation to Coralys, promote a strategy, reopen G-GATE  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates sealed policies; Coralys does not receive holdout feedback; the question is what was learned, not how to make it perform better.

---

## What this freeze is

Search #2 is frozen. This document compares the two sealed artifacts on the same 273 rows.

```text
Search #1 (immutable)     Search #2 (immutable)
        │                         │
        └──────────┬──────────────┘
                   ▼
        C.3-C comparative review
                   │
                   ✕  no Search #3
```

The question is **what did Coralys learn?** — not how to make Search #2 better.

Sidecar: `product_validation/CS-P-006/discovery/20260815T051900Z_c3/review/`

Search #1: `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`  
Search #2: `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121`

---

## Recorded conclusion

Search #2 provides preliminary evidence that the decision-value formulation and broader candidate selection materially improve policy selection under the fixed TMV information set. Holdout economic value improved from −0.583% to −0.296%, regret improved from 5.618% to 5.332%, and unique-best increased from 18.7% to 51.6%. However, Search #2 eliminated NO_TRADE and remains negative in aggregate evaluation value. Therefore the result is promising but not sufficient for promotion or a change in information representation.

The original Search #1 result was at least partly an artifact of how we formulated and selected the decision problem. TMV sufficiency is **not** established.

Search #2 did not maximize development value. That is expected under a development / selection / evaluation split.

---

## 1. Symbol × slice decision matrix (L / S / NT)

SHORT and NO_TRADE in Search #2 are not concentrated in one name. Every symbol is fully directional on every slice.

| Symbol | Dev #1 | Dev #2 | Sel #1 | Sel #2 | Eval #1 | Eval #2 |
|--------|--------|--------|--------|--------|---------|---------|
| HDFCBANK | 7/0/6 | 10/3/0 | 7/0/6 | 10/3/0 | 4/0/9 | 11/2/0 |
| ICICIBANK | 5/0/8 | 11/2/0 | 7/0/6 | 11/2/0 | 1/0/12 | 10/3/0 |
| INFY | 5/0/8 | 11/2/0 | 7/0/6 | 11/2/0 | 5/0/8 | 12/1/0 |
| RELIANCE | 8/0/5 | 13/0/0 | 5/0/8 | 11/2/0 | 6/0/7 | 11/2/0 |
| TCS | 10/0/3 | 12/1/0 | 3/0/10 | 9/4/0 | 5/0/8 | 10/3/0 |
| IDEA | 9/0/4 | 11/2/0 | 6/0/7 | 13/0/0 | 7/0/6 | 10/3/0 |
| MAHABANK | 5/0/8 | 11/2/0 | 4/0/9 | 11/2/0 | 5/0/8 | 11/2/0 |

---

## 2. Evaluation value by symbol

Aggregate evaluation improvement is **not** uniform.

| Symbol | Search #1 V | Search #2 V | ΔV |
|--------|------------:|------------:|---:|
| HDFCBANK | +1.0282% | +0.5846% | −0.4436% |
| ICICIBANK | +0.5801% | +1.0004% | +0.4203% |
| INFY | +1.0375% | +1.7588% | +0.7213% |
| RELIANCE | −1.1914% | −0.9855% | +0.2059% |
| TCS | +1.1222% | +1.5587% | +0.4365% |
| IDEA | −4.3404% | −4.9173% | −0.5769% |
| MAHABANK | −2.3140% | −1.0749% | +1.2392% |

Five names improve. HDFC remains positive but lower. IDEA is more negative. MAHABANK improves and stays negative. The aggregate −0.296% is not “IDEA was repaired.”

---

## 3. NO_TRADE disappearance

Do not fix it. Measure it.

Search #1 stood aside on **152** rows. All 152 had `Trend = Bullish`. Search #2 converted **109 → LONG** and **43 → SHORT**. Zero remained NO_TRADE.

On those 152 rows: mean Search #2 V = **+0.5385%**; Search #2 unique-best **83/152**; Search #2 better than Search #1 **83/152**.

Evaluation subset (58 former stand-asides): mean Search #2 V = **+0.449%**; 30 positive / 28 negative; mean regret 4.04% → 3.59%.

`Trend Neutral` appears on **0 / 273** certified rows. The AssessmentEngine emits only Bullish or Bearish when both MAs exist.

---

## 4. Same-row pairwise

| Slice | n | Search #2 better | Search #1 better | Tie | Mean ΔV |
|-------|--:|-----------------:|-----------------:|----:|--------:|
| all | 273 | 83 | 69 | 121 | +0.300% |
| evaluation | 91 | 30 | 28 | 33 | +0.286% |

The 121 ties are the shared Bearish → LONG decisions. Search #2 is not uniformly better row-by-row. Evaluation is 30–28–33.

---

## 5. The seven rules (not modified)

First-match order, fired on the 273 rows:

| # | Predicate | Action | Fired | Status |
|---|-----------|--------|------:|--------|
| 0 | Trend Bearish | LONG | 121 | live — identical to Search #1’s traded set |
| 1 | Momentum Positive ∧ Trend Bullish ∧ Volatility present | LONG | 109 | live |
| 2 | Trend Neutral | NO_TRADE | 0 | unreachable under this TMV encoding |
| 3 | Trend Bullish | SHORT | 43 | live — residual Bullish |
| 4 | Momentum Positive ∧ Vol present ∧ Vol absent | NO_TRADE | 0 | contradictory |
| 5 | Trend Bullish ∧ Vol present | LONG | 0 | shadowed by #1 and #3 |
| 6 | Volatility present | LONG | 0 | shadowed |
| unmatched | — | LONG | 0 | never reached |

Effective discovered mapping:

```text
Bearish                         → LONG
Bullish ∧ Positive Momentum     → LONG
Bullish otherwise               → SHORT
```

NO_TRADE is present in the genome (rules 2 and 4) and absent from behaviour because Neutral never occurs and rule 4 cannot match. That is a representation / first-match fact, not a handwritten “always trade” rule.

The live surface is simple. Four of seven rules do not fire. The policy is not a 7-way lookup table in use.

---

## What this does not authorize

* Search #3
* Parameter retune
* New indicators or a volatility-state encoding
* Dropping IDEA or MAHABANK
* Editing the seven rules
* Promotion or CS-P-003 interpretation of Search #2

Engine version remains **`unfrozen-dev`**. No real capital.
