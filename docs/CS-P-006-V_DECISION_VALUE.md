# CS-P-006-V — Decision value and future certified state

**Document type:** Research-architecture vision  
**Status:** Informative — does not amend CS-P-006-A/B/B.1 genomes or freeze dates  
**Date:** 2026-08-14  
**Parent:** CS-P-006  
**Prompt:** JM Financial Services Margin Trading Facility guide (domain knowledge, not a strategy source)  
**Does not:** add VaR/ATR/leverage/interest/holding-period rules to CS-P-006-C; turn ChronoSentiment into an MTF calculator; hand-write NO_TRADE thresholds  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment remains a deterministic evaluator of certified state at T; Coralys discovers mappings; outcomes do not construct the decision.

---

## Why this document exists

A financing facility is a reminder that a trading decision is broader than “will the price go up?”

That is **domain knowledge**:

> Leverage, daily interest, margin, and liquidation risk can make a directionally correct view economically wrong.

It is **not** a strategy rule:

> Do not trade when ATR > X, or when MTF interest exceeds Y.

CS-P-006-C’s first search remains the certified state we already have:

```text
Trend + Momentum + Volatility  →  LONG | SHORT | NO_TRADE
```

Additional families below are **candidates for later certified information at T**, after a schema amendment and a fidelity snapshot. They are not CS-P-006-B.1 inputs.

---

## Prediction is not a decision

```text
Expected price return
        +
Risk
        +
Financing cost
        +
Transaction cost
        +
Expected holding period
        +
Liquidity
        +
Potential margin stress
        ↓
Economic decision  (including NO_TRADE)
```

ChronoSentiment should not ultimately answer only “Will the stock rise?” It should evaluate a sealed policy over **what was knowable at T**. Coralys should learn that ecology from history. We do not hard-code it from a broker brochure.

---

## Four families (future certification, not this genome)

| Family | Examples of knowable-at-T information | CS-P-006-C now |
|--------|----------------------------------------|----------------|
| Market | Trend, Momentum, Volatility, price, historical returns | **Certified** (TMV) |
| Position economics | purchase value, contributed margin, funded amount, leverage, holding period | Not certified |
| Risk | drawdown, liquidation/margin-call exposure, liquidity / impact | Volatility presence only |
| Cost | financing, brokerage, pledge/unpledge, square-off | Not certified |

NO_TRADE is standing aside because expected decision value does not clear risk/cost/opportunity — **Coralys may discover that boundary**. It is not “I don’t know.”

Horizon (5D / 10D / 20D / 60D) may later become state-dependent. First discovery does not invent per-state horizons from this document.

---

## How the engine extends without a rewrite

`csp006a.policy_artifact.1` already evaluates an ordered rule list over `input_schema`. New certified concepts require:

1. A **new schema version** (not silently growing `.1`)
2. Factors reconstructed at T from information ≤ T
3. A disposable research snapshot and fidelity certification
4. Coralys search on that schema — still producing `PolicyArtifact`
5. ChronoSentiment still calling `decide_from_inputs` with an explicit policy

ChronoSentiment does not grow an `MtfCalculator`. Coralys does not receive a handwritten leverage map.

Forbidden as “discovery” in any generation: copying VaR multipliers, interest schedules, or “MTF is for short-term trades” into genome predicates.

---

## Immediate programme

```text
006-B.1 partition          ✓ frozen
        │
        ▼
006-C first search         Trend / Momentum / Volatility
                           LONG / SHORT / NO_TRADE
        │
        ▼
later generation           additional certified families
                           (risk, cost, capital, financing)
```

Engine version remains **`unfrozen-dev`**. No real capital.
