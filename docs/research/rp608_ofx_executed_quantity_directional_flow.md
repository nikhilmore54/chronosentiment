# RP-608-OFX — Executed-Quantity Directional Order Flow

**Protocol:** `RP-608-OFX-EXECUTED-QTY-FLOW-001`  
**Date:** 2026-11-07  
**Authority:** 0.0% · research-only  
**Governance:** `.cursor/rules/chronosentiment-core.mdc` §3b/§3c/§3d · `.cursor/rules/rp608-ofx.mdc`  
**Construct lock:** `ACCEPT_CONSTRUCT_DISTINCTION` — this component is **not** baseline OF.

---

## 1. Implementation summary

Isolated OFX library that measures **signed executed aggressor volume**:

\[
\text{signed\_delta\_qty} = Q_{\text{buy-aggressor}} - Q_{\text{sell-aggressor}}
\]

over a documented observation window \([t_0, t_1)\).

| Path | Role |
| --- | --- |
| `research/rp608_ofx/executed_flow/aggregate.py` | Window aggregation + DQ status |
| `research/rp608_ofx/executed_flow/crypto_aggtrade.py` | Binance `aggTrade` → **DIRECT** |
| `research/rp608_ofx/executed_flow/nse_inferred.py` | Stage 0D ticks → **DERIVED_VALIDATED** (MEDIUM+) |
| `tests/test_rp608_ofx_executed_quantity_flow.py` | Unit tests |

**Not connected** to entry/exit/stops/targets, baseline `OF_SUP`, ±0.30, reverse-OF, plan books, or live authority.

---

## 2. Exact calculation

For each `ExecutedTrade` (side and quantity bound on the **same** object):

1. Filter to `instrument_id` and `timestamp_ms ∈ [window_start_ms, window_end_ms)`.
2. Skip duplicates (`sequence_id` / `raw_ref`), invalid qty (≤ 0 / non-numeric).
3. Include in signed aggregates only if `side.classification ∈ {DIRECT, DERIVED_VALIDATED}` and `aggressor ∈ {BUY, SELL}`.
4. Accumulate:
   - `buy_aggressor_qty`
   - `sell_aggressor_qty`
   - `total_classified_qty = buy + sell`
   - `signed_delta_qty = buy − sell`
   - `classification_coverage = total_classified_qty / (classified + unclassified valid qty attempted)`
5. Candle OHLC / direction / unsigned volume may attach as **metadata only** — never set aggressor side or enter ΔQ.

Statuses: `OK` | `PARTIAL_COVERAGE` / `INCOMPLETE` | `NO_TRADES_IN_WINDOW` | `NO_CLASSIFIED_AGGRESSOR_VOLUME` | `INVALID_WINDOW`.

Default research window constant: `DEFAULT_WINDOW_MS = 60_000` (callers pass explicit bounds).

---

## 3. Source semantics

### 3.0 Source boundary (locked 2026-11-07)

Do **not** conflate “source available” with “DIRECT aggressor available.”

| Market | Source | Executed quantity | Aggressor direction |
| --- | --- | --- | --- |
| Crypto | **Binance `aggTrade`** | Direct (`q`) | **DIRECT** from maker/taker flag (`m`) |
| NSE | **Zerodha Kite Connect API** | LTQ available in supported full-mode tick data (`last_traded_quantity`) | **Not** established by LTQ alone → inference only |

| NSE field / concept | Meaning |
| --- | --- |
| Kite `ltq` / `last_traded_quantity` | Last traded quantity on the tick — **not** necessarily cumulative session volume |
| Trade / aggressor direction | Requires a defensible buy/sell classification; **candle colour or price movement alone cannot establish it** |
| NSE OFX signed flow | Remain **`DERIVED_VALIDATED`** only when the Stage 0D (or successor) inference method is validated; **never** label `DIRECT` merely because Kite is the source |

Retain Kite as the NSE source. Further directional aggregation depends on documented WebSocket tick fields + market-depth context already used by Stage 0D — not on inventing a new venue.

### 3.1 Crypto — Binance `aggTrade` → `DIRECT`

| Field | Use |
| --- | --- |
| `s` / `symbol` | Instrument |
| `E` / `T` / `timestamp` | Trade/event time (ms) |
| `p` / `price` | Trade price |
| `q` / `quantity` / `volume` | Executed quantity |
| `m` / `isBuyerMaker` / `is_buyer_maker` | Aggressor |
| `a` / agg id | Sequence / dedupe |

**Direction rule (exchange contract):**

| `isBuyerMaker` | Aggressor |
| --- | --- |
| `false` | **BUY** (buyer is taker) |
| `true` | **SELL** (seller is taker) |

Missing `m` → `UNAVAILABLE` (qty not signed).

### 3.2 NSE — Kite Connect (Stage 0D inferred) → `DERIVED_VALIDATED` (not DIRECT)

**Source:** Zerodha Kite Connect full-mode WebSocket (existing Stage 0D tee). Source is **available**; DIRECT aggressor is **not**.

| Field | Use |
| --- | --- |
| `symbol` / `instrument_token` | Instrument |
| `receipt_utc_ms` | Timestamp |
| `last_traded_quantity` (LTQ) | Per-tick last trade qty candidate — not cumulative volume by itself |
| `volume` | Cumulative volume context (trade-like gating); not unsigned bar volume substitute for side |
| L1–L5 depth / bid/ask | Quote context for **inference only** |
| `trade_like` | Gate (default require true) |
| `aggressor_inference_primary.label` | BUY_/SELL_ candidate |
| `aggressor_inference_primary.confidence` | Default min **MEDIUM** |

Kite does **not** supply verified exchange aggressor side. LOW / AMBIGUOUS / UNAVAILABLE → not signed. This feed **never** claims `DIRECT` because the API vendor is Kite.

---

## 4. Feed-specific data availability

| Feed | Source | Aggressor class | Durable journal | Signed-flow readiness |
| --- | --- | --- | --- | --- |
| Crypto `aggTrade` | **Binance** | **DIRECT** (`m`) | Shadow journal **`ofx_aggtrade_capture_v1/`** when daemon capture enabled | Library READY; durable capture **IMPLEMENTED** (2026-11-07) |
| NSE Stage 0D inferred | **Kite Connect** | **DERIVED_VALIDATED** (MEDIUM+) | `ofx_tick_capture_v1/` when enabled | Library READY for DERIVED path |
| NSE exchange-verified aggressor tape | Kite does not provide | — | N/A | DIRECT signed flow **STOPPED** (methodology gap, not “no Kite source”) |
| Unsigned OHLCV / candle colour | — | — | — | **Forbidden** as aggressor or signed ΔQ |
| Resting L1 bid/ask qty | bookTicker / Kite depth | Baseline OF only | Baseline journals | **Forbidden** as OFX executed ΔQ input |

---

## 5. Data-quality / coverage report (component contract)

Each `ExecutedFlowWindowResult` exposes:

- `buy_aggressor_qty`, `sell_aggressor_qty`, `total_classified_qty`, `signed_delta_qty`
- `classification_coverage`
- `n_trades_in_window`, `n_classified`, `n_unclassified`, `n_invalid`, `n_duplicates_skipped`, `n_out_of_order`
- `data_quality`, `status`, `side_classification_mode`, `notes`
- optional `candle` metadata (direction independent of ΔQ)

No directional signal is emitted from unclassified-only volume (`status=NO_CLASSIFIED_AGGRESSOR_VOLUME`).

---

## 6. Unavailable / unvalidated capabilities

| Capability | Status |
| --- | --- |
| Crypto durable aggTrade journal on RP-608 shadow path | **IMPLEMENTED** (`ofx_aggtrade_capture_v1/`; requires running shadow daemon with capture enabled) |
| NSE / Kite as market data source | **AVAILABLE** (Kite Connect) |
| NSE exchange-verified aggressor side (DIRECT) | **UNAVAILABLE** from Kite LTQ alone — use DERIVED_VALIDATED only |
| Absorption / institutional labels from ΔQ vs candle | **Not implemented** (forbidden inference) |
| Wiring OFX ΔQ into baseline entry/exit | **Not authorized** |
| Historical relabel of baseline `OF_SUP` | **Forbidden** |
| Participant identity (“institutional”) | **UNVERIFIED** / hypothesis only |

---

## 7. Quantity–side matching validation

- Side and quantity are fields of one `ExecutedTrade` dataclass; aggregation never zips parallel arrays.
- Unit test `test_side_qty_never_cross_paired` asserts buy qty 1 and sell qty 100 stay on their own sides.
- Dedupe keys are per-trade sequence ids, not positional indices.

---

## 8. Baseline confirmation

| Baseline surface | Changed? |
| --- | --- |
| `scripts/rp608_order_flow_mark.py` | **No** |
| `OF_SUP` / ±0.30 / reverse-OF | **No** |
| Crypto/NSE plan or OF paper writers | **No** |
| H2 confirmation criteria / holding | **No** |
| H3 / Gate 2 | **No** |
| Execution authority | **0% unchanged** |

---

## 9. Tests

```bash
python3 -m unittest tests.test_rp608_ofx_executed_quantity_flow -v
```

Covers buy/sell/mixed/equal/zero/missing side/invalid qty/duplicates/window bounds/instrument separation/side–qty binding/red+positive ΔQ/green+negative ΔQ/no L1 fields/crypto DIRECT semantics/NSE DERIVED vs DIRECT.

---

## 10. Programme state (2026-11-07 — Nikhil)

**Library implemented; usable durable live executed-flow series not yet available.**

### Complete

| Item | State |
| --- | --- |
| Signed executed-quantity aggregation | Done |
| Crypto aggressor parse | `DIRECT` (`isBuyerMaker`) |
| NSE inferred parse | `DERIVED_VALIDATED` only (never DIRECT) |
| Unit tests | 22 passed |
| Baseline L1 OF / ±0.30 / reverse-OF | Untouched |
| Trading / entry-exit integration | None |

### Capture status (2026-11-07)

| Feed | Status |
| --- | --- |
| Crypto | **Source = Binance.** Durable shadow capture **AUTHORIZED · IMPLEMENTED** (`docs/research/rp608_ofx_crypto_aggtrade_capture_authorization.md`). Journal under `ofx_aggtrade_capture_v1/`; feeds existing aggregator via DQ report. Not wired to trading. |
| NSE | **Source = Kite Connect (available).** LTQ ≠ cumulative volume; aggressor **not** DIRECT from LTQ. Keep `DERIVED_VALIDATED` + confidence limits; never promote to DIRECT because Kite is the vendor. |

**Still forbidden:** OFX→baseline entry/exit / OF paper mutation, predictive Gate claims, baseline/H2/H3/authority changes, rebuilding the library.

### Volume / direction mark (parallel research path — 2026-11-07)

| Item | Detail |
| --- | --- |
| Module | `research/rp608_ofx/executed_flow/volume_mark.py` |
| Snapshot writer | `scripts/rp608_ofx_crypto_volume_flow_mark.py` → `ofx_volume_flow_snapshot.json` |
| Daemon | Refreshes ~1s from aggTrade journal (does **not** call `classify_order_flow_support`) |
| Signal | `volume_imbalance = signed_delta_qty / total_classified_qty`; LONG if ≥ +thr, SHORT if ≤ −thr |
| Dashboard | Separate **OFX VOL** pill/banner (not L1 ORDER FLOW) |
| Baseline L1 OF paper | **Unchanged** |
