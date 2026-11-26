# RP-608-OFX — Stage 0 Data Feasibility

**Protocol:** `RP-608-OFX`  
**Phase:** `STAGE_0_DATA_FEASIBILITY`  
**Disposition:** **EXECUTED · VERDICT LOCKED**  
**Date:** 2026-11-04  
**Authority:** 0.0%  
**Inspection class:** Agent Pi journal + source inspection (ultraspeed@192.168.0.9) — **not** an independent third-party audit of the registration pack  

**Parent scope:** [`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md)  
**Tracking boundary:** [`rp608_ofx_performance_tracking_boundary.md`](rp608_ofx_performance_tracking_boundary.md)  
**Artifact summary:** `artifacts/rp608_ofx/stage0_data_feasibility.json`

---

## Governance (unchanged)

| Item | State |
| --- | --- |
| Four-book baseline (A / NSE OF / crypto plan / crypto OF) | **Frozen** performance reference |
| OF ±0.30 / reverse-OF / hold / persistence | **Untouched** |
| OFX orders / policy substitution | **None** |
| Stage 1 strategy / classification engine | **Not authorized** by this note alone |
| Step 3+ / Gate R | **Held / unreached** |

Stage 0 answers only: *what durable fields exist today for aggression, liquidity change, price response, and timestamps?*

---

## Capability matrix

| Capability | NSE | Crypto | Stage-0 verdict |
| --- | --- | --- | --- |
| Displayed bid/ask (L1) | Yes — cache + depth journal + OF stamps | Yes — `bookTicker` (WS/REST) | **AVAILABLE** |
| Displayed L2–L5 depth history | Yes — durable `depth_snapshots.jsonl` | No durable multi-level book journal in RP-608 path | **NSE AVAILABLE · Crypto ABSENT** |
| Timestamps (exchange / receipt / sample) | Yes on depth rows | Event timestamps on plan/OF ledgers; quote clocks on bookTicker path | **AVAILABLE (market-specific)** |
| Cumulative volume on quote path | Yes (`volume` on depth rows) | Not as a durable trade tape | **NSE PARTIAL proxy only** |
| Last traded quantity (LTQ) / last trade time | On wire (Kite full packet bytes 8–12 / 44–48); **not** decoded or journaled in production cache | N/A (different feed) | **NSE NOT PERSISTED** |
| Durable trade print tape (price, qty, time) | **Absent** | **Absent** (no `aggTrade` journal) | **NOT AVAILABLE** |
| Aggressor side (executed) | Must be inferred from trade vs bid/ask; **no trade tape → cannot measure coverage** | Binance `aggTrade.isBuyerMaker` exists on exchange API; **not captured** | **NOT FEASIBLE TODAY** |
| Liquidity change over time | Feasible from successive L1–L5 snapshots (~5s cadence) | L1 bookTicker changes only (no depth history) | **NSE FEASIBLE · Crypto L1-ONLY** |
| Price response after displayed-liquidity events | Partial — `last_price` colocated on snapshots; 5s sampling limits intrabar resolution | Partial — plan/OF use L1 marks, not aggression events | **PARTIAL (displayed-liquidity path only)** |
| Price response after **aggression** events | Blocked until trade+quote stream exists | Blocked until `aggTrade` (or equivalent) journal exists | **BLOCKED** |

---

## NSE evidence (Pi, 2026-11-04 inspection)

### Depth / liquidity (usable for OFX liquidity-response subset)

| Item | Value |
| --- | --- |
| Path | `live_capture/shadow_validation/nse/of_book_intelligence_v1/depth_snapshots.jsonl` |
| Size / rows | ~1.30 GiB · **1,428,052** lines (at inspection) |
| Span (sample_iso) | `2026-11-02T07:42:11Z` → `2026-11-04T05:19:07Z` |
| Schema | `depth_snapshot_l1_l5` with `depth_buy`/`depth_sell` length **5**, L1 scalars, `volume`, `sample_utc_ms`, `exchange_timestamp_*`, `receipt_utc_ms`, stale/missing flags |
| Sample check (tail ~1.5 MiB) | 119 symbols · 0 missing-level · 0 stale · L2+ qty present both sides |
| Cadence | `min_interval_ms=5000` (rate-limited snapshots, not every WS packet) |
| Coverage counter (process-local) | `depth_capture_coverage.json` — live, `enabled=true`, 119 symbols last sample |

**Feasible now (read-only):** displayed-liquidity evolution, replenishment/depletion proxies, and coarse price response tied to **book** events — aligning with, but not modifying, `RP-608-NSE-OF-BOOK-INTELLIGENCE-001`.

### Aggression (not feasible from current journals)

| Source | Finding |
| --- | --- |
| Durable trade / aggressor / market_event journals under `nse/` | **None found** |
| Production decoder `scripts/rp608_kite_ticker_cache.py` | Decodes LTP + **volume** + L1–L5; **skips** LTQ (`pkt[8:12]`) and LTT |
| Probe `scripts/run_rp608_nse_live_probe.py` | Unpacks LTQ at bytes 8–12 but **does not return** it in the decoded dict |
| Depth journal fields | `last_traded_quantity` **absent** on first/mid/last sampled rows |
| OHLCV / 15m bars | Unsigned volume only — **forbidden** as aggressor substitute (same rule as intrabar protocol) |
| Related gate | `RP-608-NSE-OF-INTRABAR-ABSORPTION-001` Phase 0 data-gate still **NOT EXECUTED** — OFX Stage 0 **confirms** the same structural gap |

**Implication:** OFX cannot yet answer “when aggressive buying/selling occurs, what is the price response?” on NSE. Cumulative `Δvolume` between 5s snapshots is a **weak, unvalidated proxy** and must carry `aggressor_confidence=UNAVAILABLE` / `classification=AMBIGUOUS` if ever used exploratorily — it does **not** pass Stage 0 for aggression.

---

## Crypto evidence (Pi)

| Surface | Finding |
| --- | --- |
| Production decision clock | `btcusdt@bookTicker` (L1 bid/ask qty) |
| OF paper / plan stamps | `order_flow_source=bookTicker`; imbalance from displayed L1 |
| Durable `aggTrade` / aggressor journal | **Absent** |
| `event_driven/` | Candidate recommendation / telemetry artifacts — **not** a trade aggressor tape |
| Multi-level depth history | **Absent** on the RP-608 crypto path |

**Implication:** Crypto OFX aggression/response research requires a **new observational capture** (e.g. Binance `aggTrade` with `isBuyerMaker`, optional diff-depth), separate from the frozen bookTicker OF baseline.

---

## Stage-0 verdict (locked)

### PASS (narrow) — data enough to study **displayed liquidity response** on NSE

- Durable L1–L5 + timestamps + last_price/volume exist.
- OFX may **read** `depth_snapshots.jsonl` for research extracts under `artifacts/rp608_ofx/` / `research/rp608_ofx/data/`.
- Must not alter book-intel writers, OF paper, or Estimand A.

### FAIL — data **not** enough for **aggression → response** (core OFX question)

- No NSE trade+quote aggressor stream with confidence/quality fields.
- No crypto aggressor tape.
- First experiment in the OFX scope (“price response to aggressive flow”) is **blocked** until observational capture closes this gap.

### Overall Stage 0 disposition

| Question | Answer |
| --- | --- |
| Can Stage 1 build a full aggression/response observer now? | **No** |
| Can Stage 1 begin a **displayed-liquidity + price** observational subset on NSE depth history? | **Yes** (optional; must label as non-aggressor) |
| Should thresholds / OF paper change? | **No** |
| Is OFX a fifth performance book? | **No** |

---

## Capture gaps to close before aggression Stage 1 (research-only; not authorized here)

Documented for a **separate** observational-capture authorization — not implied by Stage 0:

1. **NSE:** Persist LTQ + LTT (and ideally trade-driven packet markers) alongside contemporaneous bid/ask; durable append-only trade+quote journal with `aggressor_inference` + `confidence` + `unknown` bucket; measure classification coverage on a live cash window (shared interest with intrabar Phase 0).
2. **Crypto:** Durable `aggTrade` (or equivalent) journal with `isBuyerMaker`, trade time, qty, price; keep separate from bookTicker OF baseline.
3. Report coverage: classified / (classified + unknown); never invent aggressor from candle colour.

---

## Explicit non-claims

- Poor 4 Nov OF book economics are **not** re-interpreted here.
- Availability of depth history is **not** proof of absorption/acceptance edge.
- This note does **not** authorize Stage 1 implementation, Step 3+, Gate R, or any baseline rule change.
- Registration of OFX earlier today is **project state**; this Stage 0 note is the first **data** inspection artifact.

---

## Amendment — Stage 0B (2026-11-04)

Kite historical candles tested: [`rp608_ofx_stage0b_kite_historical_feasibility.md`](rp608_ofx_stage0b_kite_historical_feasibility.md).

| Revision | Effect on Stage 0 |
| --- | --- |
| Historical NSE OHLCV via Kite (mapped instruments) | **Confirmed AVAILABLE** (sample PASS) |
| Candle-resolution price-response research | **FEASIBLE** |
| Aggressor / bid-ask-at-trade / L1–L5 via historical API | **Unchanged — still BLOCKED** |
| Full Stage 1 OFX observer | **Unchanged — still BLOCKED** |

Stage 0B does **not** authorize absorption labels from candles.

## Next gate

| Path | Condition |
| --- | --- |
| A — Aggression Stage 1 | Separate capture authorization + coverage measurement PASS |
| B — Displayed-liquidity observational subset | May proceed under OFX research tree **without** touching baselines; must not claim aggressor labels |
| C — Candle-only price-response research | Feasible after Stage 0B; requires **separate** authorization; no OFX mechanism labels |
| Default | Hold full OFX observer until Path A or explicit Path B/C authorization |

**Recommended default:** hold full Stage 1; prefer aggressor capture (Path A) before mechanism classification. Path C is optional and must stay clearly non-order-flow.
