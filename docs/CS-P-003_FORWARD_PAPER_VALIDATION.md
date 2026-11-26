# CS-P-003 — Forward/Paper Validation v0.1

**Document type:** Product architecture brief  
**Status:** Active — operational daily tick (confirmation clock, not the research laboratory); engine policy unchanged (`unfrozen-dev`)  
**Date:** 2026-08-14  
**Parent:** CS-P-001, CS-P-002  
**Does not supersede:** EV-GOV-003, G-Extension Methodology v1.1, B3, B4, CS-P-002-R1  
**Does not open:** G-GATE v1.2, B5, v1.1 rerun, Decision Engine v1.0 freeze  
**Parallel programme:** CS-P-004 is the historical discovery laboratory. This clock may keep running; it is not where we discover basic problems.

This is **not** a predictive-value methodology and must not be used to reopen G-GATE v1.1.

**Objective:**

> Observe, prospectively, whether ChronoSentiment’s `unfrozen-dev` decisions have useful subsequent paper outcomes — without automatic trading, without tuning, and without treating the B4 historical baseline as a complete evaluation of SHORT.

---

## 1. Why this exists

CS-P-002-R1 showed the **pipeline** works and the current **policy** is not yet suitable to trust for trading:

| Finding | Meaning |
|---------|---------|
| 195 decisions, two identical runs, B4 untouched | Engineering foundation works |
| 110 LONG / 85 SHORT / 0 NO_TRADE | Provisional policy is always directional |
| 85 SHORT lake outcomes missing | B4 historical report cannot evaluate SHORT |
| LONG attached returns negative on all horizons | Honest poor baseline; not a score to optimize against |

There is **no basis** to call ChronoSentiment profitable. Do not put `unfrozen-dev` in front of real capital.

Do **not** fix this by tuning Trend thresholds, picking a horizon, or changing LONG/SHORT/NO_TRADE rules. That would open another research loop.

Do **not** modify B4 to invent SHORT outcomes. Document the coverage gap; measure every forward decision from **raw observations after T**.

`.cursor/rules/chronosentiment-core.mdc`: same input → same output; no randomness in strategy logic; no invented methodology.

---

## 2. Two regimes, one engine (unchanged)

```text
                 HISTORICAL                         FORWARD
                    B4                          Live / delayed data
                     │                                  │
                     ▼                                  ▼
              Replay Adapter                      Forward Adapter
                     │                                  │
                     └──────────────┬───────────────────┘
                                    ▼
                            Decision Engine
                            (`unfrozen-dev`)
                                    │
                                    ▼
                            TradingDecision
                                    │
                     ┌──────────────┴───────────────┐
                     ▼                              ▼
              DecisionLedger                 Forward Decision Ledger
                     │                              │
                     ▼                              ▼
         Outcome Engine v0.1              Observation Outcome v0.1
         (B4 lake rows; incomplete          (raw prices after T;
          SHORT coverage)                    LONG and SHORT)
                     │                              │
                     └──────────────┬───────────────┘
                                    ▼
                         Performance Engine v0.1
                                    │
                     ┌──────────────┴───────────────┐
                     ▼                              ▼
             Historical baseline              Forward baseline
```

`TradingDecision`, Outcome Engine v0.1 (lake attach), and Performance Engine v0.1 are **reused unchanged**. Forward measurement is an additional observation-path producer of `DecisionOutcomeBundle`. It does not replace or rewrite B4 lake attachment.

`financial/strategies/src/paper.rs` remains RR4 **Pending** (dormant). CS-P-003 does not promote or silently rewrite it.

---

## 3. Forward/Paper v0.1 (observation system)

1. Observe the market at an explicit timestamp `T`.
2. Generate a `TradingDecision` via `decide_at` (inputs ≤ T only).
3. Persist it on an append-only forward ledger. Never rewrite a prior row.
4. Record `as_of_timestamp`, `input_set_hash`, `engine_version` (`unfrozen-dev`).
5. Never look at observations after T when deciding.
6. Never make another decision from an outcome.
7. After 5/10/20/60 calendar days, measure close-to-close returns from prices **after** T that are known by a caller-supplied `now`.
8. LONG return = `(p_h - p_0) / p_0`. SHORT return = `(p_0 - p_h) / p_0`. NO_TRADE stores the unsigned instrument path for opportunity cost only.
9. Feed `DecisionOutcomeBundle` into Performance Engine v0.1.
10. Write periodic reports. Do not judge the system after a handful of trades.

**Forbidden:** broker orders, capital, threshold tuning, parameter search, feeding performance into `TradingDecision`, modifying B3/B4, G-GATE, v1.0 freeze.

Missing horizon (not yet elapsed, or no bar) → `available: false`. Do not invent prices.

---

## 4. B4 limitation (do not “repair” the dump)

Certified B4 dump SHA-256 remains `f74e576e8e98b24058cc913b14a567d9ff4b3eabc75662ab9a96901b102f8cd6`.

Lake `knowledge_outcomes` exist only for 110 as-ofs that mapped to LONG in CS-P-002-R1. The 85 SHORT decisions have no attached lake row. That is **incomplete historical coverage**, not a reason to mutate B4 or to treat SHORT as zero.

Forward/Paper v0.1 exists so LONG and SHORT are both measurable from the price path.

---

## 5. Duration

The first forward observation period should be long enough to accumulate 5D through 60D outcomes (months, not a session). Early reports are **progress snapshots**, not go/no-go gates.

Engine version remains `unfrozen-dev` until a later, explicit freeze.

---

## 6. Implementation

| Piece | Location |
|-------|----------|
| Brief | this document |
| Forward adapter + journal | `adapters/chronosentiment/src/decision_support/forward.rs` |
| Observation outcomes | `adapters/chronosentiment/src/decision_support/observation_outcome.rs` |
| Forward daily tick | `decision_support/forward_tick.rs`, `csp003_forward_session tick` |
| Session journal | `product_validation/forward_unfrozen_dev/` |
| Cadence installer | `./install_csp003_forward_schedule.sh` (16:00 local) |

Official proof: unit tests (determinism, future exclusion, LONG and SHORT both measured, ledger immutability). No live brokerage. No `chrono_b4_test` / `chrono_b3_test` writes.

---

## 7. Stop line (no further engineering in this phase)

The engineering objective for CS-P-003 v0.1 is **achieved**:

> ChronoSentiment can generate deterministic decisions historically and prospectively, preserve them immutably, measure subsequent outcomes independently, and quantify performance without feeding results back into the decision process.

Do **not** add another adapter, optimizer, dashboard, or scoring layer before the forward sample is mature. Snapshots are evidence accumulation, not a reason to tweak `unfrozen-dev`.

**Checkpoint 2026-08-14 (later):** Capability without invocation is not observation. CS-P-003 **started operationally**: `csp003_forward_session tick` against current Yahoo daily bars, `as_of` = latest session ≤ now. This is **not** a B4 replay and not a performance judgment. Decision policy remains `unfrozen-dev` with no tuning. Cadence: once per trading day (`./run_csp003_forward_tick.sh`; launchd 16:00 local).

**Checkpoint 2026-08-14 (sequence):** The 60-day forward sample is **final confirmation**, not the primary discovery mechanism. Historical understanding is CS-P-004. The daily tick **continues in parallel** so genuine future rows accumulate; do not wait on this clock to learn what 2021–2024 already contains.

---

## 9. Operational clock

```text
Yahoo daily bars (current, delayed)
        │
        ▼
decide_at(latest session ≤ now)   # one decision per ticker per as_of
        │
        ▼
product_validation/forward_unfrozen_dev/ledger.jsonl
```

- Universe: RELIANCE.NS, TCS.NS, INFY.NS, HDFCBANK.NS, ICICIBANK.NS, IDEA.NS (Vodafone Idea) (Yahoo). Lookback is for MA20/MA50 only; historical dates are not emitted as forward decisions.
- Idempotent: same `decision_id` is not duplicated.
- 5D/10D/20D/60D outcomes mature later from `prices.jsonl`.
- B4 historical report remains the backtest. Do not treat it as the forward test.

Manual: `./run_csp003_forward_tick.sh`  
Schedule: `./install_csp003_forward_schedule.sh`

**Data maturity (first useful milestone — not a performance number):**

- enough decisions to see the action distribution;
- enough 5D outcomes for short-term behaviour;
- enough 10D/20D outcomes for persistence;
- enough **60D** outcomes to complete the first full horizon cycle;
- LONG and SHORT both represented;
- NO_TRADE recorded if/when it occurs;
- no journal mutations or duplicate `decision_id` rows;
- no future-data violations.

When that sample exists, compare — without retuning:

1. Historical reconstruction: what would `unfrozen-dev` have done? (CS-P-002-R1)
2. Prospective operation: what did `unfrozen-dev` actually decide? (forward journal)
3. Outcome: what happened after those decisions? (observation path)

That comparison is the product validation chain. It is not G-GATE v1.1 classification.

---

## 8. Eventual Decision Engine v1.0 decision (not now)

Only after a sufficiently mature forward sample:

1. **Freeze v1.0** — evidence is sufficiently useful.
2. **Author a documented successor** (e.g. v1.1) — a specific improvement, not a silent rerun.
3. **Do not promote** — evidence is not good enough.

None of those may be decided from a handful of trades. Until then, engine version remains `unfrozen-dev`. No real capital.
