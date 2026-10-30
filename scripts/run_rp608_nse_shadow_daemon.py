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
from scripts.rp608_prospective_v1 import Market as ProspectiveMarket
from scripts.rp608_prospective_v1_observer import (
    observe_bar as prospective_v1_observe_bar,
    record_recommendation as prospective_v1_record_recommendation,
)
from scripts.rp608_recommendation_trade_ledger import (
    append_nse_recommendation_trade_link as prospective_v1_append_nse_rec_trade,
    append_nse_trade_closed as prospective_v1_append_nse_trade_closed,
)
from scripts.rp608_kite_ticker_cache import KiteTickerQuoteCache

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
ARM_A_STOP_MULT = 0.990
ARM_A_TARGET_MULT = 1.015
ARM_A_STOP_BPS = 100.0
ARM_A_TARGET_BPS = 150.0
ATR_FEATURE_LOOKBACK_DAYS = 5
ATR_FEATURE_MAX_BARS = 40

# Statutory Friction Schedule (Zerodha Cash-Equity Intraday)
BROKERAGE_RATE = 0.0003          # 0.03%
BROKERAGE_CAP_PER_LEG = 20.0     # min(₹20, Turnover * 0.03%)
STT_RATE_SELL_LEG = 0.00025      # 0.025% sell side
EXCHANGE_TURNOVER_RATE = 0.0000297  # 0.00297%
GST_RATE = 0.18                  # 18.0% on (Brokerage + Exchange + SEBI)
SEBI_TURNOVER_RATE = 0.000001    # ₹10 per crore (0.0001%)
STAMP_DUTY_BUY_LEG = 0.00003     # 0.003% buy side
SLIPPAGE_RATE_PER_LEG = 0.0002   # 2.0 bps adverse execution latency

DEFAULT_NOTIONAL_PER_TRADE_INR = 100_000.0  # ₹100,000 INR reference notional

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


def calc_indian_equity_friction(
    buy_price: float,
    sell_price: float,
    quantity: float,
    half_spread_bps: float = 2.5
) -> Dict[str, float]:
    """
    Computes exact 8-component statutory & transaction friction for an intraday round trip.
    """
    buy_turnover = buy_price * quantity
    sell_turnover = sell_price * quantity
    total_turnover = buy_turnover + sell_turnover

    # 1. Brokerage: min(₹20, Turnover * 0.03%) per leg
    brok_buy = min(buy_turnover * BROKERAGE_RATE, BROKERAGE_CAP_PER_LEG)
    brok_sell = min(sell_turnover * BROKERAGE_RATE, BROKERAGE_CAP_PER_LEG)
    brokerage = brok_buy + brok_sell

    # 2. STT: 0.025% on sell leg only
    stt = sell_turnover * STT_RATE_SELL_LEG

    # 3. Exchange transaction charges: 0.00297% on total turnover
    exchange_charges = total_turnover * EXCHANGE_TURNOVER_RATE

    # 4. SEBI turnover charges: ₹10 per crore (0.0001%)
    sebi_charges = total_turnover * SEBI_TURNOVER_RATE

    # 5. GST: 18.0% on (Brokerage + Exchange + SEBI)
    gst = GST_RATE * (brokerage + exchange_charges + sebi_charges)

    # 6. Stamp duty: 0.003% on buy leg only
    stamp_duty = buy_turnover * STAMP_DUTY_BUY_LEG

    # 7. Spread crossing: Half-spread on both legs
    spread_rate = (half_spread_bps / 10000.0)
    spread_cost = (buy_turnover * spread_rate) + (sell_turnover * spread_rate)

    # 8. Slippage: 2.0 bps adverse latency on both legs
    slippage_cost = (buy_turnover * SLIPPAGE_RATE_PER_LEG) + (sell_turnover * SLIPPAGE_RATE_PER_LEG)

    total_friction = (
        brokerage + stt + exchange_charges + sebi_charges + gst + stamp_duty + spread_cost + slippage_cost
    )

    return {
        "brokerage": round(brokerage, 4),
        "stt": round(stt, 4),
        "exchange_charges": round(exchange_charges, 4),
        "sebi_charges": round(sebi_charges, 4),
        "gst": round(gst, 4),
        "stamp_duty": round(stamp_duty, 4),
        "spread_cost": round(spread_cost, 4),
        "slippage_cost": round(slippage_cost, 4),
        "total_friction": round(total_friction, 4),
        "total_turnover": round(total_turnover, 2),
        "effective_friction_bps": round((total_friction / buy_turnover) * 10000.0, 2) if buy_turnover > 0 else 0.0
    }


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
        self._initialize_instrument_map()

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
        )
        self.ticker_cache.start()
        print(
            f"[+] Kite WebSocket L1 cache started for {len(tokens)} tokens "
            f"(mode=full, authority=0.0%)",
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

            # Completed / current bar representation
            bar = BarOHLCV(
                timestamp_utc_ms=t_req_utc,
                open=o,
                high=h,
                low=l,
                close=last_price if last_price > 0 else c,
                volume=v
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
                        source="kite_ws_full",
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
        self.opportunity_path = self.output_dir / "rp608_nse_opportunity_registry.jsonl"
        self.evaluations_path = self.output_dir / "rp608_nse_evaluations.jsonl"
        self.latest_evaluations_path = self.output_dir / "rp608_nse_latest_evaluations.json"
        self.bar_history_path = self.output_dir / "rp608_nse_bar_history.json"
        self.pending_evaluations_path = self.output_dir / "rp608_nse_pending_evaluations.json"
        self.next_session_recs_path = self.output_dir / "rp608_nse_next_session_recommendations.json"

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
        # Forward outcome pending queue and active trades restored from durable storage across restarts:
        self.pending_directional_evals, self.active_trades = self.load_pending_evaluations()
        # Scrip-level decision ledger cache: {symbol: evaluation_record}
        self.last_evaluations: Dict[str, Dict[str, Any]] = {}
        # ATR/structure feature bars (may include prior sessions). Never used for traj15.
        self._atr_feature_cache: Dict[str, List[Any]] = {}
        self._atr_feature_cache_bar_idx: int = -1

        self.state = self.initialize_or_resume_state()
        if getattr(self.state, "current_session_date", ""):
            self.current_session_date = self.state.current_session_date

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

    def atomic_commit_json(self, target_path: Path, data: Any):
        """Atomically persists arbitrary JSON data to disk via tmp file + replace + fsync."""
        tmp_path = target_path.with_suffix(".tmp")
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

    def load_pending_evaluations(self) -> Tuple[Dict[int, Dict[str, Dict[str, Any]]], Dict[str, Dict[str, Any]]]:
        """Loads pending forward directional evaluations and active trades from durable storage."""
        pending_evals: Dict[int, Dict[str, Dict[str, Any]]] = {}
        active_trades: Dict[str, Dict[str, Any]] = {}
        if hasattr(self, "pending_evaluations_path") and self.pending_evaluations_path.exists():
            try:
                with open(self.pending_evaluations_path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                raw_pending = data.get("pending_directional_evals", {})
                for k, v in raw_pending.items():
                    pending_evals[int(k)] = v
                active_trades = data.get("active_trades", {})
            except Exception as e:
                print(f"[!] Warning: Failed to load pending evaluations from {self.pending_evaluations_path}: {e}", file=sys.stderr)
        return pending_evals, active_trades

    def save_pending_evaluations(self):
        """Atomically persists pending directional evaluations and active trades to durable storage."""
        payload = {
            "pending_directional_evals": {str(k): v for k, v in self.pending_directional_evals.items()},
            "active_trades": self.active_trades,
            "updated_at_utc_ms": int(time.time() * 1000),
            "updated_at_iso": datetime.now(timezone.utc).isoformat()
        }
        self.atomic_commit_json(self.pending_evaluations_path, payload)

    def bootstrap_same_day_history(
        self,
        now_utc: Optional[datetime] = None,
        target_symbols: Optional[Iterable[str]] = None
    ) -> Dict[str, Any]:
        """
        Recovers completed 15m bars for today's session from Kite historical data.
        Requirements:
        1. Only current trading session completed bars are fetched.
        2. No bootstrap at opening (< 5 completed bars).
        3. Excludes forming/incomplete candle.
        4. Non-destructive merge: preserves live observations over historical duplicates.
        5. Fails closed on API failure without crashing or manufacturing eligibility.
        6. Per-symbol recovery: queries only deficient symbols and throttles polling per bar.
        """
        if now_utc is None:
            now_utc = datetime.now(timezone.utc)
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = now_utc + ist_offset

        # Check weekend
        if now_ist.weekday() >= 5:
            return {"status": "WEEKEND_CLOSED", "recovered_symbols": 0}

        # Check exchange holiday
        if now_ist.date() in getattr(self, "exchange_holidays", NSE_2026_HOLIDAYS):
            return {"status": "EXCHANGE_HOLIDAY", "recovered_symbols": 0}

        open_time_ist = now_ist.replace(hour=9, minute=15, second=0, microsecond=0)

        # Before market open
        if now_ist < open_time_ist:
            return {"status": "PRE_OPEN", "recovered_symbols": 0}

        # Calculate completed 15m intervals since 09:15 IST
        elapsed_sec = (now_ist - open_time_ist).total_seconds()
        completed_bars = int(elapsed_sec // (15 * 60))
        completed_bars = min(completed_bars, TOTAL_BARS_PER_SESSION)

        # Rule 2: Do not bootstrap at the opening.
        # Before the 5th completed bar (i.e. before 10:30 IST), Gate 3 must remain cold.
        if completed_bars < 5:
            return {
                "status": "OPENING_WARMUP_COLD",
                "completed_bars_today": completed_bars,
                "recovered_symbols": 0,
                "reason": f"Only {completed_bars}/5 completed bars today; opening warm-up active"
            }

        # Pick up dashboard re-auth before same-day hist storm (next-session + mid-day rotate).
        reload_fn = getattr(self.feed_adapter, "reload_credentials_if_stale", None)
        if callable(reload_fn):
            reload_fn()

        now_mono = time.monotonic()
        need_n = min(completed_bars, 5)

        # Determine symbols that actually need recovery
        if target_symbols is not None:
            symbols_to_query = list(target_symbols)
        else:
            symbols_to_query = []
            for s in self.symbols:
                if len(self.price_history.get(s, [])) >= need_n:
                    continue
                # Successful empty/complete attempt for this bar: wait for next completed bar.
                if self._symbol_last_bootstrap_bar.get(s, 0) >= completed_bars:
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
                "total_backfilled_bars": 0
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

        for sym in symbols_to_query:
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
                # Definitive empty range for this bar — do not re-poll until next completed bar.
                self._symbol_last_bootstrap_bar[sym] = completed_bars
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
                continue

            # Non-destructive merge with existing price_history:
            # Preserve existing live observations, backfill missing bars
            current_bars = self.price_history.get(sym, [])

            # Map existing bars by their candle start timestamp in UTC ms
            existing_by_start: Dict[int, BarOHLCV] = {}
            for b in current_bars:
                try:
                    b_ist = datetime.fromtimestamp(b.timestamp_utc_ms / 1000.0, tz=timezone.utc) + ist_offset
                    if b_ist.date() != now_ist.date():
                        continue
                    el_min = (b_ist - open_time_ist).total_seconds() / 60.0
                    if el_min < 0:
                        continue
                    c_idx = int(el_min // 15)
                    c_start_ist = open_time_ist + timedelta(minutes=c_idx * 15)
                    c_start_ms = int((c_start_ist - ist_offset).timestamp() * 1000)
                    if c_start_ms not in existing_by_start:
                        existing_by_start[c_start_ms] = b
                except Exception:
                    continue

            added_for_sym = 0
            for hb in valid_hist:
                try:
                    hb_ist = datetime.fromtimestamp(hb.timestamp_utc_ms / 1000.0, tz=timezone.utc) + ist_offset
                    el_min = (hb_ist - open_time_ist).total_seconds() / 60.0
                    c_idx = int(el_min // 15)
                    c_start_ist = open_time_ist + timedelta(minutes=c_idx * 15)
                    c_start_ms = int((c_start_ist - ist_offset).timestamp() * 1000)

                    if c_start_ms not in existing_by_start:
                        existing_by_start[c_start_ms] = hb
                        added_for_sym += 1
                except Exception:
                    continue

            # Reconstruct sorted price_history for symbol
            merged = [existing_by_start[k] for k in sorted(existing_by_start.keys())]
            if len(merged) > 60:
                merged = merged[-60:]
            self.price_history[sym] = merged

            if added_for_sym > 0:
                recovered_count += 1
                total_backfilled_bars += added_for_sym

            # Mark bar complete only once warm enough; otherwise retry within cooldown/next pass
            # (covers filtered-empty responses without consuming the bar slot forever).
            if len(self.price_history.get(sym, [])) >= need_n:
                self._symbol_last_bootstrap_bar[sym] = completed_bars
            else:
                self._symbol_bootstrap_fail_mono[sym] = time.monotonic()

        if transient_failures:
            print(
                f"[!] Bootstrap transient failures={transient_failures}/"
                f"{len(symbols_to_query)} (will retry after {BOOTSTRAP_FAIL_COOLDOWN_SEC:.0f}s cooldown)",
                flush=True,
            )

        return {
            "status": "BOOTSTRAP_SUCCESS",
            "completed_bars_today": completed_bars,
            "recovered_symbols": recovered_count,
            "total_backfilled_bars": total_backfilled_bars,
            "transient_failures": transient_failures,
            "queried_symbols": len(symbols_to_query),
        }

    def append_jsonl(self, path: Path, record: Dict[str, Any]):
        """Appends a single JSON record with durable flush."""
        with open(path, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, separators=(",", ":")) + "\n")
            f.flush()

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
        Determines if the current moment is within the NSE active session (09:15 to 15:30 IST).
        Returns: (is_active, bar_idx, phase_name)
          bar_idx: 1..25 if active; 0 if pre-open / post-close
        """
        if now_utc is None:
            now_utc = datetime.now(timezone.utc)
        ist_offset = timedelta(hours=5, minutes=30)
        now_ist = now_utc + ist_offset

        # Check weekend (Saturday = 5, Sunday = 6)
        if now_ist.weekday() >= 5:
            return False, 0, "WEEKEND_CLOSED"

        # Check exchange holiday
        if now_ist.date() in getattr(self, "exchange_holidays", NSE_2026_HOLIDAYS):
            return False, 0, "EXCHANGE_HOLIDAY"

        # Check session hours
        open_time = now_ist.replace(hour=9, minute=15, second=0, microsecond=0)
        close_time = now_ist.replace(hour=15, minute=30, second=0, microsecond=0)

        if now_ist < open_time:
            return False, 0, "PRE_OPEN"
        if now_ist >= close_time:
            return False, 0, "POST_CLOSE"

        # Elapsed minutes from 09:15
        elapsed_min = (now_ist - open_time).total_seconds() / 60.0
        bar_idx = int(elapsed_min // 15) + 1
        bar_idx = min(bar_idx, TOTAL_BARS_PER_SESSION)
        return True, bar_idx, "ACTIVE_SESSION"

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

    def evaluate_symbol_bar(
        self,
        symbol: str,
        bar_idx: int,
        latest_bar: BarOHLCV,
        quote: Optional[TopOfBookQuote]
    ) -> Optional[Dict[str, Any]]:
        """
        Evaluates a completed 15m bar for a symbol.
        Emits Estimand B directional decision card and manages Estimand A trade lifecycle.
        """
        history = self.price_history[symbol]
        if history and history[-1].timestamp_utc_ms == latest_bar.timestamp_utc_ms:
            history[-1] = latest_bar
        else:
            history.append(latest_bar)
        if len(history) > 60:
            history.pop(0)

        # ─── Causal Momentum Scoring ───────────────────────────
        closes = [b.close for b in history]
        if len(closes) < 2:
            signal_entry = False
            traj15 = 0.0
        else:
            # 15m momentum trajectory
            lookback = min(len(closes), 15)
            ret = (closes[-1] - closes[-lookback]) / (closes[-lookback] + 1e-9)
            diffs = [(closes[i] - closes[i-1]) / (closes[i-1] + 1e-9) for i in range(-lookback + 1, 0)]
            vol = (sum(d*d for d in diffs) / len(diffs))**0.5 if diffs else 1e-3
            denom = (vol * (lookback**0.5) * 2.5) + 1e-9
            traj15 = math.tanh(ret / denom)
            # Signal entry rule: Positive momentum and non-warmup
            signal_entry = (traj15 >= 0.05) and (len(closes) >= 5)

        # Gate G4 Quote Validation (Freshness, Non-Zero, Uncrossed Top-of-Book)
        MAX_QUOTE_AGE_MS = 60000  # 60s max permitted age
        is_fresh = True
        quote_age_ms = None
        if quote and quote.timestamp_utc_ms > 0 and latest_bar.timestamp_utc_ms > 0:
            quote_age_ms = abs(latest_bar.timestamp_utc_ms - quote.timestamp_utc_ms)
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
        # Mandatory square-off at Bar 23 close (15:00 IST) — adapt to 15:00–15:15
        # closing window / F&O continuous halt (do not carry into CAS).
        if bar_idx >= MANDATORY_SQUARE_OFF_BAR and symbol in self.active_trades:
            trade = self.active_trades.pop(symbol)
            self._close_trade(
                trade,
                exit_price=p_ref,
                exit_bar=bar_idx,
                exit_reason="MANDATORY_INTRADAY_SQUARE_OFF",
            )

        # Check existing open trade for exit conditions
        elif symbol in self.active_trades:
            trade = self.active_trades[symbol]
            entry_p = trade["entry_price"]
            # Excursions
            if p_ref > trade["mfe_price"]:
                trade["mfe_price"] = p_ref
            if p_ref < trade["mae_price"]:
                trade["mae_price"] = p_ref

            bars_held = bar_idx - trade["entry_bar_idx"]
            # Upgrade legacy open trades (fixed ±1%/1.5%) once to entry-frozen dynamic bounds.
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
            # Prefer entry-frozen dynamic bounds; Arm A % only if still unset.
            sl_price = float(trade.get("stop_loss_price") or (entry_p * ARM_A_STOP_MULT))
            tp_price = float(trade.get("target_price") or (entry_p * ARM_A_TARGET_MULT))

            if p_ref <= sl_price:
                self._close_trade(self.active_trades.pop(symbol), exit_price=p_ref, exit_bar=bar_idx, exit_reason="STOP_LOSS")
            elif p_ref >= tp_price:
                self._close_trade(self.active_trades.pop(symbol), exit_price=p_ref, exit_bar=bar_idx, exit_reason="TAKE_PROFIT")
            elif bars_held >= 12:
                self._close_trade(self.active_trades.pop(symbol), exit_price=p_ref, exit_bar=bar_idx, exit_reason="MAX_HOLD_HORIZON_180M")
            elif bars_held >= 4 and p_ref > entry_p:
                self._close_trade(self.active_trades.pop(symbol), exit_price=p_ref, exit_bar=bar_idx, exit_reason="TARGET_HORIZON_60M")

        # Entry: bars 1..MAX_NEW_ENTRY_BAR so 60m horizon completes by 15:00 close-window.
        elif (
            signal_entry
            and (1 <= bar_idx <= MAX_NEW_ENTRY_BAR)
            and symbol not in self.active_trades
        ):
            trade_id = f"nse_trade_{self.state.session_idx}_{bar_idx}_{symbol}"
            qty = max(1.0, math.floor(DEFAULT_NOTIONAL_PER_TRADE_INR / p_ref))
            half_spread = quote.half_spread_bps if (quote and quote.half_spread_bps > 0) else 2.5
            # Rough round-trip friction proxy for Net R:R gate inside dynamic bounds.
            friction_proxy_bps = max(9.0, float(half_spread) * 2.0 + 8.0)
            bounds = self.resolve_exit_boundaries(symbol, p_ref, history, friction_proxy_bps)
            self.active_trades[symbol] = {
                "protocol_id": PROTOCOL_ID,
                "estimand": "ESTIMAND_A",
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
                "balanced_accuracy": None,  # Strictly None / N/A for Estimand A
                "stop_loss_price": bounds["stop_price"],
                "target_price": bounds["target_price"],
                "stop_loss_bps": bounds["stop_bps"],
                "target_bps": bounds["target_bps"],
                "atr_14": bounds.get("atr_14"),
                "exit_policy": bounds.get("policy_id"),
                "exit_trend_label": bounds.get("trend_label"),
                "exit_fallback_reason": bounds.get("fallback_reason"),
            }

        self.state.active_positions_count = len(self.active_trades)

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
        elif len(closes) < 5:
            decision_code = "COLD_START_INSUFFICIENT_HISTORY"
            explanation = f"Warmup history: {len(closes)}/5 bars required"
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
        else:
            decision_code = "QUALIFIED_MOMENTUM_BREAKOUT"
            explanation = f"Momentum breakout traj15={traj15:+.4f} >= +0.0500 with {len(closes)} bars history"
            decision = "RECOMMEND_LONG"

        rr_dict = None
        inval_text = (
            f"p <= entry_price * {ARM_A_STOP_MULT:.3f} ({ARM_A_STOP_BPS:.0f} bps stop) "
            f"or 4-bar hold without gain or max 12 bars"
        )
        if decision == "RECOMMEND_LONG":
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
                "net_rr_ratio": round((tgt_bps - eff_fric) / (sl_bps + eff_fric + 1e-9), 2)
            }
            inval_text = (
                f"p <= ₹{stop_p:.2f} (−{sl_bps:.0f} bps, {bounds.get('policy_id')}) "
                f"or TP ₹{target_p:.2f} (+{tgt_bps:.0f} bps) or 4-bar/12-bar horizon"
            )

        now_eval_ms = int(time.time() * 1000)
        q_ts = quote.timestamp_utc_ms if quote else None
        q_age_ms = (now_eval_ms - q_ts) if q_ts else None

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
            "history_length": len(closes),
            "bar_provenance": getattr(latest_bar, "source", "kite_live"),
            "history_provenance": {
                "live_count": sum(1 for b in history if getattr(b, "source", "kite_live") in ("kite_live", "persisted_live")),
                "historical_bootstrap_count": sum(1 for b in history if getattr(b, "source", "") == "kite_historical_same_day")
            },
            "half_spread_bps": round(quote.half_spread_bps, 2) if quote else None,
            "entry_conditions": (
                f"traj15 >= 0.05 and history >= 5 and 1 <= bar_idx <= {MAX_NEW_ENTRY_BAR} "
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

        return card_b

    def _close_trade(self, trade: Dict[str, Any], exit_price: float, exit_bar: int, exit_reason: str):
        """Calculates exact statutory friction and closes trade for Estimand A."""
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

        record = {
            **trade,
            "exit_bar_idx": exit_bar,
            "exit_timestamp_utc_ms": int(time.time() * 1000),
            "exit_price": exit_price,
            "exit_reason": exit_reason,
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

        self.append_jsonl(self.trade_journal_path, record)
        self.state.completed_trades_count += 1
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
            if phase == "POST_CLOSE":
                try:
                    next_sess_meta = self.emit_next_session_recommendations_if_needed(now_ist=now_ist)
                except Exception as nsr_exc:
                    print(f"[next_session_recs] emit skipped: {nsr_exc}")
            self.atomic_commit_checkpoint(self.state)
            out = {"status": phase, "bar_idx": 0, "processed": 0}
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
            # Fresh session: always load latest Kite token (daily rotate / overnight re-auth).
            reload_fn = getattr(self.feed_adapter, "reload_credentials_if_stale", None)
            if callable(reload_fn):
                reload_fn(force=True)
            self.atomic_commit_checkpoint(self.state)
        elif str(self.current_session_date) != today_session_str:
            self.state.session_idx += 1
            self.state.last_durable_bar_idx = 0
            self.state.last_accounted_bar_idx = 0
            self.price_history = {s: [] for s in self.symbols}
            self.active_trades.clear()
            self.pending_directional_evals.clear()
            self.last_evaluations.clear()
            self._symbol_last_bootstrap_bar.clear()
            self._symbol_bootstrap_fail_mono.clear()
            self.current_session_date = today_session_str
            self.state.current_session_date = today_session_str
            # Next session must not inherit yesterday's expired access_token in memory.
            reload_fn = getattr(self.feed_adapter, "reload_credentials_if_stale", None)
            if callable(reload_fn):
                reload_fn(force=True)
            self.save_bar_history()
            self.save_pending_evaluations()
            self.atomic_commit_checkpoint(self.state)

        # Determine completed bars vs forming bars in current session
        open_time = now_ist.replace(hour=9, minute=15, second=0, microsecond=0)
        elapsed_sec = (now_ist - open_time).total_seconds()
        completed_bars = int(elapsed_sec // (15 * 60))
        completed_bars = min(completed_bars, TOTAL_BARS_PER_SESSION)

        if not self.single_bar_test and is_active:
            if completed_bars == 0:
                # Interval 1 is still forming (09:15-09:30 IST); wait for candle completion
                self.state.last_heartbeat_utc_ms = now_ms
                self.state.last_heartbeat_iso = now_utc.isoformat()
                self.atomic_commit_checkpoint(self.state)
                return {"status": "FORMING_BAR_1", "bar_idx": 0, "processed": 0}

            if completed_bars <= self.state.last_durable_bar_idx:
                # Candle already evaluated at close and committed to disk; await next bar close
                self.state.last_heartbeat_utc_ms = now_ms
                self.state.last_heartbeat_iso = now_utc.isoformat()
                self.atomic_commit_checkpoint(self.state)
                return {
                    "status": "AWAITING_NEXT_BAR_CLOSE",
                    "last_durable_bar_idx": self.state.last_durable_bar_idx,
                    "completed_bars": completed_bars,
                    "next_target_bar": completed_bars + 1
                }

            # A new 15m candle has finalized! Evaluate this finalized completed bar.
            eval_bar = completed_bars
        else:
            eval_bar = bar_idx

        # Auto-recovery: If active session at bar >= 5, attempt same-day recovery for any deficient symbols
        if is_active and eval_bar >= 5:
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
                self.evaluate_symbol_bar(sym, eval_bar, latest_bar, quote)
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
                    }
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

        # ─── Persist Every Evaluation to Durable Ledger ─────────────
        for r in iteration_evals:
            self.append_jsonl(self.evaluations_path, r)

        # ─── Commit Atomic Snapshot for Immediate Dashboard Access ──
        latest_eval_snapshot = {
            "timestamp_utc_ms": now_ms,
            "timestamp_iso": now_utc.isoformat(),
            "session_idx": self.state.session_idx,
            "bar_idx": eval_bar,
            "total_evaluated": len(iteration_evals),
            "n_qualifying": len(qualifying),
            "rejection_breakdown": rejection_breakdown,
            "qualifying_candidates": qualifying,
            "evaluations": {r["symbol"]: r for r in iteration_evals}
        }
        self.atomic_commit_json(self.latest_evaluations_path, latest_eval_snapshot)
        self.save_bar_history()
        self.save_pending_evaluations()

        self.state.last_durable_bar_idx = eval_bar
        self.state.last_accounted_bar_idx = max(self.state.last_accounted_bar_idx, eval_bar)
        self.state.active_positions_count = len(self.active_trades)
        self.state.consecutive_outages = 0
        self.state.last_heartbeat_utc_ms = now_ms
        self.state.last_heartbeat_iso = now_utc.isoformat()
        self.atomic_commit_checkpoint(self.state)

        # Log telemetry heartbeat
        telem = {
            "timestamp_utc_ms": now_ms,
            "timestamp_iso": now_utc.isoformat(),
            "session_idx": self.state.session_idx,
            "bar_idx": eval_bar,
            "phase": phase,
            "successful_symbols": successful,
            "failed_symbols": failed,
            "evaluations_count": len(iteration_evals),
            "qualifying_recommendations": len(qualifying),
            "active_positions": len(self.active_trades),
            "completed_trades": self.state.completed_trades_count,
            "directional_cards": self.state.directional_cards_count,
            "unexecutable_drops": self.state.unexecutable_drops_count,
            "pending_directional_evals_count": sum(len(v) for v in self.pending_directional_evals.values())
        }
        self.append_jsonl(self.telemetry_path, telem)
        return telem

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
        payload = build_next_session_recommendations(
            self.price_history,
            from_session_date=from_date,
            next_session_date=str(nxt),
            top_n=20,
            now_utc=datetime.now(timezone.utc),
        )
        write_next_session_recommendations(self.next_session_recs_path, payload)
        self.state.next_session_recs_emitted_for = from_date
        print(
            f"[next_session_recs] emitted {payload.get('n_recommendations')} watches "
            f"for {payload.get('next_session_date')} → {self.next_session_recs_path}"
        )
        return {
            "emitted": True,
            "from_session_date": from_date,
            "next_session_date": str(nxt),
            "n_recommendations": payload.get("n_recommendations"),
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
