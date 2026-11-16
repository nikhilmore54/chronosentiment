# RP-608 NSE — Python Production Inventory (Migration Stage 1–3)

**Document ID:** `RP608-NSE-PY-INV-001`  
**Companion:** `RP608_NSE_PUSH_EVENT_RUST_ENGINE_v1.md`  
**Locked:** 2026-11-02  
**Purpose:** Identify → verify readiness for staged Rust port. Python remains behavioural reference until differential parity.

Authority **0%**. OF ±0.30 / reverse-OF / Estimand A **unchanged**.

---

## 1. Live NSE pipeline inventory

| Module | Path | Production class | Primary function | Tests (reference) | Migration readiness |
| --- | --- | --- | --- | --- | --- |
| Kite WS cache | `scripts/rp608_kite_ticker_cache.py` | **production_live** | Full-mode WS decode; LTP + L1–L5; latest-by-token cache | Partial (via daemon / book-intel / probe) | **First port candidate** — pure decode + event emit |
| NSE shadow daemon | `scripts/run_rp608_nse_shadow_daemon.py` | **production_live** | Orchestrator: bars, traj15, OF stamp, recs, paper hooks, REST fallback | `tests/test_rp608_nse_shadow_daemon.py` | Decompose; not a single-shot port |
| OF paper ledger | `scripts/rp608_nse_of_paper_ledger.py` | **production_live** | Paper open/close; reverse-OF persistence; journal writer | `tests/test_rp608_nse_of_paper_ledger.py` | Verified behaviour; port after event engine |
| Equity costs / V1 accounting | `scripts/rp608_nse_equity_costs.py` | **production_live** | Legs API; statutory schedule; direction-aware | `tests/test_rp608_nse_equity_costs.py` (+ production recon) | **Verified** — strong parity candidate |
| Book depth history | `scripts/rp608_nse_of_book_depth_history.py` | **production_observational** | Durable L1–L5 jsonl (telemetry only) | `tests/test_rp608_nse_of_book_depth_history.py` | Port with market-event path |
| Next-session recs | `scripts/rp608_nse_next_session_recs.py` | **production_batch** | Watchlist / next-session candidates | `tests/test_rp608_nse_next_session_recs.py` | After live event engine; then push-driven live watchlist |
| Session emit | `scripts/emit_nse_session_recommendations.py` | **production_batch** | Emit session recommendation artifacts | limited | Specify then port or fold into publisher |
| Market context | `scripts/nse_market_context.py` | **shared_lib** | NSE session boundaries / clocks | used transitively | Early shared Rust util |
| Live dashboard | `scripts/rp608_live_dashboard.py` | **production_live** | HTTP UI + APIs | mixed | Consumer of Rust event publisher (retain thin UI or replace later) |
| OF mark helper | (in daemon + `tests/test_rp608_order_flow_mark.py`) | **production_live** | L1 OF ±0.30 stamp | `tests/test_rp608_order_flow_mark.py` | **Verified** — port with depth events |
| Live probe | `scripts/run_rp608_nse_live_probe.py` | tooling | WS protocol reference | ad hoc | Spec fixture source for differential tests |

### Research / non-production (do not migrate as live writers)

| Path | Role |
| --- | --- |
| `scripts/research/rp608_nse_of_paper_*` | Friction audit, V1 recalc, production recon |
| `scripts/research/rp608_nse_of_book_intel_*` | Book-intel Phase 0/2 harnesses |
| `scripts/rp611_phase1_nse_observer.py` | Separate RP-611 observer |
| Historical replay / audit scripts | Validation only |

---

## 2. Verified / suitable for migration (near-term)

| Priority | Module | Why |
| --- | --- | --- |
| P0 ✅ | Kite full-mode packet decode + event normalisation | `nse_market_events::decode` + Python ref differential PASS |
| P0 ✅ | Market-event ordering / dedupe / stale marking | `MarketEventEngine` + `FeedHealth`; no cutover yet |
| P1 ✅ | L1 OF ±0.30 mark (observation; reverse-OF lifecycle still Python) | `of_mark.rs` differential PASS; threshold frozen |
| P1 ✅ | Depth↔event↔OF linkage | `LinkedObservation`; file writer to Pi artifact still deferred |
| P2 | `traj15` + bar-complete evaluator | Needs frozen bar-boundary spec |
| P2 | Watchlist live evaluation | Depends on event engine |
| P2 | Recommendation status / OF re-eval | Depends on event engine |
| P3 | OF paper ledger lifecycle | Verified; single writer must remain |
| P3 | Equity costs legs API | Verified + production recon PASS on TATASTEEL |

---

## 3. Known non-compliance to eliminate (not preserve)

These are **debts**, not features to re-implement in Rust:

- Mid-bar **polling** of OF paper state (`_of_paper_midbar_poll`)
- REST quote polling as primary freshness path when WS is healthy
- Dashboard treating stale OF as current without an explicit stale event
- Dual writers for the same ledger/state file

REST may remain a **reconcile-on-reconnect** tool only, never the steady-state update loop.

---

## 4. Differential-test fixture requirement (Stage 5)

Before any production cutover of the market-event path:

1. Record a closed set of Kite full-mode packets + bar-boundary markers (Pi session).  
2. Replay through Python reference → golden outputs (OF marks, traj15, stale flags, order).  
3. Replay through Rust → bit/field parity within documented rounding.  
4. Fail closed on order or timestamp divergence.

---

## 5. Authoritative writers (coexistence)

| Artifact | Current writer | Future writer |
| --- | --- | --- |
| `of_trade_outcomes.jsonl` | Python `NseOfPaperLedger` | Single Rust writer after parity |
| Depth `depth_snapshots.jsonl` | Python book-intel hook | Rust event engine (observational) |
| Daemon evaluation state | Python daemon | Rust engines |
| Dashboard projection | Python dashboard reads files/APIs | Rust publisher → dashboard consumer |

Until cutover, **do not** dual-write the same ledger from Python and Rust.
