# RP-608-OFX — Volume Paper Trades

**Protocol:** `RP-608-OFX-VOLUME-PAPER-001`  
**Authority:** 0.0% · PAPER only  
**Construct:** `EXECUTED_AGGRESSOR_VOLUME`  
**Date:** 2026-11-08 · **Path gates:** 2026-11-15  

## Separation

| Book | Driver | Ledger |
| --- | --- | --- |
| Baseline crypto OF paper | L1 resting imbalance ±0.30 | `trade_plan_overlay_v1/of_trade_outcomes.jsonl` |
| **OFX volume paper** | Signed aggressor ΔQ / volume imbalance | `ofx_aggtrade_capture_v1/ofx_volume_paper_outcomes.jsonl` |

Do **not** pool the two books. Do **not** change baseline OF / ±0.30 / Gate 2 / authority.

## Rules

- **Entry:** Policy B LONG/SHORT decision + **entry tick volume ≥ 3× average of the preceding 20 completed ticks** (raw trade sizes only). No L1 OF / reverse-OF.  
  **Tempo gate:** suppress **MIXED** entry-side tempo (path analysis net drag) **or** when **clubbed opposite** dominates a thin/staggered entry side.  
  **Entry qty stamp:** if entry-side tempo is **STAGGERED**, stamp qty = same-side lookback sum; if **CLUBBED**, stamp the surge tick.  
- **Exit:**  
  1. opposing qty in **−10%..+10%** band vs entry qty (min-hold 5m + persist 30s), **or**  
  2. **capped clubbed opposing completion** — opposite tempo CLUBBED, opp in `[entry×0.90, entry×1.50]`, min-hold **15m**, **or**  
  3. **early-adverse abort** — first-minute MAE ≤ −8 bps with MFE < 10 bps (bad-entry proxy).  

- **Sizing:** Fixed $100 notional (paper)  
- **Friction:** Same fee/spread model as L1 OF paper (`compute_of_paper_cost_breakdown`)  
- **Anti-churn:** One OFX volume paper open per 15m bar  

## Files

- Manager: `scripts/rp608_ofx_volume_paper_trades.py`  
- Shared mark: `research/rp608_ofx/executed_flow/volume_mark.py`  
- Wired in: `scripts/run_rp608_shadow_daemon.py`  
- Outcomes / open / summary under `live_capture/shadow_validation/crypto/ofx_aggtrade_capture_v1/`  

