# RP-608 NSE — Push-Driven Event Architecture & Rust Migration Requirement

**Document ID:** `RP608-NSE-PUSH-RUST-001`  
**Status:** **FORMAL ENGINEERING REQUIREMENT** (not optional optimisation)  
**Locked:** 2026-11-02  
**Authority:** 0.0% live order authority — unchanged  
**Aligns with:** chronosentiment-core (deterministic event-driven state machine); `docs/research/RUST_PORT_CONTRACT_v1.md` (Python = research/validation; Rust = production canonical)

---

## 1. Decision summary

| Decision | Mandate |
| --- | --- |
| Update model | **Push-driven / event-based only** for watchlist and recommended-trade intelligence |
| Shared infrastructure | **One** market-event stream serves both watchlist and recommendations |
| Production runtime target | **Rust-native** NSE intelligence engine |
| Python role | Research, experimentation, historical analysis, **behavioural reference** until Rust parity |
| Migration style | Verified modules; coexistence allowed; **one authoritative writer** per state/ledger |
| Trading rules | **No change** — OF ±0.30, reverse-OF, Estimand A, session rules, accounting V1 legs API |

---

## 2. Target architecture (push)

```text
Kite WebSocket
  LTP · L1–L5 depth · volume
        │
        ▼
Rust Market Event Engine
  ordered · timestamped · deduplicated · provenance-tagged
        │
        ├──► Rust Watchlist Engine          (≤20 candidates)
        ├──► Rust Recommendation Engine     (active recs + OF / reverse-OF)
        ├──► Rust Paper-Trade Lifecycle     (observations only; 0% authority)
        └──► Rust State & Event Publisher
                    │
                    ▼
              Live Dashboard (event-driven UI updates)
```

No separate polling loops for L1 OF, price response, recommendation status, reverse-OF, or dashboard freshness.

---

## 3. Event → behaviour contract

| Event | Trigger | Downstream effect |
| --- | --- | --- |
| Depth update | Incoming WS depth | Refresh L1 OF support for subscribed symbols; reverse-OF eval for open/recommended |
| Price / LTP update | Incoming WS trade/quote | Refresh live price response |
| Bar complete | Completed 15m bar boundary | Recalculate `traj15` (session-causal) |
| Stale / disconnect | Missed events, WS drop, age gate | Explicit **stale** status; reconcile before restoring live |
| Recommendation / strategy | Relevant market or strategy event | Re-evaluate OF, status, paper lifecycle observations |
| State change | Any material state delta | Publish to dashboard immediately |

### Watchlist (every subscribed symbol)

- Depth → L1 OF support refresh.
- Price → live price response refresh.
- Completed 15m bar → `traj15` recalculation.
- Propagate changes to dashboard immediately.
- Stale feeds → explicit status event (never silent).

### Recommended trades

- Same push path as watchlist.
- On market event: re-evaluate OF support; update recommendation status; detect reverse-OF; update paper-trade lifecycle observations; push to dashboard.
- **Invariant:** a recommendation must never display an old OF status as current.

### Provenance (hard requirement)

Every event and derived state must retain:

- Event timestamp (exchange and/or ingest, labelled)
- Sequence / order key
- Source provenance (`kite_ws_full`, reconcile, etc.)

On WebSocket disconnect or gap: mark affected symbols **stale**, reconcile on reconnect, only then restore live status.

---

## 4. Governance freeze (unchanged)

| Item | Status |
| --- | --- |
| OF ±0.30 | Frozen |
| Reverse-OF exit logic | Frozen |
| Paper ledger schema / writer authority | Unchanged except receiving push-driven observations |
| Accounting | `NSE_OF_TOUCH_PLUS_STATUTORY_V1` legs API; historical V1 bulk totals **PROVISIONAL** |
| Estimand A | Unchanged |
| Live order authority | **0%** |

---

## 5. Current production gap (honest baseline)

As of lock date, the live NSE path is **Python-orchestrated** and still contains poll-style loops (e.g. mid-bar OF paper poll, REST quote fallback, bar-recovery polling) alongside a Kite WebSocket L1–L5 cache.

| Component | Role today | Compliance vs this requirement |
| --- | --- | --- |
| `rp608_kite_ticker_cache.py` | WS full-mode L1–L5 cache | Partial — push ingest exists; consumers still polled |
| `run_rp608_nse_shadow_daemon.py` | Orchestrator, traj15, recs, OF stamp | Non-compliant long-term — timer/poll driven evaluation |
| `rp608_nse_of_paper_ledger.py` | Paper lifecycle | Receives polled observations |
| `rp608_live_dashboard.py` | Dashboard | Mix of pull APIs + state files |
| `rp608_nse_next_session_recs.py` | Next-session watchlist batch | Batch/file oriented — must move onto shared event stream for live session |

This document **forbids accumulating** further permanent Python production dependencies and requires migration per §6–§7.

---

## 6. Python → Rust migration stages

| Stage | Requirement |
| --- | --- |
| 1. Identify | Inventory Python scripts on the live NSE pipeline |
| 2. Verify | Correct behaviour + passing tests (Python = reference) |
| 3. Specify | Freeze inputs, outputs, state transitions, edge cases |
| 4. Port | Equivalent Rust module |
| 5. Differential test | Identical recorded market-event fixtures |
| 6. Validate | Outputs, ordering, timestamps, numerical parity |
| 7. Deploy | Production switch only after parity |
| 8. Retire | Remove superseded Python production dependency |

### Behavioural parity — must not change in Rust

- OF ±0.30 and LONG/SHORT handling  
- Reverse-OF persistence and exit rules  
- Paper-trade accounting (legs API / statutory schedule)  
- NSE session boundaries  
- `traj15` calculation (session-causal)  
- Stale-feed handling  
- Event ordering and deduplication  

**Authoritative writer rule:** during coexistence, exactly one production writer per state artifact and ledger.

---

## 7. Immediate engineering sequence

1. **Done:** freeze push-based architecture specification.  
2. **Done:** inventory live Python production scripts (`RP608_NSE_PYTHON_PRODUCTION_INVENTORY_v1.md`).  
3. **Done:** identify verified / migration-ready modules (inventory).  
4. **Done (P0 library + differential):** Rust market-event + depth path  
   - Crate module: `adapters/chronosentiment/src/nse_market_events/`  
   - Python reference: `scripts/rp608_nse_market_event_ref.py`  
   - Fixtures: `adapters/chronosentiment/tests/fixtures/nse_kite_full/`  
   - Gate: `cargo test --test rp608_nse_market_events_p0_tests` + `tests/test_rp608_nse_market_event_ref.py`  
   - **No production cutover** until explicit deploy decision after parity.  
5. **Done (P1 library + differential):** OF ±0.30 marking + depth/event linkage + stale OF suppression  
   - `nse_market_events::of_mark` + engine `OfMarkRecord` / `LinkedObservation` / `current_of_mark`  
   - Python reference: `scripts/rp608_nse_of_mark_event_ref.py` (wraps frozen `rp608_order_flow_mark.py`)  
   - Gate: `cargo test --test rp608_nse_of_mark_p1_tests` + `tests/test_rp608_nse_of_mark_event_ref.py`  
   - **No production cutover**; threshold **0.30 frozen**  
6. **Done (P2 library + differential):** push-driven watchlist (≤20) + recommendation evaluation on the P0/P1 event path  
   - Changes **when** evaluation runs (event-driven), not **what** qualifies (rules frozen)  
   - Module: `nse_market_events::push_eval`  
   - Python reference: `scripts/rp608_nse_push_eval_ref.py`  
   - Gate: `cargo test --test rp608_nse_push_eval_p2_tests` + `tests/test_rp608_nse_push_eval_ref.py`  
   - **No production cutover**; authority 0%  
7. **CLOSED (P3):** push-driven paper-lifecycle observations — formal hold at this boundary  
   - Module: `nse_market_events::paper_obs` (mirror open + reverse-OF/persist; **no ledger write**)  
   - Python reference: `scripts/rp608_nse_paper_obs_ref.py`  
   - Gate: Rust 3/3 · Python 3/3  
   - Authoritative writer: **Python** · dual-write: **forbidden** · cutover: **not authorized**  
   - **No further development milestone implied.** Next step, if any: separate **cutover-readiness governance review** (lifecycle + ops evidence), not an automatic P4.

No trading-rule or authority changes are implied by any stage.

### P0 acceptance (2026-11-02)

| Check | Status |
| --- | --- |
| Full-packet decode LTP/volume/L1–L5 vs Python | PASS |
| Ordering + dedupe + content fingerprint | PASS |
| Durable depth snapshots (history, not latest-only) | PASS |
| Disconnect → reconcile → live; age-stale | PASS |
| Production writer switch | **NOT DONE** (intentionally) |

### P1 acceptance (2026-11-02)

| Check | Status |
| --- | --- |
| OF marks match Python (±0.30, boundary cases) | PASS |
| L1–L5 depth linked to source event (`ingest_seq` / `content_fp`) | PASS |
| Disconnect / stale / reconcile suppress current OF | PASS |
| Dual-writer introduced | **No** |
| Production cutover | **NOT DONE** |

### P2 acceptance (2026-11-02)

| Check | Status |
| --- | --- |
| Watchlist ≤20 rank vs Python (score, OF badge) | PASS |
| Recommendation decisions vs Python (frozen tree) | PASS |
| Stale/reconcile suppress current recommendation | PASS |
| Dual-writer / production cutover | **No** |

### P3 acceptance (2026-11-02) — **FORMALLY CLOSED**

| Check | Status |
| --- | --- |
| Reverse-OF + min_hold 300s + persist 30s vs Python | PASS |
| Event identity on each observation | PASS |
| Stale/reconciling → no current observation | PASS |
| Ledger writer remains Python | PASS |
| Dual-write / cutover / authority | **No / No / 0%** |

### Hold boundary (post P0–P3)

| Item | Mandate |
| --- | --- |
| P0–P3 | **Closed** — shared Rust push path established; Python behavioural reference retained |
| Production writer cutover | **Not authorized** — requires separate governance review |
| Live authority | **0%** |
| Default next engineering milestone | **None** — do not infer P4 from P3 completion |

---

## 8. Target end-state

A **Rust-native, push-driven NSE intelligence engine**. Python remains the research and validation environment and the behavioural reference until each module clears differential validation.
