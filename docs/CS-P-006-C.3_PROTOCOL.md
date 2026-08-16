# CS-P-006-C.3 — Protocol authorization for a controlled second search

**Document type:** Protocol authorization  
**Status:** Protocol authorized — Search #2 is one complete C.3-R experiment; no iteration  
**Date:** 2026-08-15  
**Parent:** CS-P-006-N, CS-P-006-M.1, CS-P-006-C.2-S, CS-P-006-C.2-O  
**Does not:** run Search #2, retune Search #1, change the TMV information set, add indicators, drop IDEA or MAHABANK, invent ATR/Momentum cutoffs, use `unique_best` or `−regret` as fitness, feed evaluation to Coralys, reopen G-GATE, freeze Decision Engine v1.0  

`.cursor/rules/chronosentiment-core.mdc`: ChronoSentiment evaluates a sealed policy; Coralys discovers mappings; holdout does not re-enter the same search.

---

## What this freeze is

N has passed. Diagnostic layers stop here. This document **opens C.3 as a redesigned search protocol**, not as “Search #1 with different knobs.”

It does **not** start Coralys evolution.

```text
Search #1 (immutable)
        │
   C.2-O … C.2-D, M, M.1, N
        │
        ▼
C.3 protocol design          ← this document
        │
        ▼
C.3-I implementation          (CS-P-006-C.3-I; Search #2 not run)
        │
        ▼
C.3-R one authorized run     (CS-P-006-C.3-R; no iteration)
```

Search #1 remains `9a887827e8f41988987208f13e4ccbac507b3241692026c55f38d11f85971ac0`.

---

## What we have established (not reopened)

1. The search mechanism works. C.2-P: TMV and LONG/SHORT/NO_TRADE were explored; the population stayed diverse; a non-obvious policy emerged reproducibly.
2. The sealed policy is not robust across instruments or regimes. N: HDFC / ICICI / INFY / TCS stay positive on evaluation mean V; RELIANCE turns negative; MAHABANK reverses; IDEA is substantially negative.
3. The decision problem is richer than directional accuracy. Evaluation hit rate 51.5% with acted mean −1.61% and unique-best **18.7%**.
4. The sealed rule is coarse: `Bearish → LONG; otherwise → NO_TRADE`.
5. We do **not** respond with handwritten repairs (`IDEA → don't trade`, `Momentum confirmation → trade`, `ATR > X → don't trade`).

**18.7% unique-best is not “the problem is bad.”** It means that under this 20-day `V` definition, most historical states did not have a clearly dominant action. That is information. It is not a fitness target. Small advantages may still accumulate. M.1 already forbids `unique_best → {0,1}`.

---

## Scientific question

Hold the information set fixed.

> **Did Search #1 fail because of the representation, or because fitness and selection asked Coralys to optimize the wrong thing?**

C.3 answers that only if **nothing else changes** except the decision-value formulation, the candidate pool, and observability.

---

## Controlled contrast

| | Search #1 (control) | C.3 / Search #2 (not started) |
|--|--|--|
| State | TMV | **Same TMV** |
| Universe | 7 certified names | **Same 7** |
| Horizon | 20 calendar days | **Same 20D** |
| Actions | LONG / SHORT / NO_TRADE | **Same** |
| Seed / pop / gens / elite / mutation / crossover / tournament / max rules | 42 / 32 / 12 / 4 / 0.25 / 0.8 / 3 / 16 | **Same** |
| Snapshot / partition | `c21ec256…` / `4354c81e…` | **Same** |
| Fitness | traded-only mean of signed 20D returns; NO_TRADE omitted from the traded mean | **M.1 continuous V**; NO_TRADE enters as 0 |
| Protocol scalar | mean of 7 instrument means | **Same aggregator, different per-decision V** |
| Population observation | generation-best history | **C.2-O full ecology** |
| Selection pool | 2 persisted elites | **unique genomes observed during evolution** |
| Evaluation | quarantined | **quarantined** |
| Symbol matrices | post-hoc (N) | **required during the experiment (N contract)** |

If a later Search #2 improves on development/selection **and** the evaluation diagnosis is not worse in the N sense, that is evidence that formulation/selection was a material limitation. If it does not, the problem is elsewhere: representation, regime instability, or insufficient information at T. Evaluation still does not choose the winner.

---

## Four questions C.3 must answer (after a future run)

### 1. Does M.1 decision-value fitness produce a different ecology?

```text
V(LONG) = R     V(SHORT) = −R     V(NO_TRADE) = 0
protocol_value  = mean of seven per-instrument means of V
```

This is the largest methodological change. Regret, unique-best, and advantage remain diagnostics. They must not construct `ProtocolValue`.

### 2. Does the population still discover simple policies?

Coralys already found `Bearish → LONG`. Continuous `V` may keep that, or discover richer mappings **only when they create value**. Complexity is not a goal.

### 3. Does behaviour become more balanced across symbols?

N is the baseline. A later policy must not merely maximize the aggregate while destroying value on individual names. There is **no** handwritten “every symbol must be positive” constraint. Symbol-level Table A / Table B remain mandatory and observable.

### 4. Does Coralys use NO_TRADE intelligently?

Not “was NO_TRADE correct?” Ask whether NO_TRADE appears where both directional alternatives have poor `V`, while borderline positive opportunities can still be taken. No forced threshold between trade and stand-aside.

---

## What Coralys is asked to discover

Not:

```text
Find the threshold where trading becomes worthwhile.
```

That presupposes a cutoff.

Instead:

```text
Find a mapping from the certified state at T
to continuous expected decision value,
with LONG, SHORT, and NO_TRADE as competing actions.
```

The search may find a clean boundary, a fuzzy region, asymmetric LONG/SHORT behaviour, state-dependent abstention, or no useful separability. All of those are valid outcomes.

---

## Selection pool (frozen for the future run)

CS-P-006-B requires selecting **one** candidate on selection. It does not require generation-best-only (C.2-S).

C.3 requires:

```text
selection_pool = unique genome identities observed during evolution
                 (C.2-O archive of living-population slots),
                 scored on the selection slice with M.1 protocol V
```

Not the two-elite bottleneck. Deduplicate by identity hash. Evaluation is not in the pool and not in the score.

---

## Observability and measurement

A future Search #2 must attach the C.2-O observer (identity-preserving). After seal, ChronoSentiment must emit the N harness: protocol V, Table A, Table B, regret and unique-best as diagnostics. Evaluation remains holdout-only.

---

## Explicitly forbidden as “C.3”

* Starting evolution in this freeze
* Changing Trend / Momentum / Volatility or adding encodings
* Dropping or down-weighting IDEA or MAHABANK after seeing N
* `unique_best` or `−regret` as fitness
* Bands fitted to C.2-D / N quantiles
* Hand-written confluence or ATR cutoffs
* Feeding evaluation into Coralys
* Promoting Search #1 or inverting `Bearish → LONG`
* Calling 18.7% unique-best a failure of the research programme

---

## Authorization

| Item | Status |
|------|--------|
| C.3 protocol | **Authorized** |
| Search #2 evolution | **One complete experiment** — CS-P-006-C.3-R; no iteration |
| C.3 implementation (M.1 fitness in the evaluator, wider pool, observer on) | CS-P-006-C.3-I — implemented; Search #2 not run |
| Promotion / CS-P-003 interpretation | Unchanged; CS-P-003 stays last |

Engine version remains **`unfrozen-dev`**. No real capital.
