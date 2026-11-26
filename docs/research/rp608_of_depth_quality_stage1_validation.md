# RP-608-OF-DEPTH-QUALITY-001 — Stage 1 Validation

**Disposition:** LIVE · OBSERVATIONAL ONLY (Pi overlay `1.2.3`)  
**Frozen:** OF entry ±0.30 · reverse-flow exit · 0% live authority  
**Date:** 2026-11-02

## Locked baseline

| Component | State |
| --- | --- |
| Protocol | `RP-608-OF-DEPTH-QUALITY-001` |
| Primary metric | `combined_touch_notional_usdt` |
| Diagnostics | same-side / opposing-side notional, `thin_opposite` |
| P&L UI | uGross vs uNet |
| Min-depth gate | **Not active** (reverted) |

## Stage 1 — telemetry completeness (Pi journals)

Snapshot of `trade_plan_overlay_v1` at validation time:

| Journal | Rows | Rows with full depth fields |
| --- | ---: | ---: |
| `of_paper_trade_opens.jsonl` | 6 | **0** (all opens pre-date entry `depth_quality` stamp, or opened before process reload) |
| `of_trade_outcomes.jsonl` | 4 | **0** with both entry+exit depth; **1** newest close has `depth_quality_exit` only |

Interpretation:

1. Historical ledger is **pre-telemetry** for entry depth — expected; do not invent backfill for research purity.
2. Newest close (OF-PAPER-PLAN-018482…) shows **exit** depth present → close-path telemetry is live.
3. Entry-depth on open journal must be confirmed on the **next** paper open after daemon reload (unit path verified: activate + opens jsonl stamp `depth_quality`).
4. Stage 2 bucket analysis must either (a) wait for N≥30 post-1.2.3 paired entry/exit depth rows, or (b) use exit-only / reconstructed L1 from stored bid/ask qty where available — prefer (a).

## Analytical safeguards (for Stage 2)

- Compare buckets on **net** P&L after costs; report **gross** separately.
- Publish **n per bucket** and avoid threshold inference from small n.
- Stratify LONG vs SHORT when sample allows.

## Next

- Stage 2: depth-bucket vs outcomes once Stage 1 completeness on **new** opens is green.
- Stage 3–4: stability, then separately versioned experimental gate **only if** justified.
