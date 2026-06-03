# Phase 4.1: Substrate Contract v1

This contract strictly defines the conversion mapping between historical market CSVs and the JSONL substrates ingested by the Rust `trace_replay` engine. 

## 1. Input Granularity
- **Source:** Canonical Q1/Q2 CSVs located in `historical_capture/batch_q1/`.
- **Format:** 1-minute OHLCV candles (`timestamp`, `open`, `high`, `low`, `close`, `volume`).

## 2. Replay Event Model
- **Target schema:** The `trace_replay` engine consumes a flat Tick Event schema, as evidenced by all existing historical substrates (e.g., `btcusdt_1779480000000.jsonl`):
  ```json
  {"symbol": "NIFTY", "timestamp": 1779480000000, "price": 23787.6, "volume": 0.0, "is_buyer_maker": false}
  ```
- **Constraint:** The replay engine event model *does not have native fields for Open, High, or Low*. It only processes sequential `price` ticks.

## 3. Losslessness & Synthetic State Transitions
Because the target schema only accepts a single `price`, a naive 1-to-1 conversion (mapping the candle's `close` to `price`) is **lossy**. It discards the candle's `high` and `low`, making it impossible to perfectly reconstruct the Ecology metrics (`session_range_pct`, `realized_volatility`) from the substrate.

**Solution: 4-Tick Synthetic Expansion (Synthetic Ecology Replay Substrate v1)**
To preserve market ecology deterministically, every 1-minute candle will be expanded into 4 sequential ticks inside the replay substrate:
1. `timestamp + 0ms`: `price` = `open`
2. `timestamp + 15000ms`: `price` = `high` (or `low`, depending on candle direction)
3. `timestamp + 30000ms`: `price` = `low` (or `high`, depending on candle direction)
4. `timestamp + 45000ms`: `price` = `close`

This guarantees that the absolute extremes of the session are perfectly preserved in the replay event stream.

### Certification Boundaries
By inventing an intra-minute path, we are injecting synthetic execution dynamics. Therefore, this contract formally separates certification into two levels:

**Level 1 Certification (Market Ecology Validation) — PHASE 4 TARGET**
- **Goal:** Can replay preserve ecological geometry deterministically?
- **Allowed:** OHLC → synthetic tick expansion.
- **Certified for:** Ecology reconstruction (range, volatility, returns) and deterministic replay execution.

**Level 2 Certification (Execution Realism Validation) — FUTURE TARGET**
- **Goal:** Can replay model queue dynamics and latency sensitivities accurately?
- **Required:** True historical tick streams or order-book archives.
- **Not Certified by this Contract:** Queue realism, latency realism, fill realism.

## 4. Determinism
- The `csv_to_replay_substrate.py` converter will act as a pure, stateless mathematical function.
- `f(NIFTY_1m.csv) = NIFTY_substrate.jsonl`
- Multiple invocations on the same CSV will produce byte-identical JSONL files with identical SHA-256 hashes. 
- Order of operations inside the 4-tick expansion will strictly follow:
  - If `close >= open`: Open → Low → High → Close
  - If `close < open`: Open → High → Low → Close

## 5. Safeguards & Provenance

### Safeguard 1: Preserve Expansion Provenance
To prevent future work from mistaking synthetic expansion for genuine market ticks, every generated substrate batch will be accompanied by a `manifest.json` declaring provenance:
```json
{
  "generator": "synthetic_ohlc_expansion_v1",
  "source_file": "NIFTY_2025_01_06.csv",
  "expansion_policy": "directional_ohlc_4tick",
  "certification_level": "L1_ECOLOGY_ONLY"
}
```

### Safeguard 2: Reconstruction Certification
Before replay certification begins, the converter must pass a strict round-trip certification:
`CSV → Synthetic Substrate → Reconstructed OHLC → Ecology Metrics`
This roundtrip must preserve exact values for Open, High, Low, Close, Range %, Net Return %, and Ecology Label.
