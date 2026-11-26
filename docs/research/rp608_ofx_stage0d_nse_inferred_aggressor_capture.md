# RP-608-OFX — Stage 0D NSE Inferred-Aggressor Tick Capture

**Protocol:** `RP-608-OFX`  
**Phase:** `STAGE_0D_NSE_INFERRED_AGGRESSOR_TICK_CAPTURE`  
**Disposition:** **IMPLEMENTED · QUALITY GATE PASS (initial live sample 2026-11-04)**  
**Date authorized:** 2026-11-04  
**Authority:** 0.0%  
**Quality artifact:** `artifacts/rp608_ofx/stage0d_quality_report.json`  


**Parent:** [`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md)  
**Prior:** Stage 0C [`rp608_ofx_stage0c_kite_fullmode_live_tick_feasibility.md`](rp608_ofx_stage0c_kite_fullmode_live_tick_feasibility.md)

---

## Purpose

Durable, timestamped **inferred-aggressor** market events from the existing Kite full-mode WebSocket — **single connection tee**, no competing socket, no OF/execution coupling.

This is **not** a true exchange trade tape. Kite does not supply aggressor side.

---

## Governance

| Item | State |
| --- | --- |
| Four-book baseline | Untouched |
| OF ±0.30 / reverse-OF | Frozen — journal is OF-rule inert |
| Estimand A | Untouched |
| Live order authority | 0% |
| Stage 1 observer | **Blocked until quality gate PASS** |
| Crypto | Separate (`aggTrade`) — not in this stage |

Enable/disable: env `RP608_OFX_NSE_TICK_CAPTURE` (`1`/`0`; default **enabled**).

---

## Implementation

| Component | Role |
| --- | --- |
| `scripts/rp608_kite_ticker_cache.py` | Decode LTQ / LTT / ATP (additive); optional `tick_sinks` |
| `scripts/rp608_ofx_nse_tick_journal.py` | Research journal + inference helpers |
| `scripts/run_rp608_nse_shadow_daemon.py` | Registers OFX sink before `start_ticker` |
| `scripts/research/rp608_ofx_stage0d_quality_report.py` | Coverage / sync / ambiguous-share gate |

**Pi path:** `live_capture/shadow_validation/nse/ofx_tick_capture_v1/`

- `inferred_aggressor_ticks.jsonl` — append-only events  
- `capture_coverage.json` — live counters  

### Per-row fields (summary)

instrument_token, symbol, LTP, LTQ, LTT, volume, exchange/receipt timestamps, quote_age, L1–L5 depth, `update_class` / `trade_like`, same-packet + prior-quote aggressor inference, primary inference with confidence, `connection_session_id`.

### Inference rules (provisional)

- Prefer **prior-packet quote** when trade-like; else same-packet (LOW confidence).  
- LTP ≥ ask → buy candidate; LTP ≤ bid → sell candidate; inside → ambiguous; missing/crossed/stale → unavailable.

---

## Quality gate (Stage 0D)

Run:

```bash
python3 scripts/research/rp608_ofx_stage0d_quality_report.py \
  --journal live_capture/shadow_validation/nse/ofx_tick_capture_v1/inferred_aggressor_ticks.jsonl \
  --out artifacts/rp608_ofx/stage0d_quality_report.json
```

Checks (narrow):

| Check | Criterion |
| --- | --- |
| Rows | ≥ 500 |
| Trade-like share | 5%–98% |
| Ambiguous share | < 40% |
| Exchange ts backwards | < 1% |
| Missing depth levels | < 20% |

**PASS** = durable stream usable for further research design.  
**PASS does not authorize Stage 1 mechanism classification** (absorption / acceptance / failed auction).

### Initial live sample (post-deploy, ~57s)

| Metric | Value |
| --- | ---: |
| Rows | 6,752 |
| Symbols | 119 |
| Trade-like share | 52.5% |
| Ambiguous share | 12.4% |
| Classified buy/sell share | 87.6% |
| Missing LTQ / depth / exch backwards | 0 |
| Quality gate | **PASS** |
| Stage 1 | **Not authorized by 0D alone** |

Capture continues on Pi under `ofx_tick_capture_v1/`.

---

## Dataset separation

| Dataset | Role |
| --- | --- |
| 0B candles | Historical price-response |
| 0A depth snapshots | Displayed liquidity (~5s) |
| **0D inferred ticks** | Prospective quote context + inferred aggression |
| Crypto aggTrade | Future / separate |

---

## Non-claims

- Not exchange aggressor confirmation  
- Not OF paper entry/exit input  
- Not proof of absorption  
- Not a fifth performance book  
