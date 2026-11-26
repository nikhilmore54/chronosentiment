# Gate 2E B — 15m OHLCV trend-agree interim review

**Protocol:** `RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001`  
**Date:** 2026-11-26 (Asia/Calcutta)  
**Verdict:** `DO_NOT_ADVANCE` — twin armed in code; chronological soak / closed-trade floor not yet met.  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · scoped memo `rp608_gate2e_15m_ohlcv_trend_agree_scoped.md` · Gate 2 v2.

## Boundary locks

| Lock | Status |
| --- | --- |
| Coralys (`coralys-*`) | NO_TOUCH |
| traj15 / OF ±0.30 / stops/targets | UNCHANGED (Gate 5 not opened) |
| PRIMARY Gate2E stack (lever A) | UNCHANGED control |
| Agreement math | Rust `ohlcv_trend_agreement` via `rp608_position_decide --trend-agree-loop` |
| Python | I/O client only; fail-closed if bin missing |
| Books | Never pooled |

## What shipped

- Rust pure feature + CLI loop + unit tests
- Crypto overlay B twin (`trade_plan_overlay_trend_agree_parallel_v1`, `require_trend_agree`)
- NSE Estimand A B twin (`estimand_a_trend_agree_parallel_v1`) — BAR_CLOSE + TICK_PATH entry filter
- Scorecard: `scripts/research/rp608_gate2e_trend_agree_status.py` (W/L-first)

## Advance criteria (unchanged from scoped memo)

B → `ADVANCE_CANDIDATE` on a book only if holdout W/L improves vs BASELINE **and** (vs A when A has closed n≥20, else vs BASELINE with coverage floor). Payoff/net disclosed. No Gate 5 / no PRIMARY cutover from this review alone.

## Next

1. Deploy built `rp608_position_decide` with `--trend-agree-loop` on Pi (Anuj-gated restart).
2. Soak; run scorecard; re-review when B closed n meets floor.
3. Until then: `DO_NOT_ADVANCE`.
