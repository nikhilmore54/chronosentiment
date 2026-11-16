# RP-608-OFX — Stage 1 Result-Quality Audit

**Protocol:** `RP-608-OFX`  
**Phase:** `STAGE_1_RESULT_QUALITY_AUDIT`  
**Disposition:** **EXECUTED**  
**Date:** 2026-11-04  
**Authority:** 0.0%  
**Inspection class:** Agent audit of reported Stage 1 artifacts — **not** independent third-party verification  

**Preserved Stage 1 conclusion:** `DOMINANTLY_FLAT_SHORT_HORIZON`  
**Stage 2:** **Not authorized**

**Artifacts:**  
- Input: `artifacts/rp608_ofx/stage1_aggression_response_events.jsonl`  
- Output: `artifacts/rp608_ofx/stage1_result_quality_audit.json`  
- Runner: `scripts/research/rp608_ofx_stage1_result_quality_audit.py`

---

## Purpose

Clarify whether the Stage 1 flat result reflects weak price response, the ±2 bps measurement boundary, inferred-aggression limits, or sample-window artifacts — **before** any Stage 2 mechanism labeling.

---

## Diagnostic results

### 1. Denominator and deduplication

| Check | Result |
| --- | --- |
| Event count | **24,714** |
| Unique `(symbol, event_receipt_utc_ms)` | 24,714 |
| Duplicate keys | **0** |
| Near-dups ≤50ms same mid/label/ltq | **0** |

Denominator confirmed unique on the Stage 1 event key.

### 2. Inferred aggression assignment / ambiguous handling

| Item | Result |
| --- | --- |
| Labels in Stage 1 file | BUY 14,736 · SELL 9,978 |
| AMBIGUOUS / UNAVAILABLE in Stage 1 file | **0** (excluded by design) |
| Confidence | MEDIUM 24,607 · LOW 107 |

Ambiguous 0D ticks never enter the 24,714 denominator. Inference remains **candidate** (prior-quote preferred), not exchange aggressor.

### 3. Response magnitude (not only with/against/flat)

| Horizon | abs return p50 (bps) | Share \|r\| > 2 bps | Share \|r\| > 5 bps | Mean signed bps |
| --- | ---: | ---: | ---: | ---: |
| 1s | 0.00 | 6.0% | 1.0% | +0.16 |
| 5s | **0.51** | **16.5%** | 2.9% | +0.19 |
| 15s | 1.19 | 33.4% | 8.4% | +0.23 |
| 30s | 1.85 | 47.7% | 16.3% | +0.28 |

Median absolute mid move at 5s is **below** the 2 bps flat band. Directional % rising with horizon is largely the flat band releasing small moves — not evidence of strong predictive impulse.

### 4–5. Coverage and concentration

| Check | Result |
| --- | --- |
| Symbols | 119 · top10 share **20.3%** · HHI ≈ 0.011 |
| IST hour | All events in **2026-11-04 12** (short capture window) |
| Minute top10 share | ~1.0 by construction on a short window (<30 minutes) |

**Not** symbol-dominated. **Is** a short intraday window — time diversification is limited; do not over-generalize to full-session behavior.

### 6. Flat band (±2 bps)

Documented as an **observational classification boundary**, not a tuned trading threshold. Changing it would re-label the same returns; it does not create edge.

### 7. Baseline (sign-shuffle null)

At 5s: observed mean signed ≈ **+0.19 bps**; sign-shuffle null 95% interval roughly ±0.03 bps; empirical two-sided p ≈ **0** for \|mean\|.

So the signed mean is **statistically** distinguishable from pure random sign on the same magnitudes, but **economically tiny** (≪ 1 bps) and far below friction-relevant scale. Not a trading claim; not Stage 2 authorization.

---

## Synthesis

`DOMINANTLY_FLAT_SHORT_HORIZON` is preserved.

The flat result most likely reflects:

1. Median short-horizon mid moves below the 2 bps observational band  
2. Only a minority of events exceed that band at 5s  
3. Any mean signed alignment is economically negligible  
4. A short midday capture window (limits time diversity)  
5. Inferred aggression without location/structure (Stage 2 territory — still unauthorized)

It still does **not** establish that order flow has no value, absorption/failed-auction behavior, OF mechanism change, or Stage 2 readiness.

---

## Governance

Four-book baseline frozen · no OF/execution changes · authority 0% · Step 3+ held · Gate R unreached · Stage 2 not authorized.
