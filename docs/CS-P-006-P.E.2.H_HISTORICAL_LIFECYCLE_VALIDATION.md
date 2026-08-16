# CS-P-006-P.E.2.H — Historical P.E.2 Lifecycle Validation

**Document type:** Product validation evidence protocol  
**Status:** **PASS** — historical lifecycle validation completed; live P.E.2 untouched  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P.E.2  
**Does not:** modify the P.E.2 specification, attach Execution Contract v0 to 14 August, rewrite P.E.1 / Replay v0 / Replay v1 / live `prospective_execution_v0`, retune C3-002, run Search #3, start C.3-G, start P.E.3, present a statistical strategy backtest  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed artifacts → same Decision and same Execution Intent; future OHLC never chooses the target.

---

## What this is

P.E.2’s specification is **CLOSED**. Live demonstration remains `AWAITING_NEXT_SESSION` with **0 seals**. This document does not reopen that freeze.

This run executes the already-defined control against a **historical clock** so the lifecycle can be demonstrated without waiting for Monday and without contaminating the live experiment.

```text
P.E.2 specification:          CLOSED
P.E.2 live demonstration:     AWAITING_NEXT_SESSION
P.E.2 live seals:             0
P.E.2 historical time-machine: this document
```

```text
TIME MACHINE
        ↓
15 Jul 2026 03:45 UTC
        ↓
Certified state using information ≤ T
        ↓
C3-002 → Decision
        +
Execution Intent v0
        ├── target = 5.0%
        ├── max hold = 20 MARKET SESSIONS
        └── first-exit semantics
        ↓
SEAL
        ↓
Replay the next 20 market sessions
        ↓
TARGET / HORIZON + trigger audit
        ↓
Append-only evidence
```

---

## What this is not

Not a rename of P.E.1. P.E.1’s 14 May/June intents stay frozen.

```text
P.E.1
14 historical intents
        ↓
FROZEN

P.E.2 historical time-machine
NEW T = 15 Jul 2026
        ↓
NEW decisions
        ↓
NEW execution intents
        ↓
NEW evidence sidecar
        historical_pe2_replay/
```

The 14-August live cohort remains permanently decision-only. Live P.E.2 remains `AWAITING_NEXT_SESSION`.

---

## Clock

Requested T: **15 Jul 2026, 03:45 UTC**. Certified session: the latest session ≤ that timestamp (here, the session itself). Seven-name universe, including IDEA.NS and MAHABANK.NS. Twenty subsequent market sessions are present in the certified Yahoo cache.

---

## Anti-lookahead

At T:

* only information available at or before T may enter Decision or Execution Intent
* future OHLC is stripped from the decide path
* the 5% target is the frozen Execution Contract v0 parameter
* future returns must not influence direction or target
* poison of post-T prices must leave Decision and Execution Intent identical

Evidence uses the real subsequent OHLC **after** the seal. Poisoned prices are never used for TARGET / HORIZON calculation.

---

## Sidecar

`product_validation/CS-P-006/observatory/historical_pe2_replay/`

```text
Historical P.E.2 lifecycle validation: PASS
Statistical strategy backtest: NOT PERFORMED
```

Mean / median / total V, Sharpe, CAGR, win rate, alpha, and profitability conclusions are not product claims.

---

## Protected objects

| Object | Rule |
|---|---|
| 14 August prospective seven | Untouched. Decision only. |
| `observatory/prospective_execution_v0/` | Live P.E.2. Still `AWAITING_NEXT_SESSION`. Not written. |
| `targeted_execution_v0/` | P.E.1 frozen. Not rewritten. |
| Replay v0 / v1 | Not rewritten. |
| C3-002 / Search #2 | Immutable. |

> Old evidence cannot silently become new evidence.

---

## What stays frozen

* The P.E.2 specification
* Asking whether 5% is a good target (that is P.E.3)
* P.E.3 / Search #3 / C.3-G
* Adaptive / per-name targets
* Homepage performance aggregates
