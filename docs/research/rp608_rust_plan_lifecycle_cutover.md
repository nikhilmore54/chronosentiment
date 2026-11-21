# RP-608 Rust plan-lifecycle cutover

**Protocol:** `RP-608-PLAN-LIFECYCLE-MANAGE-001`  
**Date:** 2026-11-21  
**Cursor rules:** `.cursor/rules/chronosentiment-core.mdc` §3c (Gate 2E/2X/5; books never pooled)

## Authority split

| Layer | Owner |
| --- | --- |
| Adaptive TP + SL/TP + Gate 2X Protect/Exit | Rust `PlanLifecycleManager` via `rp608_position_decide --plan-lifecycle-loop` |
| WS I/O, entry signals, state/journals, OF ±0.30 paper, OFX | Python crypto overlay / NSE shadow (thin executor) |
| Pattern observational exit | Python (optional paper reverse) |
| NSE bar `MAX_HOLD_HORIZON_180M` | Python bar path (Rust expiry set far for tick) |

Fail-closed: if binary missing or loop fails → Python Adaptive/Gate2X path remains.

## Env

- `RP608_PLAN_LIFECYCLE=1` (default on)
- `RP608_PLAN_LIFECYCLE_BIN` / `RP608_POSITION_DECIDE_BIN` / `RP608_GATE2X_MANAGE_BIN`

## Not yet retired

Full WS/daemon port and deletion of Python Adaptive/Gate2X manage bodies wait until Pi shows stable `plan_lifecycle_authority=rust_rp608_plan_lifecycle` on both books.
