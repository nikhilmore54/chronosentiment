# RP-608 Gate 5 Packet — Crypto Plan Policy G

**Status:** **ACCEPTED · DEPLOYED** (2026-11-15)  
**Authority:** **0.0%** (unchanged)  
**From:** Gate 3X `PASS_GATE3X_DRAFT_GATE5_PACKET`  

## Production change (live on Pi crypto shadow)

In `scripts/rp608_trade_plan_overlay.py`, when `hit_tp and hit_sl`:

- **Before:** `AMBIGUOUS_STOP_TARGET` / `CLOSED_SL`  
- **After:** `PROFIT_TARGET_HIT` / `CLOSED_TP` at `profit_target_price`  
  (`ambiguous_tie_break=TP`, `gate5_policy_g=RP-608-GATE5-CRYPTO-G-001`, stamp `ambiguous_resolved_as=TP`)

## Deploy record

| Item | Value |
| --- | --- |
| Host | Pi `ultraspeed@192.168.0.9` |
| Module | `scripts/rp608_trade_plan_overlay.py` |
| Unit test | `test_06_ambiguous_crossing_tie_break` PASS on Pi |
| Daemon | single `run_rp608_shadow_daemon.py` via `rp608-shadow-observer.service` only (fcntl singleton lock; do not also nohup) |
| Config seen | `ambiguous_tie_break=TP` · authority `0.0` |

## Scope limits (unchanged)

- Crypto recommendation plan only  
- No ATR / horizon / sizing / OF ±0.30 change  
- H60/H120 not in this packet  

## Monitor

Watch `trade_plan_outcomes.jsonl` for `ambiguous_resolved_as=TP` / `exit_reason_detail=AMBIGUOUS_RESOLVED_TP_GATE5_G` on new closes.
