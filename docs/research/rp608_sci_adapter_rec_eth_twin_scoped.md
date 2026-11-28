# ETH SCI-v1 twin — scoped (audit-cleared · default off)

**Date:** 2026-11-27 Asia/Calcutta  
**Status:** Path landed · **Anuj-gated** (`RP608_SCI_ADAPTER_REC_ETH_TWIN=0`)  
**Audit:** `docs/research/rp608_sci_adapter_rec_eth_p1_frozen_audit.md` (conditional GO)  
**Rules:** chronosentiment-core §3b/§3c · never pool with BTC SCI/CLEAN · no Gate 5

## Book

| Piece | Value |
| --- | --- |
| Symbol | ETHUSDT only |
| Book id | `CRYPTO_PLAN_SCI_ADAPTER_REC_ETH` |
| Dir | `trade_plan_overlay_sci_adapter_rec_eth_v1` |
| Pack | `eth_p1_side_models.json` (override: `RP608_SCI_ADAPTER_REC_ETH_PACK`) |
| DI/regime | Bypassed (selection ownership; PATH_INVALID kept) |
| Net honesty | Offline holdout net still **−0.031** — soak W/L+payoff+net, not WR-only |

## Arm (Anuj)

1. Deploy daemon + `eth_p1` pack to Pi ETH observer.  
2. Drop-in: `RP608_SCI_ADAPTER_REC_ETH_TWIN=1`.  
3. Do **not** set BTC SCI env on ETH unit.  
4. Score with diag ledger `--root .../crypto_eth` when journals live there.

---

*End — ETH twin scoped · 2026-11-27 Asia/Calcutta.*
