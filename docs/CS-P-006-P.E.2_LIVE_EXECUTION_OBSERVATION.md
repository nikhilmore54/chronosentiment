# CS-P-006-P.E.2 — Live Execution Observation

**Document type:** Product capability protocol  
**Status:** Frozen — prospective lifecycle with fixed Execution Contract v0; not a test of whether 5% is a good target  
**Successor:** CS-P-006-P.E.3 — Coralys Target Discovery (specified; not started; P.E.2 is the control)  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P.E, CS-P-006-P.E.1  
**Does not:** mutate the 14 August seals, rewrite P.E.1 / Replay v0 / Replay v1, retune C3-002, make `target_pct` adaptive, run Search #3, start C.3-G, authorize stops, fetch into the certified snapshot cache, seal every subsequent session in this freeze, ask whether 5% is a good target  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; a **separate** execution intent is also sealed at T; future OHLC never chooses the target.

---

## One sentence

> ChronoSentiment preserves the temporal integrity of a decision: Coralys interprets the certified state, the decision and execution intent are sealed at T, and the Observatory records the evidence that emerges afterward.

---

## What this freeze tests

Not: is 5% a good target.

> Can the complete temporal lifecycle work when an execution contract is attached **at decision time**?

P.E.2 should remain boring. It proves the clock, sealing, execution, trigger classification, and append-only evidence. Coralys target research is P.E.3.

```text
C.3-F     Does certified TMV state contain useful structure?
C3-002    Direction
P.E.1     Can a sealed direction be evaluated against a predefined execution contract?
P.E.2     Can that contract operate prospectively from the live clock?   ← this freeze
P.E.3     Can Coralys derive the execution parameters from state at T?   (not started)
```

---

## Decision ≠ Execution Intent

Both are frozen at T. They answer different questions. The target is not part of C3-002.

```text
Decision
    ├── direction
    ├── certified state
    ├── policy artifact
    └── timestamp

Execution Intent
    ├── target
    ├── maximum hold
    ├── trigger semantics
    └── execution-contract artifact
```

For this freeze the execution intent is Execution Contract v0 (`target_pct = 5.0%`, 20 sessions, TARGET/HORIZON, trigger audit). That pairing is correct **for P.E.2**. It is not the eventual Coralys-generated target.

```text
             STATE AT T
                  │
                  ▼
              CORALYS
          direction + eventually
          execution parameters
                  │
                  ▼
          CHRONOSENTIMENT
             SEAL AT T
          (Decision + Execution Intent)
                  │
                  ▼
        EXECUTION CONTRACT
                  │
          ┌───────┴────────┐
          ▼                ▼
       TARGET           HORIZON
                  │
                  ▼
             OBSERVATORY
              EVIDENCE
```

---

## Product question

Not: can we find a better strategy.

> Can ChronoSentiment generate, seal, execute, and subsequently evidence decisions under Execution Contract v0 without hindsight?

```text
Historical replay
      ↓
P.E.1 mechanical validation   (frozen)
      ↓
P.E.2 prospective execution   (this document; frozen)
      ↓
real-time TARGET / HORIZON observation
      ↓
append-only evidence
```

C3-002 is not touched. P.E.1 asked whether a sealed direction can be executed under a predefined contract. P.E.2 asks whether that same fixed contract can operate on the live clock. P.E.3 (not started) will ask whether Coralys can derive the execution parameters. P.E.2 is the **control** for that later comparison.

Two live cohorts stay distinct:

```text
14-Aug cohort
Decision only
7 OBSERVING
No execution intent
```

```text
Next eligible cohort
Decision + Execution Intent
P.E.2 control
```

The 14-August cohort was sealed without an execution intent and remains untouched. P.E.2 will attach Execution Contract v0 only to the next eligible cohort at T. The 5% contract is not applied retrospectively to 14 August.

---

## What stays protected

| Object | Rule |
|---|---|
| 14 August prospective seven | Decision only. Sealed without an execution intent. **No retrospective contract.** |
| `observatory/prospective/` | Not written |
| `targeted_execution_v0/` | P.E.1 frozen. Not rewritten |
| Replay v0 / v1 | Not rewritten |
| C3-002 / Search #2 | Immutable |

The latest bar in the certified Yahoo cache is **14 Aug 2026, 03:45 UTC**. 15 Aug 2026 is a Saturday (and an NSE holiday). There is therefore **no eligible live session yet**.

```text
FOURTEEN_AUG_COHORT_MUTATION_AUTHORIZED = false
PE1_SIDECAR_MUTATION_AUTHORIZED = false
CONTINUOUS_SESSION_SEAL_AUTHORIZED = false
LIVE_YAHOO_FETCH_AUTHORIZED = false
```

Until a certified session **strictly after** 14 Aug 03:45 UTC exists, P.E.2 status is:

```text
AWAITING_NEXT_SESSION
```

That is a valid product state. It is not an empty implementation.

---

## What P.E.2 will seal (when the next session exists)

Same seven names, including IDEA.NS and MAHABANK.NS. Same C3-002. Same Execution Contract v0.

```text
Certified state at T'  (T' > 14 Aug 2026 03:45 UTC)
        │
        ▼
     C3-002          → Direction
        ▼
  Execution Contract v0
        ├── target_pct = 5.0%
        ├── Max hold = 20 market sessions
        └── Exit = TARGET or HORIZON
        ▼
   Future OHLC
        ▼
 First exit event + trigger audit
        ▼
 Append-only evidence
```

This freeze seals **one** next cohort (the first eligible T'). Daily new-T generation remains unauthorized here (`CONTINUOUS_SESSION_SEAL_AUTHORIZED = false`). Re-runs only observe that cohort.

---

## Trigger audit (why the target was considered hit)

P.E.1 recorded `exit_reason` and `exit_price`. P.E.2 also records **why**.

TARGET, high/low reached inside the session (open had not gapped through):

```text
exit_reason       = TARGET
target_pct        = 5.0
target_price      = …
trigger_session   = 6
trigger_timestamp = …
trigger_price     = session high (LONG) / session low (SHORT)
trigger_type      = HIGH_REACHED | LOW_REACHED
execution_price   = target_price
```

TARGET, gap-through:

```text
exit_reason       = TARGET
trigger_type      = GAP_THROUGH
trigger_price     = session open
execution_price   = SESSION_OPEN
```

HORIZON, target never printed:

```text
exit_reason       = HORIZON
holding_sessions  = 20
exit_price        = session_20_close
trigger_type      = SESSION_CLOSE
execution_price   = session_20_close
```

OBSERVING: no trigger fields. No realized value. Intermediate “we are up 3%” interpretation remains unauthorized.

IDEA and MAHABANK use exactly this contract. They are not special-cased.

---

## Sidecar

`product_validation/CS-P-006/observatory/prospective_execution_v0/`

Path kind: `prospective_execution_v0`. Product label: **Execution Contract v0**. Machine id unchanged: `targeted_execution_v0_fixed_5pct_20_sessions`.

This replay/observation is a backtesting-adjacent mechanism once bars exist. It is not yet a statistical strategy backtest.

---

## What stays frozen

* Adaptive / per-name targets
* C.3-G
* Search #3
* C3-002 retune
* Stop exits
* Homepage performance aggregates
* The 14 August prospective seven
* P.E.1 `targeted_execution_v0/` files
* Replay v0 and v1 files
* Continuous per-session reseal
* Asking whether 5% is a good target (that is P.E.3)
