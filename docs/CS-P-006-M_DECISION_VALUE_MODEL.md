# CS-P-006-M — Decision Value Model

**Document type:** Research milestone / protocol freeze  
**Status:** Milestone recorded — problem stated; CS-P-006-M.1 is the formal spec; Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-V, CS-P-006-C.2-D, CS-P-006-C.2-S  
**Does not:** run Search #2, change Search #1, turn C.2-D `advantage_vs_*` into Coralys fitness, invent a ±X% borderline band, invent product-UI decision-value estimates, amend `csp006a.policy_artifact.1`, feed evaluation to Coralys, reopen G-GATE, freeze Decision Engine v1.0, add indicators  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy deterministically; Coralys discovers mappings; outcomes do not re-enter the same search.

---

## Why this document exists

This is a **turning point**, not a search authorization.

> We now have a formally defined decision-value problem, but we have not yet defined the final optimization objective Coralys should learn.

C.2-O through C.2-D are complete. They show that Search #1 explored TMV and the full action space, selected a simple policy over a richer near-best, and that **both** elites failed evaluation. The demonstrated problem is not “missing indicators.”

C.2-D then measured, on the existing 273 recommendations, how each recommended action compared with LONG / SHORT / NO_TRADE, **preserving magnitude**. That measurement is observational.

This document freezes what a **decision-value model** means before anyone turns those observations into a new evolutionary objective. CS-P-006-M.1 is the first formal specification. It does not authorize Search #2.

```text
C.2-D observes the landscape
        │
        ▼
CS-P-006-M defines the decision problem
        │
        ▼
CS-P-006-M.1 specification (12 clauses)
        │
        ▼
CS-P-006-N harness (not implemented)
        │
        ✕  not authorized
     Search #2
```

CS-P-006-V remains the vision that prediction ≠ economic decision and that cost/risk/financing families are not certified yet. This document does not replace V. It specifies the comparison model V implied.

---

## Principle

Coralys should eventually learn policies from historical decision outcomes. It should learn a **decision problem**, not a hand-written prediction rule.

```text
Given the information available at T,
is taking this action economically worthwhile
relative to the alternatives?
```

Not:

```text
Did the predicted direction match?
```

C.2-D’s `advantage_vs_*` and `recommended_is_unique_best` fields are **ChronoSentiment measurements of a sealed artifact**. They are not Search #1 fitness and they are not authorized as Search #2 fitness.

Do **not** encode `unique_best` as `{1, 0}` fitness. That would restore the binary classification problem C.2-D was written to leave. Do **not** jump to “minimize regret” because evaluation mean regret is 5.62%. Regret has a reference-action problem: regret relative to what information and what action set? Historical regret optimized carelessly can produce a retrospective policy that does not generalize. Evaluation unique-best of **18.7%** is evidence that many states may not contain enough separation to distinguish the three actions. That is not necessarily an optimizer problem.

---

## Discovery chain (frozen)

| ID | Result | Status |
|----|--------|--------|
| C.2-O | Same artifact with observer on or off | PASS |
| C.2-P | TMV and LONG/SHORT/NO_TRADE explored; population diverse | SEARCH-SPACE EXPLORED |
| C.2-R | 121 LONG / 0 SHORT / 152 NO_TRADE; evaluation 17/33 = 51.5% | GENERALIZATION FAILED |
| C.2-S | Simple policy won selection; richer TMV elite lost selection; both fail evaluation | COMPLETE |
| C.2-D | Continuous landscape of recommended value, regret, and advantage | COMPLETE |

Search #1 stays immutable (`9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`). CS-P-003 stays independent.

---

## Ten questions

Each answer is either **FROZEN** (usable as protocol language now) or **OPEN** (must be frozen in a later methodology document before any new search). OPEN items are not filled from C.2-D quantiles.

### 1. What does “value of an action” mean?

**FROZEN as an observational identity on the certified 20-day path; OPEN as an economic objective.**

On the C.2-D measurement path, with costs not certified:

```text
value(LONG)     =  R
value(SHORT)    = −R
value(NO_TRADE) =  0
```

`R` is the realized raw 20-day close-to-close return already used by Search #1 / C.2-R.

`NO_TRADE = 0` means standing aside: the policy did not take the market. It is **not** a traded 0% inside Search #1 fitness (those rows were excluded from the traded mean).

This identity is a **measurement**. It is not yet “economic value” in the CS-P-006-V sense. Financing, transaction cost, drawdown, and holding-period cost are not subtracted because they are not certified. Adding them requires a new certified state family and a new schema version, not a silent change to this identity.

### 2. How is LONG compared with SHORT?

**FROZEN observationally; not a fitness rule.**

LONG and SHORT are opposite signed exposures to the same `R`:

```text
value(LONG) + value(SHORT) = 0
advantage(LONG, SHORT) = 2R
```

If `R > 0`, LONG is uniquely best among the three actions. If `R < 0`, SHORT is uniquely best. The comparison retains magnitude: `+0.3%` versus `+8%` are different advantages even though both prefer LONG.

A later search must not collapse this to “direction matched.”

### 3. How is each compared with NO_TRADE?

**FROZEN observationally; NO_TRADE is never “correct.”**

```text
advantage(LONG, NO_TRADE)  =  R
advantage(SHORT, NO_TRADE) = −R
advantage(NO_TRADE, LONG)  = −R
advantage(NO_TRADE, SHORT) =  R
```

Standing aside always forgoes `|R|` and always avoids `|R|`. Those two quantities are the same number. That is why “Was NO_TRADE correct?” is not a well-posed question.

NO_TRADE is uniquely best only when `R = 0`. Otherwise it is the middle action: better than the losing direction, worse than the winning direction.

### 4. How are outcome magnitudes retained?

**FROZEN: continuously. No post-hoc classifier.**

Every row keeps `R`, `value(*)`, `recommended_value`, `regret`, and `advantage_vs_*` as floating values.

C.2-R’s correct/incorrect overlay remains a descriptive share of sign. It is not the decision-value statistic. C.2-D does not bin rows into “strongly favourable / borderline / severely adverse” with a cutoff fitted to these returns.

### 5. How are downside and upside represented?

**FROZEN as signed magnitude and regret; OPEN as risk.**

On this path:

* upside of an acted recommendation is positive `recommended_value`
* downside is negative `recommended_value`
* `regret = best_value − recommended_value` is the missed alternative, always ≥ 0
* a left tail (for example evaluation LONG min −30.82%) is visible in the distribution, not as a binary miss

Drawdown, path, duration, and liquidation exposure are **not** represented. CS-P-006-V already said they belong to future certified families. This model does not invent a drawdown term from Search #1 returns.

### 6. How should overlapping horizons be treated?

**OPEN.**

Search #1 scores each month-end `T` independently on a 20-calendar-day observation path. Adjacent month-ends overlap. That is a known property of the first-discovery design, not a hidden defect.

A later methodology may treat:

* each `T` as an independent decision (current measurement), or
* a portfolio of overlapping holdings, or
* a non-overlapping subset of timestamps

None of those is chosen here. Do not “correct” C.2-D means for overlap after seeing the numbers.

### 7. How should multiple instruments be aggregated?

**FROZEN as a distinction; OPEN as the next objective.**

Search #1 fitness is the **mean of seven per-instrument means** of signed traded 20-day returns, with an untraded name contributing 0.

C.2-D reports **row-level** recommended value (NO_TRADE = 0) and slice means of those rows. That is a different statistic from the protocol mean. Both may be shown. Neither is authorized as a new search objective.

A later freeze must say whether decision value is:

* mean of per-instrument means (Search #1 style),
* equal-weighted across decision points,
* or another certified aggregator

It must not be chosen by peeking at which aggregator would have rescued the holdout.

### 8. How should borderline advantages be represented?

**FROZEN: as continuous advantage, not as a class.**

```text
LONG = +0.3%,  NO_TRADE = 0, SHORT = −0.3%
LONG = +8.0%,  NO_TRADE = 0, SHORT = −8.0%
```

Both have the same unique-best action. They do not have the same decision value. The representation is the magnitude of `advantage_vs_*`, not a label `borderline`.

Coralys may later **discover** a decision boundary. We do not hand-code `return > X% → trade`. `X` is not taken from C.2-D quantiles.

### 9. What constitutes economically meaningful separation?

**OPEN. Must not be fitted to this landscape.**

Meaningful separation is not “unique best” and is not “directionally correct.” It would have to say how large an advantage must be, after certified costs and risk, before acting is worthwhile.

Costs are not certified. Therefore no `±X%` economic-significance band is frozen. If one is ever frozen, it comes from a separately declared cost/risk methodology or from Coralys discovering the boundary — not from inspection of Search #1 evaluation tails.

### 10. Which information is available to Coralys during discovery versus only to ChronoSentiment during evaluation?

**FROZEN.**

| Information | Coralys evolution (development) | Coralys selection | ChronoSentiment evaluation | C.2-D landscape |
|-------------|-------------------------------:|------------------:|---------------------------:|----------------:|
| State at T (certified TMV) | yes | yes | yes | yes |
| Realized 20D outcome on development | yes (fitness) | no | diagnosis only | diagnosis |
| Realized 20D outcome on selection | no | yes (seal one candidate) | diagnosis only | diagnosis |
| Realized 20D outcome on evaluation | **no** | **no** | **yes, diagnostic** | **yes, diagnostic** |
| `advantage_vs_*` / regret | **no** | **no** | measurement only | measurement only |
| Evaluation result as search feedback | **no** | **no** | **no** | **no** |

```text
Coralys learns candidate policies
        │
        ▼
PolicyArtifact
        │
        ▼
ChronoSentiment
        │
   state at T + policy decision + realized outcome
        │
        ▼
Decision value  (C.2-D measurement)
        │
        ▼
Research evidence
        │
        ✕  no arrow back into the same search
```

Historically accumulated evidence may design the **next research question**. It must not retune Search #1 or silently become Search #2 fitness.

---

## What is explicitly not authorized

* C.3 / Search #2
* Using `advantage_vs_no_trade`, `regret`, or `recommended_is_unique_best` as a Coralys fitness function
* Adding Momentum encodings, volatility encodings, or new indicators because Search #1 “needed more inputs”
* Inventing a borderline cutoff from C.2-D / C.2-R returns
* Product-UI numbers such as “LONG +X.XX / confidence” — those are a later schema question
* Promoting or inverting the sealed Bearish → LONG policy
* Feeding evaluation into Coralys

---

## Formal specification

The twelve operational clauses — action values, continuous outcomes, horizon, overlap, aggregation, NO_TRADE, opportunity cost, unavailable costs, discovery fitness, selection, holdout, and learning boundary — are frozen in **CS-P-006-M.1**.

This document remains the problem statement. M.1 is the protocol. CS-P-006-N is the measurement harness (not implemented). Search #2 is still not justified: M.1 fitness is specified for a later search and is **not** `−regret`.

Still open after M.1: certified-cost separation (question 9), V-family inputs, a different horizon or portfolio-overlap model, and selection-pool width.

---

## Product implication (not a schema)

ChronoSentiment should eventually present a recommendation as a comparison of actions, not as a prediction of tomorrow’s sign. The mock:

```text
RECOMMENDATION  LONG
Decision-value landscape
  LONG       …
  NO_TRADE   0
  SHORT      …
Relative advantage
  LONG vs NO_TRADE   …
  LONG vs SHORT      …
Confidence  UNAVAILABLE
```

is the **shape** of the product. The numerical estimates are **not invented here**. Confidence remains UNAVAILABLE until a certified confidence methodology exists.

---

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
