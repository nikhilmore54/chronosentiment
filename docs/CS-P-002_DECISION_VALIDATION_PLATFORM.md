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

1. Freeze **Decision Engine object** (CS-P-001 deliverable 1): as-of `T`, `NO TRADE` first-class, lineage, bounded fields. Contract: `adapters/chronosentiment/src/decision_support/` (`TradingDecision`). Decision Engine v1.0 remains unfrozen.
2. Replay adapter against **B4** (read-only): `decision_support::replay`. Reconstructs inputs with `evaluation_timestamp` / `effective_from` ≤ T; never reads outcomes; engine version stays `unfrozen-dev`. Official SQL proof: `./run_replay_b4_validate.sh` (disposable restore, never `chrono_b4_test`).
3. Replay/backtest orchestration + immutable decision ledger (`decision_support::backtest`). Iterates T1…Tn → `TradingDecision`. B4 schedule driver: `populate_ledger_from_assessment_schedule`. No scoring, Decision Engine v1.0 unfrozen.
4. Outcome Engine v0.1 (`decision_support::outcome`): 5/10/20/60D measurement after ledger `as_of`; never mutates decisions; no performance scoring.
5. Performance Engine v0.1 (`decision_support::performance`): ledger + outcome bundles → reproducible four-layer report. No optimization. Decision Engine v1.0 unfrozen.
6. Live / forward adapter (CS-P-003): delayed/current data; observation outcomes from raw prices after T; still no brokerage.
7. Journal + dashboard.

Do not start by designing G-GATE v1.2. Do not mutate B3/B4.

---

## 9. Outcome Engine v0.1 (bounded)

Independent of the Replay Adapter. Official SQL proof remains `./run_replay_b4_validate.sh` (disposable restore `chrono_replay_b4_validate`; never `chrono_b4_test` / `chrono_b3_test`).

```text
DecisionLedger record
        │
        ├── 5D observation
        ├── 10D observation
        ├── 20D observation
        └── 60D observation
        │
        ▼
   DecisionOutcomeBundle
```

Invariants:

1. Consume only decisions already present in the immutable ledger.
2. Evaluate after each record’s `as_of_timestamp` (parent lake decision at T; `horizon_expiry_timestamp` > T).
3. Attach existing B4 `knowledge_outcomes` fields/horizons where supported (`outcome_return`, entry/target/stop, MFE/MAE/drawdown, exit reason). Missing horizon → `available: false`; do not invent returns.
4. Never modify `TradingDecision` or `DecisionLedger`.
5. Never call `decide_at` or make another decision from the outcome.
6. Preserve lineage: ledger `decision_id` → lake `outcome_id` / lake `decision_id`.
7. Read-only SQL. Knowledge Lake `validation::outcome::OutcomeEngine` is not replaced and is not used here.
8. Decision Engine v1.0 still unfrozen. NO_TRADE still receives lake outcomes when present; opportunity-cost measurement is Performance Engine v0.1, not trading P&L.

---

## 10. Performance Engine v0.1 (bounded)

Measurement only. Official SQL proof remains `./run_replay_b4_validate.sh` (performance step on the **clean** restore, before mutating outcome/adapter tests).

```text
DecisionLedger + DecisionOutcomeBundle
        │
        ▼
PerformanceReport
  ├── trading outcomes     (LONG / SHORT only)
  ├── risk                 (per trading and opportunity path)
  ├── decision behavior    (counts, frequency, by action / horizon)
  └── opportunity cost     (NO_TRADE only)
```

Invariants:

1. Consume only `DecisionLedger` and `OutcomeReport`. No database writes. No `decide_at`.
2. Never modify `TradingDecision`, the ledger, or outcome bundles.
3. Never tune thresholds, search parameters, select a “best” horizon, or change LONG/SHORT/NO_TRADE rules.
4. Never feed performance back into a decision.
5. `NO_TRADE` is **not** a zero-return trade. Trading cumulative return / win rate / drawdown use LONG and SHORT only. Opportunity cost uses NO_TRADE attached lake returns separately.
6. Attached `outcome_return` is reported as stored (B4 lake path). v0.1 does not invent action-signed P&L or recompute prices.
7. `cumulative_return` is the sum of per-decision attached simple returns in ledger order (overlapping horizons are not a portfolio).
8. All four horizons are always reported. None is selected.
9. No freeze of Decision Engine v1.0. No G-GATE reopen. No B5.

---

## 11. First B4 historical product validation (`unfrozen-dev`)

Not G-GATE. Not a v1.0 freeze. Not a strategy score. Runner: `./run_b4_historical_product_validation.sh` (disposable `chrono_replay_b4_historical`; never `chrono_b4_test` / `chrono_b3_test`). Artifact: `product_validation/B4_unfrozen_dev/`.

Pipeline: certified B4 → Replay Adapter → `TradingDecision` → `DecisionLedger` → Outcome Engine v0.1 → Performance Engine v0.1 → historical performance report. Two consecutive runs must match `performance.content_hash`.

Decision Engine version on this report is **`unfrozen-dev`**. Do not present it as a production trading strategy.

**Known limitation (do not mutate B4):** 85 SHORT decisions have no attached lake outcomes; 0 NO_TRADE. The historical report cannot evaluate SHORT. See `product_validation/B4_unfrozen_dev/KNOWN_LIMITATIONS.md` and CS-P-003.

CS-P-002-R1 is the **baseline**, not the research programme. Historical discovery (regime, walk-forward, robustness, decision-vs-baseline) is CS-P-004, using this pipeline with **no engine change**. CS-P-003 remains the forward confirmation clock and may run in parallel. Not G-GATE, not parameter tuning.
