# RP-611 Phase-1 Observer Deployment Receipt

**Authorization:** RP-611 PHASE-1 OBSERVER DEPLOYMENT AUTHORIZATION  
**Executed:** 2026-11-01  
**Live order authority:** 0.0%  
**Experiment T0 (preserved):** `2026-11-01T10:47:24Z`

Isolation honored: no RP-608 / existing OF / Gate5 / chronosentiment-core policy changes.

---

## NSE — `RP-611-NSE-VOLUME-ABSORPTION-001`

| Item | Result |
|------|--------|
| Preflight | **PASS** |
| Universe | 119 symbols |
| Source | `live_capture/shadow_validation/nse/rp608_nse_bar_history.json` (read-only) |
| Deployed | **YES** (`observer_deployed: true`) |
| Deploy UTC | `2026-11-01T10:59:14Z` (workspace) / `2026-11-01T10:59:35Z` (Pi stamp) |
| Pre-deployment capture gap | ~710–730 s after T0 |
| Post-T0 bars in source | **NO** (Sunday / last bars pre-T0; expected until next NSE session) |
| First-pass candidates | 0 (no post-T0 completed bars yet) |
| Integrity | SOURCE_CENSUS written; structural integrity OK |

**Validation:** Observer armed for Phase-1. Capture begins when bar history advances past T0 (next session). Stale-source gap is recorded, not ignored.

---

## Crypto — `RP-611-CRYPTO-VOLUME-ABSORPTION-001`

| Item | Result |
|------|--------|
| Preflight | **PASS** |
| Volume source | Binance **spot** `kline_15m` only |
| L1 source | Binance **spot** `bookTicker` |
| Futures / perpetual volume | **Excluded** |
| Deployed | **YES** |
| Deploy UTC | `2026-11-01T10:59:14Z` |
| Pre-deployment capture gap | ~710 s after T0 |
| Spot klines (completed) | 99 |
| First-pass candidates | 0 (no **completed** 15m bar with `open_time >= T0` yet; next eligible close after T0) |
| L1 snapshot | Written (`RP611C_l1_telemetry.jsonl`) |

**Validation:** Spot-only path confirmed. Observer armed; candle integrity/candidates will append on subsequent completed post-T0 bars.

---

## Artifacts

- Journals under `artifacts/rp611_{nse,crypto}_volume_absorption/journals/`
- Deploy events: `RP611_phase1_deploy_receipt.jsonl`, `RP611_clock_events.jsonl`
- Scripts: `scripts/rp611_phase1_{nse,crypto}_observer.py`, `scripts/rp611_phase1_deploy.py`

## Disposition

**REGISTERED · CLOCK RUNNING · PHASE-1 DEPLOYED** (0% authority)

Full simulated-trade lifecycle (1.5R exits) remains for later phase work; Phase-1 scope is integrity + candidate/confirmation/L1 capture only.
