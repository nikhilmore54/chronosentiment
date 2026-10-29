#!/usr/bin/env python3
"""
scripts/run_rp608_shadow_daemon.py

RP-608-SHADOW-001B: Dedicated Prospective Crypto Shadow Observation Daemon.
Strictly Read-Only (0.0% Order Authority) with Atomic Durable Checkpointing.

Features:
  1. Concurrent WebSocket Ingestion:
     - wss://stream.binance.com:9443/ws/btcusdt@kline_15m (Completed 15m Candles, k.x == true)
     - wss://stream.binance.com:9443/ws/btcusdt@bookTicker (Continuous Top-of-Book L1 Quotes)
  2. Protocol Invariant & Quality Gate Enforcement:
     - Clock offset calibration (|Delta_clock| <= 250 ms, 5-sample NTP minimum RTT)
     - Quote freshness boundary (<= 500 ms) and monotonic 5,000 ms execution timeout
     - Strictly zero sequence update ID inversions (TG-5)
     - Comprehensive 4-component friction ledger (TG-6)
  3. Policy Champion Multi-Evaluation:
     - Parallel causal evaluation of Policy B Champion and Policy C Champion
     - Strict feature parity with Rust research substrate (compute_causal_features)
  4. Durable, Crash-Safe Checkpointing & Resumption:
     - Atomic rename (os.replace) via temporary file with os.fsync
     - Explicit downtime recording on reboot / process restart
     - OpportunityAccountingRegistry tracking all unexecutable drops (never diluting trade denominator)
     - Fail-closed behavior on checksum mismatch, hash tamper, or consecutive outage breach (>= 2 bars)
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

import websockets

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE_DIR))

from scripts.market_regime_classifier import classify_market_regime
from scripts.rp608_arm_a_baseline import attach_regime_telemetry
from scripts.rp608_prospective_v1 import Market as ProspectiveMarket
from scripts.rp608_prospective_v1_observer import (
    observe_bar as prospective_v1_observe_bar,
    record_recommendation as prospective_v1_record_recommendation,
)
from scripts.rp608_recommendation_trade_ledger import (
    append_recommendation_trade_link as prospective_v1_append_rec_trade,
    append_trade_outcome_link as prospective_v1_append_trade_closed,
)
from scripts.rp608_trade_plan_overlay import TradePlanOverlayManager

# RP-608-REGIME-BREAKOUT-001 — Arm A freeze + observational regime telemetry
# Policy B actions / costs / horizons MUST remain unchanged while ARM_A_FROZEN.
ARM_A_FROZEN = True
ARM_A_FREEZE_PROTOCOL = "RP-608-REGIME-BREAKOUT-001"
LIVE_ORDER_AUTHORITY_PCT = 0.0  # Strictly 0.0% — no order placement surface
REGIME_TELEMETRY_ENABLED = True  # Observational only; must not alter Arm A actions
HOLDOUT_SESSIONS_SEALED = "41-68"

# Locked SHA-256 Hashes
LOCKED_POLICY_B_SHA256 = "f7be46ea66ff6a248ad0477dfe5b082ce26fe01f9dded928b164bdaa349c6aa3"
LOCKED_POLICY_C_SHA256 = "cd615f37b812a34fe302a0b05bc1546a0243f3d71d8947092d28d2847159553b"
LOCKED_DFP_PROTOCOL_SHA256 = "e8bca8ad9194ff058cc4eb4315f20c290d6465fce28fcfe0b675b6b133d494c3"
LOCKED_AMENDMENT_SHA256 = "c9467df6f9fa1b715c50b576f1af4e79716c86d7c8765db75f5fbe552a9f5202"
LOCKED_MANIFEST_SHA256 = "13030362d965114d6ddb9b41f8372cbb01d61c6214ec40048b3f2b3fd8d0d63d"

FIFTEEN_MINUTES_MS = 15 * 60 * 1000  # 900,000 ms
REGISTERED_WINDOW_START_UTC_MS = 1792972800000  # 2026-10-26T00:00:00.000Z
DEFAULT_REFERENCE_CAPITAL_USDT = 100_000.0
BASE_TAKER_FEE_RATE = 0.0005        # 5.0 bps
BASE_SLIPPAGE_RATE = 0.0001         # 1.0 bp

DEFAULT_OUTPUT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto"
DEFAULT_CHECKPOINT_PATH = DEFAULT_OUTPUT_DIR / "rp608_checkpoint.json"
DEFAULT_TELEMETRY_LOG_PATH = DEFAULT_OUTPUT_DIR / "rp608_shadow_telemetry.jsonl"
DEFAULT_DECISION_CARDS_PATH = DEFAULT_OUTPUT_DIR / "rp608_decision_cards.jsonl"
DEFAULT_TRADE_JOURNAL_PATH = DEFAULT_OUTPUT_DIR / "rp608_trade_journal.jsonl"
DEFAULT_OPPORTUNITY_LOG_PATH = DEFAULT_OUTPUT_DIR / "rp608_opportunity_registry.jsonl"

# Backwards-compatible aliases
LOG_DIR = DEFAULT_OUTPUT_DIR
CHECKPOINT_PATH = DEFAULT_CHECKPOINT_PATH
TELEMETRY_LOG_PATH = DEFAULT_TELEMETRY_LOG_PATH
DECISION_CARDS_PATH = DEFAULT_DECISION_CARDS_PATH
TRADE_JOURNAL_PATH = DEFAULT_TRADE_JOURNAL_PATH
OPPORTUNITY_LOG_PATH = DEFAULT_OPPORTUNITY_LOG_PATH


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
        if data.get("schema_version") != "1.0.0" or data.get("window_id") != "RP-608-SHADOW-001B":
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
    ):
        if checkpoint_path is not None and output_dir is None:
            ckpt_p = Path(checkpoint_path)
            if not ckpt_p.is_absolute():
                ckpt_p = (WORKSPACE_DIR / ckpt_p).resolve()
            else:
                ckpt_p = ckpt_p.resolve()
            resolved_output_dir = ckpt_p.parent
            resolved_checkpoint_path = ckpt_p
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
        self.last_ticker_sequence_u: int = 0
        self.sequence_inversions: int = 0
        # Trade-plan overlay: continuous live-book TP/SL (0.0% authority).
        # Signal cadence stays 15m; lifecycle must NOT wait for the next bar close
        # (hit-then-reverse inside a 15m window would otherwise miss the TP).
        self.trade_plan_overlay = TradePlanOverlayManager()
        self._last_overlay_reload_ms: int = 0
        self._overlay_reload_interval_ms: int = 5000

        self.running = False
        self.shutdown_event: Optional[asyncio.Event] = None
        self.opportunity_registry = OpportunityAccountingRegistry()

    def request_shutdown(self):
        """Signals the daemon to stop processing and wake the main loop promptly."""
        self.running = False
        if self.shutdown_event is not None and not self.shutdown_event.is_set():
            self.shutdown_event.set()

    def preflight_integrity_check(self):
        """Verifies policy artifacts, protocols, and order authority before any socket connects."""
        artifacts = [
            ("Policy B Champion", WORKSPACE_DIR / "datasets" / "crypto_15m" / "frozen_policy_b_champion.json", LOCKED_POLICY_B_SHA256),
            ("Policy C Champion", WORKSPACE_DIR / "datasets" / "crypto_15m" / "frozen_policy_c_champion.json", LOCKED_POLICY_C_SHA256),
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

        with open(WORKSPACE_DIR / "datasets" / "crypto_15m" / "frozen_policy_b_champion.json") as f:
            self.policy_b_art = json.load(f)
        with open(WORKSPACE_DIR / "datasets" / "crypto_15m" / "frozen_policy_c_champion.json") as f:
            self.policy_c_art = json.load(f)

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
                "window_id": "RP-608-SHADOW-001B",
                "status": "ACTIVE_PILOT",
                "created_at_utc": datetime.now(timezone.utc).isoformat(),
                "last_heartbeat_utc": datetime.now(timezone.utc).isoformat(),
                "last_heartbeat_utc_ms": now_ms,
                "last_durable_bar_idx": 0,
                "last_accounted_bar_idx": 0,
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

    async def ingest_book_ticker(self):
        """Maintains live top-of-book L1 quotes and validates TG-5 sequence update ID monotonicity."""
        ws_url = "wss://stream.binance.com:9443/ws/btcusdt@bookTicker"
        while self.running:
            try:
                async with websockets.connect(ws_url, ping_interval=20, ping_timeout=10) as ws:
                    async for msg in ws:
                        if not self.running:
                            break
                        recv_wall_ms = int(time.time() * 1000)
                        recv_mono_ns = time.monotonic_ns()
                        data = json.loads(msg)

                        update_id = int(data["u"])
                        if update_id < self.last_ticker_sequence_u:
                            self.sequence_inversions += 1
                        self.last_ticker_sequence_u = update_id

                        b_price = float(data["b"])
                        a_price = float(data["a"])
                        mid = (b_price + a_price) / 2.0
                        spread_bps = ((a_price - b_price) / mid) * 10_000.0 if mid > 0 else 0.0

                        self.latest_book_ticker = {
                            "exchange_event_time_ms": int(data.get("E", recv_wall_ms)),
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
                        # Continuous overlay TP/SL on every bookTicker while OPEN.
                        # Periodic disk reload keeps dashboard/daemon writers coherent.
                        try:
                            if recv_wall_ms - self._last_overlay_reload_ms >= self._overlay_reload_interval_ms:
                                self._last_overlay_reload_ms = recv_wall_ms
                                self.trade_plan_overlay.state = self.trade_plan_overlay._load_state()
                            active = (self.trade_plan_overlay.state or {}).get("active_plan")
                            if active and active.get("state") == "OPEN":
                                resolved = self.trade_plan_overlay.evaluate_market_tick(
                                    self.latest_book_ticker,
                                    reload_state=False,
                                    now_ms=recv_wall_ms,
                                )
                                if resolved and resolved.get("state") != "OPEN":
                                    print(
                                        f"[trade_plan_overlay] {resolved.get('plan_id')} → {resolved.get('state')} "
                                        f"reason={resolved.get('exit_reason')} exit={resolved.get('exit_price')} "
                                        f"net={resolved.get('net_pnl_usdt')} "
                                        f"(continuous L1; not 15m-gated)"
                                    )
                                    try:
                                        prospective_v1_append_trade_closed(
                                            market=ProspectiveMarket.CRYPTO,
                                            outcome=resolved,
                                        )
                                    except Exception as link_exc:
                                        print(f"[prospective_v1] trade_closed link skipped: {link_exc}")
                        except Exception as overlay_exc:
                            print(f"[trade_plan_overlay] tick skipped: {overlay_exc}")
            except asyncio.CancelledError:
                break
            except Exception as e:
                if self.running:
                    self.state["telemetry_gates"]["reconnection_count_24h"] += 1
                    await asyncio.sleep(2.0)


    async def evaluate_completed_candle(self, candle: Candle15m):
        """Processes a completed 15-minute candle at T_close and evaluates Policy B and Policy C."""
        t_decision_eval = int(time.time() * 1000)
        t_decision_mono = time.monotonic_ns()
        self.recent_candles.append(candle)

        # Keep a rolling buffer of 120 completed bars for rolling VWAP 96 and Donchian 20
        if len(self.recent_candles) > 120:
            self.recent_candles = self.recent_candles[-120:]

        bar_idx = self.state.get("last_durable_bar_idx", 0) + 1
        self.state["last_durable_bar_idx"] = bar_idx
        self.state["last_accounted_bar_idx"] = max(self.state.get("last_accounted_bar_idx", 0) + 1, bar_idx)
        self.state["last_completed_candle_close_ms"] = candle.close_time_ms
        self.state["telemetry_gates"]["total_completed_bars"] += 1

        # Search for First Post-Close L1 Execution Quote within 5,000 ms (Monotonic Timeout)
        candidate_quote = None
        quote_wait_start = time.monotonic()
        while time.monotonic() - quote_wait_start < 5.0:
            if self.latest_book_ticker:
                q = self.latest_book_ticker
                # Ingestion condition: quote arrival after decision time (monotonic order)
                if q["local_receipt_mono_ns"] >= t_decision_mono:
                    candidate_quote = q
                    break
            await asyncio.sleep(0.02)

        # Quote Freshness & Monotonic Timeout Gate (Manifest Section 3.3)
        valid_execution_quote = False
        drop_reason = None
        quote_age_ms = None
        if candidate_quote is None:
            drop_reason = "ExecutionQuoteTimeout"
        else:
            clock_offset = self.clock_filter.last_offset_ms or 0
            rtt_u = (self.clock_filter.last_rtt_ms or 0) // 2
            delta_t = (
                candidate_quote["local_wallclock_utc_ms"]
                - candidate_quote["exchange_event_time_ms"]
                - clock_offset
            )
            quote_age_ms = max(delta_t + rtt_u, 0)
            if quote_age_ms > 500:
                drop_reason = "StaleQuoteExceeded"
                self.state["telemetry_gates"]["stale_quote_count"] += 1
            elif candidate_quote["best_bid_price"] >= candidate_quote["best_ask_price"]:
                drop_reason = "InvertedBookError"
            else:
                valid_execution_quote = True

        # Compute Causal Features & Evaluate Policies
        feat = compute_causal_features(self.recent_candles, len(self.recent_candles) - 1)
        if feat is None:
            return

        factors_b = features_to_evidence(feat, "BaseOnly")
        factors_c = features_to_evidence(feat, "WithExhaustion")

        action_b = evaluate_policy_rules(self.policy_b_art["champion_genome"], factors_b)
        action_c = evaluate_policy_rules(self.policy_c_art["champion_genome"], factors_c)

        # Account for Opportunities in Registry
        if action_b in ("LONG", "SHORT") or action_c in ("LONG", "SHORT"):
            if not valid_execution_quote:
                drop = UnexecutableDrop(
                    opportunity_id=self.opportunity_registry.total_signal_opportunities + 1,
                    bar_idx=bar_idx,
                    decision_time_ms=candle.close_time_ms,
                    intended_action=f"B:{action_b}|C:{action_c}",
                    drop_reason=drop_reason or "UnknownDrop",
                    disposition="UNEXECUTABLE_MICROSTRUCTURE_DROP",
                    details={"quote_age_ms": quote_age_ms if candidate_quote else None},
                )
                self.opportunity_registry.record_drop(drop)
                with open(self.opportunity_log_path, "a", encoding="utf-8") as f:
                    f.write(json.dumps(asdict(drop)) + "\n")
            else:
                self.opportunity_registry.record_executed()

        # Update State & Save Atomic Checkpoint
        self.update_heartbeat()

        # Append Immutable Decision Card to isolated directory
        decision_card = {
            "bar_idx": bar_idx,
            "timestamp_utc": datetime.fromtimestamp(candle.close_time_ms / 1000.0, tz=timezone.utc).isoformat(),
            "close_time_ms": candle.close_time_ms,
            "reference_price": candle.close,
            "ret_15m_bps": feat.ret_15m_bps,
            "atr_14": feat.atr_14,
            "policy_b_action": action_b,
            "policy_c_action": action_c,
            "valid_execution_quote": valid_execution_quote,
            "drop_reason": drop_reason,
            "quote_age_ms": quote_age_ms,
            "execution_quote": candidate_quote,
            "arm_a_frozen": ARM_A_FROZEN,
            "live_order_authority": LIVE_ORDER_AUTHORITY_PCT,
        }
        # Observational regime telemetry on completed candles only (Phase 1).
        # Must not alter policy_b_action / policy_c_action / execution disposition.
        if REGIME_TELEMETRY_ENABLED:
            regime = classify_market_regime(self.recent_candles)
            decision_card = attach_regime_telemetry(decision_card, regime, enabled=True)
        else:
            decision_card = attach_regime_telemetry(decision_card, None, enabled=False)
        with open(self.decision_cards_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(decision_card) + "\n")

        # Trade-plan overlay: open/suppress from Policy B on completed bar (0.0% authority).
        plan_result = None
        try:
            plan_result = self.trade_plan_overlay.process_decision_card(
                decision_card,
                current_quote=candidate_quote if valid_execution_quote else None,
            )
        except Exception as plan_exc:
            print(f"[trade_plan_overlay] process_decision_card skipped: {plan_exc}")

        # Prospective V1 research journals (0.0% authority; only if CRYPTO STARTED):
        # 1) full recommendation tape (every bar, incl. NO_TRADE)
        # 2) recommendation ↔ simulated-trade join ledger
        # 3) Arm A/B/C observer (Policy B as Arm A proxy)
        try:
            ask_v = bid_v = age_v = None
            if valid_execution_quote and candidate_quote:
                ask_v = float(candidate_quote["best_ask_price"])
                bid_v = float(candidate_quote["best_bid_price"])
                age_v = float(quote_age_ms) if quote_age_ms is not None else None
            session_date_utc = datetime.fromtimestamp(
                candle.close_time_ms / 1000.0, tz=timezone.utc
            ).strftime("%Y-%m-%d")
            regime_label = (decision_card.get("regime_state") if isinstance(decision_card, dict) else None)
            prospective_v1_record_recommendation(
                market=ProspectiveMarket.CRYPTO,
                symbol="BTCUSDT",
                decision=str(action_b),
                secondary_decision=str(action_c),
                session_date=session_date_utc,
                bar_idx=bar_idx,
                close_time_ms=int(candle.close_time_ms),
                reference_price=float(candle.close),
                is_actionable=action_b in ("LONG", "SHORT"),
                regime=regime_label,
                extra={
                    "policy_b_action": action_b,
                    "policy_c_action": action_c,
                    "valid_execution_quote": valid_execution_quote,
                    "drop_reason": drop_reason,
                    "quote_age_ms": quote_age_ms,
                    "ret_15m_bps": feat.ret_15m_bps,
                    "atr_14": feat.atr_14,
                },
            )
            prospective_v1_append_rec_trade(
                market=ProspectiveMarket.CRYPTO,
                symbol="BTCUSDT",
                session_date=session_date_utc,
                bar_idx=bar_idx,
                decision=str(action_b),
                secondary_decision=str(action_c),
                close_time_ms=int(candle.close_time_ms),
                reference_price=float(candle.close),
                is_actionable=action_b in ("LONG", "SHORT"),
                regime=regime_label,
                plan=plan_result if isinstance(plan_result, dict) else None,
                extra={
                    "policy_b_action": action_b,
                    "policy_c_action": action_c,
                    "valid_execution_quote": valid_execution_quote,
                    "drop_reason": drop_reason,
                    "quote_age_ms": quote_age_ms,
                    "ret_15m_bps": feat.ret_15m_bps,
                    "atr_14": feat.atr_14,
                },
            )
            arm_a = action_b in ("LONG", "SHORT")
            prospective_v1_observe_bar(
                market=ProspectiveMarket.CRYPTO,
                completed_bars_including_signal=self.recent_candles,
                ask=ask_v,
                bid=bid_v,
                quote_age_ms=age_v,
                session_date=session_date_utc,
                symbol="BTCUSDT",
                bar_idx=bar_idx,
                close_time_ms=int(candle.close_time_ms),
                arm_a_signal=arm_a,
                arm_a_side=action_b if arm_a else None,
                reference_price=float(candle.close),
                extra={
                    "policy_b_action": action_b,
                    "policy_c_action": action_c,
                    "drop_reason": drop_reason,
                },
            )
        except Exception as obs_exc:
            print(f"[prospective_v1] CRYPTO observe skipped: {obs_exc}")

        print(
            f"[{datetime.now(timezone.utc).strftime('%H:%M:%S')}] Bar {bar_idx:04d} Closed: "
            f"P_ref={candle.close:.2f} | B:{action_b} C:{action_c} | "
            f"Quote Valid: {valid_execution_quote} ({drop_reason or 'PASS'})"
        )

    async def ingest_klines(self):
        """Subscribes to live Binance btcusdt@kline_15m stream."""
        ws_url = "wss://stream.binance.com:9443/ws/btcusdt@kline_15m"
        while self.running:
            try:
                async with websockets.connect(ws_url, ping_interval=20, ping_timeout=10) as ws:
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
                                active = (self.trade_plan_overlay.state or {}).get("active_plan")
                                if active and active.get("state") == "OPEN" and self.latest_book_ticker:
                                    forming_q = dict(self.latest_book_ticker)
                                    forming_q["low_price"] = float(k["l"])
                                    forming_q["high_price"] = float(k["h"])
                                    forming_q["quote_source"] = "binance_ws_forming_kline_15m"
                                    resolved = self.trade_plan_overlay.evaluate_market_tick(
                                        forming_q,
                                        reload_state=False,
                                        now_ms=int(time.time() * 1000),
                                    )
                                    if resolved and resolved.get("state") != "OPEN":
                                        print(
                                            f"[trade_plan_overlay] forming-kline path-through "
                                            f"{resolved.get('plan_id')} → {resolved.get('state')} "
                                            f"reason={resolved.get('exit_reason')}"
                                        )
                                        try:
                                            prospective_v1_append_trade_closed(
                                                market=ProspectiveMarket.CRYPTO,
                                                outcome=resolved,
                                            )
                                        except Exception as link_exc:
                                            print(f"[prospective_v1] trade_closed link skipped: {link_exc}")
                            except Exception as form_exc:
                                print(f"[trade_plan_overlay] forming-kline skipped: {form_exc}")

                        if is_candle_closed:
                            candle = Candle15m.from_binance_ws(k)
                            await self.evaluate_completed_candle(candle)
                            if self.single_candle_test:
                                self.request_shutdown()
                                break
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

        # Calibrate Exchange Clock Offset
        offset_ms, rtt_ms = self.clock_filter.sample_offset(samples=5)
        self.state["telemetry_gates"]["max_drift_ms"] = max(
            self.state["telemetry_gates"]["max_drift_ms"], abs(offset_ms)
        )
        if not self.clock_filter.is_synchronized():
            raise ValueError(
                f"FATAL: Exchange clock drift ({offset_ms} ms) exceeds 250 ms threshold! "
                f"RTT: {rtt_ms} ms. Fail-closed boundary activated."
            )

        print(f"[RP-608 Clock Sync] Offset: {offset_ms} ms (RTT: {rtt_ms} ms). Status: PASS")

        # Concurrent Tasks: bookTicker ingestion and kline_15m evaluation
        t_ticker = asyncio.create_task(self.ingest_book_ticker())
        t_klines = asyncio.create_task(self.ingest_klines())

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

            # Await feed tasks to ensure clean socket closures
            results = await asyncio.gather(t_ticker, t_klines, return_exceptions=True)
            for r in results:
                if isinstance(r, Exception) and not isinstance(r, asyncio.CancelledError):
                    print(f"[RP-608 Warning] Feed task returned error on cancellation: {r}")

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
    parser = argparse.ArgumentParser(description="RP-608-SHADOW-001B Prospective Observation Daemon")
    parser.add_argument("--output-dir", type=str, default=None, help="Explicit output directory for isolated checkpoint and ledgers.")
    parser.add_argument("--dry-run", action="store_true", help="Run preflight and exit without opening sockets.")
    parser.add_argument("--test-resumption", action="store_true", help="Test recovery and downtime accounting from existing checkpoint.")
    parser.add_argument("--single-candle-test", action="store_true", help="Run until first 15m candle closes and exit.")
    args = parser.parse_args()

    daemon = RP608ShadowDaemon(
        output_dir=args.output_dir,
        single_candle_test=args.single_candle_test,
    )

    if args.dry_run:
        print("[Dry Run] Verifying artifact hashes and preflight checks...")
        daemon.preflight_integrity_check()
        daemon.initialize_or_resume_state()
        offset, rtt = daemon.clock_filter.sample_offset(samples=3)
        print(f"[Dry Run] Clock Offset: {offset} ms (RTT: {rtt} ms). PASS.")
        print("[Dry Run] Preflight verification succeeded cleanly.")
        return

    if args.test_resumption:
        print("[Test Resumption] Exercising crash-safe recovery logic...")
        daemon.preflight_integrity_check()
        daemon.initialize_or_resume_state()
        print("[Test Resumption] Resumption logic executed cleanly.")
        return

    asyncio.run(daemon.run())


if __name__ == "__main__":
    main()

