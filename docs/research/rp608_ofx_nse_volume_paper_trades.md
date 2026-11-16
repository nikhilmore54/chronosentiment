# RP-608-OFX — NSE Volume Paper Trades

**Protocol:** `RP-608-OFX-NSE-VOLUME-PAPER-001`  
**Authority:** 0.0% · PAPER only  
**Construct:** `DERIVED_VALIDATED_AGGRESSOR_VOLUME` (Stage 0D inferred; never DIRECT)  
**Date:** 2026-11-08 · **Path gates:** 2026-11-15  

## Separation

| Book | Driver | Ledger |
| --- | --- | --- |
| Baseline NSE OF paper | L1 resting imbalance ±0.30 | `of_paper_ledger_v1/` |
| **OFX NSE volume paper** | Signed derived aggressor ΔQ | `ofx_tick_capture_v1/ofx_nse_volume_paper_*` |
| Crypto OFX volume paper | DIRECT aggTrade aggressor ΔQ | `crypto/ofx_aggtrade_capture_v1/` |

Do **not** pool books. Do **not** change baseline OF / ±0.30 / Gate 2 / authority.

## Rules (parity with crypto OFX volume paper)

- **Entry:** directional decision + **3×/20 surge** (raw sizes; no L1 OF) + **tempo gate** (suppress MIXED entry-side **or** clubbed opposite dominates).  
  Staggered entry-side → stamp same-side lookback sum; clubbed → surge tick.  
- **Exit (per symbol):** ±10% opposing match (5m hold) **or** capped clubbed completion (`opp ≤ entry×1.50`, 15m hold) **or** early-adverse abort (−8 bps / 60s).  
  Window primary; cumulative also eligible. Persist 30s on match/clubbed.  
  RELIANCE open ↔ RELIANCE opposing qty only. Session square-off still force-flats.  
- **Construct:** `DERIVED_VALIDATED` (Stage 0D) — never labeled DIRECT.  
- **Sizing:** Same participation clamps as NSE OF paper (INR)  
- **Anti-churn:** One open per symbol per 15m bar  

## Muhurat 2026-11-08

- Continuous cash **14:20–15:30 IST** (Sunday special; overrides holiday/weekend gate)  
- Warmup **1** completed 15m bar (regular 5-bar warmup impossible in ~70m)  
- Square-off bar **4**  

## Files

- Manager: `scripts/rp608_ofx_nse_volume_paper_trades.py`  
- Volume mark buffer: `scripts/rp608_ofx_nse_volume_flow_mark.py`  
- Session calendar: `scripts/rp608_nse_muhurat_session.py`  
- Wired in: `scripts/run_rp608_nse_shadow_daemon.py`  
