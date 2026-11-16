# RP-608-NSE-OF-BOOK-INTELLIGENCE-001 — Phase 1 Durable L1–L5 History

**Objective:** retain timestamped five-level depth so we can later detect persistent bid support while price is still falling.  
**Authority:** 0.0% · OF ±0.30 / reverse-OF paper / Estimand A **unchanged**

## What shipped

| Piece | Role |
| --- | --- |
| `scripts/rp608_kite_ticker_cache.py` | Full-mode decode now **keeps** `depth_buy` / `depth_sell` (5 levels) alongside L1 scalars |
| `scripts/rp608_nse_of_book_depth_history.py` | Rate-limited append-only journal |
| Daemon hook | Samples on feed-health write + midbar await path |
| Output | `live_capture/shadow_validation/nse/of_book_intelligence_v1/depth_snapshots.jsonl` |
| Coverage | `…/depth_capture_coverage.json` |

Default sample interval: **5s** (skips if faster).

## Explicit non-goals (this phase)

- No replenishment / persistence signal yet  
- No new OF threshold  
- No paper entry/exit influence  

## Live proof (2026-11-02, Pi)

After daemon reload:

- `depth_capture_coverage.json`: `snapshots_written≥357`, `last_symbols_sampled=119`, `rows_with_l2_plus_bid_qty` / ask matching writes, `rows_stale=0`
- `depth_snapshots.jsonl`: rows with `depth_buy`/`depth_sell` length **5**, timestamps present

Phase 1 capture gate: **PASS**. Next step on this tape only: bid persistence / replenishment / price-response features (still no OF rule change).
