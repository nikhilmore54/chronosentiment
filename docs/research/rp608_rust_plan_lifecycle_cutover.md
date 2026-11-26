# RP-608 Rust plan-lifecycle cutover

**Protocol:** `RP-608-PLAN-LIFECYCLE-MANAGE-001`  
**Date:** 2026-11-21 (updated 2026-11-23)  
**Cursor rules:** `.cursor/rules/chronosentiment-core.mdc` §3c (Gate 2E/2X/5; books never pooled)

## Authority split

| Layer | Owner |
| --- | --- |
| Adaptive TP + SL/TP + Gate 2X Protect/Exit + trailing SL + partial Adaptive latch | Rust `PlanLifecycleManager` via `rp608_position_decide --plan-lifecycle-loop` |
| WS I/O, entry signals, state/journals, OF ±0.30 paper, OFX | Python crypto overlay / NSE shadow (thin executor) |
| Pattern observational exit | Python (optional paper reverse) |
| NSE bar `MAX_HOLD_HORIZON_180M` | Python bar path (Rust expiry set far for tick) |
| Crypto `TIME_EXIT` (60m) when Rust unavailable under require-rust | Python safety valve only |

## Adaptive Coralys-intelligent trail (2026-11-23)

Gate 5 geometry (not OF ±0.30 / entry change): Adaptive is **not** a blind ATR chase. After quality-path arm (Gate 2X), **Coralys** decides Hold vs Protect/Exit when giveback / reversal doubt presents; Protect|Exit → close (`ADAPTIVE_CORALYS_*`). On Hold, SL ratchets only to Coralys protect/ratchet floors (cost-clear + peak ratchet) — never loosens. **Does not latch SL on TP hit** — TP is runner-only. Tightened SL fills only on **current** bid/ask/low/high. NSE REC001h required under REQUIRE_RUST.

## Env

- `RP608_PLAN_LIFECYCLE=1` (default on)
- `RP608_PLAN_LIFECYCLE_REQUIRE_RUST=1` — **hard fail-closed**: if binary missing or `--plan-lifecycle-loop` tick fails → log `[FATAL][plan_lifecycle] REQUIRE_RUST` and **do not** mutate via Python Adaptive TP / Gate 2X giveback / hard SL-TP manage. TIME_EXIT / NSE bar horizon may remain Python.
- `RP608_PLAN_LIFECYCLE_BIN` / `RP608_POSITION_DECIDE_BIN` / `RP608_GATE2X_MANAGE_BIN`
- `RP608_NSE_REC001H_DIR` / `RP608_CRYPTO_REC001H_DIR` — REC001h evidence dirs (missing → loop fails to start)

Without `REQUIRE_RUST`, missing bin still returns None and Python Adaptive/Gate2X path remains (soft cutover).

## Track A manage retirement (2026-11-24) — **done**

Pi soak (24h): CRYPTO_BASELINE + CRYPTO_ETH_BASELINE **100% rust** (`PASS_PARTIAL`; PRIMARY silent under Gate2E).

| Change | Behavior |
| --- | --- |
| `PLAN_LIFECYCLE=1` | Python Adaptive TP / Gate2X giveback / hard SL-TP **unreachable** (Rust-only manage) |
| `PLAN_LIFECYCLE=0` | Python bodies remain for unit tests |
| `REQUIRE_RUST=1` | unchanged fail-closed if bin/loop missing |
| Still Python | WS, entries, journals, OF/OFX, pattern exit, crypto TIME_EXIT, NSE bar horizon |

**Python manage retired:** overlay + NSE early-return / skip when `PLAN_LIFECYCLE=1`; obsolete Adaptive TP-extension unit tests skipped (Coralys trail is Rust-owned). Full WS/daemon port remains Track C.
