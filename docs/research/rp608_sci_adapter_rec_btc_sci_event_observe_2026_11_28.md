# BTC SCI stacked soak — event OBSERVE (2026-11-28 Asia/Calcutta)

**Book:** `CRYPTO_PLAN_SCI_ADAPTER_REC` only (never pooled)  
**Rules:** chronosentiment-core §3b/§3c · HOLD_SOAK · no retune · no Gate 5  
**Tool:** `scripts/research/rp608_sci_adapter_rec_event_observe.py`  
**ELR:** `ELR-003` / `ELR-003-OBSERVE`

## Aggregate (fail)

| Metric | Value |
| --- | --- |
| Closed W/L | 1 / 37 (n=38) |
| Σ gross | −35.48 USDT |
| Σ friction | 45.61 USDT |
| Σ net | −112.31 USDT |
| Side mix | SHORT 28 · LONG 10 |

## Primary failure categories

| Primary | n | Note |
| --- | --- | --- |
| EXECUTION_FRICTION | 24 | Dominant closed-book primary |
| DIRECTIONAL_FAILURE | 13 | Minority but real |
| WIN | 1 | — |

## Co-evidence

- **TIMING_EXIT_GEOMETRY** on 25 losses (peak MFE ≥ friction + $1 floor — hypothesis only).  
- **OUTCOME_HORIZON_MISMATCH** on 10 (MAX_HORIZON_60M_EXPIRED losses).  
- Exit mix: GIVEBACK_PROTECT 12 · MAX_HORIZON 11 · TRAILING 8 · SL 7.  
- Soft SELECTION_WEAK_ENTER co-tags most ENTER losses (p̂ often &lt;0.40 by construction — **not** alone a cut-ratchet mandate).

## What this teaches (must change next experiment)

1. Do **not** respond with another directional classifier or ENTER threshold sweep.  
2. Dominant demonstrated weakness: **execution / exit-geometry economics** after path support appears often.  
3. Directional failure remains a secondary stream — keep separate if FIX'd.  
4. Stacked soak stays **HOLD_SOAK**; any FIX is a **new sealed package**.  
5. P1a CLEAN (ELR-004) already tests coverage ownership from ledger evidence — leave untouched; it does **not** replace an execution-economics FIX if CLEAN also fails after-cost.

## Explicit non-actions

- No modification of armed SCI soak.  
- No OF ±0.30 / stops / traj15 / Gate 5.  
- No pooling with PRIMARY / CLEAN / ETH.  
- No ML-1D rescue.

---

*End — event OBSERVE · 2026-11-28 Asia/Calcutta.*
