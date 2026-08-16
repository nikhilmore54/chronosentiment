# CS-P-006-M.1 — Decision Value Specification

**Document type:** Formal research protocol  
**Status:** Frozen — Search #2 / C.3 not authorized  
**Date:** 2026-08-15  
**Parent:** CS-P-006-M, CS-P-006-V, CS-P-006-C.2-D  
**Does not:** run Search #2, change Search #1, make regret or `unique_best` the fitness, invent a ±X% band, add handwritten confluence, feed evaluation to Coralys, reopen G-GATE, freeze Decision Engine v1.0, implement CS-P-006-N  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy deterministically; Coralys discovers mappings from historical outcomes; the same search does not receive holdout results.

---

## What this freeze is

CS-P-006-M defined the decision-value **problem**. This document is the first **specification** of that problem: what one decision is, how action values are computed, what Coralys may learn, and what ChronoSentiment reports.

```text
We stop asking:  was the recommendation correct?
We start asking: given what was knowable at T,
                 which action created the greatest subsequent value,
                 and under what states does that relationship persist?
```

Search #1 stays immutable (`9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`). This specification does **not** retune it. It is the protocol a later search would have to obey. CS-P-006-N may implement measurement against this spec. C.3 / Search #2 is still not authorized.

---

## Principle

```text
Coralys learns   State(T) → subsequent decision value
ChronoSentiment  evaluates a sealed PolicyArtifact
```

ChronoSentiment is not made “smarter” by adding hand-written rules. Coralys is not told `Bullish + Positive → LONG` or `R > 1% → good`. Boundaries, if they exist, must **emerge** from evolutionary search on development and be sealed on selection.

The future price path after T is used **only** to measure outcomes. It is not part of the state Coralys may condition on.

---

## Twelve frozen clauses

### 1. Action-value equations

On a realized raw 20-day close-to-close return `R`:

```text
V(LONG)     =  R
V(SHORT)    = −R
V(NO_TRADE) =  0
```

There is no “correct” threshold. `+0.1%` remains `+0.1%`. `−0.1%` remains `−0.1%`.

Hindsight best and historical regret (diagnostic, not fitness):

```text
V*     = max(R, −R, 0) = |R|
regret = V* − V(policy(S))
```

A wrong LONG or wrong SHORT has regret `2|R|`. Standing aside on the same move has regret `|R|`.

### 2. Continuous outcome treatment

Every decision keeps the floating values `R`, `V(*)`, `V(policy)`, `regret`, and `advantage_vs_*`.

Forbidden:

```text
R > +1% → good
R < −1% → bad
otherwise → ignore

unique_best = true  → fitness 1
unique_best = false → fitness 0

advantage > 1%
regret > 5% = bad
```

Borderline magnitudes remain evidence. A candidate may survive because many small gains and occasional larger gains outweigh losses. Coralys may also discover that a state is too unstable and that NO_TRADE is better there. That is learning, not a hand-written band.

### 3. Horizon semantics

The discovery observation horizon is **20 calendar days**, the same path already used by Search #1 / C.2-R / C.2-D.

This is an explicit first-specification choice. It is **not** claimed to be the product-complete co-pilot horizon. Changing it requires a new spec version. Do not retune 20D from evaluation tails.

### 4. Overlapping-window treatment

One decision is `(instrument, T)` — see clause A below. Adjacent month-end 20-day windows overlap in calendar time. That overlap is **permitted**.

```text
2021-10-31 15:30 UTC  →  +20D
2021-11-30 15:30 UTC  →  +20D
```

Each `T` is an independent decision opportunity. Windows are not merged, not HAC-corrected, and not down-weighted after seeing C.2-D numbers. A later spec may treat overlapping holdings as a portfolio. That is not this freeze.

### 5. Instrument aggregation

The certified universe remains the seven names in CS-P-006-B.1.

The **protocol scalar** (discovery fitness and selection score) is the **mean of seven per-instrument means** of per-decision `V(policy)`. An instrument with no rows in a slice is undefined and is a protocol error, not a silent 0.

Equal instrument weight is an explicit decision: it prevents one name’s return scale from dominating a pooled 273-row mean.

C.2-D row-level means and quantile tables remain **diagnostic**. They are not the discovery scalar.

### 6. NO_TRADE semantics

NO_TRADE is a first-class action: standing aside. `V = 0`.

It is never scored “correct.” It is uniquely best only when `R = 0`. Otherwise it is the middle action: better than the losing direction, worse than the winning direction.

In this specification, a NO_TRADE row **enters** the instrument mean as 0. That is different from Search #1 fitness, which excluded stand-asides from the traded mean and assigned 0 only when a name was never traded. Search #1 is not rewritten.

### 7. Opportunity-cost semantics

For a NO_TRADE decision, opportunity cost and loss avoided are the same number `|R|`.

```text
forgone if the winning direction was taken   = |R|
avoided if the losing direction was taken    = |R|
```

“Was NO_TRADE correct?” is not a well-posed question. Report `|R|` as opportunity cost / loss avoided, not as accuracy.

### 8. Costs are absent because they are unavailable

The equations in clause 1 have **no cost term** because transaction costs, financing, liquidity, and margin are **not certified** (CS-P-006-V). This is not an assumption that markets are frictionless.

```text
cost_term_present = false
```

Adding a cost term requires a certified state family and a new spec version. Do not invent costs from Search #1 returns.

### 9. Discovery fitness

Coralys may use realized outcomes **on the development slice only**.

```text
V(policy, instrument, T) = V(action chosen from State(T))
fitness                  = mean_i ( mean_T V(policy, i, T) )
```

`T` ranges over development timestamps. `i` ranges over the seven instruments.

This scalar is **expected decision value of the chosen action**, including NO_TRADE as 0. It is **not**:

* directional accuracy
* `unique_best` as `{0,1}`
* `−regret`
* C.2-D’s evaluation mean regret of 5.62%

Search #1’s traded-only mean remains the fitness of the sealed experiment. This clause is the fitness a later search under this spec would use. It is not applied to Search #1 and is not Search #2.

### 10. Selection criterion

After evolution, **one** candidate is sealed using the **same scalar** as clause 9, computed on the **selection** slice only.

CS-P-006-B requires selecting one candidate. It does **not** require generation-best-only (C.2-S). Pool width is a later harness choice (CS-P-006-N / a future C.3). It is not frozen here as “two elites” and not widened by peeking at the C.2-O archive.

Evaluation is not available to selection.

### 11. Holdout criterion

ChronoSentiment applies the sealed artifact to the evaluation slice and reports, at least:

* mean `V(policy)` (protocol scalar and row-level diagnostic)
* regret distribution (mean, median, tails) — **diagnostic**
* unique-best share — **diagnostic**, not an accuracy KPI
* opportunity cost on NO_TRADE rows
* per-instrument breakdowns

Nobody learns. The result does not select, retune, invert, or feed Coralys. “Acted better than NO_TRADE” must not be presented as accuracy when the policy can emit only LONG.

A relationship is **not** promoted from holdout numbers. Promotion remains a later programme decision. CS-P-003 remains last-mile confirmation of a historically defensible candidate, not a substitute for this holdout.

### 12. What Coralys is and is not allowed to learn

**Allowed**

* Certified state at T: Trend, Momentum, Volatility (presence / direction as already reconstructed ≤ T)
* Development realized `V(*)` for evolution
* Selection realized `V(*)` only to seal one candidate
* Emitting LONG, SHORT, or NO_TRADE
* Ignoring any subset of factors
* Discovering that the available information is insufficient (stand aside)

**Forbidden**

* Evaluation outcomes, evaluation regret, evaluation unique-best
* Future prices, intra-horizon path, or `R` as a state feature
* Hand-written confluence (`Bullish + Positive → LONG`)
* Bands or cutoffs fitted to C.2-R / C.2-D returns
* New indicators, volatility encodings, or V-family cost fields
* Using `unique_best` or `−regret` as the clause-9 fitness
* Mutating Search #1 or `csp006a.policy_artifact.1` silently

```text
              HISTORY
                 │
                 ▼
       ┌──────────────────┐
       │  Coralys learns  │
       │  development V   │
       └────────┬─────────┘
                │
         selection V seals one
                │
                ▼
          PolicyArtifact
                │
                ▼
       ChronoSentiment
                │
       ┌────────┴────────┐
       ▼                 ▼
  evaluation         prospective
  (diagnostic)       (CS-P-003 last)
       │                 │
       └────────┬────────┘
                ▼
          decision value
                │
                ✕  no arrow into the same search
```

---

## Atomic unit

**A. One decision** is `(instrument, T)` on the certified month-end grid.

State at T is the certified TMV reconstruction. The 20-day path after T is outcome measurement only.

---

## Relation to Search #1

| | Search #1 (immutable) | This specification (future search only) |
|--|--|--|
| Action values | signed traded return; NO_TRADE omitted from traded mean | `V ∈ {R, −R, 0}`; NO_TRADE enters as 0 |
| Fitness | mean of per-instrument traded means | mean of per-instrument means of `V` |
| Regret / unique-best | not in the objective | diagnostic only |
| Horizon | 20 calendar days | 20 calendar days (explicit) |
| Overlap | independent `T` (implicit) | independent `T` (explicit) |
| Aggregation | mean of 7 instrument means | mean of 7 instrument means (explicit) |
| Evaluation | quarantined | quarantined |

Do not re-score Search #1 with clause 9 and call that a new winner.

---

## What remains open (not this freeze)

* Economically meaningful separation after certified costs (CS-P-006-M question 9)
* CS-P-006-V families (financing, margin, liquidity, drawdown)
* A different horizon or a portfolio-overlap model
* Whether a later search presents a wider candidate pool than generation-best
* CS-P-006-N implementation
* C.3 / Search #2

---

## Programme sequence after this freeze

```text
Phase 1  Historical discovery     Coralys on development V
Phase 2  Historical holdout       ChronoSentiment on evaluation
Phase 3  Robustness / landscape   value, regret, opportunity cost, instruments
Phase 4  Freeze candidate         only if evidence supports it
Phase 5  Forward / paper          CS-P-003 last
```

CS-P-003 asks whether a historically validated relationship **continues in the present**. It is not “wait 60 days to see if something works.”

Next authorized engineering step: **CS-P-006-N** (measurement harness, no evolution). Not Search #2.

---

Engine version remains **`unfrozen-dev`**. No real capital. No Search #2.
