# RP-608-OFX — Order Flow Response Research

**Protocol ID:** `RP-608-OFX`  
**Working name:** Order Flow Response Research  
**Disposition:** **REGISTERED · OBSERVATIONAL RESEARCH ONLY**  
**Date locked:** 2026-11-04  
**Live order authority:** **0.0%**  
**Gate R:** **UNREACHED**  
**Step 3+ (ECON-EDGE):** **HELD** — OFX does not authorize or substitute for it  

---

## 0. Governing intent

The current RP-608 order-flow paper mechanism is a **displayed L1 imbalance snapshot** entry (±0.30) with an **opposite-snapshot exit**, plus minimum hold / persistence / square-off. That is a **design class**, not merely a threshold choice.

**OFX investigates whether a fundamentally different mechanism**—aggression + liquidity response + price movement + structural confirmation—can produce incremental information. It does **not** replace ±0.30 with another imbalance threshold inside the live/paper OF books.

Central research distinction:

> The baseline measures what the current system actually does. OFX investigates whether a different order-flow mechanism can do better.

---

## 1. Separation boundary (hard)

| Component | Treatment |
| --- | --- |
| NSE Estimand A execution | **Frozen baseline** — unchanged |
| Crypto recommendation / trade-plan | **Frozen baseline** — unchanged |
| NSE OF paper (±0.30 + reverse-OF) | **Preserved baseline** — unchanged |
| Crypto OF paper | **Preserved baseline** — unchanged |
| OFX research | **Separate** implementation, data, and reports |
| OFX orders / execution authority | **None** |
| Shared mutable policy config with OF books | **Forbidden** |
| Rule / threshold / exit changes | **Not authorized** by this protocol |
| Pooling OFX results into baseline PnL | **Forbidden** |
| Gate R / live cutover | **Unreached / not authorized** |

OFX **must not** modify current entry, exit, sizing, or execution code paths. Baseline OF paper remains the comparison reference.

Cross-reference: performance tracking contract in  
[`rp608_ofx_performance_tracking_boundary.md`](rp608_ofx_performance_tracking_boundary.md).

---

## 2. Related protocols (do not conflate)

| Protocol | Relationship |
| --- | --- |
| `RP-608-NSE-OF-PAPER-TRADE-001` | Baseline OF lifecycle; **not modified** by OFX |
| `RP-608-NSE-OF-INTRABAR-ABSORPTION-001` | Complementary telemetry candidate; may feed Stage 0–1 data audit; **not** an OFX entry filter |
| `RP-608-NSE-OF-BOOK-INTELLIGENCE-001` | Displayed depth history; complementary input for liquidity response; **not** a substitute for aggression/price-response |
| `RP-608-OF-REC-PERF-001` | Matched OF vs non-OF recommendation outcomes; separate cohort work |
| `RP-608-OF-DEPTH-QUALITY-001` | Depth telemetry quality only |
| `RP-608-ECON-EDGE-DIAG-001` | Session baseline postmortem; OFX reports sit **beside**, never inside, baseline attribution |
| `RP-608-EXIT-FALSIFICATION-001` | Estimand A exit PATH_ONLY; Gate R not reached; not OFX |

---

## 3. Four research questions

1. **Aggression** — Are buyers or sellers initiating trades, and how reliable is aggressor inference (esp. NSE: inferred from trade vs bid/ask; confidence/quality required)?
2. **Response** — Does price move after that aggression, and does the move hold, reverse, or stall?
3. **Liquidity behavior** — Is aggressive flow absorbed, accepted into a new level, or rejected?
4. **Incremental value** — Does OF response add information beyond directional context and price structure alone?

### Candidate mechanisms (research hypotheses — not rules)

| ID | Setup | What is tested |
| --- | --- | --- |
| A | Absorption → reclaim → continuation | Aggressive opposing flow fails to advance price; reclaim confirms |
| B | Breakout → acceptance → retest | Aggression **with** movement and acceptance at a defined location |
| C | Failed auction → rejection → reversal | Break fails; return below/above level; retest fails |

Do **not** collapse these into “imbalance sign flipped.” Labels must be mutually exclusive **primary** classifications with secondary observations kept separate.

---

## 4. Conceptual pipeline (research architecture)

```
Market data
  → Trade aggressor inference (w/ confidence)
  → Bid/ask liquidity changes
  → Price and volume events
  → Order-flow event stream
       → Location context
       → Aggression + persistence
       → Price response
       → Setup classification
            · Absorption
            · Acceptance / continuation
            · Failed auction / rejection
            · No valid setup
            · Unavailable / ambiguous
       → Confirmation + invalidation (observational)
       → Risk / execution **simulation only**
       → Independent outcome attribution
```

**Design principle:** Order flow should explain price response—not replace price with an imbalance number. Imbalance is an observation; absorption / acceptance / rejection / continuation are hypotheses requiring independent validation.

---

## 5. Proposed module layout (not yet claimed as implemented)

```
rp608-ofx/
├── data/
│   └── immutable_market_events
├── observer/
│   ├── aggressor_inference
│   ├── liquidity_change_observer
│   └── price_response_observer
├── research/
│   ├── absorption
│   ├── acceptance
│   └── failed_auction
├── evaluation/
│   ├── event_reconstruction
│   ├── baseline_comparison
│   └── friction_analysis
└── reports/
    └── research_only
```

Workspace home for artifacts (when produced): `artifacts/rp608_ofx/`  
Scaffold note: `research/rp608_ofx/README.md` (structure marker only until Stage 1).

---

## 6. Development stages

| Stage | Name | Authorized work | Not authorized |
| --- | --- | --- | --- |
| **0** | Data feasibility | Inspect NSE/crypto trade, bid/ask, depth, timestamps; aggressor inference coverage + quality fields; reuse prior depth/intrabar audits where applicable | Strategy logic, thresholds as policy, paper influence |
| **1** | Observational engine | Independent event observer: aggression, liquidity change, price response, persistence, data quality | Recommendations, orders, shared OF config |
| **2** | Mechanism reconstruction | Mutually exclusive primary labels + secondary observations; reconstruct existing baseline OF entries’ post-entry price response (descriptive) | Causal promotion into baseline books |
| **3** | Research evaluation | Compare OFX vs OF baseline vs directional baseline; separate entry / exit / friction / concentration; OOS + session-separated validation | Live deployment, threshold swaps |
| **4** | Independent review | Evidence pack, limitations, go/no-go **research** recommendation | Any production rule change (needs **separate** governance) |

**Current stage stack:**
- **0 / 0A:** [`rp608_ofx_stage0_data_feasibility.md`](rp608_ofx_stage0_data_feasibility.md) — depth PASS (narrow); production journals lack aggressor tape.  
- **0B:** [`rp608_ofx_stage0b_kite_historical_feasibility.md`](rp608_ofx_stage0b_kite_historical_feasibility.md) — Kite OHLCV PASS; candle price-response only.  
- **0C:** [`rp608_ofx_stage0c_kite_fullmode_live_tick_feasibility.md`](rp608_ofx_stage0c_kite_fullmode_live_tick_feasibility.md) — inferred-aggressor capture feasible (not true trade tape).  
- **0D:** [`rp608_ofx_stage0d_nse_inferred_aggressor_capture.md`](rp608_ofx_stage0d_nse_inferred_aggressor_capture.md) — durable single-WS tee **live**; quality gate **PASS**.  
- **Stage 1:** [`rp608_ofx_stage1_aggression_response_observer.md`](rp608_ofx_stage1_aggression_response_observer.md) — locked `DOMINANTLY_FLAT_SHORT_HORIZON`; quality audit complete.  
- **Stage 2:** Mechanism **classifier not authorized**. Design/readiness only: [`rp608_ofx_stage2_design_readiness.md`](rp608_ofx_stage2_design_readiness.md).  

Candle-only price-response and inferred-aggressor capture remain distinct datasets. No absorption labels from Stage 1. No baseline rule changes.

---

## 7. First experiment (Stage 0→1 focus)

Not a trading strategy. For each observed aggression event, measure:

- Aggressor-side confidence / quality
- Executed volume and event persistence
- Immediate subsequent price movement
- Whether movement is proportionate to aggression
- Whether price holds, reverses, or fails to advance
- Data completeness

Initial question:

> When aggressive buying or selling occurs, does the subsequent price response contain stable, incremental information?

---

## 8. Success criteria (research)

Success is **not** “more signals” or “higher gross win rate.”

Required:

- Reliable, reproducible event reconstruction
- Sufficient data coverage (disclose gaps)
- Mutually exclusive, auditable classifications
- Incremental value vs existing baselines on **comparable cohorts**
- Economics after realistic friction, **or** explicit inconclusive documentation
- Robustness across sessions / regimes (no same-sample select-and-validate)

Failure of these tests is a **research finding**. It is **not** a reason to change the frozen execution or OF paper setup.

---

## 9. Explicit non-claims

This registration does **not** establish that:

- Current OF±0.30 is useless (poor realized books ≠ proof that “order flow” as a concept fails)
- Reverse-OF exits caused losses without entry-edge and incremental tests
- Absorption / acceptance / failed-auction setups are tradeable
- Any OFX label should become an entry or exit rule
- Step 3+ ECON-EDGE or Gate R is unlocked

---

## 10. Governance freeze statement

| Item | State |
| --- | --- |
| Production / paper entry·exit·sizing | **FROZEN** |
| OF ±0.30 / reverse-OF / hold / persistence | **FROZEN** (baseline) |
| Live order authority | **0.0%** |
| Books | **Never pooled** |
| OFX → execution substitution | **Forbidden** |
| Step 3+ | **Held** |
| Gate R | **Unreached** |

---

## References

- Performance tracking: `docs/research/rp608_ofx_performance_tracking_boundary.md`
- Session postmortem method: `docs/research/rp608_econ_edge_session_postmortem_method.md`
- Economic edge plan: `docs/research/rp608_economic_edge_diagnosis_plan.md`
- NSE OF paper: `docs/research/rp608_nse_of_paper_trade.md`
- Intrabar absorption (queued): `docs/research/rp608_nse_of_intrabar_absorption_scoped.md`
- Book intelligence: `docs/research/rp608_nse_of_book_intelligence_scoped.md`
