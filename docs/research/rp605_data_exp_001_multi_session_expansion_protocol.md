# RP-605-DATA-EXP-001: Multi-Session Data Expansion Protocol & Coverage Audit

**Document Identifier**: `RP-605-DATA-EXP-001`  
**Protocol Series**: `RP-605 (NSE Prospective Freshness Validation)`  
**Date**: 2026-10-22  
**Status**: REGISTERED SPECIFICATION & DATA COVERAGE AUDIT  
**Governance Authority**: Strictly **0.0% Live Order Authority** (Shadow Evaluation Only)  
**Adapter State**: `adapters/chronosentiment` **100% FROZEN** (13/13 discipline invariants green)

---

## 1. Executive Mandate & Purpose

Single-session trade audits—regardless of how rigorously verified—cannot establish that a trading decision rule possesses persistent predictive skill across market regimes. A sample of trades from a single morning or four consecutive sessions risks temporal clustering, market-wide trend confounding, and over-fitting to localized liquidity dynamics.

This protocol expands RP-605 into a **multi-session, multi-regime, execution-aware empirical program**. It establishes:
1. An immutable inventory of available historical and prospective NSE market data.
2. A five-layer linked dataset architecture with stable opportunity IDs and deterministic timestamp provenance.
3. Strict separation between development, validation, untouched prospective holdout, and shadow execution stages.

---

## 2. Comprehensive Data Coverage & Integrity Audit

A comprehensive inventory of local repository archives reveals two distinct data tiers:

### Tier 1: 3-Month 5-Minute Continuous OHLCV Substrate (`data/nse/5m/`)
- **Symbol Universe**: 27 top liquid NSE equities (`ADANIENT`, `ADANIPORTS`, `AXISBANK`, `BHARTIARTL`, `BSE`, `CDSL`, `COFORGE`, `DEEPAKNTR`, `DLF`, `GODREJPROP`, `HDFCBANK`, `HINDUNILVR`, `ICICIBANK`, `INFY`, `IRCTC`, `ITC`, `JSWSTEEL`, `KOTAKBANK`, `LT`, `NYKAA`, `ONGC`, `PAYTM`, `POLICYBZR`, `RELIANCE`, `SBIN`, `TATASTEEL`, `TCS`).
- **Date Range**: `2026-01-23 03:45:00 UTC` to `2026-04-24 09:55:00 UTC` (64 full consecutive trading sessions).
- **Bar Granularity**: 5-minute continuous OHLCV (75 bars / session).
- **Data Completeness**: Exactly **4,366 bars per symbol** across all 27 instruments (Total: **117,882 OHLCV bars**).
- **Missingness Audit**: **0 null values, 0 missing bars, 0 negative prices, 0 unaligned timestamps**.
- **Purpose**: Multi-horizon historical behavioural archetype estimation, volatility regime clustering, and multi-month persistence analysis.

### Tier 2: 1-Minute High-Resolution Prospective & Replication Journals (`live_capture/shadow_validation/`)
- **Session Range**:
  - **Replication Sessions**: September 28, 29, 30, and October 01, 2026 (4 full 375-tick intraday sessions).
  - **Prospective Shadow Session**: October 22, 2026 (Tick #54 Authoritative Checkpoint + continuous real-time streaming).
- **Symbol Universe**: 99 active liquid NSE constituents (spanning Nifty 50 and key liquid high-beta scrips).
- **Completed Trade Volume**: **2,029 completed trades** across the 4 replication sessions + **318 exits** at Tick #54.
- **Fields Logged**: `trade_id`, `session_date`, `symbol`, `side`, `entry_timestamp`, `entry_minute_idx`, `entry_price`, `quantity`, `stop_price`, `is_warmup`, `macro_s30`, `macro_r60_bps`, `vol_ratio`, `mfe_pnl`, `mae_pnl`, `exit_timestamp`, `exit_price`, `holding_mins`, `exit_reason`, `gross_pnl`, `friction_inr`, `net_pnl`.
- **Integrity**: 100.0% unrounded composite key uniqueness; exact 1-to-1 counterfactual reconciliation verified.
- **Purpose**: High-frequency entry qualification ablation, warm-up policy verification (Ledger C), and execution fee reconciliation.

---

## 3. Five-Layer Linked Dataset Architecture

To eliminate monolithic CSV structures and prevent lookahead contamination, RP-605 data is structured into five linked schemas:

```
[Layer 1: Market Observations] ────► [Layer 2: Signal Journal]
               │                                   │
               ▼                                   ▼
[Layer 4: Execution & Positions] ◄─── [Layer 3: Opportunity Ledger]
               │
               ▼
[Layer 5: Cost & Evaluation Ledger]
```

1. **Market Observations Ledger**:
   - Schema: `(symbol, timestamp_utc, open, high, low, close, volume, vwap, bid_ask_spread, data_status)`.
   - Invariant: Zero forward leakage; strictly records observations available at or before $t$.
2. **Signal Journal**:
   - Schema: `(signal_id, symbol, timestamp_utc, f15_score, c5_predicted_bps, vol_ratio, macro_regime, is_cold_start)`.
   - Invariant: Point-in-time calculation; frozen adapter outputs only.
3. **Opportunity Ledger (Full Population)**:
   - Schema: `(opportunity_id, signal_id, symbol, entry_eligible, admission_status, rejection_reason, expiry_timestamp)`.
   - Invariant: Conserves 100% of initiated candidates (admitted into B, routed to R, or expired).
4. **Execution & Positions Ledger**:
   - Schema: `(trade_id, opportunity_id, symbol, side, order_type, entry_px, exit_px, entry_ts, exit_ts, exit_reason)`.
   - Invariant: Records both simulated market crossing and passive limit queue states.
5. **Cost & Evaluation Ledger**:
   - Schema: `(trade_id, gross_pnl, brokerage, stt, exch_charges, stamp_duty, sebi_charges, gst, net_pnl_statutory, mtm_drawdown)`.
   - Invariant: Conservation of statutory charges verified to $₹0.000000$ precision.

---

## 4. Multi-Stage Validation Protocol

To guard against data snooping across overlapping sessions:

```
+------------------------------------------------------------------------------------------------------+
| Stage 1: Data Qualification  | Schema validation, monotonic timestamps, zero silent drops.           |
+------------------------------+-----------------------------------------------------------------------+
| Stage 2: Historical Dev      | 64 sessions (Jan-Apr 2026) for behavioral archetype discovery.        |
+------------------------------+-----------------------------------------------------------------------+
| Stage 3: Calibration Set     | Sep 28 - Oct 01 replication sessions for policy ablation.             |
+------------------------------+-----------------------------------------------------------------------+
| Stage 4: Untouched Holdout   | Oct 02 - Oct 21 sessions strictly sequestered until policy freeze.    |
+------------------------------+-----------------------------------------------------------------------+
| Stage 5: Prospective Shadow  | Live continuous streaming without execution authority (0% live).      |
+------------------------------+-----------------------------------------------------------------------+
```
