# RP-608 Gate 5 Packet — Crypto Plan Policy H120_PATH

**Status:** **ACCEPTED · DEPLOYED** (2026-11-16)  
**Authority:** **0.0%** (unchanged)  
**From:** Gate 3X `PASS_GATE3X_DRAFT_GATE5_H_PACKET_BOTH`  

## Production change (live on Pi crypto shadow)

In `scripts/rp608_trade_plan_overlay.py` continuous L1 evaluation:

- Track L1 mid adverse beyond entry±6 bps, then revisit into the band  
- **When:** ATR stop would fire and `hold < 120s` **and** adverse→revisit path stamps are true  
- **Then:** `CLOSED_SCRATCH` / `EARLY_SL_PATH_SCRATCH` at `entry_price` (gross 0 − RT friction)  
  (`gate5_policy_h=RP-608-GATE5-CRYPTO-H-001`, `early_sl_path_scratch_max_hold_s=120`)  
- **Else:** keep ATR `STOP_LOSS_HIT`  
- Adverse-only (no revisit) → **keep SL** (PATH, not OPT)

Gate 5 G (`ambiguous_tie_break=TP`) unchanged.

## Deploy record

| Item | Value |
| --- | --- |
| Host | Pi `ultraspeed@192.168.0.9` |
| Module | `scripts/rp608_trade_plan_overlay.py` v1.2.4 |
| Unit tests | `test_06b_early_sl_path_scratch_gate5_h` · `test_06c_early_sl_without_revisit_keeps_stop` · G still PASS |
| Daemon | `rp608-shadow-observer` restarted after deploy |
| Config | `early_sl_path_scratch_enabled=True` · max hold 120s · authority `0.0` |

## Scope limits (unchanged)

- Crypto recommendation plan only  
- No ATR / horizon / sizing / OF ±0.30 change  
- No OPT-all early SL scratch  

## Monitor

Watch `trade_plan_outcomes.jsonl` for `EARLY_SL_PATH_SCRATCH` / `exit_reason_detail=EARLY_SL_PATH_SCRATCH_GATE5_H120`.

## Rollback

Set `early_sl_path_scratch_enabled=False` (or revert overlay); Gate 5 G remains.
