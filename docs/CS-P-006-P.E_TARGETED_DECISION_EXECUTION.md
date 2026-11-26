# CS-P-006-P.E — Targeted Decision Execution

**Document type:** Product capability protocol  
**Status:** P.E.1 frozen; P.E.2 spec closed / live AWAITING_NEXT_SESSION; P.E.2.H historical lifecycle validation — PASS; P.E.3 specified not started; P.E.3.A artifact contract only  
**Successor:** CS-P-006-P.E.3 — Coralys Target Discovery; CS-P-006-P.E.3.A artifact contract; do not replace the +5% control  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P, CS-P-006-P.H.2, CS-P-006-P.H.3  
**Does not:** run Search #3, start C.3-G, add indicators, retune C3-002, path-optimize the target, mutate the 14 August seals, reinterpret Replay v0/v1, authorize stops, authorize real capital  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; the target is sealed at T; future OHLC never chooses the target.

---

## What this is

P.7 already names the Observatory product screens. This document is **not** P.7.

This is the next product layer after the Decision Evidence Engine:

> **Decision + Execution Intent, then Observation, then Evidence**

The Coralys target (when it exists) is an execution hypothesis. It is not evidence.

C3-002 still answers only LONG / SHORT / NO_TRADE. It is not rewritten. It does **not** carry a 5% target.

**Execution Contract v0** adds the mechanics: `target_pct = 5.0%`, maximum hold 20 market sessions, exit = TARGET or HORIZON.

```text
Certified state at T
        │
        ▼
     C3-002
        │
        ├── Direction
        │
        ▼
  Execution Contract v0
        │
        ├── target_pct = 5.0%
        ├── Max hold = 20 market sessions
        └── Exit = target OR horizon
        │
        ▼
   Future OHLC
        │
        ▼
 First exit event
        │
        ├── TARGET
        └── HORIZON
```

Three measurements stay separate: **Decision** (direction) · **Execution Intent** (how to act) · **Observation** (what happened) · **Evidence** (the measured record). The Coralys target is not evidence. See CS-P-006-P.E.1 and CS-P-006-P.E.3.A.

---

## What does not change

| Component | Action |
|---|---|
| C3-002 decision logic | No change |
| TMV state | No change |
| Search #2 | No change |
| Replay v0 / v1 observations | Not reinterpreted |
| 14 August prospective seals | No change — sealed without an execution intent; remains untouched |
| C.3-G / Search #3 | Not started |

---

## Sealed at T

The target must be known from information available at T. Looking at the subsequent path and choosing a target that would have worked is prohibited.

```text
TARGET_PATH_OPTIMIZATION_AUTHORIZED = false
STOP_EXIT_AUTHORIZED = false
```

First execution contract (`targeted_execution_v0_fixed_5pct_20_sessions`):

| Decision component | This freeze |
|---|---|
| Direction | C3-002 |
| Target | +5.0% of entry (LONG) / −5.0% of entry (SHORT) |
| Stop | None |
| Maximum holding period | 20 market sessions after T |
| Exit condition | Target hit OR horizon expires |
| Target timestamp | Derived at T |
| Target source | Deterministic policy parameter of this contract |
| Outcome | Appended later |

+5.0% is **`target_pct = 5.0%` on Execution Contract v0**. C3-002 does not have a 5% target. The value is deliberately fixed. Do not make it adaptive in this freeze.

Entry price is the adjusted close of the decision session. Session 0 is that session. Monitoring starts on the next eligible market session.

---

## How "target hit" is defined

Daily OHLC. Intraday high/low count.

```text
Entry price (T close)
     ↓
Target price (sealed at T)
     ↓
Future OHLC bars after T
     ↓
Did High reach target?   (LONG)
Did Low reach target?    (SHORT)
     ↓
YES → TARGET exit
```

Fill rule (deterministic):

* LONG: if the session **open** is already at or through the target, fill at open; otherwise fill at the target price.
* SHORT: if the session **open** is already at or through the target, fill at open; otherwise fill at the target price.

A session whose high reaches +5.4% while the close is +3% is a **target hit**.

Adjusted OHLC is used (high/low scaled by `adj_close / close`) so the entry and the excursion share one price basis.

Same-bar target-and-stop is **AMBIGUOUS**. Stops are not authorized in this freeze, so AMBIGUOUS cannot occur on the v0 replay.

NO_TRADE produces no target and exits as `NO_TRADE`.

---

## Observation model

Replay v1 remains:

```text
Decision → 20 sessions → close-to-close V(action)
```

P.E is a different contract:

```text
Decision → Entry → Target / horizon → FIRST EXIT EVENT → V(action) on the fill
```

Do not relabel v1 close-to-close values as target-exit results.

Sidecar: `product_validation/CS-P-006/observatory/targeted_execution_v0/`

---

## Recorded fields

```text
decision_id
entry_price
direction
target_price
target_pct
target_hit
target_hit_session
exit_price
exit_reason
holding_sessions
decision_value
```

The sealed decision still contains no future return. Execution evidence is append-only.

P.E.1 sidecar `targeted_execution_v0/` is frozen in this field set. P.E.2 adds trigger audit (`trigger_type`, `trigger_session`, `trigger_timestamp`, `trigger_price`, `execution_price`) on a new live ledger. Do not backfill P.E.1.

This replay is a backtesting mechanism. It is not yet a statistical strategy backtest. Replay integrity is not strategy validation.

---

## What stays frozen

* C.3-G
* Search #3
* C3-002 retune
* Universe expansion
* Stop exits
* Path-optimized targets
* Homepage performance aggregates
* Real capital
* The 14 August prospective seven
* Replay v0 and v1 files
