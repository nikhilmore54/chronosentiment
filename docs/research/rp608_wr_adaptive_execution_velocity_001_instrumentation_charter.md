# RP-608-WR-ADAPTIVE-EXECUTION-VELOCITY-001 — Instrumentation Charter

**Protocol:** `RP-608-WR-ADAPTIVE-EXECUTION-VELOCITY-001` / tranche `INSTRUMENTATION-CHARTER-001`  
**Date:** 2026-11-06  
**Authority:** 0.0% · baseline frozen · Gate 2 **CLOSED** · Gate 3 unauthorized  
**Book:** NSE Estimand A (primary) · NSE OF paper **separate** (never pooled)  
**Status:** **REGISTERED — DEPENDENCY FOR E2** · research-only logging · **no** urgency policy · **no** execution change  

## Disposition of prior work (locked)

| Step | Status |
| --- | --- |
| Continuous market-state audit (RANKING-001) | **Completed** — E1-class evidence; not re-run here |
| Pi journal inventory | **Completed** — Estimand A lifecycle **MISSING** · E2 **BLOCKED** |
| Adaptive-velocity hypothesis | **Open** — **not** disproven by missing data |
| Execution change | **Not authorized** |

Missing instrumentation is **not** a positive or negative result for adaptive velocity.

## Purpose

Define the **minimum prospective capture** required before scope estimand **E2** (state strata vs observed decision→fill latency / slippage) can become runnable.

This charter does **not** authorize:

- Adaptive execution-velocity policies  
- Changes to OF ±0.30, stops, targets, or horizon  
- Relabeling modeled friction as observed slippage  
- Constructing urgency rules from Oct 28 / Oct 30  

## Required event chain (Estimand A)

Each approved trade must emit linked events (same `trade_id` / `client_order_id` lineage):

| # | Event | Required fields |
| --- | --- | --- |
| 1 | Signal approval | `trade_id`, `symbol`, `side`, `signal_approved_ts_utc_ms`, decision-time feature snapshot id or inline continuous state |
| 2 | Order submission | `trade_id`, `order_id`, `submitted_ts_utc_ms`, order type, limit/price if any, qty |
| 3 | Broker / exchange acknowledgment | `order_id`, `ack_ts_utc_ms`, ack status |
| 4 | First fill | `order_id`, `first_fill_ts_utc_ms`, `first_fill_price`, `first_fill_qty` |
| 5 | Fill completion | `order_id`, `fill_complete_ts_utc_ms`, `filled_qty`, `avg_fill_price`, terminal status (`FILLED` / `PARTIAL` / `CANCELLED` / `REJECTED` / `UNFILLED`) |

Derived latencies (computed, not logged as primary truth):

- Decision→submit  
- Submit→ack  
- Ack→first fill  
- Decision→first fill  
- First fill→complete (partial-fill span)

## Required reference data (look-ahead-safe)

Captured **at or before** signal approval (decision-time), never backfilled from post-entry tape as if known at decision:

| Field | Rule |
| --- | --- |
| `decision_bid` / `decision_ask` / `decision_mid` | At decision; side-consistent touch documented |
| `quote_ts_utc_ms` | Timestamp of the quote used |
| `quote_age_ms` | `signal_approved_ts - quote_ts` (freshness) |
| Fill price / qty | From fill events only |
| Contemporaneous quote at fill | Optional **execution-quality** descriptor; **not** a decision-time feature |

**Modeled 2 bps/leg slippage** remains labeled `modeled_slippage_bps_per_leg`. It **must not** be renamed or mixed into `observed_slippage_*` fields.

## Required integrity checks

Before any E2 diagnostic:

1. **Trade-ID linkage** — every fill event joins to signal approval; orphan rate reported.  
2. **Missing events** — per-stage completeness %; incomplete chains **excluded** from latency estimands and counted in a `MISSING_EVENT_CHAIN` bucket (not silently dropped from the denominator story).  
3. **Duplicates** — duplicate submits/acks/fills flagged.  
4. **Timestamp ordering** — `approved ≤ submitted ≤ ack ≤ first_fill ≤ complete` (equal allowed only where instantaneous and documented); violations → `CLOCK_OR_ORDERING_VIOLATION`.  
5. **Partial / unfilled** — retained as first-class outcomes; unfilled approved signals remain in the approved population for fillability rates.  
6. **Book separation** — NSE Estimand A and NSE OF paper populations never pooled; OF capture (if any) uses distinct `book_id`.  
7. **Clock semantics** — all event times in UTC ms; single documented clock domain (exchange vs host) with offset note if dual clocks exist.

Incomplete observations **cannot** become a selected “successful execution” sample.

## Completeness gate for opening E2

E2 may be marked **runnable** only when a prospective window satisfies predeclared floors (defaults):

| Check | Default floor |
| --- | ---: |
| Approved trades with full event chain (1–5) | ≥ 80% |
| Decision bid/ask/mid + quote freshness present | ≥ 80% of full-chain trades |
| Ordering violation rate | ≤ 2% |
| Orphan fill rate | ≤ 1% |

Floors may be tightened in a later note; they may not be loosened silently to force E2 open.

## After instrumentation exists (ordered)

1. Validate event chain + timestamp quality (integrity report).  
2. Publish observed latency and execution-quality distributions (descriptive).  
3. Run **observational E2** only (state strata vs latency / observed fill quality).  
4. Compare execution outcomes across decision-time states — still no policy.  
5. Separate governance decision before any Mechanism A experiment (E3) or Mechanism B work (E4).

Instrumentation alone does **not** construct an adaptive-velocity policy.

## Explicit non-goals

- Rerunning E1 / continuous-ranking under an execution-velocity label  
- Inferring latency from bar open/close or single `entry_timestamp`  
- Using Oct 28 / Oct 30 to fit urgency schedules  
- Live order authority > 0%  

## Artifacts

- This charter: `docs/research/rp608_wr_adaptive_execution_velocity_001_instrumentation_charter.md`  
- Inventory (completed): `artifacts/rp608_econ_edge_diag/rp608_wr_adaptive_execution_velocity_001_journal_inventory.md`  
- Scope: `docs/research/rp608_wr_adaptive_execution_velocity_001_scoped.md`  
- Gate 2: `docs/research/rp608_wr_adaptive_execution_velocity_001_gate2.md`  

## Governance

Baseline frozen · OF ±0.30 frozen · stops/targets/horizon unchanged · Gate 2 CLOSED · live authority 0% · books not pooled · OFX boundary unchanged.
