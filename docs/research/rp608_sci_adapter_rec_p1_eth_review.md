# RP-608-SCI-ADAPTER-REC-001 — ETHUSDT P1 sealed review (SCI-v1)

**Date:** 2026-11-27 Asia/Calcutta  
**Book:** `ETHUSDT_15M` — **never pooled with BTC**  
**Rules:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c · Coralys sealed · no Gate 5  
**Pack:** `docs/research/rp608_sci_adapter_rec_p1/eth_p1_side_models.json`  
**Report:** `docs/research/rp608_sci_adapter_rec_p1/eth_p1_package_report.json`

---

## Verdict

**ADVANCE_CANDIDATE** (SCI-v1 side generator on ETH) · ladder A · cuts `p_min=0.38` / `p_lb=0.18`

| Fold | n | WR | base | cov | net proxy |
| --- | ---: | ---: | ---: | ---: | ---: |
| validation | 82 | 0.366 | 0.318 | 0.035 | — |
| **holdout** | **139** | **0.331** | 0.293 | 0.059 | **−0.031** (asym 1.87) |

Also serves as **H11 negative control** for ETH ML-1D (which is now **DO_NOT_ADVANCE**).

## Governance

- No automatic Pi twin from this memo  
- ETH SCI twin (if ever) = separate Anuj arm · own book id · never pool with BTC SCI twin  
- Cuts frozen for any soak epoch — no ratchet  

---

*End — ETH P1 SCI-v1 review · 2026-11-27 Asia/Calcutta.*
