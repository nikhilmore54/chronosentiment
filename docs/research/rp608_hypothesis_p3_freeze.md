# RP-608 — Hypothesis Definitions Freeze (P3)

**Date:** 2026-11-06  
**Depends on:** P2 `artifacts/rp608_econ_edge_diag/rp608_crypto_hypothesis_availability_matrix.md`  
**Programme:** `rp608_hypothesis_validation_programme.md`  
**Authority:** 0% · Gate 2 CLOSED · observational only  

## Freeze rule

These definitions are locked **before** P4 runs. No threshold scans, no post-hoc field mining, no book pooling. Changing a frozen definition requires a new protocol id.

---

## H2 — Session regime (FIRST P4 TEST)

### Question (locked)

Is observed WR / net variation on **crypto plan** primarily associated with **session-level** conditions rather than a stable within-session entry-state relationship?

### Book

**Crypto plan only** for the primary test (`trade_plan_outcomes.jsonl`).  
Crypto OF paper: optional **descriptive** companion (thin n); not pooled; not used to “confirm” plan.

### Outcomes (locked)

| Metric | Definition |
| --- | --- |
| Win | `net_pnl_usdt > 0` |
| Loss | `net_pnl_usdt ≤ 0` |
| WR | wins / (wins+losses) |
| Payoff | mean\|win\| / mean\|loss\| on net_pnl_usdt |
| Net | sum `net_pnl_usdt` |
| Friction | sum `friction_usdt` (existing modeled) |

### Eligible population

All crypto plan closed outcomes with non-null `entry_time_ms` and `net_pnl_usdt`.  
Session key: IST calendar day of `entry_time_ms`.

### Predictors / contrasts (H2)

- Session identity and session-level aggregates only for the primary claim.  
- Optional secondary (reported, not optimized): within-session split by `order_flow_supported` ∈ {True, False, Missing} **without** dropping Missing silently — feeds H3 descriptive overlap, not H2 success.

### Exclusions

- No entry filters, thr scans, or best-session promotion as policy.  
- No NSE data. No crypto OF pooled into plan metrics.  
- No `ret_15m_bps` as predictor.  
- No velocity / latency claims.

### Chronological populations (crypto plan, IST) — LOCKED

| Role | Sessions | Approx n (from P2) |
| --- | --- | ---: |
| **Discovery** | `≤ 2026-10-31` | ~554 |
| **Validation** | `2026-11-01` … `2026-11-05` inclusive | ~441 |
| **Confirmation** | `≥ 2026-11-06` (+ future) | ~42 today — **thin**; do not lower bar |

Discovery may **inspect** session heterogeneity.  
Validation evaluates **predeclared** session-regime summary stats only (see pass/fail).  
Confirmation used only if validation warrants; if n too small, status = `NEED_MORE_SESSIONS` (honest), not pass.

### Predeclared H2 analyses (no fishing)

1. Discovery: per-session n, WR, payoff, net; dispersion across sessions.  
2. Discovery: fraction of total wins/net from top session (concentration).  
3. Validation: same table on validation days **only** — compare whether discovery’s “strong session” pattern reappears as the same structural story (high cross-session WR variance; concentration risk).  
4. Report all sessions, including weak ones.

### Interpretation (H2)

| Result | Label |
| --- | --- |
| High cross-session WR variance; within-book entry-state not required to explain most WR gaps | **Session-regime dominant** (descriptive) — not an entry policy |
| Validation shows similarly unstable session WR without a transferable session predictor frozen in discovery | Supports B3-style instability; **not** Gate 2 advance |
| A frozen session-level predictor (none authorized in this freeze) would be needed for policy claims | Out of scope for this H2 freeze |

H2 success for governance is **clarity**, not a tradable rule. A “pass” means the session-regime account is documented with validation; it does **not** authorize filters.

---

## H1 — Continuous entry quality (FROZEN AS NOT RUNNABLE YET)

| Item | Freeze |
| --- | --- |
| Question | Do decision-time continuous mom / trend_slope / price_location distinguish W/L+economics within signal population? |
| Crypto plan/OF | **Blocked** — native fields absent (P2) |
| Proxies | **Not authorized** in P4 until a separate proxy protocol (leakage-cleared card join) is registered |
| Status | `NOT_READY` — do not run ad hoc |

---

## H3 — OF alignment (FROZEN PROTOCOL; RUN AFTER H2 IF SCHEDULED)

### Construct lock (2026-11-07)

If/when H3 opens, `order_flow_supported` means **decision-time L1 resting liquidity imbalance support** (`ACCEPT_CONSTRUCT_DISTINCTION`). It is **not** executed aggressor alignment and **not** absorption. Aggression + response hypotheses belong under **RP-608-OFX**, not H3-on-baseline-OF. H3 remains **UNOPENED**.

### Question (locked)

On **crypto plan**, does decision-time `order_flow_supported` associate with W/L, payoff, net, with session stability?

### Strata (locked — Missing is first-class)

1. `order_flow_supported == True`  
2. `order_flow_supported == False`  
3. `order_flow_supported` null/absent  

Never drop stratum 3 without reporting it.

### Populations

Same Discovery / Validation / Confirmation session cuts as H2.  
Crypto OF outcomes: **H3 not runnable** (OF fields null on outcomes per P2).

### Exclusions

No OF ±0.30 change; no claim that OF proves entry/exit defect; no pooling; no reinterpretation of `OF_SUP` as executed buying pressure.

### Run order

**After** H2 P4 report. Optional; not required to close H2.

---

## H4 — Adaptive execution velocity

**`UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN`.** See `rp608_h4_current_engine_boundary.md` and P1 `rp608_crypto_paper_engine_capability_audit.md`. Reopen only after authorized engine redesign.

---

## P4 authorization (narrow)

P4 may run **H2 crypto plan** under this freeze only.  
H1 not run. H3 only after H2 artifact exists. H4 not run.  
No execution or live changes.

## P4 outcome (recorded 2026-11-06 · docs 2026-11-07)

| Item | Result |
| --- | --- |
| Artifact | `artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.{md,json}` |
| Gate memo | `docs/research/rp608_h2_crypto_plan_session_regime_p4_gate2.md` |
| H2 disposition | `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive) |
| Gate 2 | **DO NOT ADVANCE** |
| Confirmation | `NEED_MORE_SESSIONS` |

Freeze definitions above remain the authority for any H2 rerun; do not redefine windows post-hoc.

**Post-P4 (2026-11-07):** disposition frozen; next work is prospective confirmation accumulation only — `docs/research/rp608_h2_prospective_confirmation_accumulation.md`. No discovery reopen.
