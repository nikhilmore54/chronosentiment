# RP-608-OFX — Stage 1 Aggression → Price-Response Observer

**Protocol:** `RP-608-OFX`  
**Phase:** `STAGE_1_AGGRESSION_RESPONSE_OBSERVER`  
**Disposition:** **AUTHORIZED · IMPLEMENTED · INITIAL RUN COMPLETE**  
**Date:** 2026-11-04  
**Authority:** 0.0%  

**Parent:** [`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md)  
**Input:** Stage 0D `inferred_aggressor_ticks.jsonl`  
**Artifacts:**  
- `artifacts/rp608_ofx/stage1_aggression_response_events.jsonl`  
- `artifacts/rp608_ofx/stage1_aggression_response_summary.json`

---

## Scope (authorized)

Independent observational engine that, for each **trade-like** inferred buy/sell aggression event, records:

- aggressor label + confidence (from 0D)
- LTQ / update class
- same-side persistence (5s)
- L1 displayed liquidity delta (5s)
- mid-price response at 1s / 5s / 15s / 30s
- descriptive response class: `ADVANCES_WITH` | `ADVANCES_AGAINST` | `FLAT` | `INSUFFICIENT_FORWARD`

**Initial question:** When aggressive buying or selling occurs (inferred), does the subsequent price response contain stable, incremental information?

## Explicitly not in Stage 1

| Forbidden | Status |
| --- | --- |
| Absorption / acceptance / failed-auction primary labels | Stage **2** only |
| Recommendations / orders | Forbidden |
| OF ±0.30 / reverse-OF / Estimand A changes | Frozen |
| Pooling into baseline books | Forbidden |
| Claiming exchange aggressor truth | Forbidden — inferred only |

---

## Implementation

| Path | Role |
| --- | --- |
| `research/rp608_ofx/observer/aggression_response.py` | Helpers |
| `scripts/research/rp608_ofx_stage1_aggression_response_observer.py` | Offline runner over 0D journal |

```bash
python3 scripts/research/rp608_ofx_stage1_aggression_response_observer.py \
  --journal live_capture/shadow_validation/nse/ofx_tick_capture_v1/inferred_aggressor_ticks.jsonl \
  --out-events artifacts/rp608_ofx/stage1_aggression_response_events.jsonl \
  --out-summary artifacts/rp608_ofx/stage1_aggression_response_summary.json
```

Flat band default: **±2 bps** mid move.

---

## Initial run (2026-11-04, Pi journal)

| Metric | Value |
| --- | ---: |
| Ticks loaded | ~49,957 |
| Events (trade-like buy/sell) | **24,714** |
| Buy / sell candidates | 14,736 / 9,978 |

| Horizon | With | Against | Flat | Mean signed bps | Aligned share |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1s | 4.3% | 1.7% | **94.0%** | +0.16 | 22.4% |
| 5s | 9.9% | 6.7% | **83.5%** | +0.19 | 37.3% |
| 15s | 18.7% | 14.7% | **66.6%** | +0.23 | 45.3% |
| 30s | 26.4% | 21.4% | **52.3%** | +0.28 | 48.1% |

**Stability read:** `DOMINANTLY_FLAT_SHORT_HORIZON` (after correcting the flat-aware heuristic).

### Interpretation (narrow)

At ±2 bps, most inferred aggression events do **not** produce a meaningful immediate mid move. Alignment with aggression rises toward ~0.5 only as the flat share falls at longer horizons — consistent with noise / random walk, **not** evidence of a stable incremental edge from inferred aggression alone.

This does **not** falsify absorption/acceptance setups (Stage 2), which require location + multi-observation structure. It does caution against treating every inferred imbalance print as directional.

---

## Follow-on

Stage 1 result-quality audit (executed): [`rp608_ofx_stage1_result_quality_audit.md`](rp608_ofx_stage1_result_quality_audit.md). Preserves `DOMINANTLY_FLAT_SHORT_HORIZON`; does not authorize Stage 2.

## Governance

Rules frozen · books separate · authority 0% · Step 3+ held · Gate R unreached · Stage 2 not auto-authorized.
