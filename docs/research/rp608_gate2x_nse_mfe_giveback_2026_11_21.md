# RP-608 Gate 2X — NSE Estimand A MFE give-back protect (2026-11-21)

**Protocol:** `RP-608-GATE2X-NSE-MFE-GIVEBACK-001`  
**Authority:** 0.0%  
**Book:** NSE Estimand A paper (never pooled with crypto / OF / OFX)  
**Aligned with:** `.cursor/rules/chronosentiment-core.mdc` §3b–3c · crypto twin `rp608_gate2x_crypto_mfe_giveback_2026_11_21.md`

## Decision authority (2026-11-21 — adapter + Coralys)

Same hybrid as crypto: shadow builds features → `Gate2xPositionManager` / `gate2x_decide` → frozen `RecommendationEngineV1` via `rp608_position_decide --manage-loop` (in-process). NSE REC001h: `datasets/recommendation/historical/` (`*_NS.jsonl`). Python is executor-only; fail-closed Hold if Rust binary missing.

## Why

Crypto session audit showed quality path (MFE toward TP) then unprotected give-back into SL/horizon. Same exit-side hole applies to NSE Estimand A LONG lifecycle.

## Rule (v1.1 — match crypto)

In `scripts/run_rp608_nse_shadow_daemon.py` tick manage + durable bar manage:

1. Track peak MFE (INR) from `mfe_price` and peak **tp_frac**
2. **Arm** when peak MFE &gt; max(friction-proxy bps, 5 bps)×notional **and** peak tp_frac ≥ **0.40**
3. After hold ≥ **120s**, if current MFE ≤ **cost-clear floor** →  
   `GATE2X_MFE_GIVEBACK_PROTECT` (`TICK_PATH_GATE2X` / `BAR_CLOSE_GATE2X`)
4. Hard SL/TP still win when path hits those prices first

Friction proxy: `max(9, half_spread_bps×2+8)` (same family as entry bounds).

Env kill-switch: `RP608_NSE_GATE2X_MFE_GIVEBACK=0`

## v1.2 — volume-confirmed reversal

Same as crypto:

1. Soft-breach at protect floor  
2. Latched 15m vol burst (`gate2x_exit_vol_surge` from bar history SMA20) **HIGH** → exit now  
3. LOW/MED/UNK → dwell; exit only if still through floor  
4. Recovery above floor clears soft-breach  

Bar path refreshes volume latch each durable bar; tick path uses last latched surge.

## v1.3 — ratchet + tiered/deep dwell

Match crypto: floor `max(cost_clear, 0.30×peak_mfe)`; MED 40s / LOW 90s; deep giveback (≥50% of peak) scales dwell ×0.40.

## v1.4 — Adaptive TP runner (2026-11-21)

Paired with Gate 5 `RP-608-GATE5-NSE-ADAPTIVE-TP-001` (crypto twin parity):

1. Open TP/SL stay entry-frozen (dynamic ATR/structure or Arm A ±1.5%/±1.0%).
2. On TP touch with `tp_level < max_extensions` (default **3**): **extend** TP by **+0.5×ATR** (or **+0.5% entry** if ATR cold); **lock SL** to prior TP — do **not** hard `TAKE_PROFIT`.
3. Gate 2X Hold/Protect/Exit decides continue vs take profit after adaptive arm.
4. Snapshot fields: `tp_level`, `adaptive_armed`. Hard cap → `TAKE_PROFIT` / `ADAPTIVE_TP_MAX_EXTENSIONS`.
5. Ambiguous SL∧TP after `tp_level >= 1` → prefer locked SL.

Env: `RP608_NSE_ADAPTIVE_TP` (default on) · `RP608_NSE_ADAPTIVE_TP_ATR_MULT` · `RP608_NSE_ADAPTIVE_TP_MAX_EXT` · `RP608_NSE_ADAPTIVE_TP_PCT`.

## Not changed

traj15 entry · OF ±0.30 · **initial** ATR/dynamic stop & target at open · 60m/180m horizons · mandatory 15:00 square-off · pattern-gate defaults  

**Changed under Gate 5 Adaptive TP:** post-touch TP may extend; SL may lock to prior TP.

## Non-goals

- No pooling with crypto books  
- No OFX authority  
- No OF ±0.30 change
