# RP-609 Indian Retail Multi-Horizon Observation Protocol

**Protocol Reference:** `RP-609-INDIAN-RETAIL-MULTI-HORIZON-001`  
**Companion (frozen, do not alter):** `RP-608-NSE-SHADOW-001` intraday MIS/shadow book  
**Execution Authority:** Strictly Read-Only Observation (**Live Order Authority: 0.0%**)  
**Market:** NSE Cash Equities (delivery CNC + observational MIS link)  
**Audience model:** Retail investor in India (individual demat, CNC delivery, no leverage by default)  
**Date:** 2026-10-29  

Aligned with `.cursor/rules/chronosentiment-core.mdc`: deterministic state machines, multi-scenario robustness, sealed research ops. Aligned with ChronoSentiment Personal intent vocabulary (thesis → outcome → learning).

---

## 1. Governing objective

Extend the **observation framework** proven on RP-608 intraday shadow into the three additional retail-relevant holding styles **without** changing the frozen intraday protocol and **without** enabling live orders.

**Research questions (observe first; conclude later):**

1. Do swing (multi-session CNC) candidates selected under a frozen rule set show positive expectancy after Indian delivery costs and overnight gap risk?
2. Can a rules-based **portfolio build** (staggered CNC adds, sector/name caps) improve path dependency vs single-ticket entries?
3. Do **long-term hold** theses (months–years) produce measurable decision quality when reviewed on a fixed calendar, independent of short-horizon noise?

No horizon may promote to live authority until its own observation log has multi-day/multi-week evidence and an explicit promotion gate is written.

---

## 2. Horizon taxonomy (Indian retail)

| Horizon ID | Retail product framing | Product / product code (observational) | Intended hold | Primary exit clock |
|---|---|---|---|---|
| `H0_INTRADAY` | Intraday / MIS (linked only) | MIS (shadow; RP-608 owns execution sim) | Same session | Bars / 60m–180m rules |
| `H1_SWING` | Swing / positional delivery | **CNC** | 2–20 NSE sessions | Session count + structure invalidation |
| `H2_PORTFOLIO_BUILD` | Staggered accumulation | **CNC** (multiple lots) | Build window 5–60 sessions; hold thereafter under H3 or explicit exit | Schedule + caps |
| `H3_LONG_TERM` | Long-term holding | **CNC** | ≥ 60 sessions / thesis review cycle | Quarterly review + thesis break |

**Hard separation:** H0 remains owned by RP-608. RP-609 may **read** H0 journals for contrast; it must not mutate RP-608 daemon config, exits, or TOD observation rules.

---

## 3. Indian retail constraint surface (must be recorded)

Every RP-609 recommendation / simulated fill records:

| Field | Why |
|---|---|
| `product_type` | `CNC` (default for H1–H3); `MIS` only when linking H0 |
| `settlement` | T+1 cash equity settlement awareness (no same-day CNC recycle assumption) |
| `overnight_gap_risk` | Required for H1–H3; mark open vs prior close |
| `corporate_action_flag` | Dividend / split / bonus window (observe; do not invent adjustments in v1) |
| `friction_model_id` | `IN_EQ_CNC_FRICTION_V1` (brokerage + STT delivery + exchange + SEBI + GST + stamp + spread proxy) |
| `tax_lot_hint` | `STCG_LTE_12M` vs `LTCG_GT_12M` **hint only** (not tax advice; no computation of liability in v1) |
| `live_order_authority` | Always `0.0` in observation phase |

**Out of scope for v1 observation:** F&O, MTFs with leverage, SMS/bracket live orders, advisory language that constitutes investment advice.

---

## 4. Observation artifacts

Root: `artifacts/rp609_indian_retail_multi_horizon/`

```
journals/
  RP609_horizon_registry.json          # frozen horizon definitions
  RP609_candidates.jsonl               # append-only candidate intents
  RP609_swing_trades.jsonl             # H1 simulated CNC lifecycle
  RP609_portfolio_builds.jsonl         # H2 build schedules + fills
  RP609_long_term_holds.jsonl          # H3 thesis-linked holds
  RP609_daily_observation_log.jsonl    # end-of-day snapshot rows
  RP609_promotion_gates.json           # gates; all CLOSED initially
```

Schemas are versioned in `scripts/rp609_indian_retail_horizons.py` (`SCHEMA_VERSION`).

---

## 5. Deterministic lifecycle (per horizon)

### H1 Swing
1. `CANDIDATE` — symbol + thesis_tag + entry_zone + invalidation + max_sessions ∈ [2,20]  
2. `OPEN` — simulated CNC fill at next eligible session open/reference (policy frozen per run config)  
3. `HOLD` — daily mark-to-close; track MFE/MAE, gap opens  
4. Terminal: `TARGET` | `INVALIDATION` | `MAX_SESSIONS` | `MANUAL_OBSERVE_CLOSE`

### H2 Portfolio build
1. `PLAN` — symbol weights or equal-risk lots; `max_name_pct`, `max_sector_pct`, `n_tranches`  
2. `TRANCHE_k` — scheduled CNC adds on session calendar  
3. `BUILT` — when tranches complete or capital exhausted  
4. Terminal / handoff: remain as book under H3 review, or `BUILD_ABORTED`

### H3 Long-term hold
1. `THESIS_OPEN` — subject + thesis_version + review_cadence (`QUARTERLY` default)  
2. `HOLDING` — marks on session closes; no intraday stop required  
3. `REVIEW_DUE` — calendar gate  
4. Terminal: `THESIS_INVALIDATED` | `REBALANCE` | `HOLD_CONTINUE` (explicit)

---

## 6. Shared metrics (aggregate across scenarios)

For robustness (GA principle): report metrics **per horizon** and **across session regimes** (trend / chop proxy from RP-608 classifier when available):

- Hit rate, net expectancy after `IN_EQ_CNC_FRICTION_V1`
- Avg hold (sessions), MFE/MAE, overnight gap contribution
- Concentration: max single-name %, sector %, concurrent opens
- For H2: implementation shortfall vs single-ticket VWAP/open benchmark
- For H3: review adherence rate; thesis break latency

---

## 7. Promotion gates (all CLOSED at start)

| Gate | Requirement (summary) |
|---|---|
| G-H1 | ≥ N swing closes across ≥ D distinct sessions; net and robustness thresholds TBD after observation |
| G-H2 | ≥ N completed builds; concentration breaches = 0 under caps; shortfall distribution documented |
| G-H3 | ≥ N quarterly reviews completed; decision log complete; no live authority |
| G-LIVE | Explicit human unlock; separate from observation |

RP-608 TOD multi-day observation continues in parallel and does **not** auto-enable any RP-609 gate.

---

## 8. Implementation entrypoints

| Script | Role |
|---|---|
| `scripts/rp609_indian_retail_horizons.py` | Horizon registry, friction helpers, record constructors |
| `scripts/rp609_multi_horizon_observation_snapshot.py` | Daily observation snapshot + registry bootstrap |
| `scripts/rp608_nse_tod_observation_snapshot.py` | Unchanged H0 TOD observer |

---

## 9. Non-goals

- Do not change RP-608 intraday entry/exit/TOD rules while H0 observation runs  
- Do not place live CNC/MIS orders  
- Do not give personalized investment advice  
- Do not silently merge H1–H3 fills into the RP-608 trade journal  

---

## 10. Immediate operating posture

**OBSERVE_ONLY_NO_CONCLUSION** for H1–H3, same posture as RP-608 TOD.  
Collect candidates and daily marks for several sessions/weeks; then write conclusion notes into `RP609_daily_observation_log.jsonl` with `decision_status` updates only after multi-day evidence exists.

---

## 11. Implementation plan (v1) — executed

| Step | Deliverable | Status |
|---|---|---|
| 1 | Horizon registry + CNC friction helper | Done — `scripts/rp609_indian_retail_horizons.py` |
| 2 | Deterministic engine (H1/H2/H3 books + EOD) | Done — `scripts/rp609_multi_horizon_engine.py` |
| 3 | Operator CLI | Done — `scripts/rp609_observe.py` |
| 4 | Daily snapshot wiring (+ optional EOD) | Done — `scripts/rp609_multi_horizon_observation_snapshot.py` |
| 5 | Unit tests (target/stop/max-hold, caps, review, EOD) | Done — `scripts/tests/test_rp609_multi_horizon_engine.py` |
| 6 | Multi-day observation (parallel with RP-608 TOD) | In progress — human/ops cadence |
| 7 | Promotion gate thresholds | Blocked until observation evidence exists |

### Operator cadence (observation only)

```bash
# one-time
python3 scripts/rp609_observe.py bootstrap

# open paper positions / plans
python3 scripts/rp609_observe.py swing-open --symbol HDFCBANK --price 1650 --qty 10 \
  --stop 1580 --target 1750 --thesis swing_observe --max-sessions 8
python3 scripts/rp609_observe.py portfolio-plan --plan-id build_a --capital 500000
python3 scripts/rp609_observe.py long-open --symbol RELIANCE --price 2800 --qty 5 \
  --thesis "long-term observe" --version v1

# each session after RP-608 bars available
python3 scripts/rp609_observe.py eod
python3 scripts/rp609_observe.py report
```

EOD marks use `live_capture/shadow_validation/nse/rp608_nse_bar_history.json` last close (fallback: latest evaluation reference price). No live CNC/MIS orders.
