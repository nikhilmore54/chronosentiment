# CS-P-006-P.H — Historical Observatory Replay

**Document type:** Product validation protocol  
**Status:** Achieved — 14 closed-window paper decisions; replay integrity PASS; not a statistical strategy backtest; not C.3-G  
**Date:** 2026-08-15  
**Parent:** CS-P-006-P  
**Does not:** start C.3-G, run Search #3, retune C3-002, mutate the 14 August prospective cohort, authorize real capital, build a performance dashboard  

`.cursor/rules/chronosentiment-core.mdc`: same certified state at T + same sealed policy → same decision; outcomes never construct the decision.

---

## What this is

Research stays frozen at C.3-F. The Observatory is tested.

This is **the production Observatory running against a historical clock**. Historical replay is a backtesting mechanism. This replay is not yet a statistical strategy backtest. It is not a look-ahead reconstruction of “what would we have predicted?”

```text
HISTORICAL CLOCK T
        ↓
Certified state from bars ≤ T only
        ↓
Sealed C3-002
        ↓
SEAL DECISION          ← peeked_returns = false
        ↓
────── no data from the future ──────
        ↓
Observation window
        ↓
FUTURE EVIDENCE        ← only after contractual close
```

Two execution modes share the same decide path and the same policy:

| Mode | Clock | Ledger | Status |
|---|---|---|---|
| Prospective Observatory | Live paper clock | `observatory/prospective/` | Untouched — 14 Aug 2026, 7 OBSERVING |
| Historical Observatory Replay | Historical T | `observatory/historical_replay_v0/` (calendar days, archived) and `observatory/historical_replay_v1/` (market sessions, product) | This document |

```text
BAD

Historical outcome
       ↓
Current/reconstructed knowledge
       ↓
"What would we have decided?"

GOOD

Historical timestamp T
       ↓
State available at T
       ↓
C3-002
       ↓
Decision sealed
       ↓
Historical future unfolds
       ↓
Outcome revealed
```

---

## Contract

State construction uses **only information available at T**. The decide function is called on `bars_at_or_before(bars, T)`. Future sessions are not in the slice. Poisoning later prices cannot change the sealed decision.

`generate_decision` takes certified TMV strings. It does not take bars, returns, or outcomes. `first_match_action_from_tmv` has no return input. The engine has **no mechanism** through which future evidence can influence the decision.

| Stage | Rule |
|---|---|
| State construction | Historical information only (`bars ≤ T`) |
| Policy | C3-002 / `5a43b9df…` |
| Decision | Deterministic; same engine as prospective |
| Decision timestamp | Frozen to the session ≤ T |
| Outcome | Hidden until close |
| Observation window | Replay v0: `decision_time + 20 calendar days`. Product / Replay v1: 20th eligible market session after T (`unit = MARKET_SESSIONS`). |
| Outcome reveal | `append_matured_observation` after close |
| Retrospective editing | Prohibited |
| 14 August cohort | Must not be written |

`peeked_returns = false` at seal. The sealed record does not contain `future_return`, `outcome`, `regret`, `evaluation_score`, `confidence`, or `realized_return`.

---

## Tests

1. **Determinism** — same historical state + C3-002 twice → bit-identical sealed decision.
2. **No-lookahead** — full bar history, truncated history, and future-poisoned history all produce the same decision.
3. **Evidence lifecycle** — OBSERVING → (refuse early append) → OUTCOME DUE → COMPLETED. Outcome lives on the observation, not the decision.

Customer lifecycle:

```text
SEALED → OBSERVING → OUTCOME DUE → COMPLETED EVIDENCE
```

---

## First cohort

Two requested clocks × seven names = **14 decisions**. 14 June 2026 is not a session. The historical clock uses the last session at or before that timestamp. IDEA and MAHABANK remain. Universe is not expanded.

Replay `now`: **15 Aug 2026, 06:30 UTC**. The 14 August live cohort is a different ledger and stays OBSERVING.

### Replay v0 — 20 calendar days (archived)

| Requested clock | Latest session ≤ T | Observation closes |
|---|---|---|
| 15 May 2026, 03:45 UTC | 15 May 2026, 03:45 UTC | 4 Jun 2026, 03:45 UTC |
| 14 Jun 2026, 03:45 UTC | 12 Jun 2026, 03:45 UTC (Friday session) | 2 Jul 2026, 03:45 UTC |

Sidecar: `product_validation/CS-P-006/observatory/historical_replay_v0/`

Do not present these outcomes as 20-session results.

### Replay v1 — 20 market sessions (product)

| Requested clock | Latest session ≤ T | 20th market session |
|---|---|---|
| 15 May 2026, 03:45 UTC | 15 May 2026, 03:45 UTC | 12 Jun 2026, 03:45 UTC |
| 14 Jun 2026, 03:45 UTC | 12 Jun 2026, 03:45 UTC (Friday session) | 10 Jul 2026, 03:45 UTC |

Sidecar: `product_validation/CS-P-006/observatory/historical_replay_v1/`

This is **not** a profitability claim and **not** a statistical strategy backtest. Historical replay is a backtesting mechanism. Replay integrity is not strategy validation. Session counting must be correct before any performance study is considered. Winners and losers are shown because their evidence has matured. Do not put mean / median / total V on a homepage.

Product capability: CS-P-006-P.H.2. Horizon is 20 market sessions. Session rule: latest certified market session ≤ requested clock.

---

## What stays frozen

* C.3-G experiment
* Search #3
* C3-002 retune
* Universe expansion
* Real capital
* The 14 August prospective seven
* Automatic 20D attach on the live cohort
