# CS-P-007 — Statistical Strategy Validation

**Document type:** Research protocol  
**Status:** Specified — not run; confirmatory backtest of frozen C3-002 + Execution Contract v0  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P.E, CS-P-006-C.3-F, CS-P-006-B.1, CS-P-006-M.1  
**Successor:** CS-P-007-G evidence gate (not opened); P.E.3 waits  
**Does not:** run in this freeze, retune C3-002, change the +5% contract, change the 20-session horizon, expand or drop the universe, start P.E.3, start Search #3, start C.3-G, authorize real capital, put Sharpe / mean V / win rate on the Observatory homepage  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; strategies are evaluated across a predefined sample; holdout does not retune the policy; no invented methodology after seeing results.

---

## Mission

> Determine whether frozen C3-002, under the frozen 20-market-session execution contract, produces decision value that is statistically and economically distinguishable from predefined null/baseline strategies on an untouched historical sample.

We have proven: **the machine doesn't cheat.**

We have not proven: **the machine has useful predictive/execution information.**

The 91 historical observations, Replay v0/v1, and P.E.2.H’s seven intents establish **lifecycle integrity**, not strategy efficacy. They are not this test.

---

## Order (do not collapse)

```text
Statistical validation          ← this document
        ↓
Coralys execution validation    ← P.E.3; waits
        ↓
Prospective paper validation
        ↓
Operational / risk validation
        ↓
Tiny controlled capital         ← execution/operational experiment, not strategy proof
```

P.E.3 is **not** the next build. Real capital is last.

```text
FREEZE → DEFINE TEST → RUN → OBSERVE → REPORT → DECIDE WHETHER NEXT EXPERIMENT IS AUTHORIZED
```

Not:

```text
Run → see results → change target → change horizon → change universe → change C3-002 → run again
```

---

## What is frozen before any run

| Object | Frozen value |
|---|---|
| Direction policy | C3-002 / `5a43b9df97daa76d85edd7f7ef1c12c3a230ef292f7ecfa98ef9587647392121` |
| Universe | HDFCBANK, ICICIBANK, INFY, RELIANCE, TCS, IDEA, MAHABANK |
| State | Certified TMV at T; bars ≤ T only |
| Session rule | Latest certified session ≤ requested clock |
| Horizon | 20 **market sessions** |
| Execution | Execution Contract v0: `target_pct = 5.0%`, TARGET or HORIZON, no stops |
| Target detection | Adjusted OHLC; LONG high / SHORT low; gap-through at open; first exit |
| Engine | `unfrozen-dev` |

```text
CS007_RUN_AUTHORIZED = false
CS007_RETUNE_C3_002_AUTHORIZED = false
CS007_UNIVERSE_MUTATION_AUTHORIZED = false
CS007_TARGET_SEARCH_AUTHORIZED = false
CS007_HOMEPAGE_PERFORMANCE_AUTHORIZED = false
PE3_RUN_AUTHORIZED = false
REAL_CAPITAL_AUTHORIZED = false
```

This freeze **defines** the test. It does not run it.

---

## Two samples (do not mix)

### A. Already seen — diagnostic only

CS-P-006-B.1 39-timestamp grid (91 × 3 observations) was used to discover and diagnose C3-002. Development, selection, and evaluation on that grid **cannot** be the confirmatory gate. If recomputed under Execution Contract v0 they must be labelled **already diagnosed**.

### B. Confirmatory — the actual test

Untouched relative to Search #2 / C3-002 selection:

```text
eligible T =
    certified sessions where all seven names have a bar
    AND T > 2024-12-31T15:30:00Z          (after B.1 evaluation)
    AND T < 2026-08-14T03:45:00Z          (14-August live cohort excluded as a decision T)
    AND the 20th subsequent market session exists in the certified Yahoo cache
```

Cadence: **every** such T. Do not thin the grid after seeing results. Do not drop IDEA or MAHABANK.

14 August remains a live decision-only cohort and is not a CS-P-007 decision T. P.E.1, Replay v0/v1, and P.E.2.H sidecars are not rewritten.

---

## Per-T procedure

```text
T
│
├── information ≤ T
├── C3-002 → direction
├── Execution Intent v0 (+5%, 20 sessions)
└── seal
      │
      └── future OHLC only after seal
              │
              └── TARGET / HORIZON + trigger audit
```

Same anti-lookahead and poison-test discipline as P.E.2.H. Future path must not enter Decision or Execution Intent.

---

## Frozen baselines (before looking at results)

All baselines use the **same** Ts, universe, 20-session calendar, and (except where noted) Execution Contract v0.

| Id | Rule |
|---|---|
| `c3_002_fixed_5pct` | **Candidate.** C3-002 direction + +5% |
| `always_long_fixed_5pct` | LONG every name at every T + +5% |
| `always_short_fixed_5pct` | SHORT every name at every T + +5% |
| `always_no_trade` | NO_TRADE; V = 0 |
| `sign_flip_fixed_5pct` | Opposite of C3-002 (LONG↔SHORT; NO_TRADE stays) + +5% |
| `always_long_horizon` | LONG, no target, exit at 20th session close |
| `hash_direction_fixed_5pct` | Deterministic LONG/SHORT from `SHA256(instrument \|\| T \|\| "cs-p-007-null-v0")`; bit 0 → LONG else SHORT; +5% |

No wall-clock RNG. The hash null is a frozen function of T.

---

## What the research report must calculate

The Observatory **homepage stays clean**. These statistics belong in the CS-P-007 research report only.

* n decisions, n TARGET, n HORIZON, trigger mix
* decision-value distribution; mean / median V; dispersion
* overlap-robust confidence intervals (overlapping 20-session windows are not independent)
* LONG vs SHORT
* per-instrument heterogeneity; leave-one-name-out as a **pre-registered** sensitivity, not a license to drop names
* temporal stability (e.g. calendar-year or half-year blocks inside the confirmatory window, pre-declared in the run freeze)
* downside: quantiles of V; maximum drawdown under the frozen portfolio simulations below
* comparison of `c3_002_fixed_5pct` against every frozen baseline
* sensitivity: report TARGET vs HORIZON mix; do **not** then change 5% or 20 sessions

Primary estimand: equal-weight mean of per-instrument V at each T, then mean across confirmatory T (`protocol V` in the M.1 sense, including NO_TRADE as 0).

### Frozen portfolio simulations (drawdown)

| Id | Rule |
|---|---|
| `decision_iid` | Every eligible T is a decision. Overlap allowed. Used for the V distribution. |
| `one_position_per_name` | A name may have at most one open intent. If still in the 20-session window, skip that name at later T. Equal 1/7 notionals when opened. Used for path drawdown. |

Costs, slippage, and borrow are **unavailable** in this freeze (M.1: unavailable costs stay out of V). The report must say so. Do not invent a cost model after seeing results.

---

## Evidence gate (not this freeze)

```text
CS-P-007-G_EVIDENCE_GATE_OPENED = false
```

Passing CS-P-007 is **not** automatic from a p-value invented after the run. A later freeze (CS-P-007-G) may authorize the next experiment using **only** the metrics and baselines listed here. It may not add a new metric, drop a name, retune C3-002, or change 5%/20 sessions in order to pass.

P.E.3 remains unauthorized until that gate exists **and** records that the confirmatory sample was run as specified.

Real capital remains unauthorized regardless of CS-P-007-G. After a pass: prospective paper → operational/risk validation → tiny capital as an execution experiment.

---

## Sidecar (when later authorized to run)

`product_validation/CS-P-007/statistical_validation_v0/`

Must state:

```text
Statistical strategy validation: RUN|NOT RUN
C3-002 retuned: false
Universe mutated: false
Observatory homepage performance: not published
```

---

## What stays frozen

* C3-002 / Search #2 / Search #3 / C.3-G
* Execution Contract v0
* 14-August prospective seven
* P.E.1, Replay v0/v1, P.E.2 live, P.E.2.H
* P.E.3 generator
* Homepage Sharpe, CAGR, win rate, mean/median/total V as product claims
