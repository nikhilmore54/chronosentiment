# RP-608 Trailing SL (favorable-only) · 2026-11-21

**Protocol:** `RP-608-TRAILING-SL-001`  
**Authority:** 0% paper · Rust plan-lifecycle  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · Adaptive TP / Gate 2X

## Rule

On each L1 tick (after MFE update, before Adaptive TP / Gate 2X):

1. Stamp immutable `original_stop_loss_price` at open (never moves).
2. Arm only when favorable excursion ≥ **`trailing_sl_arm_risk_frac` × initial risk** (default **0.30**).
3. Propose trail: LONG `mfe − atr_mult×ATR` · SHORT `mfe + atr_mult×ATR` (default atr_mult **0.50**).
4. Tighten only:
   - LONG: `new_sl = max(current_sl, proposed, original_sl)` and `new_sl < mfe`
   - SHORT: `new_sl = min(current_sl, proposed, original_sl)` and `new_sl > mfe`
5. **Never** move SL to cause more damage than the original stop.

Adaptive TP lock-to-prior-TP remains; it can only tighten further within the same bound.

## Config

| Key | Default |
| --- | --- |
| `trailing_sl_enabled` | `true` |
| `trailing_sl_atr_mult` | `0.5` |
| `trailing_sl_arm_risk_frac` | `0.30` |

## Exit reason (industry label)

When the **trailed** stop is hit (armed + `stop_loss_price` ≠ `original_stop_loss_price`):

| Field | Value |
| --- | --- |
| `exit_reason` | **`TRAILING_STOP_HIT`** |
| terminal state (crypto) | `CLOSED_TRAIL` |
| terminal state (NSE) | `TRAILING_STOP` |

Unmoved / initial protective stop remains **`STOP_LOSS_HIT`**. This matches common OMS/brokerage language (IBKR/TT “Trailing Stop” vs “Stop Loss”).

## Non-goals

- No OF ±0.30 change · no OFX · no pooling books  
- Does not replace Gate 2X giveback protect (v1.5 hard floor still applies)
