# CS-P-006-P.H.1 — Decision Evidence Engine

**Document type:** Product capability protocol  
**Status:** Achieved — Historical Decision Replay is a product surface; 14-tick integrity PASS is not a statistical strategy backtest  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P, CS-P-006-P.H, CS-P-006-M.1  
**Successor:** CS-P-006-P.H.2 — Observatory horizon is now 20 market sessions; this document's 14-tick run is Replay v0 and is not reinterpreted.  
**Successor dashboard:** CS-P-006-P.H.3 — Decision Evidence Dashboard.  
**Does not:** start C.3-G, run Search #3, retune C3-002, mutate the 14 August cohort, put aggregates on a homepage, authorize real capital  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision.

---

## What this freeze is

The 14-tick replay tested the **backtesting/replay mechanism**. It did not complete a statistical strategy backtest.

| Claim | Status |
|---|---|
| Historical replay integrity | **PASS** |
| Statistical C3-002 performance study | **Not done** |

Fourteen observations are too few for an economic claim. Winners and losers stay visible as evidence. Mean / median / total signed V are **not** homepage metrics.

Research stays at C.3-F. Coralys already showed that conditional structure can be discovered. The product job now is to **record, observe, and prove decisions**.

```text
                 CORALYS
             Decision intelligence
                    │
                    ▼
             CHRONOSENTIMENT
             Decision @ T
                    │
             ┌──────┴──────┐
             │             │
        IMMUTABLE       IMMUTABLE
          STATE           ACTION
             └──────┬──────┘
                    ▼
             OBSERVATORY
                    │
             observation window
                    ▼
              FUTURE EVIDENCE
```

Same architecture, both directions:

```text
LIVE                 T ──────────────► FUTURE
HISTORICAL REPLAY    T ──────────────► HISTORICAL FUTURE
```

ChronoSentiment does not need a special backtester. The Observatory runs the same lifecycle on a live clock or a historical clock.

---

## Contract-definition check — 20D

The 14-tick dates are **20 elapsed calendar days**:

```text
15 May 2026 03:45 UTC  →  4 Jun 2026 03:45 UTC
12 Jun 2026 03:45 UTC  →  2 Jul 2026 03:45 UTC
```

That matches the already-frozen discovery contract (CS-P-006-M.1, CS-P-006-C.3, CS-P-006-C.3-I):

```text
horizon:
    duration = 20 days
    calendar_basis = CALENDAR_DAYS
```

Replay v0 must say **20 calendar days**, not an unqualified **20D**, and must not be presented as 20 market-session results.

A trading-session horizon is a **different contract** (CS-P-006-P.H.2, Replay v1):

```text
horizon:
    duration = 20
    unit = MARKET_SESSIONS
    calendar_basis = TRADING_DAYS
```

This document's 14 ticks stay frozen under `CALENDAR_DAYS`. Do **not** recompute or relabel them. Do not retune C3-002. The product Observatory now uses P.H.2.

`TradingDecision.horizon_trading_days` is a separate CS-P-004 field name. It does not redefine the Observatory observation window.

---

## Session resolution

```text
requested_clock
        ↓
latest certified market session ≤ requested_clock
        ↓
certified market timestamp T
```

Example that must stay visible:

| Field | Value |
|---|---|
| Requested observation clock | 14 Jun 2026, 03:45 UTC |
| Certified market timestamp | 12 Jun 2026, 03:45 UTC |

14 June 2026 is not a session. The two-day gap is provenance, not a bug.

---

## Historical Decision Replay (product)

Customer selects:

| Input | This freeze |
|---|---|
| Instrument / universe | Seven-name paper universe, or one name from it |
| Historical date | Requested observation clock |
| Policy | C3-002 only |
| Horizon | This freeze: 20 calendar days (Replay v0). Product successor: 20 market sessions (P.H.2). Not customer-selectable |

ChronoSentiment produces:

**At T**

> Certified state → Decision → Sealed  
> This decision was generated without access to information after T.

**After the historical observation window**

> Outcome → Decision Value → Evidence

The 14 August live cohort remains a different ledger and stays untouched.

---

## What stays frozen

* C.3-G
* Search #3
* C3-002 retune
* Reinterpretation of these 14 Replay v0 ticks as 20-session results
* Homepage performance aggregates
* Real capital
* The 14 August prospective seven
