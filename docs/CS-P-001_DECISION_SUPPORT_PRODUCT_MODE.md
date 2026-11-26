# CS-P-001 — ChronoSentiment Decision-Support Product Mode

**Document type:** Product programme brief  
**Status:** Active  
**Date:** 2026-08-14  
**Does not supersede:** EV-GOV-003, G-Extension Methodology v1.1, B3, B4  
**Does not open:** v1.2, B5, G-GATE rerun

This is a **product** brief. It is not a predictive-value methodology and must not be used to reopen G-GATE.

---

## 1. Closed research cycle (do not reopen)

G-GATE v1.1 is finished. Authoritative statement (EV-GOV-003):

> Predictive value was not established under the v1.1 protocol. B4 passed leakage and lineage; the experiment remained inconclusive because required bootstrap inference was undefined for some horizons.

| Artifact | Status |
|----------|--------|
| B3 | Immutable |
| B4 | Immutable |
| E-GATE v3 | PASS |
| G-GATE v1.1 | INCONCLUSIVE |
| v1.1 methodology | Closed |
| v1.2 | Not opened |
| AUC = 0.50 | Descriptive only — not a negative finding |

`INCONCLUSIVE` under v1.1 is **not** the claim “ChronoSentiment cannot be useful for trading.” Those are different propositions. Do not modify bootstrap, candidate, split, or Holm rules to force `DETECTED`.

---

## 2. Product goal (still alive)

> Build ChronoSentiment into a trustworthy decision-support system for trading decisions, initially in paper trading, with the evidence and controls necessary to know when its signals should and should not be trusted.

Authority: `docs/CHRONOSENTIMENT_PRD_V1.md` (decision intelligence, structured record, replay). Simulation / no live brokerage: PRD v1.0 out of scope includes execution and automated trading. Personal product line: ChronoSentiment does not replace the user’s final decision (`docs/ChronoSentiment_Product_Strategy_v1.md`).

Safety boundary: ChronoSentiment is a **co-pilot**. It must not control real-money orders. The user remains the authority.

`.cursor/rules/chronosentiment-core.mdc`: deterministic state machine; event-driven; no invented methodology; parameters bounded.

---

## 3. Three tracks

| Track | Status | Meaning |
|-------|--------|---------|
| Knowledge Lake integrity | Established | B4 temporally repaired; E-GATE v3 PASS |
| G-GATE v1.1 predictive-value claim | Closed / INCONCLUSIVE | No claim of predictive value under that protocol |
| ChronoSentiment product | Continue | Build the research / decision-support system |

---

## 4. What the system tells you

Outputs must be explicit. `NO TRADE` is a first-class decision. A system that always emits BUY/SELL is out of scope.

Minimum decision surface (illustrative, not a frozen scoring rule):

- Instrument, regime, sentiment state, signal, confidence, expected horizon, risk, action (`TRADE` / `NO TRADE`), invalidation, evidence factors, as-of timestamp.

No field may be derived from information after the decision as-of time.

---

## 5. Next six product deliverables

Not G-GATE experiments. Independent of v1.1.

1. **Decision Engine object** — typed inputs/outputs; `NO TRADE` first-class; as-of timestamp; lineage to assessment/decision artifacts.
2. **Signal / regime engine** — map existing sentiment, trend, momentum, volatility into actionable states (deterministic, bounded).
3. **Paper-trading engine** — decisions → simulated orders → fills → P&L → risk. Prefer promoting or wrapping existing `financial/strategies/src/paper.rs` after an ownership check (RR4 dormant-asset note), not a silent rewrite.
4. **Live decision dashboard** — what the system thinks *today*, why, and at what confidence, using only information available now.
5. **Decision journal** — every recommendation is an immutable evidence record (workspace / timeline primitives already exist in `adapters/chronosentiment`).
6. **Personal paper-trading trial** — 8–12 weeks continuous paper use before any discussion of real capital.

Evidence accumulation path (product use, not v1.1):

- Stage A: historical replay (no future information).
- Stage B: live paper trading (no real orders).
- Stage C: decision-quality analysis (calibration, drawdown, hit rate, costs, regime slices — not P&L alone).
- Stage D: shadow trading (system vs user vs outcome).

---

## 6. Milestone (CS-P-002)

The next product milestone is **Decision Validation Platform v1**: one Decision Engine, replay vs live adapters, common outcome engine, backtest / walk-forward / holdout / forward paper tests. See `docs/CS-P-002_DECISION_VALIDATION_PLATFORM.md`.

G-GATE v1.1 remains closed. B4 is the certified historical foundation for replay only. Historical research on that reconstruction is CS-P-004. Forward/paper observation (CS-P-003) is confirmation and may run in parallel; it is not the discovery laboratory.

## 7. First engineering start

When product work begins, start with **deliverable 1** (the Decision object). Then the replay adapter on read-only B4. Do not start by designing G-GATE v1.2. Do not mutate B3/B4. Do not wire live brokerage.
