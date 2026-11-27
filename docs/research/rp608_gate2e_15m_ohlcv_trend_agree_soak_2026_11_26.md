# Gate 2E B — soak check (2026-11-26)

**Protocol:** `RP-608-GATE2E-15M-OHLCV-TREND-AGREE-001`  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c  
**Pi arm:** ~11:07 IST Asia/Calcutta · BTC/ETH/NSE shadow units active  
**Verdict:** `DO_NOT_ADVANCE` — filter live; closed-trade floor not met.

## Live evidence (read ~11:13 IST Asia/Calcutta)

| Venue | B activity |
| --- | --- |
| BTC | `PLAN_CREATED` ×1 (OPEN LONG ~60928); twin dir armed |
| ETH | `SUPPRESSED_GATE2E_TREND_AGREE` ×4 — `LOOKBACK_OPPOSE`, `rust_ok=true`, authority=`rust_ohlcv_trend_agreement` |
| NSE | Twin dir armed; journal not yet written (no B open this morning) |

PRIMARY / BASELINE / Gate2X twins unchanged. Coralys no-touch. No Gate 5.

## Scorecard

`scripts/research/rp608_gate2e_trend_agree_status.py` (paths fixed to Pi `trade_plan_overlay_v1` layout).

## Next

Continue chronological soak; re-run scorecard when B has closed n meeting coverage floor before any ADVANCE.
