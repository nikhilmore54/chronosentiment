# RP-608-GATE2X-003 — Review Memo

**Date:** 2026-11-16  
**Authority:** 0% · **no live NSE exit edits**  
**Artifacts:** `artifacts/rp608_gate2x_003/`  
**Scope:** `docs/research/rp608_gate2x_003_nse_a_close_grid_fpath_scoped.md`  

## Disposition (CLOSE_GRID relative)

**`ADVANCE_CANDIDATE:C,F_path`** — vs **CLOSE_GRID A only** (not vs live journal net).

| Policy | Δnet vs CLOSE_GRID A | Paired sessions Δ≥0 | Label |
| --- | ---: | --- | --- |
| **C** (no 60m green horizon) | **+17,867 INR** | 8/11 (0.73) | ADVANCE_CANDIDATE |
| **F_path** (continue if 60m green net≤0) | **+1,530 INR** | 8/11 (0.73) | ADVANCE_CANDIDATE |
| **E** (adverse cut) | −7,007 INR | 3/11 | DO_NOT_ADVANCE |

## Binding fidelity caveat

CLOSE_GRID A reconstructs **+9.67M INR** net (WR ~65%) on 2452 trades, while the **production journal** remains largely negative on the same book. CLOSE_GRID closes on **bar.close** and therefore **under-fires wick/tick stops** relative to live Estimand A (`BAR_OHLC_PATH` / tick path).

Therefore:

1. C / F_path ADVANCE is **relative among CLOSE_GRID policies only**.  
2. **Do not** draft Gate 5 / live horizon changes from this package alone.  
3. **Gate 3X next:** journal-vs-CLOSE_GRID A fidelity audit + path stress before any production proposal.

## Crypto status (parallel book)

Gate 5 **H120_PATH** deployed 2026-11-16 on crypto overlay; monitor `EARLY_SL_PATH_SCRATCH`. Books not pooled.

## Authorized next

1. Gate 3X NSE CLOSE_GRID fidelity package (A_PATH vs journal)  
2. Continue crypto H monitoring  
3. No Estimand A live exit edit  

## Unauthorized

Live daemon horizon/stop order change · OF ±0.30 · traj15 · treating CLOSE_GRID absolute nets as live PnL  
