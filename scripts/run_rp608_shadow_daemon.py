#!/usr/bin/env python3
"""
scripts/run_rp608_shadow_daemon.py

RP-608-SHADOW-TICK-001: Prospective Crypto Shadow Observation Daemon (tick decision clock).
Strictly Read-Only (0.0% Order Authority) with Atomic Durable Checkpointing.

Features:
  1. Concurrent WebSocket Ingestion (per --symbol; default BTCUSDT):
     - wss://stream.binance.com:9443/ws/{symbol}@bookTicker → canonical Policy B/C decisions
     - wss://stream.binance.com:9443/ws/{symbol}@kline_15m → completed-bar buffer for ATR(14) geometry
     - wss://…/{symbol}@aggTrade → OFX research durable journal only (baseline OF inert; env RP608_OFX_CRYPTO_AGGTRADE_CAPTURE)
     ETH/SOL run as separate processes + output dirs (books never pooled; Gate 2E off until symbol library).
  2. Protocol Invariant & Quality Gate Enforcement:
     - Clock offset calibration (|Delta_clock| <= 250 ms, 5-sample NTP minimum RTT)
     - Quote freshness boundary (<= 500 ms); min refractory 1.0s between decision emits
     - Strictly zero sequence update ID inversions (TG-5)
  3. Policy Champion Multi-Evaluation (tick-native):
     - Frozen tick champions (TickBase / TickWithExhaustion); SHA-locked
     - ATR(14) SL/TP geometry still from completed 15m OHLCV buffer
  4. Durable, Crash-Safe Checkpointing & Resumption:
     - Atomic rename (os.replace) via temporary file with os.fsync
     - Explicit downtime recording on reboot / process restart
     - OpportunityAccountingRegistry tracking all unexecutable drops (never diluting trade denominator)
     - Fail-closed behavior on checksum mismatch, hash tamper, or consecutive outage breach (>= 2 bars)

Gate C (event_driven/) remains an isolated parallel comparator — not canonical.
"""

from __future__ import annotations

import argparse
import asyncio
import copy
import hashlib
import http.client
import json
import math
import os
import signal
import ssl
import sys
import time
import urllib.request
from dataclasses import asdict, dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))


def _websockets():
    """Lazy import so unit tests can load helpers without the live WS dependency."""
    import websockets  # type: ignore

    return websockets

from scripts.market_regime_classifier import classify_market_regime
from scripts.rp608_arm_a_baseline import attach_regime_telemetry
from scripts.rp608_gate2e_regime_align import (
    evaluate_regime_align,
    regime_dict_from_card,
)
from scripts.rp608_crypto_tick_policy import (
    DECISION_CLOCK as TICK_DECISION_CLOCK,
    MIN_HISTORY_WARMUP_SEC,
    PROTOCOL_ID as TICK_PROTOCOL_ID,
    REC_HYGIENE_PROTOCOL_ID,
    RecommendationStabilizer,
    TickEmissionGate,
    TickFeatureBuffer,
    TickQuote,
    build_tick_snapshot,
    evaluate_tick_policies,
    evaluate_tick_policies_hygiened,
    snapshot_to_dict,
    validate_tick_quote,
)
from scripts.rp608_crypto_tick_policy_client import (
    log_python_fallback_once,
    resolve_tick_eval_mode,
    rust_eval_tick,
    tick_policy_backend,
)
from scripts.rp608_prospective_v1 import Market as ProspectiveMarket
from scripts.rp608_prospective_v1_observer import (
    observe_bar as prospective_v1_observe_bar,
    record_recommendation as prospective_v1_record_recommendation,
)
from scripts.rp608_recommendation_trade_ledger import (
    append_recommendation_trade_link as prospective_v1_append_rec_trade,
    append_trade_outcome_link as prospective_v1_append_trade_closed,
)
from scripts.rp608_order_flow_mark import (
    classify_order_flow_support,
    l1_from_book_ticker,
    l1_from_tick_features,
)
from scripts.rp608_trade_plan_overlay import TradePlanOverlayManager
from scripts.rp608_adapter_select_15m import twin_enabled as adapter_select_15m_twin_enabled
from scripts.rp608_ofx_crypto_aggtrade_journal import (
    OfxCryptoAggTradeJournal,
    ofx_crypto_aggtrade_capture_enabled,
)
from scripts.rp608_ofx_crypto_volume_flow_mark import refresh_ofx_volume_flow_mark
from scripts.rp608_ofx_volume_paper_trades import OfxVolumePaperManager
from scripts.rp608_singleton_lock import acquire_or_exit

# RP-608-REGIME-BREAKOUT-001 — Arm A freeze + observational regime telemetry
# Policy B actions / costs / horizons MUST remain unchanged while ARM_A_FROZEN.
ARM_A_FROZEN = True
ARM_A_FREEZE_PROTOCOL = "RP-608-REGIME-BREAKOUT-001"
LIVE_ORDER_AUTHORITY_PCT = 0.0  # Strictly 0.0% — no order placement surface
REGIME_TELEMETRY_ENABLED = True  # Observational only; must not alter Arm A actions
HOLDOUT_SESSIONS_SEALED = "41-68"
SHADOW_PROTOCOL_ID = TICK_PROTOCOL_ID  # RP-608-SHADOW-TICK-001
CHECKPOINT_WINDOW_ID = "RP-608-SHADOW-001B"  # durable window id (checkpoint schema lock)

# Locked SHA-256 Hashes — tick champions (RP-608-SHADOW-TICK-001)
LOCKED_POLICY_B_TICK_SHA256 = "acf17394caf402e204bd297510c26b67dcc8480ac7ad13130eb25343ccca314a"
LOCKED_POLICY_C_TICK_SHA256 = "be909f57201c3dd67591ec01ace6bbcec607f08b70ba35ca1804a4f7e34c08ff"
# Legacy 15m champions retained for research reference / migration audits only
LOCKED_POLICY_B_SHA256 = "f7be46ea66ff6a248ad0477dfe5b082ce26fe01f9dded928b164bdaa349c6aa3"
LOCKED_POLICY_C_SHA256 = "cd615f37b812a34fe302a0b05bc1546a0243f3d71d8947092d28d2847159553b"
LOCKED_DFP_PROTOCOL_SHA256 = "e8bca8ad9194ff058cc4eb4315f20c290d6465fce28fcfe0b675b6b133d494c3"
LOCKED_AMENDMENT_SHA256 = "c9467df6f9fa1b715c50b576f1af4e79716c86d7c8765db75f5fbe552a9f5202"
LOCKED_MANIFEST_SHA256 = "13030362d965114d6ddb9b41f8372cbb01d61c6214ec40048b3f2b3fd8d0d63d"

POLICY_B_TICK_PATH = WORKSPACE_DIR / "datasets" / "crypto_tick" / "frozen_policy_b_tick_champion.json"
POLICY_C_TICK_PATH = WORKSPACE_DIR / "datasets" / "crypto_tick" / "frozen_policy_c_tick_champion.json"

FIFTEEN_MINUTES_MS = 15 * 60 * 1000  # 900,000 ms
REGISTERED_WINDOW_START_UTC_MS = 1792972800000  # 2026-10-26T00:00:00.000Z
DEFAULT_REFERENCE_CAPITAL_USDT = 100_000.0
BASE_TAKER_FEE_RATE = 0.0005        # 5.0 bps
BASE_SLIPPAGE_RATE = 0.0001         # 1.0 bp

DEFAULT_OUTPUT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto"
DEFAULT_SINGLETON_LOCK_PATH = DEFAULT_OUTPUT_DIR / "rp608_crypto_shadow.singleton.lock"
DEFAULT_CHECKPOINT_PATH = DEFAULT_OUTPUT_DIR / "rp608_checkpoint.json"
# Kept alive for process lifetime so flock is held.
_CRYPTO_SHADOW_LOCK_HOLD: list = []
DEFAULT_TELEMETRY_LOG_PATH = DEFAULT_OUTPUT_DIR / "rp608_shadow_telemetry.jsonl"
DEFAULT_DECISION_CARDS_PATH = DEFAULT_OUTPUT_DIR / "rp608_decision_cards.jsonl"
DEFAULT_TRADE_JOURNAL_PATH = DEFAULT_OUTPUT_DIR / "rp608_trade_journal.jsonl"
DEFAULT_OPPORTUNITY_LOG_PATH = DEFAULT_OUTPUT_DIR / "rp608_opportunity_registry.jsonl"

# Observational multi-symbol twins (books never pooled; BTC remains live PRIMARY).
# SOL is allow-listed for later wiring — do not enable until ETH shadow is proven.
ALLOWED_CRYPTO_SHADOW_SYMBOLS = ("BTCUSDT", "ETHUSDT", "SOLUSDT")
DEFAULT_CRYPTO_SHADOW_SYMBOL = "BTCUSDT"

# Backwards-compatible aliases
LOG_DIR = DEFAULT_OUTPUT_DIR
CHECKPOINT_PATH = DEFAULT_CHECKPOINT_PATH
TELEMETRY_LOG_PATH = DEFAULT_TELEMETRY_LOG_PATH
DECISION_CARDS_PATH = DEFAULT_DECISION_CARDS_PATH
TRADE_JOURNAL_PATH = DEFAULT_TRADE_JOURNAL_PATH
OPPORTUNITY_LOG_PATH = DEFAULT_OPPORTUNITY_LOG_PATH


def normalize_crypto_shadow_symbol(symbol: Optional[str]) -> str:
    """Canonical Binance spot symbol for a crypto shadow instance."""
    s = str(symbol or DEFAULT_CRYPTO_SHADOW_SYMBOL).strip().upper()
    if s not in ALLOWED_CRYPTO_SHADOW_SYMBOLS:
        raise ValueError(
            f"FATAL: Unsupported crypto shadow symbol {s!r}. "
            f"Allowed: {', '.join(ALLOWED_CRYPTO_SHADOW_SYMBOLS)}"
        )
    return s


def default_output_dir_for_symbol(symbol: str) -> Path:
    """
    Per-symbol ledger root so books never share state.
    BTCUSDT → …/crypto (legacy path locked).
    ETHUSDT → …/crypto_eth ; SOLUSDT → …/crypto_sol.
    """
    sym = normalize_crypto_shadow_symbol(symbol)
    if sym == DEFAULT_CRYPTO_SHADOW_SYMBOL:
        return DEFAULT_OUTPUT_DIR
    base = sym[:-4].lower() if sym.endswith("USDT") else sym.lower()
    return WORKSPACE_DIR / "live_capture" / "shadow_validation" / f"crypto_{base}"


def binance_ws_stream_url(symbol: str, stream: str) -> str:
    """Binance combined single-stream WS URL for symbol@stream."""
    sym = normalize_crypto_shadow_symbol(symbol)
    st = str(stream).strip()
    if not st:
        raise ValueError("stream must be non-empty")
    return f"wss://stream.binance.com:9443/ws/{sym.lower()}@{st}"


def resolve_output_dir(output_dir: Optional[str | Path]) -> Path:
    """
    Resolves and validates an output directory for daemon state, ledgers, and telemetry.
    Rejects unsafe paths (root '/', workspace root, existing regular files, empty strings).
    """
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



def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def compute_payload_checksum(data: Dict[str, Any]) -> str:
    """Computes a deterministic SHA-256 hash of a dictionary excluding any existing checksum field."""
    d = {k: v for k, v in data.items() if k != "checksum"}
    canonical = json.dumps(d, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


# -------------------------------------------------------------------------
# 1. Causal Feature Extraction & Factor Mapping (Exact Rust Parity)
# -------------------------------------------------------------------------

@dataclass
class Candle15m:
    open_time_ms: i64
    close_time_ms: i64
    open: float
    high: float
    low: float
    close: float
    volume: float

    @classmethod
    def from_binance_ws(cls, k: Dict[str, Any]) -> Candle15m:
        return cls(
            open_time_ms=int(k["t"]),
            close_time_ms=int(k["T"]),
            open=float(k["o"]),
            high=float(k["h"]),
            low=float(k["l"]),
            close=float(k["c"]),
            volume=float(k["v"]),
        )

    @classmethod
    def from_dict(cls, d: Dict[str, Any]) -> Candle15m:
        return cls(
            open_time_ms=int(d["open_time_ms"]),
            close_time_ms=int(d["close_time_ms"]),
            open=float(d["open"]),
            high=float(d["high"]),
            low=float(d["low"]),
            close=float(d["close"]),
            volume=float(d["volume"]),
        )

    def typical_price(self) -> float:
        return (self.high + self.low + self.close) / 3.0


@dataclass
class CausalFeatureVector:
    bar_idx: int
    open_time_ms: int
    close_time_ms: int
    reference_price: float
    ret_15m_bps: float
    atr_14: float
    donchian_breakout_20: bool
    price_range_pos_20: Optional[float]
    rolling_vwap_96: Optional[float]
    dist_vwap_bps: Optional[float]
    vol_burst_20: Optional[float]


def compute_causal_features(candles: List[Candle15m], idx: int) -> Optional[CausalFeatureVector]:
    if idx >= len(candles):
        return None
    curr = candles[idx]
    p_ref = curr.close
    ret_15m_bps = ((curr.close - curr.open) / curr.open) * 10_000.0

    # ATR 14
    if idx >= 14:
        tr_sum = 0.0
        for i in range(idx - 13, idx + 1):
            if i == 0:
                tr = candles[i].high - candles[i].low
            else:
                prev_c = candles[i - 1].close
                h_l = candles[i].high - candles[i].low
                h_pc = abs(candles[i].high - prev_c)
                l_pc = abs(candles[i].low - prev_c)
                tr = max(h_l, h_pc, l_pc)
            tr_sum += tr
        atr_14 = tr_sum / 14.0
    else:
        atr_14 = curr.high - curr.low

    # Donchian Breakout 20 and Price Range Position
    if idx >= 20:
        max_h = max(candles[i].high for i in range(idx - 20, idx))
        min_l = min(candles[i].low for i in range(idx - 20, idx))
        breakout = curr.close > max_h
        rng = max_h - min_l
        pos = (curr.close - min_l) / rng if rng > 0.0 else 0.5
        donchian_breakout_20 = breakout
        price_range_pos_20 = pos
    else:
        donchian_breakout_20 = False
        price_range_pos_20 = None

    # Rolling VWAP 96 bars (24 hours)
    if idx >= 95:
        sum_pv = 0.0
        sum_v = 0.0
        for i in range(idx - 95, idx + 1):
            tp = candles[i].typical_price()
            v = candles[i].volume
            sum_pv += tp * v
            sum_v += v
        if sum_v > 0.0:
            vwap = sum_pv / sum_v
            dist = ((p_ref - vwap) / vwap) * 10_000.0
            rolling_vwap_96 = vwap
            dist_vwap_bps = dist
        else:
            rolling_vwap_96 = None
            dist_vwap_bps = None
    else:
        rolling_vwap_96 = None
        dist_vwap_bps = None

    # Volume Burst 20
    if idx >= 20:
        sum_v = sum(candles[i].volume for i in range(idx - 20, idx))
        sma_v = sum_v / 20.0
        vol_burst_20 = curr.volume / sma_v if sma_v > 0.0 else 1.0
    else:
        vol_burst_20 = None

    return CausalFeatureVector(
        bar_idx=idx,
        open_time_ms=curr.open_time_ms,
        close_time_ms=curr.close_time_ms,
        reference_price=p_ref,
        ret_15m_bps=ret_15m_bps,
        atr_14=atr_14,
        donchian_breakout_20=donchian_breakout_20,
        price_range_pos_20=price_range_pos_20,
        rolling_vwap_96=rolling_vwap_96,
        dist_vwap_bps=dist_vwap_bps,
        vol_burst_20=vol_burst_20,
    )


def features_to_evidence(feat: CausalFeatureVector, alphabet: str) -> List[Dict[str, Any]]:
    factors = []

    # Trend
    if feat.ret_15m_bps > 10.0:
        trend_dir = "Bullish"
    elif feat.ret_15m_bps < -10.0:
        trend_dir = "Bearish"
    else:
        trend_dir = "Neutral"
    factors.append({"concept": "Trend", "present": True, "direction": trend_dir})

    # Momentum
    if feat.ret_15m_bps > 2.0:
        mom_dir = "Positive"
    elif feat.ret_15m_bps < -2.0:
        mom_dir = "Negative"
    else:
        mom_dir = "Neutral"
    factors.append({"concept": "Momentum", "present": True, "direction": mom_dir})

    # Volatility
    vol_present = (feat.atr_14 / feat.reference_price) > 0.0035
    factors.append({"concept": "Volatility", "present": vol_present, "direction": None})

    if alphabet == "WithExhaustion":
        # PriceRangeStretch
        if feat.price_range_pos_20 is not None:
            if feat.price_range_pos_20 > 0.90:
                factors.append({"concept": "PriceRangeStretch", "present": True, "direction": "ExhaustedHigh"})
            elif feat.price_range_pos_20 < 0.10:
                factors.append({"concept": "PriceRangeStretch", "present": True, "direction": "ExhaustedLow"})
            else:
                factors.append({"concept": "PriceRangeStretch", "present": False, "direction": None})

        # VWAPStretch
        if feat.dist_vwap_bps is not None:
            if feat.dist_vwap_bps > 60.0:
                factors.append({"concept": "VWAPStretch", "present": True, "direction": "Overbought"})
            elif feat.dist_vwap_bps < -60.0:
                factors.append({"concept": "VWAPStretch", "present": True, "direction": "Oversold"})
            else:
                factors.append({"concept": "VWAPStretch", "present": False, "direction": None})

        # VolumeBurst
        if feat.vol_burst_20 is not None:
            burst_present = feat.vol_burst_20 > 1.8
            factors.append({
                "concept": "VolumeBurst",
                "present": burst_present,
                "direction": "Burst" if burst_present else None
            })

    return factors


def predicate_matches(pred: Dict[str, Any], factors: List[Dict[str, Any]]) -> bool:
    target_concept = pred["concept"]
    factor = next((f for f in factors if f["concept"] == target_concept), None)
    pred_present = pred.get("present")
    pred_dir = pred.get("direction")

    if pred_present is False and factor is None:
        return pred_dir is None
    if pred_present is False and factor is not None:
        return (not factor["present"]) and (pred_dir is None)
    if pred_present is True and factor is not None and factor["present"]:
        if pred_dir is None:
            return True
        return factor.get("direction") == pred_dir
    if pred_present is True and (factor is None or not factor["present"]):
        return False
    if pred_present is None and factor is not None:
        if pred_dir is None:
            return True
        return factor["present"] and (factor.get("direction") == pred_dir)
    if pred_present is None and factor is None:
        return pred_dir is None
    return False


def evaluate_policy_rules(genome: Dict[str, Any], factors: List[Dict[str, Any]]) -> str:
    rules = genome.get("rules", [])
    for rule in rules:
        when = rule.get("when", [])
        if all(predicate_matches(p, factors) for p in when):
            return rule.get("action", "NO_TRADE")
    return genome.get("unmatched_action", "NO_TRADE")


# -------------------------------------------------------------------------
# 2. Opportunity Accounting Registry & Telemetry Data Structures
# -------------------------------------------------------------------------

@dataclass
class UnexecutableDrop:
    opportunity_id: int
    bar_idx: int
    decision_time_ms: int
    intended_action: str
    drop_reason: str
    disposition: str
    details: Dict[str, Any] = field(default_factory=dict)


@dataclass
class OpportunityAccountingRegistry:
    total_signal_opportunities: int = 0
    executed_trades_count: int = 0
    unexecutable_drops_count: int = 0
    execution_drop_rate_pct: float = 0.0
    unexecutable_drops: List[Dict[str, Any]] = field(default_factory=list)

    def record_executed(self):
        self.total_signal_opportunities += 1
        self.executed_trades_count += 1
        self._recompute()

    def record_drop(self, drop: UnexecutableDrop):
        self.total_signal_opportunities += 1
        self.unexecutable_drops_count += 1
        self.unexecutable_drops.append(asdict(drop))
        self._recompute()

    def _recompute(self):
        if self.total_signal_opportunities > 0:
            self.execution_drop_rate_pct = (
                self.unexecutable_drops_count / self.total_signal_opportunities
            ) * 100.0


# -------------------------------------------------------------------------
# 3. Crash-Safe Durable Checkpoint Manager
# -------------------------------------------------------------------------

class DurableCheckpointManager:
    """Manages atomic, crash-resilient checkpoints on disk with schema and checksum validation."""

    def __init__(self, checkpoint_path: Path):
        self.checkpoint_path = checkpoint_path
        self.tmp_path = checkpoint_path.with_suffix(".tmp")
        self.checkpoint_path.parent.mkdir(parents=True, exist_ok=True)

    def load_checkpoint(self) -> Optional[Dict[str, Any]]:
        if not self.checkpoint_path.exists():
            return None
        with open(self.checkpoint_path, "r", encoding="utf-8") as f:
            data = json.load(f)

        # Integrity Validation: Verify Checksum
        recorded_checksum = data.get("checksum")
        computed_checksum = compute_payload_checksum(data)
        if recorded_checksum != computed_checksum:
            raise ValueError(
                f"FATAL: Checkpoint checksum mismatch at {self.checkpoint_path}! "
                f"Recorded: {recorded_checksum}, Computed: {computed_checksum}. "
                f"Fail-closed boundary activated."
            )

        # Schema Validation
        if data.get("schema_version") != "1.0.0" or data.get("window_id") != CHECKPOINT_WINDOW_ID:
            raise ValueError(
                f"FATAL: Incompatible checkpoint schema or window_id: {data.get('window_id')}. "
                f"Fail-closed boundary activated."
            )

        return data

    def save_checkpoint_atomic(self, state_dict: Dict[str, Any]):
        """Writes to temporary file, flushes to disk via fsync, and atomically renames."""
        state = copy.deepcopy(state_dict)
        state["checksum"] = compute_payload_checksum(state)

        with open(self.tmp_path, "w", encoding="utf-8") as f:
            json.dump(state, f, indent=2)
            f.flush()
            os.fsync(f.fileno())

        os.replace(self.tmp_path, self.checkpoint_path)


# -------------------------------------------------------------------------
# 4. Exchange Clock Synchronization Filter
# -------------------------------------------------------------------------

class ExchangeClockFilter:
    """Performs NTP minimum-RTT clock offset estimation against Binance REST /api/v3/time."""

    def __init__(self, max_allowed_offset_ms: int = 250):
        self.max_allowed_offset_ms = max_allowed_offset_ms
        self.last_offset_ms: Optional[int] = None
        self.last_rtt_ms: Optional[int] = None

    def sample_offset(self, samples: int = 5) -> Tuple[int, int]:
        ctx = ssl.create_default_context()
        conn = http.client.HTTPSConnection("api.binance.com", timeout=4.0, context=ctx)
        try:
            conn.request("GET", "/api/v3/time")
            res = conn.getresponse()
            res.read()
        except Exception:
            pass

        best_rtt = float("inf")
        best_offset = 0

        for _ in range(samples):
            t_send = int(time.time() * 1000)
            try:
                conn.request("GET", "/api/v3/time")
                res = conn.getresponse()
                raw = res.read()
                t_recv = int(time.time() * 1000)
                server_time = json.loads(raw)["serverTime"]
                rtt = max(t_recv - t_send, 1)
                midpoint = t_send + (rtt // 2)
                offset = midpoint - server_time

                if rtt < best_rtt:
                    best_rtt = rtt
                    best_offset = offset
            except Exception:
                continue
            time.sleep(0.05)
        conn.close()

        if best_rtt == float("inf"):
            raise ConnectionError("Failed to reach Binance REST time endpoint.")

        self.last_offset_ms = best_offset
        self.last_rtt_ms = int(best_rtt)
        return best_offset, int(best_rtt)

    def sample_offset_with_ebo(
        self,
        samples: int = 5,
        *,
        max_attempts: int = 12,
        base_delay_sec: float = 2.0,
        max_delay_sec: float = 60.0,
    ) -> Tuple[int, int]:
        """
        Exponential back-off (EBO) around sample_offset for reboot/DNS races.

        Deterministic delays: base * 2^(attempt-1), capped at max_delay_sec.
        No jitter (chronosentiment-core determinism). Does not relax the
        post-sync ±250 ms drift fail-closed gate.
        """
        if max_attempts < 1:
            raise ValueError("max_attempts must be >= 1")
        last_err: Optional[BaseException] = None
        for attempt in range(1, max_attempts + 1):
            try:
                return self.sample_offset(samples=samples)
            except ConnectionError as exc:
                last_err = exc
                if attempt >= max_attempts:
                    break
                delay = min(max_delay_sec, base_delay_sec * (2 ** (attempt - 1)))
                print(
                    f"[RP-608 Clock EBO] attempt {attempt}/{max_attempts} "
                    f"Binance time unreachable ({exc}); retry in {delay:.1f}s"
                )
                time.sleep(delay)
        raise ConnectionError(
            f"Failed to reach Binance REST time endpoint after {max_attempts} EBO attempts: {last_err}"
        )

    def is_synchronized(self) -> bool:
        if self.last_offset_ms is None:
            return False
        return abs(self.last_offset_ms) <= self.max_allowed_offset_ms


# -------------------------------------------------------------------------
# 5. Core RP-608 Shadow Observer Daemon
# -------------------------------------------------------------------------

class RP608ShadowDaemon:
    def __init__(
        self,
        output_dir: Optional[Path | str] = None,
        checkpoint_path: Optional[Path | str] = None,
        enable_dry_run: bool = False,
        single_candle_test: bool = False,
        window_start_utc_ms: Optional[int] = None,
        symbol: Optional[str] = None,
    ):
        self.symbol = normalize_crypto_shadow_symbol(symbol)
        if checkpoint_path is not None and output_dir is None:
            ckpt_p = Path(checkpoint_path)
            if not ckpt_p.is_absolute():
                ckpt_p = (WORKSPACE_DIR / ckpt_p).resolve()
            else:
                ckpt_p = ckpt_p.resolve()
            resolved_output_dir = ckpt_p.parent
            resolved_checkpoint_path = ckpt_p
        else:
            if output_dir is None:
                resolved_output_dir = resolve_output_dir(default_output_dir_for_symbol(self.symbol))
            else:
                resolved_output_dir = resolve_output_dir(output_dir)
            if checkpoint_path is not None:
                ckpt_p = Path(checkpoint_path)
                if not ckpt_p.is_absolute():
                    ckpt_p = (WORKSPACE_DIR / ckpt_p).resolve()
                else:
                    ckpt_p = ckpt_p.resolve()
                if ckpt_p.parent != resolved_output_dir:
                    raise ValueError(
                        f"FATAL: Unsafe path configuration: checkpoint_path {ckpt_p} is outside output_dir {resolved_output_dir}"
                    )
                resolved_checkpoint_path = ckpt_p
            else:
                resolved_checkpoint_path = resolved_output_dir / "rp608_checkpoint.json"

        self.output_dir = resolved_output_dir
        self.checkpoint_path = resolved_checkpoint_path
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.telemetry_log_path = self.output_dir / "rp608_shadow_telemetry.jsonl"
        self.decision_cards_path = self.output_dir / "rp608_decision_cards.jsonl"
        self.trade_journal_path = self.output_dir / "rp608_trade_journal.jsonl"
        self.opportunity_log_path = self.output_dir / "rp608_opportunity_registry.jsonl"

        self.checkpoint_manager = DurableCheckpointManager(self.checkpoint_path)
        self.clock_filter = ExchangeClockFilter()
        self.enable_dry_run = enable_dry_run
        self.single_candle_test = single_candle_test
        self.window_start_utc_ms = window_start_utc_ms if window_start_utc_ms is not None else REGISTERED_WINDOW_START_UTC_MS


        self.policy_b_art: Dict[str, Any] = {}
        self.policy_c_art: Dict[str, Any] = {}
        self.state: Dict[str, Any] = {}
        self.recent_candles: List[Candle15m] = []
        self.latest_book_ticker: Optional[Dict[str, Any]] = None
        # Forming 15m kline extremes for Gate 2E pre-entry path-invalid checks.
        self.latest_forming_kline: Optional[Dict[str, Any]] = None
        self.last_ticker_sequence_u: int = 0
        self.sequence_inversions: int = 0
        # Tick decision clock (RP-608-SHADOW-TICK-001)
        self.tick_feature_buffer = TickFeatureBuffer()
        self.tick_emission_gate = TickEmissionGate()
        # Track C v2: Python-fallback stabilizer (Rust loop has its own persistent state).
        self.tick_rec_stabilizer = RecommendationStabilizer()
        self._last_rec_hygiene: Optional[Dict[str, Any]] = None
        self._last_tick_features: Optional[Dict[str, Any]] = None
        # Trade-plan overlay: open on tick Policy B; TP/SL via continuous L1 (0.0% authority).
        # ATR geometry still frozen from completed 15m buffer at plan open.
        # Primary = volume-pattern axes (v2). Parallel twin = pre-volume baseline book
        # (OF paper inert on twin; books never pooled).
        # Gate 2E: BTC PRIMARY may use allow_only. Non-BTC: same env gate only when a
        # *symbol-local* library exists under output_dir — never reuse BTC cohorts.
        _primary_gate = os.environ.get("RP608_CRYPTO_PATTERN_GATE", "0")
        _baseline_gate = os.environ.get("RP608_CRYPTO_PATTERN_GATE_BASELINE", "0")
        _symbol_lib = self.output_dir / "rp608_crypto_plan_trade_pattern_library.json"
        if self.symbol != DEFAULT_CRYPTO_SHADOW_SYMBOL and not _symbol_lib.exists():
            _primary_gate = "0"
            _baseline_gate = "0"
            print(
                f"[trade_plan_overlay] {self.symbol}: Gate2E forced off — "
                f"no symbol library at {_symbol_lib} (books never pooled)"
            )
        elif self.symbol != DEFAULT_CRYPTO_SHADOW_SYMBOL:
            print(
                f"[trade_plan_overlay] {self.symbol}: Gate2E symbol library "
                f"{_symbol_lib} gate_primary={_primary_gate!r} "
                f"gate_baseline={_baseline_gate!r}"
            )
        _book_suffix = "" if self.symbol == DEFAULT_CRYPTO_SHADOW_SYMBOL else f"_{self.symbol}"
        _primary_book = f"CRYPTO_PLAN{_book_suffix}"
        _baseline_book = f"CRYPTO_PLAN_BASELINE_PARALLEL{_book_suffix}"
        # Always under this process output_dir (BTC default dir == legacy OVERLAY_DIR parent).
        _primary_dir = self.output_dir / "trade_plan_overlay_v1"
        self.trade_plan_overlay = TradePlanOverlayManager(
            base_dir=_primary_dir,
            book_id=_primary_book,
            symbol=self.symbol,
            of_paper_enabled=True,
            volume_pattern_axes=True,
            pattern_entry_gate_mode=_primary_gate,
            pattern_library_path=_symbol_lib
            if self.symbol != DEFAULT_CRYPTO_SHADOW_SYMBOL and _symbol_lib.exists()
            else None,
        )
        twin_dir = self.output_dir / "trade_plan_overlay_baseline_parallel_v1"
        self.trade_plan_overlay_baseline = TradePlanOverlayManager(
            base_dir=twin_dir,
            book_id=_baseline_book,
            symbol=self.symbol,
            of_paper_enabled=False,
            volume_pattern_axes=False,
            pattern_entry_gate_mode=_baseline_gate,
        )
        # Gate 2X r50 booking twin: same ungated entry as BASELINE, ratchet 0.50.
        # PRIMARY/BASELINE stay r30 until Gate 5. Books never pooled.
        _r50_book = f"CRYPTO_PLAN_GATE2X_R50_PARALLEL{_book_suffix}"
        _r50_frac = float(
            os.environ.get("RP608_CRYPTO_GATE2X_RATCHET_FRAC_R50", "0.50")
        )
        r50_dir = self.output_dir / "trade_plan_overlay_gate2x_r50_parallel_v1"
        self.trade_plan_overlay_g2x_r50 = TradePlanOverlayManager(
            base_dir=r50_dir,
            book_id=_r50_book,
            symbol=self.symbol,
            of_paper_enabled=False,
            volume_pattern_axes=False,
            pattern_entry_gate_mode=_baseline_gate,
            config_overrides={
                "gate2x_preset": "r50_booking",
                "gate2x_ratchet_peak_frac": _r50_frac,
                "gate2x_mfe_giveback_protocol": "RP-608-GATE2X-R50-BOOKING-TWIN-001",
            },
        )
        self._last_overlay_reload_ms: int = 0
        self._overlay_reload_interval_ms: int = 5000
        print(
            f"[trade_plan_overlay] symbol={self.symbol} parallel twin armed → "
            f"{twin_dir} (book={_baseline_book}; OF inert; no volume axes; "
            f"gate2e primary={_primary_gate!r} baseline={_baseline_gate!r})"
        )
        print(
            f"[gate2x_r50_twin] symbol={self.symbol} booking twin armed → "
            f"{r50_dir} (book={_r50_book}; ratchet={_r50_frac}; entry=baseline ungated; "
            f"PRIMARY/BASELINE r30 unchanged)",
            flush=True,
        )
        # Gate 2X floor-lock twin: same ungated entry; exit at protect floor
        # immediately (no COST_CLEAR HARD_FLOOR bleed). Observational; authority 0%.
        _fl_on = str(os.environ.get("RP608_CRYPTO_GATE2X_FLOOR_LOCK_TWIN", "1")).strip().lower() not in {
            "0",
            "false",
            "no",
            "off",
        }
        self.trade_plan_overlay_g2x_floorlock = None
        if _fl_on:
            _fl_book = f"CRYPTO_PLAN_GATE2X_FLOOR_LOCK_PARALLEL{_book_suffix}"
            _fl_frac = float(
                os.environ.get("RP608_CRYPTO_GATE2X_FLOOR_LOCK_RATCHET", "0.50")
            )
            fl_dir = self.output_dir / "trade_plan_overlay_gate2x_floorlock_parallel_v1"
            self.trade_plan_overlay_g2x_floorlock = TradePlanOverlayManager(
                base_dir=fl_dir,
                book_id=_fl_book,
                symbol=self.symbol,
                of_paper_enabled=False,
                volume_pattern_axes=False,
                pattern_entry_gate_mode=_baseline_gate,
                config_overrides={
                    "gate2x_preset": "floor_lock_r50",
                    "gate2x_ratchet_peak_frac": _fl_frac,
                    "gate2x_floor_lock_immediate": True,
                    # H11: disk+ops must match Rust floor_lock_twin_defaults (not PRIMARY 120s).
                    "gate2x_min_hold_s_before_exit": 0.0,
                    "gate2x_mfe_giveback_protocol": "RP-608-GATE2X-FLOOR-LOCK-TWIN-001",
                    "gate2x_floor_lock_protocol": "RP-608-GATE2X-FLOOR-LOCK-TWIN-001",
                },
            )
            print(
                f"[gate2x_floorlock_twin] symbol={self.symbol} floor-lock twin armed → "
                f"{fl_dir} (book={_fl_book}; ratchet={_fl_frac}; "
                f"immediate protect floor; PRIMARY exits unchanged)",
                flush=True,
            )

        # Clean-slate 15m adapter-select twin (D3: BTC+ETH only; default off until Anuj arms).
        # Independent of Gate 2E B trend-agree soak. Coralys / Gate 5 floors untouched.
        self.trade_plan_overlay_adapter_select_15m = None
        _as_on = adapter_select_15m_twin_enabled(default=False)
        _as_symbol_ok = self.symbol in ("BTCUSDT", "ETHUSDT")
        if _as_on and _as_symbol_ok:
            _as_book = f"CRYPTO_PLAN_ADAPTER_SELECT_15M{_book_suffix}"
            as_dir = self.output_dir / "trade_plan_overlay_adapter_select_15m_v1"
            self.trade_plan_overlay_adapter_select_15m = TradePlanOverlayManager(
                base_dir=as_dir,
                book_id=_as_book,
                symbol=self.symbol,
                of_paper_enabled=False,
                volume_pattern_axes=False,
                pattern_entry_gate_mode="0",
                config_overrides={
                    "require_adapter_select_15m": True,
                    "plan_require_of": False,
                    "b_persist_min_fires": 0,
                },
            )
            print(
                f"[adapter_select_15m_twin] symbol={self.symbol} observational twin armed → "
                f"{as_dir} (book={_as_book}; D1=a |S|≥0.35; PRIMARY/Gate2E-B untouched)",
                flush=True,
            )
        elif _as_on and not _as_symbol_ok:
            print(
                f"[adapter_select_15m_twin] symbol={self.symbol} skipped "
                f"(D3 BTC+ETH only; NSE/other out of scope for slice 1)",
                flush=True,
            )

        # Gate 2E B: 15m OHLCV L3/L8/L20 trend-agreement twin (PRIMARY unchanged).
        _ta_on = str(
            os.environ.get("RP608_GATE2E_TREND_AGREE_TWIN", "1")
        ).strip().lower() not in {"0", "false", "no", "off"}
        self.trade_plan_overlay_trend_agree = None
        if _ta_on:
            _ta_book = f"CRYPTO_PLAN_TREND_AGREE_PARALLEL{_book_suffix}"
            ta_dir = self.output_dir / "trade_plan_overlay_trend_agree_parallel_v1"
            self.trade_plan_overlay_trend_agree = TradePlanOverlayManager(
                base_dir=ta_dir,
                book_id=_ta_book,
                symbol=self.symbol,
                of_paper_enabled=False,
                volume_pattern_axes=False,
                pattern_entry_gate_mode="0",
                config_overrides={
                    "require_trend_agree": True,
                    "plan_require_of": False,
                    "b_persist_min_fires": 0,
                },
            )
            print(
                f"[gate2e_trend_agree_twin] symbol={self.symbol} B twin armed → "
                f"{ta_dir} (book={_ta_book}; L3/L8/L20 agree; PRIMARY Gate2E stack unchanged)",
                flush=True,
            )

        # OFX research-only: durable Binance aggTrade journal (separate WS; baseline OF inert)
        self.ofx_aggtrade_dir = self.output_dir / "ofx_aggtrade_capture_v1"
        self.ofx_aggtrade_journal = OfxCryptoAggTradeJournal(
            self.ofx_aggtrade_dir,
            enabled=ofx_crypto_aggtrade_capture_enabled(default=True),
            symbol=self.symbol,
            flush_every=32,
        )
        self._ofx_volume_mark_last_ms: int = 0
        self._ofx_volume_mark_interval_ms: int = 1000  # research snapshot throttle
        self.ofx_volume_paper = OfxVolumePaperManager(self.ofx_aggtrade_dir)

        self.running = False
        self.shutdown_event: Optional[asyncio.Event] = None
        self.opportunity_registry = OpportunityAccountingRegistry()

    def request_shutdown(self):
        """Signals the daemon to stop processing and wake the main loop promptly."""
        self.running = False
        if self.shutdown_event is not None and not self.shutdown_event.is_set():
            self.shutdown_event.set()

    def _crypto_recommendation_disposition(
        self,
        *,
        action_b: str,
        plan_result: Optional[Dict[str, Any]],
        now_ms: int,
    ) -> Dict[str, Any]:
        """
        Map raw Policy B to product recommendation disposition.

        policy_b_action stays observational. Actionable=True only when PRIMARY
        Gate2E actually opens (or would open) that side — not when pattern /
        regime-align / path / cooldown standaside.
        """
        d = str(action_b or "").upper()
        if d not in ("LONG", "SHORT"):
            return {
                "actionable": False,
                "disposition": "NO_TRADE",
                "reason": None,
            }
        # Fresh PRIMARY open for this side within a few seconds ⇒ actionable.
        if (
            isinstance(plan_result, dict)
            and plan_result.get("state") == "OPEN"
            and str(plan_result.get("direction") or "").upper() == d
        ):
            try:
                entry_ms = int(plan_result.get("entry_time_ms") or 0)
            except (TypeError, ValueError):
                entry_ms = 0
            if entry_ms <= 0 or abs(int(now_ms) - entry_ms) <= 8_000:
                return {
                    "actionable": True,
                    "disposition": "ACTIONABLE",
                    "reason": None,
                }

        st = (self.trade_plan_overlay.state or {}) if self.trade_plan_overlay else {}
        # Prefer most recent suppress stamp matching this side.
        ra = st.get("last_suppressed_regime_align")
        if isinstance(ra, dict) and str(ra.get("direction") or "").upper() == d:
            try:
                sms = int(ra.get("suppressed_ms") or 0)
            except (TypeError, ValueError):
                sms = 0
            if sms <= 0 or abs(int(now_ms) - sms) <= 15_000:
                guard = str(ra.get("guard") or "")
                if guard == "DI_SKEW" or "DI_SKEW" in str(ra.get("block_reason") or ""):
                    return {
                        "actionable": False,
                        "disposition": "STANDASIDE_DI_SKEW",
                        "reason": ra.get("block_reason") or "ADVERSE_DI_SKEW",
                    }
                return {
                    "actionable": False,
                    "disposition": "STANDASIDE_REGIME_ALIGN",
                    "reason": ra.get("block_reason") or "ADVERSE_REGIME",
                }
        pat = st.get("last_suppressed_trade_pattern")
        if isinstance(pat, dict) and str(pat.get("direction") or "").upper() == d:
            try:
                sms = int(pat.get("suppressed_ms") or 0)
            except (TypeError, ValueError):
                sms = 0
            if sms <= 0 or abs(int(now_ms) - sms) <= 15_000:
                pred = str(pat.get("prediction") or "STANDASIDE_TRADE_PATTERN")
                return {
                    "actionable": False,
                    "disposition": "STANDASIDE_TRADE_PATTERN",
                    "reason": pred,
                }
        # Path-invalid / cooldown / open-plan concurrency: still not actionable.
        return {
            "actionable": False,
            "disposition": "STANDASIDE_GATE2E",
            "reason": "PRIMARY_DID_NOT_OPEN",
        }

    def preflight_integrity_check(self):
        """Verifies tick policy artifacts, protocols, and order authority before any socket connects."""
        artifacts = [
            ("Policy B Tick Champion", POLICY_B_TICK_PATH, LOCKED_POLICY_B_TICK_SHA256),
            ("Policy C Tick Champion", POLICY_C_TICK_PATH, LOCKED_POLICY_C_TICK_SHA256),
            ("DFP Protocol", WORKSPACE_DIR / "docs" / "research" / "rp608_dfp_001_directional_feasibility_pilot_protocol.md", LOCKED_DFP_PROTOCOL_SHA256),
            ("DFP Amendment 01", WORKSPACE_DIR / "docs" / "research" / "rp608_dfp_001_amendment_01.md", LOCKED_AMENDMENT_SHA256),
            ("Activation Manifest", WORKSPACE_DIR / "docs" / "research" / "rp608_formal_activation_manifest.md", LOCKED_MANIFEST_SHA256),
        ]

        for name, path, expected in artifacts:
            if not path.exists():
                raise FileNotFoundError(f"Missing required artifact {name} at {path}")
            digest = sha256_file(path)
            if digest != expected:
                raise ValueError(
                    f"FATAL: Artifact checksum mismatch for {name}! "
                    f"Computed: {digest}, Expected: {expected}. Fail-closed boundary activated."
                )

        # Order Authority Check: Confirm strictly 0 trading keys in environment
        forbidden_env_keys = ["BINANCE_API_SECRET", "BINANCE_SECRET_KEY", "BINANCE_SECRET"]
        for key in forbidden_env_keys:
            if os.environ.get(key):
                raise PermissionError(f"FATAL: Forbidden API secret {key} found in environment! Zero order authority violated.")

        with open(POLICY_B_TICK_PATH, encoding="utf-8") as f:
            self.policy_b_art = json.load(f)
        with open(POLICY_C_TICK_PATH, encoding="utf-8") as f:
            self.policy_c_art = json.load(f)
        if self.policy_b_art.get("protocol_id") != SHADOW_PROTOCOL_ID:
            raise ValueError(
                f"FATAL: Policy B tick champion protocol_id mismatch: "
                f"{self.policy_b_art.get('protocol_id')} != {SHADOW_PROTOCOL_ID}"
            )
        if self.policy_c_art.get("protocol_id") != SHADOW_PROTOCOL_ID:
            raise ValueError(
                f"FATAL: Policy C tick champion protocol_id mismatch: "
                f"{self.policy_c_art.get('protocol_id')} != {SHADOW_PROTOCOL_ID}"
            )

    def initialize_or_resume_state(self):
        """Restores state from checkpoint or initializes a clean technical pilot state."""
        checkpoint = self.checkpoint_manager.load_checkpoint()
        now_ms = int(time.time() * 1000)

        if checkpoint:
            self.state = checkpoint
            # Rehydrate opportunity registry
            reg_data = checkpoint.get("opportunity_registry", {})
            self.opportunity_registry = OpportunityAccountingRegistry(
                total_signal_opportunities=reg_data.get("total_signal_opportunities", 0),
                executed_trades_count=reg_data.get("executed_trades_count", 0),
                unexecutable_drops_count=reg_data.get("unexecutable_drops_count", 0),
                execution_drop_rate_pct=reg_data.get("execution_drop_rate_pct", 0.0),
                unexecutable_drops=reg_data.get("unexecutable_drops", []),
            )

            # Record Explicit Downtime Interval & Reconcile Missed Bars
            window_start_ms = self.state.get("window_start_utc_ms", self.window_start_utc_ms)
            last_heartbeat_ms = self.state.get("last_heartbeat_utc_ms", now_ms)
            last_accounted_bar_idx = self.state.get("last_accounted_bar_idx", self.state.get("last_durable_bar_idx", 0))

            # Downtime Accounting strictly respects the registered window start boundary:
            # Pre-start elapsed time before window_start_ms is pre-activation buffer time.
            # Missed intervals accrue only for elapsed time after window_start_ms.
            if now_ms >= window_start_ms:
                effective_downtime_start = max(last_heartbeat_ms, window_start_ms)
                downtime_ms = max(now_ms - effective_downtime_start, 0)
                missed_intervals = downtime_ms // FIFTEEN_MINUTES_MS
            else:
                effective_downtime_start = last_heartbeat_ms
                downtime_ms = 0
                missed_intervals = 0

            outage_id = f"OUTAGE_{effective_downtime_start}_{now_ms}"

            # Idempotency Guard: Avoid duplicate accounting if this outage was already processed
            existing_outage_ids = {o.get("outage_id") for o in self.state.get("outage_history", [])}
            if outage_id not in existing_outage_ids and missed_intervals > 0:
                outage_entry = {
                    "outage_id": outage_id,
                    "downtime_start_ms": effective_downtime_start,
                    "downtime_end_ms": now_ms,
                    "duration_s": round(downtime_ms / 1000.0, 2),
                    "missed_15m_intervals": missed_intervals,
                    "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
                }
                self.state.setdefault("outage_history", []).append(outage_entry)

                # Update Quality Gates on Outage
                gates = self.state.setdefault("telemetry_gates", {
                    "total_completed_bars": 0,
                    "quote_coverage_pct": 100.0,
                    "stale_quote_count": 0,
                    "max_drift_ms": 0,
                    "consecutive_missing_bars": 0,
                    "total_missing_bars": 0,
                    "reconnection_count_24h": 0,
                    "window_tainted_by_breach": False,
                })
                gates["total_missing_bars"] += missed_intervals
                if missed_intervals >= 2:
                    gates["window_tainted_by_breach"] = True
                    gates["consecutive_missing_bars"] = max(gates["consecutive_missing_bars"], missed_intervals)

                # Account for missed intervals in Opportunity Registry as drops (never faking trades)
                for m in range(missed_intervals):
                    missed_bar_idx = last_accounted_bar_idx + m + 1
                    drop = UnexecutableDrop(
                        opportunity_id=self.opportunity_registry.total_signal_opportunities + 1,
                        bar_idx=missed_bar_idx,
                        decision_time_ms=last_heartbeat_ms + (m * FIFTEEN_MINUTES_MS),
                        intended_action="UNKNOWN_DURING_DOWNTIME",
                        drop_reason="MissingBarFeedDrop",
                        disposition="UNEXECUTABLE_REBOOT_DOWNTIME_DROP",
                        details={"outage_id": outage_id, "downtime_ms": downtime_ms, "missed_interval_index": m + 1},
                    )
                    self.opportunity_registry.record_drop(drop)

                self.state["last_accounted_bar_idx"] = last_accounted_bar_idx + missed_intervals

            # Advance heartbeat to current resumption time and commit recovery transaction atomically
            self.state["last_heartbeat_utc"] = datetime.now(timezone.utc).isoformat()
            self.state["last_heartbeat_utc_ms"] = now_ms
            self.state["opportunity_registry"] = asdict(self.opportunity_registry)
            self.checkpoint_manager.save_checkpoint_atomic(self.state)

            print(
                f"[RP-608 Resumption] Resumed from durable checkpoint. "
                f"last_durable_bar_idx={self.state.get('last_durable_bar_idx')}, "
                f"last_accounted_bar_idx={self.state.get('last_accounted_bar_idx')}. "
                f"Recorded downtime: {round(downtime_ms / 1000.0, 2)}s ({missed_intervals} missed intervals)."
            )
        else:
            # Clean Technical Pilot Initialization
            self.state = {
                "schema_version": "1.0.0",
                "window_id": CHECKPOINT_WINDOW_ID,
                "protocol_id": SHADOW_PROTOCOL_ID,
                "decision_clock": TICK_DECISION_CLOCK,
                "status": "ACTIVE_PILOT",
                "created_at_utc": datetime.now(timezone.utc).isoformat(),
                "last_heartbeat_utc": datetime.now(timezone.utc).isoformat(),
                "last_heartbeat_utc_ms": now_ms,
                "last_durable_bar_idx": 0,
                "last_accounted_bar_idx": 0,
                "last_decision_event_idx": 0,
                "last_completed_candle_close_ms": 0,
                "policy_b_state": {
                    "open_trade": None,
                    "closed_trades_count": 0,
                    "cumulative_net_pnl_usdt": 0.0,
                    "current_equity_usdt": DEFAULT_REFERENCE_CAPITAL_USDT,
                },
                "policy_c_state": {
                    "open_trade": None,
                    "closed_trades_count": 0,
                    "cumulative_net_pnl_usdt": 0.0,
                    "current_equity_usdt": DEFAULT_REFERENCE_CAPITAL_USDT,
                },
                "opportunity_registry": asdict(self.opportunity_registry),
                "telemetry_gates": {
                    "total_completed_bars": 0,
                    "quote_coverage_pct": 100.0,
                    "stale_quote_count": 0,
                    "max_drift_ms": 0,
                    "consecutive_missing_bars": 0,
                    "total_missing_bars": 0,
                    "reconnection_count_24h": 0,
                    "window_tainted_by_breach": False,
                },
                "outage_history": [],
            }
            self.checkpoint_manager.save_checkpoint_atomic(self.state)
            print("[RP-608 Initialization] Created clean durable checkpoint under window RP-608-SHADOW-001B.")

    def update_heartbeat(self):
        now_ms = int(time.time() * 1000)
        self.state["last_heartbeat_utc"] = datetime.now(timezone.utc).isoformat()
        self.state["last_heartbeat_utc_ms"] = now_ms
        self.state["opportunity_registry"] = asdict(self.opportunity_registry)
        self.checkpoint_manager.save_checkpoint_atomic(self.state)

        telemetry_entry = {
            "timestamp_utc": self.state["last_heartbeat_utc"],
            "heartbeat_ms": now_ms,
            "last_durable_bar_idx": self.state.get("last_durable_bar_idx", 0),
            "last_accounted_bar_idx": self.state.get("last_accounted_bar_idx", 0),
            "telemetry_gates": self.state.get("telemetry_gates", {}),
            "opportunity_summary": {
                "total_signal_opportunities": self.opportunity_registry.total_signal_opportunities,
                "executed_trades_count": self.opportunity_registry.executed_trades_count,
                "unexecutable_drops_count": self.opportunity_registry.unexecutable_drops_count,
                "execution_drop_rate_pct": self.opportunity_registry.execution_drop_rate_pct,
            },
        }
        with open(self.telemetry_log_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(telemetry_entry) + "\n")

    def bootstrap_15m_buffer(self, limit: int = 120) -> int:
        """Warm completed 15m buffer from Binance REST so ATR(14) is available immediately."""
        url = (
            f"https://api.binance.com/api/v3/klines?symbol={self.symbol}"
            f"&interval=15m&limit={int(limit)}"
        )
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "rp608-shadow-tick/1.0"})
            with urllib.request.urlopen(req, timeout=8.0) as resp:
                rows = json.loads(resp.read().decode("utf-8"))
        except Exception as exc:
            print(f"[RP-608 Bootstrap] 15m REST warm failed: {exc}")
            return 0
        candles: List[Candle15m] = []
        now_ms = int(time.time() * 1000)
        for row in rows:
            open_ms = int(row[0])
            close_ms = int(row[6])
            # Skip still-forming bar (close in the future / incomplete).
            if close_ms >= now_ms:
                continue
            candles.append(
                Candle15m(
                    open_time_ms=open_ms,
                    close_time_ms=close_ms,
                    open=float(row[1]),
                    high=float(row[2]),
                    low=float(row[3]),
                    close=float(row[4]),
                    volume=float(row[5]),
                )
            )
        self.recent_candles = candles[-120:]
        print(f"[RP-608 Bootstrap] Warmed {len(self.recent_candles)} completed 15m bars for ATR geometry.")
        return len(self.recent_candles)

    def _atr14_from_bar_buffer(self) -> Optional[float]:
        if not self.recent_candles:
            return None
        feat = compute_causal_features(self.recent_candles, len(self.recent_candles) - 1)
        return None if feat is None else float(feat.atr_14)

    def _vol_burst_20(
        self, *, forming_volume: Optional[float] = None
    ) -> Optional[float]:
        """
        Candle OHLCV volume / SMA20 of prior completed 15m bars.
        Used by plan pattern Gate 2E/2X only — not OFX aggression surge.
        """
        if len(self.recent_candles) < 20:
            return None
        prior = self.recent_candles[-20:]
        sma = sum(float(c.volume) for c in prior) / 20.0
        if sma <= 0:
            return None
        if forming_volume is not None:
            return float(forming_volume) / sma
        # Last completed bar vs prior 20 ending at that bar
        feat = compute_causal_features(self.recent_candles, len(self.recent_candles) - 1)
        return None if feat is None else feat.vol_burst_20

    def _completed_closes(self) -> List[float]:
        return [c.close for c in self.recent_candles]

    async def ingest_ofx_aggtrade(self):
        """
        OFX research capture: Binance aggTrade → durable journal.
        Separate socket from bookTicker. Never calls classify_order_flow_support /
        overlay activation. Failures stay on this task only.
        """
        journal = getattr(self, "ofx_aggtrade_journal", None)
        if journal is None or not journal.enabled:
            return
        ws_url = getattr(journal, "ws_url", None) or binance_ws_stream_url(
            self.symbol, "aggTrade"
        )
        while self.running:
            try:
                async with _websockets().connect(ws_url, ping_interval=20, ping_timeout=10) as ws:
                    async for msg in ws:
                        if not self.running:
                            break
                        try:
                            recv_wall_ms = int(time.time() * 1000)
                            data = json.loads(msg)
                            journal.on_raw_event(data, receipt_utc_ms=recv_wall_ms)
                            # Volume/direction OFX mark (parallel to baseline L1 OF; never mutates it)
                            if recv_wall_ms - self._ofx_volume_mark_last_ms >= self._ofx_volume_mark_interval_ms:
                                self._ofx_volume_mark_last_ms = recv_wall_ms
                                try:
                                    snap = refresh_ofx_volume_flow_mark(
                                        self.ofx_aggtrade_dir,
                                        symbol=self.symbol,
                                        now_ms=recv_wall_ms,
                                    )
                                    # Parallel OFX volume paper exit eval (never touches L1 OF paper)
                                    self.ofx_volume_paper.evaluate_tick(
                                        volume_snapshot=snap,
                                        quote=self.latest_book_ticker
                                        if isinstance(self.latest_book_ticker, dict)
                                        else None,
                                        now_ms=recv_wall_ms,
                                    )
                                except Exception:
                                    pass
                        except Exception:
                            # Research path fail-closed — never disrupt decision clock
                            continue
            except asyncio.CancelledError:
                raise
            except Exception as exc:
                print(f"[OFX aggTrade] reconnect after error: {exc}", flush=True)
                await asyncio.sleep(2.0)

    async def ingest_book_ticker(self):
        """bookTicker path: L1 maintenance, overlay exits, and tick Policy B/C decisions."""
        ws_url = binance_ws_stream_url(self.symbol, "bookTicker")
        while self.running:
            try:
                async with _websockets().connect(ws_url, ping_interval=20, ping_timeout=10) as ws:
                    async for msg in ws:
                        if not self.running:
                            break
                        recv_wall_ms = int(time.time() * 1000)
                        recv_mono_ns = time.monotonic_ns()
                        data = json.loads(msg)

                        update_id = int(data["u"])
                        if update_id < self.last_ticker_sequence_u:
                            self.sequence_inversions += 1
                        prev_u = self.last_ticker_sequence_u
                        self.last_ticker_sequence_u = update_id

                        b_price = float(data["b"])
                        a_price = float(data["a"])
                        mid = (b_price + a_price) / 2.0
                        spread_bps = ((a_price - b_price) / mid) * 10_000.0 if mid > 0 else 0.0
                        event_ms = int(data.get("E", recv_wall_ms))

                        self.latest_book_ticker = {
                            "exchange_event_time_ms": event_ms,
                            "local_wallclock_utc_ms": recv_wall_ms,
                            "local_receipt_mono_ns": recv_mono_ns,
                            "sequence_update_id": update_id,
                            "best_bid_price": b_price,
                            "best_bid_qty": float(data["B"]),
                            "best_ask_price": a_price,
                            "best_ask_qty": float(data["A"]),
                            "spread_bps": spread_bps,
                            "quote_source": "binance_ws_bookTicker",
                        }

                        quote = TickQuote(
                            update_id=update_id,
                            event_time_ms=event_ms,
                            receive_time_ms=recv_wall_ms,
                            bid_price=b_price,
                            bid_qty=float(data["B"]),
                            ask_price=a_price,
                            ask_qty=float(data["A"]),
                        )
                        try:
                            self.evaluate_on_tick(quote, prev_update_id=prev_u if prev_u else None)
                        except Exception as tick_exc:
                            print(f"[tick_policy] evaluate_on_tick skipped: {tick_exc}")

                        # Continuous overlay TP/SL on every bookTicker while OPEN.
                        # OF recommendation exit is reverse-flow L1 — independent of plan open/60m.
                        try:
                            if recv_wall_ms - self._last_overlay_reload_ms >= self._overlay_reload_interval_ms:
                                self._last_overlay_reload_ms = recv_wall_ms
                                # Preserve in-memory OPEN marks/MFE (disk may lag hold ticks).
                                self.trade_plan_overlay.reload_state_from_disk(
                                    preserve_open_progress=True
                                )
                                self.trade_plan_overlay_baseline.reload_state_from_disk(
                                    preserve_open_progress=True
                                )
                                self.trade_plan_overlay_g2x_r50.reload_state_from_disk(
                                    preserve_open_progress=True
                                )
                                if self.trade_plan_overlay_g2x_floorlock is not None:
                                    self.trade_plan_overlay_g2x_floorlock.reload_state_from_disk(
                                        preserve_open_progress=True
                                    )
                                if self.trade_plan_overlay_adapter_select_15m is not None:
                                    self.trade_plan_overlay_adapter_select_15m.reload_state_from_disk(
                                        preserve_open_progress=True
                                    )
                                if self.trade_plan_overlay_trend_agree is not None:
                                    self.trade_plan_overlay_trend_agree.reload_state_from_disk(
                                        preserve_open_progress=True
                                    )
                            _tick_overlays = [
                                (self.trade_plan_overlay, "trade_plan_overlay"),
                                (
                                    self.trade_plan_overlay_baseline,
                                    "trade_plan_overlay_baseline",
                                ),
                                (
                                    self.trade_plan_overlay_g2x_r50,
                                    "trade_plan_overlay_g2x_r50",
                                ),
                            ]
                            if self.trade_plan_overlay_g2x_floorlock is not None:
                                _tick_overlays.append(
                                    (
                                        self.trade_plan_overlay_g2x_floorlock,
                                        "trade_plan_overlay_g2x_floorlock",
                                    )
                                )
                            if self.trade_plan_overlay_adapter_select_15m is not None:
                                _tick_overlays.append(
                                    (
                                        self.trade_plan_overlay_adapter_select_15m,
                                        "trade_plan_overlay_adapter_select_15m",
                                    )
                                )
                            if self.trade_plan_overlay_trend_agree is not None:
                                _tick_overlays.append(
                                    (
                                        self.trade_plan_overlay_trend_agree,
                                        "trade_plan_overlay_trend_agree",
                                    )
                                )
                            for overlay, tag in _tick_overlays:
                                active = (overlay.state or {}).get("active_plan")
                                if active and active.get("state") == "OPEN":
                                    resolved = overlay.evaluate_market_tick(
                                        self.latest_book_ticker,
                                        reload_state=False,
                                        now_ms=recv_wall_ms,
                                    )
                                    if resolved and resolved.get("state") != "OPEN":
                                        print(
                                            f"[{tag}] {resolved.get('plan_id')} → {resolved.get('state')} "
                                            f"book={resolved.get('book_id')} "
                                            f"reason={resolved.get('exit_reason')} "
                                            f"exit={resolved.get('exit_price')} "
                                            f"net={resolved.get('net_pnl_usdt')} "
                                            f"(continuous L1; not 15m-gated)"
                                        )
                                        if overlay is self.trade_plan_overlay:
                                            try:
                                                prospective_v1_append_trade_closed(
                                                    market=ProspectiveMarket.CRYPTO,
                                                    outcome=resolved,
                                                )
                                            except Exception as link_exc:
                                                print(
                                                    f"[prospective_v1] trade_closed link skipped: {link_exc}"
                                                )
                            of_active = (self.trade_plan_overlay.state or {}).get(
                                "active_order_flow_recommendation"
                            )
                            # Only primary book owns L1 OF paper.
                            if of_active and of_active.get("active"):
                                active_primary = (self.trade_plan_overlay.state or {}).get(
                                    "active_plan"
                                )
                                if not (
                                    active_primary and active_primary.get("state") == "OPEN"
                                ):
                                    prev_of = True
                                    of_now = self.trade_plan_overlay.evaluate_order_flow_tick(
                                        self.latest_book_ticker,
                                        now_ms=recv_wall_ms,
                                        persist=True,
                                        candle_ctx={
                                            "bar_idx": self.state.get("last_durable_bar_idx"),
                                            "close_time_ms": int(
                                                (self.latest_book_ticker or {}).get(
                                                    "exchange_event_time_ms"
                                                )
                                                or event_ms
                                                or recv_wall_ms
                                            ),
                                        },
                                    )
                                    if prev_of and of_now is None:
                                        print(
                                            "[order_flow] ORDER_FLOW_EXIT "
                                            "(opposite-side L1 confirm; not 60m-gated)"
                                        )
                            else:
                                # Entry persist continuation: arm from decision-card OF,
                                # complete open on sustained same-side L1 (not ±0.30 change).
                                opened = (
                                    self.trade_plan_overlay.continue_of_paper_entry_persist(
                                        quote=self.latest_book_ticker,
                                        now_ms=recv_wall_ms,
                                        persist=True,
                                    )
                                )
                                if (
                                    isinstance(opened, dict)
                                    and opened.get("active")
                                    and opened.get("of_trade_id")
                                ):
                                    print(
                                        "[order_flow] OF_PAPER_OPEN "
                                        f"dir={opened.get('direction')} "
                                        f"id={opened.get('of_trade_id')} "
                                        "(entry persist cleared)"
                                    )
                        except Exception as overlay_exc:
                            print(f"[trade_plan_overlay] tick skipped: {overlay_exc}")
            except asyncio.CancelledError:
                break
            except Exception as e:
                if self.running:
                    self.state["telemetry_gates"]["reconnection_count_24h"] += 1
                    await asyncio.sleep(2.0)

    def evaluate_on_tick(
        self,
        quote: TickQuote,
        *,
        prev_update_id: Optional[int] = None,
    ) -> Optional[Dict[str, Any]]:
        """Canonical tick decision path: features → B/C → refractory emit → overlay open."""
        ok, reason = validate_tick_quote(quote, prev_update_id=prev_update_id)
        if not ok:
            if reason == "STALE_QUOTE":
                self.state.setdefault("telemetry_gates", {})
                self.state["telemetry_gates"]["stale_quote_count"] = (
                    int(self.state["telemetry_gates"].get("stale_quote_count", 0)) + 1
                )
            return None

        self.tick_feature_buffer.append(quote)
        if self.tick_feature_buffer.history_sec() < MIN_HISTORY_WARMUP_SEC:
            return None

        closes = self._completed_closes()
        genome_b = self.policy_b_art["champion_genome"]
        genome_c = self.policy_c_art["champion_genome"]
        mode = resolve_tick_eval_mode()
        action_b: str
        action_c: str
        tick_authority = "python_crypto_tick_policy"

        self._last_rec_hygiene = None
        if mode in {"rust", "compare_rust_primary"}:
            rust_out = rust_eval_tick(
                quote=quote,
                buffer_quotes=self.tick_feature_buffer.quotes(),
                closes=closes,
                policy_b_genome=genome_b,
                policy_c_genome=genome_c,
            )
            if rust_out is not None:
                action_b, action_c, feats, hygiene = rust_out
                self._last_tick_features = feats
                hygiene = hygiene if isinstance(hygiene, dict) else {}
                rust_hygiene_live = (
                    str(hygiene.get("protocol_id") or "") == REC_HYGIENE_PROTOCOL_ID
                )
                # Stale Pi bins predate adapter hygiene — apply Python overhaul.
                if not rust_hygiene_live:
                    snap = build_tick_snapshot(quote, self.tick_feature_buffer, closes)
                    self._last_tick_features = snapshot_to_dict(snap)
                    action_b, action_c, _fb, _fc, hygiene = evaluate_tick_policies_hygiened(
                        snap, genome_b, genome_c, self.tick_rec_stabilizer
                    )
                    tick_authority = "python_rec_hygiene_bridge"
                    # #region agent log
                    try:
                        _rust_b = str(rust_out[0])
                        _interesting = (
                            _rust_b != action_b
                            or bool(hygiene.get("microconfirm_gated"))
                            or bool(hygiene.get("confirm_pending"))
                            or bool(hygiene.get("sticky_held"))
                            or bool(hygiene.get("rearm_blocked"))
                            or bool(hygiene.get("gap_tolerated"))
                            or action_b in ("LONG", "SHORT")
                            or _rust_b in ("LONG", "SHORT")
                        )
                        if _interesting:
                            import json as _json_dbg
                            import time as _time_dbg
                            _dbg_path = os.environ.get(
                                "RP608_DEBUG_LOG",
                                "/Users/nikhil/ChronoSentiment_MEGA_FINAL/.cursor/debug-17c9cb.log",
                            )
                            with open(_dbg_path, "a", encoding="utf-8") as _dbg_f:
                                _dbg_f.write(
                                    _json_dbg.dumps(
                                        {
                                            "sessionId": "17c9cb",
                                            "runId": "hygiene_v3",
                                            "hypothesisId": "H9",
                                            "location": "run_rp608_shadow_daemon.py:tick_bridge",
                                            "message": "python_hygiene_bridge_v3",
                                            "data": {
                                                "symbol": self.symbol,
                                                "update_id": int(quote.update_id),
                                                "rust_raw_b": _rust_b,
                                                "rust_raw_c": str(rust_out[1]),
                                                "out_b": action_b,
                                                "out_c": action_c,
                                                "hygiene_rev": hygiene.get(
                                                    "hygiene_rev"
                                                ),
                                                "microconfirm_gated": bool(
                                                    hygiene.get("microconfirm_gated")
                                                ),
                                                "confirm_pending": bool(
                                                    hygiene.get("confirm_pending")
                                                ),
                                                "confirm_streak": int(
                                                    hygiene.get("confirm_streak") or 0
                                                ),
                                                "confirm_span_ms": int(
                                                    hygiene.get("confirm_span_ms") or 0
                                                ),
                                                "gap_tolerated": bool(
                                                    hygiene.get("gap_tolerated")
                                                ),
                                                "rearm_blocked": bool(
                                                    hygiene.get("rearm_blocked")
                                                ),
                                                "sticky_held": bool(
                                                    hygiene.get("sticky_held")
                                                ),
                                                "protocol_id": hygiene.get(
                                                    "protocol_id"
                                                ),
                                            },
                                            "timestamp": int(_time_dbg.time() * 1000),
                                        }
                                    )
                                    + "\n"
                                )
                    except Exception:
                        pass
                    # #endregion
                else:
                    self._last_rec_hygiene = hygiene
                    tick_authority = "rust_crypto_tick_policy"
                    # #region agent log
                    try:
                        import json as _json_dbg
                        import time as _time_dbg
                        _dbg_path = os.environ.get(
                            "RP608_DEBUG_LOG",
                            "/Users/nikhil/ChronoSentiment_MEGA_FINAL/.cursor/debug-17c9cb.log",
                        )
                        if action_b in ("LONG", "SHORT") or hygiene.get(
                            "microconfirm_gated"
                        ) or hygiene.get("confirm_pending") or hygiene.get("sticky_held"):
                            with open(_dbg_path, "a", encoding="utf-8") as _dbg_f:
                                _dbg_f.write(
                                    _json_dbg.dumps(
                                        {
                                            "sessionId": "17c9cb",
                                            "runId": "rust_hygiene",
                                            "hypothesisId": "H2",
                                            "location": "run_rp608_shadow_daemon.py:rust_hygiene",
                                            "message": "rust_rec_hygiene_live",
                                            "data": {
                                                "symbol": self.symbol,
                                                "update_id": int(quote.update_id),
                                                "raw_b": hygiene.get("raw_b"),
                                                "out_b": action_b,
                                                "out_c": action_c,
                                                "microconfirm_gated": bool(
                                                    hygiene.get("microconfirm_gated")
                                                ),
                                                "confirm_pending": bool(
                                                    hygiene.get("confirm_pending")
                                                ),
                                                "confirm_streak": int(
                                                    hygiene.get("confirm_streak") or 0
                                                ),
                                                "sticky_held": bool(
                                                    hygiene.get("sticky_held")
                                                ),
                                            },
                                            "timestamp": int(_time_dbg.time() * 1000),
                                        }
                                    )
                                    + "\n"
                                )
                    except Exception:
                        pass
                    # #endregion
                self._last_rec_hygiene = hygiene
                if mode == "compare_rust_primary":
                    # Compare raw genome eval only (hygiene is stateful per process).
                    snap_py = build_tick_snapshot(quote, self.tick_feature_buffer, closes)
                    ab_py, ac_py, _, _ = evaluate_tick_policies(
                        snap_py, genome_b, genome_c
                    )
                    raw_b = str((hygiene or {}).get("raw_b") or action_b)
                    raw_c = str((hygiene or {}).get("raw_c") or action_c)
                    if ab_py != raw_b or ac_py != raw_c:
                        print(
                            f"[crypto_tick_policy][compare] DIVERGE update_id={quote.update_id} "
                            f"rust_raw=({raw_b},{raw_c}) python=({ab_py},{ac_py})",
                            flush=True,
                        )
            else:
                log_python_fallback_once("rust_eval_failed_or_bin_down")
                snap = build_tick_snapshot(quote, self.tick_feature_buffer, closes)
                self._last_tick_features = snapshot_to_dict(snap)
                action_b, action_c, _fb, _fc, hygiene = evaluate_tick_policies_hygiened(
                    snap, genome_b, genome_c, self.tick_rec_stabilizer
                )
                self._last_rec_hygiene = hygiene
        else:
            if mode.startswith("python_fallback"):
                log_python_fallback_once(mode)
            snap = build_tick_snapshot(quote, self.tick_feature_buffer, closes)
            self._last_tick_features = snapshot_to_dict(snap)
            action_b, action_c, _fb, _fc, hygiene = evaluate_tick_policies_hygiened(
                snap, genome_b, genome_c, self.tick_rec_stabilizer
            )
            self._last_rec_hygiene = hygiene

        emit_ok, emit_reason = self.tick_emission_gate.should_emit(
            quote.receive_time_ms, action_b, action_c
        )
        if not emit_ok:
            return None

        atr_14 = self._atr14_from_bar_buffer()
        if atr_14 is None or atr_14 <= 0:
            # Overlay needs ATR geometry; suppress opens until 15m buffer is warm.
            if action_b in ("LONG", "SHORT") or action_c in ("LONG", "SHORT"):
                return None

        bar_idx = int(self.state.get("last_durable_bar_idx", 0))
        decision_event_idx = int(self.state.get("last_decision_event_idx", 0)) + 1
        self.state["last_decision_event_idx"] = decision_event_idx
        self.state["protocol_id"] = SHADOW_PROTOCOL_ID
        self.state["decision_clock"] = TICK_DECISION_CLOCK

        valid_execution_quote = True
        drop_reason = None
        candidate_quote = self.latest_book_ticker
        vol_burst_20 = self._vol_burst_20()

        if action_b in ("LONG", "SHORT") or action_c in ("LONG", "SHORT"):
            self.opportunity_registry.record_executed()

        self.tick_emission_gate.mark_emitted(quote.receive_time_ms, action_b, action_c)

        # Order-flow confirm stamp (Policy B side); exit is opposite-side OF later
        of_side = action_b if action_b in ("LONG", "SHORT") else None
        of_mark = {
            "order_flow_supported": False,
            "order_flow_label": None,
            "order_flow_imbalance": None,
            "order_flow_source": "none",
        }
        if of_side:
            of_mark = classify_order_flow_support(
                of_side, l1_from_book_ticker(candidate_quote if isinstance(candidate_quote, dict) else None)
            )
            if not of_mark.get("order_flow_supported"):
                of_mark = classify_order_flow_support(
                    of_side, l1_from_tick_features(self._last_tick_features)
                )

        feats = self._last_tick_features or {}
        mid_px = float(feats.get("mid_price") or quote.mid_price)
        macro_16 = feats.get("macro_trend_16_bps")
        quote_age_ms = float(feats.get("quote_age_ms") or (quote.receive_time_ms - quote.event_time_ms))

        decision_card: Dict[str, Any] = {
            "protocol_id": SHADOW_PROTOCOL_ID,
            "decision_clock": TICK_DECISION_CLOCK,
            "decision_event_idx": decision_event_idx,
            "bar_idx": bar_idx,
            "symbol": self.symbol,
            "timestamp_utc": datetime.fromtimestamp(
                quote.event_time_ms / 1000.0, tz=timezone.utc
            ).isoformat(),
            "close_time_ms": quote.event_time_ms,
            "quote_event_id": quote.update_id,
            "quote_receipt_utc_ms": quote.receive_time_ms,
            "reference_price": mid_px,
            "ret_15m_bps": float(macro_16) if macro_16 is not None else 0.0,
            "atr_14": atr_14 if atr_14 is not None else 0.0,
            "vol_burst_20": vol_burst_20,
            "features_tick": self._last_tick_features,
            "policy_b_action": action_b,
            "policy_c_action": action_c,
            "tick_policy_authority": tick_authority,
            "valid_execution_quote": valid_execution_quote,
            "drop_reason": drop_reason,
            "quote_age_ms": quote_age_ms,
            "execution_quote": candidate_quote,
            "arm_a_frozen": ARM_A_FROZEN,
            "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
            "emit_reason": emit_reason,
            "order_flow_supported": bool(of_mark.get("order_flow_supported")),
            "order_flow_label": of_mark.get("order_flow_label"),
            "order_flow_imbalance": of_mark.get("order_flow_imbalance"),
            "order_flow_source": of_mark.get("order_flow_source"),
            "rec_hygiene": self._last_rec_hygiene,
            # Completed 15m closes for clean-slate adapter_select_15m (decision-time only).
            "ohlcv_closes_15m": list(self._completed_closes()),
            "completed_closes": list(self._completed_closes()),
        }
        if REGIME_TELEMETRY_ENABLED and self.recent_candles:
            regime = classify_market_regime(self.recent_candles)
            decision_card = attach_regime_telemetry(decision_card, regime, enabled=True)
        else:
            decision_card = attach_regime_telemetry(decision_card, None, enabled=False)

        self.update_heartbeat()

        plan_result = None
        q = candidate_quote if valid_execution_quote else None
        # Attach forming 15m high/low so Gate 2E can refuse opens already through stop.
        if isinstance(q, dict) and isinstance(self.latest_forming_kline, dict):
            q = dict(q)
            for k in ("high_price", "low_price", "vol_burst_20"):
                if self.latest_forming_kline.get(k) is not None and q.get(k) is None:
                    q[k] = self.latest_forming_kline.get(k)
            q.setdefault(
                "forming_kline_source",
                self.latest_forming_kline.get("quote_source") or "binance_ws_forming_kline_15m",
            )
        try:
            plan_result = self.trade_plan_overlay.process_decision_card(
                decision_card,
                current_quote=q,
            )
        except Exception as plan_exc:
            print(f"[trade_plan_overlay] process_decision_card skipped: {plan_exc}")
        try:
            self.trade_plan_overlay_baseline.process_decision_card(
                dict(decision_card),
                current_quote=q,
            )
        except Exception as twin_exc:
            print(f"[trade_plan_overlay_baseline] process_decision_card skipped: {twin_exc}")
        try:
            self.trade_plan_overlay_g2x_r50.process_decision_card(
                dict(decision_card),
                current_quote=q,
            )
        except Exception as r50_exc:
            print(f"[trade_plan_overlay_g2x_r50] process_decision_card skipped: {r50_exc}")
        if self.trade_plan_overlay_g2x_floorlock is not None:
            try:
                self.trade_plan_overlay_g2x_floorlock.process_decision_card(
                    dict(decision_card),
                    current_quote=q,
                )
            except Exception as fl_exc:
                print(
                    f"[trade_plan_overlay_g2x_floorlock] process_decision_card skipped: {fl_exc}"
                )
        if self.trade_plan_overlay_adapter_select_15m is not None:
            try:
                self.trade_plan_overlay_adapter_select_15m.process_decision_card(
                    dict(decision_card),
                    current_quote=q,
                )
            except Exception as as_exc:
                print(
                    f"[trade_plan_overlay_adapter_select_15m] process_decision_card skipped: {as_exc}"
                )
        if self.trade_plan_overlay_trend_agree is not None:
            try:
                self.trade_plan_overlay_trend_agree.process_decision_card(
                    dict(decision_card),
                    current_quote=q,
                )
            except Exception as ta_exc:
                print(
                    f"[trade_plan_overlay_trend_agree] process_decision_card skipped: {ta_exc}"
                )

        # Product surface: raw Policy B stays on card, but "actionable recommendation"
        # must match PRIMARY Gate2E (pattern / regime-align / path / cooldown).
        disp = self._crypto_recommendation_disposition(
            action_b=str(action_b),
            plan_result=plan_result if isinstance(plan_result, dict) else None,
            now_ms=int(quote.receive_time_ms),
        )
        decision_card["recommendation_actionable"] = bool(disp.get("actionable"))
        decision_card["recommendation_disposition"] = disp.get("disposition")
        decision_card["gate2e_block_reason"] = disp.get("reason")
        with open(self.decision_cards_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(decision_card) + "\n")

        # Parallel OFX volume PAPER book (research-only; baseline L1 OF paper untouched)
        if action_b in ("LONG", "SHORT"):
            try:
                vol_path = self.ofx_aggtrade_dir / "ofx_volume_flow_snapshot.json"
                vol_snap: Optional[Dict[str, Any]] = None
                if vol_path.exists():
                    loaded = json.loads(vol_path.read_text(encoding="utf-8"))
                    if isinstance(loaded, dict):
                        vol_snap = loaded
                if vol_snap is None:
                    vol_snap = refresh_ofx_volume_flow_mark(
                        self.ofx_aggtrade_dir, symbol=self.symbol, now_ms=quote.receive_time_ms
                    )
                plan_id = None
                if isinstance(plan_result, dict):
                    plan_id = (plan_result.get("plan") or {}).get("plan_id") or plan_result.get(
                        "plan_id"
                    )
                ofx_align = evaluate_regime_align(
                    direction=str(action_b),
                    regime=regime_dict_from_card(
                        decision_card if isinstance(decision_card, dict) else None
                    ),
                    book_id=getattr(self.trade_plan_overlay, "book_id", None),
                )
                # OFX follows PRIMARY disposition: no paper open when Gate2E standaside.
                if ofx_align.get("blocks_entry") or not disp.get("actionable"):
                    print(
                        f"[ofx_volume_paper] standaside "
                        f"regime={ofx_align.get('block_reason')} "
                        f"disp={disp.get('disposition')} "
                        f"(OFX authority 0%; baseline OF inert)",
                        flush=True,
                    )
                else:
                    self.ofx_volume_paper.try_open_from_decision(
                        direction=action_b,
                        volume_snapshot=vol_snap,
                        quote=candidate_quote if valid_execution_quote else None,
                        plan_id=plan_id,
                        bar_idx=bar_idx,
                        decision_event_idx=decision_event_idx,
                        entry_price=float(mid_px),
                        now_ms=int(quote.receive_time_ms),
                    )
            except Exception as ofx_paper_exc:
                print(f"[ofx_volume_paper] try_open skipped: {ofx_paper_exc}")

        try:
            ask_v = bid_v = age_v = None
            if candidate_quote:
                ask_v = float(candidate_quote["best_ask_price"])
                bid_v = float(candidate_quote["best_bid_price"])
                age_v = float(quote_age_ms)
            session_date_utc = datetime.fromtimestamp(
                quote.event_time_ms / 1000.0, tz=timezone.utc
            ).strftime("%Y-%m-%d")
            regime_label = (
                decision_card.get("regime_state") if isinstance(decision_card, dict) else None
            )
            rec_actionable = bool(disp.get("actionable"))
            prospective_v1_record_recommendation(
                market=ProspectiveMarket.CRYPTO,
                symbol=self.symbol,
                decision=str(action_b),
                secondary_decision=str(action_c),
                session_date=session_date_utc,
                bar_idx=decision_event_idx,
                close_time_ms=int(quote.event_time_ms),
                reference_price=float(mid_px),
                is_actionable=rec_actionable,
                regime=regime_label,
                extra={
                    "policy_b_action": action_b,
                    "policy_c_action": action_c,
                    "decision_clock": TICK_DECISION_CLOCK,
                    "quote_event_id": quote.update_id,
                    "atr_14": atr_14,
                    "features_tick": self._last_tick_features,
                    "tick_policy_authority": tick_authority,
                    "recommendation_disposition": disp.get("disposition"),
                    "gate2e_block_reason": disp.get("reason"),
                    "recommendation_actionable": rec_actionable,
                },
            )
            prospective_v1_append_rec_trade(
                market=ProspectiveMarket.CRYPTO,
                symbol=self.symbol,
                session_date=session_date_utc,
                bar_idx=decision_event_idx,
                decision=str(action_b),
                secondary_decision=str(action_c),
                close_time_ms=int(quote.event_time_ms),
                reference_price=float(mid_px),
                is_actionable=rec_actionable,
                regime=regime_label,
                plan=plan_result if isinstance(plan_result, dict) else None,
                extra={
                    "policy_b_action": action_b,
                    "policy_c_action": action_c,
                    "decision_clock": TICK_DECISION_CLOCK,
                    "atr_14": atr_14,
                    "tick_policy_authority": tick_authority,
                    "recommendation_disposition": disp.get("disposition"),
                    "gate2e_block_reason": disp.get("reason"),
                },
            )
            if self.recent_candles:
                # Arm A observe follows actionable disposition (not raw Policy B alone).
                arm_a = bool(rec_actionable)
                prospective_v1_observe_bar(
                    market=ProspectiveMarket.CRYPTO,
                    completed_bars_including_signal=self.recent_candles,
                    ask=ask_v,
                    bid=bid_v,
                    quote_age_ms=age_v,
                    session_date=session_date_utc,
                    symbol=self.symbol,
                    bar_idx=decision_event_idx,
                    close_time_ms=int(quote.event_time_ms),
                    arm_a_signal=arm_a,
                    arm_a_side=action_b if arm_a else None,
                    reference_price=float(mid_px),
                    extra={
                        "policy_b_action": action_b,
                        "policy_c_action": action_c,
                        "decision_clock": TICK_DECISION_CLOCK,
                        "tick_policy_authority": tick_authority,
                        "recommendation_disposition": disp.get("disposition"),
                    },
                )
        except Exception as obs_exc:
            print(f"[prospective_v1] CRYPTO observe skipped: {obs_exc}")

        imb = feats.get("imbalance")
        v5 = feats.get("velocity_5s_bps")
        print(
            f"[{datetime.now(timezone.utc).strftime('%H:%M:%S')}] Tick evt {decision_event_idx:06d}: "
            f"mid={mid_px:.2f} | B:{action_b} C:{action_c} | "
            f"imb={imb} v5={v5} atr14={atr_14} auth={tick_authority}"
        )
        if self.single_candle_test and action_b in ("LONG", "SHORT"):
            self.request_shutdown()
        return decision_card

    async def evaluate_completed_candle(self, candle: Candle15m):
        """Ingest completed 15m bar for ATR / macro context only (no Policy B/C decision)."""
        self.recent_candles.append(candle)
        if len(self.recent_candles) > 120:
            self.recent_candles = self.recent_candles[-120:]

        bar_idx = self.state.get("last_durable_bar_idx", 0) + 1
        self.state["last_durable_bar_idx"] = bar_idx
        self.state["last_accounted_bar_idx"] = max(self.state.get("last_accounted_bar_idx", 0) + 1, bar_idx)
        self.state["last_completed_candle_close_ms"] = candle.close_time_ms
        self.state["telemetry_gates"]["total_completed_bars"] += 1
        self.state["protocol_id"] = SHADOW_PROTOCOL_ID
        self.state["decision_clock"] = TICK_DECISION_CLOCK
        self.update_heartbeat()

        atr = self._atr14_from_bar_buffer()
        print(
            f"[{datetime.now(timezone.utc).strftime('%H:%M:%S')}] Bar {bar_idx:04d} Closed (ATR context): "
            f"P_ref={candle.close:.2f} | atr_14={atr} | decision_clock={TICK_DECISION_CLOCK}"
        )
        if self.single_candle_test:
            # Tick path owns shutdown on actionable; bar close alone no longer exits.
            pass

    async def ingest_klines(self):
        """Subscribes to live Binance {symbol}@kline_15m stream."""
        ws_url = binance_ws_stream_url(self.symbol, "kline_15m")
        while self.running:
            try:
                async with _websockets().connect(ws_url, ping_interval=20, ping_timeout=10) as ws:
                    async for msg in ws:
                        if not self.running:
                            break
                        data = json.loads(msg)
                        k = data.get("k", {})
                        is_candle_closed = k.get("x", False)

                        # Forming-bar high/low path-through for open trade plans
                        # (covers TP touch that reverses before the 15m close).
                        if not is_candle_closed:
                            try:
                                if self.latest_book_ticker:
                                    forming_q = dict(self.latest_book_ticker)
                                    forming_q["low_price"] = float(k["l"])
                                    forming_q["high_price"] = float(k["h"])
                                    forming_q["quote_source"] = "binance_ws_forming_kline_15m"
                                    try:
                                        fv = float(k.get("v")) if k.get("v") is not None else None
                                    except (TypeError, ValueError):
                                        fv = None
                                    burst = self._vol_burst_20(forming_volume=fv)
                                    if burst is not None:
                                        forming_q["vol_burst_20"] = burst
                                    self.latest_forming_kline = {
                                        "high_price": forming_q.get("high_price"),
                                        "low_price": forming_q.get("low_price"),
                                        "vol_burst_20": forming_q.get("vol_burst_20"),
                                        "quote_source": forming_q.get("quote_source"),
                                        "kline_start_ms": k.get("t"),
                                    }
                                    now_ms = int(time.time() * 1000)
                                    _kline_overlays = [
                                        (self.trade_plan_overlay, "trade_plan_overlay"),
                                        (
                                            self.trade_plan_overlay_baseline,
                                            "trade_plan_overlay_baseline",
                                        ),
                                        (
                                            self.trade_plan_overlay_g2x_r50,
                                            "trade_plan_overlay_g2x_r50",
                                        ),
                                    ]
                                    if self.trade_plan_overlay_g2x_floorlock is not None:
                                        _kline_overlays.append(
                                            (
                                                self.trade_plan_overlay_g2x_floorlock,
                                                "trade_plan_overlay_g2x_floorlock",
                                            )
                                        )
                                    if self.trade_plan_overlay_adapter_select_15m is not None:
                                        _kline_overlays.append(
                                            (
                                                self.trade_plan_overlay_adapter_select_15m,
                                                "trade_plan_overlay_adapter_select_15m",
                                            )
                                        )
                                    if self.trade_plan_overlay_trend_agree is not None:
                                        _kline_overlays.append(
                                            (
                                                self.trade_plan_overlay_trend_agree,
                                                "trade_plan_overlay_trend_agree",
                                            )
                                        )
                                    for overlay, tag in _kline_overlays:
                                        active = (overlay.state or {}).get("active_plan")
                                        if not (active and active.get("state") == "OPEN"):
                                            continue
                                        resolved = overlay.evaluate_market_tick(
                                            forming_q,
                                            reload_state=False,
                                            now_ms=now_ms,
                                        )
                                        if resolved and resolved.get("state") != "OPEN":
                                            print(
                                                f"[{tag}] forming-kline path-through "
                                                f"{resolved.get('plan_id')} → {resolved.get('state')} "
                                                f"book={resolved.get('book_id')} "
                                                f"reason={resolved.get('exit_reason')}"
                                            )
                                            if overlay is self.trade_plan_overlay:
                                                try:
                                                    prospective_v1_append_trade_closed(
                                                        market=ProspectiveMarket.CRYPTO,
                                                        outcome=resolved,
                                                    )
                                                except Exception as link_exc:
                                                    print(
                                                        f"[prospective_v1] trade_closed link skipped: {link_exc}"
                                                    )
                            except Exception as form_exc:
                                print(f"[trade_plan_overlay] forming-kline skipped: {form_exc}")

                        if is_candle_closed:
                            candle = Candle15m.from_binance_ws(k)
                            await self.evaluate_completed_candle(candle)
            except asyncio.CancelledError:
                break
            except Exception as e:
                if self.running:
                    self.state["telemetry_gates"]["reconnection_count_24h"] += 1
                    await asyncio.sleep(2.0)

    async def run(self):
        """Launches the observer daemon event loop with graceful shutdown signal handling."""
        self.running = True
        if self.shutdown_event is None:
            self.shutdown_event = asyncio.Event()
        else:
            self.shutdown_event.clear()

        # Register OS signal handlers for graceful shutdown (SIGTERM / SIGINT)
        loop = asyncio.get_running_loop()
        def handle_signal(sig_num: int):
            sig_name = signal.Signals(sig_num).name
            print(f"\n[RP-608 Signal] Received {sig_name}. Initiating graceful shutdown...")
            self.request_shutdown()

        for sig in (signal.SIGTERM, signal.SIGINT):
            try:
                loop.add_signal_handler(sig, lambda s=sig: handle_signal(s))
            except NotImplementedError:
                signal.signal(sig, lambda s, f, num=sig: handle_signal(num))

        self.preflight_integrity_check()
        self.initialize_or_resume_state()
        self.bootstrap_15m_buffer(limit=120)
        print(
            f"[RP-608] symbol={self.symbol} Decision clock={TICK_DECISION_CLOCK} "
            f"protocol={SHADOW_PROTOCOL_ID} authority={LIVE_ORDER_AUTHORITY_PCT}% "
            f"(Gate C isolated; books never pooled)."
        )
        print(
            f"[RP-608] tick_policy backend={tick_policy_backend()} "
            f"mode={resolve_tick_eval_mode()} "
            f"(Track C: RP608_CRYPTO_TICK_POLICY=python|rust|compare; default python)",
            flush=True,
        )

        # Calibrate Exchange Clock Offset (EBO: survive post-reboot DNS/network lag)
        offset_ms, rtt_ms = self.clock_filter.sample_offset_with_ebo(
            samples=5,
            max_attempts=12,
            base_delay_sec=2.0,
            max_delay_sec=60.0,
        )
        self.state["telemetry_gates"]["max_drift_ms"] = max(
            self.state["telemetry_gates"]["max_drift_ms"], abs(offset_ms)
        )
        if not self.clock_filter.is_synchronized():
            raise ValueError(
                f"FATAL: Exchange clock drift ({offset_ms} ms) exceeds 250 ms threshold! "
                f"RTT: {rtt_ms} ms. Fail-closed boundary activated."
            )

        print(f"[RP-608 Clock Sync] Offset: {offset_ms} ms (RTT: {rtt_ms} ms). Status: PASS")

        # Concurrent Tasks: bookTicker + kline_15m + OFX aggTrade research capture
        if getattr(self, "ofx_aggtrade_journal", None) is not None and self.ofx_aggtrade_journal.enabled:
            print(
                f"[+] OFX crypto aggTrade durable capture armed → {self.ofx_aggtrade_dir} "
                "(RP608_OFX_CRYPTO_AGGTRADE_CAPTURE; baseline OF inert)",
                flush=True,
            )
        else:
            print("[*] OFX crypto aggTrade capture disabled (RP608_OFX_CRYPTO_AGGTRADE_CAPTURE)", flush=True)

        t_ticker = asyncio.create_task(self.ingest_book_ticker())
        t_klines = asyncio.create_task(self.ingest_klines())
        t_ofx_agg = asyncio.create_task(self.ingest_ofx_aggtrade())

        shutdown_persisted = False
        try:
            while self.running:
                try:
                    await asyncio.wait_for(self.shutdown_event.wait(), timeout=5.0)
                    # Woken promptly by shutdown_event
                    break
                except asyncio.TimeoutError:
                    # 5-second periodic heartbeat
                    self.update_heartbeat()
                    if self.single_candle_test and not self.running:
                        break
        finally:
            self.running = False
            # Cancel feed tasks
            t_ticker.cancel()
            t_klines.cancel()
            t_ofx_agg.cancel()

            # Await feed tasks to ensure clean socket closures
            results = await asyncio.gather(
                t_ticker, t_klines, t_ofx_agg, return_exceptions=True
            )
            for r in results:
                if isinstance(r, Exception) and not isinstance(r, asyncio.CancelledError):
                    print(f"[RP-608 Warning] Feed task returned error on cancellation: {r}")

            if getattr(self, "ofx_aggtrade_journal", None) is not None:
                try:
                    self.ofx_aggtrade_journal.close()
                except Exception as ofx_exc:
                    print(f"[OFX aggTrade] journal close warning: {ofx_exc}", flush=True)

            # Persist final heartbeat and atomic checkpoint
            print("[RP-608 Shutdown] Persisting final heartbeat and atomic checkpoint...")
            try:
                self.update_heartbeat()
                shutdown_persisted = True
                print("[RP-608 Shutdown] Graceful shutdown completed cleanly.")
            except Exception as e:
                print(f"[RP-608 FATAL] Failed to persist final checkpoint on shutdown: {e}", file=sys.stderr)
                raise

        if not shutdown_persisted:
            raise RuntimeError("FATAL: Daemon shutdown finalized without atomic checkpoint commitment!")



def main():
    parser = argparse.ArgumentParser(description="RP-608-SHADOW-TICK-001 Prospective Observation Daemon")
    parser.add_argument(
        "--symbol",
        type=str,
        default=DEFAULT_CRYPTO_SHADOW_SYMBOL,
        help="Binance spot symbol (BTCUSDT default; ETHUSDT observational twin; SOLUSDT deferred).",
    )
    parser.add_argument("--output-dir", type=str, default=None, help="Explicit output directory for isolated checkpoint and ledgers.")
    parser.add_argument("--dry-run", action="store_true", help="Run preflight and exit without opening sockets.")
    parser.add_argument("--test-resumption", action="store_true", help="Test recovery and downtime accounting from existing checkpoint.")
    parser.add_argument("--single-candle-test", action="store_true", help="Run until first actionable tick Policy B emit and exit.")
    parser.add_argument(
        "--allow-multiple",
        action="store_true",
        help="Skip singleton flock (tests only). Live Pi must not use this.",
    )
    args = parser.parse_args()
    symbol = normalize_crypto_shadow_symbol(args.symbol)

    daemon = RP608ShadowDaemon(
        output_dir=args.output_dir,
        single_candle_test=args.single_candle_test,
        symbol=symbol,
    )

    if args.dry_run:
        print("[Dry Run] Verifying artifact hashes and preflight checks...")
        daemon.preflight_integrity_check()
        daemon.initialize_or_resume_state()
        daemon.bootstrap_15m_buffer(limit=30)
        offset, rtt = daemon.clock_filter.sample_offset(samples=3)
        print(f"[Dry Run] Clock Offset: {offset} ms (RTT: {rtt} ms). PASS.")
        print(
            f"[Dry Run] symbol={daemon.symbol} protocol={SHADOW_PROTOCOL_ID} "
            f"decision_clock={TICK_DECISION_CLOCK}."
        )
        print("[Dry Run] Preflight verification succeeded cleanly.")
        return

    if args.test_resumption:
        print("[Test Resumption] Exercising crash-safe recovery logic...")
        daemon.preflight_integrity_check()
        daemon.initialize_or_resume_state()
        print("[Test Resumption] Resumption logic executed cleanly.")
        return

    # Live path: one process per output-dir (BTC and ETH twins may run in parallel).
    if not args.allow_multiple:
        out = Path(args.output_dir) if args.output_dir else default_output_dir_for_symbol(symbol)
        lock_path = out / "rp608_crypto_shadow.singleton.lock"
        acquire_or_exit(
            lock_path,
            label=f"RP-608-SHADOW-TICK-001/{symbol}",
            hold=_CRYPTO_SHADOW_LOCK_HOLD,
        )
        print(
            f"[RP-608] Singleton lock acquired → {lock_path} "
            f"(symbol={symbol} pid={os.getpid()})",
            flush=True,
        )

    asyncio.run(daemon.run())


if __name__ == "__main__":
    main()

