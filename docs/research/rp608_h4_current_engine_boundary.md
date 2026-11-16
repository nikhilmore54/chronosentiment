# RP-608 H4 — Current-Engine Boundary (Crypto)

**Protocol:** `RP-608-H4-CURRENT-ENGINE-BOUNDARY-001`  
**Date:** 2026-11-07  
**Status:** **CLOSED for current engines**  
**Disposition:** `UNTESTABLE_UNDER_CURRENT_ENGINE_DESIGN`  

**Authority:** 0% · Gate 2 CLOSED · no execution change  

## Precise disposition

| Question | Disposition |
| --- | --- |
| Can the **current** crypto engines establish actual execution velocity? | **No** |
| Can touch-based outcomes describe entry behavior? | **Yes, descriptively** (not velocity) |
| Can we infer actual latency from simulated / instant timestamps? | **No** |
| Can H4 be reconsidered after a separately authorized engine redesign? | **Yes** — new instrumentation + governance required |

This supersedes indefinite “parked / open” wording for **current** crypto plan + crypto OF paper engines.

## Evidence already obtained

- Crypto journal inventory: no submit→ack→fill chain  
- P1 capability audit (`rp608_crypto_paper_engine_capability_audit.md`):  
  - Crypto OF paper: `activated_ms == entry_time_ms` by design (instant touch)  
  - Crypto plan: simulated timing / modeled friction; no order lifecycle  
- Logging stubs with equal timestamps would **not** create measurable urgency  

## What remains unauthorized

- Inferring latency from `activated_ms`  
- Relabeling `slippage_model: none` / modeled friction as observed  
- Re-inventorying crypto lifecycle fields without an authorized redesign  
- Any urgency / execution / live-authority change  

## Future reopen path (only)

1. Separately authorized engine/instrumentation redesign capable of non-degenerate lifecycle timestamps.  
2. Capture per `rp608_crypto_prospective_capture_spec.md` (or successor).  
3. Integrity floors met.  
4. Observational E2-class diagnostics.  
5. Governance review before any experiment.

NSE H4 / adaptive velocity remains on its **own** instrumentation charter — this doc does not authorize NSE changes either.

## Relation to H2

H2 session-regime results **must not** be interpreted as execution-velocity evidence.
