# RP-608 — Gate 3X Crypto Plan Policy G (Ambiguous → TP)

**Protocol:** `RP-608-GATE3X-CRYPTO-G-001`  
**Parent:** Gate 2X ADVANCE_CANDIDATE (G) · Gate 2 v2  
**Date:** 2026-11-15  
**Authority:** 0% · **no live overlay edit** until Gate 5  

## Change under test

Production (`rp608_trade_plan_overlay.py`): when TP and SL both print in one evaluation → `AMBIGUOUS_STOP_TARGET` / **SL**.  
Challenger **G**: same condition → resolve as **TP** (exit at `profit_target_price`).

Entries, sizing, ATR geometry, horizons: **frozen**.

## Offline validation (this tranche)

1. Journal CF on full history + chronological holdout (same split as Gate 2 historical).  
2. Session-paired Δnet robustness.  
3. Sensitivity: TP fill vs mid(entry, TP) haircut (conservative).  
4. Explicit non-goals: H60/H120 (separate, path-dependent); OF changes; Gate 5.

## Pass → next

If holdout Δnet > 0 and session Δ share ≥ 0.5: draft Gate 5 proposal packet (still no auto-deploy).  
Else: DO_NOT_PROMOTE.
