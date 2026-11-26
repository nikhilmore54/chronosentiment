# RP-608-WL-IMPROVE — Kite Historical Path for Decision-Time Features

**Date:** 2026-11-06  
**Authority:** 0.0% · baseline frozen · Gate 2 criteria unchanged  
**Execution host:** **Pi** (`docs/research/rp608_pi_research_execution.md`)

## Authorization

Kite Connect **historical OHLCV** may be used to reconstruct look-ahead-safe decision-time features (trend / momentum / price location) for **NSE** books.

## Scope

| Book | Kite applicable? | Status |
| --- | --- | --- |
| NSE Estimand A | Yes | **Join DONE** (Pi, 2026-11-06) |
| NSE OF | Yes (same bars) | Deferred until A path used in discovery |
| Crypto plan | **No** — Kite has no BTCUSDT | Feature gap unchanged |
| Crypto OF | No | Unchanged |

## Causal rule

Only bars with **close time &lt; entry time** enter features. No future-bar leakage. No Gate 2 reopen from descriptive W/L alone.

## Tooling (run on Pi)

```bash
# Preferred from Mac:
scripts/research/rp608_pi_research_run.sh kite-nse-a-join

# Or on Pi directly:
python3 scripts/research/rp608_wl_kite_nse_decision_time_bars.py \
  --export-dir artifacts/rp608_econ_edge_diag \
  --out-dir artifacts/rp608_econ_edge_diag
```

Uses production `KiteDataFeedAdapter.fetch_historical_15m_bars` (Stage 0B path).

## Status (2026-11-06, post re-auth)

| Item | State |
| --- | --- |
| Mac Kite token | Stale — **do not** use Mac for Kite historical |
| Pi credentials | Fresh · fetch **OK** |
| NSE A join (disc+holdout days) | **DONE** on Pi · 1625/1625 feature-ready |
| NSE A discovery v2 | **DONE** · Gate 2 **DO NOT ADVANCE** (payoff/net) |
| Gate 2 | Still closed (criteria unchanged) |

## Artifacts

- `artifacts/rp608_econ_edge_diag/wl_kite_nse_decision_time_bars.{md,json}`
- `artifacts/rp608_econ_edge_diag/wl_kite_nse_15m_bars_cache.jsonl`
- `artifacts/rp608_econ_edge_diag/wl_nse_a_decision_time_features_kite_join.jsonl`

## Governance

Does not change OF ±0.30, stops/targets, or Gate 2 criteria. Crypto discovery v2 Gate 2 disposition remains **DO NOT ADVANCE**.
