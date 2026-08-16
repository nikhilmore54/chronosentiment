# CS-P-006-C.3-G — Regime-persistence question

**Document type:** Research-target authorization  
**Status:** Question stated — experiment not authorized; research loop stopped at C.3-F; Search #3 is not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-C.3-F  
**Does not:** run Search #3, retune Search #2, implement a regime detector, add MTF / ATR / VaR / financing / liquidity, hand-write a persistence rule, drop IDEA or MAHABANK, promote a strategy, freeze Decision Engine v1.0, feed evaluation to Coralys, authorize a product claim  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates certified state at T; Coralys does not receive holdout feedback; no invented methodology.

---

## What this freeze is

C.3-F is frozen. This document **states the next research question** and does **not** start an experiment. Product work, if any, is CS-P-006-P and does not answer C.3-G.

```text
C.3-F state × action landscape     (frozen)
        │
        ▼
C.3-G regime-persistence question  ← this document
        │
        ✕  no experiment
        ✕  no Search #3
        ✕  no handwritten detector
        ✕  no new indicators
```

Search #1 remains the control (`9a887827…`). Search #2 remains a **candidate research artifact — promising but not validated for promotion**.

The research question has changed.

Not:

> Can Coralys discover a better policy?

Instead:

> **Is the discovered state → action-value relationship stable enough to be useful?**

---

## What C.3-F established (not reopened)

1. TMV is not empty. Four occupied cells each have a complete LONG / SHORT / NO_TRADE surface.
2. Search #1 stood aside on both Bullish cells. Search #2 acted on them.
3. **Bullish ∧ Negative → SHORT** has development → selection → evaluation directional persistence. Evaluation n = 16. Research evidence, not validation.
4. **Bearish ∧ Negative → LONG** was rational on search-visible data (+1.44% / +0.71%) and reversed on evaluation (−2.64%). Coralys could not have known that from development/selection.
5. **Bullish ∧ Positive → LONG** is borderline and still has a positive expected-value difference versus SHORT and NO_TRADE.
6. **Bearish ∧ Positive → LONG** is the most stable LONG region and has nine evaluation rows.
7. State-dependent decision value is **not** a permanently stationary mapping.

C.3-F numbers are not reopened here.

---

## Scientific question

Using **only information available at T**:

> **Can we detect when a historically discovered state → action relationship is likely to persist or fail?**

That is the missing bridge between:

```text
State(T) → Action
```

and:

```text
State(T)
    ↓
Expected decision value
    ↓
Action
    ↓
Observed outcome
```

It is **not** answered by asking Coralys to search harder on the same TMV cube.

---

## Explicit non-decisions

The CS-P-006 research loop **stops here as a question**. Observing a reversal does not require a regime detector. C.3-G must not become Search #3 in disguise.

This document does **not** authorize an experiment that would answer the question.

It does **not** authorize:

| Path | Status |
|---|---|
| Search #3 | **Not authorized** |
| Handwritten `Bearish ∧ Negative → SHORT` | **Forbidden** |
| Handwritten regime detector (ATR, vol-of-vol, calendar, “risk-off”) | **Forbidden** |
| New certified families (MTF, VaR, financing, liquidity, RSI) | **Not this target** |
| Promotion of Search #2 | **Not authorized** |
| Product opportunity card | **Not authorized** |

If a later experiment is authorized, it must remain information-at-T only, keep evaluation quarantined, keep IDEA and MAHABANK, and treat any persistence score as diagnosis — not as Coralys fitness — until a later methodology freeze says otherwise.

---

## Why this target, not Search #3

Search #3 on the same TMV information would mix two different failures:

- relationships Coralys had not yet discovered
- relationships Coralys discovered that later reversed

C.3-F already separated those. Searching harder would blur them again.

The four-cell landscape is the experimental clue. It is enough to state the question. It is not enough to invent a detector.

---

## Labels (unchanged)

| Artifact | Role |
|----------|------|
| `9a887827…` | Search #1 **control** |
| `5a43b9df…` | Search #2 **candidate research artifact** |
| C.3-F | **Frozen diagnostic** |

Engine version remains **`unfrozen-dev`**. No real capital.
