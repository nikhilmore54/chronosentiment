# RP-608 — Crypto Prospective Capture Specification (P0)

**Protocol:** `RP-608-CRYPTO-PROSPECTIVE-CAPTURE-001`  
**Date:** 2026-11-06  
**Authority:** 0.0% · Gate 2 CLOSED  
**Books:** Crypto plan and Crypto OF paper — **separate** (never pooled; never pooled with NSE)  
**Status:** REGISTERED specification · **does not** authorize engine or execution changes  

## Purpose

Define the event chain and integrity rules required before crypto **H4** (adaptive execution velocity) can become observationally testable.

This is **not** the NSE instrumentation charter. NSE remains on `rp608_wr_adaptive_execution_velocity_001_instrumentation_charter.md`.

## Inventory boundary (closed)

Crypto journal inventory established: neither crypto plan nor crypto OF has submit→ack→fill. Crypto OF has touch bid/ask/mid (paper). Do not reopen inventory as analysis.

## Required event chain (per book, per approved signal)

```text
Signal approved → Decision-time quote snapshot → Order submitted
  → Order acknowledged → First fill → Fill completion
  → Execution-quality record (fill vs decision reference)
```

| Stage | Required evidence |
| --- | --- |
| Signal | `signal_id` / `plan_id` or `of_trade_id`, decision timestamp UTC ms |
| Reference | Decision-time bid, ask, mid, quote source, quote timestamp, quote age |
| Submit | `order_id`, submit timestamp, order type, qty, limit if any |
| Acknowledgement | Ack timestamp, status |
| First fill | Timestamp, price, qty |
| Completion | Timestamp, total filled qty, terminal status (FILLED/PARTIAL/CANCELLED/REJECTED/UNFILLED) |
| Integrity | Missing/duplicate events, clock domain, reconciliation to signal id |

### Hard rules

1. **Do not** compute latency until timestamps pass ordering and clock checks.  
2. **Do not** compute realized slippage until actual fill can be compared to a verified **decision-time** reference.  
3. Modeled friction / `slippage_model: none` / `slippage_usdt=0` remain labeled as such — never “observed.”  
4. Incomplete chains → `MISSING_EVENT_CHAIN` bucket; not a selected “good execution” sample.

## Completeness floors (defaults to open crypto H4/E2-class diagnostics)

| Check | Floor |
| --- | ---: |
| Full chain coverage among approved signals | ≥ 80% |
| Decision bid/ask/mid + freshness on full-chain rows | ≥ 80% |
| Ordering violations | ≤ 2% |
| Orphan fills | ≤ 1% |

## P1 dependency — paper-engine capability (not this doc’s authorization)

Before implementing capture:

1. Audit whether the **existing** crypto paper engine can **emit** these events without changing trading behavior (logging-only).  
2. If **no**: register a separate instrumentation-change request for governance — this spec alone is insufficient authorization.  
3. If paper engine is instant-touch by design and cannot produce meaningful submit/ack/fill spans: mark H4 for that book **permanently unavailable** under current architecture; touch-based analysis may continue as **descriptive only**, not velocity.

## Book-specific notes

| Book | Capture note |
| --- | --- |
| Crypto plan | Primary B1 gap; outcomes today are simulated authority `0.0%` with single `entry_time_ms` |
| Crypto OF paper | Instant paper + touch refs; capability audit must decide if lifecycle spans are meaningful |

## After capture exists

1. Integrity validation report  
2. Latency / execution-quality distributions (descriptive)  
3. Observational state×execution diagnostics only if floors met  
4. Separate governance before any urgency experiment  

## Non-goals

- Changing urgency, filters, OF, stops, targets, or live authority  
- Using capture alone to construct an adaptive-velocity policy  
- Treating NSE charter as crypto authorization (or vice versa)  
