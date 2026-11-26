# RP-608 — Crypto Paper-Engine Capability Audit (P1)

**Protocol:** `RP-608-CRYPTO-PAPER-ENGINE-CAPABILITY-001`  
**Date:** 2026-11-06  
**Depends on:** P0 `rp608_crypto_prospective_capture_spec.md`  
**Code inspected:** `scripts/rp608_trade_plan_overlay.py`  
**Authority:** 0% · Gate 2 CLOSED · **no** code change in this audit  

## Question

Can the **existing** crypto paper engine emit submit→ack→first-fill→complete (+ decision-time quotes) **without modifying trading behavior**?

## Findings

### Crypto OF paper

| Capability | Verdict | Evidence |
| --- | --- | --- |
| Instant touch fill | **By design** | `activated_ms` and `entry_time_ms` set to the same `fill_ms` at open (`_of_paper_open` ~L1331–1360) |
| Entry at bid/ask | **Yes** | `_of_entry_exec_price` → LONG ask / SHORT bid; journals show touch fills |
| Decision-time bid/ask logged | **Yes** | `entry_bid` / `entry_ask` / depth mid on open |
| Distinct submit / ack / first fill / complete | **No** | Single open event `OF_PAPER_OPEN`; no order_id lifecycle |
| Meaningful latency spans | **No under current semantics** | Zero intentional delay between signal and paper fill |

**Logging-only without behavior change:** Could append synthetic equal-timestamp “submit/ack/fill” rows for schema compliance. That would **not** create measurable urgency variation → **does not resolve H4**.

**Meaningful lifecycle:** Would require simulating delay, queueing, or partial fills → **changes paper execution semantics** → separate instrumentation authorization (not granted by P0/P1).

### Crypto plan (simulated overlay)

| Capability | Verdict | Evidence |
| --- | --- | --- |
| Entry timestamp | Quote/`close_time` derived (`entry_time_ms` from `exec_quote.exchange_event_time_ms`) | ~L544–607 |
| Friction | **Modeled** (e.g. 6+6 bps), not fill-observed | ~L531–541 |
| Order lifecycle | **No** | `PLAN_CREATED` / TP/SL/TIME_EXIT events only |
| Authority | `0.0% (SIMULATED)` | outcomes |

Same conclusion: schema-only logging ≠ velocity evidence; delayed fills need authorized sim changes.

## P1 disposition

| Book | Emit lifecycle **logging-only** (no behavior change)? | Meaningful H4 evidence without behavior change? | Status for B1 |
| --- | --- | --- | --- |
| Crypto OF paper | Schema stubs possible; **not useful** | **No** | **Permanently unavailable** under instant-touch architecture unless governance authorizes a latency-capable paper model |
| Crypto plan | Schema stubs possible; **not useful** | **No** | Same — needs authorized sim/order-path instrumentation |

Touch-based OF entry analysis remains **descriptive only** (B2 partial), not adaptive velocity.

## Exit criteria mapping

| Exit | Met? |
| --- | --- |
| Confirm emit-without-changing-execution for **useful** latency | **No** |
| Mark permanently unavailable vs needs governance change | **Permanently unavailable for H4** under current engine; reopen only with separate authorized instrumentation |

## Non-actions

- No overlay code changes from this audit  
- No urgency policy  
- No claiming NSE charter covers crypto  

## Next for H4 path

Only if governance later authorizes a **latency-capable** paper/sim path: implement P0 event chain with non-degenerate timestamps, then integrity floors, then observational E2. Until then H4 stays parked; progress via H1–H3 observational track.
