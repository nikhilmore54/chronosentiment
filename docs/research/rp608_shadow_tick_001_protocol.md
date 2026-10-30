# RP-608-SHADOW-TICK-001 — Crypto Tick Decision Clock

**Status:** Active (Approach B)  
**Authority:** 0.0% live order authority (shadow only)  
**Checkpoint window_id:** `RP-608-SHADOW-001B` (durable schema lock retained)  
**Canonical decision clock:** `tick_book_ticker` (`btcusdt@bookTicker`)

## Decision locked

Rewrite canonical Policy B / Policy C evaluation from completed 15m closes to live Binance L1 bookTicker ticks, while keeping ATR(14) stop/target geometry from the completed 15m OHLCV buffer.

Gate C under `event_driven/` remains an **isolated** parallel multi-lookback comparator and is **not** the canonical B/C path.

## Why not re-run 15m genomes on ticks

15m BaseOnly / WithExhaustion factors (`ret_15m_bps`, Donchian20, VWAP96, vol_burst…) are constant between bar closes. Evaluating those genomes on ticks would not observe intra-bar microstructure. Tick-native features and frozen tick champions are required.

## Streams

| Stream | Role |
|---|---|
| `btcusdt@bookTicker` | Canonical Policy B/C decisions + overlay TP/SL exits |
| `btcusdt@kline_15m` | Completed-bar buffer → ATR(14) for SL/TP; optional macro trend soft filter |

## Frozen artifacts (SHA-locked)

- `datasets/crypto_tick/frozen_policy_b_tick_champion.json` — alphabet `TickBase`
- `datasets/crypto_tick/frozen_policy_c_tick_champion.json` — alphabet `TickWithExhaustion`

Initial genomes are **hand-frozen** deterministic rules aligned with Gate C constants (`IMBALANCE_THRESHOLD`, velocity bps, `TREND_FILTER_BPS`). Full GA retrain on historical ticks is a later research follow-on.

## Decision card contract

Emitted to `rp608_decision_cards.jsonl` on **action change** subject to refractory:

- `protocol_id`: `RP-608-SHADOW-TICK-001`
- `decision_clock`: `tick_book_ticker`
- `decision_event_idx`: monotonic emitted-event counter (prospective sample N)
- `quote_event_id`, `quote_receipt_utc_ms`
- `features_tick`: imbalance, micro vs mid, velocity 5s/30s, tick vol, macro 16/64
- `atr_14`: from completed 15m buffer (overlay geometry)
- `policy_b_action` / `policy_c_action`

Cadence gates:

- Min refractory **1.0 s** between emits
- Quote age ≤ **500 ms**
- Suppress identical NO_TRADE / unchanged action spam

## Overlay

`RP-608-TRADE-PLAN-OVERLAY-001` v1.2.0:

- `signal_cadence`: `tick_book_ticker`
- `atr_geometry_source`: `completed_15m_atr14`
- SL/TP still 1.0× / 1.5× ATR frozen at open; exits remain continuous L1
- One-open-plan concurrency unchanged

## Window accounting / Stage-1 Sharpe note

Prospective sample counts are **emitted decision events**, not 15m bars. Higher N and a different holding horizon vs the prior 15m clock imply Stage-1 Sharpe figures are **not** directly comparable to `RP-608-SHADOW-001B` bar-clock results without an explicit horizon renormalization.

## Non-goals (this protocol)

- Live order authority > 0%
- Replacing Gate C
- Tick-native ATR / stop model
- Full GA retrain on historical ticks

## Alignment

Deterministic event-driven SimEvent path; same quote sequence → same B/C actions (see `.cursor/rules/chronosentiment-core.mdc`).
