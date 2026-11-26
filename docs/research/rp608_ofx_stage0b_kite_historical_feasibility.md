# RP-608-OFX — Stage 0B Kite Historical Data Feasibility

**Protocol:** `RP-608-OFX`  
**Phase:** `STAGE_0B_KITE_HISTORICAL_FEASIBILITY`  
**Disposition:** **EXECUTED · VERDICT LOCKED**  
**Date:** 2026-11-04  
**Authority:** 0.0%  
**Inspection class:** Agent authenticated Kite Connect sample on Pi — **not** independent third-party verification  

**Parent:** [`rp608_ofx_order_flow_response_scoped.md`](rp608_ofx_order_flow_response_scoped.md)  
**Prior:** [`rp608_ofx_stage0_data_feasibility.md`](rp608_ofx_stage0_data_feasibility.md)  
**Artifact:** `artifacts/rp608_ofx/stage0b_kite_historical_feasibility.json`

---

## Why Stage 0B exists

Stage 0 showed live capture lacks a durable aggressor tape. A separate question remained: whether **Kite historical** APIs can fill any retrospective research gap.

**Documented API fact (pre-test):** Kite `/instruments/historical/{token}/{interval}` returns **OHLCV candles**, not trade-by-trade aggressor events, not per-trade bid/ask, and not L1–L5 book history.

Stage 0B tests the **account-specific** path: credentials, instrument map, sample fetch, continuity, and join to the existing depth journal.

---

## Governance (unchanged)

| Item | State |
| --- | --- |
| Four-book baseline | Untouched |
| OF ±0.30 / reverse-OF | Frozen |
| Absorption / acceptance / failed-auction labels | **Still not authorized** from candles alone |
| Stage 1 full aggression→response observer | **Still blocked** |
| Step 3+ / Gate R | Held / unreached |

---

## Integration inspected

| Component | Finding |
| --- | --- |
| Credentials | `~/.kite_credentials.json` on Pi (daemon hot-reload path) |
| Auth check | `GET /user/profile` → `status=success` |
| Instrument map | `live_capture/.../kite_nse_instrument_map.json` · **119** symbols · `{instrument_token, tradingsymbol}` |
| Existing production use | `KiteNSEFeedAdapter.fetch_historical_15m_bars` already calls `.../historical/{token}/15minute` for same-day bootstrap (read-only) |
| Stage 0B sample | Also exercised `minute`, `3minute`, `5minute`, `15minute` |

---

## Sample design

| Parameter | Value |
| --- | --- |
| Session | **2026-11-03** IST 09:15–15:30 (overlaps depth journal) |
| Symbols | RELIANCE.NS, TCS.NS, INFY.NS, HDFCBANK.NS, WIPRO.NS |
| Intervals | minute · 3minute · 5minute · 15minute |

---

## Results — candle fetch

For all five symbols, all four intervals:

| Interval | Candles / symbol | Gaps (>1.5× interval) | Zero-volume bars |
| --- | ---: | ---: | ---: |
| minute | 360 | 0 | 0 |
| 3minute | 120 | 0 | 0 |
| 5minute | 72 | 0 | 0 |
| 15minute | 24 | 0 | 0 |

**Schema observed:** candle arrays are `[datetime, open, high, low, close, volume]` only (6 fields; no extras). Response `data` keys: `candles` only.

**Timestamps:** ISO with `+0530` (IST). Continuous through the cash session for the sample day.

---

## Results — join to depth journal

Probe symbol: **RELIANCE.NS** (`instrument_token=738561`) on 2026-11-03.

| Metric | Value |
| --- | ---: |
| Depth snapshots (symbol/day) | 2,548 |
| Token match map ↔ depth | **Yes** |
| Minute candles | 360 |
| Minutes with ≥1 depth sample | **356 / 360** |
| Minutes without depth | 4 |

**Join feasible** for time-aligned **price (candle) ↔ displayed liquidity (depth snapshot)** co-occurrence on overlapping sessions.

**Caveat:** depth rows are ~5s sampled quotes; candles are OHLCV aggregates. This is **not** trade-level reconstruction and **does not** supply aggressor side.

---

## Capability matrix (revised after 0B)

| Data capability | Assessment |
| --- | --- |
| Historical NSE OHLCV (this account, mapped 119) | **AVAILABLE** (sample PASS) |
| Historical price response at candle resolution | **FEASIBLE** |
| Breakout acceptance / failure at candle resolution | **FEASIBLE** (price-structure only) |
| High-volume candle subsequent response | **FEASIBLE** (unsigned volume) |
| Join candles ↔ existing depth journal (overlap days) | **FEASIBLE** (time co-occurrence) |
| Historical aggressor side | **NOT** supplied by Kite candles — **BLOCKED** |
| Historical bid/ask at each trade | **NOT** supplied — **BLOCKED** |
| Historical L1–L5 book changes via historical API | **NOT** established / **NOT** in candle API — **BLOCKED** |
| Absorption / acceptance / failed-auction OFX labels | **Still blocked** without event-level aggression evidence |
| Stage 1 full order-flow observer | **Still blocked** |

---

## What this unblocks vs what it does not

### Unblocks (narrow research component)

Retrospective **price-response** studies at 1m/3m/5m/15m using Kite history for mapped NSE symbols — optionally co-plotted with depth snapshots on days the depth journal covers.

### Does not unblock

- Aggression → liquidity response → price response observer  
- Inferring buyer/seller initiation from candle volume or colour  
- Treating depth+candle joins as absorption proof  
- Any baseline OF/execution change  

---

## Stage 0B verdict (locked)

| Gate | Result |
| --- | --- |
| Kite historical OHLCV for RP-608 map | **PASS (sample)** |
| Candle-resolution price-response research | **FEASIBLE** |
| Aggression path / full Stage 1 | **STILL BLOCKED** |
| Baseline | **Untouched** |

Overall: Stage 0B **revises** Stage 0 only by confirming a **historical price-response** path. It does **not** overturn the Stage 0 aggressor FAIL.

---

## Explicit non-claims

- Sample of five symbols / one session ≠ full historical coverage audit for all intervals and lookback limits.
- NSE’s commercial historical trade products are **out of scope** of this Kite candle test.
- This note is project-reported execution, not independent verification of Pi journals or governance files by the human reviewer.

---

## Next boundary (unchanged priority)

1. Observational **trade+quote / aggTrade** capture + coverage report (required for Stage 1 OFX).  
2. Optional separate authorization: candle-only price-response research harness (must not emit OFX absorption labels).  
3. No strategy, threshold, or execution changes.
