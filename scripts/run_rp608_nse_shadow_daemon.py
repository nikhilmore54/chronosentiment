#!/usr/bin/env python3
"""
scripts/run_rp608_nse_shadow_daemon.py

RP-608-NSE-SHADOW-001: Dedicated Prospective Indian Equity Shadow Observation Daemon.
Strictly Read-Only (0.0% Order Authority) with Atomic Durable Checkpointing.

Features:
  1. 119 Predeclared Liquid NSE Symbol Universe:
     - Exact census from RP-608-NSE-SHADOW-001 Section 2 (ADANIENT.NS to WIPRO.NS)
     - Concurrent market-data ingestion with rate-limiting and connection pooling
  2. Strict Session Schedule & 15-Minute Bar Alignment:
     - 25 calendar 15m intervals (09:15–15:30 IST); F&O continuous halt at 15:15 (CAS era)
     - Decision timestamp T_decision(k) == T_close(k)
     - Closing window 15:00–15:15 IST (bars 23–24): no new entries; flatten at bar 23 (15:00)
     - Pre-open exclusion (< 09:15 IST) and post-close idle (> 15:30 IST)
  3. Dual Estimand Architecture (Section 5):
     - Estimand B (Directional Opportunity Prediction):
       * Bars 1-19 eligible (T_outcome <= 15:00 IST close-window start, fixed 4-bar forward)
       * Bars 20-25 tagged IntradayBoundaryExcluded
       * Ex-ante forecasts: Long = 0.58, StandAside = 0.38
       * Frozen training prevalence reference: 0.512
     - Estimand A (Trade-Outcome Prediction):
       * Entered trade lifecycle with 60m / 180m / 15:00 close-window square-off
       * New entries bars 1..19 (60m completes by 15:00)
       * Frozen training prevalence reference: 0.58
       * Statistical invariant: Balanced accuracy strictly marked None (N/A)
  4. Multi-Component Indian Statutory Friction Ledger (Section 4):
     - Brokerage: min(₹20, Turnover * 0.03%) both legs
     - STT: 0.025% on turnover (Sell leg only)
     - Exchange charges: 0.00297% on turnover both legs
     - GST: 18.0% on (Brokerage + Exchange + SEBI) both legs
     - SEBI charges: ₹10/Cr (0.0001% on turnover) both legs
     - Stamp duty: 0.003% on turnover (Buy leg only)
     - Spread crossing: Half-spread based on observed L1 top-of-book quote
     - Slippage: 2.0 bps exogenous adverse latency both legs
  5. Durable, Crash-Safe Checkpointing & Resumption:
     - Atomic rename (os.replace) via temporary file with os.fsync
     - Decoupled cursors: last_durable_bar_idx vs last_accounted_bar_idx
     - Outage registry tracking drops without diluting trade denominator
     - Fail-closed behavior on tamper, checksum mismatch, or consecutive outage breach
"""

from __future__ import annotations

import argparse
import asyncio
from concurrent.futures import ThreadPoolExecutor
from dataclasses import asdict, dataclass, field
from datetime import date, datetime, timezone, timedelta
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import sys
import threading
import time
from typing import Any, Dict, List, Optional, Tuple
import urllib.request
import urllib.error

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.dynamic_trade_boundaries import compute_dynamic_boundaries
from scripts.market_regime_classifier import classify_market_regime
from scripts.nse_market_context import (
    apply_market_chop_to_regime,
    compute_market_context,
    session_return_bps_from_bars,
)
from scripts.rp608_arm_a_baseline import attach_regime_telemetry
from scripts.rp608_order_flow_mark import (
    inspect_order_flow_book,
    l1_from_nse_quote,
)
from scripts.rp608_nse_of_paper_ledger import NseOfPaperLedger
from scripts.rp608_gate2x_adapter_client import (
    DEFAULT_NSE_EVIDENCE,
    apply_soft_breach_to_trade,
    build_snapshot as gate2x_build_snapshot,
    decide as gate2x_adapter_decide,
)
from scripts.rp608_plan_lifecycle_client import (
    get_session as plan_lifecycle_session,
    lifecycle_enabled as plan_lifecycle_enabled,
    log_require_rust_block as plan_lifecycle_log_require_rust,
    require_rust_lifecycle as plan_lifecycle_require_rust,
    map_lifecycle_close,
    merge_lifecycle_into_plan,
    plan_from_overlay,
)
from scripts.rp608_nse_muhurat_session import (
    completed_bars_since_open,
    resolve_session_window,
    session_phase,
)
from scripts.rp608_ofx_nse_volume_flow_mark import (
    build_ofx_nse_volume_flow_snapshot,
    global_nse_volume_flow_buffer,
    write_ofx_nse_volume_flow_snapshot,
)
from scripts.rp608_ofx_nse_volume_paper_trades import OfxNseVolumePaperManager
from scripts.rp608_nse_of_book_depth_history import (
    NseOfBookDepthHistoryJournal,
    collect_ticks_by_symbol,
)
from scripts.rp608_prospective_v1 import Market as ProspectiveMarket
from scripts.rp608_prospective_v1_observer import (
    observe_bar as prospective_v1_observe_bar,
    record_recommendation as prospective_v1_record_recommendation,
)
from scripts.rp608_recommendation_trade_ledger import (
    append_nse_recommendation_trade_link as prospective_v1_append_nse_rec_trade,
    append_nse_trade_closed as prospective_v1_append_nse_trade_closed,
)
from scripts.rp608_kite_ticker_cache import KiteTickerQuoteCache, TickSink
from scripts.rp608_ofx_nse_tick_journal import (
    OfxNseInferredAggressorTickJournal,
    classify_trade_like,
    ofx_tick_capture_enabled,
)
from scripts.rp608_nse_equity_costs import (
    BROKERAGE_CAP_PER_LEG,
    BROKERAGE_RATE,
    EXCHANGE_TURNOVER_RATE,
    GST_RATE,
    SEBI_TURNOVER_RATE,
    SLIPPAGE_RATE_PER_LEG,
    STAMP_DUTY_BUY_LEG,
    STT_RATE_SELL_LEG,
    calc_indian_equity_friction,
    calc_nse_of_paper_accounting,
)

# ─── Registered 119 NSE Symbol Universe (RP-608-NSE-SHADOW-001 Section 2) ───
NSE_119_SYMBOLS: Tuple[str, ...] = (
    "ADANIENT.NS", "ADANIGREEN.NS", "ADANIPORTS.NS", "AMBUJACEM.NS", "APOLLOHOSP.NS", "ASIANPAINT.NS",
    "ATGL.NS", "AUROPHARMA.NS", "AXISBANK.NS", "BAJAJ-AUTO.NS", "BAJAJFINSV.NS", "BAJFINANCE.NS",
    "BALKRISIND.NS", "BANDHANBNK.NS", "BANKBARODA.NS", "BERGEPAINT.NS", "BHARATFORG.NS", "BHARTIARTL.NS",
    "BIOCON.NS", "BOSCHLTD.NS", "BPCL.NS", "BRITANNIA.NS", "CANBK.NS", "CHOLAFIN.NS",
    "CIPLA.NS", "COALINDIA.NS", "COFORGE.NS", "COLPAL.NS", "CONCOR.NS", "CUMMINSIND.NS",
    "DABUR.NS", "DIVISLAB.NS", "DLF.NS", "DRREDDY.NS", "EICHERMOT.NS", "ESCORTS.NS",
    "FEDERALBNK.NS", "GAIL.NS", "GLENMARK.NS", "GODREJCP.NS", "GODREJPROP.NS", "GRASIM.NS",
    "HAL.NS", "HAVELLS.NS", "HCLTECH.NS", "HDFCBANK.NS", "HDFCLIFE.NS", "HEROMOTOCO.NS",
    "HINDALCO.NS", "HINDUNILVR.NS", "ICICIBANK.NS", "ICICIGI.NS", "ICICIPRULI.NS", "IDEA.NS",
    "IDFCFIRSTB.NS", "IGL.NS", "INDHOTEL.NS", "INDUSINDBK.NS", "INDUSTOWER.NS", "INFY.NS",
    "IOC.NS", "IPCALAB.NS", "IRCTC.NS", "ITC.NS", "JINDALSTEL.NS", "JSWSTEEL.NS",
    "JUBLFOOD.NS", "KOTAKBANK.NS", "LICI.NS", "LT.NS", "LUPIN.NS", "M&M.NS",
    "MAHABANK.NS", "MARICO.NS", "MARUTI.NS", "MPHASIS.NS", "MRF.NS", "MUTHOOTFIN.NS",
    "NATIONALUM.NS", "NAUKRI.NS", "NESTLEIND.NS", "NMDC.NS", "NTPC.NS", "OBEROIRLTY.NS",
    "OFSS.NS", "ONGC.NS", "PAGEIND.NS", "PERSISTENT.NS", "PETRONET.NS", "PFC.NS",
    "PIDILITIND.NS", "PNB.NS", "POLYCAB.NS", "POWERGRID.NS", "RECLTD.NS", "RELIANCE.NS",
    "SAIL.NS", "SBICARD.NS", "SBILIFE.NS", "SBIN.NS", "SHREECEM.NS", "SIEMENS.NS",
    "SRF.NS", "SUNPHARMA.NS", "TATACONSUM.NS", "TATASTEEL.NS", "TCS.NS", "TECHM.NS",
    "TITAN.NS", "TMCV.NS", "TORNTPHARM.NS", "TRENT.NS", "TVSMOTOR.NS", "ULTRACEMCO.NS",
    "UNITDSPR.NS", "UPL.NS", "VBL.NS", "VEDL.NS", "WIPRO.NS"
)

# ─── Registered Governance & Protocol Constants ──────────────────────────────
PROTOCOL_ID = "RP-608-NSE-SHADOW-001"
REGISTERED_WINDOW_START_UTC_MS = 1792986300000  # 2026-10-26T03:45:00.000Z (09:15:00 IST)
FIFTEEN_MINUTES_MS = 15 * 60 * 1000  # 900,000 ms
TOTAL_BARS_PER_SESSION = 25
# Closing-window adaptation (F&O continuous ends 15:15; CAS ~15:15–15:35):
#   Bar 23 closes 15:00 IST — start of 15:00–15:15 closing window
#   Bar 24 closes 15:15 IST — continuous halt / CAS open
CLOSING_WINDOW_START_BAR = 23
MANDATORY_SQUARE_OFF_BAR = 23  # flatten at 15:00 bar close
MAX_NEW_ENTRY_BAR = 19  # 60m (4 bars) completes by bar 23 (15:00)
ESTIMAND_B_ELIGIBLE_BARS = 19  # T+4 endpoint <= bar 23 (15:00 continuous reference)
MAX_CONSECUTIVE_OUTAGES_FAIL_CLOSED = 1  # TG-4: < 1 consecutive missing bar for NSE

# RP-608-REGIME-BREAKOUT-001 — Arm A freeze + observational regime telemetry
# ARM_A_FROZEN still authenticates the fixed ±1.0%/±1.5% baseline for fair comparison.
# Live NSE shadow exits may use ENTRY-TIME dynamic ATR/structure bounds when enabled.
ARM_A_FROZEN = True
ARM_A_FREEZE_PROTOCOL = "RP-608-REGIME-BREAKOUT-001"
LIVE_ORDER_AUTHORITY_PCT = 0.0  # Strictly 0.0% — no order placement surface
REGIME_TELEMETRY_ENABLED = True  # Observational only; must not alter Arm A actions
HOLDOUT_SESSIONS_SEALED = "41-68"

# Exit policy: DYNAMIC_OHLCV_STRUCTURE_V1 (ATR/swing, frozen at entry) or ARM_A_FIXED.
# Dynamic needs ≥15 completed 15m bars (prior sessions allowed for ATR only; traj15 stays session-causal).
EXIT_POLICY = os.environ.get("RP608_NSE_EXIT_POLICY", "DYNAMIC_OHLCV_STRUCTURE_V1").strip().upper()
# Entry timing: TICK_PATH = arm on durable 15m close, fill on each WS L1 tick; BAR_CLOSE = legacy 15m close.
ENTRY_PATH = os.environ.get("RP608_NSE_ENTRY_PATH", "TICK_PATH").strip().upper()
# Reject quote-only WS refreshes for Estimand A entry (volume/LT trade-like filter).
TICK_TRADE_LIKE_ENTRY = os.environ.get("RP608_NSE_TICK_TRADE_LIKE_ENTRY", "1").strip().lower() not in {
    "0",
    "false",
    "no",
    "off",
}
ARM_A_STOP_MULT = 0.990
ARM_A_TARGET_MULT = 1.015
ARM_A_STOP_BPS = 100.0
ARM_A_TARGET_BPS = 150.0
ATR_FEATURE_LOOKBACK_DAYS = 5
ATR_FEATURE_MAX_BARS = 40

# Gate-3 warmup: cross_gap_neutral_v1 unlocks before 10:30 using prior-session bars
# with overnight gap removed from traj15. Override with RP608_G3_WARMUP_POLICY=same_session.
from scripts.rp608_nse_g3_warmup import (  # noqa: E402
    DEFAULT_G3_WARMUP_POLICY,
    POLICY_CROSS_GAP_NEUTRAL_V1,
    evaluate_g3_momentum,
)
from scripts.rp608_nse_coralys_input_context import (  # noqa: E402
    CoralysInputContextStore,
    decision_ts_from_bar_ms,
    gate_enabled as coralys_input_gate_enabled,
)
from scripts.rp608_nse_trade_pattern_match import (  # noqa: E402
    entry_gate_enabled as pattern_entry_gate_enabled,
    entry_gate_mode as pattern_entry_gate_mode,
    entry_gate_mode_baseline as pattern_entry_gate_mode_baseline,
    exit_gate_enabled as pattern_exit_gate_enabled,
    features_from_coralys_and_candles,
    load_library as load_trade_pattern_library,
    match_entry as match_trade_pattern_entry,
    match_exit as match_trade_pattern_exit,
)
from scripts.rp608_gate2e_shadow_insufficient import (  # noqa: E402
    build_gate2e_shadow_insufficient_stamp,
    gate2e_shadow_insufficient_enabled,
    matches_shadow_observe_abstain_insufficient,
)
from scripts.rp608_gate2e_regime_align import (  # noqa: E402
    evaluate_regime_align,
)
from scripts.rp608_nse_trade_pattern_library import (  # noqa: E402
    default_library_path as trade_pattern_library_path,
    session_phase_ist as _pattern_session_phase_ist,
    traj15_bucket as _pattern_traj15_bucket,
)

G3_WARMUP_POLICY = DEFAULT_G3_WARMUP_POLICY

# Cost schedule constants re-exported from scripts.rp608_nse_equity_costs
DEFAULT_NOTIONAL_PER_TRADE_INR = 100_000.0  # ₹100,000 INR reference notional

# Gate 2X (2026-11-21): MFE give-back protect — port of crypto v1.1+.
# Arm after peak tp_frac + MFE clears cost/floor; exit at cost-clear floor.
# v1.4 Adaptive TP may extend post-touch TP / lock SL; OF ±0.30 / traj15 unchanged.
GATE2X_MFE_GIVEBACK_ENABLED = os.environ.get(
    "RP608_NSE_GATE2X_MFE_GIVEBACK", "1"
).strip().lower() not in {"0", "false", "no", "off"}
GATE2X_ARM_MIN_TP_FRAC = float(os.environ.get("RP608_NSE_GATE2X_ARM_MIN_TP_FRAC", "0.40"))
GATE2X_MIN_HOLD_S_BEFORE_EXIT = float(
    os.environ.get("RP608_NSE_GATE2X_MIN_HOLD_S", "120")
)
GATE2X_MEANINGFUL_FLOOR_BPS = float(
    os.environ.get("RP608_NSE_GATE2X_FLOOR_BPS", "5.0")
)
GATE2X_PROTECT_FLOOR_MODE = os.environ.get(
    "RP608_NSE_GATE2X_PROTECT_FLOOR", "COST_CLEAR"
).strip().upper()  # COST_CLEAR | ENTRY
# v1.2: volume-confirm reversal (HIGH immediate; MED/LOW/UNK dwell).
GATE2X_REVERSAL_VOLUME_CONFIRM = os.environ.get(
    "RP608_NSE_GATE2X_VOL_CONFIRM", "1"
).strip().lower() not in {"0", "false", "no", "off"}
GATE2X_REVERSAL_MED_VOL_DWELL_MS = int(
    float(os.environ.get("RP608_NSE_GATE2X_MED_VOL_DWELL_MS", "40000"))
)
GATE2X_REVERSAL_LOW_VOL_DWELL_MS = int(
    float(os.environ.get("RP608_NSE_GATE2X_LOW_VOL_DWELL_MS", "90000"))
)
GATE2X_REVERSAL_IMMEDIATE_SURGES = {
    s.strip().upper()
    for s in os.environ.get("RP608_NSE_GATE2X_IMMEDIATE_VOL", "HIGH").split(",")
    if s.strip()
} or {"HIGH"}
# v1.3: ratchet floor + deep-giveback dwell scale.
GATE2X_RATCHET_PEAK_FRAC = float(os.environ.get("RP608_NSE_GATE2X_RATCHET_FRAC", "0.30"))
# Gate 2X r50 booking twin (2026-11-24): sensible MFE lock; PRIMARY/BASELINE stay 0.30.
GATE2X_RATCHET_PEAK_FRAC_R50 = float(
    os.environ.get("RP608_NSE_GATE2X_RATCHET_FRAC_R50", "0.50")
)
GATE2X_R50_BOOKING_PROTOCOL = "RP-608-GATE2X-R50-BOOKING-TWIN-001"
# Gate 2X floor-lock twin (2026-11-24): same ungated entry as BASELINE; exit at
# protect floor immediately (no COST_CLEAR HARD_FLOOR bleed). Authority 0%.
GATE2X_FLOOR_LOCK_TWIN_ENABLED = os.environ.get(
    "RP608_NSE_GATE2X_FLOOR_LOCK_TWIN", "1"
).strip().lower() not in {"0", "false", "no", "off"}
GATE2X_FLOOR_LOCK_RATCHET = float(
    os.environ.get("RP608_NSE_GATE2X_FLOOR_LOCK_RATCHET", "0.50")
)
GATE2X_FLOOR_LOCK_PROTOCOL = "RP-608-GATE2X-FLOOR-LOCK-TWIN-001"
GATE2X_DEEP_GIVEBACK_FRAC = float(os.environ.get("RP608_NSE_GATE2X_DEEP_GB_FRAC", "0.50"))
GATE2X_DEEP_GIVEBACK_DWELL_SCALE = float(
    os.environ.get("RP608_NSE_GATE2X_DEEP_GB_DWELL_SCALE", "0.40")
)
GATE2X_MFE_GIVEBACK_PROTOCOL = "RP-608-GATE2X-NSE-MFE-GIVEBACK-001"
# Gate 5 Adaptive TP (2026-11-21): match crypto — first TP frozen at entry;
# on touch extend +0.5×ATR (or +0.5% entry if ATR cold), lock SL to prior TP.
# Gate 2X Hold/Protect/Exit decides continue vs take profit. Env kill-switch.
ADAPTIVE_TP_ENABLED = os.environ.get("RP608_NSE_ADAPTIVE_TP", "1").strip().lower() not in {
    "0",
    "false",
    "no",
    "off",
}
ADAPTIVE_TP_EXTEND_ATR_MULT = float(os.environ.get("RP608_NSE_ADAPTIVE_TP_ATR_MULT", "0.5"))
ADAPTIVE_TP_MAX_EXTENSIONS = int(float(os.environ.get("RP608_NSE_ADAPTIVE_TP_MAX_EXT", "3")))
ADAPTIVE_TP_EXTEND_PCT = float(
    os.environ.get("RP608_NSE_ADAPTIVE_TP_PCT", "0.005")
)  # fallback step when atr_14 missing (0.5% of entry)
ADAPTIVE_TP_PROTOCOL = "RP-608-GATE5-NSE-ADAPTIVE-TP-001"
# Adaptive = Coralys/Gate2X intelligent trail (not blind ATR). Legacy latches off.
ADAPTIVE_PARTIAL_LATCH_ENABLED = os.environ.get(
    "RP608_NSE_ADAPTIVE_PARTIAL_LATCH", "0"
).strip().lower() in {"1", "true", "yes", "on"}
ADAPTIVE_DUMB_ATR_TRAIL = os.environ.get(
    "RP608_NSE_ADAPTIVE_DUMB_ATR_TRAIL", "0"
).strip().lower() in {"1", "true", "yes", "on"}
ADAPTIVE_TRAIL_ATR_MULT = float(os.environ.get("RP608_NSE_ADAPTIVE_TRAIL_ATR_MULT", "0.5"))
ADAPTIVE_TRAIL_ARM_RISK_FRAC = float(
    os.environ.get("RP608_NSE_ADAPTIVE_TRAIL_ARM_RISK_FRAC", "0.30")
)


def _nse_vol_surge_bucket(ratio: Optional[float]) -> str:
    """Same candle burst buckets as crypto/NSE pattern library (not OFX)."""
    if ratio is None:
        return "UNK"
    r = float(ratio)
    if r < 0.8:
        return "LOW"
    if r < 1.5:
        return "MED"
    return "HIGH"

# Brier References & Forecasts
ESTIMAND_A_TRAIN_PREVALENCE = 0.58
ESTIMAND_B_TRAIN_PREVALENCE = 0.512
FORECAST_LONG_PROB = 0.58
FORECAST_STANDASIDE_PROB = 0.38

# Official NSE Trading Holidays 2026 (Excludes Saturdays & Sundays)
NSE_2026_HOLIDAYS = {
    datetime(2026, 1, 26).date(),  # Republic Day
    datetime(2026, 3, 3).date(),   # Holi
    datetime(2026, 3, 20).date(),  # Id-Ul-Fitr
    datetime(2026, 3, 27).date(),  # Ram Navami
    datetime(2026, 3, 31).date(),  # Mahavir Jayanti
    datetime(2026, 4, 3).date(),   # Good Friday
    datetime(2026, 4, 14).date(),  # Dr. Ambedkar Jayanti
    datetime(2026, 5, 1).date(),   # Maharashtra Day
    datetime(2026, 5, 27).date(),  # Bakri Id
    datetime(2026, 6, 26).date(),  # Muharram
    datetime(2026, 8, 15).date(),  # Independence Day
    datetime(2026, 10, 2).date(),  # Mahatma Gandhi Jayanti
    datetime(2026, 10, 20).date(), # Dussehra
    datetime(2026, 11, 8).date(),  # Diwali Balipratipada
    datetime(2026, 11, 24).date(), # Guru Nanak Jayanti
    datetime(2026, 12, 25).date(), # Christmas
}

# Default Output Paths
DEFAULT_OUTPUT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "nse"
DEFAULT_CHECKPOINT_PATH = DEFAULT_OUTPUT_DIR / "rp608_nse_checkpoint.json"
DEFAULT_TELEMETRY_LOG_PATH = DEFAULT_OUTPUT_DIR / "rp608_nse_shadow_telemetry.jsonl"
DEFAULT_DECISION_CARDS_PATH = DEFAULT_OUTPUT_DIR / "rp608_nse_decision_cards.jsonl"
DEFAULT_TRADE_JOURNAL_PATH = DEFAULT_OUTPUT_DIR / "rp608_nse_trade_journal.jsonl"
# Gate2E BASELINE twin journal (crypto-style; never pooled with PRIMARY Estimand A).
DEFAULT_BASELINE_TWIN_DIR = DEFAULT_OUTPUT_DIR / "estimand_a_baseline_parallel_v1"
DEFAULT_TRADE_JOURNAL_BASELINE_PATH = (
    DEFAULT_BASELINE_TWIN_DIR / "rp608_nse_trade_journal.jsonl"
)
NSE_BOOK_PRIMARY = "NSE_ESTIMAND_A"
NSE_BOOK_BASELINE = "NSE_ESTIMAND_A_BASELINE_PARALLEL"
NSE_BOOK_G2X_R50 = "NSE_ESTIMAND_A_GATE2X_R50_PARALLEL"
NSE_BOOK_G2X_FLOOR_LOCK = "NSE_ESTIMAND_A_GATE2X_FLOOR_LOCK_PARALLEL"
# PRIMARY (Gate2E) + BASELINE (ungated r30) + Gate2X r50 + floor-lock twin.
# Floor-lock optional via RP608_NSE_GATE2X_FLOOR_LOCK_TWIN (default on).
_ESTIMAND_BOOK_KEYS_CORE = ("primary", "baseline", "gate2x_r50")
ESTIMAND_BOOK_KEYS = (
    _ESTIMAND_BOOK_KEYS_CORE + ("gate2x_floor_lock",)
    if GATE2X_FLOOR_LOCK_TWIN_ENABLED
    else _ESTIMAND_BOOK_KEYS_CORE
)
DEFAULT_OPPORTUNITY_LOG_PATH = DEFAULT_OUTPUT_DIR / "rp608_nse_opportunity_registry.jsonl"
DEFAULT_INSTRUMENT_MAP_CACHE = DEFAULT_OUTPUT_DIR / "kite_nse_instrument_map.json"
DEFAULT_KITE_READINESS_REPORT = DEFAULT_OUTPUT_DIR / "kite_readiness_report.json"
INSTRUMENT_FETCH_COOLDOWN_SEC = 300.0
INSTRUMENT_FETCH_MAX_ATTEMPTS = 3
# Kite historical soft limit ~3 req/s; stay under for 119-symbol same-day bootstrap.
HISTORICAL_FETCH_MIN_INTERVAL_SEC = 0.35
# After a transient hist failure (429/5xx/network), retry the symbol without waiting for next bar.
BOOTSTRAP_FAIL_COOLDOWN_SEC = 45.0


class KiteAuthError(Exception):
    """Kite HTTP 401/403 — access token invalid or expired."""


class KiteRateLimitError(Exception):
    """Kite HTTP 429 — too many requests; caller should back off and retry."""


def resolve_output_dir(output_dir: Optional[str | Path]) -> Path:
    """Resolves and validates an output directory, rejecting unsafe paths."""
    if output_dir is None:
        return DEFAULT_OUTPUT_DIR
    s = str(output_dir).strip()
    if not s:
        raise ValueError("FATAL: Unsafe output_dir: empty path is prohibited")
    p = Path(s)
    if not p.is_absolute():
        p = (WORKSPACE_DIR / p).resolve()
    else:
        p = p.resolve()
    if str(p) == "/" or p == Path("/"):
        raise ValueError("FATAL: Unsafe output_dir: cannot use root directory '/'")
    if p == WORKSPACE_DIR.resolve():
        raise ValueError("FATAL: Unsafe output_dir: cannot use workspace root directly")
    if p.exists() and p.is_file():
        raise ValueError(f"FATAL: Unsafe output_dir: {p} is an existing regular file, not a directory")
    return p


def compute_payload_checksum(data: Dict[str, Any]) -> str:
    """Computes a deterministic SHA-256 hash of a dictionary excluding 'checksum'."""
    d = {k: v for k, v in data.items() if k != "checksum"}
    canonical_json = json.dumps(d, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical_json.encode("utf-8")).hexdigest()


@dataclass
class BarOHLCV:
    timestamp_utc_ms: int
    open: float
    high: float
    low: float
    close: float
    volume: float
    is_completed: bool = True
    source: str = "kite_live"  # "kite_live" | "persisted_live" | "kite_historical_same_day"


@dataclass
class TopOfBookQuote:
    timestamp_utc_ms: int
    bid_price: float
    ask_price: float
    bid_qty: float = 0.0
    ask_qty: float = 0.0

    @property
    def mid_price(self) -> float:
        return (self.bid_price + self.ask_price) / 2.0

    @property
    def spread_bps(self) -> float:
        if self.mid_price <= 0:
            return 0.0
        return ((self.ask_price - self.bid_price) / self.mid_price) * 10000.0

    @property
    def half_spread_bps(self) -> float:
        return self.spread_bps / 2.0


@dataclass
class NSECheckpointState:
    schema_version: str = "rp608_nse_shadow_v1"
    protocol_id: str = PROTOCOL_ID
    status: str = "INITIALIZING"
    window_start_utc_ms: int = REGISTERED_WINDOW_START_UTC_MS
    last_heartbeat_utc_ms: int = 0
    last_heartbeat_iso: str = ""
    session_idx: int = 1
    last_durable_bar_idx: int = 0
    last_accounted_bar_idx: int = 0
    consecutive_outages: int = 0
    active_positions_count: int = 0
    completed_trades_count: int = 0
    directional_cards_count: int = 0
    unexecutable_drops_count: int = 0
    accounted_outage_ids: List[str] = field(default_factory=list)
    current_session_date: str = ""
    next_session_recs_emitted_for: str = ""  # IST session date already emitted
    checksum: str = ""

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


class KiteDataFeedAdapter:
    """
    Live market data adapter for the 119 NSE symbols via Zerodha Kite Connect.
    Strictly Read-Only (0.0% Order Authority).
    Uses batch /quote and /instruments/historical without order-placement capability.
    """
    SYMBOL_OVERRIDE_MAP = {
        "ICICIPRULI.NS": "ICICILIFE",
        "TATAMOTORS.NS": "TATAMOTORS",
    }

    def __init__(
        self,
        symbols: Tuple[str, ...] = NSE_119_SYMBOLS,
        credentials_file: Optional[Path] = None,
        timeout_sec: float = 6.0,
        instrument_cache_path: Optional[Path] = None,
        readiness_report_path: Optional[Path] = None,
    ):
        self.symbols = symbols
        self.credentials_file = credentials_file or (Path.home() / ".kite_credentials.json")
        self.timeout_sec = timeout_sec
        self.instrument_cache_path = Path(instrument_cache_path) if instrument_cache_path else DEFAULT_INSTRUMENT_MAP_CACHE
        self.readiness_report_path = Path(readiness_report_path) if readiness_report_path else DEFAULT_KITE_READINESS_REPORT
        self.creds = self._load_credentials()
        self.api_key = self.creds.get("api_key", "")
        self.access_token = self.creds.get("access_token", "")
        self._creds_mtime: Optional[float] = self._credentials_mtime()
        self.instrument_map: Dict[str, Tuple[int, str]] = {}
        self._instrument_map_source: Optional[str] = None
        self._last_instrument_fetch_attempt_mono: float = 0.0
        self._last_historical_fetch_mono: float = 0.0
        self.ticker_cache: Optional[KiteTickerQuoteCache] = None
        self._tick_sinks: List[TickSink] = []
        self._initialize_instrument_map()

    def register_tick_sink(self, sink: TickSink) -> None:
        """Research-only tee (OFX Stage 0D). Survives ticker restarts; must not raise."""
        self._tick_sinks.append(sink)
        if self.ticker_cache is not None:
            self.ticker_cache.add_tick_sink(sink)

    def _credentials_mtime(self) -> Optional[float]:
        try:
            return float(self.credentials_file.stat().st_mtime)
        except OSError:
            return None

    def reload_credentials_if_stale(self, *, force: bool = False) -> bool:
        """
        Hot-reload ~/.kite_credentials.json when the dashboard re-auths Kite.
        Restarts the WebSocket ticker if the access_token changed so the next
        session (and mid-session re-auth) does not keep a stale 403 token.
        Returns True when credentials were reloaded from disk.
        """
        mtime = self._credentials_mtime()
        if (
            not force
            and mtime is not None
            and self._creds_mtime is not None
            and mtime == self._creds_mtime
            and self.access_token
        ):
            return False

        new_creds = self._load_credentials()
        new_key = str(new_creds.get("api_key", "") or "")
        new_token = str(new_creds.get("access_token", "") or "")
        changed = (new_key != self.api_key) or (new_token != self.access_token)
        self.creds = new_creds
        self.api_key = new_key
        self.access_token = new_token
        self._creds_mtime = mtime

        if changed:
            if self.ticker_cache is not None:
                self.stop_ticker()
                self.start_ticker()
            print(
                f"[+] Kite credentials reloaded from {self.credentials_file} "
                f"(token_len={len(self.access_token)}, ticker_restarted={self.ticker_cache is not None})",
                flush=True,
            )
        return True

    def start_ticker(self) -> None:
        """Start continuous Kite WebSocket L1 cache (1 connection, universe tokens)."""
        if self.ticker_cache is not None:
            return
        if not self.instrument_map or not self.api_key or not self.access_token:
            print("[!] Kite ticker not started (missing instrument map or credentials)", flush=True)
            return
        tokens = [tok for tok, _ in self.instrument_map.values()]
        self.ticker_cache = KiteTickerQuoteCache(
            api_key=self.api_key,
            access_token=self.access_token,
            instrument_tokens=tokens,
            max_quote_age_ms=60_000.0,
            tick_sinks=list(self._tick_sinks),
        )
        self.ticker_cache.start()
        print(
            f"[+] Kite WebSocket full-mode cache started for {len(tokens)} tokens "
            f"(L1+L1–L5 retained in memory; ofx_sinks={len(self._tick_sinks)}; authority=0.0%)",
            flush=True,
        )

    def stop_ticker(self) -> None:
        if self.ticker_cache is not None:
            self.ticker_cache.stop()
            self.ticker_cache = None

    def _load_credentials(self) -> Dict[str, str]:
        if not self.credentials_file.exists():
            return {}
        try:
            with open(self.credentials_file, "r", encoding="utf-8") as f:
                return json.load(f)
        except Exception:
            return {}

    def _apply_raw_tradingsymbol_map(self, raw_map: Dict[str, int]) -> Dict[str, Tuple[int, str]]:
        resolved: Dict[str, Tuple[int, str]] = {}
        for full_sym in self.symbols:
            tsym = self.SYMBOL_OVERRIDE_MAP.get(full_sym, full_sym.replace(".NS", ""))
            if tsym in raw_map:
                resolved[full_sym] = (int(raw_map[tsym]), tsym)
        return resolved

    def _save_instrument_map_cache(self, resolved: Dict[str, Tuple[int, str]], *, source: str) -> None:
        payload = {
            "protocol_id": "RP-608-NSE-SHADOW-001",
            "saved_utc": datetime.now(timezone.utc).isoformat(),
            "source": source,
            "n_symbols": len(resolved),
            "instruments": {
                sym: {"instrument_token": tok, "tradingsymbol": tsym}
                for sym, (tok, tsym) in resolved.items()
            },
        }
        try:
            self.instrument_cache_path.parent.mkdir(parents=True, exist_ok=True)
            tmp = self.instrument_cache_path.with_suffix(self.instrument_cache_path.suffix + ".tmp")
            with open(tmp, "w", encoding="utf-8") as fh:
                json.dump(payload, fh, indent=2)
                fh.write("\n")
                fh.flush()
                os.fsync(fh.fileno())
            os.replace(tmp, self.instrument_cache_path)
        except Exception as e:
            print(f"[!] Warning: Failed to persist instrument map cache: {e}", file=sys.stderr)

    def _load_instrument_map_cache(self) -> Dict[str, Tuple[int, str]]:
        path = self.instrument_cache_path
        if not path.exists():
            return {}
        try:
            doc = json.loads(path.read_text(encoding="utf-8"))
            instruments = doc.get("instruments") or {}
            resolved: Dict[str, Tuple[int, str]] = {}
            for full_sym in self.symbols:
                row = instruments.get(full_sym)
                if not isinstance(row, dict):
                    continue
                tok = row.get("instrument_token")
                tsym = row.get("tradingsymbol")
                if tok is None or not tsym:
                    continue
                resolved[full_sym] = (int(tok), str(tsym))
            return resolved
        except Exception as e:
            print(f"[!] Warning: Failed to load instrument map cache: {e}", file=sys.stderr)
            return {}

    def _seed_instrument_map_from_readiness(self) -> Dict[str, Tuple[int, str]]:
        """Build map from prior kite_readiness_report all_quotes (no live instruments dump)."""
        path = self.readiness_report_path
        if not path.exists():
            alt = path.parent / "test_run" / "kite_readiness_report.json"
            path = alt if alt.exists() else path
        if not path.exists():
            return {}
        try:
            doc = json.loads(path.read_text(encoding="utf-8"))
            quotes = doc.get("all_quotes") or doc.get("sample_quotes") or []
            if not isinstance(quotes, list):
                return {}
            by_symbol = {
                str(q.get("symbol")): q
                for q in quotes
                if isinstance(q, dict) and q.get("symbol") and q.get("tradingsymbol") and q.get("instrument_token") is not None
            }
            resolved: Dict[str, Tuple[int, str]] = {}
            for full_sym in self.symbols:
                row = by_symbol.get(full_sym)
                if not row:
                    continue
                resolved[full_sym] = (int(row["instrument_token"]), str(row["tradingsymbol"]))
            return resolved
        except Exception as e:
            print(f"[!] Warning: Failed to seed instrument map from readiness report: {e}", file=sys.stderr)
            return {}

    def _fetch_instruments_csv_raw_map(self) -> Dict[str, int]:
        """Fetch Kite NSE instruments CSV with short retry/backoff on 429."""
        import csv
        import io

        url = "https://api.kite.trade/instruments/NSE"
        headers = {"User-Agent": "ChronoSentiment-RP608-Shadow"}
        # Prefer authenticated fetch when available (same CSV; better rate class on some paths).
        if self.api_key and self.access_token:
            headers.update(
                {
                    "X-Kite-Version": "3",
                    "Authorization": f"token {self.api_key}:{self.access_token}",
                }
            )

        last_err: Optional[BaseException] = None
        for attempt in range(1, INSTRUMENT_FETCH_MAX_ATTEMPTS + 1):
            req = urllib.request.Request(url, headers=headers)
            try:
                with urllib.request.urlopen(req, timeout=20) as resp:
                    content = resp.read().decode("utf-8")
                reader = csv.DictReader(io.StringIO(content))
                raw_map: Dict[str, int] = {}
                for row in reader:
                    if row.get("segment") == "NSE" and row.get("instrument_type") == "EQ":
                        tsym = row.get("tradingsymbol", "").strip()
                        tok = int(row.get("instrument_token"))
                        raw_map[tsym] = tok
                return raw_map
            except urllib.error.HTTPError as e:
                last_err = e
                if e.code == 429 and attempt < INSTRUMENT_FETCH_MAX_ATTEMPTS:
                    sleep_s = min(30.0, 2.0 ** attempt)
                    print(
                        f"[!] Kite instruments 429 (attempt {attempt}/{INSTRUMENT_FETCH_MAX_ATTEMPTS}); "
                        f"backoff {sleep_s:.1f}s",
                        file=sys.stderr,
                    )
                    time.sleep(sleep_s)
                    continue
                raise
            except Exception as e:
                last_err = e
                raise
        if last_err:
            raise last_err
        return {}

    def _initialize_instrument_map(self, *, force_network: bool = False):
        """
        Resolve NSE instrument tokens for the 119-symbol universe.

        Order:
          1) Live Kite instruments CSV (when forced or map empty and cooldown elapsed)
          2) Durable on-disk cache
          3) Seed from kite_readiness_report.json all_quotes
        """
        now_mono = time.monotonic()
        if (
            self.instrument_map
            and not force_network
        ):
            return

        can_network = force_network or (
            (now_mono - self._last_instrument_fetch_attempt_mono) >= INSTRUMENT_FETCH_COOLDOWN_SEC
            or self._last_instrument_fetch_attempt_mono == 0.0
        )

        if can_network:
            self._last_instrument_fetch_attempt_mono = now_mono
            try:
                raw_map = self._fetch_instruments_csv_raw_map()
                resolved = self._apply_raw_tradingsymbol_map(raw_map)
                if len(resolved) >= max(1, int(0.9 * len(self.symbols))):
                    self.instrument_map = resolved
                    self._instrument_map_source = "kite_instruments_csv"
                    self._save_instrument_map_cache(resolved, source="kite_instruments_csv")
                    print(
                        f"[+] Instrument map loaded from Kite CSV: {len(resolved)}/{len(self.symbols)}",
                        flush=True,
                    )
                    return
                print(
                    f"[!] Warning: Kite CSV mapped only {len(resolved)}/{len(self.symbols)}; trying cache/seed",
                    file=sys.stderr,
                )
            except Exception as e:
                print(f"[!] Warning: Failed to fetch Kite instruments map: {e}", file=sys.stderr)

        cached = self._load_instrument_map_cache()
        if cached:
            self.instrument_map = cached
            self._instrument_map_source = "disk_cache"
            print(
                f"[+] Instrument map restored from cache: {len(cached)}/{len(self.symbols)} "
                f"({self.instrument_cache_path})",
                flush=True,
            )
            return

        seeded = self._seed_instrument_map_from_readiness()
        if seeded:
            self.instrument_map = seeded
            self._instrument_map_source = "readiness_report_seed"
            self._save_instrument_map_cache(seeded, source="readiness_report_seed")
            print(
                f"[+] Instrument map seeded from readiness report: {len(seeded)}/{len(self.symbols)}",
                flush=True,
            )
            return

        self.instrument_map = {}
        self._instrument_map_source = None
        print(
            "[!] Warning: Instrument map empty after network/cache/seed; symbols will be UNMAPPED_SYMBOL",
            file=sys.stderr,
        )

    def fetch_all_symbols(self) -> Dict[str, Dict[str, Any]]:
        """
        Fetches full quotes for all 119 symbols in a SINGLE batch HTTP request.
        Returns: {symbol: {"bars": [BarOHLCV], "quote": TopOfBookQuote, "error": Optional[str]}}
        """
        self.reload_credentials_if_stale()
        if not self.access_token:
            return {s: {"bars": None, "quote": None, "error": "MISSING_KITE_ACCESS_TOKEN"} for s in self.symbols}

        if not self.instrument_map:
            self._initialize_instrument_map()

        # Build batch query: i=NSE:TRADINGSYMBOL
        kite_keys = [f"NSE:{tsym}" for _, (_, tsym) in self.instrument_map.items()]
        query_str = urllib.parse.urlencode({"i": kite_keys}, doseq=True)
        url = f"https://api.kite.trade/quote?{query_str}"

        headers = {
            "X-Kite-Version": "3",
            "Authorization": f"token {self.api_key}:{self.access_token}",
            "User-Agent": "ChronoSentiment-RP608-Shadow"
        }

        t_req_utc = int(time.time() * 1000)
        data: Optional[Dict[str, Any]] = None

        def _quote_once(auth_headers: Dict[str, str]) -> Dict[str, Any]:
            req = urllib.request.Request(url, headers=auth_headers)
            with urllib.request.urlopen(req, timeout=self.timeout_sec) as resp:
                if resp.status != 200:
                    raise urllib.error.HTTPError(url, resp.status, f"HTTP_{resp.status}", resp.headers, None)
                return json.loads(resp.read().decode("utf-8")).get("data", {}) or {}

        try:
            data = _quote_once(headers)
        except urllib.error.HTTPError as e:
            err_code = f"KITE_HTTP_{e.code}"
            # Daily Kite token rotate via dashboard: pick up new file and retry once.
            if e.code in (401, 403) and self.reload_credentials_if_stale(force=True) and self.access_token:
                headers = {
                    **headers,
                    "Authorization": f"token {self.api_key}:{self.access_token}",
                }
                try:
                    data = _quote_once(headers)
                except Exception:
                    return {s: {"bars": None, "quote": None, "error": err_code} for s in self.symbols}
            else:
                return {s: {"bars": None, "quote": None, "error": err_code} for s in self.symbols}
        except Exception as e:
            return {s: {"bars": None, "quote": None, "error": f"EXCEPTION_{type(e).__name__}"} for s in self.symbols}

        if data is None:
            return {s: {"bars": None, "quote": None, "error": "KITE_HTTP_EMPTY"} for s in self.symbols}

        results = {}
        for full_sym in self.symbols:
            mapping = self.instrument_map.get(full_sym)
            if not mapping:
                results[full_sym] = {"bars": None, "quote": None, "error": "UNMAPPED_SYMBOL"}
                continue

            tok, tsym = mapping
            k = f"NSE:{tsym}"
            q_data = data.get(k)
            if not q_data:
                results[full_sym] = {"bars": None, "quote": None, "error": "QUOTE_NOT_RETURNED"}
                continue

            last_price = float(q_data.get("last_price", 0.0))
            ohlc = q_data.get("ohlc", {})
            o = float(ohlc.get("open", last_price))
            h = float(ohlc.get("high", last_price))
            l = float(ohlc.get("low", last_price))
            c = float(ohlc.get("close", last_price))
            v = float(q_data.get("volume", 0.0))

            # Quote batch returns day OHLC + LTP — NOT a completed 15m candle.
            # Mark incomplete so evaluate_symbol_bar upserts one slot instead of
            # flooding price_history (which zeroed traj15 / locked dashboard G4).
            bar = BarOHLCV(
                timestamp_utc_ms=t_req_utc,
                open=o,
                high=h,
                low=l,
                close=last_price if last_price > 0 else c,
                volume=v,
                is_completed=False,
                source="kite_quote_snapshot",
            )

            # L1 Top-of-Book depth
            depth = q_data.get("depth", {})
            buys = depth.get("buy", [])
            sells = depth.get("sell", [])

            best_bid = float(buys[0].get("price", 0.0)) if buys else 0.0
            best_ask = float(sells[0].get("price", 0.0)) if sells else 0.0
            bid_qty = float(buys[0].get("quantity", 0.0)) if buys else 0.0
            ask_qty = float(sells[0].get("quantity", 0.0)) if sells else 0.0

            # Prefer fresh Kite WebSocket L1 when available (G4 freshness); else REST depth.
            ws_l1 = self.ticker_cache.get_l1(tok) if self.ticker_cache is not None else None
            if ws_l1 and float(ws_l1.get("best_bid") or 0) > 0 and float(ws_l1.get("best_ask") or 0) > 0:
                best_bid = float(ws_l1["best_bid"])
                best_ask = float(ws_l1["best_ask"])
                bid_qty = float(ws_l1.get("bid_qty") or 0.0)
                ask_qty = float(ws_l1.get("ask_qty") or 0.0)
                quote_ts = int(ws_l1.get("receipt_utc_ms") or t_req_utc)
                if float(ws_l1.get("last_price") or 0) > 0:
                    last_price = float(ws_l1["last_price"])
                    bar = BarOHLCV(
                        timestamp_utc_ms=quote_ts,
                        open=o,
                        high=h,
                        low=l,
                        close=last_price,
                        volume=float(ws_l1.get("volume") or v),
                        is_completed=False,
                        source="kite_quote_snapshot",
                    )
            else:
                quote_ts = t_req_utc

            # During market hours, valid top-of-book quotes exist
            if best_bid > 0 and best_ask > 0:
                quote = TopOfBookQuote(
                    timestamp_utc_ms=quote_ts,
                    bid_price=best_bid,
                    ask_price=best_ask,
                    bid_qty=bid_qty,
                    ask_qty=ask_qty
                )
            else:
                # Market closed or depth empty: record quote with 0 bid/ask
                quote = TopOfBookQuote(
                    timestamp_utc_ms=t_req_utc,
                    bid_price=best_bid,
                    ask_price=best_ask,
                    bid_qty=bid_qty,
                    ask_qty=ask_qty
                )

            results[full_sym] = {
                "bars": [bar],
                "quote": quote,
                "error": None
            }

        return results

    def _throttle_historical_fetch(self) -> None:
        elapsed = time.monotonic() - float(self._last_historical_fetch_mono or 0.0)
        wait_s = HISTORICAL_FETCH_MIN_INTERVAL_SEC - elapsed
        if wait_s > 0:
            time.sleep(wait_s)
        self._last_historical_fetch_mono = time.monotonic()

    def fetch_historical_15m_bars(
        self,
        symbol: str,
        from_utc: datetime,
        to_utc: datetime
    ) -> List[BarOHLCV]:
        """
        Fetches historical 15m candles from Kite Connect for a given symbol within [from_utc, to_utc].
        Strictly Read-Only (0.0% Order Authority).
        Returns list of BarOHLCV marked with source='kite_historical_same_day'.

        Raises:
          KiteAuthError on HTTP 401/403 (after one forced credential reload retry).
          KiteRateLimitError on HTTP 429.
        """
        self.reload_credentials_if_stale()
        if not self.access_token or not self.api_key:
            raise KiteAuthError("MISSING_KITE_ACCESS_TOKEN")

        if not self.instrument_map:
            self._initialize_instrument_map()

        mapping = self.instrument_map.get(symbol)
        if not mapping:
            return []

        token, tsym = mapping
        ist_offset = timedelta(hours=5, minutes=30)
        from_ist = from_utc + ist_offset
        to_ist = to_utc + ist_offset

        from_str = from_ist.strftime("%Y-%m-%d %H:%M:%S")
        to_str = to_ist.strftime("%Y-%m-%d %H:%M:%S")

        query_str = urllib.parse.urlencode({
            "from": from_str,
            "to": to_str
        })
        url = f"https://api.kite.trade/instruments/historical/{token}/15minute?{query_str}"

        def _hist_once() -> List[Any]:
            self._throttle_historical_fetch()
            headers = {
                "X-Kite-Version": "3",
                "Authorization": f"token {self.api_key}:{self.access_token}",
                "User-Agent": "ChronoSentiment-RP608-Shadow"
            }
            req = urllib.request.Request(url, headers=headers)
            with urllib.request.urlopen(req, timeout=self.timeout_sec) as resp:
                if resp.status != 200:
                    raise urllib.error.HTTPError(url, int(resp.status), f"HTTP_{resp.status}", resp.headers, None)
                body = json.loads(resp.read().decode("utf-8"))
                return list((body.get("data") or {}).get("candles") or [])

        try:
            candles = _hist_once()
        except urllib.error.HTTPError as e:
            if e.code in (401, 403):
                self.reload_credentials_if_stale(force=True)
                if not self.access_token:
                    raise KiteAuthError(f"KITE_HTTP_{e.code}") from e
                try:
                    candles = _hist_once()
                except urllib.error.HTTPError as e2:
                    if e2.code in (401, 403):
                        raise KiteAuthError(f"KITE_HTTP_{e2.code}") from e2
                    if e2.code == 429:
                        raise KiteRateLimitError("KITE_HTTP_429") from e2
                    raise
            elif e.code == 429:
                raise KiteRateLimitError("KITE_HTTP_429") from e
            else:
                raise
        except KiteAuthError:
            raise
        except KiteRateLimitError:
            raise

        bars = []
        for c in candles:
            if not isinstance(c, list) or len(c) < 6:
                continue
            c_time_raw = c[0]
            try:
                if isinstance(c_time_raw, str):
                    c_dt = datetime.fromisoformat(c_time_raw)
                else:
                    continue
                c_dt_utc = c_dt.astimezone(timezone.utc)
                ts_ms = int(c_dt_utc.timestamp() * 1000)
                bars.append(
                    BarOHLCV(
                        timestamp_utc_ms=ts_ms,
                        open=float(c[1]),
                        high=float(c[2]),
                        low=float(c[3]),
                        close=float(c[4]),
                        volume=float(c[5]),
                        is_completed=True,
                        source="kite_historical_same_day"
                    )
                )
            except Exception:
                continue

        return bars


class RP608NSEShadowDaemon:
    """
    Main prospective observer daemon for the 119 NSE symbols.
    """
    def __init__(
        self,
        output_dir: Optional[str | Path] = None,
        feed_adapter: Optional[Any] = None,
        single_bar_test: bool = False,
        reset_outages: bool = False
    ):
        self.output_dir = resolve_output_dir(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.reset_outages = reset_outages

        self.checkpoint_path = self.output_dir / "rp608_nse_checkpoint.json"
        self.telemetry_path = self.output_dir / "rp608_nse_shadow_telemetry.jsonl"
        self.decision_cards_path = self.output_dir / "rp608_nse_decision_cards.jsonl"
        self.trade_journal_path = self.output_dir / "rp608_nse_trade_journal.jsonl"
        # Gate2E BASELINE twin (crypto parity): ungated control; never pooled with PRIMARY.
        self.baseline_twin_dir = self.output_dir / "estimand_a_baseline_parallel_v1"
        self.baseline_twin_dir.mkdir(parents=True, exist_ok=True)
        self.trade_journal_baseline_path = (
            self.baseline_twin_dir / "rp608_nse_trade_journal.jsonl"
        )
        # Gate 2X r50 booking twin: same ungated entry as BASELINE; ratchet 0.50.
        self.g2x_r50_twin_dir = self.output_dir / "estimand_a_gate2x_r50_parallel_v1"
        self.g2x_r50_twin_dir.mkdir(parents=True, exist_ok=True)
        self.trade_journal_g2x_r50_path = (
            self.g2x_r50_twin_dir / "rp608_nse_trade_journal.jsonl"
        )
        # Gate 2X floor-lock twin: ungated entry; immediate protect-floor exit.
        self.g2x_floor_lock_twin_dir = (
            self.output_dir / "estimand_a_gate2x_floorlock_parallel_v1"
        )
        self.trade_journal_g2x_floor_lock_path = (
            self.g2x_floor_lock_twin_dir / "rp608_nse_trade_journal.jsonl"
        )
        if GATE2X_FLOOR_LOCK_TWIN_ENABLED:
            self.g2x_floor_lock_twin_dir.mkdir(parents=True, exist_ok=True)
        self._primary_gate_mode = pattern_entry_gate_mode()
        self._baseline_gate_mode = pattern_entry_gate_mode_baseline()
        print(
            f"[gate2e_twin] PRIMARY book={NSE_BOOK_PRIMARY} gate={self._primary_gate_mode!r} "
            f"BASELINE book={NSE_BOOK_BASELINE} gate={self._baseline_gate_mode!r} "
            f"dir={self.baseline_twin_dir}",
            flush=True,
        )
        print(
            f"[gate2x_r50_twin] book={NSE_BOOK_G2X_R50} gate={self._baseline_gate_mode!r} "
            f"ratchet={GATE2X_RATCHET_PEAK_FRAC_R50} dir={self.g2x_r50_twin_dir} "
            f"(PRIMARY/BASELINE r30 unchanged)",
            flush=True,
        )
        if GATE2X_FLOOR_LOCK_TWIN_ENABLED:
            print(
                f"[gate2x_floorlock_twin] book={NSE_BOOK_G2X_FLOOR_LOCK} "
                f"gate={self._baseline_gate_mode!r} ratchet={GATE2X_FLOOR_LOCK_RATCHET} "
                f"dir={self.g2x_floor_lock_twin_dir} "
                f"(immediate protect floor; PRIMARY/BASELINE/r50 unchanged)",
                flush=True,
            )
        self.opportunity_path = self.output_dir / "rp608_nse_opportunity_registry.jsonl"
        self.evaluations_path = self.output_dir / "rp608_nse_evaluations.jsonl"
        self.latest_evaluations_path = self.output_dir / "rp608_nse_latest_evaluations.json"
        # Gate2E cold-start ABSTAIN_INSUFFICIENT observe (PRIMARY; stamp-only).
        self.gate2e_shadow_insufficient_path = (
            self.output_dir / "gate2e_shadow_insufficient.jsonl"
        )
        self.bar_history_path = self.output_dir / "rp608_nse_bar_history.json"
        self.pending_evaluations_path = self.output_dir / "rp608_nse_pending_evaluations.json"
        self.next_session_recs_path = self.output_dir / "rp608_nse_next_session_recommendations.json"
        self.l1_feed_health_path = self.output_dir / "rp608_nse_l1_feed_health.json"
        self.of_book_intel_dir = self.output_dir / "of_book_intelligence_v1"
        self.ofx_tick_dir = self.output_dir / "ofx_tick_capture_v1"
        # Adapter-side Coralys inputs (prior + pre-open + phase). Crates untouched.
        self.coralys_input_store = CoralysInputContextStore(self.output_dir)
        self._coralys_prior_logged_for: Optional[str] = None
        # Closed Estimand A trade_ids already journaled (prevents midbar re-entry churn).
        self._journaled_trade_ids: set[str] = set()
        self._journaled_trade_ids_baseline: set[str] = set()
        self._journaled_trade_ids_g2x_r50: set[str] = set()
        self._journaled_trade_ids_g2x_floor_lock: set[str] = set()
        # Pattern gate: session-level "prior stop in similar ToA/traj family" (NOT symbol bans).
        self._pattern_session_date: Optional[str] = None
        self._pattern_session_stop_families: set[str] = set()
        self._trade_pattern_library: Optional[Dict[str, Any]] = None
        self._trade_pattern_library_mtime: float = 0.0
        # TICK_PATH: durable close arms signal_bar; fills happen on WS L1 ticks (not batch-at-close).
        self._estimand_a_lock = threading.Lock()
        self._estimand_a_armed_signal_bar: int = 0
        self._token_to_symbol: Dict[int, str] = {}
        self._estimand_a_tick_opens_since_log: int = 0
        self._estimand_a_last_persist_mono: float = 0.0
        # Prior WS packet per symbol for volume/LT trade-like authenticity (false-print filter).
        self._estimand_a_prev_ws_tick: Dict[str, Dict[str, Any]] = {}
        # Last bar decision armed for OF / OFX per-tick open attempts (books stay separate).
        self._paper_armed_eval_by_symbol: Dict[str, Dict[str, Any]] = {}

        self.feed_adapter = feed_adapter or KiteDataFeedAdapter(NSE_119_SYMBOLS)
        self.symbols = getattr(self.feed_adapter, "symbols", NSE_119_SYMBOLS)
        self.single_bar_test = single_bar_test
        self.running = False
        self.stop_event: Optional[asyncio.Event] = None

        self.exchange_holidays = set(NSE_2026_HOLIDAYS)
        self.current_session_date: Optional[Any] = None
        self._symbol_last_bootstrap_bar: Dict[str, int] = {}
        # Transient hist failures (429/auth/network): cooldown then retry without waiting for next bar.
        self._symbol_bootstrap_fail_mono: Dict[str, float] = {}
        # Price history ring buffer per symbol loaded from durable storage: {symbol: [BarOHLCV]}
        self.price_history: Dict[str, List[BarOHLCV]] = self.load_bar_history()
        purged = self._purge_quote_snapshot_spam()
        if purged:
            print(f"[bar_history] purged {purged} quote-snapshot/dupe bars after load", flush=True)
        # Forward outcome pending queue and active trades restored from durable storage across restarts:
        (
            self.pending_directional_evals,
            self.active_trades,
            self.active_trades_baseline,
            self.active_trades_g2x_r50,
            self.active_trades_g2x_floor_lock,
        ) = self.load_pending_evaluations()
        self._journaled_trade_ids = self._load_journaled_trade_ids(self.trade_journal_path)
        self._journaled_trade_ids_baseline = self._load_journaled_trade_ids(
            self.trade_journal_baseline_path
        )
        self._journaled_trade_ids_g2x_r50 = self._load_journaled_trade_ids(
            self.trade_journal_g2x_r50_path
        )
        self._journaled_trade_ids_g2x_floor_lock = self._load_journaled_trade_ids(
            self.trade_journal_g2x_floor_lock_path
        )
        self._bootstrap_pattern_session_stop_families()
        # Scrip-level decision ledger cache: {symbol: evaluation_record}
        self.last_evaluations: Dict[str, Dict[str, Any]] = {}
        # ATR/structure feature bars (may include prior sessions). Never used for traj15.
        self._atr_feature_cache: Dict[str, List[Any]] = {}
        self._atr_feature_cache_bar_idx: int = -1

        # NSE OF PAPER ledger (0% authority; independent of Estimand A journal)
        self.of_paper = NseOfPaperLedger(
            self.output_dir,
            friction_fn=calc_nse_of_paper_accounting,
        )
        # Book-intelligence Phase 1: durable L1–L5 snapshots (OF-rule inert)
        self.of_book_depth = NseOfBookDepthHistoryJournal(
            self.of_book_intel_dir,
            min_interval_ms=5_000,
            enabled=True,
        )
        # OFX Stage 0D: inferred-aggressor tick tee on the same WS (OF-rule inert)
        self.ofx_tick_journal = OfxNseInferredAggressorTickJournal(
            self.ofx_tick_dir,
            enabled=ofx_tick_capture_enabled(default=True),
        )
        # OFX NSE volume PAPER (DERIVED_VALIDATED; parallel to L1 OF paper; baseline inert)
        self.ofx_nse_volume_buf = global_nse_volume_flow_buffer()
        self.ofx_nse_volume_paper = OfxNseVolumePaperManager(self.ofx_tick_dir)
        self.ofx_tick_journal.register_row_listener(self.ofx_nse_volume_buf.ingest_row)
        self._active_session_window = None  # set by is_ist_session_active

        self.state = self.initialize_or_resume_state()
        if getattr(self.state, "current_session_date", ""):
            self.current_session_date = self.state.current_session_date
        if ENTRY_PATH == "TICK_PATH":
            self._estimand_a_armed_signal_bar = int(
                getattr(self.state, "last_durable_bar_idx", 0) or 0
            )

    def initialize_or_resume_state(self) -> NSECheckpointState:
        """Loads state from durable checkpoint with fail-closed tamper checking."""
        now_ms = int(time.time() * 1000)
        now_iso = datetime.now(timezone.utc).isoformat()

        if not self.checkpoint_path.exists():
            st = NSECheckpointState(
                status="ACTIVE_PILOT",
                last_heartbeat_utc_ms=now_ms,
                last_heartbeat_iso=now_iso
            )
            st.checksum = compute_payload_checksum(st.to_dict())
            self.atomic_commit_checkpoint(st)
            return st

        try:
            with open(self.checkpoint_path, "r", encoding="utf-8") as f:
                data = json.load(f)
        except Exception as e:
            raise RuntimeError(f"FATAL: Checkpoint corruption reading {self.checkpoint_path}: {e}")

        recorded_checksum = data.get("checksum", "")
        calculated_checksum = compute_payload_checksum(data)
        if recorded_checksum != calculated_checksum:
            raise RuntimeError(
                f"FATAL: Checkpoint tamper/corruption in {self.checkpoint_path}! "
                f"Recorded: {recorded_checksum}, Calculated: {calculated_checksum}"
            )

        # Recovery accounting
        last_hb = data.get("last_heartbeat_utc_ms", 0)
        accounted_outages = list(data.get("accounted_outage_ids", []))
        last_accounted_idx = data.get("last_accounted_bar_idx", 0)
        consecutive_outages = data.get("consecutive_outages", 0)
        drops_count = data.get("unexecutable_drops_count", 0)

        if getattr(self, "reset_outages", False):
            consecutive_outages = 0
            data["consecutive_outages"] = 0
            data["status"] = "ACTIVE_PILOT"

        # Pre-start exclusion: gaps before REGISTERED_WINDOW_START_UTC_MS do not count as missed bars
        effective_start_ms = max(last_hb, REGISTERED_WINDOW_START_UTC_MS)
        if last_hb > 0 and now_ms > effective_start_ms and not getattr(self, "reset_outages", False):
            downtime_ms = now_ms - effective_start_ms
            missed_intervals = downtime_ms // FIFTEEN_MINUTES_MS
            missed_intervals = downtime_ms // FIFTEEN_MINUTES_MS
            if missed_intervals > 0:
                outage_id = f"nse_outage_{effective_start_ms}_{now_ms}"
                if outage_id not in accounted_outages:
                    accounted_outages.append(outage_id)
                    consecutive_outages += missed_intervals
                    drops_count += (missed_intervals * len(NSE_119_SYMBOLS))
                    last_accounted_idx += missed_intervals
                    self.append_opportunity_drop(
                        drop_type="NSE_OUTAGE_DOWNTIME_MISSED_BARS",
                        reason=f"Process downtime of {downtime_ms/1000.0:.1f}s caused {missed_intervals} missed 15m intervals",
                        outage_id=outage_id,
                        missed_bars=missed_intervals
                    )

        if consecutive_outages >= MAX_CONSECUTIVE_OUTAGES_FAIL_CLOSED and not self.single_bar_test:
            # Protocol TG-4 fail closed
            data["status"] = "HALTED_CONSECUTIVE_OUTAGES"
            st = NSECheckpointState(**data)
            st.checksum = compute_payload_checksum(st.to_dict())
            self.atomic_commit_checkpoint(st)
            raise RuntimeError(
                f"FATAL: Protocol TG-4 violation: {consecutive_outages} consecutive missing bars >= {MAX_CONSECUTIVE_OUTAGES_FAIL_CLOSED}"
            )

        data["last_heartbeat_utc_ms"] = now_ms
        data["last_heartbeat_iso"] = now_iso
        data["accounted_outage_ids"] = accounted_outages
        data["last_accounted_bar_idx"] = last_accounted_idx
        data["unexecutable_drops_count"] = drops_count

        st = NSECheckpointState(**data)
        st.checksum = compute_payload_checksum(st.to_dict())
        self.atomic_commit_checkpoint(st)
        return st

    def write_l1_feed_health(self) -> None:
        """Persist live ticker-cache stats for the dashboard continuous-feed tile.

        Observational only — does not affect entry/exit eligibility.
        """
        now_utc = datetime.now(timezone.utc)
        adapter = self.feed_adapter
        cache = getattr(adapter, "ticker_cache", None)
        # Stuck "connected" sockets with no fresh full-mode packets lock G5
        # (zero/stale L1). Bounce the ticker thread only — no credential changes.
        if cache is not None and hasattr(cache, "ensure_fresh"):
            try:
                cache.ensure_fresh(max_age_sec=90.0)
            except Exception as exc:
                print(f"[l1_feed_health] ticker ensure_fresh skipped: {exc}")
        stats: Dict[str, Any] = {}
        if cache is not None and hasattr(cache, "stats"):
            try:
                stats = dict(cache.stats() or {})
            except Exception:
                stats = {}
        payload = {
            "protocol_id": "RP-608-NSE-L1-FEED-HEALTH-001",
            "timestamp_iso": now_utc.isoformat(),
            "timestamp_utc_ms": int(now_utc.timestamp() * 1000),
            "session_date": getattr(self.state, "current_session_date", "") or "",
            "durable_bar": int(getattr(self.state, "last_durable_bar_idx", 0) or 0),
            "accounted_bar": int(getattr(self.state, "last_accounted_bar_idx", 0) or 0),
            "connected": bool(stats.get("connected")),
            "subscribed_tokens": int(stats.get("subscribed_tokens") or 0),
            "cached_tokens": int(stats.get("cached_tokens") or 0),
            "packets": int(stats.get("packets") or 0),
            "last_packet_age_sec": stats.get("last_packet_age_sec"),
            "last_packet_utc": (
                None
                if stats.get("last_packet_age_sec") is None
                else (now_utc - timedelta(seconds=float(stats["last_packet_age_sec"]))).isoformat()
            ),
            "ticker_cache_present": cache is not None,
        }
        try:
            self.atomic_commit_json(self.l1_feed_health_path, payload)
        except Exception as exc:
            print(f"[l1_feed_health] write skipped: {exc}")
        # Observational book-intel journal (does not affect OF / paper / Arm A).
        try:
            self._sample_of_book_depth_history()
        except Exception as exc:
            print(f"[of_book_intel] depth sample skipped: {exc}")

    def _token_to_symbol_map(self) -> Dict[int, str]:
        adapter = self.feed_adapter
        imap = getattr(adapter, "instrument_map", None) or {}
        out: Dict[int, str] = {}
        if isinstance(imap, dict):
            for sym, pair in imap.items():
                try:
                    if isinstance(pair, (tuple, list)) and pair:
                        tok = int(pair[0])
                    elif isinstance(pair, dict):
                        tok = int(pair.get("instrument_token") or 0)
                    else:
                        continue
                except (TypeError, ValueError):
                    continue
                if tok > 0:
                    out[tok] = str(sym)
        return out

    def _sample_of_book_depth_history(self) -> None:
        """Append rate-limited L1–L5 snapshots from the live ticker cache."""
        journal = getattr(self, "of_book_depth", None)
        if journal is None:
            return
        adapter = self.feed_adapter
        cache = getattr(adapter, "ticker_cache", None)
        ticks = collect_ticks_by_symbol(cache, self._token_to_symbol_map())
        if not ticks:
            return
        journal.maybe_sample(ticks_by_symbol=ticks)

    def atomic_commit_checkpoint(self, state: NSECheckpointState):
        """Atomically persists state to disk via tmp file + replace + fsync."""
        d = state.to_dict()
        d["checksum"] = compute_payload_checksum(d)
        tmp_path = self.checkpoint_path.with_suffix(".tmp")
        try:
            with open(tmp_path, "w", encoding="utf-8") as f:
                json.dump(d, f, indent=2)
                f.flush()
                os.fsync(f.fileno())
            os.replace(tmp_path, self.checkpoint_path)
        finally:
            if tmp_path.exists():
                try:
                    tmp_path.unlink()
                except OSError:
                    pass
        # Best-effort companion artifact for dashboard continuous-feed status.
        try:
            self.write_l1_feed_health()
        except Exception:
            pass

    def atomic_commit_json(self, target_path: Path, data: Any):
        """Atomically persists arbitrary JSON data to disk via tmp file + replace + fsync."""
        # Unique tmp avoids WS-tick vs main-loop races on the same .tmp path.
        tmp_path = target_path.parent / (
            f".{target_path.name}.{os.getpid()}.{threading.get_ident()}.{time.time_ns()}.tmp"
        )
        try:
            with open(tmp_path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2)
                f.flush()
                os.fsync(f.fileno())
            os.replace(tmp_path, target_path)
        finally:
            if tmp_path.exists():
                try:
                    tmp_path.unlink()
                except OSError:
                    pass

    def load_bar_history(self, session_date: Optional[Any] = None) -> Dict[str, List[BarOHLCV]]:
        """
        Loads price history ring buffers from durable JSON storage.
        If session_date is specified, excludes bars from prior sessions to ensure zero overnight bleed.
        """
        history = {s: [] for s in self.symbols}
        if hasattr(self, "bar_history_path") and self.bar_history_path.exists():
            try:
                with open(self.bar_history_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                ist_offset = timedelta(hours=5, minutes=30)
                for sym, bars_raw in data.items():
                    if sym in history and isinstance(bars_raw, list):
                        loaded_bars = []
                        for b in bars_raw:
                            if not isinstance(b, dict):
                                continue
                            ts_ms = int(b.get("timestamp_utc_ms", 0))
                            if session_date is not None:
                                try:
                                    b_ist = datetime.fromtimestamp(ts_ms / 1000.0, tz=timezone.utc) + ist_offset
                                    if b_ist.date() != session_date:
                                        continue
                                except Exception:
                                    continue
                            loaded_bars.append(
                                BarOHLCV(
                                    timestamp_utc_ms=ts_ms,
                                    open=float(b.get("open", 0.0)),
                                    high=float(b.get("high", 0.0)),
                                    low=float(b.get("low", 0.0)),
                                    close=float(b.get("close", 0.0)),
                                    volume=float(b.get("volume", 0.0)),
                                    is_completed=bool(b.get("is_completed", True)),
                                    source=str(b.get("source", "persisted_live"))
                                )
                            )
                        history[sym] = loaded_bars
            except Exception as e:
                print(f"[!] Warning: Failed to load bar history from {self.bar_history_path}: {e}", file=sys.stderr)
        return history

    def save_bar_history(self):
        """Atomically saves price history ring buffers to durable JSON storage with provenance."""
        raw_dict = {}
        for sym, bars in self.price_history.items():
            raw_dict[sym] = [
                {
                    "timestamp_utc_ms": b.timestamp_utc_ms,
                    "open": b.open,
                    "high": b.high,
                    "low": b.low,
                    "close": b.close,
                    "volume": b.volume,
                    "is_completed": getattr(b, "is_completed", True),
                    "source": getattr(b, "source", "persisted_live")
                }
                for b in bars
            ]
        self.atomic_commit_json(self.bar_history_path, raw_dict)

    def load_pending_evaluations(
        self,
    ) -> Tuple[
        Dict[int, Dict[str, Dict[str, Any]]],
        Dict[str, Dict[str, Any]],
        Dict[str, Dict[str, Any]],
        Dict[str, Dict[str, Any]],
        Dict[str, Dict[str, Any]],
    ]:
        """Loads pending forward evals + PRIMARY/BASELINE/r50/floor-lock actives (never pooled)."""
        pending_evals: Dict[int, Dict[str, Dict[str, Any]]] = {}
        active_trades: Dict[str, Dict[str, Any]] = {}
        active_trades_baseline: Dict[str, Dict[str, Any]] = {}
        active_trades_g2x_r50: Dict[str, Dict[str, Any]] = {}
        active_trades_g2x_floor_lock: Dict[str, Dict[str, Any]] = {}
        if hasattr(self, "pending_evaluations_path") and self.pending_evaluations_path.exists():
            try:
                with open(self.pending_evaluations_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                raw_pending = data.get("pending_directional_evals", {})
                for k, v in raw_pending.items():
                    pending_evals[int(k)] = v
                active_trades = data.get("active_trades", {}) or {}
                active_trades_baseline = data.get("active_trades_baseline", {}) or {}
                active_trades_g2x_r50 = data.get("active_trades_g2x_r50", {}) or {}
                active_trades_g2x_floor_lock = (
                    data.get("active_trades_g2x_floor_lock", {}) or {}
                )
            except Exception as e:
                print(f"[!] Warning: Failed to load pending evaluations from {self.pending_evaluations_path}: {e}", file=sys.stderr)
        return (
            pending_evals,
            active_trades,
            active_trades_baseline,
            active_trades_g2x_r50,
            active_trades_g2x_floor_lock,
        )

    def _load_journaled_trade_ids(self, journal_path: Optional[Path] = None) -> set:
        """Load Estimand A trade_ids already present in a trade journal (any twin book)."""
        ids: set = set()
        path = journal_path if journal_path is not None else getattr(self, "trade_journal_path", None)
        if path is None or not Path(path).exists():
            return ids
        try:
            with open(path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        r = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    tid = r.get("trade_id")
                    if tid:
                        ids.add(str(tid))
        except OSError as e:
            print(f"[!] Warning: Failed to load trade_ids from {path}: {e}", file=sys.stderr)
        return ids

    @staticmethod
    def _estimand_normalize_book(book: str) -> str:
        b = str(book or "primary").strip().lower()
        if b in {
            "baseline",
            "bl",
            "nse_estimand_a_baseline_parallel",
            NSE_BOOK_BASELINE.lower(),
        }:
            return "baseline"
        if b in {
            "gate2x_r50",
            "r50",
            "g2x_r50",
            "nse_estimand_a_gate2x_r50_parallel",
            NSE_BOOK_G2X_R50.lower(),
        }:
            return "gate2x_r50"
        if b in {
            "gate2x_floor_lock",
            "floor_lock",
            "floorlock",
            "g2x_floor_lock",
            "nse_estimand_a_gate2x_floor_lock_parallel",
            NSE_BOOK_G2X_FLOOR_LOCK.lower(),
        }:
            return "gate2x_floor_lock"
        return "primary"

    @classmethod
    def _estimand_book_is_baseline(cls, book: str) -> bool:
        return cls._estimand_normalize_book(book) == "baseline"

    @classmethod
    def _estimand_book_is_g2x_r50(cls, book: str) -> bool:
        return cls._estimand_normalize_book(book) == "gate2x_r50"

    @classmethod
    def _estimand_book_is_g2x_floor_lock(cls, book: str) -> bool:
        return cls._estimand_normalize_book(book) == "gate2x_floor_lock"

    @classmethod
    def _estimand_entry_like_baseline(cls, book: str) -> bool:
        """BASELINE + r50 + floor-lock twins share ungated entry; exit geometry differs."""
        return cls._estimand_normalize_book(book) in (
            "baseline",
            "gate2x_r50",
            "gate2x_floor_lock",
        )

    def _estimand_active_trades(self, book: str = "primary") -> Dict[str, Dict[str, Any]]:
        key = self._estimand_normalize_book(book)
        if key == "baseline":
            return self.active_trades_baseline
        if key == "gate2x_r50":
            return self.active_trades_g2x_r50
        if key == "gate2x_floor_lock":
            return self.active_trades_g2x_floor_lock
        return self.active_trades

    def _estimand_journaled_ids(self, book: str = "primary") -> set:
        key = self._estimand_normalize_book(book)
        if key == "baseline":
            return self._journaled_trade_ids_baseline
        if key == "gate2x_r50":
            return self._journaled_trade_ids_g2x_r50
        if key == "gate2x_floor_lock":
            return self._journaled_trade_ids_g2x_floor_lock
        return self._journaled_trade_ids

    def _estimand_journal_path(self, book: str = "primary") -> Path:
        key = self._estimand_normalize_book(book)
        if key == "baseline":
            return self.trade_journal_baseline_path
        if key == "gate2x_r50":
            return self.trade_journal_g2x_r50_path
        if key == "gate2x_floor_lock":
            return self.trade_journal_g2x_floor_lock_path
        return self.trade_journal_path

    def _estimand_book_id(self, book: str = "primary") -> str:
        key = self._estimand_normalize_book(book)
        if key == "baseline":
            return NSE_BOOK_BASELINE
        if key == "gate2x_r50":
            return NSE_BOOK_G2X_R50
        if key == "gate2x_floor_lock":
            return NSE_BOOK_G2X_FLOOR_LOCK
        return NSE_BOOK_PRIMARY

    def _estimand_gate_mode_for_book(self, book: str = "primary") -> str:
        if self._estimand_entry_like_baseline(book):
            return self._baseline_gate_mode
        return self._primary_gate_mode

    def _estimand_ratchet_frac(self, book: str = "primary") -> float:
        if self._estimand_book_is_g2x_floor_lock(book):
            return float(GATE2X_FLOOR_LOCK_RATCHET)
        if self._estimand_book_is_g2x_r50(book):
            return float(GATE2X_RATCHET_PEAK_FRAC_R50)
        return float(GATE2X_RATCHET_PEAK_FRAC)

    def _estimand_gate2x_protocol(self, book: str = "primary") -> str:
        if self._estimand_book_is_g2x_floor_lock(book):
            return GATE2X_FLOOR_LOCK_PROTOCOL
        if self._estimand_book_is_g2x_r50(book):
            return GATE2X_R50_BOOKING_PROTOCOL
        return GATE2X_MFE_GIVEBACK_PROTOCOL

    def _estimand_gate2x_preset(self, book: str = "primary") -> str:
        """R4 Rust Gate2xPresetId — books never share one corrective."""
        if self._estimand_book_is_g2x_floor_lock(book):
            return "floor_lock_r50"
        if self._estimand_book_is_g2x_r50(book):
            return "r50_booking"
        return "baseline_r30"

    def _estimand_trade_id(self, *, book: str, signal_bar_idx: int, symbol: str) -> str:
        key = self._estimand_normalize_book(book)
        if key == "baseline":
            return f"nse_bl_trade_{self.state.session_idx}_{int(signal_bar_idx)}_{symbol}"
        if key == "gate2x_r50":
            return f"nse_r50_trade_{self.state.session_idx}_{int(signal_bar_idx)}_{symbol}"
        if key == "gate2x_floor_lock":
            return f"nse_fl_trade_{self.state.session_idx}_{int(signal_bar_idx)}_{symbol}"
        return f"nse_trade_{self.state.session_idx}_{int(signal_bar_idx)}_{symbol}"

    def _estimand_active_count(self) -> int:
        return (
            len(self.active_trades)
            + len(self.active_trades_baseline)
            + len(self.active_trades_g2x_r50)
            + len(self.active_trades_g2x_floor_lock)
        )

    def _regime_align_for_long(
        self, *, book: str, history: Optional[List[Any]] = None
    ) -> Dict[str, Any]:
        """Gate2E PRIMARY: block NSE LONG on DIRECTIONAL BEARISH or DI-skew adverse.

        BASELINE / g2x_r50 books stay ungated (CF) via regime_align_applies_to_book.
        """
        book_id = self._estimand_book_id(book)
        regime: Dict[str, Any]
        try:
            regime = classify_market_regime(history or [])
        except Exception:
            regime = {"regime": "UNAVAILABLE", "reason": "CLASSIFIER_ERROR"}
        return evaluate_regime_align(
            direction="LONG",
            regime=regime if isinstance(regime, dict) else None,
            book_id=book_id,
        )

    def _ist_session_date_str(self, ts_ms: Optional[int] = None) -> str:
        ist = timezone(timedelta(hours=5, minutes=30))
        if ts_ms is not None and int(ts_ms) > 0:
            return datetime.fromtimestamp(int(ts_ms) / 1000.0, tz=timezone.utc).astimezone(ist).date().isoformat()
        return datetime.now(timezone.utc).astimezone(ist).date().isoformat()

    def _bootstrap_pattern_session_stop_families(self) -> None:
        """Rebuild today's ToA|traj stop families (NOT symbol bans) from journal."""
        today = self._ist_session_date_str()
        self._pattern_session_date = today
        self._pattern_session_stop_families = set()
        path = getattr(self, "trade_journal_path", None)
        if path is None or not Path(path).exists():
            return
        try:
            with open(path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        r = json.loads(line)
                    except json.JSONDecodeError:
                        continue
                    if str(r.get("estimand") or "ESTIMAND_A") not in ("ESTIMAND_A", "A"):
                        continue
                    if str(r.get("exit_reason") or "") != "STOP_LOSS":
                        continue
                    et_ms = r.get("entry_timestamp_utc_ms")
                    if et_ms is None or self._ist_session_date_str(int(et_ms)) != today:
                        continue
                    phase = None
                    traj_b = "UNK"
                    ctx = r.get("coralys_input_context") or {}
                    if isinstance(ctx, dict):
                        phase = ctx.get("session_phase")
                        if ctx.get("traj15") is not None:
                            traj_b = _pattern_traj15_bucket(float(ctx["traj15"]))
                    if not phase:
                        phase = _pattern_session_phase_ist(
                            datetime.fromtimestamp(int(et_ms) / 1000.0, tz=timezone.utc).astimezone(
                                timezone(timedelta(hours=5, minutes=30))
                            )
                        )
                    if r.get("traj15_at_entry") is not None:
                        traj_b = _pattern_traj15_bucket(float(r["traj15_at_entry"]))
                    self._pattern_session_stop_families.add(f"{phase}|{traj_b}")
        except OSError as e:
            print(f"[!] Warning: Failed to bootstrap pattern stop families: {e}", file=sys.stderr)

    def _refresh_pattern_session_day(self) -> None:
        today = self._ist_session_date_str()
        if self._pattern_session_date != today:
            self._bootstrap_pattern_session_stop_families()

    def _note_pattern_session_stop(self, *, trade: Dict[str, Any], exit_reason: str) -> None:
        if str(exit_reason) != "STOP_LOSS":
            return
        self._refresh_pattern_session_day()
        ctx = trade.get("coralys_input_context") or {}
        phase = ctx.get("session_phase") if isinstance(ctx, dict) else None
        traj = None
        if isinstance(ctx, dict) and ctx.get("traj15") is not None:
            traj = float(ctx["traj15"])
        if traj is None and trade.get("traj15_at_entry") is not None:
            traj = float(trade["traj15_at_entry"])
        et_ms = trade.get("entry_timestamp_utc_ms")
        if not phase and et_ms:
            phase = _pattern_session_phase_ist(
                datetime.fromtimestamp(int(et_ms) / 1000.0, tz=timezone.utc).astimezone(
                    timezone(timedelta(hours=5, minutes=30))
                )
            )
        family = f"{phase or 'UNK'}|{_pattern_traj15_bucket(traj)}"
        self._pattern_session_stop_families.add(family)

    def _load_trade_pattern_library(self) -> Dict[str, Any]:
        path = trade_pattern_library_path(self.output_dir)
        if not path.exists():
            path = trade_pattern_library_path(self.output_dir.parent)
        try:
            mtime = path.stat().st_mtime if path.exists() else 0.0
        except OSError:
            mtime = 0.0
        if self._trade_pattern_library is not None and mtime == self._trade_pattern_library_mtime:
            return self._trade_pattern_library
        lib = load_trade_pattern_library(path)
        self._trade_pattern_library = lib
        self._trade_pattern_library_mtime = mtime
        return lib

    def _bars_dict_for_pattern(self, symbol: str) -> Dict[int, Dict[str, float]]:
        """Map session bar_idx → OHLCV for candle-path features."""
        history = self.price_history.get(symbol) or []
        out: Dict[int, Dict[str, float]] = {}
        ist = timezone(timedelta(hours=5, minutes=30))
        for b in history:
            try:
                dt = datetime.fromtimestamp(int(b.timestamp_utc_ms) / 1000.0, tz=timezone.utc)
                # Daemon IST convention: wall = UTC+5h30 via replace after offset add in places;
                # use astimezone for pattern bar grid.
                dt_ist = dt.astimezone(ist)
                open_ist = dt_ist.replace(hour=9, minute=15, second=0, microsecond=0)
                el_min = (dt_ist - open_ist).total_seconds() / 60.0
                bar_idx = int(el_min // 15) + 1
                if not (1 <= bar_idx <= 25):
                    continue
                out[bar_idx] = {
                    "open": float(b.open),
                    "high": float(b.high),
                    "low": float(b.low),
                    "close": float(b.close),
                    "volume": float(b.volume),
                }
            except Exception:
                continue
        return out

    def _match_trade_pattern(
        self,
        *,
        input_ctx: Optional[Dict[str, Any]],
        traj15: float,
        bar_idx: int,
        decision_ts_ist: Optional[datetime],
        symbol: Optional[str] = None,
    ) -> Dict[str, Any]:
        self._refresh_pattern_session_day()
        phase = (input_ctx or {}).get("session_phase") or "UNK"
        traj_b = _pattern_traj15_bucket(float(traj15))
        family = f"{phase}|{traj_b}"
        prior_stop = family in self._pattern_session_stop_families
        bars = self._bars_dict_for_pattern(symbol) if symbol else {}
        feats = features_from_coralys_and_candles(
            input_ctx=input_ctx,
            traj15=float(traj15),
            bar_idx=int(bar_idx),
            decision_ts_ist=decision_ts_ist,
            same_session_prior_stop=prior_stop,
            bars_for_symbol_session=bars or None,
        )
        lib = self._load_trade_pattern_library()
        # PRIMARY = volume axes + Gate2E allow_only; BASELINE twin = pre-volume + ungated
        # control (crypto Gate2E parity). Books never pooled.
        primary = match_trade_pattern_entry(
            library=lib,
            feats=feats,
            include_volume=True,
            gate_mode=self._primary_gate_mode,
        )
        baseline = match_trade_pattern_entry(
            library=lib,
            feats=feats,
            include_volume=False,
            gate_mode=self._baseline_gate_mode,
        )
        primary["book"] = NSE_BOOK_PRIMARY
        baseline["book"] = NSE_BOOK_BASELINE
        primary["baseline_parallel"] = baseline
        primary["twin_book"] = NSE_BOOK_BASELINE
        return primary

    def _match_exit_pattern_for_trade(
        self,
        *,
        symbol: str,
        trade: Dict[str, Any],
        bar_idx: int,
    ) -> Dict[str, Any]:
        ctx = trade.get("coralys_input_context") or {}
        phase = ctx.get("session_phase") if isinstance(ctx, dict) else None
        traj = None
        if isinstance(ctx, dict) and ctx.get("traj15") is not None:
            traj = float(ctx["traj15"])
        if traj is None and trade.get("traj15_at_entry") is not None:
            traj = float(trade["traj15_at_entry"])
        if traj is None:
            traj = 0.05
        if not phase:
            phase = "UNK"
        bars = self._bars_dict_for_pattern(symbol)
        lib = self._load_trade_pattern_library()
        primary = match_trade_pattern_exit(
            library=lib,
            session_phase=str(phase),
            traj15=float(traj),
            bars_dict=bars,
            entry_bar_idx=int(trade.get("entry_bar_idx") or bar_idx),
            current_bar_idx=int(bar_idx),
            entry_price=float(trade.get("entry_price") or 0) or 1.0,
            include_volume=True,
        )
        baseline = match_trade_pattern_exit(
            library=lib,
            session_phase=str(phase),
            traj15=float(traj),
            bars_dict=bars,
            entry_bar_idx=int(trade.get("entry_bar_idx") or bar_idx),
            current_bar_idx=int(bar_idx),
            entry_price=float(trade.get("entry_price") or 0) or 1.0,
            include_volume=False,
        )
        primary["baseline_parallel"] = baseline
        primary["twin_book"] = NSE_BOOK_BASELINE
        return primary

    def save_pending_evaluations(self):
        """Atomically persists pending directional evaluations and active trades to durable storage."""
        payload = {
            "pending_directional_evals": {str(k): v for k, v in self.pending_directional_evals.items()},
            "active_trades": self.active_trades,
            "active_trades_baseline": self.active_trades_baseline,
            "active_trades_g2x_r50": self.active_trades_g2x_r50,
            "active_trades_g2x_floor_lock": self.active_trades_g2x_floor_lock,
            "updated_at_utc_ms": int(time.time() * 1000),
            "updated_at_iso": datetime.now(timezone.utc).isoformat()
        }
        self.atomic_commit_json(self.pending_evaluations_path, payload)

    @staticmethod
    def _ist_naive(dt: datetime) -> datetime:
        """
        Normalize to naive IST wall-clock for candle-grid arithmetic.

        Daemon convention: IST wall = UTC datetime + 5h30m timedelta (tzinfo may
        still say UTC). Strip tzinfo only — do NOT astimezone(IST) or the wall
        clock shifts by another +5h30m.
        """
        return dt.replace(tzinfo=None)

    def _expected_session_candle_starts_ms(
        self,
        *,
        open_time_ist: datetime,
        completed_bars: int,
        ist_offset: timedelta,
    ) -> List[int]:
        """UTC-ms candle starts for completed bars 1..N (excludes forming candle)."""
        if completed_bars <= 0:
            return []
        open_n = self._ist_naive(open_time_ist)
        out: List[int] = []
        for i in range(int(completed_bars)):
            c_start_ist = open_n + timedelta(minutes=i * 15)
            # Interpret naive IST as UTC+05:30 when converting to epoch ms.
            c_start_utc = c_start_ist - ist_offset
            out.append(
                int(
                    c_start_utc.replace(tzinfo=timezone.utc).timestamp() * 1000
                )
            )
        return out

    def _session_candle_index_map(
        self,
        symbol: str,
        *,
        open_time_ist: datetime,
        session_date,
        ist_offset: timedelta,
    ) -> Dict[int, Any]:
        """Map today's durable candle-start ms → bar (quote-snapshot spam excluded)."""
        existing_by_start: Dict[int, Any] = {}
        open_n = self._ist_naive(open_time_ist)
        for b in self.price_history.get(symbol) or []:
            try:
                if not bool(getattr(b, "is_completed", True)):
                    continue
                if str(getattr(b, "source", "") or "") == "kite_quote_snapshot":
                    continue
                b_ist = self._ist_naive(
                    datetime.fromtimestamp(
                        int(b.timestamp_utc_ms) / 1000.0, tz=timezone.utc
                    )
                    + ist_offset
                )
                if b_ist.date() != session_date:
                    continue
                el_min = (b_ist - open_n).total_seconds() / 60.0
                if el_min < 0:
                    continue
                c_idx = int(el_min // 15)
                c_start_ist = open_n + timedelta(minutes=c_idx * 15)
                c_start_utc = c_start_ist - ist_offset
                c_start_ms = int(
                    c_start_utc.replace(tzinfo=timezone.utc).timestamp() * 1000
                )
                if c_start_ms not in existing_by_start:
                    existing_by_start[c_start_ms] = b
            except Exception:
                continue
        return existing_by_start

    def _missing_session_candle_starts(
        self,
        symbol: str,
        *,
        open_time_ist: datetime,
        completed_bars: int,
        session_date,
        ist_offset: timedelta,
    ) -> List[int]:
        """Return expected candle-start ms that are absent for this symbol today."""
        expected = self._expected_session_candle_starts_ms(
            open_time_ist=open_time_ist,
            completed_bars=completed_bars,
            ist_offset=ist_offset,
        )
        have = self._session_candle_index_map(
            symbol,
            open_time_ist=open_time_ist,
            session_date=session_date,
            ist_offset=ist_offset,
        )
        return [ms for ms in expected if ms not in have]

    def bootstrap_same_day_history(
        self,
        now_utc: Optional[datetime] = None,
        target_symbols: Optional[Iterable[str]] = None
    ) -> Dict[str, Any]:
        """
        Recovers completed 15m bars for today's session from Kite historical data.
        Requirements:
        1. Only current trading session completed bars are fetched.
        2. No bootstrap at opening (< warmup completed bars).
        3. Excludes forming/incomplete candle.
        4. Non-destructive merge: preserves live observations over historical duplicates.
        5. Fails closed on API failure without crashing or manufacturing eligibility.
        6. Per-symbol recovery: queries deficient symbols OR any with gaps in the
           completed session candle grid, then fetches missing candles in code.
        """
        if now_utc is None:
            now_utc = datetime.now(timezone.utc)
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = now_utc + ist_offset

        window, closed_reason = resolve_session_window(
            now_ist,
            exchange_holidays=getattr(self, "exchange_holidays", NSE_2026_HOLIDAYS),
        )
        if window is None:
            return {"status": closed_reason or "SESSION_CLOSED", "recovered_symbols": 0}

        open_time_ist = now_ist.replace(
            hour=window.open_ist.hour,
            minute=window.open_ist.minute,
            second=0,
            microsecond=0,
        )

        # Before market open
        if now_ist < open_time_ist:
            return {"status": f"PRE_OPEN_{window.kind}", "recovered_symbols": 0}

        # Calculate completed 15m intervals since session open
        completed_bars = completed_bars_since_open(now_ist, window)

        # Rule 2: Do not bootstrap at the opening (muhurat uses reduced warmup).
        warmup_n = int(window.warmup_completed_bars)
        if completed_bars < warmup_n:
            return {
                "status": "OPENING_WARMUP_COLD",
                "completed_bars_today": completed_bars,
                "recovered_symbols": 0,
                "session_kind": window.kind,
                "reason": (
                    f"Only {completed_bars}/{warmup_n} completed bars today; "
                    f"opening warm-up active ({window.label})"
                ),
            }

        # Pick up dashboard re-auth before same-day hist storm (next-session + mid-day rotate).
        reload_fn = getattr(self.feed_adapter, "reload_credentials_if_stale", None)
        if callable(reload_fn):
            reload_fn()

        now_mono = time.monotonic()
        need_n = min(completed_bars, 5)
        # Drop poll-spam before measuring readiness — fake kite_live day-OHLC clones
        # previously satisfied need_n and blocked real 15m historical recovery (G4 lock).
        self._purge_quote_snapshot_spam()

        def _durable_bar_count(sym: str) -> int:
            return sum(
                1
                for b in (self.price_history.get(sym) or [])
                if bool(getattr(b, "is_completed", True))
                and str(getattr(b, "source", "") or "") != "kite_quote_snapshot"
            )

        def _needs_fetch(sym: str) -> bool:
            missing = self._missing_session_candle_starts(
                sym,
                open_time_ist=open_time_ist,
                completed_bars=completed_bars,
                session_date=now_ist.date(),
                ist_offset=ist_offset,
            )
            if missing:
                return True
            if _durable_bar_count(sym) < need_n:
                return True
            return False

        # Determine symbols that actually need recovery (warmup shortfall OR any gap).
        if target_symbols is not None:
            symbols_to_query = list(target_symbols)
        else:
            symbols_to_query = []
            for s in self.symbols:
                if not _needs_fetch(s):
                    # Grid complete for this completed_bars level.
                    self._symbol_last_bootstrap_bar[s] = completed_bars
                    continue
                # Successful empty/complete attempt for this bar: wait for next completed bar
                # only when there are no detected gaps (gap detection forces retry).
                missing = self._missing_session_candle_starts(
                    s,
                    open_time_ist=open_time_ist,
                    completed_bars=completed_bars,
                    session_date=now_ist.date(),
                    ist_offset=ist_offset,
                )
                if (
                    not missing
                    and self._symbol_last_bootstrap_bar.get(s, 0) >= completed_bars
                ):
                    continue
                # Transient failure: short cooldown then retry within the same bar.
                fail_at = float(self._symbol_bootstrap_fail_mono.get(s, 0.0) or 0.0)
                if fail_at > 0.0 and (now_mono - fail_at) < BOOTSTRAP_FAIL_COOLDOWN_SEC:
                    continue
                symbols_to_query.append(s)

        if not symbols_to_query:
            return {
                "status": "ALREADY_UP_TO_DATE",
                "completed_bars_today": completed_bars,
                "recovered_symbols": 0,
                "total_backfilled_bars": 0,
                "missing_candle_symbols": 0,
            }

        # Determine exact from and to boundaries for historical API
        # from_utc: 09:15:00 IST today
        from_utc = (open_time_ist - ist_offset).astimezone(timezone.utc)
        # to_utc: start of the last COMPLETED candle
        # Candle k starts at open_time_ist + (k-1)*15m, completes at open_time_ist + k*15m
        last_completed_candle_start_ist = open_time_ist + timedelta(minutes=(completed_bars - 1) * 15)
        to_utc = (last_completed_candle_start_ist + timedelta(minutes=1) - ist_offset).astimezone(timezone.utc)

        # Verify feed_adapter has fetch_historical_15m_bars
        if not hasattr(self.feed_adapter, "fetch_historical_15m_bars"):
            return {"status": "ADAPTER_NO_HISTORICAL_CAPABILITY", "recovered_symbols": 0}

        recovered_count = 0
        total_backfilled_bars = 0
        transient_failures = 0
        auth_reloaded = False
        gap_symbols = 0

        for sym in symbols_to_query:
            missing_before = self._missing_session_candle_starts(
                sym,
                open_time_ist=open_time_ist,
                completed_bars=completed_bars,
                session_date=now_ist.date(),
                ist_offset=ist_offset,
            )
            if missing_before:
                gap_symbols += 1
            try:
                hist_bars = self.feed_adapter.fetch_historical_15m_bars(sym, from_utc, to_utc)
            except KiteAuthError as e:
                # Fail closed: do NOT mark bar complete — retry after cooldown / re-auth.
                self._symbol_bootstrap_fail_mono[sym] = time.monotonic()
                transient_failures += 1
                if not auth_reloaded and callable(reload_fn):
                    reload_fn(force=True)
                    auth_reloaded = True
                    print(f"[!] Bootstrap auth failure ({sym}: {e}); credentials force-reloaded once", flush=True)
                continue
            except KiteRateLimitError:
                self._symbol_bootstrap_fail_mono[sym] = time.monotonic()
                transient_failures += 1
                time.sleep(2.0)
                continue
            except Exception:
                # Fail closed on API failure for this symbol: do not crash; do not consume the bar slot.
                self._symbol_bootstrap_fail_mono[sym] = time.monotonic()
                transient_failures += 1
                continue

            # Clear transient fail marker on any successful HTTP round-trip.
            self._symbol_bootstrap_fail_mono.pop(sym, None)

            if not hist_bars:
                # Empty hist with remaining gaps → retry later; only mark complete if grid full.
                if not missing_before:
                    self._symbol_last_bootstrap_bar[sym] = completed_bars
                else:
                    self._symbol_bootstrap_fail_mono[sym] = time.monotonic()
                continue

            # Filter hist_bars strictly:
            # Must be from today's session, completed, and not after last_completed_candle_start_ist
            valid_hist = []
            for b in hist_bars:
                try:
                    b_dt_ist = datetime.fromtimestamp(b.timestamp_utc_ms / 1000.0, tz=timezone.utc) + ist_offset
                except Exception:
                    continue
                # Must be same day
                if b_dt_ist.date() != now_ist.date():
                    continue
                # Must be >= 09:15 IST
                if b_dt_ist < open_time_ist:
                    continue
                # Must not exceed last completed candle start (excludes forming candle)
                if b_dt_ist > last_completed_candle_start_ist:
                    continue
                valid_hist.append(b)

            if not valid_hist:
                if missing_before:
                    self._symbol_bootstrap_fail_mono[sym] = time.monotonic()
                continue

            # Non-destructive merge with existing price_history:
            # Preserve existing live observations, backfill missing bars
            existing_by_start = self._session_candle_index_map(
                sym,
                open_time_ist=open_time_ist,
                session_date=now_ist.date(),
                ist_offset=ist_offset,
            )
            # Keep non-today bars from the ring buffer.
            prior_bars = []
            for b in self.price_history.get(sym) or []:
                try:
                    b_ist = datetime.fromtimestamp(
                        int(b.timestamp_utc_ms) / 1000.0, tz=timezone.utc
                    ) + ist_offset
                    if b_ist.date() != now_ist.date():
                        prior_bars.append(b)
                except Exception:
                    prior_bars.append(b)

            added_for_sym = 0
            open_n = self._ist_naive(open_time_ist)
            for hb in valid_hist:
                try:
                    hb_ist = self._ist_naive(
                        datetime.fromtimestamp(
                            hb.timestamp_utc_ms / 1000.0, tz=timezone.utc
                        )
                        + ist_offset
                    )
                    el_min = (hb_ist - open_n).total_seconds() / 60.0
                    c_idx = int(el_min // 15)
                    c_start_ist = open_n + timedelta(minutes=c_idx * 15)
                    c_start_utc = c_start_ist - ist_offset
                    c_start_ms = int(
                        c_start_utc.replace(tzinfo=timezone.utc).timestamp() * 1000
                    )

                    if c_start_ms not in existing_by_start:
                        existing_by_start[c_start_ms] = hb
                        added_for_sym += 1
                except Exception:
                    continue

            # Reconstruct sorted price_history for symbol (prior sessions + today grid).
            today_merged = [existing_by_start[k] for k in sorted(existing_by_start.keys())]
            merged = list(prior_bars) + today_merged
            if len(merged) > 60:
                merged = merged[-60:]
            self.price_history[sym] = merged

            if added_for_sym > 0:
                recovered_count += 1
                total_backfilled_bars += added_for_sym

            missing_after = self._missing_session_candle_starts(
                sym,
                open_time_ist=open_time_ist,
                completed_bars=completed_bars,
                session_date=now_ist.date(),
                ist_offset=ist_offset,
            )
            # Mark complete only when the full completed-bar grid is present.
            if not missing_after and _durable_bar_count(sym) >= need_n:
                self._symbol_last_bootstrap_bar[sym] = completed_bars
            else:
                self._symbol_bootstrap_fail_mono[sym] = time.monotonic()

        if transient_failures:
            print(
                f"[!] Bootstrap transient failures={transient_failures}/"
                f"{len(symbols_to_query)} (will retry after {BOOTSTRAP_FAIL_COOLDOWN_SEC:.0f}s cooldown)",
                flush=True,
            )
        if total_backfilled_bars > 0 or gap_symbols > 0:
            print(
                f"[bootstrap] gap_symbols={gap_symbols} recovered={recovered_count} "
                f"backfilled_bars={total_backfilled_bars} completed_bars={completed_bars}",
                flush=True,
            )

        return {
            "status": "BOOTSTRAP_SUCCESS",
            "completed_bars_today": completed_bars,
            "recovered_symbols": recovered_count,
            "total_backfilled_bars": total_backfilled_bars,
            "transient_failures": transient_failures,
            "queried_symbols": len(symbols_to_query),
            "missing_candle_symbols": gap_symbols,
        }

    def append_jsonl(self, path: Path, record: Dict[str, Any]):
        """Appends a single JSON record with durable flush."""
        with open(path, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, separators=(",", ":")) + "\n")
            f.flush()

    def _maybe_log_gate2e_shadow_insufficient(
        self,
        *,
        symbol: str,
        pattern_stamp: Dict[str, Any],
        suppressed_ms: int,
        decision_bar_idx: Optional[int] = None,
        source: str = "live_suppress",
        direction: str = "LONG",
        session_phase: Optional[str] = None,
        require_blocks_entry: bool = True,
    ) -> Optional[Dict[str, Any]]:
        """Stamp-only cold-start ABSTAIN_INSUFFICIENT on PRIMARY (authority 0%).

        Never opens a trade. Does not change live allow_only.
        """
        if not gate2e_shadow_insufficient_enabled():
            return None
        if not isinstance(pattern_stamp, dict):
            return None
        if pattern_stamp.get("book") == NSE_BOOK_BASELINE:
            return None
        if not matches_shadow_observe_abstain_insufficient(pattern_stamp):
            return None
        if require_blocks_entry and not bool(pattern_stamp.get("blocks_entry")):
            return None
        stamp = build_gate2e_shadow_insufficient_stamp(
            pattern_stamp=pattern_stamp,
            book_id=NSE_BOOK_PRIMARY,
            twin_book=NSE_BOOK_BASELINE,
            suppressed_ms=int(suppressed_ms),
            symbol=symbol,
            direction=direction,
            plan_id=None,
            decision_bar_idx=decision_bar_idx,
            source=source,
            extra={"session_phase": session_phase} if session_phase else None,
        )
        stamp["timestamp_utc"] = datetime.now(timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )
        self.append_jsonl(self.gate2e_shadow_insufficient_path, stamp)
        return stamp

    def append_opportunity_drop(self, drop_type: str, reason: str, symbol: str = "ALL", **kwargs):
        """Logs an unexecutable or dropped opportunity to the opportunity ledger."""
        rec = {
            "timestamp_utc_ms": int(time.time() * 1000),
            "timestamp_iso": datetime.now(timezone.utc).isoformat(),
            "symbol": symbol,
            "drop_type": drop_type,
            "reason": reason,
            **kwargs
        }
        self.append_jsonl(self.opportunity_path, rec)

    def is_ist_session_active(self, now_utc: Optional[datetime] = None) -> Tuple[bool, int, str]:
        """
        Determines if the current moment is within an NSE active session.
        Regular cash: 09:15–15:30 IST. Muhurat (e.g. 2026-11-08): 14:20–15:30 IST.
        Returns: (is_active, bar_idx, phase_name)
          bar_idx: 1..N if active; 0 if pre-open / post-close
        """
        if now_utc is None:
            now_utc = datetime.now(timezone.utc)
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = now_utc + ist_offset

        window, closed_reason = resolve_session_window(
            now_ist,
            exchange_holidays=getattr(self, "exchange_holidays", NSE_2026_HOLIDAYS),
        )
        self._active_session_window = window
        if window is None:
            return False, 0, closed_reason or "SESSION_CLOSED"
        return session_phase(now_ist, window)

    @staticmethod
    def _arm_a_fixed_boundaries(entry_price: float) -> Dict[str, Any]:
        """Frozen Arm A percent exits (+1.5% / -1.0%)."""
        return {
            "ok": True,
            "stop_price": round(entry_price * ARM_A_STOP_MULT, 2),
            "target_price": round(entry_price * ARM_A_TARGET_MULT, 2),
            "stop_bps": ARM_A_STOP_BPS,
            "target_bps": ARM_A_TARGET_BPS,
            "atr_14": None,
            "trend_label": "ARM_A_FIXED",
            "policy_id": "ARM_A_FIXED_PCT",
            "frozen_at_entry": True,
            "fallback_reason": None,
        }

    def _feature_bars_for_boundaries(self, symbol: str, session_history: List[Any]) -> List[Any]:
        """
        Build OHLCV series for ATR/swing exits.
        Prior-session bars are allowed here; traj15 continues to use session price_history only.
        """
        now_utc = datetime.now(timezone.utc)
        eval_bar = int(getattr(self.state, "last_durable_bar_idx", 0) or 0)
        if self._atr_feature_cache_bar_idx != eval_bar:
            self._atr_feature_cache.clear()
            self._atr_feature_cache_bar_idx = eval_bar

        cached = self._atr_feature_cache.get(symbol)
        if cached is None:
            cached = []
            fetch = getattr(self.feed_adapter, "fetch_historical_15m_bars", None)
            if callable(fetch):
                from_utc = now_utc - timedelta(days=ATR_FEATURE_LOOKBACK_DAYS)
                try:
                    cached = list(fetch(symbol, from_utc, now_utc) or [])
                except Exception:
                    cached = []
                # Kite historical soft rate-limit (~3 req/s)
                time.sleep(0.34)
            self._atr_feature_cache[symbol] = cached

        by_ts: Dict[int, Any] = {}
        for b in list(cached) + list(session_history or []):
            ts = int(getattr(b, "timestamp_utc_ms", 0) or (b.get("timestamp_utc_ms") if isinstance(b, dict) else 0) or 0)
            if ts <= 0:
                continue
            by_ts[ts] = b
        merged = [by_ts[k] for k in sorted(by_ts.keys())]
        if len(merged) > ATR_FEATURE_MAX_BARS:
            merged = merged[-ATR_FEATURE_MAX_BARS:]
        return merged

    def resolve_exit_boundaries(
        self,
        symbol: str,
        entry_price: float,
        session_history: List[Any],
        friction_bps: float,
    ) -> Dict[str, Any]:
        """
        Entry-time stop/target. Dynamic ATR/structure when enabled; Arm A fallback if ATR cold.
        """
        arm = self._arm_a_fixed_boundaries(entry_price)
        if EXIT_POLICY != "DYNAMIC_OHLCV_STRUCTURE_V1":
            return arm

        feature_bars = self._feature_bars_for_boundaries(symbol, session_history)
        dyn = compute_dynamic_boundaries(
            feature_bars,
            side="LONG",
            entry_price=entry_price,
            friction_bps=max(1.0, float(friction_bps)),
        )
        # Prefer ATR/structure prices whenever they exist. Net R:R failure is recorded
        # but must not silently revert to fixed ±1.0%/±1.5% (user-expected dynamic exits).
        if dyn.get("stop_price") is not None and dyn.get("target_price") is not None and dyn.get("atr_14"):
            soft = not bool(dyn.get("ok"))
            return {
                "ok": True,
                "stop_price": round(float(dyn["stop_price"]), 2),
                "target_price": round(float(dyn["target_price"]), 2),
                "stop_bps": float(dyn["stop_bps"]),
                "target_bps": float(dyn["target_bps"]),
                "atr_14": dyn.get("atr_14"),
                "trend_label": dyn.get("trend_label"),
                "er_16": dyn.get("er_16"),
                "adx_14": dyn.get("adx_14"),
                "regime": dyn.get("regime"),
                "stop_atr_mult": dyn.get("stop_atr_mult"),
                "target_atr_mult": dyn.get("target_atr_mult"),
                "policy_id": dyn.get("policy_id") or "RP608_DYNAMIC_OHLCV_STRUCTURE_V1",
                "frozen_at_entry": True,
                "net_rr": dyn.get("net_rr"),
                "fallback_reason": (dyn.get("reason") if soft else None),
            }

        arm["fallback_reason"] = dyn.get("reason") or "DYNAMIC_UNAVAILABLE"
        arm["atr_14"] = dyn.get("atr_14")
        arm["trend_label"] = dyn.get("trend_label") or "DYNAMIC_FALLBACK_ARM_A"
        return arm

    def _bar_idx_candle_start_utc_ms(self, bar_idx: int) -> Optional[int]:
        """UTC ms for the IST 15m candle open of session bar_idx (1-based)."""
        if int(bar_idx) < 1:
            return None
        window = getattr(self, "_active_session_window", None)
        now_utc = datetime.now(timezone.utc)
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = (now_utc + ist_offset).replace(tzinfo=None)
        if window is None:
            window, _ = resolve_session_window(
                now_ist,
                exchange_holidays=getattr(self, "exchange_holidays", NSE_2026_HOLIDAYS),
            )
        if window is None:
            return None
        open_ist = now_ist.replace(
            hour=window.open_ist.hour,
            minute=window.open_ist.minute,
            second=0,
            microsecond=0,
        )
        candle_ist = open_ist + timedelta(minutes=(int(bar_idx) - 1) * 15)
        return int((candle_ist - ist_offset).replace(tzinfo=timezone.utc).timestamp() * 1000)

    @staticmethod
    def _is_quote_snapshot_bar(bar: Any) -> bool:
        src = str(getattr(bar, "source", "") or "")
        if src == "kite_quote_snapshot":
            return True
        if not bool(getattr(bar, "is_completed", True)):
            return True
        return False

    def _purge_quote_snapshot_spam(self, symbol: Optional[str] = None) -> int:
        """Drop day-OHLC quote-poll spam from price_history without touching unit-test bars.

        Live poll spam uses real wall-clock timestamps (year >= 2024) with non-zero
        seconds and source kite_live/persisted_live. Synthetic test bars use tiny
        epoch offsets and must be preserved verbatim.
        """
        # 2024-01-01 UTC — live market stamps are far above this; unittest bars are not.
        live_epoch_floor_ms = 1_704_067_200_000
        ist_offset = timedelta(hours=5, minutes=30)
        symbols = [symbol] if symbol else list(self.price_history.keys())
        removed_total = 0
        for sym in symbols:
            bars = list(self.price_history.get(sym) or [])
            if not bars:
                continue
            before = len(bars)
            kept: List[BarOHLCV] = []
            live_slots: Dict[int, BarOHLCV] = {}
            for b in bars:
                ts = int(getattr(b, "timestamp_utc_ms", 0) or 0)
                if ts <= 0:
                    continue
                src = str(getattr(b, "source", "") or "")
                if src == "kite_quote_snapshot":
                    continue
                if not bool(getattr(b, "is_completed", True)) and src != "kite_historical_same_day":
                    continue
                is_live_stamp = ts >= live_epoch_floor_ms and src in {
                    "kite_live",
                    "persisted_live",
                }
                if not is_live_stamp:
                    kept.append(b)
                    continue
                try:
                    b_ist = datetime.fromtimestamp(ts / 1000.0, tz=timezone.utc) + ist_offset
                    aligned = b_ist.minute % 15 == 0 and b_ist.second == 0
                    floor_min = (b_ist.hour * 60 + b_ist.minute) // 15 * 15
                    slot_ist = b_ist.replace(
                        hour=floor_min // 60, minute=floor_min % 60, second=0, microsecond=0
                    )
                    slot_ms = int((slot_ist - ist_offset).timestamp() * 1000)
                except Exception:
                    aligned = False
                    slot_ms = ts
                if not aligned:
                    # Unaligned wall-clock poll — drop (re-upserted as snapshot live).
                    continue
                prev = live_slots.get(slot_ms)
                if prev is None:
                    live_slots[slot_ms] = b
                    continue
                # Prefer keeping a single aligned bar per slot.
                live_slots[slot_ms] = b
            # Merge preserved synthetic/historical bars with deduped live slots.
            by_ts: Dict[int, BarOHLCV] = {
                int(b.timestamp_utc_ms): b for b in kept if int(getattr(b, "timestamp_utc_ms", 0) or 0) > 0
            }
            for slot_ms, b in live_slots.items():
                by_ts[int(b.timestamp_utc_ms)] = b
            merged = [by_ts[k] for k in sorted(by_ts.keys())]
            if len(merged) > 60:
                merged = merged[-60:]
            self.price_history[sym] = merged
            removed_total += max(0, before - len(merged))
        return removed_total

    def _upsert_session_bar(self, symbol: str, bar_idx: int, latest_bar: BarOHLCV) -> BarOHLCV:
        """Insert/replace bars without letting quote-poll spam flood traj15 history."""
        history = self.price_history.setdefault(symbol, [])
        src = str(getattr(latest_bar, "source", "kite_live") or "kite_live")
        is_snap = src == "kite_quote_snapshot"

        # Unit tests + durable/historical bars: keep exact-timestamp append/replace.
        if not is_snap:
            if history and history[-1].timestamp_utc_ms == latest_bar.timestamp_utc_ms:
                history[-1] = latest_bar
            else:
                history.append(latest_bar)
            if len(history) > 60:
                history.pop(0)
            return latest_bar

        candle_ts = self._bar_idx_candle_start_utc_ms(bar_idx)
        if candle_ts is None:
            # No active session window — replace-in-place by identical stamp only.
            if history and history[-1].timestamp_utc_ms == latest_bar.timestamp_utc_ms:
                history[-1] = latest_bar
            else:
                history.append(latest_bar)
            if len(history) > 60:
                history.pop(0)
            return latest_bar

        bar = BarOHLCV(
            timestamp_utc_ms=int(candle_ts),
            open=float(latest_bar.open),
            high=float(latest_bar.high),
            low=float(latest_bar.low),
            close=float(latest_bar.close),
            volume=float(latest_bar.volume),
            is_completed=False,
            source="kite_quote_snapshot",
        )
        for i, existing in enumerate(history):
            if int(existing.timestamp_utc_ms) == int(candle_ts):
                # Never clobber a completed candle with a day-OHLC snapshot poll.
                if bool(getattr(existing, "is_completed", True)) and str(
                    getattr(existing, "source", "")
                ) != "kite_quote_snapshot":
                    return existing
                history[i] = bar
                return bar
        history.append(bar)
        history.sort(key=lambda b: int(b.timestamp_utc_ms))
        if len(history) > 60:
            self.price_history[symbol] = history[-60:]
        return bar

    def evaluate_symbol_bar(
        self,
        symbol: str,
        bar_idx: int,
        latest_bar: BarOHLCV,
        quote: Optional[TopOfBookQuote],
        *,
        mutate_trades: bool = True,
    ) -> Optional[Dict[str, Any]]:
        """
        Evaluates a completed 15m bar for a symbol.
        Emits Estimand B directional decision card and manages Estimand A trade lifecycle.
        mutate_trades=False on provisional midbar polls (no open/close churn).
        """
        latest_bar = self._upsert_session_bar(symbol, bar_idx, latest_bar)
        history = self.price_history[symbol]
        # Session bars for G3 traj must exclude incomplete quote snapshots so the
        # 15-bar lookback is not filled with identical day-OHLC clones (G4 lock).
        session_bars_for_g3 = [
            b for b in history
            if not (
                str(getattr(b, "source", "")) == "kite_quote_snapshot"
                and not bool(getattr(b, "is_completed", True))
            )
        ]
        # Forming bar still counts toward session_n when it is the only today mark.
        if not session_bars_for_g3 and history:
            session_bars_for_g3 = list(history[-1:])

        # ─── Causal Momentum Scoring (G3 warmup policy) ────────
        # same_session: today's bars only (legacy cold until ~10:30).
        # cross_gap_neutral_v1: prior session + today; overnight gap zeroed in traj15.
        sess_date = str(
            self.current_session_date
            or getattr(self.state, "current_session_date", "")
            or ""
        )
        if hasattr(self.current_session_date, "isoformat"):
            sess_date = self.current_session_date.isoformat()
        feature_bars = None
        if str(G3_WARMUP_POLICY).lower() in {
            POLICY_CROSS_GAP_NEUTRAL_V1,
            "c",
            "policy_c",
            "cross_gap_neutral",
        }:
            feature_bars = self._feature_bars_for_boundaries(symbol, session_bars_for_g3)
        g3_m = evaluate_g3_momentum(
            policy=G3_WARMUP_POLICY,
            session_bars=session_bars_for_g3,
            feature_bars=feature_bars,
            today_session_date=sess_date,
        )
        traj15 = float(g3_m["traj15"])
        signal_entry = bool(g3_m["signal_long"])
        closes = [b.close for b in history]

        # Coralys adapter inputs (prior + pre-open + phase). Crates untouched.
        _t_decision_ms = int(getattr(latest_bar, "timestamp_utc_ms", 0) or 0)
        sess_date_str = str(sess_date)[:10] if sess_date else ""
        _decision_ts_ist = (
            decision_ts_from_bar_ms(_t_decision_ms)
            if _t_decision_ms > 0
            else datetime.now(timezone(timedelta(hours=5, minutes=30)))
        )
        if not sess_date_str:
            sess_date_str = _decision_ts_ist.date().isoformat()
        input_ctx = self.coralys_input_store.evaluate(
            symbol=symbol,
            session_date=sess_date_str,
            decision_ts_ist=_decision_ts_ist,
            traj15=traj15,
            signal_long=bool(signal_entry),
        )
        pattern_stamp = self._match_trade_pattern(
            input_ctx=input_ctx,
            traj15=traj15,
            bar_idx=bar_idx,
            decision_ts_ist=_decision_ts_ist,
            symbol=symbol,
        )
        pattern_blocks = bool(
            signal_entry and pattern_stamp.get("blocks_entry")
        )
        coralys_blocks = bool(
            signal_entry
            and coralys_input_gate_enabled()
            and not bool(input_ctx.get("allow_long_entry"))
        )
        # Gate2E regime-align (PRIMARY): block LONG into DIRECTIONAL BEARISH.
        regime_align_stamp = self._regime_align_for_long(book="primary", history=history)
        regime_blocks = bool(signal_entry and regime_align_stamp.get("blocks_entry"))
        input_gate_blocks = bool(coralys_blocks or pattern_blocks or regime_blocks)

        # Gate G4 Quote Validation (Freshness, Non-Zero, Uncrossed Top-of-Book)
        # Freshness is vs wall clock — NOT vs bar open (15m bar age ≠ stale L1).
        MAX_QUOTE_AGE_MS = 60000  # 60s max permitted age
        is_fresh = True
        quote_age_ms = None
        if quote and quote.timestamp_utc_ms > 0:
            wall_ms = int(time.time() * 1000)
            quote_age_ms = abs(wall_ms - int(quote.timestamp_utc_ms))
            if quote_age_ms > MAX_QUOTE_AGE_MS:
                is_fresh = False

        has_valid_quote = (
            quote is not None
            and quote.bid_price > 0
            and quote.ask_price > 0
            and quote.ask_price > quote.bid_price
            and is_fresh
        )

        p_ref = latest_bar.close
        p_decision = p_ref
        t_decision = latest_bar.timestamp_utc_ms

        # ─── Estimand B: Directional Opportunity Prediction ───
        is_eligible_estimand_b = (1 <= bar_idx <= ESTIMAND_B_ELIGIBLE_BARS)
        card_b = {
            "protocol_id": PROTOCOL_ID,
            "estimand": "ESTIMAND_B",
            "symbol": symbol,
            "session_idx": self.state.session_idx,
            "bar_idx": bar_idx,
            "decision_timestamp_utc_ms": t_decision,
            "reference_price": p_ref,
            "prediction": 1 if signal_entry else 0,
            "forecast_probability_up": FORECAST_LONG_PROB if signal_entry else FORECAST_STANDASIDE_PROB,
            "reference_training_prevalence": ESTIMAND_B_TRAIN_PREVALENCE,
            "eligible": is_eligible_estimand_b,
            "status": "PENDING_OUTCOME" if is_eligible_estimand_b else "INTRADAY_BOUNDARY_EXCLUDED",
            "forward_target_bar_idx": (bar_idx + 4) if is_eligible_estimand_b else None,
            "outcome_timestamp_utc_ms": None,
            "outcome_price": None,
            "directional_label": None,
            "brier_score": None
        }

        # Store in pending queue if eligible for 4-bar forward evaluation
        if is_eligible_estimand_b:
            target_bar = bar_idx + 4
            if target_bar not in self.pending_directional_evals:
                self.pending_directional_evals[target_bar] = {}
            self.pending_directional_evals[target_bar][symbol] = {
                "decision_price": p_ref,
                "prediction": 1 if signal_entry else 0,
                "forecast_prob": card_b["forecast_probability_up"],
                "card": card_b
            }
        else:
            # Emit boundary-excluded card immediately
            self.append_jsonl(self.decision_cards_path, card_b)
            self.state.directional_cards_count += 1

        # ─── Check & Resolve Pending Estimand B Outcomes ──────
        if bar_idx in self.pending_directional_evals and symbol in self.pending_directional_evals[bar_idx]:
            pending = self.pending_directional_evals[bar_idx].pop(symbol)
            p_orig = pending["decision_price"]
            p_out = p_ref
            label = 1 if p_out > p_orig else 0
            resolved_card = pending["card"]
            resolved_card["status"] = "EVALUATED"
            resolved_card["outcome_timestamp_utc_ms"] = t_decision
            resolved_card["outcome_price"] = p_out
            resolved_card["directional_label"] = label
            # Brier score: (forecast_prob - label)^2
            resolved_card["brier_score"] = round((resolved_card["forecast_probability_up"] - label)**2, 6)
            self.append_jsonl(self.decision_cards_path, resolved_card)
            self.state.directional_cards_count += 1

        # ─── Estimand A: Trade Lifecycle Simulation ───────────
        # Management path: L1/tick midbar for SL/TP (TICK_PATH); durable bar
        # close uses OHLC path-through (not close-only) + horizon / square-off.
        # Stop/target *levels* remain frozen (Gate 5) — fidelity only.
        # Provisional midbar must NOT open/close (prevents day-OHLC re-entry churn).
        # PRIMARY (Gate2E) + BASELINE twin (ungated control) — never pooled.
        bar_hi = float(getattr(latest_bar, "high", p_ref) or p_ref)
        bar_lo = float(getattr(latest_bar, "low", p_ref) or p_ref)

        if mutate_trades:
            for book in ESTIMAND_BOOK_KEYS:
                active = self._estimand_active_trades(book)
                journaled = self._estimand_journaled_ids(book)
                if book == "primary":
                    stamp = pattern_stamp
                    gate_blocks = input_gate_blocks
                else:
                    # BASELINE + r50 + floor-lock: ungated twin stamp (entry parity).
                    stamp = pattern_stamp.get("baseline_parallel") or {}
                    gate_blocks = bool(coralys_blocks or stamp.get("blocks_entry"))

                # Mandatory square-off at Bar 23 close (15:00 IST)
                if bar_idx >= MANDATORY_SQUARE_OFF_BAR and symbol in active:
                    with self._estimand_a_lock:
                        trade = active.pop(symbol, None)
                        if trade is not None:
                            self._close_trade(
                                trade,
                                exit_price=p_ref,
                                exit_bar=bar_idx,
                                exit_reason="MANDATORY_INTRADAY_SQUARE_OFF",
                                exit_path="BAR_CLOSE",
                                exit_mark_source="BAR_CLOSE",
                                book=book,
                            )
                    continue

                # Check existing open trade for exit conditions (OHLC path-through).
                if symbol in active:
                    with self._estimand_a_lock:
                        trade = active.get(symbol)
                        if trade is None:
                            continue
                        entry_p = trade["entry_price"]
                        if bar_hi > trade["mfe_price"]:
                            trade["mfe_price"] = bar_hi
                        if bar_lo < trade["mae_price"]:
                            trade["mae_price"] = bar_lo
                        if p_ref > trade["mfe_price"]:
                            trade["mfe_price"] = p_ref
                        if p_ref < trade["mae_price"]:
                            trade["mae_price"] = p_ref

                        bars_held = bar_idx - trade["entry_bar_idx"]
                        if trade.get("stop_loss_price") is None and EXIT_POLICY == "DYNAMIC_OHLCV_STRUCTURE_V1":
                            fric = max(9.0, float(trade.get("half_spread_bps") or 2.5) * 2.0 + 8.0)
                            ub = self.resolve_exit_boundaries(symbol, entry_p, history, fric)
                            trade["stop_loss_price"] = ub["stop_price"]
                            trade["target_price"] = ub["target_price"]
                            trade["stop_loss_bps"] = ub["stop_bps"]
                            trade["target_bps"] = ub["target_bps"]
                            trade["atr_14"] = ub.get("atr_14")
                            trade["exit_policy"] = ub.get("policy_id")
                            trade["exit_trend_label"] = ub.get("trend_label")
                            trade["exit_fallback_reason"] = ub.get("fallback_reason")
                            trade["bounds_upgraded_at_bar"] = bar_idx
                        allow_ohlc_path = bars_held >= 1
                        try:
                            self._latch_nse_gate2x_bar_volume(trade, history)
                        except Exception:
                            pass

                        rust_bar = None
                        closed_this_bar = False
                        # R3: floor-lock twin is Rust-owned (params on trade).
                        if allow_ohlc_path:
                            rust_bar = self._try_rust_nse_plan_lifecycle(
                                trade,
                                mark=float(p_ref),
                                path_high=float(bar_hi),
                                path_low=float(bar_lo),
                                now_ms=int(t_decision),
                            )
                        if (
                            not closed_this_bar
                            and rust_bar is not None
                            and rust_bar.get("closed")
                        ):
                            _term, exit_reason = map_lifecycle_close(rust_bar, book="nse")
                            closed = active.pop(symbol)
                            if rust_bar.get("exit_reason_detail"):
                                closed["exit_reason_detail"] = rust_bar["exit_reason_detail"]
                            exit_px = float(
                                rust_bar["exit_price"]
                                if rust_bar.get("exit_price") is not None
                                else p_ref
                            )
                            if exit_reason == "STOP_LOSS":
                                path, src = "BAR_OHLC_PATH", "BAR_LOW"
                            elif exit_reason == "TAKE_PROFIT":
                                path, src = "BAR_OHLC_PATH", "BAR_HIGH"
                            elif exit_reason == "GATE2X_MFE_GIVEBACK_PROTECT":
                                path, src = "BAR_CLOSE_GATE2X", "BAR_CLOSE"
                            else:
                                path, src = "BAR_CLOSE", "BAR_CLOSE"
                            self._close_trade(
                                closed,
                                exit_price=exit_px,
                                exit_bar=bar_idx,
                                exit_reason=exit_reason,
                                exit_path=path,
                                exit_mark_source=src,
                                book=book,
                            )
                            closed_this_bar = True
                        elif not closed_this_bar and rust_bar is None:
                            # Track A: Python Adaptive/Gate2X/SL only if PLAN_LIFECYCLE=0.
                            _nse_require_block = bool(
                                plan_lifecycle_enabled() or plan_lifecycle_require_rust()
                            )
                            if not _nse_require_block:
                                try:
                                    giveback = self._gate2x_nse_update_mfe_giveback(
                                        trade, mark=float(p_ref), now_ms=int(t_decision)
                                    )
                                except Exception:
                                    giveback = None
                                if giveback:
                                    closed = active.pop(symbol)
                                    if ADAPTIVE_TP_ENABLED:
                                        closed["adaptive_tp_armed"] = True
                                        closed["exit_reason_detail"] = (
                                            f"ADAPTIVE_CORALYS_{giveback}"
                                        )
                                    self._close_trade(
                                        closed,
                                        exit_price=float(p_ref),
                                        exit_bar=bar_idx,
                                        exit_reason="GATE2X_MFE_GIVEBACK_PROTECT",
                                        exit_path="BAR_CLOSE_GATE2X",
                                        exit_mark_source="BAR_CLOSE",
                                        book=book,
                                    )
                                    closed_this_bar = True
                                else:
                                    if allow_ohlc_path:
                                        self._adaptive_tp_nse_try_extend(
                                            trade,
                                            mark=float(p_ref),
                                            path_high=float(bar_hi),
                                            now_ms=int(t_decision),
                                        )
                                    sl_price = float(
                                        trade.get("stop_loss_price")
                                        or (entry_p * ARM_A_STOP_MULT)
                                    )
                                    tp_price = float(
                                        trade.get("target_price")
                                        or (entry_p * ARM_A_TARGET_MULT)
                                    )
                                    hit_sl = allow_ohlc_path and bar_lo <= sl_price
                                    hit_tp = allow_ohlc_path and bar_hi >= tp_price
                                    if ADAPTIVE_TP_ENABLED:
                                        hit_tp = False
                                    if hit_sl:
                                        exit_reason = (
                                            "TRAILING_STOP"
                                            if trade.get("trailing_sl_armed")
                                            or trade.get("adaptive_tp_armed")
                                            else "STOP_LOSS"
                                        )
                                        self._close_trade(
                                            active.pop(symbol),
                                            exit_price=sl_price,
                                            exit_bar=bar_idx,
                                            exit_reason=exit_reason,
                                            exit_path="BAR_OHLC_PATH",
                                            exit_mark_source="BAR_LOW",
                                            book=book,
                                        )
                                        closed_this_bar = True
                                    elif hit_tp:
                                        self._close_trade(
                                            active.pop(symbol),
                                            exit_price=tp_price,
                                            exit_bar=bar_idx,
                                            exit_reason="TAKE_PROFIT",
                                            exit_path="BAR_OHLC_PATH",
                                            exit_mark_source="BAR_HIGH",
                                            book=book,
                                        )
                                        closed_this_bar = True

                        if not closed_this_bar and symbol in active:
                            if bars_held >= 12:
                                self._close_trade(
                                    active.pop(symbol),
                                    exit_price=p_ref,
                                    exit_bar=bar_idx,
                                    exit_reason="MAX_HOLD_HORIZON_180M",
                                    exit_path="BAR_CLOSE",
                                    exit_mark_source="BAR_CLOSE",
                                    book=book,
                                )
                            elif bars_held >= 4 and p_ref > entry_p:
                                self._close_trade(
                                    active.pop(symbol),
                                    exit_price=p_ref,
                                    exit_bar=bar_idx,
                                    exit_reason="TARGET_HORIZON_60M",
                                    exit_path="BAR_CLOSE",
                                    exit_mark_source="BAR_CLOSE",
                                    book=book,
                                )
                            else:
                                try:
                                    exit_stamp = self._match_exit_pattern_for_trade(
                                        symbol=symbol,
                                        trade=trade,
                                        bar_idx=bar_idx,
                                    )
                                    if self._estimand_entry_like_baseline(book):
                                        exit_stamp = exit_stamp.get("baseline_parallel") or exit_stamp
                                    trade["trade_pattern_exit"] = exit_stamp
                                    if (
                                        bars_held >= 1
                                        and exit_stamp.get("apply_early_exit")
                                        and pattern_exit_gate_enabled()
                                    ):
                                        self._close_trade(
                                            active.pop(symbol),
                                            exit_price=p_ref,
                                            exit_bar=bar_idx,
                                            exit_reason="PATTERN_REVERSE",
                                            exit_path="BAR_CLOSE_PATTERN",
                                            exit_mark_source="BAR_CLOSE",
                                            book=book,
                                        )
                                except Exception:
                                    pass
                    continue

                # Entry on durable bar close only when ENTRY_PATH=BAR_CLOSE (legacy).
                # TICK_PATH fills on each WS L1 tick after durable arms signal_bar (not here).
                if (
                    ENTRY_PATH == "BAR_CLOSE"
                    and signal_entry
                    and (not gate_blocks)
                    and (1 <= bar_idx <= MAX_NEW_ENTRY_BAR)
                    and symbol not in active
                ):
                    trade_id = self._estimand_trade_id(
                        book=book, signal_bar_idx=bar_idx, symbol=symbol
                    )
                    with self._estimand_a_lock:
                        if trade_id not in journaled and symbol not in active:
                            qty = max(1.0, math.floor(DEFAULT_NOTIONAL_PER_TRADE_INR / p_ref))
                            half_spread = quote.half_spread_bps if (quote and quote.half_spread_bps > 0) else 2.5
                            friction_proxy_bps = max(9.0, float(half_spread) * 2.0 + 8.0)
                            bounds = self.resolve_exit_boundaries(symbol, p_ref, history, friction_proxy_bps)
                            active[symbol] = {
                                "protocol_id": PROTOCOL_ID,
                                "estimand": "ESTIMAND_A",
                                "book": self._estimand_book_id(book),
                                "gate2x_ratchet_peak_frac": self._estimand_ratchet_frac(book),
                                "gate2x_protocol": self._estimand_gate2x_protocol(book),
                                "gate2x_floor_lock_immediate": self._estimand_book_is_g2x_floor_lock(
                                    book
                                ),
                                "gate2x_preset": self._estimand_gate2x_preset(book),
                                "trade_id": trade_id,
                                "symbol": symbol,
                                "side": "LONG",
                                "session_idx": self.state.session_idx,
                                "entry_bar_idx": bar_idx,
                                "entry_timestamp_utc_ms": t_decision,
                                "entry_price": p_ref,
                                "quantity": qty,
                                "half_spread_bps": half_spread,
                                "mfe_price": p_ref,
                                "mae_price": p_ref,
                                "forecast_prob": FORECAST_LONG_PROB,
                                "reference_training_prevalence": ESTIMAND_A_TRAIN_PREVALENCE,
                                "balanced_accuracy": None,
                                "stop_loss_price": bounds["stop_price"],
                                "target_price": bounds["target_price"],
                                "stop_loss_bps": bounds["stop_bps"],
                                "target_bps": bounds["target_bps"],
                                "atr_14": bounds.get("atr_14"),
                                "tp_level": 0,
                                "adaptive_tp_armed": False,
                                "adaptive_tp_protocol": ADAPTIVE_TP_PROTOCOL,
                                "exit_policy": bounds.get("policy_id"),
                                "exit_trend_label": bounds.get("trend_label"),
                                "exit_fallback_reason": bounds.get("fallback_reason"),
                                "entry_path": "BAR_CLOSE",
                                "management_path": "TICK_PATH",
                                "coralys_input_context": input_ctx,
                                "trade_pattern_match": stamp,
                                "regime_align": self._regime_align_for_long(
                                    book=book, history=history
                                ),
                            }

        self.state.active_positions_count = self._estimand_active_count()

        # ─── Scrip-Level Decision Record (RP-608 Observational Decision Ledger) ───
        # Determine explicit decision code and explanation matching exact evaluator branches
        if symbol in self.active_trades and self.active_trades[symbol].get("entry_bar_idx") != bar_idx:
            # Active from a prior bar takes precedence: accurately reflects active portfolio state
            decision_code = "EXISTING_POSITION_ACTIVE"
            explanation = f"Existing active trade open since bar {self.active_trades[symbol]['entry_bar_idx']}"
            decision = "NO_TRADE"
        elif bar_idx >= MANDATORY_SQUARE_OFF_BAR:
            decision_code = "INTRADAY_BOUNDARY_EXCLUDED"
            explanation = (
                f"Session bar {bar_idx} >= {MANDATORY_SQUARE_OFF_BAR} "
                f"(15:00–15:15 closing window / mandatory square-off)"
            )
            decision = "NO_TRADE"
        elif bar_idx > MAX_NEW_ENTRY_BAR:
            decision_code = "INTRADAY_BOUNDARY_EXCLUDED"
            explanation = (
                f"Session bar {bar_idx} > {MAX_NEW_ENTRY_BAR} "
                f"(entry cutoff so 60m horizon completes by 15:00 close-window)"
            )
            decision = "NO_TRADE"
        elif not g3_m["g3_ready"]:
            decision_code = "COLD_START_INSUFFICIENT_HISTORY"
            explanation = (
                f"Warmup history: {g3_m['g3_history_length']}/5 bars required "
                f"(policy={g3_m['g3_warmup_policy']}; session={g3_m['g3_session_history_length']})"
            )
            decision = "NO_TRADE"
        elif traj15 < 0.05:
            decision_code = "SUB_THRESHOLD_MOMENTUM"
            explanation = f"Momentum traj15={traj15:+.4f} below entry threshold +0.0500"
            decision = "NO_TRADE"
        elif not has_valid_quote:
            decision_code = "QUOTE_FEASIBILITY_FAILURE"
            if quote is None:
                explanation = "Missing top-of-book quote fails Gate G4 execution feasibility"
            elif not is_fresh:
                explanation = f"Stale top-of-book quote (age {quote_age_ms}ms > {MAX_QUOTE_AGE_MS}ms) fails Gate G4 execution feasibility"
            elif quote.bid_price <= 0 or quote.ask_price <= 0:
                explanation = "Zero top-of-book bid/ask fails Gate G4 execution feasibility"
            elif quote.ask_price <= quote.bid_price:
                explanation = "Crossed or locked top-of-book quote fails Gate G4 execution feasibility"
            else:
                explanation = "Invalid top-of-book quote fails Gate G4 execution feasibility"
            decision = "NO_TRADE"
        elif input_gate_blocks:
            if regime_blocks and not coralys_blocks and not pattern_blocks:
                _ra_guard = str(regime_align_stamp.get("guard") or "")
                if _ra_guard == "DI_SKEW" or "DI_SKEW" in str(
                    regime_align_stamp.get("block_reason") or ""
                ):
                    decision_code = "STANDASIDE_DI_SKEW"
                    explanation = (
                        f"Gate2E DI-skew standaside "
                        f"(regime={regime_align_stamp.get('regime_state')}; "
                        f"trend={regime_align_stamp.get('trend_dir')}; "
                        f"di_skew={regime_align_stamp.get('di_skew')}; "
                        f"reason={regime_align_stamp.get('block_reason')})"
                    )
                else:
                    decision_code = "STANDASIDE_REGIME_ALIGN"
                    explanation = (
                        f"Gate2E regime-align standaside "
                        f"(regime={regime_align_stamp.get('regime_state')}; "
                        f"trend={regime_align_stamp.get('trend_dir')}; "
                        f"reason={regime_align_stamp.get('block_reason')})"
                    )
            elif pattern_blocks and not coralys_blocks:
                decision_code = "STANDASIDE_TRADE_PATTERN"
                explanation = (
                    f"Past-trade entry pattern standaside "
                    f"(pattern_id={pattern_stamp.get('pattern_id')}; "
                    f"hist_wr={pattern_stamp.get('hist_wr')}; "
                    f"n={pattern_stamp.get('cohort_n')})"
                )
            else:
                decision_code = "STANDASIDE_CORALYS_INPUT_GATE"
                explanation = (
                    f"Coralys input gate blocks long entry "
                    f"(phase={input_ctx.get('session_phase')}; "
                    f"reason={input_ctx.get('standaside_reason')})"
                )
            decision = "NO_TRADE"
        else:
            decision_code = "QUALIFIED_MOMENTUM_BREAKOUT"
            explanation = (
                f"Momentum breakout traj15={traj15:+.4f} >= +0.0500 with "
                f"{g3_m['g3_history_length']} bars history "
                f"(policy={g3_m['g3_warmup_policy']}; session={g3_m['g3_session_history_length']})"
            )
            decision = "RECOMMEND_LONG"

        rr_dict = None
        inval_text = (
            f"p <= entry_price * {ARM_A_STOP_MULT:.3f} ({ARM_A_STOP_BPS:.0f} bps stop) "
            f"or 4-bar hold without gain or max 12 bars"
        )
        # Stamp ATR/structure geometry for traj-clear names even when Gate2E/Coralys
        # standaside (board high-target surface; does not authorize entry / Gate 5 frozen).
        _stamp_rr = decision == "RECOMMEND_LONG" or (
            float(traj15) >= 0.05
            and bool(has_valid_quote)
            and bool(g3_m.get("g3_ready"))
            and decision_code
            in (
                "STANDASIDE_TRADE_PATTERN",
                "STANDASIDE_CORALYS_INPUT_GATE",
                "STANDASIDE_REGIME_ALIGN",
                "QUALIFIED_MOMENTUM_BREAKOUT",
            )
        )
        if _stamp_rr:
            qty = max(1.0, math.floor(DEFAULT_NOTIONAL_PER_TRADE_INR / p_ref))
            half_spread = quote.half_spread_bps if quote else 2.5
            friction_proxy_bps = max(9.0, float(half_spread) * 2.0 + 8.0)
            # Prefer frozen bounds from the trade opened this bar; else resolve fresh.
            opened = self.active_trades.get(symbol)
            if opened and opened.get("entry_bar_idx") == bar_idx and opened.get("stop_loss_price"):
                bounds = {
                    "stop_price": opened["stop_loss_price"],
                    "target_price": opened["target_price"],
                    "stop_bps": opened.get("stop_loss_bps", ARM_A_STOP_BPS),
                    "target_bps": opened.get("target_bps", ARM_A_TARGET_BPS),
                    "atr_14": opened.get("atr_14"),
                    "policy_id": opened.get("exit_policy"),
                    "trend_label": opened.get("exit_trend_label"),
                    "fallback_reason": opened.get("exit_fallback_reason"),
                }
            else:
                bounds = self.resolve_exit_boundaries(symbol, p_ref, history, friction_proxy_bps)
            target_p = float(bounds["target_price"])
            stop_p = float(bounds["stop_price"])
            tgt_bps = float(bounds["target_bps"])
            sl_bps = float(bounds["stop_bps"])
            friction = calc_indian_equity_friction(
                buy_price=p_ref,
                sell_price=target_p,
                quantity=qty,
                half_spread_bps=half_spread
            )
            eff_fric = friction.get("effective_friction_bps", 4.8)
            tot_fric = friction.get("total_friction", 0.0)
            rr_dict = {
                "target_price": round(target_p, 2),
                "stop_loss_price": round(stop_p, 2),
                "target_bps": round(tgt_bps, 2),
                "stop_loss_bps": round(sl_bps, 2),
                "atr_14": bounds.get("atr_14"),
                "exit_policy": bounds.get("policy_id"),
                "trend_label": bounds.get("trend_label"),
                "fallback_reason": bounds.get("fallback_reason"),
                "estimated_friction_bps": round(eff_fric, 2),
                "estimated_friction_inr": round(tot_fric, 2),
                "net_reward_bps": round(tgt_bps - eff_fric, 2),
                "net_risk_bps": round(sl_bps + eff_fric, 2),
                "net_rr_ratio": round((tgt_bps - eff_fric) / (sl_bps + eff_fric + 1e-9), 2),
                "geometry_observational": decision != "RECOMMEND_LONG",
            }
            inval_text = (
                f"p <= ₹{stop_p:.2f} (−{sl_bps:.0f} bps, {bounds.get('policy_id')}) "
                f"or TP ₹{target_p:.2f} (+{tgt_bps:.0f} bps) or 4-bar/12-bar horizon"
            )

        now_eval_ms = int(time.time() * 1000)
        q_ts = quote.timestamp_utc_ms if quote else None
        q_age_ms = (now_eval_ms - q_ts) if q_ts else None

        # Observational OF inspection for every symbol (0% authority; does not gate entry).
        # Primary side = open trade side when held, else LONG (NSE Arm A is LONG-only).
        active_side = "LONG"
        if symbol in self.active_trades:
            active_side = str(self.active_trades[symbol].get("side") or "LONG").upper() or "LONG"
        of_insp = inspect_order_flow_book(
            l1_from_nse_quote(quote, source="kite_l1"),
            primary_side=active_side,
        )

        eval_record = {
            "symbol": symbol,
            "tradingsymbol": symbol.replace(".NS", ""),
            "exchange": "NSE",
            "session_idx": self.state.session_idx,
            "bar_idx": bar_idx,
            "bar_timestamp_utc_ms": latest_bar.timestamp_utc_ms,
            "bar_timestamp_iso": datetime.fromtimestamp(latest_bar.timestamp_utc_ms / 1000.0, timezone.utc).isoformat(),
            "quote_timestamp_utc_ms": q_ts,
            "quote_age_ms": q_age_ms,
            "reference_price": round(p_ref, 2),
            "decision": decision,
            "decision_code": decision_code,
            "explanation": explanation,
            "score": round(traj15, 4),
            "provisional_rank": None,  # Computed across all qualifying candidates in run_single_iteration
            # history_length = G3 warmup length (may include prior session under Policy C).
            "history_length": int(g3_m["g3_history_length"]),
            "session_history_length": int(g3_m["g3_session_history_length"]),
            "g3_ready": bool(g3_m["g3_ready"]),
            "g3_warmup_policy": g3_m["g3_warmup_policy"],
            "g3_warmup_policy_id": g3_m["g3_warmup_policy_id"],
            "overnight_gap_bps": g3_m.get("overnight_gap_bps"),
            "score_kind": g3_m.get("score_kind"),
            "bar_provenance": getattr(latest_bar, "source", "kite_live"),
            "history_provenance": {
                "live_count": sum(1 for b in history if getattr(b, "source", "kite_live") in ("kite_live", "persisted_live")),
                "historical_bootstrap_count": sum(1 for b in history if getattr(b, "source", "") == "kite_historical_same_day"),
                "g3_warmup_policy": g3_m["g3_warmup_policy"],
                "g3_history_length": int(g3_m["g3_history_length"]),
                "g3_session_history_length": int(g3_m["g3_session_history_length"]),
            },
            "half_spread_bps": round(quote.half_spread_bps, 2) if quote else None,
            "entry_conditions": (
                f"traj15 >= 0.05 and g3_history >= 5 (policy={g3_m['g3_warmup_policy']}) "
                f"and 1 <= bar_idx <= {MAX_NEW_ENTRY_BAR} "
                f"and not active_trade (60m completes by 15:00 close-window)"
            ),
            "invalidation": inval_text,
            "risk_reward": rr_dict,
            "exit_policy": EXIT_POLICY,
            "data_quality": {
                "status": "VALID_L1" if (quote and quote.bid_price > 0 and quote.ask_price > 0) else "DEGRADED",
                "feed_source": "kite_connect",
                "bid_qty": quote.bid_qty if quote else 0,
                "ask_qty": quote.ask_qty if quote else 0
            },
            "order_flow_supported": of_insp["order_flow_supported"],
            "order_flow_label": of_insp["order_flow_label"],
            "order_flow_imbalance": of_insp["order_flow_imbalance"],
            "order_flow_source": of_insp["order_flow_source"],
            "order_flow_side": of_insp["order_flow_side"],
            "order_flow_long_supported": of_insp["order_flow_long_supported"],
            "order_flow_short_supported": of_insp["order_flow_short_supported"],
            "order_flow_inspection": True,
            "arm_a_frozen": ARM_A_FROZEN,
            "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
            "horizon_class": "INTRADAY",
            "horizon_id": "H0_NSE_INTRADAY",
            "horizon_label": "Intraday",
            "horizon_term": (
                f"15m bars · 60m / 180m max · square-off 15:00 "
                f"(bar {MANDATORY_SQUARE_OFF_BAR}; close-window 15:00–15:15)"
            ),
            "closing_window_ist": "15:00-15:15",
            "max_new_entry_bar": MAX_NEW_ENTRY_BAR,
            "mandatory_square_off_bar": MANDATORY_SQUARE_OFF_BAR,
            "coralys_input_context": input_ctx,
            "coralys_input_gate_enabled": coralys_input_gate_enabled(),
            "coralys_input_gate_blocks": bool(coralys_blocks),
            "trade_pattern_match": pattern_stamp,
            "trade_pattern_gate_enabled": pattern_entry_gate_enabled(),
            "trade_pattern_gate_mode": pattern_entry_gate_mode(),
            "trade_pattern_gate_blocks": bool(pattern_blocks),
            "regime_align": regime_align_stamp,
            "regime_align_blocks": bool(regime_blocks),
        }
        # Observational regime telemetry on completed-bar history only (Phase 1–2).
        # Must not alter decision / entry / exit / cost fields above.
        if REGIME_TELEMETRY_ENABLED:
            regime = classify_market_regime(history)
            # Phase 2: NSE two-level MARKET_CHOP fusion (observational; no Arm A filter).
            sess_rets = []
            for hist in self.price_history.values():
                r = session_return_bps_from_bars(hist)
                if r is not None:
                    sess_rets.append(r)
            mctx = compute_market_context(sess_rets)
            regime = apply_market_chop_to_regime(regime, mctx)
            eval_record = attach_regime_telemetry(eval_record, regime, enabled=True)
            eval_record["market_chop"] = mctx.get("market_chop")
            eval_record["session_breadth"] = mctx.get("session_breadth")
            eval_record["market_return_bps"] = mctx.get("market_return_bps")
            eval_record["market_dispersion_bps"] = mctx.get("market_dispersion_bps")
            eval_record["directional_er_hurdle"] = regime.get("directional_er_hurdle")
        else:
            eval_record = attach_regime_telemetry(eval_record, None, enabled=False)
        card_b["evaluation"] = eval_record
        self.last_evaluations[symbol] = eval_record

        # Gate2E observe: PRIMARY ABSTAIN_INSUFFICIENT suppress (stamp-only).
        if (
            decision_code == "STANDASIDE_TRADE_PATTERN"
            and pattern_blocks
            and matches_shadow_observe_abstain_insufficient(pattern_stamp)
        ):
            try:
                self._maybe_log_gate2e_shadow_insufficient(
                    symbol=symbol,
                    pattern_stamp=pattern_stamp,
                    suppressed_ms=int(t_decision) if t_decision else int(time.time() * 1000),
                    decision_bar_idx=int(bar_idx),
                    source="live_suppress",
                    direction="LONG",
                    session_phase=(
                        str(input_ctx.get("session_phase"))
                        if isinstance(input_ctx, dict) and input_ctx.get("session_phase")
                        else None
                    ),
                )
            except Exception:
                pass

        # Prospective V1 research journals (0.0% authority; only if NSE STARTED):
        # 1) full per-symbol recommendation tape (all decisions, not only RECOMMEND_LONG)
        # 2) recommendation ↔ Estimand A trade join ledger
        # 3) Arm A/B/C observer (RECOMMEND_LONG as Arm A)
        try:
            sess = self.current_session_date or getattr(self.state, "current_session_date", "") or ""
            if hasattr(sess, "isoformat"):
                sess = sess.isoformat()
            sess = str(sess)
            ask_v = bid_v = age_v = None
            if has_valid_quote and quote is not None:
                ask_v = float(quote.ask_price)
                bid_v = float(quote.bid_price)
                age_v = float(q_age_ms) if q_age_ms is not None else None
            regime_label = None
            if isinstance(eval_record, dict):
                regime_label = eval_record.get("regime_state") or (
                    (eval_record.get("regime") or {}).get("regime")
                    if isinstance(eval_record.get("regime"), dict)
                    else eval_record.get("regime")
                )
            prospective_v1_record_recommendation(
                market=ProspectiveMarket.NSE,
                symbol=symbol,
                decision=str(decision),
                decision_code=str(decision_code),
                session_date=sess,
                bar_idx=bar_idx,
                close_time_ms=int(latest_bar.timestamp_utc_ms),
                reference_price=float(p_ref),
                is_actionable=(decision == "RECOMMEND_LONG"),
                regime=str(regime_label) if regime_label is not None else None,
                extra={
                    "decision_code": decision_code,
                    "explanation": explanation,
                    "session_idx": self.state.session_idx,
                    "score": eval_record.get("score") if isinstance(eval_record, dict) else None,
                    "quote_age_ms": q_age_ms,
                    "half_spread_bps": eval_record.get("half_spread_bps") if isinstance(eval_record, dict) else None,
                },
            )
            opened_trade = None
            blocking_trade = None
            if symbol in self.active_trades:
                active = self.active_trades[symbol]
                if int(active.get("entry_bar_idx", -1)) == int(bar_idx):
                    opened_trade = active
                else:
                    blocking_trade = active
            prospective_v1_append_nse_rec_trade(
                symbol=symbol,
                session_date=sess,
                bar_idx=bar_idx,
                decision=str(decision),
                decision_code=str(decision_code),
                close_time_ms=int(latest_bar.timestamp_utc_ms),
                reference_price=float(p_ref),
                is_actionable=(decision == "RECOMMEND_LONG"),
                regime=str(regime_label) if regime_label is not None else None,
                session_idx=int(self.state.session_idx),
                opened_trade=opened_trade,
                blocking_trade=blocking_trade,
                extra={
                    "decision_code": decision_code,
                    "explanation": explanation,
                    "session_idx": self.state.session_idx,
                    "score": eval_record.get("score") if isinstance(eval_record, dict) else None,
                },
            )
            prospective_v1_observe_bar(
                market=ProspectiveMarket.NSE,
                completed_bars_including_signal=history,
                ask=ask_v,
                bid=bid_v,
                quote_age_ms=age_v,
                session_date=sess,
                symbol=symbol,
                bar_idx=bar_idx,
                close_time_ms=int(latest_bar.timestamp_utc_ms),
                arm_a_signal=(decision == "RECOMMEND_LONG"),
                arm_a_side="LONG" if decision == "RECOMMEND_LONG" else None,
                reference_price=float(p_ref),
                extra={
                    "decision": decision,
                    "decision_code": decision_code,
                    "session_idx": self.state.session_idx,
                },
            )
        except Exception as obs_exc:
            print(f"[prospective_v1] NSE observe skipped ({symbol}): {obs_exc}")

        # NSE OF PAPER + OFX volume PAPER — arm decision for per-tick WS action;
        # also run once here so bar-close path stays live if WS is quiet.
        sq_bar = int(
            getattr(self._active_session_window, "square_off_bar", MANDATORY_SQUARE_OFF_BAR)
            or MANDATORY_SQUARE_OFF_BAR
        )
        if (
            bar_idx < sq_bar
            and str(decision) in ("RECOMMEND_LONG", "RECOMMEND_SHORT")
        ):
            self._paper_armed_eval_by_symbol[symbol] = dict(eval_record)
        elif bar_idx >= sq_bar:
            self._paper_armed_eval_by_symbol.pop(symbol, None)
        try:
            self.of_paper.evaluate_open_tick(
                symbol=symbol,
                quote=quote,
                now_ms=now_eval_ms,
                bar_idx=bar_idx,
                force_reason=(
                    "SESSION_SQUARE_OFF"
                    if bar_idx >= sq_bar
                    else None
                ),
            )
            if bar_idx < sq_bar:
                self.of_paper.try_open_from_eval(
                    symbol=symbol,
                    eval_record=eval_record,
                    quote=quote,
                    now_ms=now_eval_ms,
                )
        except Exception as ofp_exc:
            print(f"[nse_of_paper] eval hook skipped ({symbol}): {ofp_exc}")

        # OFX NSE volume PAPER — DERIVED_VALIDATED aggressor volume (baseline L1 OF inert)
        try:
            vol_snap = build_ofx_nse_volume_flow_snapshot(
                symbol=symbol,
                buffer=self.ofx_nse_volume_buf,
                now_ms=now_eval_ms,
            )
            write_ofx_nse_volume_flow_snapshot(self.ofx_tick_dir, vol_snap)
            self.ofx_nse_volume_paper.evaluate_open_tick(
                symbol=symbol,
                volume_snapshot=vol_snap,
                quote=quote,
                now_ms=now_eval_ms,
                force_reason=("SESSION_SQUARE_OFF" if bar_idx >= sq_bar else None),
            )
            if bar_idx < sq_bar:
                self.ofx_nse_volume_paper.try_open_from_eval(
                    symbol=symbol,
                    eval_record=eval_record,
                    volume_snapshot=vol_snap,
                    quote=quote,
                    now_ms=now_eval_ms,
                )
        except Exception as ofx_vol_exc:
            print(f"[ofx_nse_volume_paper] eval hook skipped ({symbol}): {ofx_vol_exc}")

        # Next-session indicative levels: structure on completed bar; validity on price.
        try:
            self._update_next_session_watch_levels(
                symbol=symbol,
                last_price=float(p_ref),
                completed_bar=True,
                bar_timestamp_utc_ms=int(latest_bar.timestamp_utc_ms),
                feed_stale=bool(
                    (q_age_ms is not None and q_age_ms > 120_000)
                    or (quote is None)
                ),
            )
        except Exception as nsr_lvl_exc:
            print(f"[next_session_levels] bar update skipped ({symbol}): {nsr_lvl_exc}")

        return card_b

    @staticmethod
    def _estimand_a_mark_from_feed(res: Any) -> Tuple[Optional[float], str]:
        """Prefer L1 last (quote snapshot bar.close); else mid. Fidelity only."""
        if not isinstance(res, dict):
            return None, "NONE"
        bars = res.get("bars") or []
        last = None
        if bars:
            b0 = bars[0]
            try:
                last = float(getattr(b0, "close", None) or (b0.get("close") if isinstance(b0, dict) else 0) or 0)
            except (TypeError, ValueError, AttributeError):
                last = None
        quote = res.get("quote")
        mid = None
        if quote is not None:
            try:
                bid = float(getattr(quote, "bid_price", 0) or 0)
                ask = float(getattr(quote, "ask_price", 0) or 0)
                if bid > 0 and ask > 0:
                    mid = (bid + ask) / 2.0
            except (TypeError, ValueError):
                mid = None
        if last is not None and last > 0:
            return last, "LAST"
        if mid is not None and mid > 0:
            return mid, "MID"
        return None, "NONE"

    def _session_bars_for_g3(self, symbol: str) -> List[Any]:
        """Completed session bars only (exclude incomplete quote-snapshot clones)."""
        history = self.price_history.get(symbol) or []
        session_bars = [
            b
            for b in history
            if not (
                str(getattr(b, "source", "")) == "kite_quote_snapshot"
                and not bool(getattr(b, "is_completed", True))
            )
        ]
        return session_bars

    def _try_estimand_a_tick_entry(
        self,
        *,
        symbol: str,
        signal_bar_idx: int,
        quote: Optional[Any],
        mark: float,
        mark_src: str,
        now_ms: int,
        book: str = "primary",
    ) -> bool:
        """
        Tick-path entry: traj15 from completed bars; fill at live L1/last.
        One trade_id per session/signal_bar/symbol/book (no bar-close fill).
        PRIMARY and BASELINE are independent books (never pooled).
        """
        if ENTRY_PATH != "TICK_PATH":
            return False
        active = self._estimand_active_trades(book)
        journaled = self._estimand_journaled_ids(book)
        if symbol in active:
            return False
        if not (1 <= int(signal_bar_idx) <= MAX_NEW_ENTRY_BAR):
            return False
        if quote is None:
            return False
        try:
            bid = float(getattr(quote, "bid_price", 0) or 0)
            ask = float(getattr(quote, "ask_price", 0) or 0)
        except (TypeError, ValueError):
            return False
        if bid <= 0 or ask <= 0 or ask <= bid:
            return False

        trade_id = self._estimand_trade_id(
            book=book, signal_bar_idx=int(signal_bar_idx), symbol=symbol
        )
        if trade_id in journaled:
            return False

        session_bars = self._session_bars_for_g3(symbol)
        sess_date = (
            self.current_session_date
            or getattr(self.state, "current_session_date", "")
            or ""
        )
        if hasattr(sess_date, "isoformat"):
            sess_date = sess_date.isoformat()
        sess_date = str(sess_date)[:10]
        feature_bars = session_bars
        if str(G3_WARMUP_POLICY).strip().lower() in {
            POLICY_CROSS_GAP_NEUTRAL_V1,
            "c",
            "policy_c",
            "cross_gap_neutral",
        }:
            feature_bars = self._feature_bars_for_boundaries(symbol, session_bars)
        g3_m = evaluate_g3_momentum(
            policy=G3_WARMUP_POLICY,
            session_bars=session_bars,
            feature_bars=feature_bars,
            today_session_date=sess_date,
        )
        traj15 = float(g3_m["traj15"])
        signal_entry = bool(g3_m["signal_long"])
        if not signal_entry:
            return False

        decision_ts_ist = datetime.now(timezone.utc).astimezone(
            timezone(timedelta(hours=5, minutes=30))
        )
        if not sess_date:
            sess_date = decision_ts_ist.date().isoformat()
        input_ctx = self.coralys_input_store.evaluate(
            symbol=symbol,
            session_date=sess_date,
            decision_ts_ist=decision_ts_ist,
            traj15=traj15,
            signal_long=True,
        )
        pattern_full = self._match_trade_pattern(
            input_ctx=input_ctx,
            traj15=traj15,
            bar_idx=int(signal_bar_idx),
            decision_ts_ist=decision_ts_ist,
            symbol=symbol,
        )
        if self._estimand_entry_like_baseline(book):
            pattern_stamp = pattern_full.get("baseline_parallel") or pattern_full
        else:
            pattern_stamp = pattern_full
        if coralys_input_gate_enabled() and not bool(input_ctx.get("allow_long_entry")):
            return False
        if pattern_stamp.get("blocks_entry"):
            # PRIMARY cold-start observe (TICK_PATH); BASELINE/r50 may still open.
            if not self._estimand_entry_like_baseline(book):
                try:
                    self._maybe_log_gate2e_shadow_insufficient(
                        symbol=symbol,
                        pattern_stamp=pattern_stamp
                        if pattern_stamp is pattern_full
                        else pattern_full,
                        suppressed_ms=int(now_ms),
                        decision_bar_idx=int(signal_bar_idx),
                        source="tick_path_suppress",
                        direction="LONG",
                        session_phase=(
                            str(input_ctx.get("session_phase"))
                            if isinstance(input_ctx, dict) and input_ctx.get("session_phase")
                            else None
                        ),
                    )
                except Exception:
                    pass
            return False

        history = self.price_history.get(symbol) or []
        regime_align = self._regime_align_for_long(book=book, history=history)
        if regime_align.get("blocks_entry"):
            return False

        half_spread = float(getattr(quote, "half_spread_bps", 0) or 0) or 2.5
        friction_proxy_bps = max(9.0, float(half_spread) * 2.0 + 8.0)
        bounds = self.resolve_exit_boundaries(symbol, mark, history, friction_proxy_bps)
        qty = max(1.0, math.floor(DEFAULT_NOTIONAL_PER_TRADE_INR / mark))
        book_id = self._estimand_book_id(book)
        ratchet_frac = self._estimand_ratchet_frac(book)
        active[symbol] = {
            "protocol_id": PROTOCOL_ID,
            "estimand": "ESTIMAND_A",
            "book": book_id,
            "gate2x_ratchet_peak_frac": ratchet_frac,
            "gate2x_protocol": self._estimand_gate2x_protocol(book),
            "gate2x_floor_lock_immediate": self._estimand_book_is_g2x_floor_lock(book),
            "gate2x_preset": self._estimand_gate2x_preset(book),
            "trade_id": trade_id,
            "symbol": symbol,
            "side": "LONG",
            "session_idx": self.state.session_idx,
            "entry_bar_idx": int(signal_bar_idx),
            "entry_timestamp_utc_ms": int(now_ms),
            "entry_price": float(mark),
            "quantity": qty,
            "half_spread_bps": half_spread,
            "mfe_price": float(mark),
            "mae_price": float(mark),
            "forecast_prob": FORECAST_LONG_PROB,
            "reference_training_prevalence": ESTIMAND_A_TRAIN_PREVALENCE,
            "balanced_accuracy": None,
            "stop_loss_price": bounds["stop_price"],
            "target_price": bounds["target_price"],
            "stop_loss_bps": bounds["stop_bps"],
            "target_bps": bounds["target_bps"],
            "atr_14": bounds.get("atr_14"),
            "tp_level": 0,
            "adaptive_tp_armed": False,
            "adaptive_tp_protocol": ADAPTIVE_TP_PROTOCOL,
            "exit_policy": bounds.get("policy_id"),
            "exit_trend_label": bounds.get("trend_label"),
            "exit_fallback_reason": bounds.get("fallback_reason"),
            "entry_path": "TICK_PATH",
            "entry_mark_source": mark_src,
            "management_path": "TICK_PATH",
            "traj15_at_entry": round(traj15, 6),
            "coralys_input_context": input_ctx,
            "trade_pattern_match": pattern_stamp,
            "regime_align": regime_align,
            "entry_trade_like": True,
        }
        return True

    def _rebuild_token_to_symbol(self) -> None:
        """Map Kite instrument tokens → universe symbols for WS tick fills."""
        out: Dict[int, str] = {}
        imap = getattr(self.feed_adapter, "instrument_map", None) or {}
        for sym, pair in imap.items():
            try:
                out[int(pair[0])] = str(sym)
            except Exception:
                continue
        self._token_to_symbol = out

    def _persist_estimand_a_pending(self, *, force: bool = False) -> None:
        """Debounced pending-trade persist for WS tick path."""
        now_m = time.monotonic()
        if (not force) and (now_m - float(self._estimand_a_last_persist_mono or 0.0) < 1.0):
            return
        self._estimand_a_last_persist_mono = now_m
        self.state.active_positions_count = self._estimand_active_count()
        try:
            self.save_pending_evaluations()
        except Exception as exc:
            print(f"[estimand_a_tick] persist skipped: {exc}")

    @staticmethod
    def _gate2x_nse_friction_proxy_bps(trade: Dict[str, Any]) -> float:
        half = float(trade.get("half_spread_bps") or 2.5)
        return max(9.0, half * 2.0 + 8.0)

    def _adaptive_tp_nse_try_extend(
        self,
        trade: Dict[str, Any],
        *,
        mark: float,
        path_high: Optional[float] = None,
        now_ms: Optional[int] = None,
    ) -> bool:
        """
        Gate 5 Adaptive (NSE LONG): Coralys-intelligent floor trail.

        Ratchets SL only to Gate2X/Coralys protect|ratchet floors (INR→price).
        Does **not** blind-ATR chase MFE; does **not** latch on TP touch.
        Coralys Protect/Exit (via Gate2X) owns reversal-doubt exits.
        Parity with Rust `apply_adaptive_coralys_floor_trail`.
        Track A: no-op when PLAN_LIFECYCLE=1 (Rust owns Adaptive).
        """
        if plan_lifecycle_enabled():
            return False
        if not ADAPTIVE_TP_ENABLED:
            return False
        _ = path_high
        try:
            entry_p = float(trade.get("entry_price") or 0.0)
            qty = float(trade.get("quantity") or 0.0)
            sl = float(
                trade.get("stop_loss_price")
                or (entry_p * ARM_A_STOP_MULT if entry_p else 0.0)
            )
            orig = trade.get("original_stop_loss_price")
            if orig is None:
                orig = sl
            orig = float(orig)
            mfe_px = float(trade.get("mfe_price") or mark or 0.0)
        except (TypeError, ValueError):
            return False
        if entry_p <= 0 or qty <= 0:
            return False
        trade["original_stop_loss_price"] = orig

        # Prefer Coralys floors; optional legacy dumb ATR only if explicitly enabled.
        floor_inr = trade.get("gate2x_protect_floor_inr")
        if floor_inr is None:
            floor_inr = trade.get("gate2x_ratchet_floor_inr")
        armed = bool(trade.get("gate2x_mfe_armed") or trade.get("adaptive_tp_armed"))
        proposed = None
        if armed and floor_inr is not None:
            try:
                floor_f = float(floor_inr)
            except (TypeError, ValueError):
                floor_f = 0.0
            if floor_f > 0:
                proposed = round(max(orig, entry_p + floor_f / qty), 2)

        if proposed is None and ADAPTIVE_DUMB_ATR_TRAIL:
            atr = float(trade.get("atr_14") or 0.0)
            risk = entry_p - orig
            fav = mfe_px - entry_p
            if risk > 0 and fav >= risk * ADAPTIVE_TRAIL_ARM_RISK_FRAC:
                trail_dist = (
                    ADAPTIVE_TRAIL_ATR_MULT * atr
                    if atr > 0
                    else ADAPTIVE_TP_EXTEND_PCT * entry_p
                )
                if trail_dist > 0:
                    proposed = round(max(orig, mfe_px - trail_dist), 2)

        if proposed is None or proposed <= sl or proposed >= mfe_px:
            if armed:
                trade["adaptive_tp_armed"] = True
            return False
        ts = int(now_ms if now_ms is not None else time.time() * 1000)
        trade["stop_loss_price"] = proposed
        trade["adaptive_tp_armed"] = True
        trade["trailing_sl_armed"] = True
        trade["adaptive_partial_latch_armed"] = False
        trade["adaptive_tp_touch_ms"] = ts
        trade["adaptive_tp_protocol"] = ADAPTIVE_TP_PROTOCOL
        trade["stop_loss_bps"] = round((1.0 - proposed / entry_p) * 10000.0, 2)
        return True

    def _gate2x_nse_arm_threshold_inr(self, trade: Dict[str, Any]) -> float:
        entry_p = float(trade.get("entry_price") or 0.0)
        qty = float(trade.get("quantity") or 0.0)
        notional = entry_p * qty if entry_p > 0 and qty > 0 else DEFAULT_NOTIONAL_PER_TRADE_INR
        fric_bps = self._gate2x_nse_friction_proxy_bps(trade)
        floor_bps = float(GATE2X_MEANINGFUL_FLOOR_BPS)
        return notional * (max(fric_bps, floor_bps) / 10000.0)

    def _gate2x_nse_update_mfe_giveback(
        self,
        trade: Dict[str, Any],
        *,
        mark: float,
        now_ms: int,
    ) -> Optional[str]:
        """
        Gate 2X NSE feature construction + adapter decide (Coralys).

        Peak MFE/tp_frac tracked locally; Protect/Exit only from
        `rp608_position_decide` → `gate2x_decide` → RecommendationEngineV1.
        Track A: no-op when PLAN_LIFECYCLE=1 (Rust owns Gate2X manage).
        """
        if plan_lifecycle_enabled():
            return None
        if not GATE2X_MFE_GIVEBACK_ENABLED:
            return None
        try:
            entry_p = float(trade.get("entry_price") or 0.0)
            qty = float(trade.get("quantity") or 0.0)
            mfe_px = float(trade.get("mfe_price") or entry_p)
            tp_price = float(
                trade.get("target_price") or (entry_p * ARM_A_TARGET_MULT if entry_p else 0.0)
            )
        except (TypeError, ValueError):
            return None
        if entry_p <= 0 or qty <= 0 or mark <= 0:
            return None

        # Estimand A is LONG-only — feature construction only.
        peak_mfe_inr = (mfe_px - entry_p) * qty
        prev_peak = trade.get("gate2x_peak_mfe_inr")
        try:
            prev_peak_f = float(prev_peak) if prev_peak is not None else None
        except (TypeError, ValueError):
            prev_peak_f = None
        if prev_peak_f is None or peak_mfe_inr > prev_peak_f:
            trade["gate2x_peak_mfe_inr"] = round(peak_mfe_inr, 4)

        tp_span = tp_price - entry_p
        tp_frac = ((mfe_px - entry_p) / tp_span) if tp_span > 0 else None
        if tp_frac is not None:
            prev_tf = trade.get("gate2x_peak_tp_frac")
            try:
                prev_tf_f = float(prev_tf) if prev_tf is not None else None
            except (TypeError, ValueError):
                prev_tf_f = None
            if prev_tf_f is None or tp_frac > prev_tf_f:
                trade["gate2x_peak_tp_frac"] = round(tp_frac, 4)

        threshold = self._gate2x_nse_arm_threshold_inr(trade)
        try:
            entry_ms = int(trade.get("entry_timestamp_utc_ms") or 0)
            hold_s = (int(now_ms) - entry_ms) / 1000.0 if entry_ms > 0 else 0.0
        except (TypeError, ValueError):
            hold_s = 0.0

        cur_mfe_inr = (float(mark) - entry_p) * qty
        trade["gate2x_current_mfe_inr"] = round(cur_mfe_inr, 4)

        since = trade.get("gate2x_soft_breach_since_ms")
        try:
            since_i = int(since) if since is not None else None
        except (TypeError, ValueError):
            since_i = None

        instrument = str(
            trade.get("tradingsymbol")
            or trade.get("symbol")
            or trade.get("instrument")
            or "RELIANCE.NS"
        )
        try:
            tp_level_i = int(trade.get("tp_level") or 0)
        except (TypeError, ValueError):
            tp_level_i = 0
        snap = gate2x_build_snapshot(
            decision_id=str(trade.get("trade_id") or trade.get("decision_id") or "NSE"),
            instrument=instrument,
            direction="LONG",
            reference_price=entry_p,
            peak_mfe=float(trade.get("gate2x_peak_mfe_inr") or peak_mfe_inr),
            current_mfe=float(cur_mfe_inr),
            peak_tp_frac=float(trade.get("gate2x_peak_tp_frac") or 0.0),
            hold_s=hold_s,
            arm_threshold=threshold,
            already_armed=bool(trade.get("gate2x_mfe_armed")),
            vol_surge=str(
                trade.get("gate2x_exit_vol_surge")
                or trade.get("exit_vol_surge")
                or "UNK"
            ),
            soft_breach_since_ms=since_i,
            now_ms=int(now_ms),
            protocol=self._estimand_gate2x_protocol(str(trade.get("book") or "primary")),
            tp_level=tp_level_i,
            adaptive_armed=bool(trade.get("adaptive_tp_armed")) or tp_level_i >= 1,
            params={
                "enabled": True,
                "arm_min_tp_frac": float(GATE2X_ARM_MIN_TP_FRAC),
                "min_hold_s_before_exit": float(GATE2X_MIN_HOLD_S_BEFORE_EXIT),
                "meaningful_floor_bps": float(GATE2X_MEANINGFUL_FLOOR_BPS),
                "protect_floor": GATE2X_PROTECT_FLOOR_MODE,
                "reversal_volume_confirm": bool(GATE2X_REVERSAL_VOLUME_CONFIRM),
                "immediate_vol_surges": sorted(GATE2X_REVERSAL_IMMEDIATE_SURGES),
                "med_vol_dwell_ms": int(GATE2X_REVERSAL_MED_VOL_DWELL_MS),
                "low_vol_dwell_ms": int(GATE2X_REVERSAL_LOW_VOL_DWELL_MS),
                "ratchet_peak_frac": float(
                    self._estimand_ratchet_frac(str(trade.get("book") or "primary"))
                ),
                "deep_giveback_frac": float(GATE2X_DEEP_GIVEBACK_FRAC),
                "deep_giveback_dwell_scale": float(GATE2X_DEEP_GIVEBACK_DWELL_SCALE),
            },
        )
        evidence = os.environ.get("RP608_NSE_REC001H_DIR", DEFAULT_NSE_EVIDENCE)
        action, reason, soft = gate2x_adapter_decide(snap, evidence_dir=evidence)
        apply_soft_breach_to_trade(trade, soft)
        if soft.get("protect_floor") is not None:
            trade["gate2x_protect_floor_inr"] = round(float(soft["protect_floor"]), 4)
        if soft.get("ratchet_floor") is not None:
            trade["gate2x_ratchet_floor_inr"] = round(float(soft["ratchet_floor"]), 4)
        if soft.get("armed") and not trade.get("gate2x_mfe_armed"):
            trade["gate2x_mfe_armed"] = True
            trade["gate2x_mfe_armed_threshold_inr"] = round(threshold, 4)
            trade["gate2x_mfe_armed_tp_frac"] = round(
                float(trade.get("gate2x_peak_tp_frac") or 0.0), 4
            )
            trade["gate2x_protocol"] = self._estimand_gate2x_protocol(
                str(trade.get("book") or "primary")
            )
            trade["gate2x_ratchet_peak_frac"] = float(
                self._estimand_ratchet_frac(str(trade.get("book") or "primary"))
            )

        if action in ("Protect", "Exit"):
            detail = reason or "MFE_GIVEBACK_TO_COST_CLEAR"
            trade["exit_reason_detail"] = detail
            trade["gate2x_adapter_action"] = action
            trade["gate2x_adapter_authority"] = "chronosentiment_adapter+coralys"
            return detail
        return None

    def _latch_nse_gate2x_bar_volume(
        self, trade: Dict[str, Any], history: List[Any]
    ) -> None:
        """Latch 15m volume burst vs SMA20 onto trade for Gate 2X confirm."""
        try:
            vols = [float(getattr(b, "volume", 0.0) or 0.0) for b in (history or [])]
            vols = [v for v in vols if v > 0]
            if len(vols) < 2:
                trade["gate2x_exit_vol_surge"] = "UNK"
                return
            cur = vols[-1]
            prior = vols[-21:-1] if len(vols) > 21 else vols[:-1]
            if not prior:
                trade["gate2x_exit_vol_surge"] = "UNK"
                return
            sma = sum(prior) / len(prior)
            burst = (cur / sma) if sma > 0 else None
            trade["gate2x_exit_vol_burst_20"] = round(float(burst), 6) if burst else None
            trade["gate2x_exit_vol_surge"] = _nse_vol_surge_bucket(burst)
        except Exception:
            trade["gate2x_exit_vol_surge"] = trade.get("gate2x_exit_vol_surge") or "UNK"

    def _try_rust_nse_plan_lifecycle(
        self,
        trade: Dict[str, Any],
        *,
        mark: float,
        path_high: Optional[float],
        path_low: Optional[float],
        now_ms: int,
    ) -> Optional[Dict[str, Any]]:
        """Rust Adaptive TP + Gate 2X + SL/TP for NSE. Returns result or None (fallback)."""
        if not plan_lifecycle_enabled():
            if plan_lifecycle_require_rust():
                plan_lifecycle_log_require_rust(
                    "nse",
                    "RP608_PLAN_LIFECYCLE disabled while REQUIRE_RUST=1",
                )
            return None
        evidence = os.environ.get("RP608_NSE_REC001H_DIR", DEFAULT_NSE_EVIDENCE)
        sess = plan_lifecycle_session(book="nse", evidence_dir=evidence)
        if sess is None:
            if plan_lifecycle_require_rust():
                plan_lifecycle_log_require_rust(
                    "nse",
                    "lifecycle bin missing or lacks --plan-lifecycle-loop",
                )
            return None
        rust_plan = plan_from_overlay(trade, book="nse")
        hi = float(path_high) if path_high is not None else mark
        lo = float(path_low) if path_low is not None else mark
        quote = {
            "best_bid_price": mark,
            "best_ask_price": mark,
            "low_price": lo,
            "high_price": hi,
            "now_ms": int(now_ms),
            "vol_surge": trade.get("gate2x_exit_vol_surge") or trade.get("vol_surge"),
        }
        result = sess.tick(rust_plan, quote)
        if not result or not isinstance(result, dict):
            if plan_lifecycle_require_rust():
                plan_lifecycle_log_require_rust(
                    "nse",
                    "plan-lifecycle-loop tick failed",
                )
            return None
        # merge returns a new dict — must write back onto the live trade.
        merged = merge_lifecycle_into_plan(trade, result, book="nse")
        trade.update(merged)
        return result

    def _manage_estimand_a_open_on_mark(
        self,
        *,
        symbol: str,
        mark: float,
        mark_src: str,
        now_ms: int,
        bar_idx: int,
        book: str = "primary",
    ) -> bool:
        """Tick-path SL/TP for one open. Caller holds `_estimand_a_lock`. True if closed."""
        active = self._estimand_active_trades(book)
        trade = active.get(symbol)
        if not trade or mark is None or mark <= 0:
            return False
        entry_p = float(trade["entry_price"])
        if mark > float(trade.get("mfe_price") or entry_p):
            trade["mfe_price"] = mark
        if mark < float(trade.get("mae_price") or entry_p):
            trade["mae_price"] = mark
        trade["last_tick_mark"] = mark
        trade["last_tick_mark_source"] = mark_src
        trade["last_tick_utc_ms"] = now_ms

        # R3: floor-lock twin is Rust-owned (gate2x_floor_lock_immediate on trade).
        rust = self._try_rust_nse_plan_lifecycle(
            trade, mark=float(mark), path_high=mark, path_low=mark, now_ms=int(now_ms)
        )
        if rust is not None and rust.get("closed"):
            _term, exit_reason = map_lifecycle_close(rust, book="nse")
            closed = active.pop(symbol)
            if rust.get("exit_reason_detail"):
                closed["exit_reason_detail"] = rust["exit_reason_detail"]
            exit_px = float(
                rust["exit_price"]
                if rust.get("exit_price") is not None
                else mark
            )
            path = (
                "TICK_PATH_GATE2X"
                if exit_reason == "GATE2X_MFE_GIVEBACK_PROTECT"
                else "TICK_PATH"
            )
            self._close_trade(
                closed,
                exit_price=exit_px,
                exit_bar=bar_idx,
                exit_reason=exit_reason,
                exit_path=path,
                exit_mark_source=mark_src,
                book=book,
            )
            return True
        # Track A: PLAN_LIFECYCLE=1 → Rust owns Adaptive/Gate2X/SL-TP (Python
        # bodies only when RP608_PLAN_LIFECYCLE=0 for tests).
        if rust is None and not plan_lifecycle_enabled():
            # Coralys/Gate2X first (Adaptive intelligence: doubt → Protect/Exit).
            try:
                giveback = self._gate2x_nse_update_mfe_giveback(
                    trade, mark=float(mark), now_ms=int(now_ms)
                )
            except Exception:
                giveback = None
            if giveback:
                closed = active.pop(symbol)
                if ADAPTIVE_TP_ENABLED:
                    closed["adaptive_tp_armed"] = True
                    closed["exit_reason_detail"] = f"ADAPTIVE_CORALYS_{giveback}"
                self._close_trade(
                    closed,
                    exit_price=float(mark),
                    exit_bar=bar_idx,
                    exit_reason="GATE2X_MFE_GIVEBACK_PROTECT",
                    exit_path="TICK_PATH_GATE2X",
                    exit_mark_source=mark_src,
                    book=book,
                )
                return True
            # Adaptive Coralys floor trail (not blind ATR) after Hold.
            self._adaptive_tp_nse_try_extend(
                trade, mark=float(mark), path_high=float(mark), now_ms=int(now_ms)
            )
            sl_price = float(trade.get("stop_loss_price") or (entry_p * ARM_A_STOP_MULT))
            tp_price = float(trade.get("target_price") or (entry_p * ARM_A_TARGET_MULT))
            hit_sl = mark <= sl_price
            hit_tp = mark >= tp_price
            if ADAPTIVE_TP_ENABLED:
                hit_tp = False
            if hit_sl:
                exit_reason = (
                    "TRAILING_STOP"
                    if trade.get("trailing_sl_armed") or trade.get("adaptive_tp_armed")
                    else "STOP_LOSS"
                )
                self._close_trade(
                    active.pop(symbol),
                    exit_price=sl_price,
                    exit_bar=bar_idx,
                    exit_reason=exit_reason,
                    exit_path="TICK_PATH",
                    exit_mark_source=mark_src,
                    book=book,
                )
                return True
            if hit_tp:
                self._close_trade(
                    active.pop(symbol),
                    exit_price=tp_price,
                    exit_bar=bar_idx,
                    exit_reason="TAKE_PROFIT",
                    exit_path="TICK_PATH",
                    exit_mark_source=mark_src,
                    book=book,
                )
                return True
        # Observational exit-pattern stamp on tick manage; optional paper reverse exit.
        try:
            exit_stamp = self._match_exit_pattern_for_trade(
                symbol=symbol,
                trade=trade,
                bar_idx=bar_idx,
            )
            if self._estimand_entry_like_baseline(book):
                exit_stamp = exit_stamp.get("baseline_parallel") or exit_stamp
            trade["trade_pattern_exit"] = exit_stamp
            if exit_stamp.get("apply_early_exit") and pattern_exit_gate_enabled():
                self._close_trade(
                    active.pop(symbol),
                    exit_price=float(mark),
                    exit_bar=bar_idx,
                    exit_reason="PATTERN_REVERSE",
                    exit_path="TICK_PATH_PATTERN",
                    exit_mark_source=mark_src,
                    book=book,
                )
                return True
        except Exception:
            pass
        return False

    def _on_estimand_a_ws_tick(self, tick: Dict[str, Any]) -> None:
        """
        True tick-path hub: Estimand A (if TICK_PATH) + OF/OFX paper per WS packet.
        Signal (traj15) stays completed-bar causal; fill time/price are per-tick.
        Estimand A / OFX entry require trade-like volume/LT (reject quote-only false prints).
        Baseline OF acts on every valid L1 packet (resting imbalance construct).
        """
        try:
            tok = int(tick.get("instrument_token") or 0)
        except (TypeError, ValueError):
            return
        symbol = self._token_to_symbol.get(tok)
        if not symbol:
            return
        try:
            bid = float(tick.get("best_bid") or 0.0)
            ask = float(tick.get("best_ask") or 0.0)
            last = float(tick.get("last_price") or 0.0)
            ltq = float(tick.get("last_traded_quantity") or 0.0)
            volume = float(tick.get("volume") or 0.0)
            ltt_sec = int(tick.get("last_trade_time_sec") or 0)
        except (TypeError, ValueError):
            return
        if bid <= 0.0 or ask <= 0.0 or ask <= bid:
            return

        prev = self._estimand_a_prev_ws_tick.get(symbol)
        update_class, trade_like = classify_trade_like(
            prev=prev,
            ltp=last,
            ltq=ltq,
            ltt_sec=ltt_sec,
            volume=volume,
        )
        # Always advance prev so the next packet can detect volume/LT deltas.
        self._estimand_a_prev_ws_tick[symbol] = {
            "last_price": last,
            "last_traded_quantity": ltq,
            "last_trade_time_sec": ltt_sec,
            "volume": volume,
            "best_bid": bid,
            "best_ask": ask,
        }

        mark = last if last > 0.0 else (bid + ask) / 2.0
        mark_src = "WS_LAST" if last > 0.0 else "WS_MID"
        try:
            now_ms = int(tick.get("receipt_utc_ms") or int(time.time() * 1000))
        except (TypeError, ValueError):
            now_ms = int(time.time() * 1000)
        quote = TopOfBookQuote(
            timestamp_utc_ms=now_ms,
            bid_price=bid,
            ask_price=ask,
            bid_qty=float(tick.get("bid_qty") or 0.0),
            ask_qty=float(tick.get("ask_qty") or 0.0),
        )
        signal_bar = int(
            self._estimand_a_armed_signal_bar
            or getattr(self.state, "last_durable_bar_idx", 0)
            or 0
        )

        if ENTRY_PATH == "TICK_PATH" and signal_bar >= 1:
            opened = False
            closed = False
            with self._estimand_a_lock:
                try:
                    # Entry decision: volume/LT trade-like only (false-print filter).
                    allow_entry = (not TICK_TRADE_LIKE_ENTRY) or bool(trade_like)
                    for book in ESTIMAND_BOOK_KEYS:
                        active = self._estimand_active_trades(book)
                        if allow_entry and symbol not in active:
                            did_open = bool(
                                self._try_estimand_a_tick_entry(
                                    symbol=symbol,
                                    signal_bar_idx=signal_bar,
                                    quote=quote,
                                    mark=float(mark),
                                    mark_src=mark_src,
                                    now_ms=now_ms,
                                    book=book,
                                )
                            )
                            if did_open:
                                opened = True
                                tr = active.get(symbol)
                                if tr is not None:
                                    tr["entry_update_class"] = update_class
                                    tr["entry_ltq"] = ltq
                                    tr["entry_session_volume"] = volume
                        # Manage opens on any fresh L1 mark (stop/target fidelity).
                        if symbol in active:
                            if self._manage_estimand_a_open_on_mark(
                                symbol=symbol,
                                mark=float(mark),
                                mark_src=mark_src,
                                now_ms=now_ms,
                                bar_idx=signal_bar,
                                book=book,
                            ):
                                closed = True
                except Exception as exc:
                    print(f"[estimand_a_tick] ws tick skipped ({symbol}): {exc}", flush=True)
                    return
                if opened or closed:
                    self.state.active_positions_count = (
                        self._estimand_active_count()
                    )
            if opened:
                self._estimand_a_tick_opens_since_log += 1
                if self._estimand_a_tick_opens_since_log >= 5:
                    n = self._estimand_a_tick_opens_since_log
                    self._estimand_a_tick_opens_since_log = 0
                    print(
                        f"[estimand_a_tick] ws_fills+={n} signal_bar={signal_bar} "
                        f"active_p={len(self.active_trades)} "
                        f"active_bl={len(self.active_trades_baseline)} "
                        f"active_r50={len(self.active_trades_g2x_r50)} "
                        f"active_fl={len(self.active_trades_g2x_floor_lock)} "
                        f"trade_like_gate={int(TICK_TRADE_LIKE_ENTRY)}",
                        flush=True,
                    )
            if opened or closed:
                self._persist_estimand_a_pending(force=closed)

        # OF (L1 resting) + OFX (volume paper): per-tick action; books never pooled.
        try:
            self._of_ofx_action_on_ws_tick(
                symbol=symbol,
                quote=quote,
                now_ms=now_ms,
                bar_idx=int(signal_bar or getattr(self.state, "last_durable_bar_idx", 0) or 0),
                trade_like=bool(trade_like),
            )
        except Exception as of_tick_exc:
            print(f"[nse_of_ofx_tick] skipped ({symbol}): {of_tick_exc}", flush=True)

    def _of_ofx_action_on_ws_tick(
        self,
        *,
        symbol: str,
        quote: TopOfBookQuote,
        now_ms: int,
        bar_idx: int,
        trade_like: bool,
    ) -> None:
        """
        Per-tick OF / OFX paper actions (authority 0%; books separate).

        - Baseline OF: manage + try_open on every valid L1 packet (resting imbalance).
        - OFX volume: manage + try_open on trade-like ticks only (false-print filter + surge).
        Armed decision comes from last bar eval (`_paper_armed_eval_by_symbol`).
        """
        sq_bar = int(
            getattr(self._active_session_window, "square_off_bar", MANDATORY_SQUARE_OFF_BAR)
            or MANDATORY_SQUARE_OFF_BAR
        )
        force = "SESSION_SQUARE_OFF" if bar_idx >= sq_bar else None
        armed = self._paper_armed_eval_by_symbol.get(symbol)

        # --- Baseline OF (L1 resting imbalance; not aggressor volume) ---
        try:
            self.of_paper.evaluate_open_tick(
                symbol=symbol,
                quote=quote,
                now_ms=now_ms,
                bar_idx=bar_idx,
                force_reason=force,
            )
            if armed is not None and bar_idx < sq_bar:
                # Re-check live L1 on this tick (do not trust stale bar OF stamp alone).
                of_eval = dict(armed)
                of_eval["order_flow_supported"] = False
                of_eval["order_flow_long_supported"] = False
                self.of_paper.try_open_from_eval(
                    symbol=symbol,
                    eval_record=of_eval,
                    quote=quote,
                    now_ms=now_ms,
                )
        except Exception as ofp_exc:
            print(f"[nse_of_paper] ws tick skipped ({symbol}): {ofp_exc}", flush=True)

        # --- OFX volume paper (executed/inferred aggressor; baseline OF inert) ---
        if not trade_like and TICK_TRADE_LIKE_ENTRY:
            return
        try:
            vol_snap = build_ofx_nse_volume_flow_snapshot(
                symbol=symbol,
                buffer=self.ofx_nse_volume_buf,
                now_ms=now_ms,
            )
            self.ofx_nse_volume_paper.evaluate_open_tick(
                symbol=symbol,
                volume_snapshot=vol_snap,
                quote=quote,
                now_ms=now_ms,
                force_reason=force,
            )
            if armed is not None and bar_idx < sq_bar:
                self.ofx_nse_volume_paper.try_open_from_eval(
                    symbol=symbol,
                    eval_record=armed,
                    volume_snapshot=vol_snap,
                    quote=quote,
                    now_ms=now_ms,
                )
        except Exception as ofx_exc:
            print(f"[ofx_nse_volume_paper] ws tick skipped ({symbol}): {ofx_exc}", flush=True)

    def _estimand_a_tick_path_poll(self, *, bar_idx: int = 0, feed_results: Optional[Dict[str, Any]] = None) -> int:
        """
        Fallback midbar Estimand A: catch-up fills + L1 SL/TP if a WS tick was missed.
        Primary fills are `_on_estimand_a_ws_tick`. Gate 5 levels frozen.
        """
        now_ms = int(time.time() * 1000)
        if feed_results is None:
            try:
                feed_results = self.feed_adapter.fetch_all_symbols()
            except Exception as exc:
                print(f"[estimand_a_tick] midbar fetch skipped: {exc}")
                return 0

        signal_bar = int(
            self._estimand_a_armed_signal_bar
            or bar_idx
            or getattr(self.state, "last_durable_bar_idx", 0)
            or 0
        )
        opened = 0
        closed = 0
        mutated = False

        with self._estimand_a_lock:
            # Manage-only on REST midbar. Entries are WS-tick only (no candle-batch fills).
            for book in ESTIMAND_BOOK_KEYS:
                active = self._estimand_active_trades(book)
                for sym in list(active.keys()):
                    res = feed_results.get(sym, {}) if isinstance(feed_results, dict) else {}
                    mark, mark_src = self._estimand_a_mark_from_feed(res)
                    if mark is None or mark <= 0:
                        continue
                    mutated = True
                    if self._manage_estimand_a_open_on_mark(
                        symbol=sym,
                        mark=float(mark),
                        mark_src=str(mark_src),
                        now_ms=now_ms,
                        bar_idx=signal_bar,
                        book=book,
                    ):
                        closed += 1

            if mutated or closed:
                self.state.active_positions_count = self._estimand_active_count()

        if mutated or closed:
            try:
                self.save_pending_evaluations()
            except Exception as exc:
                print(f"[estimand_a_tick] persist skipped: {exc}")
        return closed

    def _of_paper_midbar_poll(self, *, bar_idx: int = 0) -> None:
        """
        While awaiting the next 15m close, still tick open OF papers on live L1
        so min-hold / opposite-persist anti-churn can fire (crypto-parity).
        Also ticks open OFX NSE volume papers on derived aggressor volume marks.
        Also Estimand A tick entry + L1 SL/TP (ENTRY_PATH/TICK_PATH).
        """
        of_opens = self.of_paper.open_snapshot()
        ofx_opens = self.ofx_nse_volume_paper.open_snapshot()
        need_a = True  # always poll Estimand A tick entry/manage midbar
        if not of_opens and not ofx_opens and not need_a:
            return
        now_ms = int(time.time() * 1000)
        try:
            feed_results = self.feed_adapter.fetch_all_symbols()
        except Exception as exc:
            print(f"[nse_of_paper] midbar fetch skipped: {exc}")
            return
        # Estimand A tick-path entry + management (shared feed fetch).
        try:
            self._estimand_a_tick_path_poll(bar_idx=bar_idx, feed_results=feed_results)
        except Exception as exc:
            print(f"[estimand_a_tick] midbar poll skipped: {exc}")
        of_syms = {str(t.get("symbol")) for t in of_opens if t.get("symbol")}
        ofx_syms = {str(t.get("symbol")) for t in ofx_opens if t.get("symbol")}
        syms = sorted(of_syms | ofx_syms)
        for sym in syms:
            res = feed_results.get(sym, {}) if isinstance(feed_results, dict) else {}
            quote = res.get("quote") if isinstance(res, dict) else None
            if sym in of_syms:
                try:
                    self.of_paper.evaluate_open_tick(
                        symbol=sym,
                        quote=quote,
                        now_ms=now_ms,
                        bar_idx=bar_idx,
                    )
                except Exception as exc:
                    print(f"[nse_of_paper] midbar tick skipped ({sym}): {exc}")
            if sym in ofx_syms:
                try:
                    vol_snap = build_ofx_nse_volume_flow_snapshot(
                        symbol=sym,
                        buffer=self.ofx_nse_volume_buf,
                        now_ms=now_ms,
                    )
                    self.ofx_nse_volume_paper.evaluate_open_tick(
                        symbol=sym,
                        volume_snapshot=vol_snap,
                        quote=quote,
                        now_ms=now_ms,
                    )
                except Exception as exc:
                    print(f"[ofx_nse_volume_paper] midbar tick skipped ({sym}): {exc}")

    def _close_trade(
        self,
        trade: Dict[str, Any],
        exit_price: float,
        exit_bar: int,
        exit_reason: str,
        *,
        exit_path: str = "BAR_CLOSE",
        exit_mark_source: str = "BAR_CLOSE",
        book: str = "primary",
    ):
        """Calculates exact statutory friction and closes trade for Estimand A twin books."""
        book_key = self._estimand_normalize_book(book)
        if book_key == "primary":
            # Prefer durable stamp on the trade when caller passed a loose key.
            book_key = self._estimand_normalize_book(str(trade.get("book") or book))
        journaled = self._estimand_journaled_ids(book_key)
        journal_path = self._estimand_journal_path(book_key)
        book_id = self._estimand_book_id(book_key)

        entry_p = trade["entry_price"]
        qty = trade["quantity"]
        gross_pnl = (exit_price - entry_p) * qty
        gross_return_bps = ((exit_price - entry_p) / entry_p) * 10000.0

        friction = calc_indian_equity_friction(
            buy_price=entry_p,
            sell_price=exit_price,
            quantity=qty,
            half_spread_bps=trade["half_spread_bps"]
        )

        net_pnl = gross_pnl - friction["total_friction"]
        net_return_bps = gross_return_bps - friction["effective_friction_bps"]

        mfe_bps = ((trade["mfe_price"] - entry_p) / entry_p) * 10000.0
        mae_bps = ((entry_p - trade["mae_price"]) / entry_p) * 10000.0
        edge_ratio = round(mfe_bps / mae_bps, 2) if mae_bps > 0 else None

        tid = str(trade.get("trade_id") or "")
        if tid and tid in journaled:
            # Idempotent close: already journaled (midbar re-eval churn guard).
            return

        record = {
            **trade,
            "book": book_id,
            "exit_bar_idx": exit_bar,
            "exit_timestamp_utc_ms": int(time.time() * 1000),
            "exit_price": exit_price,
            "exit_reason": exit_reason,
            "exit_path": exit_path,
            "exit_mark_source": exit_mark_source,
            "gross_pnl_inr": round(gross_pnl, 2),
            "gross_return_bps": round(gross_return_bps, 2),
            "win_label": 1 if gross_pnl > 0.0 else 0,
            "friction": friction,
            "net_pnl_inr": round(net_pnl, 2),
            "net_return_bps": round(net_return_bps, 2),
            "mfe_bps": round(mfe_bps, 2),
            "mae_bps": round(mae_bps, 2),
            "edge_ratio": edge_ratio
        }

        self.append_jsonl(journal_path, record)
        if tid:
            journaled.add(tid)
        # Pattern session stop families feed PRIMARY Gate2E only (not baseline control).
        if not is_bl:
            try:
                self._note_pattern_session_stop(trade=record, exit_reason=str(exit_reason or ""))
            except Exception:
                pass
        self.state.completed_trades_count += 1
        if not is_bl:
            try:
                prospective_v1_append_nse_trade_closed(trade=record)
            except Exception as link_exc:
                print(f"[prospective_v1] NSE trade_closed link skipped: {link_exc}")

    def run_single_iteration(self) -> Dict[str, Any]:
        """
        Executes a single market data collection and bar evaluation cycle.
        """
        now_ms = int(time.time() * 1000)
        now_utc = datetime.now(timezone.utc)
        is_active, bar_idx, phase = self.is_ist_session_active(now_utc)
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = now_utc + ist_offset

        # In single_bar_test mode, force active session if closed (outside market hours)
        if self.single_bar_test and not is_active:
            is_active = True
            bar_idx = 1
            phase = "TEST_FORCED_ACTIVE"

        if not is_active:
            # Emit heartbeat and return idle.
            # Closed-market heartbeat must NOT accidentally initialize or erase active-session state!
            self.state.last_heartbeat_utc_ms = now_ms
            self.state.last_heartbeat_iso = now_utc.isoformat()
            next_sess_meta = None
            if phase.startswith("POST_CLOSE"):
                try:
                    # Force-flat residual NSE OF paper + OFX volume paper after cash/muhurat
                    need_quotes = bool(
                        self.of_paper.open_snapshot()
                        or self.ofx_nse_volume_paper.open_snapshot()
                    )
                    feed_results: Dict[str, Any] = {}
                    if need_quotes:
                        try:
                            feed_results = self.feed_adapter.fetch_all_symbols()
                        except Exception:
                            feed_results = {}
                    if self.of_paper.open_snapshot():
                        quotes = {
                            sym: (feed_results.get(sym) or {}).get("quote")
                            for sym in [t.get("symbol") for t in self.of_paper.open_snapshot()]
                            if sym
                        }
                        self.of_paper.square_off_all(
                            quotes_by_symbol=quotes,
                            now_ms=now_ms,
                            reason="SESSION_SQUARE_OFF",
                        )
                    if self.ofx_nse_volume_paper.open_snapshot():
                        quotes_x = {
                            sym: (feed_results.get(sym) or {}).get("quote")
                            for sym in [
                                t.get("symbol") for t in self.ofx_nse_volume_paper.open_snapshot()
                            ]
                            if sym
                        }
                        self.ofx_nse_volume_paper.square_off_all(
                            quotes_by_symbol=quotes_x,
                            now_ms=now_ms,
                            reason="SESSION_SQUARE_OFF",
                        )
                    next_sess_meta = self.emit_next_session_recommendations_if_needed(now_ist=now_ist)
                except Exception as nsr_exc:
                    print(f"[next_session_recs] emit skipped: {nsr_exc}")
            self.atomic_commit_checkpoint(self.state)
            out = {
                "status": phase,
                "bar_idx": 0,
                "processed": 0,
                "nse_of_paper": self.of_paper.summary(),
                "ofx_nse_volume_paper": self.ofx_nse_volume_paper.summary(),
            }
            if next_sess_meta:
                out["next_session_recommendations"] = next_sess_meta
            return out

        # Determine authoritative trading session date in IST
        today_session_str = str(now_ist.date())

        # Session transition occurs ONLY when active trading session is entered
        if self.current_session_date is None:
            self.current_session_date = today_session_str
            self.state.current_session_date = today_session_str
            self.state.last_durable_bar_idx = 0
            self.state.last_accounted_bar_idx = 0
            self._estimand_a_armed_signal_bar = 0
            self._paper_armed_eval_by_symbol.clear()
            self._estimand_a_prev_ws_tick.clear()
            # Fresh session: always load latest Kite token (daily rotate / overnight re-auth).
            reload_fn = getattr(self.feed_adapter, "reload_credentials_if_stale", None)
            if callable(reload_fn):
                reload_fn(force=True)
            self.atomic_commit_checkpoint(self.state)
        elif str(self.current_session_date) != today_session_str:
            self.state.session_idx += 1
            self.state.last_durable_bar_idx = 0
            self.state.last_accounted_bar_idx = 0
            self._estimand_a_armed_signal_bar = 0
            self._paper_armed_eval_by_symbol.clear()
            self._estimand_a_prev_ws_tick.clear()
            self.price_history = {s: [] for s in self.symbols}
            self.active_trades.clear()
            self.active_trades_baseline.clear()
            self.active_trades_g2x_r50.clear()
            self.active_trades_g2x_floor_lock.clear()
            self.pending_directional_evals.clear()
            self.last_evaluations.clear()
            self._symbol_last_bootstrap_bar.clear()
            self._symbol_bootstrap_fail_mono.clear()
            # Square-off prior-session OF paper leftovers before new session id advances
            try:
                if self.of_paper.open_snapshot():
                    self.of_paper.square_off_all(
                        quotes_by_symbol={},
                        now_ms=now_ms,
                        reason="SESSION_ROLLOVER_SQUARE_OFF",
                    )
                if self.ofx_nse_volume_paper.open_snapshot():
                    self.ofx_nse_volume_paper.square_off_all(
                        quotes_by_symbol={},
                        now_ms=now_ms,
                        reason="SESSION_ROLLOVER_SQUARE_OFF",
                    )
            except Exception as ofp_exc:
                print(f"[nse_of_paper] session rollover square-off skipped: {ofp_exc}")
            self.current_session_date = today_session_str
            self.state.current_session_date = today_session_str
            # Next session must not inherit yesterday's expired access_token in memory.
            reload_fn = getattr(self.feed_adapter, "reload_credentials_if_stale", None)
            if callable(reload_fn):
                reload_fn(force=True)
            self.save_bar_history()
            self.save_pending_evaluations()
            self.atomic_commit_checkpoint(self.state)

        # Once per session: classify prior-watch freshness (V3 ladder, 6-day holiday buffer).
        if is_active and today_session_str and self._coralys_prior_logged_for != today_session_str:
            try:
                self.coralys_input_store.invalidate_watch_cache()
                self.coralys_input_store.load_watchlist_for_session(today_session_str)
                meta = dict(getattr(self.coralys_input_store, "_watch_meta", {}) or {})
                print(
                    f"[coralys_input] prior_status={meta.get('status')} "
                    f"next_session_date={meta.get('next_session_date')} "
                    f"n={meta.get('n_recommendations')} path={meta.get('path')}"
                )
            except Exception as prior_exc:
                print(f"[coralys_input] prior watch classify skipped: {prior_exc}")
            self._coralys_prior_logged_for = today_session_str

        # Determine completed bars vs forming bars in current session (regular or muhurat)
        window = self._active_session_window
        if window is None:
            window, _ = resolve_session_window(
                now_ist,
                exchange_holidays=getattr(self, "exchange_holidays", NSE_2026_HOLIDAYS),
            )
        if window is not None:
            completed_bars = completed_bars_since_open(now_ist, window)
            warmup_n = int(window.warmup_completed_bars)
        else:
            open_time = now_ist.replace(hour=9, minute=15, second=0, microsecond=0)
            elapsed_sec = (now_ist - open_time).total_seconds()
            completed_bars = min(int(elapsed_sec // (15 * 60)), TOTAL_BARS_PER_SESSION)
            warmup_n = 5

        # Durable eval runs only when a new 15m bar has closed. Provisional live
        # evals run while bar 1 (or the next bar) is still forming so G1/recs are
        # available from session open — not only after the first close (~15m wait).
        provisional_eval = False
        if not self.single_bar_test and is_active:
            if completed_bars == 0:
                provisional_eval = True
                eval_bar = max(1, int(bar_idx) or 1)
                try:
                    self._of_paper_midbar_poll(bar_idx=eval_bar)
                except Exception as ofp_exc:
                    print(f"[nse_of_paper] forming-bar midbar poll skipped: {ofp_exc}")
                try:
                    self._next_session_levels_midbar_poll()
                except Exception as nsr_lvl_exc:
                    print(f"[next_session_levels] forming-bar midbar poll skipped: {nsr_lvl_exc}")
                try:
                    self._sample_of_book_depth_history()
                except Exception as book_exc:
                    print(f"[of_book_intel] forming-bar depth sample skipped: {book_exc}")
            elif completed_bars <= self.state.last_durable_bar_idx:
                provisional_eval = True
                eval_bar = max(int(completed_bars) + 1, int(bar_idx) or 1)
                try:
                    self._of_paper_midbar_poll(bar_idx=int(self.state.last_durable_bar_idx or completed_bars))
                except Exception as ofp_exc:
                    print(f"[nse_of_paper] midbar poll skipped: {ofp_exc}")
                try:
                    self._next_session_levels_midbar_poll()
                except Exception as nsr_lvl_exc:
                    print(f"[next_session_levels] midbar poll skipped: {nsr_lvl_exc}")
                try:
                    self._sample_of_book_depth_history()
                except Exception as book_exc:
                    print(f"[of_book_intel] midbar depth sample skipped: {book_exc}")
            else:
                # A new 15m candle has finalized! Durable evaluate at close.
                eval_bar = completed_bars
        else:
            eval_bar = bar_idx

        # Auto-recovery after warmup bars (1 on muhurat; 5 on regular).
        # On provisional forming-bar-1, still try bootstrap so G3 can use history.
        # Always purge quote-snapshot spam first so traj15 lookback is not stuffed
        # with identical day-OHLC clones (dashboard G4 lock at +0.0000).
        if is_active:
            self._purge_quote_snapshot_spam()
        if is_active and (eval_bar >= warmup_n or provisional_eval):
            self.bootstrap_same_day_history(now_utc)

        # Fetch market data for all symbols
        feed_results = self.feed_adapter.fetch_all_symbols()

        successful = 0
        failed = 0
        iteration_evals: List[Dict[str, Any]] = []

        for sym in self.symbols:
            res = feed_results.get(sym, {})
            bars = res.get("bars")
            quote = res.get("quote")
            err = res.get("error")

            if bars and len(bars) > 0:
                latest_bar = bars[-1]
            else:
                latest_bar = None

            if latest_bar is not None:
                # Provisional midbar: observe only — no Estimand A open/close mutations.
                self.evaluate_symbol_bar(
                    sym,
                    eval_bar,
                    latest_bar,
                    quote,
                    mutate_trades=not bool(provisional_eval),
                )
                eval_rec = self.last_evaluations.get(sym)
                if eval_rec:
                    iteration_evals.append(eval_rec)
                successful += 1
            else:
                self.append_opportunity_drop(
                    drop_type="NSE_FEED_DATA_MISSING",
                    reason=err or "NO_BARS_RETURNED",
                    symbol=sym,
                    bar_idx=bar_idx
                )
                self.state.unexecutable_drops_count += 1
                failed += 1

                # Data-quality failure evaluation record (ensuring 100% census coverage)
                fail_of = inspect_order_flow_book(
                    l1_from_nse_quote(None),
                    primary_side="LONG",
                )
                fail_record = {
                    "symbol": sym,
                    "tradingsymbol": sym.replace(".NS", ""),
                    "exchange": "NSE",
                    "session_idx": self.state.session_idx,
                    "bar_idx": bar_idx,
                    "bar_timestamp_utc_ms": now_ms,
                    "bar_timestamp_iso": now_utc.isoformat(),
                    "quote_timestamp_utc_ms": None,
                    "quote_age_ms": None,
                    "reference_price": None,
                    "decision": "NO_TRADE",
                    "decision_code": "DATA_QUALITY_FAILURE",
                    "explanation": f"Data quality failure: {err or 'NO_BARS_RETURNED'}",
                    "score": 0.0,
                    "provisional_rank": None,
                    "history_length": len(self.price_history.get(sym, [])),
                    "half_spread_bps": None,
                    "entry_conditions": (
                        f"traj15 >= 0.05 and history >= 5 and 1 <= bar_idx <= {MAX_NEW_ENTRY_BAR} "
                        f"and not active_trade (60m completes by 15:00 close-window)"
                    ),
                    "invalidation": "p <= entry_price * 0.99 (1.0% stop) or 4-bar hold without gain or max 12 bars",
                    "risk_reward": None,
                    "data_quality": {
                        "status": "DATA_FAILURE",
                        "feed_source": "kite_connect",
                        "error": str(err or "NO_BARS_RETURNED")
                    },
                    "order_flow_supported": fail_of["order_flow_supported"],
                    "order_flow_label": fail_of["order_flow_label"],
                    "order_flow_imbalance": fail_of["order_flow_imbalance"],
                    "order_flow_source": fail_of["order_flow_source"],
                    "order_flow_side": fail_of["order_flow_side"],
                    "order_flow_long_supported": fail_of["order_flow_long_supported"],
                    "order_flow_short_supported": fail_of["order_flow_short_supported"],
                    "order_flow_inspection": True,
                }
                iteration_evals.append(fail_record)
                self.last_evaluations[sym] = fail_record

        # ─── Provisional Ranking of Eligible Candidates ─────────────
        # Filter qualifying long recommendations and sort by trajectory score descending
        qualifying = [r for r in iteration_evals if r.get("decision") == "RECOMMEND_LONG"]
        qualifying.sort(key=lambda r: r.get("score", 0.0), reverse=True)
        for rank_idx, q_rec in enumerate(qualifying, start=1):
            q_rec["provisional_rank"] = rank_idx

        # ─── Rejection Reason Breakdown ─────────────────────────────
        rejection_breakdown: Dict[str, int] = {}
        for r in iteration_evals:
            code = r.get("decision_code", "UNKNOWN")
            rejection_breakdown[code] = rejection_breakdown.get(code, 0) + 1

        # Tag provisional vs durable for dashboard freshness without waiting on bar close
        for r in iteration_evals:
            r["provisional_eval"] = bool(provisional_eval)
            r["eval_mode"] = "PROVISIONAL_LIVE" if provisional_eval else "DURABLE_BAR_CLOSE"

        # After durable bar close: arm signal_bar only. Fills happen on each WS L1 tick.
        if (not provisional_eval) and ENTRY_PATH == "TICK_PATH" and is_active:
            self._estimand_a_armed_signal_bar = int(eval_bar)
            self._estimand_a_tick_opens_since_log = 0
            print(
                f"[estimand_a_tick] armed signal_bar={int(eval_bar)} "
                "(fills on WS ticks, not candle batch)",
                flush=True,
            )

        # Durable ledger: only commit closed-bar evals (avoid jsonl flood on mid-bar polls)
        if not provisional_eval:
            for r in iteration_evals:
                self.append_jsonl(self.evaluations_path, r)

        # ─── Commit Atomic Snapshot for Immediate Dashboard Access ──
        latest_eval_snapshot = {
            "timestamp_utc_ms": now_ms,
            "timestamp_iso": now_utc.isoformat(),
            "session_idx": self.state.session_idx,
            "bar_idx": eval_bar,
            "provisional": bool(provisional_eval),
            "eval_mode": "PROVISIONAL_LIVE" if provisional_eval else "DURABLE_BAR_CLOSE",
            "session_kind": getattr(window, "kind", None) if window is not None else None,
            "total_evaluated": len(iteration_evals),
            "n_qualifying": len(qualifying),
            "rejection_breakdown": rejection_breakdown,
            "qualifying_candidates": qualifying,
            "evaluations": {r["symbol"]: r for r in iteration_evals}
        }
        self.atomic_commit_json(self.latest_evaluations_path, latest_eval_snapshot)
        self.save_bar_history()
        self.save_pending_evaluations()

        if not provisional_eval:
            self.state.last_durable_bar_idx = eval_bar
            self.state.last_accounted_bar_idx = max(self.state.last_accounted_bar_idx, eval_bar)
        self.state.active_positions_count = self._estimand_active_count()
        self.state.consecutive_outages = 0
        self.state.last_heartbeat_utc_ms = now_ms
        self.state.last_heartbeat_iso = now_utc.isoformat()
        self.atomic_commit_checkpoint(self.state)

        # Log telemetry heartbeat
        of_paper_summary = self.of_paper.summary()
        telem = {
            "timestamp_utc_ms": now_ms,
            "timestamp_iso": now_utc.isoformat(),
            "session_idx": self.state.session_idx,
            "bar_idx": eval_bar,
            "phase": phase,
            "provisional": bool(provisional_eval),
            "eval_mode": "PROVISIONAL_LIVE" if provisional_eval else "DURABLE_BAR_CLOSE",
            "session_kind": getattr(window, "kind", None) if window is not None else None,
            "successful_symbols": successful,
            "failed_symbols": failed,
            "evaluations_count": len(iteration_evals),
            "qualifying_recommendations": len(qualifying),
            "active_positions": len(self.active_trades),
            "active_positions_baseline": len(self.active_trades_baseline),
            "active_positions_g2x_r50": len(self.active_trades_g2x_r50),
            "active_positions_g2x_floor_lock": len(self.active_trades_g2x_floor_lock),
            "nse_of_paper_n_open": of_paper_summary.get("n_open"),
            "nse_of_paper_n_trades": (of_paper_summary.get("of_outcomes_summary") or {}).get("n_trades"),
            "completed_trades": self.state.completed_trades_count,
            "directional_cards": self.state.directional_cards_count,
            "unexecutable_drops": self.state.unexecutable_drops_count,
            "pending_directional_evals_count": sum(len(v) for v in self.pending_directional_evals.values())
        }
        self.append_jsonl(self.telemetry_path, telem)
        return telem

    def _next_session_levels_session_actionable(self, payload: Dict[str, Any]) -> bool:
        next_date = str(payload.get("next_session_date") or "")
        sess = str(
            self.current_session_date
            or getattr(self.state, "current_session_date", "")
            or ""
        )
        if hasattr(sess, "isoformat"):
            sess = sess.isoformat()
        bar_idx = int(getattr(self.state, "last_durable_bar_idx", 0) or 0)
        return bool(sess and next_date and sess == next_date and bar_idx >= 5)

    def _update_next_session_watch_levels(
        self,
        *,
        symbol: str,
        last_price: float,
        completed_bar: bool,
        bar_timestamp_utc_ms: Optional[int] = None,
        feed_stale: bool = False,
        reconcile_pending: bool = False,
    ) -> None:
        """
        Push-path maintenance of indicative next-session levels (0% authority).

        - completed_bar=True → may refresh RP608_DYNAMIC_OHLCV_STRUCTURE_V1 for that symbol
        - always updates level_status from the price event (respecting invalidate latch)
        - mutates only the matching watchlist row (no cross-symbol close-price rewrite)
        """
        from scripts.rp608_nse_next_session_recs import (
            apply_price_event_to_watch_levels,
            compute_indicative_watch_levels,
            write_next_session_recommendations,
        )

        path = getattr(self, "next_session_recs_path", None)
        if path is None or not path.exists():
            return
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            return
        recs = payload.get("recommendations")
        if not isinstance(recs, list) or not recs:
            return

        target_i = next(
            (i for i, r in enumerate(recs) if isinstance(r, dict) and str(r.get("symbol")) == symbol),
            None,
        )
        if target_i is None:
            return

        session_actionable = self._next_session_levels_session_actionable(payload)
        row = dict(recs[target_i])
        side = str(row.get("side") or "LONG")
        levels = row.get("levels") if isinstance(row.get("levels"), dict) else None
        prior_gen = int((levels or {}).get("structure_generation") or 1)
        prior_bar_ts = (levels or {}).get("levels_source_bar_timestamp_utc_ms")

        if completed_bar and session_actionable:
            bars = list(self.price_history.get(symbol) or [])
            if bars:
                calc_at = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
                fresh = compute_indicative_watch_levels(
                    bars,
                    side=side,
                    reference_price=None,
                    calculated_at_utc=calc_at,
                )
                new_bar_ts = fresh.get("levels_source_bar_timestamp_utc_ms")
                if fresh.get("ok") and new_bar_ts is not None and new_bar_ts != prior_bar_ts:
                    fresh = dict(fresh)
                    fresh["structure_generation"] = prior_gen + 1
                    fresh["invalidation_latched"] = False
                    levels = fresh
                elif levels is None and fresh.get("ok"):
                    levels = fresh

        if not isinstance(levels, dict):
            return

        levels = apply_price_event_to_watch_levels(
            levels,
            side=side,
            last_price=float(last_price),
            feed_stale=bool(feed_stale),
            reconcile_pending=bool(reconcile_pending),
            session_actionable=session_actionable,
        )
        row["levels"] = levels
        row["level_status"] = levels.get("level_status")
        row["invalidation_latched"] = levels.get("invalidation_latched")
        row["entry_low"] = levels.get("entry_low")
        row["entry_high"] = levels.get("entry_high")
        row["stop_price"] = levels.get("stop_price")
        row["target_price"] = levels.get("target_price")
        row["risk_reward_display"] = levels.get("risk_reward_display")
        recs = list(recs)
        recs[target_i] = row
        payload["recommendations"] = recs
        payload["levels_last_push_update_utc"] = datetime.now(timezone.utc).isoformat().replace(
            "+00:00", "Z"
        )
        if bar_timestamp_utc_ms is not None:
            payload["levels_last_bar_timestamp_utc_ms"] = int(bar_timestamp_utc_ms)
        write_next_session_recommendations(path, payload)

    def _next_session_levels_midbar_poll(self) -> None:
        """
        L1 price-event validity for stamped watchlist symbols between bar closes.

        Structure is never recalculated here. This is the daemon interim event proxy
        (same cadence as OF-paper midbar), not UI poll evaluation. Authority 0%.
        """
        from scripts.rp608_nse_next_session_recs import (
            apply_price_event_to_watch_levels,
            write_next_session_recommendations,
        )

        path = getattr(self, "next_session_recs_path", None)
        if path is None or not path.exists():
            return
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            return
        recs = payload.get("recommendations")
        if not isinstance(recs, list) or not recs:
            return
        if not self._next_session_levels_session_actionable(payload):
            return
        try:
            feed_results = self.feed_adapter.fetch_all_symbols()
        except Exception as exc:
            print(f"[next_session_levels] midbar fetch skipped: {exc}")
            return

        session_actionable = True
        changed = False
        out: List[Dict[str, Any]] = []
        for raw in recs:
            if not isinstance(raw, dict):
                out.append(raw)
                continue
            row = dict(raw)
            sym = str(row.get("symbol") or "")
            levels = row.get("levels") if isinstance(row.get("levels"), dict) else None
            if not sym or not levels:
                out.append(row)
                continue
            res = feed_results.get(sym, {}) if isinstance(feed_results, dict) else {}
            quote = res.get("quote") if isinstance(res, dict) else None
            if quote is None:
                out.append(row)
                continue
            try:
                last_px = float(
                    getattr(quote, "last_price", None)
                    or getattr(quote, "ltp", None)
                    or 0.0
                )
            except (TypeError, ValueError):
                last_px = 0.0
            if last_px <= 0:
                try:
                    bid = float(getattr(quote, "bid_price", 0.0) or 0.0)
                    ask = float(getattr(quote, "ask_price", 0.0) or 0.0)
                    if bid > 0 and ask > 0:
                        last_px = 0.5 * (bid + ask)
                except (TypeError, ValueError):
                    last_px = 0.0
            if last_px <= 0:
                out.append(row)
                continue
            age_ms = getattr(quote, "age_ms", None)
            feed_stale = bool(age_ms is not None and float(age_ms) > 120_000)
            lv = apply_price_event_to_watch_levels(
                levels,
                side=str(row.get("side") or "LONG"),
                last_price=last_px,
                feed_stale=feed_stale,
                reconcile_pending=False,
                session_actionable=session_actionable,
            )
            row["levels"] = lv
            row["level_status"] = lv.get("level_status")
            row["invalidation_latched"] = lv.get("invalidation_latched")
            changed = True
            out.append(row)

        if not changed:
            return
        payload["recommendations"] = out
        payload["levels_last_push_update_utc"] = datetime.now(timezone.utc).isoformat().replace(
            "+00:00", "Z"
        )
        write_next_session_recommendations(path, payload)

    def emit_next_session_recommendations_if_needed(
        self,
        *,
        now_ist: datetime,
        force: bool = False,
    ) -> Optional[Dict[str, Any]]:
        """
        Once per completed IST session date, emit next-session intraday watchlist
        from today's fetch-backed bar history (POST_CLOSE only).
        """
        from scripts.rp608_nse_next_session_recs import (
            build_next_session_recommendations,
            l1_snapshots_from_latest_evaluations,
            load_rp607_classified_symbols,
            next_weekday_session,
            write_next_session_recommendations,
        )

        from_date = str(
            self.current_session_date
            or getattr(self.state, "current_session_date", "")
            or now_ist.date()
        )
        if not force and getattr(self.state, "next_session_recs_emitted_for", "") == from_date:
            return {
                "emitted": False,
                "reason": "already_emitted",
                "from_session_date": from_date,
                "path": str(self.next_session_recs_path),
            }

        try:
            d0 = date.fromisoformat(from_date)
        except ValueError:
            d0 = now_ist.date()
            from_date = str(d0)

        nxt = next_weekday_session(d0, holidays=set(getattr(self, "exchange_holidays", set()) or set()))
        # Prefer in-memory last-eval L1; fall back to durable latest_evaluations.json
        l1_by_symbol: Dict[str, Dict[str, Any]] = {}
        for sym, ev in (getattr(self, "last_evaluations", {}) or {}).items():
            if not isinstance(ev, dict):
                continue
            dq = ev.get("data_quality") if isinstance(ev.get("data_quality"), dict) else {}
            try:
                bid_qty = float(dq.get("bid_qty") or 0.0)
                ask_qty = float(dq.get("ask_qty") or 0.0)
            except (TypeError, ValueError):
                continue
            snap = {
                "bid_qty": bid_qty,
                "ask_qty": ask_qty,
                "source": "daemon_last_evaluations_l1",
            }
            for src_k, dst_k in (
                ("bid_price", "bid_price"),
                ("ask_price", "ask_price"),
                ("best_bid", "bid_price"),
                ("best_ask", "ask_price"),
            ):
                if ev.get(src_k) is not None and dst_k not in snap:
                    try:
                        snap[dst_k] = float(ev[src_k])
                    except (TypeError, ValueError):
                        pass
            l1_by_symbol[str(sym)] = snap
        if not l1_by_symbol:
            l1_by_symbol = l1_snapshots_from_latest_evaluations(self.latest_evaluations_path)

        # Actionable watch universe = RP-607 classified 102 (exclude C_Quarantined)
        rp607_path = self.output_dir / "rp607_stock_profiles.json"
        classified = load_rp607_classified_symbols(rp607_path)
        if not classified:
            # Fail-open only if profile artifact missing; still prefer 102 when present
            classified = None  # type: ignore[assignment]

        payload = build_next_session_recommendations(
            self.price_history,
            from_session_date=from_date,
            next_session_date=str(nxt),
            top_n=20,
            now_utc=datetime.now(timezone.utc),
            l1_by_symbol=l1_by_symbol,
            eligible_symbols=classified,
            universe_label="RP-607-classified-102",
        )
        write_next_session_recommendations(self.next_session_recs_path, payload)
        self.state.next_session_recs_emitted_for = from_date
        print(
            f"[next_session_recs] emitted {payload.get('n_recommendations')} watches "
            f"({payload.get('n_order_flow_supported', 0)} order-flow supported) "
            f"for {payload.get('next_session_date')} → {self.next_session_recs_path}"
        )
        return {
            "emitted": True,
            "from_session_date": from_date,
            "next_session_date": str(nxt),
            "n_recommendations": payload.get("n_recommendations"),
            "n_order_flow_supported": payload.get("n_order_flow_supported"),
            "path": str(self.next_session_recs_path),
        }

    async def run_async_loop(self):
        """Asynchronous main loop with graceful termination handling."""
        self.running = True
        self.stop_event = asyncio.Event()

        loop = asyncio.get_running_loop()
        for sig in (signal.SIGTERM, signal.SIGINT):
            loop.add_signal_handler(sig, self._handle_signal, sig)

        print(f"[{datetime.now(timezone.utc).isoformat()}] RP-608 NSE Shadow Daemon started (Output: {self.output_dir})")
        try:
            # Token→symbol map for OFX tee + Estimand A WS tick fills (before start).
            self._rebuild_token_to_symbol()
            tok_map = dict(self._token_to_symbol)
            # Always register: Estimand A (if TICK_PATH) + OF/OFX per-tick paper actions.
            self.feed_adapter.register_tick_sink(self._on_estimand_a_ws_tick)
            print(
                f"[+] WS tick actions: Estimand A ENTRY_PATH={ENTRY_PATH} "
                f"(armed_signal_bar={self._estimand_a_armed_signal_bar}); "
                f"OF L1 per tick; OFX volume on trade-like ticks; "
                f"trade_like_gate={int(TICK_TRADE_LIKE_ENTRY)}; tokens={len(tok_map)}",
                flush=True,
            )
            if getattr(self, "ofx_tick_journal", None) is not None:
                self.ofx_tick_journal.set_token_map(tok_map)
                if self.ofx_tick_journal.enabled:
                    self.feed_adapter.register_tick_sink(self.ofx_tick_journal.on_tick)
                    print(
                        f"[+] OFX Stage 0D inferred-aggressor tick capture ENABLED → {self.ofx_tick_dir} "
                        f"(authority=0.0%; OF rules untouched)",
                        flush=True,
                    )
                    print(
                        f"[+] OFX NSE volume PAPER armed → {self.ofx_tick_dir} "
                        "(RP-608-OFX-NSE-VOLUME-PAPER-001; DERIVED_VALIDATED; baseline L1 OF inert)",
                        flush=True,
                    )
                    try:
                        now_ist_boot = datetime.now(timezone.utc) + timedelta(hours=5, minutes=30)
                        win_boot, _ = resolve_session_window(
                            now_ist_boot.replace(tzinfo=None),
                            exchange_holidays=self.exchange_holidays,
                        )
                        if win_boot is not None and win_boot.kind == "MUHURAT":
                            print(
                                f"[+] NSE MUHURAT session calendar armed today "
                                f"{win_boot.open_ist.strftime('%H:%M')}–"
                                f"{win_boot.close_ist.strftime('%H:%M')} IST "
                                f"({win_boot.label}; warmup_bars={win_boot.warmup_completed_bars})",
                                flush=True,
                            )
                    except Exception as muh_exc:
                        print(f"[muhurat] calendar note skipped: {muh_exc}", flush=True)
                else:
                    print("[*] OFX Stage 0D tick capture disabled (RP608_OFX_NSE_TICK_CAPTURE)", flush=True)
            self.feed_adapter.start_ticker()
        except Exception as e:
            print(f"[!] Kite ticker start skipped: {e}", file=sys.stderr)

        try:
            while self.running:
                try:
                    # Run iteration in thread pool to prevent blocking event loop on network I/O
                    await asyncio.to_thread(self.run_single_iteration)
                except Exception as e:
                    print(f"ERROR in run_single_iteration: {e}", file=sys.stderr)

                if self.single_bar_test:
                    print("Single-bar test completed.")
                    break

                try:
                    # Sleep 15 seconds or until interrupted by signal
                    await asyncio.wait_for(self.stop_event.wait(), timeout=15.0)
                except asyncio.TimeoutError:
                    pass
        finally:
            try:
                self.feed_adapter.stop_ticker()
            except Exception:
                pass
            try:
                if getattr(self, "ofx_tick_journal", None) is not None:
                    self.ofx_tick_journal.close()
            except Exception:
                pass

        print(f"[{datetime.now(timezone.utc).isoformat()}] RP-608 NSE Shadow Daemon terminating cleanly...")
        self.state.last_heartbeat_utc_ms = int(time.time() * 1000)
        self.state.last_heartbeat_iso = datetime.now(timezone.utc).isoformat()
        self.atomic_commit_checkpoint(self.state)

    def _handle_signal(self, sig):
        print(f"\nReceived signal {sig}, waking main loop for graceful shutdown...")
        self.running = False
        if self.stop_event:
            self.stop_event.set()


def parse_args():
    parser = argparse.ArgumentParser(description="RP-608 NSE Prospective Shadow Observation Daemon")
    parser.add_argument("--output-dir", type=str, default=None, help="Directory for checkpoint, telemetry, and ledgers")
    parser.add_argument("--single-bar-test", action="store_true", help="Run a single bar iteration and exit cleanly")
    parser.add_argument("--setup-only", action="store_true", help="Initialize directory and checkpoint then exit")
    parser.add_argument("--reset-outages", action="store_true", help="Reset consecutive outages counter on authorized service start")
    parser.add_argument(
        "--emit-next-session-recs",
        action="store_true",
        help="Force-emit next-session watchlist from bar history and exit (post-close tool)",
    )
    return parser.parse_args()


def main():
    args = parse_args()
    daemon = RP608NSEShadowDaemon(
        output_dir=args.output_dir,
        single_bar_test=args.single_bar_test,
        reset_outages=args.reset_outages
    )

    if args.setup_only:
        print(f"Setup complete. Output dir: {daemon.output_dir}")
        print(f"Checkpoint status: {daemon.state.status}, checksum: {daemon.state.checksum}")
        sys.exit(0)

    if args.emit_next_session_recs:
        ist = datetime.now(timezone.utc) + timedelta(hours=5, minutes=30)
        meta = daemon.emit_next_session_recommendations_if_needed(now_ist=ist, force=True)
        daemon.atomic_commit_checkpoint(daemon.state)
        print(json.dumps(meta, indent=2))
        sys.exit(0)

    if args.single_bar_test:
        res = daemon.run_single_iteration()
        print("Single bar execution result:", json.dumps(res, indent=2))
        sys.exit(0)

    asyncio.run(daemon.run_async_loop())


if __name__ == "__main__":
    main()
