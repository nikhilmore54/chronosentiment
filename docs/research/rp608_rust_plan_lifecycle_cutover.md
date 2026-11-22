# RP-608 Rust plan-lifecycle cutover

**Protocol:** `RP-608-PLAN-LIFECYCLE-MANAGE-001`  
**Date:** 2026-11-21 (updated 2026-11-22)  
**Cursor rules:** `.cursor/rules/chronosentiment-core.mdc` §3c (Gate 2E/2X/5; books never pooled)

## Authority split

| Layer | Owner |
| --- | --- |
| Adaptive TP + SL/TP + Gate 2X Protect/Exit | Rust `PlanLifecycleManager` via `rp608_position_decide --plan-lifecycle-loop` |
| WS I/O, entry signals, state/journals, OF ±0.30 paper, OFX | Python crypto overlay / NSE shadow (thin executor) |
| Pattern observational exit | Python (optional paper reverse) |
| NSE bar `MAX_HOLD_HORIZON_180M` | Python bar path (Rust expiry set far for tick) |
| Crypto `TIME_EXIT` (60m) when Rust unavailable under require-rust | Python safety valve only |

## Env

- `RP608_PLAN_LIFECYCLE=1` (default on)
- `RP608_PLAN_LIFECYCLE_REQUIRE_RUST=1` — **hard fail-closed**: if binary missing or `--plan-lifecycle-loop` tick fails → log `[FATAL][plan_lifecycle] REQUIRE_RUST` and **do not** mutate via Python Adaptive TP / Gate 2X giveback / hard SL-TP manage. TIME_EXIT / NSE bar horizon may remain Python.
- `RP608_PLAN_LIFECYCLE_BIN` / `RP608_POSITION_DECIDE_BIN` / `RP608_GATE2X_MANAGE_BIN`

Without `REQUIRE_RUST`, missing bin still returns None and Python Adaptive/Gate2X path remains (soft cutover).

## Not yet retired

Full WS/daemon port and deletion of Python Adaptive/Gate2X manage bodies wait until Pi shows stable `plan_lifecycle_authority=rust_rp608_plan_lifecycle` on both books under require-rust (Track A deliverable 3).
