# RP-608 — Gate 2X-003 NSE A CLOSE_GRID + F_path

**Protocol:** `RP-608-GATE2X-003`  
**Parent:** Gate 2 v2 · Gate 2X-001 (NSE F scratch DO_NOT_ADVANCE; CLOSE_GRID queued) · Gate 2X-002 G_ohlc DO_NOT_ADVANCE  
**Date:** 2026-11-16  
**Authority:** 0% · journals/bars **read-only** · no live exit edits  

## Objective

On **frozen Estimand A entries**, multi-session **CLOSE_GRID** replay with friction-aware **F_path** (continue past 60m green when post-friction net ≤ 0). Primary: **net**; W/L disclosed. NSE A only.

## Data

| Input | Use |
| --- | --- |
| `live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl` | Frozen entries |
| `artifacts/rp608_nse_kite_hist_bars/rp608_nse_bar_history_backfill.json` | Multi-day 15m closes (3m Kite hist; not ticks) |

## Predeclared policies

| ID | Rule |
| --- | --- |
| **A** | Production CLOSE_GRID (stop→target→square-off→max180→60m green horizon) |
| **C** | No profit-conditioned 60m horizon (continue to max/square-off) |
| **E** | Adverse cut at 60m if close≤entry; else same green horizon as A |
| **F_path** | Like A, but at 60m green: if modeled net(entry→close) ≤ 0, **continue** (do not take TARGET_HORIZON_60M) |

## ADVANCE

Challenger beats A on **total net** and majority of IST sessions Δnet≥0 (≥60% if ≥10 sessions). W/L-only / net-worse = fail. No live Gate 5 from this package alone.

## Non-goals

Live daemon edits · traj15 / OF ±0.30 · OPT wick SL/TP · crypto pooling  
