# CS-P-002 — ChronoSentiment Decision Validation Platform v1

**Document type:** Product architecture brief  
**Status:** Active  
**Date:** 2026-08-14  
**Parent:** CS-P-001  
**Does not supersede:** EV-GOV-003, G-Extension Methodology v1.1, B3, B4  
**Does not open:** G-GATE v1.2, B5, v1.1 rerun

This is **not** a predictive-value methodology. It does not replace G-GATE v1.1 and must not be used to reopen it.

**Objective:**

> Build a reproducible system that can generate decisions historically and prospectively, execute them under realistic paper-trading conditions, and continuously measure whether those decisions create useful trading outcomes.

---

## 1. Two regimes, one engine

```text
                         CHRONOSENTIMENT
                               │
                 ┌─────────────┴─────────────┐
                 │                           │
          HISTORICAL REPLAY              LIVE DATA
                 │                           │
                 ↓                           ↓
          Replay Adapter               Live Adapter
                 │                           │
                 └─────────────┬─────────────┘
                               ↓
                       Decision Engine
                               │
                 ┌─────────────┴─────────────┐
                 ↓                           ↓
        Simulated execution           Paper execution
                 └─────────────┬─────────────┘
                               ↓
                     Common Outcome Engine
                               │
                               ↓
                     Performance Analysis
```

Backtest and forward test **share the same Decision Engine**. Adapters supply as-of data. They must not implement different decision logic.

Forbidden:

```text
Backtest algorithm  ≠  Live algorithm
```

Required:

```text
Decision Engine  ←  Replay Adapter (historical, ≤ T)
Decision Engine  ←  Live Adapter (current, ≤ now)
```

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; event-driven; no randomness in strategy logic.

---

## 2. Temporal firewall

For every decision timestamp `T`:

- Inputs are artifacts and observations with as-of time `≤ T`.
- The engine computes state and a decision (`TRADE` or `NO TRADE`).
- Simulated / paper execution may use market evolution **after** `T` only as **outcome**, never as input.
- Nothing is rewritten after the outcome is known.

This is the same invariant repaired for B4 (`assessment.evaluation_timestamp` stamped from replay `dt`, not `Utc::now()`). B4 is the **certified historical foundation** for replay. B3 remains an immutable leakage-fail historical artifact. Neither dump is mutated.

Existing code to reuse, not duplicate: `TemporalFirewall` / `DecisionReplay` in `adapters/chronosentiment/src/validation/replay_decision.rs`; `AssessmentEngine::assess_at`.

---

## 3. Engine versioning (no silent reruns)

Decisions are produced by an explicit engine version:

```text
Decision Engine vN
  ├── Test A  Research backtest
  ├── Test B  Walk-forward
  ├── Test C  Unseen holdout
  └── Forward tests (shadow → paper → extended paper)
```

If results are poor: **do not change vN and rerun it.** Author **vN+1** with a documented delta, then compare evidence:

```text
vN   → evidence bundle (immutable)
vN+1 → evidence bundle (immutable)
```

Decision Engine v1.0 is **not yet frozen**. Freezing it is a later product act: typed Decision object, bounded parameters, hashed config. Until that freeze, no claim of a completed backtest of “the” ChronoSentiment engine.

---

## 4. Historical tests (progressively harder)

All chronological. No training on future information. Walk-forward windows are examples, not a freeze.

| Test | Question |
|------|----------|
| **A — Research backtest** | If the frozen engine had run historically, what decisions and outcomes? |
| **B — Walk-forward** | Does performance hold when train/validate/test windows roll forward in time? |
| **C — Unseen holdout** | Completely untouched period after freeze. |

G-GATE v1.1’s 55/27/28 ranks and Holm/AUC detection rules are **not** reused here. That protocol is closed.

---

## 5. Forward tests (after the historical engine is frozen)

| Stage | What happens |
|-------|----------------|
| **1 — Shadow** | Engine emits immutable timestamped decisions. No simulated execution required yet. |
| **2 — Paper** | Simulated orders; actual subsequent market used only as outcome. |
| **3 — Extended paper** | Meaningful calendar span across regimes. |

Real capital is not in this milestone. PRD v1.0: no live brokerage, no automated trading.

---

## 6. Measurement catalog (not a single gate)

Do not reduce the platform to one AUC / DETECTED label. Four layers, all reported; none is a silent substitute for the others.

**Prediction quality:** ROC-AUC, precision/recall, hit rate, calibration, Brier, confidence reliability.

**Trading performance:** gross/net return, CAGR, Sharpe, Sortino, max drawdown, profit factor, expectancy, win/loss, turnover.

**Execution realism:** slippage, transaction costs, latency, liquidity, partial fills, position limits.

**Decision quality:** correct trade, incorrect trade, correct no-trade, missed opportunity, adverse regime, confidence vs outcome, performance by regime and by horizon.

AUC here is a **dashboard metric**, not G-GATE v1.1 classification.

---

## 7. Milestone components

**ChronoSentiment Decision Validation Platform v1**

1. Historical Replay / Backtest Engine (replay adapter + temporal firewall).
2. Walk-Forward Validation Engine (rolling chronological windows).
3. Paper-Trading / Forward-Test Engine (live adapter; prefer existing `financial/strategies/src/paper.rs` after ownership check).
4. Common Outcome & Performance Engine (shared by backtest and forward).
5. Decision Journal + Dashboard (current state, backtest / walk-forward / forward panels, reliability including NO-TRADE accuracy).

Dashboard copy is illustrative. It must not invent numbers.

---

## 8. First build order

1. Freeze **Decision Engine object** (CS-P-001 deliverable 1): as-of `T`, `NO TRADE` first-class, lineage, bounded fields.
2. Replay adapter against **B4** (read-only).
3. Live adapter (delayed/current data; still no brokerage).
4. Common outcome + metric layers.
5. Journal + dashboard.

Do not start by designing G-GATE v1.2. Do not mutate B3/B4.
