# RP-608-SCI-ADAPTER-REC-ML1D — P1b live-plan transfer

**Date:** 2026-11-27 Asia/Calcutta  
**Package:** `ML1D_P1B_LIVE_PLAN_TRANSFER`  
**Verdict:** **DO_NOT_ADVANCE_TRANSFER_OBSERVATIONAL** (no twin · no Gate 5)  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed  

Frozen **2024** ML-1D reliability packs + cuts applied to **2026** crypto trade-plan TP/SL outcomes with real `net_pnl_usdt`. Candles: Binance BTCUSDT 15m 2026-09-01→2026-11-27.

---

## Join

| Item | Value |
| --- | --- |
| Plans source | `artifacts/.../trade_plan_outcomes.jsonl` (overlay 001) |
| Plans n | 1014 |
| Joined (TP/SL) | **890** |
| Horizon/ambiguous censored | 124 |
| Map/feat skips | 0 / 0 |
| Entry span | 2026-10-26 → 2026-11-06 UTC (~11d) |

---

## Scorecard (live net USDT)

| Exp | n | WR | cov | mean net USDT |
| --- | ---: | ---: | ---: | ---: |
| **E0** (always ACT) | 890 | 0.403 | 1.00 | **−0.30** |
| **E1** (R only) | 554 | **0.487** | 0.62 | **+0.63** |
| **E3** (dual) | 29 | 0.448 | 0.03 | −0.94 |

E1 lifts closed W/L (~+8.4 pp) and flips mean net positive vs E0 on this short live window. E3 thins to n=29 and destroys net.

---

## Why still DO_NOT_ADVANCE

1. ~11-day label span — **no sealed train/val/holdout firewall**  
2. Transfer of 2024 packs → 2026 plans is **one observational window**, not a predeclared multi-fold package  
3. No Gate 5 · no Pi twin · no authority  

**Supportive signal only:** reliability ACT/PASS (E1) may transfer better than dual Ec1 on live friction nets — worth a longer P1b once more months of plans+candles exist.

---

## Artifacts

- Candles: `datasets/crypto_15m/binance_btcusdt_2026q4_15m.json`  
- Report JSON: `docs/research/rp608_sci_adapter_rec_ml1d/btc_ml1d_p1b_transfer_report.json`  
- Runner: `rp608_sci_adapter_rec_ml1d_p1b_transfer`

---

*End — P1b transfer · 2026-11-27 Asia/Calcutta.*
