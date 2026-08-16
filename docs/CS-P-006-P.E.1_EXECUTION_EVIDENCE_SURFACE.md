# CS-P-006-P.E.1 — Execution Evidence Surface

**Document type:** Product capability protocol  
**Status:** Frozen — three measurement layers; Execution Contract v0 owns `target_pct`  
**Successor:** CS-P-006-P.E.2 — frozen prospective lifecycle; CS-P-006-P.E.3 — Coralys Target Discovery (not started)  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P.E, CS-P-006-P.H.3  
**Does not:** make the target adaptive, retune C3-002, run Search #3, start C.3-G, authorize stops, mutate the 14 August seals, reinterpret Replay v0/v1, put 8/14 TARGET on a homepage  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; `target_pct` is an Execution Contract v0 parameter, not a C3-002 field.

---

## Product stack

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

C3-002 remains a **direction policy**. It does not have a 5% target.

```text
Coralys              → intelligence
ChronoSentiment      → decision integrity
Execution Contract   → predefined action mechanics
Observatory          → subsequent evidence
```

---

## Three measurements (do not mix)

| Layer | Question |
|---|---|
| **Decision** | Was LONG / SHORT / NO_TRADE selected from the certified state? |
| **Execution** | Did the predefined target get reached before the maximum holding period? |
| **Evidence** | What was the realized value after that exit? |

Customer language for a completed tick:

```text
Decision: LONG
Target: +5.0%
Maximum hold: 20 sessions
Exit: TARGET
Holding period: 2 sessions
Realized decision value: +5.00%
```

and, equally visible:

```text
Decision: LONG
Target: +5.0%
Maximum hold: 20 sessions
Exit: HORIZON
Holding period: 20 sessions
Realized decision value: −3.20%
```

The second is not hidden. **Both are evidence.**

---

## Terminology

| Say | Do not say |
|---|---|
| Execution Contract v0 | “C3-002’s 5% target” |
| `target_pct = 5.0%` | “C3-002 target” |
| Direction from C3-002 | “C3-002 executed at +5%” |

Machine id stays `targeted_execution_v0_fixed_5pct_20_sessions`. The product label is **Execution Contract v0**.

---

## 5% stays boring

`target_pct = 5.0%` is a sealed parameter of Execution Contract v0. It is not per-instrument and not learned from later prices.

```text
TARGET_PATH_OPTIMIZATION_AUTHORIZED = false
```

Do not ask Coralys in this freeze whether IDEA needs 8% and HDFCBANK needs 4%. That would be a **separate policy artifact** and a separate experiment.

---

## What P.E already demonstrated

Mechanical validation, not strategy validation:

* 14 sealed execution intents
* 14 exits
* 8 TARGET / 6 HORIZON
* target determined before future prices
* OHLC target detection
* gap-through fill defined
* Replay v0/v1 untouched
* 14 August seven untouched

INFY on 15 May is the teaching case: Replay v1 closed at session 20; Execution Contract v0 exited TARGET at session 2 from the same sealed direction.

Sidecar `product_validation/CS-P-006/observatory/targeted_execution_v0/` is **frozen**. Do not rewrite it. Trigger-audit fields (`trigger_type`, `trigger_session`, `trigger_timestamp`, `trigger_price`, `execution_price`) belong to P.E.2 onward; they are not backfilled here.

This replay is a backtesting mechanism. It is not yet a statistical strategy backtest.

---

## What stays frozen

* Adaptive / state-dependent targets
* C.3-G
* Search #3
* C3-002 retune
* Stop exits
* Homepage performance aggregates
* The 14 August prospective seven
* Replay v0 and v1 files
* This P.E.1 sidecar
