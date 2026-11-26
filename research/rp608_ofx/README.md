# RP-608-OFX — workspace scaffold

**Status:** Stage 0–0D + Stage 1 observer live · Stage 2 not authorized  
**Authority:** 0.0% · does not touch RP-608 execution or OF paper code  

Canonical scope: `docs/research/rp608_ofx_order_flow_response_scoped.md`  
Stage 1: `docs/research/rp608_ofx_stage1_aggression_response_observer.md`  
Artifacts: `artifacts/rp608_ofx/`

```
rp608-ofx/   (this tree — research-only)
├── data/           # immutable event extracts (Stage 0+)
├── observer/       # Stage 1 aggression → price-response helpers
├── executed_flow/  # signed executed-qty ΔQ (DIRECT crypto / DERIVED NSE)
├── research/       # absorption / acceptance / failed_auction (Stage 2+)
├── evaluation/     # reconstruction, baseline compare, friction (Stage 3)
└── reports/        # research_only outputs
```

**Executed-quantity directional flow (2026-11-07):**  
`executed_flow/` + `docs/research/rp608_ofx_executed_quantity_directional_flow.md`  
Crypto `aggTrade.isBuyerMaker` = DIRECT; NSE Stage 0D = DERIVED_VALIDATED (MEDIUM+). Not baseline L1 OF. No trading hooks.  
**Capture:** crypto durable `aggTrade` journal = **AUTHORIZED · IMPLEMENTED** (`docs/research/rp608_ofx_crypto_aggtrade_capture_authorization.md` · `ofx_aggtrade_capture_v1/` via shadow daemon).

Do not import or mutate `scripts/rp608_nse_of_paper_ledger.py`, crypto OF paper writers, Estimand A entry/exit, or shared OF±0.30 policy config from this tree.
