# RP-608 Gate 5 Packet — Crypto Adaptive TP Runner

**Status:** **ACCEPTED · DEPLOYED** (2026-11-21) — Pi crypto + NSE shadows restarted with Adaptive TP  
**Protocols:** `RP-608-GATE5-CRYPTO-ADAPTIVE-TP-001` · `RP-608-GATE5-NSE-ADAPTIVE-TP-001`  
**Authority:** **0.0%** (unchanged)  
**Books:** `CRYPTO_PLAN` · `CRYPTO_PLAN_BASELINE_PARALLEL` · NSE Estimand A (never pooled)  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · Gate 2X v1.4

## Production change — crypto

In `scripts/rp608_trade_plan_overlay.py` continuous L1 evaluation:

1. **Open geometry unchanged:** stop = **1.0×ATR**, first target = **1.5×ATR**.
2. When executable path tags the **current** TP and `tp_level < max_extensions` (default **3**):
   - **Do not** close `PROFIT_TARGET_HIT`
   - Extend TP by **+0.5×ATR** (frozen `atr_14` at entry)
   - Lock SL to the **touched** TP (never loosen)
   - Increment `tp_level`; stamp `adaptive_tp_protocol`
3. At max extensions, next TP touch → hard close `PROFIT_TARGET_HIT` / `CLOSED_TP`.
4. Ambiguous TP∧SL: if `tp_level >= 1` prefer **locked SL**; else Gate 5 G (TP).
5. Gate 2X Hold/Protect/Exit still owns giveback continue-vs-exit after adaptive arm.

Config: `adaptive_tp_enabled=True` · `adaptive_tp_extend_atr_mult=0.5` · `adaptive_tp_max_extensions=3`.

## Production change — NSE Estimand A (parity)

In `scripts/run_rp608_nse_shadow_daemon.py` tick + bar manage (LONG only):

1. Open TP/SL stay entry-frozen (dynamic ATR/structure or Arm A ±1.5%/±1.0%).
2. Same extend/lock/max-3 / ambiguous→locked-SL / Gate 2X continue-vs-exit as crypto.
3. Step = **+0.5×ATR** when `atr_14` present; else **+0.5% of entry**.
4. Env: `RP608_NSE_ADAPTIVE_TP=1` (default) · kill-switch `=0`.

## Scope limits

- No OF ±0.30 / OFX / traj15 entry / horizon / sizing change
- Initial open target/stop multipliers unchanged; books never pooled

## Rollback

- Crypto: `adaptive_tp_enabled=False`
- NSE: `RP608_NSE_ADAPTIVE_TP=0`  
Gate 5 G/H unchanged.

## Monitor

- Fields: `tp_level`, `adaptive_tp_touch_ms`, `adaptive_tp_protocol`
- Crypto: fewer instant `PROFIT_TARGET_HIT` at first touch
- NSE: fewer instant `TAKE_PROFIT` at first touch; runners with `tp_level>=1`
- Per-book W/L + payoff + net (never pooled)
