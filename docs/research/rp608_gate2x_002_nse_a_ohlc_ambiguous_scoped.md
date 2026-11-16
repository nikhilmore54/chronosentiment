# RP-608 — Gate 2X-002 NSE A OHLC Ambiguous Tie-Break

**Protocol:** `RP-608-GATE2X-002`  
**Parent:** `RP-608-GATE2-AMENDMENT-V2` · prior `RP-608-GATE2X-001`  
**Date:** 2026-11-16  
**Authority:** 0% · production journal **read-only** · no live exit edits  

## Objective

On **frozen identical Estimand A entries**, test whether an OHLC same-path **ambiguous SL∧TP → TP** challenger improves **net expectancy** vs production (NSE A only). Primary: net; W/L disclosed. Books never pooled.

## Motivation (not a result)

- Gate 2X-001: crypto **G** (ambiguous→TP) **ADVANCE_CANDIDATE**; NSE **F** scratch **DO_NOT_ADVANCE**.
- EXIT / session diagnostics: NSE production prefers `bar_lo` SL before `bar_hi` TP on `BAR_OHLC_PATH`.
- 2026-11-16 post-13:00 PATH_ONLY diag (single-session) observed both-touch mass under that stack — **diagnostic only**; does **not** ADVANCE. This package predeclares the crypto-G analogue for NSE and tests **multi-session** journal-anchored CF.

## Data

| Input | Use |
| --- | --- |
| `live_capture/shadow_validation/nse/rp608_nse_trade_journal.jsonl` | Frozen entries + production exits |
| Pi `rp608_nse_bar_history.json` | **Not required** for this journal-anchored tranche |

CLOSE_GRID A–E / F_path remains **queued** until multi-day bars are restored (separate ops).

## Predeclared policy (frozen before multi-session results)

| ID | Rule | Intent |
| --- | --- | --- |
| **A** | Production journal `net_pnl_inr` | Control |
| **G_ohlc** | If `exit_path=BAR_OHLC_PATH` and `exit_reason=STOP_LOSS` and recorded `mae_price <= stop_loss_price` and `mfe_price >= target_price`, replace exit with **TP** at `target_price`; recompute gross; friction = production friction total (statutory-dominated bound) | Fix adverse OHLC tie-break when both sides touched |
| **G0** | Same cohort: set net **0** (void) | Diagnostic upper-ish bound only — cannot ADVANCE |

Eligibility for flip: both-touch on journal MFE/MAE vs frozen stop/target. No tuning on evaluation windows.

## Windows / power

- Sessions = IST calendar days with ≥1 Estimand A close in journal.
- **ADVANCE** only if **G_ohlc** beats **A** on **total net** and session-robust sign: **majority of sessions** with Δnet(G−A) ≥ 0 (or ≥60% if ≥10 sessions).
- W/L-only improvement that worsens net = **fail**.
- **G0** cannot ADVANCE.
- 2026-11-16 was previously PATH_ONLY-inspected → report separately as **confirm-style** day; discovery sign uses **all other sessions**; full-sample also reported for transparency.

## Explicit non-goals

- Live daemon OHLC order change / Gate 5
- Entry / traj15 / OF ±0.30 changes
- Symbol bans / hour bans
- Treating single-session diag as ADVANCE
- Scratch@entry as fill model

## Deliverables

1. This scope  
2. `scripts/research/rp608_gate2x_002_nse_a_ohlc_ambiguous.py`  
3. `artifacts/rp608_gate2x_002/` JSON + MD  
4. Review memo `docs/research/rp608_gate2x_002_nse_a_ohlc_ambiguous_review.md`  
