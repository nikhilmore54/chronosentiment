# PHASE 4: Replay Asset Audit

**Date:** 2026-06-03  
**Objective:** Determine the availability of replay assets, schemas, and pipelines before writing any orchestration or conversion code for Phase 4.

## 1. Asset Inventory

| Asset | Exists? | Purpose / Findings |
|-------|---------|--------------------|
| **Historical JSONL Substrates** | Partial | There are 91 substrate directories in `core/chronology/historical/` (e.g., `2024_cpi_shock_1h_1m`, `2026_aapl_overnight_gap_1m`). However, these are highly specific, hand-picked events. There is **no broad coverage** of the Q1/Q2 2025 NIFTY/BANKNIFTY catalog. |
| **Substrate Schema/Pipelines** | Yes | Multiple substrate generators exist (`build_ecology_substrate.py`, `build_crypto_substrate.py`, `candle_substrate.py`). These prove the system already has a defined process for converting data (e.g., `yfinance` or CSVs) into valid JSONL replay event streams. |
| **Replay Outputs (`trace_summary.json`)** | Yes | The `trace_replay` engine generates a `trace_summary.json` for different cognitions (e.g., `rolling_50`, `event_reset`) containing execution metrics like `persistence`. |
| **Deterministic Replay Harness** | Yes | `scripts/run_tier1_observability_demo.py` perfectly demonstrates how to orchestrate the Rust engine, emit manifests, and certify equivalence. |

## 2. Unit of Replay Mapping

**The Mismatch:**
- **Market Ecology:** Defined daily (e.g., the NIFTY session for 2025-01-06).
- **Replay Substrate:** Defined per continuous event stream (e.g., `btcusdt_1779480000000.jsonl`).

**The Required Contract:**
To benchmark execution by ecology, we must establish a formal mapping:
`Historical Session (Date + Symbol) -> Replay Substrate (JSONL)`

## 3. Findings on 5–10 Ecology-Labelled Sessions
I cross-referenced the 91 existing `core/chronology/historical/` directories with the Q1/Q2 NIFTY/BANKNIFTY session catalog. 
**Result:** They do not overlap. The existing substrates are predominantly crypto (BTCUSDT), US Equities (AAPL, TSLA, NVDA), or specific macro shocks from 2024 and 2026. The Q1/Q2 2025 daily Indian equity sessions do not have pre-built JSONL replay substrates.

---

## Conclusion & Recommended Next Step

The audit clearly reveals that while a mature **conversion pipeline and schema** exist, the specific **Q1/Q2 2025 substrates do not**.

Therefore, the first mandatory engineering task of Phase 4 is to build the missing link:
**Create `scripts/csv_to_replay_substrate.py`** to formally convert the `historical_capture/batch_q1` CSVs into the JSONL format expected by the replay engine, defining the `Historical Session -> Replay Substrate` contract in the process.
