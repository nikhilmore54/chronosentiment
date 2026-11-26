# CS-P-006-P.H.2 — 20 Market Session Horizon

**Document type:** Product specification correction  
**Status:** Started — Observatory horizon is 20 market sessions; Replay v1 run complete; replay integrity ≠ strategy validation  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P, CS-P-006-P.H, CS-P-006-P.H.1  
**Does not:** change C3-002, change TMV, reopen C.3-F, run Search #3, start C.3-G, mutate the 14 August seals, reinterpret Replay v0 outcomes, start a statistical performance study  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision.

---

## What this is

A product specification correction. **"20D" on the Observatory means 20 eligible market sessions after the decision timestamp.**

It is not a research change.

```text
horizon:
    duration = 20
    unit = MARKET_SESSIONS
    calendar_basis = TRADING_DAYS
    weekends = excluded
    market_holidays = excluded
```

Session 0 is the decision session. Observation closes at session 20.

```text
Decision
   ↓
Session 0 = decision session
   ↓
Session 1 … Session 20
   ↓
Observation closes
```

The exchange trading calendar is the Yahoo session series for that instrument: a session exists only when a bar exists. Weekends and holidays are excluded because they have no session.

C3-002 discovery (M.1 / C.3) used 20 calendar days. That artifact is not rewritten. The Observatory observation window is now a different, explicit product contract.

---

## What does not change

| Component | Action |
|---|---|
| C3-002 decision logic | No change |
| TMV state | No change |
| Search #2 | No change |
| C.3-F | No change |
| 14 August prospective seals | No change |
| Decision timestamp | No change |
| Replay v0 outcomes | Not reinterpreted |

---

## Replay versions

| Version | Contract | Path | Role |
|---|---|---|---|
| v0 | 20 calendar days | `observatory/historical_replay_v0/` | Archived integrity evidence |
| v1 | 20 market sessions | `observatory/historical_replay_v1/` | Product replay |

v0 remains valid evidence of the old contract. It must not be presented as 20-session results.

v1 is a **new run** of the same decide path on the same requested clocks, with due dates counted on the exchange calendar.

First v1 clocks (same requests as v0):

| Requested clock | Certified session T | 20th market session (INFY series) |
|---|---|---|
| 15 May 2026, 03:45 UTC | 15 May 2026, 03:45 UTC | 12 Jun 2026, 03:45 UTC |
| 14 Jun 2026, 03:45 UTC | 12 Jun 2026, 03:45 UTC | 10 Jul 2026, 03:45 UTC |

14 August 2026 + 20 sessions is not yet in the certified cache. The live cohort stays OBSERVING. Display may project a weekday close (**11 Sep 2026, 03:45 UTC**); the window does not close until 20 sessions have actually occurred.

---

## First v1 run

Replay `now`: 15 Aug 2026, 06:30 UTC. Same requested clocks as v0. Same C3-002. Same decide path. Different observation-close timestamps.

| Check | Result |
|---|---|
| Session counting (15 May → 12 Jun; 12 Jun → 10 Jul) | Established |
| peeked_returns | false |
| determinism | PASS |
| no-lookahead | PASS |
| prospective cohort mutated | false |
| Statistical C3-002 performance study | **Not done** |

Fourteen v1 observations are evidence that the session window closes on the exchange calendar. They are not a homepage metric. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. Replay integrity is not strategy validation.

---

## What stays frozen

* C.3-G
* Search #3
* C3-002 retune
* Homepage performance aggregates
* Real capital
* The 14 August prospective seven
* Replay v0 files
