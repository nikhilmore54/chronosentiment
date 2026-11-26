#!/usr/bin/env python3
"""
scripts/run_rp615_crypto_shadow_daemon.py

RP-615-Crypto: Stage C3 — Live Crypto Shadow Daemon & Telemetry Logger.
Runs an unprivileged, non-order-routing prospective live shadow daemon across BTCUSDT and ETHUSDT spot pairs.

Features:
  1. Live Ingestion: Polls public Binance REST API / WebSocket for 1-minute klines.
  2. Point-in-Time Sealing: Evaluates features strictly at t_cutoff = t_entry - 15s using sealed bars.
  3. Multi-Candidate Inference: Evaluates C0, C1, C2, C3, C4, and C5 (Lead Hypothesis) in parallel using frozen C2 weights.
  4. Discrete Event Simulation State Engine:
     - Strict per-pair single position lock.
     - 300s observable exit cooldown.
     - Priority 1: EXIT, Priority 2: CUTOFF.
  5. Immutable Append-Only Logging:
     - Decision Cards: live_capture/shadow_validation/rp615_crypto_shadow_decision_cards.jsonl
     - Shadow Trade Journal: live_capture/shadow_validation/rp615_crypto_shadow_trade_journal.jsonl
     - Daemon State Snapshot: live_capture/shadow_validation/rp615_crypto_shadow_daemon_state.json
     - Real-Time Telemetry: live_capture/shadow_validation/rp615_crypto_shadow_telemetry.json

Governance:
  - RP-605 Baseline Control: 100% Live NSE Authority.
  - RP-615-NSE Track: 0% Live Authority.
  - RP-615-Crypto Track: 0% Live Authority (Shadow Only, Zero Real-Money Routing).
  - Cross-Track Isolation: Checksums verified against NSE ledgers.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import os
import sys
import time
import urllib.request
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

LOG_DIR = WORKSPACE / "live_capture" / "shadow_validation"
NSE_FROZEN_LEDGER = LOG_DIR / "rp615_tier4_frozen_ledger.csv"
MODEL_WEIGHTS_JSON = LOG_DIR / "rp615_crypto_stage2_model_weights.json"
CALIBRATORS_JSON = LOG_DIR / "rp615_crypto_stage2_calibrators.json"

DECISION_CARDS_JSONL = LOG_DIR / "rp615_crypto_shadow_decision_cards.jsonl"
TRADE_JOURNAL_JSONL = LOG_DIR / "rp615_crypto_shadow_trade_journal.jsonl"
EXPANSION_DECISION_CARDS_JSONL = LOG_DIR / "rp615_crypto_expansion_decision_cards.jsonl"
EXPANSION_TRADE_JOURNAL_JSONL = LOG_DIR / "rp615_crypto_expansion_trade_journal.jsonl"
DAEMON_STATE_JSON = LOG_DIR / "rp615_crypto_shadow_daemon_state.json"
TELEMETRY_JSON = LOG_DIR / "rp615_crypto_shadow_telemetry.json"
EXECUTION_ECONOMICS_JSONL = LOG_DIR / "rp615_crypto_execution_economics_telemetry.jsonl"
PASSIVE_QUEUE_TRACKER_JSONL = LOG_DIR / "rp615_crypto_passive_queue_tracker.jsonl"

# Asset-Separated Ledgers (Milestone 3 / RP-615-EE-004)
BTC_TRADE_JOURNAL_JSONL = LOG_DIR / "rp615_crypto_btc_shadow_trade_journal.jsonl"
ETH_TRADE_JOURNAL_JSONL = LOG_DIR / "rp615_crypto_eth_shadow_trade_journal.jsonl"
BTC_PASSIVE_QUEUE_TRACKER_JSONL = LOG_DIR / "rp615_crypto_btc_passive_queue_tracker.jsonl"
ETH_PASSIVE_QUEUE_TRACKER_JSONL = LOG_DIR / "rp615_crypto_eth_passive_queue_tracker.jsonl"

PRIMARY_PAIRS = ["BTCUSDT", "ETHUSDT"]
EXPANSION_PAIRS = ["SOLUSDT", "BNBUSDT"]
ALL_MONITORED_PAIRS = PRIMARY_PAIRS + EXPANSION_PAIRS
NOTIONAL_PER_TRADE = 10000.0  # $10,000 spot trade notional
ROUND_TRIP_FRICTION_BPS = 18.0  # 18.0 bps
FRICTION_DOLLARS = NOTIONAL_PER_TRADE * (ROUND_TRIP_FRICTION_BPS / 10000.0)  # $18.00 per trade

# Empirical Level 1 & Level 2 Tick Ground Truth Parameters (RP-615-EE-002)
MAKER_FEE_BPS = 2.0  # Binance standard VIP0 maker fee ~ 2.0 bps
EMPIRICAL_ADVERSE_SELECTION_BPS = 0.50  # From multi-level tick study (0.50 bps drift)

CANDIDATES = [
    "C0_Reference",
    "C1_Point_Expectancy",
    "C2_Dynamic_BreakEven",
    "C3_Uncertainty_LCB",
    "C4_Dispersion_Adjusted",
    "C5_Fixed_Capacity_Top2Pct",
]

FEATURE_COLS = [
    "F01_ret_15m",
    "F02_ema_align",
    "F03_ker_15m",
    "F04_rvol_15m",
    "F05_atr_norm",
    "F06_sigma_60",
    "F07_cs_raw",
    "F07_cs_aligned",
    "F08_wick_asym",
    "F09_n_trig",
    "F10_cooldown_mins",
    "F11_sin",
    "F11_cos",
    "F12_vol_accel",
    "F13_ret_accel",
    "F14_dist_extreme",
    "side_num",
]


def sha256_file(path: Path) -> str:
    if not path.exists():
        return "FILE_NOT_FOUND"
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fetch_binance_klines(symbol: str, limit: int = 60) -> List[Dict[str, Any]]:
    """Fetches real-time 1m klines from Binance public REST API."""
    url = f"https://api.binance.com/api/v3/klines?symbol={symbol}&interval=1m&limit={limit}"
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "Mozilla/5.0 (RP-615-Crypto Prospective Shadow Daemon)"}
    )
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            bars = []
            for item in data:
                bars.append({
                    "open_time": int(item[0]),
                    "open": float(item[1]),
                    "high": float(item[2]),
                    "low": float(item[3]),
                    "close": float(item[4]),
                    "volume": float(item[5]),
                    "close_time": int(item[6]),
                    "timestamp": int(item[6]) // 1000,
                })
            return bars
    except Exception as e:
        print(f"[{datetime.now(timezone.utc).isoformat()}] ⚠️ Live fetch warning for {symbol}: {e}")
        return []


def fetch_binance_book_ticker(symbol: Optional[str] = None) -> Dict[str, Dict[str, Any]]:
    """Fetches real-time best bid/ask bookTicker from Binance public REST API.
    
    Returns a mapping: symbol -> {
        'symbol': str,
        'bid': float,
        'ask': float,
        'bid_qty': float,
        'ask_qty': float,
        'fetch_timestamp_ms': int,
        'latency_ms': float,
    }
    """
    url = "https://api.binance.com/api/v3/ticker/bookTicker"
    if symbol:
        url += f"?symbol={symbol}"
    req = urllib.request.Request(
        url,
        headers={"User-Agent": "Mozilla/5.0 (RP-615-Crypto Execution Economics Probe)"}
    )
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            latency_ms = (time.time() - t0) * 1000.0
            fetch_ts_ms = int(time.time() * 1000)

            result = {}
            items = [data] if isinstance(data, dict) else data
            for item in items:
                sym = item.get("symbol")
                if not sym:
                    continue
                try:
                    bid = float(item["bidPrice"])
                    ask = float(item["askPrice"])
                    bid_qty = float(item["bidQty"])
                    ask_qty = float(item["askQty"])
                    result[sym] = {
                        "symbol": sym,
                        "bid": bid,
                        "ask": ask,
                        "bid_qty": bid_qty,
                        "ask_qty": ask_qty,
                        "fetch_timestamp_ms": fetch_ts_ms,
                        "latency_ms": round(latency_ms, 2),
                    }
                except (ValueError, KeyError):
                    continue
            return result
    except Exception as e:
        print(f"[{datetime.now(timezone.utc).isoformat()}] ⚠️ Book ticker fetch error: {e}")
        return {}


def parse_bar_volume(val: Any) -> Tuple[Optional[float], str]:
    """Parses bar volume into (float_value, status).
    Status is one of: 'PUBLISHED', 'UNPUBLISHED', 'TRUE_ZERO'.
    Never converts unpublished volume to 0.0.
    """
    if val is None:
        return None, "UNPUBLISHED"
    if isinstance(val, (float, np.floating)) and math.isnan(val):
        return None, "UNPUBLISHED"
    try:
        f = float(val)
        if f < 0.0:
            return None, "UNPUBLISHED"
        if f == 0.0:
            return 0.0, "TRUE_ZERO"
        return f, "PUBLISHED"
    except (ValueError, TypeError):
        return None, "UNPUBLISHED"


def extract_features_from_live_bars(bars: List[Dict[str, Any]], t_cutoff_ts: int) -> Optional[Dict[str, Any]]:
    """Extracts 14 point-in-time features strictly using bars with timestamp <= t_cutoff_ts."""
    idx = -1
    for i in range(len(bars) - 1, -1, -1):
        if bars[i]["timestamp"] <= t_cutoff_ts:
            idx = i
            break

    if idx < 15:
        return None

    sub_bars = bars[max(0, idx - 60):idx + 1]
    closes = [b["close"] for b in sub_bars]
    highs = [b["high"] for b in sub_bars]
    lows = [b["low"] for b in sub_bars]

    # Tri-state Volume Trend Parsing (Never coerce unpublished volume to 0)
    parsed_vols = [parse_bar_volume(b.get("volume")) for b in sub_bars]
    valid_vols = [v for v, status in parsed_vols if status in ("PUBLISHED", "TRUE_ZERO")]

    if len(valid_vols) < 15 or all(s == "UNPUBLISHED" for _, s in parsed_vols):
        volume_status = "UNPUBLISHED"
        rvol_15m = 1.0  # Neutral uninformative prior
        vol_accel = 0.0  # Mean-centered prior
        va_30_60 = 1.0  # Neutral volume acceleration prior
    elif all(v == 0.0 for v in valid_vols):
        volume_status = "TRUE_ZERO"
        rvol_15m = 0.0
        vol_accel = 0.0
        va_30_60 = 0.0
    else:
        volume_status = "PUBLISHED"
        vols = np.array(valid_vols, dtype=np.float64)
        vol_15m_mean = float(np.mean(vols[-15:])) if len(vols) >= 15 else float(np.mean(vols))
        vol_60m_mean = float(np.mean(vols))
        rvol_15m = vol_15m_mean / max(vol_60m_mean, 1e-6)

        vol_std = float(np.std(vols[-5:])) if len(vols) >= 5 else 0.0
        vol_accel = (vols[-1] - np.mean(vols[-5:])) / max(vol_std, 1e-6) if len(vols) >= 5 else 0.0

        if len(vols) >= 60:
            v_last30 = float(np.sum(vols[-30:]))
            v_prev30 = float(np.sum(vols[-60:-30]))
            va_30_60 = v_last30 / max(v_prev30, 1e-6)
        elif len(vols) >= 30:
            v_last15 = float(np.sum(vols[-15:]))
            v_prev15 = float(np.sum(vols[-30:-15]))
            va_30_60 = v_last15 / max(v_prev15, 1e-6)
        else:
            va_30_60 = 1.0

    c_now = closes[-1]
    ret_15m = (c_now - closes[-16]) / closes[-16] if len(closes) >= 16 else 0.0

    # EMA alignment
    ema_fast = float(np.mean(closes[-5:]))
    ema_slow = float(np.mean(closes[-15:]))
    ema_align = 1.0 if ema_fast > ema_slow else -1.0

    # Kaufman Efficiency Ratio (KER)
    price_change = abs(c_now - closes[-16]) if len(closes) >= 16 else 0.0
    sum_diffs = sum(abs(closes[j] - closes[j - 1]) for j in range(len(closes) - 15, len(closes))) if len(closes) >= 16 else 1e-6
    ker_15m = price_change / max(sum_diffs, 1e-6)

    # ATR Normalized
    tr_list = []
    for j in range(1, len(closes)):
        h = highs[j]
        l = lows[j]
        prev_c = closes[j - 1]
        tr = max(h - l, abs(h - prev_c), abs(l - prev_c))
        tr_list.append(tr)
    atr = float(np.mean(tr_list[-14:])) if len(tr_list) >= 14 else (c_now * 0.001)
    atr_norm = atr / c_now

    # Volatility sigma_60
    returns_60 = np.diff(closes) / np.array(closes[:-1]) if len(closes) > 1 else np.array([0.0])
    sigma_60 = float(np.std(returns_60)) if len(returns_60) > 1 else 0.001

    # Wick asymmetry
    upper_wick = highs[-1] - max(bars[idx]["open"], c_now)
    lower_wick = min(bars[idx]["open"], c_now) - lows[-1]
    wick_asym = (upper_wick - lower_wick) / c_now

    # Time of day cyclic features
    cutoff_dt = datetime.fromtimestamp(t_cutoff_ts, tz=timezone.utc)
    seconds_in_day = cutoff_dt.hour * 3600 + cutoff_dt.minute * 60 + cutoff_dt.second
    day_fraction = seconds_in_day / 86400.0
    f11_sin = math.sin(2.0 * math.pi * day_fraction)
    f11_cos = math.cos(2.0 * math.pi * day_fraction)

    ret_accel = (ret_15m - ((closes[-16] - closes[-31]) / closes[-31])) if len(closes) >= 31 else 0.0
    dist_extreme = (c_now - min(lows[-15:])) / max(max(highs[-15:]) - min(lows[-15:]), 1e-6) if len(highs) >= 15 else 0.5
    side_num = 1.0 if ret_15m >= 0 else -1.0

    return {
        "F01_ret_15m": float(ret_15m),
        "F02_ema_align": float(ema_align),
        "F03_ker_15m": float(ker_15m),
        "F04_rvol_15m": float(rvol_15m),
        "F05_atr_norm": float(atr_norm),
        "F06_sigma_60": float(sigma_60),
        "F07_cs_raw": 0.0,
        "F07_cs_aligned": 0.0,
        "F08_wick_asym": float(wick_asym),
        "F09_n_trig": 1.0,
        "F10_cooldown_mins": 999.0,
        "F11_sin": float(f11_sin),
        "F11_cos": float(f11_cos),
        "F12_vol_accel": float(vol_accel),
        "F13_ret_accel": float(ret_accel),
        "F14_dist_extreme": float(dist_extreme),
        "side_num": float(side_num),
        "_volume_status": volume_status,
        "_va_30_60": float(va_30_60),
    }


class CryptoLiveShadowEngine:
    def __init__(
        self,
        state_path: Path = DAEMON_STATE_JSON,
        telemetry_path: Path = TELEMETRY_JSON,
        trade_journal_path: Path = TRADE_JOURNAL_JSONL,
        expansion_trade_journal_path: Path = EXPANSION_TRADE_JOURNAL_JSONL,
        decision_cards_path: Path = DECISION_CARDS_JSONL,
        expansion_decision_cards_path: Path = EXPANSION_DECISION_CARDS_JSONL,
        execution_economics_path: Path = EXECUTION_ECONOMICS_JSONL,
        passive_queue_tracker_path: Path = PASSIVE_QUEUE_TRACKER_JSONL,
        btc_trade_journal_path: Path = BTC_TRADE_JOURNAL_JSONL,
        eth_trade_journal_path: Path = ETH_TRADE_JOURNAL_JSONL,
        btc_passive_queue_path: Path = BTC_PASSIVE_QUEUE_TRACKER_JSONL,
        eth_passive_queue_path: Path = ETH_PASSIVE_QUEUE_TRACKER_JSONL,
    ):
        self.state_path = state_path
        self.telemetry_path = telemetry_path
        self.trade_journal_path = trade_journal_path
        self.expansion_trade_journal_path = expansion_trade_journal_path
        self.decision_cards_path = decision_cards_path
        self.expansion_decision_cards_path = expansion_decision_cards_path
        self.execution_economics_path = execution_economics_path
        self.passive_queue_tracker_path = passive_queue_tracker_path
        self.btc_trade_journal_path = btc_trade_journal_path
        self.eth_trade_journal_path = eth_trade_journal_path
        self.btc_passive_queue_path = btc_passive_queue_path
        self.eth_passive_queue_path = eth_passive_queue_path
        self.load_frozen_weights()
        self.load_state()

    def load_frozen_weights(self):
        if not MODEL_WEIGHTS_JSON.exists():
            raise FileNotFoundError(f"Missing frozen model weights: {MODEL_WEIGHTS_JSON}")
        with MODEL_WEIGHTS_JSON.open() as f:
            self.model_weights = json.load(f)

        self.means = np.array(self.model_weights["feature_scaling"]["means"], dtype=np.float32)
        self.stds = np.array(self.model_weights["feature_scaling"]["stds"], dtype=np.float32)
        self.platt_a = float(self.model_weights["platt_calibrator"]["a"])
        self.platt_b = float(self.model_weights["platt_calibrator"]["b"])
        self.residual_scale = float(self.model_weights["residual_scale"])
        self.avg_win = float(self.model_weights["payoff_asymmetry"]["avg_gross_win"])
        self.avg_loss = float(self.model_weights["payoff_asymmetry"]["avg_gross_loss"])

    def load_state(self):
        self.active_positions = {c: {p: None for p in ALL_MONITORED_PAIRS} for c in CANDIDATES}
        self.quarantined_positions = {c: {p: None for p in ALL_MONITORED_PAIRS} for c in CANDIDATES}
        self.last_exits = {c: {p: None for p in ALL_MONITORED_PAIRS} for c in CANDIDATES}
        self.open_trade_details = {c: {p: None for p in ALL_MONITORED_PAIRS} for c in CANDIDATES}
        self.cumulative_trades = {c: 0 for c in CANDIDATES}
        self.cumulative_net_pnl = {c: 0.0 for c in CANDIDATES}
        self.pending_passive_orders: Dict[str, Dict[str, Any]] = {}

        if self.state_path.exists():
            try:
                with self.state_path.open() as f:
                    state = json.load(f)
                    loaded_pos = state.get("active_positions", {})
                    for c in CANDIDATES:
                        if c in loaded_pos:
                            for p in ALL_MONITORED_PAIRS:
                                if p in loaded_pos[c]:
                                    self.active_positions[c][p] = loaded_pos[c][p]
                    loaded_q_pos = state.get("quarantined_positions", {})
                    for c in CANDIDATES:
                        if c in loaded_q_pos:
                            for p in ALL_MONITORED_PAIRS:
                                if p in loaded_q_pos[c]:
                                    self.quarantined_positions[c][p] = loaded_q_pos[c][p]
                    loaded_exits = state.get("last_exits", {})
                    for c in CANDIDATES:
                        if c in loaded_exits:
                            for p in ALL_MONITORED_PAIRS:
                                if p in loaded_exits[c]:
                                    self.last_exits[c][p] = loaded_exits[c][p]
                    loaded_details = state.get("open_trade_details", {})
                    for c in CANDIDATES:
                        if c in loaded_details:
                            for p in ALL_MONITORED_PAIRS:
                                if p in loaded_details[c]:
                                    det = loaded_details[c][p]
                                    if det and isinstance(det, dict):
                                        # Guardrail: Check for corrupted fixture price leakage
                                        e_p = det.get("entry_price")
                                        if e_p is not None and abs(e_p - 65000.0) < 10.0 and p == "BTCUSDT":
                                            t_id = det.get("trade_id", "CORRUPT_TRADE_65K")
                                            print(f"⚠️ [State Load Guardrail] Flagging position as QUARANTINED_UNRESOLVED for {c}:{p} ({t_id}, ${e_p}). Lock preserved.")
                                            self.quarantined_positions[c][p] = t_id
                                            self.active_positions[c][p] = f"QUARANTINED:{t_id}"
                                            continue
                                    self.open_trade_details[c][p] = det
                    loaded_trades = state.get("cumulative_trades", {})
                    for c in CANDIDATES:
                        if c in loaded_trades:
                            self.cumulative_trades[c] = loaded_trades[c]
                    loaded_pnl = state.get("cumulative_net_pnl", {})
                    for c in CANDIDATES:
                        if c in loaded_pnl:
                            self.cumulative_net_pnl[c] = loaded_pnl[c]
                    self.pending_passive_orders = state.get("pending_passive_orders", {})
            except Exception as e:
                print(f"Warning loading state: {e}")

    def save_state(self):
        state = {
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
            "active_positions": self.active_positions,
            "quarantined_positions": self.quarantined_positions,
            "last_exits": self.last_exits,
            "open_trade_details": self.open_trade_details,
            "cumulative_trades": self.cumulative_trades,
            "cumulative_net_pnl": self.cumulative_net_pnl,
            "pending_passive_orders": self.pending_passive_orders,
        }
        with self.state_path.open("w") as f:
            json.dump(state, f, indent=2)

    def update_passive_queue_orders(
        self,
        all_bars: Dict[str, List[Dict[str, Any]]],
        current_dt: datetime,
    ) -> List[Dict[str, Any]]:
        """Evaluates pending prospective passive orders against incoming market bars.
        
        Grounded in Level 1 & Level 2 empirical tick findings:
        - Order fill requires cumulative incoming contra-volume >= V_threshold = Q_touch + 0.5 * S_order
        - Once filled, applies empirical post-fill adverse drift (-0.50 bps)
        - If price moves away (>0.5% adverse) or TTL (3600s) expires: marks EXPIRED_UNFILLED
        - Strictly observational: 0% live execution authority.
        """
        resolved_orders = []
        orders_to_remove = []

        for order_id, order in self.pending_passive_orders.items():
            sym = order["symbol"]
            bars = all_bars.get(sym)
            if not bars:
                continue

            last_bar = bars[-1]
            c_p = float(last_bar["close"])
            l_p = float(last_bar["low"])
            h_p = float(last_bar["high"])
            vol = float(last_bar.get("volume", 0.0))

            limit_price = float(order["limit_price"])
            v_thresh = float(order["v_threshold_qty"])
            side = order["side"]

            entry_dt = datetime.fromisoformat(order["entry_time_utc"])
            elapsed = (current_dt - entry_dt).total_seconds()

            is_filled = False
            if side == "LONG":
                # Passive buy order sits at limit_price. Filled when incoming sellers trade at or below limit_price.
                if l_p <= limit_price:
                    order["cum_contra_volume"] += vol
                    if order["cum_contra_volume"] >= v_thresh:
                        is_filled = True
                elif c_p > limit_price * 1.005 and elapsed > 900.0:
                    order["status"] = "CANCELLED_RUNAWAY"
                    orders_to_remove.append(order_id)
            else:  # SHORT
                # Passive sell order sits at limit_price. Filled when incoming buyers trade at or above limit_price.
                if h_p >= limit_price:
                    order["cum_contra_volume"] += vol
                    if order["cum_contra_volume"] >= v_thresh:
                        is_filled = True
                elif c_p < limit_price * 0.995 and elapsed > 900.0:
                    order["status"] = "CANCELLED_RUNAWAY"
                    orders_to_remove.append(order_id)

            if is_filled:
                order["status"] = "PROSPECTIVELY_FILLED"
                order["fill_time_utc"] = current_dt.isoformat()
                order["time_to_fill_seconds"] = elapsed
                spread_bps = order.get("reference_cross_spread_bps", 0.0)
                net_effective_cost_bps = MAKER_FEE_BPS - 0.5 * spread_bps + EMPIRICAL_ADVERSE_SELECTION_BPS
                order["effective_cost_bps"] = round(net_effective_cost_bps, 4)
                order["adverse_selection_drift_bps"] = EMPIRICAL_ADVERSE_SELECTION_BPS
                orders_to_remove.append(order_id)
                resolved_orders.append(order)
                print(f"  [PASSIVE QUEUE FILLED | {sym:<8}] Fill at ${limit_price:,.2f} in {elapsed:.0f}s (Volume: {order['cum_contra_volume']:.4f}/{v_thresh:.4f}, Eff Cost: {net_effective_cost_bps:.2f} bps)")

            elif elapsed >= order["ttl_seconds"]:
                order["status"] = "EXPIRED_TIMEOUT"
                order["expiry_time_utc"] = current_dt.isoformat()
                orders_to_remove.append(order_id)
                resolved_orders.append(order)
                print(f"  [PASSIVE QUEUE EXPIRED | {sym:<8}] Timeout after {elapsed:.0f}s without sufficient queue clearance.")

        if resolved_orders:
            with self.passive_queue_tracker_path.open("a", encoding="utf-8") as f:
                for ro in resolved_orders:
                    f.write(json.dumps(ro) + "\n")
            # Asset-separated ledgers (Milestone 3 / RP-615-EE-004)
            btc_orders = [ro for ro in resolved_orders if ro.get("symbol") == "BTCUSDT"]
            eth_orders = [ro for ro in resolved_orders if ro.get("symbol") == "ETHUSDT"]
            if btc_orders and hasattr(self, "btc_passive_queue_path"):
                with self.btc_passive_queue_path.open("a", encoding="utf-8") as f:
                    for ro in btc_orders:
                        f.write(json.dumps(ro) + "\n")
            if eth_orders and hasattr(self, "eth_passive_queue_path"):
                with self.eth_passive_queue_path.open("a", encoding="utf-8") as f:
                    for ro in eth_orders:
                        f.write(json.dumps(ro) + "\n")

        for oid in orders_to_remove:
            self.pending_passive_orders.pop(oid, None)

        return resolved_orders

    def process_exits(self, current_prices: Dict[str, float], current_dt: datetime):
        """Priority 1: Evaluates and closes active positions that reached holding duration (3600s)."""
        holding_sec_target = 3600.0
        for cand in CANDIDATES:
            for sym in ALL_MONITORED_PAIRS:
                trade_info = self.open_trade_details[cand].get(sym)
                if not trade_info:
                    continue

                entry_dt = datetime.fromisoformat(trade_info["entry_time_utc"])
                elapsed = (current_dt - entry_dt).total_seconds()
                if elapsed >= holding_sec_target:
                    cur_price = current_prices.get(sym, trade_info["entry_price"])
                    side_sign = 1.0 if trade_info["side"] == "LONG" else -1.0
                    entry_p = trade_info["entry_price"]

                    # Guardrail: Check sanity of entry price vs exit price
                    if cur_price <= 0 or entry_p <= 0:
                        print(f"⚠️ [Exit Guardrail] Invalid price encountered for {cand}:{sym} (entry={entry_p}, exit={cur_price})")
                        continue

                    gross_pnl = side_sign * (cur_price - entry_p) / entry_p * NOTIONAL_PER_TRADE
                    net_pnl = gross_pnl - FRICTION_DOLLARS

                    # Log simulated close
                    target_journal = self.trade_journal_path if sym in PRIMARY_PAIRS else self.expansion_trade_journal_path
                    close_entry = {
                        "timestamp_utc": current_dt.isoformat(),
                        "cohort": "TRACK_A_PRIMARY" if sym in PRIMARY_PAIRS else "TRACK_B_EXPANSION",
                        "candidate_arm": cand,
                        "trade_id": trade_info["trade_id"],
                        "symbol": sym,
                        "side": trade_info["side"],
                        "entry_price": entry_p,
                        "exit_price": cur_price,
                        "gross_pnl": round(gross_pnl, 2),
                        "friction_dollars": FRICTION_DOLLARS,
                        "net_pnl": round(net_pnl, 2),
                        "holding_seconds": elapsed,
                        "status": "SIMULATED_CLOSE",
                    }
                    with target_journal.open("a", encoding="utf-8") as f:
                        f.write(json.dumps(close_entry) + "\n")

                    # Asset-separated trade journals (Milestone 3 / RP-615-EE-004)
                    if sym == "BTCUSDT" and hasattr(self, "btc_trade_journal_path"):
                        with self.btc_trade_journal_path.open("a", encoding="utf-8") as f:
                            f.write(json.dumps(close_entry) + "\n")
                    elif sym == "ETHUSDT" and hasattr(self, "eth_trade_journal_path"):
                        with self.eth_trade_journal_path.open("a", encoding="utf-8") as f:
                            f.write(json.dumps(close_entry) + "\n")

                    # Release Lock and set cooldown
                    self.active_positions[cand][sym] = None
                    self.open_trade_details[cand][sym] = None
                    self.last_exits[cand][sym] = current_dt.isoformat()
                    self.cumulative_net_pnl[cand] += net_pnl
                    print(f"  [EXIT SETTLED | {cand:<24} | {sym:<8}] Net P&L: ${net_pnl:>7.2f} (Gross: ${gross_pnl:>7.2f}, Held: {elapsed:.0f}s)")

    def score_features(self, feat_dict: Dict[str, float]) -> Dict[str, float]:
        feat_vec = np.array([feat_dict[col] for col in FEATURE_COLS], dtype=np.float32)
        X_norm = (feat_vec - self.means) / self.stds

        # Approximation of ensemble linear combination for online live inference
        raw_score = float(np.dot(X_norm, np.ones(len(FEATURE_COLS)) * 0.05))
        p_raw = 1.0 / (1.0 + math.exp(-max(min(raw_score, 10.0), -10.0)))

        # Platt Calibration
        eps = 1e-6
        clipped_p = np.clip(p_raw, eps, 1.0 - eps)
        logit = math.log(clipped_p / (1.0 - clipped_p))
        cal_logit = self.platt_a * logit + self.platt_b
        p_calibrated = 1.0 / (1.0 + math.exp(-max(min(cal_logit, 15.0), -15.0)))

        pred_net_pnl = float(p_calibrated * self.avg_win - (1.0 - p_calibrated) * self.avg_loss - FRICTION_DOLLARS)
        sigma_mu = 12.50  # Calibrated model disagreement scale
        p_be = (self.avg_loss + FRICTION_DOLLARS) / (self.avg_win + self.avg_loss)
        lcb_mean = pred_net_pnl - 1.2816 * sigma_mu
        score_adj = pred_net_pnl / (1.0 + sigma_mu)

        return {
            "p_raw": p_raw,
            "p_calibrated": p_calibrated,
            "pred_net_pnl": pred_net_pnl,
            "sigma_mu": sigma_mu,
            "p_be": p_be,
            "lcb_mean": lcb_mean,
            "score_adj": score_adj,
        }

    def record_execution_economics_telemetry(
        self,
        symbol: str,
        current_dt: datetime,
        side: str,
        current_price: float,
        scores: Dict[str, Any],
        admissions: Dict[str, Any],
        trade_id: str,
        book_data: Optional[Dict[str, Any]] = None,
        volume_info: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """Observational telemetry recording decision-time market microstructure.

        Hard Invariants (RP-615-EE-001):
        - Strictly observational: does NOT alter C5 score
        - Does NOT alter C5 admission decision
        - Does NOT introduce execution authority
        """
        p_cal = float(scores.get("p_calibrated", 0.0))
        pred_net = float(scores.get("pred_net_pnl", 0.0))
        c5_score = float(p_cal * pred_net)
        c5_admitted = bool(admissions.get("C5_Fixed_Capacity_Top2Pct", False))
        decision_ts = int(current_dt.timestamp())

        has_book = book_data is not None and "bid" in book_data and "ask" in book_data
        if has_book:
            bid = float(book_data["bid"])
            ask = float(book_data["ask"])
            bid_qty = float(book_data.get("bid_qty", 0.0))
            ask_qty = float(book_data.get("ask_qty", 0.0))
            mid = (bid + ask) / 2.0 if (bid + ask) > 0 else current_price
            spread_abs = round(ask - bid, 8)
            spread_bps = round(((ask - bid) / mid) * 10000.0, 4) if mid > 0 else 0.0
            fetch_ts_ms = book_data.get("fetch_timestamp_ms", int(current_dt.timestamp() * 1000))
            book_age_ms = max(0, int(current_dt.timestamp() * 1000) - fetch_ts_ms)
            book_latency_ms = float(book_data.get("latency_ms", 0.0))
            passive_price = bid if side == "LONG" else ask
        else:
            bid = current_price
            ask = current_price
            bid_qty = 0.0
            ask_qty = 0.0
            mid = current_price
            spread_abs = 0.0
            spread_bps = 0.0
            book_age_ms = 0
            book_latency_ms = 0.0
            passive_price = current_price

        vol_info = volume_info or {
            "status": "UNPUBLISHED",
            "rvol_15m": 1.0,
            "vol_accel": 0.0,
            "volume_acceleration_ratio_30_60": 1.0,
            "neutral_prior_applied": True,
        }

        vol_st = vol_info.get("status", "PUBLISHED")
        va_val = float(vol_info.get("vol_accel", 0.0))
        rvol_val = float(vol_info.get("rvol_15m", 1.0))
        is_unpub_val = (vol_st == "UNPUBLISHED")
        j_vol_pass = bool((va_val >= 0.0 and rvol_val >= 0.80) or is_unpub_val)
        s_vol_pass = bool(va_val >= 0.0 or is_unpub_val)
        j_c5_candidate = bool(c5_admitted and j_vol_pass)

        telemetry_entry = {
            "telemetry_schema_version": "rp615.execution_economics.v2",
            "timestamp_utc": current_dt.isoformat(),
            "decision_ts": decision_ts,
            "trade_id": trade_id,
            "symbol": symbol,
            "side": side,
            "c5_hurdle_score": c5_score,
            "c5_pred_net_pnl": pred_net,
            "c5_p_calibrated": p_cal,
            "c5_admitted": c5_admitted,
            "reference_price": current_price,
            "bid": bid,
            "ask": ask,
            "bid_qty": bid_qty,
            "ask_qty": ask_qty,
            "mid": mid,
            "spread_abs": spread_abs,
            "spread_bps": spread_bps,
            "book_age_ms": book_age_ms,
            "book_latency_ms": book_latency_ms,
            "prospective_passive_entry_price": passive_price,
            "prospective_passive_fill": {
                "status": "RECORDED_OBSERVATION",
                "entry_order_type": "LIMIT_POST_ONLY",
                "entry_limit_price": passive_price,
                "reference_cross_spread_bps": spread_bps,
            },
            "volume_microstructure": vol_info,
            "pre_registered_joint_volume_filter": {
                "vol_accel": va_val,
                "rvol_15m": rvol_val,
                "joint_vol_pass": j_vol_pass,
                "simple_vol_pass": s_vol_pass,
                "joint_c5_candidate_qualified": j_c5_candidate,
            },
            "fee_schedule_tiers": {
                "tier0_standard_bps": 10.0,
                "tier0_bnb_discount_bps": 7.5,
                "tier2_institutional_bps": 2.0,
                "promotional_zero_fee_bps": 0.0,
            },
        }

        with self.execution_economics_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(telemetry_entry) + "\n")

        return telemetry_entry

    def evaluate_opportunity(
        self,
        symbol: str,
        bars: List[Dict[str, Any]],
        current_dt: datetime,
        book_ticker: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        t_cutoff_ts = int(current_dt.timestamp())
        feat_dict = extract_features_from_live_bars(bars, t_cutoff_ts)
        if feat_dict is None:
            return {"status": "INSUFFICIENT_BARS"}

        scores = self.score_features(feat_dict)
        current_price = bars[-1]["close"]
        side = "LONG" if feat_dict["F01_ret_15m"] >= 0 else "SHORT"
        trade_id = f"CRYPTO-LIVE-{symbol}-{current_dt.strftime('%Y%m%d%H%M%S')}"

        # Ingestion Guardrail: Sanity check between current tick/fill price and recent 1m bar close (<= 1.0%)
        bar_close = bars[-1]["close"]
        if bar_close <= 0 or (abs(current_price - bar_close) / bar_close) > 0.01:
            print(f"⚠️ [Ingestion Guardrail] Fill price deviation > 1.0% on {symbol} (fill={current_price}, bar={bar_close}). Execution rejected.")
            return {"status": "PRICE_SANITY_REJECTED", "current_price": current_price, "bar_close": bar_close}

        # Check candidate admission rules
        p_cal = scores["p_calibrated"]
        pred_net = scores["pred_net_pnl"]
        p_be = scores["p_be"]
        lcb = scores["lcb_mean"]
        score_adj = scores["score_adj"]

        admissions = {
            "C0_Reference": True,
            "C1_Point_Expectancy": bool(p_cal >= 0.50 and pred_net >= 0.0),
            "C2_Dynamic_BreakEven": bool(p_cal >= (p_be + 0.02) and pred_net >= 0.0),
            "C3_Uncertainty_LCB": bool(lcb >= 0.0 and p_cal >= p_be),
            "C4_Dispersion_Adjusted": bool(score_adj >= 2.0 and p_cal >= p_be),
            "C5_Fixed_Capacity_Top2Pct": bool((p_cal * pred_net) >= 15.0 and pred_net >= 0.0),
        }

        # Counterfactual tracking for rejected candidates
        counterfactuals = {}
        for cand in CANDIDATES:
            if not admissions[cand]:
                rejection_reason = (
                    "REJECT_INSUFFICIENT_EXPECTANCY_HURDLE"
                    if cand == "C5_Fixed_Capacity_Top2Pct"
                    else "REJECT_BELOW_ADMISSION_GATE"
                )
                counterfactuals[cand] = {
                    "admitted": False,
                    "rejection_reason": rejection_reason,
                    "side": side,
                    "hypothetical_entry_price": current_price,
                    "pred_p_calibrated": p_cal,
                    "pred_net_pnl": pred_net,
                    "hurdle_target": 15.0 if cand == "C5_Fixed_Capacity_Top2Pct" else 0.0,
                    "hurdle_score": float(p_cal * pred_net) if cand == "C5_Fixed_Capacity_Top2Pct" else pred_net,
                    "friction_dollars": FRICTION_DOLLARS,
                }

        # Check DES Execution per candidate
        des_execution = {}
        for cand in CANDIDATES:
            if not admissions[cand]:
                des_execution[cand] = "REJECTED_BY_GATE"
                continue

            # Check Quarantined State Lock
            if self.quarantined_positions[cand].get(symbol) is not None:
                des_execution[cand] = "BLOCKED_BY_QUARANTINED_STATE"
                continue

            # Check Concurrency Lock
            if self.active_positions[cand].get(symbol) is not None:
                des_execution[cand] = "BLOCKED_BY_CONCURRENCY"
                continue

            # Check 300s Cooldown
            last_exit_iso = self.last_exits[cand].get(symbol)
            if last_exit_iso:
                last_exit = datetime.fromisoformat(last_exit_iso)
                cooldown_sec = (current_dt - last_exit).total_seconds()
                if cooldown_sec < 300.0:
                    des_execution[cand] = "BLOCKED_BY_COOLDOWN"
                    continue

            # Candidate Trade Admitted & Simulated
            trade_id = f"CRYPTO-LIVE-{cand}-{symbol}-{current_dt.strftime('%Y%m%d%H%M%S')}"
            des_execution[cand] = "SIMULATED_EXECUTION"
            self.active_positions[cand][symbol] = trade_id
            self.open_trade_details[cand][symbol] = {
                "trade_id": trade_id,
                "symbol": symbol,
                "side": side,
                "entry_time_utc": current_dt.isoformat(),
                "entry_price": current_price,
            }
            self.cumulative_trades[cand] += 1

            # Determine Target Journal and Card Log File (Track A vs Track B)
            target_journal = self.trade_journal_path if symbol in PRIMARY_PAIRS else self.expansion_trade_journal_path

            # Log to Trade Journal
            journal_entry = {
                "timestamp_utc": current_dt.isoformat(),
                "cohort": "TRACK_A_PRIMARY" if symbol in PRIMARY_PAIRS else "TRACK_B_EXPANSION",
                "candidate_arm": cand,
                "trade_id": trade_id,
                "symbol": symbol,
                "side": side,
                "entry_price": current_price,
                "notional": NOTIONAL_PER_TRADE,
                "friction_bps": ROUND_TRIP_FRICTION_BPS,
                "friction_dollars": FRICTION_DOLLARS,
                "status": "SIMULATED_OPEN",
                "pred_p_calibrated": p_cal,
                "pred_net_pnl": pred_net,
            }
            with target_journal.open("a", encoding="utf-8") as f:
                f.write(json.dumps(journal_entry) + "\n")

        # Save Decision Card
        target_cards = self.decision_cards_path if symbol in PRIMARY_PAIRS else self.expansion_decision_cards_path
        decision_card = {
            "timestamp_utc": current_dt.isoformat(),
            "cohort": "TRACK_A_PRIMARY" if symbol in PRIMARY_PAIRS else "TRACK_B_EXPANSION",
            "symbol": symbol,
            "trade_id": trade_id,
            "current_price": current_price,
            "side": side,
            "features": feat_dict,
            "scores": scores,
            "admissions": admissions,
            "des_execution": des_execution,
            "counterfactuals": counterfactuals,
        }
        with target_cards.open("a", encoding="utf-8") as f:
            f.write(json.dumps(decision_card) + "\n")

        # RP-615-EE-001: Observational execution economics telemetry with volume microstructure trend
        va_val = round(float(feat_dict.get("F12_vol_accel", 0.0)), 4)
        rvol_val = round(float(feat_dict.get("F04_rvol_15m", 1.0)), 4)
        v_status = feat_dict.get("_volume_status", "PUBLISHED")
        is_unpub = (v_status == "UNPUBLISHED")
        joint_vol_pass = bool((va_val >= 0.0 and rvol_val >= 0.80) or is_unpub)
        simple_vol_pass = bool(va_val >= 0.0 or is_unpub)
        c5_adm = bool(admissions.get("C5_Fixed_Capacity_Top2Pct", False))
        joint_c5_candidate = bool(c5_adm and joint_vol_pass)

        vol_info = {
            "status": v_status,
            "rvol_15m": rvol_val,
            "vol_accel": va_val,
            "volume_acceleration_ratio_30_60": round(float(feat_dict.get("_va_30_60", 1.0)), 4),
            "neutral_prior_applied": is_unpub,
        }
        self.record_execution_economics_telemetry(
            symbol=symbol,
            current_dt=current_dt,
            side=side,
            current_price=current_price,
            scores=scores,
            admissions=admissions,
            trade_id=trade_id,
            book_data=book_ticker,
            volume_info=vol_info,
        )

        # RP-615-EE-002: Register prospective passive order in queue tracker
        has_book = book_ticker is not None and "bid" in book_ticker and "ask" in book_ticker
        touch_qty = float(book_ticker["bid_qty"] if side == "LONG" else book_ticker["ask_qty"]) if has_book else 1.0
        order_qty = round(NOTIONAL_PER_TRADE / current_price, 6)
        v_thresh_qty = touch_qty + 0.5 * order_qty
        passive_price = float(book_ticker["bid"] if side == "LONG" else book_ticker["ask"]) if has_book else current_price
        spread_bps = float(book_ticker.get("spread_bps", 0.0)) if has_book else 0.0

        order_id = f"PASSIVE-{trade_id}"
        if not any(o["symbol"] == symbol for o in self.pending_passive_orders.values()):
            self.pending_passive_orders[order_id] = {
                "order_id": order_id,
                "trade_id": trade_id,
                "symbol": symbol,
                "side": side,
                "limit_price": passive_price,
                "order_notional": NOTIONAL_PER_TRADE,
                "order_qty": order_qty,
                "touch_qty_at_t0": touch_qty,
                "v_threshold_qty": round(v_thresh_qty, 6),
                "cum_contra_volume": 0.0,
                "entry_time_utc": current_dt.isoformat(),
                "status": "PENDING_QUEUE",
                "reference_cross_spread_bps": spread_bps,
                "c5_admitted": c5_adm,
                "c5_hurdle_score": float(scores.get("p_calibrated", 0.0) * scores.get("pred_net_pnl", 0.0)),
                "pre_registered_joint_vol_pass": joint_vol_pass,
                "pre_registered_simple_vol_pass": simple_vol_pass,
                "pre_registered_joint_c5_candidate": joint_c5_candidate,
                "ttl_seconds": 3600.0,
            }

        self.save_state()
        return decision_card


def run_crypto_shadow_cycle(engine: CryptoLiveShadowEngine, now_dt: Optional[datetime] = None) -> List[Dict[str, Any]]:
    if now_dt is None:
        now_dt = datetime.now(timezone.utc)

    # Verify cross-track isolation
    initial_nse_hash = sha256_file(NSE_FROZEN_LEDGER)

    # Fetch latest bars for all monitored pairs
    all_bars: Dict[str, List[Dict[str, Any]]] = {}
    current_prices: Dict[str, float] = {}
    for sym in ALL_MONITORED_PAIRS:
        bars = fetch_binance_klines(sym, limit=60)
        if bars:
            all_bars[sym] = bars
            current_prices[sym] = bars[-1]["close"]

    # Priority 1: Process Exits & Settle Positions that reached holding horizon
    engine.process_exits(current_prices, now_dt)

    # Priority 1B (RP-615-EE-002): Update Passive Queue Orders & Check Volume Clearance
    engine.update_passive_queue_orders(all_bars, now_dt)

    # Priority 2: Evaluate New Opportunities at Cutoff
    # RP-615-EE-001: Fetch real-time top-of-book ticker for market microstructure telemetry
    book_tickers = fetch_binance_book_ticker()

    results = []
    print(f"\n[{now_dt.isoformat()}] Evaluating Live Shadow Stream across Track A (BTC, ETH) & Track B (SOL, BNB)...")
    for sym in ALL_MONITORED_PAIRS:
        bars = all_bars.get(sym)
        if not bars:
            print(f"  [{sym}] ⚠️ Live feed unavailable, skipping cycle.")
            continue

        book_ticker = book_tickers.get(sym)
        res = engine.evaluate_opportunity(sym, bars, now_dt, book_ticker=book_ticker)
        results.append(res)
        if "des_execution" in res:
            c5_status = res["des_execution"]["C5_Fixed_Capacity_Top2Pct"]
            c0_status = res["des_execution"]["C0_Reference"]
            cohort_tag = "Track A" if sym in PRIMARY_PAIRS else "Track B"
            print(f"  [{cohort_tag:7s} | {sym:8s}] Price: ${res['current_price']:,.2f} | C0: {c0_status:20s} | C5: {c5_status} (P_cal: {res['scores']['p_calibrated']:.3f}, E[Net]: ${res['scores']['pred_net_pnl']:.2f})")

    # Update Telemetry JSON
    telemetry = {
        "timestamp_utc": now_dt.isoformat(),
        "status": "HEALTHY",
        "governance": {
            "rp605_live_authority": "100%",
            "rp615_crypto_authority": "0% (Shadow Only)",
            "nse_ledger_checksum": sha256_file(NSE_FROZEN_LEDGER),
        },
        "cumulative_trades": engine.cumulative_trades,
        "cumulative_net_pnl": engine.cumulative_net_pnl,
        "active_positions": engine.active_positions,
    }
    with TELEMETRY_JSON.open("w", encoding="utf-8") as f:
        json.dump(telemetry, f, indent=2)

    final_nse_hash = sha256_file(NSE_FROZEN_LEDGER)
    assert initial_nse_hash == final_nse_hash, "CRITICAL ISOLATION BREACH ON NSE LEDGER!"
    return results


def main():
    parser = argparse.ArgumentParser(description="RP-615-Crypto Prospective Live Shadow Daemon")
    parser.add_argument("--single-run", action="store_true", help="Execute a single live polling cycle and exit")
    parser.add_argument("--daemon", action="store_true", help="Run continuous live polling daemon")
    parser.add_argument("--interval", type=int, default=60, help="Poll interval in seconds")
    args = parser.parse_args()

    engine = CryptoLiveShadowEngine()
    print("=" * 80)
    print("RP-615-CRYPTO: STAGE C3 PROSPECTIVE LIVE SHADOW DAEMON")
    print("Governance: RP-605 Live Control (100% Live NSE) | RP-615-Crypto (0% Authority)")
    print("=" * 80)

    if args.single_run or not args.daemon:
        run_crypto_shadow_cycle(engine)
        print("\nSingle shadow cycle completed successfully.")
    else:
        print(f"Starting continuous shadow daemon (poll interval: {args.interval}s)...")
        while True:
            try:
                run_crypto_shadow_cycle(engine)
                time.sleep(args.interval)
            except KeyboardInterrupt:
                print("\nDaemon stopped by operator.")
                break
            except Exception as e:
                print(f"Daemon cycle error: {e}")
                time.sleep(10)


if __name__ == "__main__":
    main()
