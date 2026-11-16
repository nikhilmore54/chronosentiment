# RP-608 — Gate 2X-001 Multi-Session Exit/Cost Package

**Protocol:** `RP-608-GATE2X-001`  
**Parent:** `RP-608-GATE2-AMENDMENT-V2`  
**Date:** 2026-11-15  
**Authority:** 0% · production journal **read-only** · no live exit edits  

## Objective

On **frozen identical entries**, test whether predeclared exit/cost challengers improve **net expectancy** vs production (per book). Primary metric: net; W/L disclosed; books never pooled.

## Data constraint (binding)

| Book | Path replay (CLOSE_GRID) | Journal-anchored CF |
| --- | --- | --- |
| NSE A | **Blocked** — durable `rp608_nse_bar_history.json` on Pi is empty (2026-11-15); Nov 2 artifact is single-session only | **Authorized this tranche** |
| Crypto plan | Tick path not durable for full SL delay replay | **Authorized** for mark-known CFs |

Full A–E PATH_ONLY multi-session (EXIT-FALSIFICATION-001) is **queued** until bar history is restored on Pi. It does not block journal-anchored 2X.

## Predeclared policies (frozen before results)

### NSE Estimand A

| ID | Rule | Intent |
| --- | --- | --- |
| **A** | Production journal outcomes | Control |
| **F** | If `exit_reason=TARGET_HORIZON_60M` and `net_pnl_inr < 0`, replace exit with **scratch at entry** (gross 0, statutory statutory friction for flat round-trip at entry) | Friction-aware gate: do not take post-friction-red “greens” |
| **F0** | Same cohort: replace net with **0** (void the exit PnL; optimistic “never took the trade’s horizon P&L”) | Upper-bound diagnostic only — not a production proposal |

### Crypto plan

| ID | Rule | Intent |
| --- | --- | --- |
| **A** | Production journal | Control |
| **G** | If `AMBIGUOUS_STOP_TARGET`, exit at **TP** price instead of SL | Fix adverse tie-break |
| **H60** | If `STOP_LOSS_HIT` and hold &lt; 60s, **scratch at entry** (gross 0 − modeled RT friction) | Suppress microstructure noise stops |
| **H120** | Same with hold &lt; 120s | Wider noise window |

## Power / ADVANCE rule (from Gate 2 v2 §2.2)

- Multi-session required for Gate 2X ADVANCE (session-clustered).  
- Single-session PATH_ONLY = diagnostic only.  
- This tranche: journal-anchored across all IST sessions in the pulled journals.  
- **ADVANCE** only if a non-diagnostic policy (F, G, H60, or H120) beats A on **net** with session-robust sign (majority of sessions Δnet ≥ 0 or predeclared bootstrap); W/L-only wins that worsen net = fail.  
- F0 cannot ADVANCE (diagnostic bound).

## Explicit non-goals

- Live daemon changes  
- Entry/sizing changes  
- OF ±0.30 / OFX as exit evidence  
- Remining EXIT-FALSIFICATION single-session as multi-session ADVANCE  

## Deliverables

1. This scope  
2. `scripts/research/rp608_gate2x_001_multisession.py`  
3. `artifacts/rp608_gate2x_001/` results JSON + MD  
4. Gate 2X review memo  

## Ops follow-up (not this script)

Restore durable NSE 15m bar history on Pi so EXIT-FALSIFICATION A–E can run multi-session CLOSE_GRID.
