# RP-608 H2 — Prospective Confirmation Accumulation

**Protocol:** `RP-608-H2-PROSPECTIVE-CONFIRMATION-001`  
**Date registered:** 2026-11-07  
**Predecessor:** P4 H2 `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive)  
**Authority:** 0.0% · Gate 2 **CLOSED** · Gate 3 unauthorized  
**Book:** Crypto recommendation plan only (never pooled)  
**Status:** **ACTIVE** under programme **CONFIRMATION-ONLY HOLDING** (`rp608_confirmation_only_holding_state.md`) — accumulate untouched sessions; no new hypothesis / workstream  

## Frozen P4 disposition (do not reopen)

| Item | Locked |
| --- | --- |
| H2 assessment | `SUPPORTED_SESSION_REGIME_DOMINANT` (descriptive session-regime finding) |
| Gate 2 | **DO NOT ADVANCE** |
| H1 | `NOT_READY` / not runnable |
| H3 | Unopened |
| H4 | `UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN` — `rp608_h4_current_engine_boundary.md` |
| Baseline / OF ±0.30 | Frozen |
| Live authority | 0% |

**Not established:** a transferable trading edge or pre-session predictor of favorable sessions.

P4 showed session WR heterogeneity in discovery (25.6 pp) and validation (39.2 pp), discovery concentration on 2026-10-29, and poor validation economics (WR 33.1%, payoff 0.53, net −747 USDT). Confirmation remains thin (n=42, one day).

## Research question (confirmation only)

> Does the session-regime dominance observed in discovery and validation persist across subsequent independently observed sessions, without relying on one exceptional day?

**Not:** Can another session achieve a high WR?  
**Not:** Which session threshold maximizes WR?

## Window (locked)

| Role | Sessions (IST) | Rule |
| --- | --- | --- |
| Discovery | ≤ 2026-10-31 | **Frozen** — do not rerun / redefine |
| Validation | 2026-11-01 … 2026-11-05 | **Frozen** — do not merge confirmation here |
| **Confirmation** | **≥ 2026-11-06** (+ future) | Accumulate only; report separately |

2026-11-06 is an **initial observation**, not independent multi-session confirmation.

**Correction (locked 2026-11-07):** The 11 historical sessions (6 discovery + 5 validation) are **already consumed** in P4. They must **not** be reclassified as confirmation. Confirmation = **new** IST sessions on/after 2026-11-06 only.

## Per-session fields to preserve

For each new eligible confirmation session:

- Full-session W/L counts and WR  
- Payoff and net (`net_pnl_usdt`) + friction  
- Trade count (coverage)  
- Contribution to confirmation-window WR dispersion (across sessions with n≥5)  
- Consistency check vs frozen story: high cross-session WR heterogeneity without promoting the best day  

Aggregate confirmation WR alone is insufficient; examine the **distribution** of session outcomes.

## Integrity

- Same eligibility as P4: non-null `entry_time_ms` + `net_pnl_usdt` on crypto plan outcomes  
- No threshold scans, feature search, or discovery rerun  
- No pooling confirmation into validation  
- No H1/H3/H4 claims; no latency/slippage inference from H2  
- Modeled friction remains modeled  

## Completeness / interpretation

**Normative assessment rules** (frozen before further evaluation):  
`docs/research/rp608_h2_prospective_confirmation_protocol.md`

**Automation:** run on Pi  
`python3 scripts/research/rp608_h2_confirmation_dq_monitor.py`  
→ `artifacts/rp608_econ_edge_diag/rp608_h2_confirmation_dq_monitor.{md,json}`  

Monitor emits `ELIGIBLE` / `INCOMPLETE` / `INTEGRITY_FAILURE` only. It does **not** declare confirmation labels (`PERSISTS` / etc.).

This accumulation charter remains the waitlist / ops doc. Do not invent new pass thresholds after seeing results.

## Artifacts

- This charter: `docs/research/rp608_h2_prospective_confirmation_accumulation.md`  
- P4 report: `artifacts/rp608_econ_edge_diag/rp608_h2_crypto_plan_session_regime_p4.md`  
- P4 memo: `docs/research/rp608_h2_crypto_plan_session_regime_p4_gate2.md`  
- Future updates: append/refresh confirmation tables via the same P4 script windowing (`>= 2026-11-06`) or a thin confirmation-only runner — **no new discovery logic**

## Observation log (confirmation window only — not H2 labels)

| As-of (IST) | Source | Powered sess | Trades | Agg WR | Agg net | DQ | Notes |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| 2026-11-07 mid | Pi DQ | 2/5 | 78/200 | 43.6% | −14.79 | `NEED_MORE_SESSIONS` | Nov 7 then n=24 WR 16.7% net −37.72; Nov 6 share ~69% |
| 2026-11-07 eve | Pi DQ | 2/5 | 88/200 | *(see session rows)* | *(see session rows)* | `NEED_MORE_SESSIONS` | Nov 7 closed n=34 W/L 4/30 WR 11.8% payoff 0.53 net −47.95; Nov 6 n=54 W/L 30/24 WR 55.6% net +22.93; top-session share 61.4% still flagged |

**Interpretation (locked with operating decision):** H2 confirmation still accumulating — no `PERSISTS` / `WEAKENS` yet. Baseline OF remains L1 resting (not aggressor). Negative aggregate net keeps WR and economics separate. Nov 7 crypto OF paper opposite-side exits remain resting-L1 observations.

Canonical monitor: `artifacts/rp608_econ_edge_diag/rp608_h2_confirmation_dq_monitor.md`

## Forbidden

- Rerunning discovery or scanning session-specific thresholds  
- Declaring success from 2026-11-06 alone  
- Declaring H2 confirmed or `WEAKENS` from Nov 6–7 alone  
- Inferring execution velocity / H4 from session WR  
- Opening H1/H3 under this milestone  
- Execution or live-authority changes  
- Changing DQ floors / criteria because one confirmation session was weak  
