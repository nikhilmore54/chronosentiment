# RP-608 Crypto Recommendation Plan — Trade Pattern Prediction v1

**Protocol:** `RP-608-CRYPTO-PLAN-TRADE-PATTERN-V1`  
**Date:** 2026-11-20  
**Authority:** 0.0%  
**Book:** crypto trade-plan overlay only (never pooled with NSE / OF paper / OFX)  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc`, `.cursor/rules/rp608-ofx.mdc`, Gate 2 amendment v2

## Two uses

1. **ENTRY (Gate 2E):** match decision-time direction / UTC session / OF support / imbalance / ATR / stop-target geometry / **candle `vol_surge`** (15m volume vs SMA20) to past cohorts → ALLOW / STANDASIDE / ABSTAIN (W/L-first; payoff+net disclosed).
2. **EXIT (Gate 2X observational):** match MFE + reverse-after-green / hold / **`exit_vol_surge`** (forming/completed 15m volume vs SMA20) → HOLD / EXIT_REVERSE stamps. Frozen TP/SL/60m unchanged unless Gate 5; optional paper `RP608_CRYPTO_PATTERN_EXIT_GATE=1`.

Library schema: `crypto_plan_trade_pattern_library_v2` (remine required). Volume = Binance 15m OHLCV burst — **not** OFX aggTrade aggression (`.cursor/rules/rp608-ofx.mdc`).

## Non-goals

- No symbol bans (even though the live plan is BTCUSDT-centric).
- No OF ±0.30 change; OFX remains independent / 0% authority.
- No pooling with NSE A pattern library.
- No substituting OFX 3× tick surge for plan-pattern candle volume.

## Mine

```bash
python3 scripts/rp608_crypto_trade_pattern_library.py \
  --outcomes live_capture/shadow_validation/crypto/trade_plan_overlay_v1/trade_plan_outcomes.jsonl \
  --output live_capture/shadow_validation/crypto/rp608_crypto_plan_trade_pattern_library.json
```

## Live

| Env | Default | Effect |
| --- | --- | --- |
| `RP608_CRYPTO_PATTERN_GATE` | `allow_only` (PRIMARY systemd) | `0` off · `1`/`adverse` block STANDASIDE · **`allow_only`/`2` block unless ALLOW** |
| `RP608_CRYPTO_PATTERN_GATE_BASELINE` | `0` | Twin control — never suppress |
| `RP608_CRYPTO_PATTERN_EXIT_GATE` | `0` | reverse match → `PATTERN_REVERSE` paper close |

Wired in `scripts/rp608_trade_plan_overlay.py` (`process_decision_card`, `evaluate_market_tick`). Matcher uses **parent-cell BACKOFF** (`scripts/rp608_crypto_trade_pattern_match.py`).

**2026-11-21:** PRIMARY `allow_only` (no trade > bad trade); BASELINE gate `0`. Sealed: `rp608_gate2e_crypto_pattern_sealed_package_2026_11_21.md`.

## Code

- `scripts/rp608_crypto_trade_pattern_library.py`
- `scripts/rp608_crypto_trade_pattern_match.py`
- Tests: `tests/test_rp608_crypto_trade_pattern.py`
