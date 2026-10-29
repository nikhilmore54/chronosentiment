#!/usr/bin/env python3
"""
scripts/rp608_crypto_event_engine.py

RP-608-EVENT-001: Parallel Multi-Timescale Event-Driven Shadow Recommendation Engine
Market: Binance BTC/USDT (L1 Top-of-Book stream: btcusdt@bookTicker + completed 15m candles)

Architectural Specification:
1. Shared Live Stream: Ingests Binance bookTicker quote events and completed 15m candles.
2. Four Independent Lookback Engines (Running in Parallel):
   - Candidate A: 16 bars (4 hours)
   - Candidate B: 32 bars (8 hours)
   - Candidate C: 64 bars (16 hours)
   - Candidate D: 120 bars (30 hours)
3. Three Analytical Timescales per Candidate:
   - Fast (Microstructure): Micro-price, top-of-book imbalance, 5s velocity, tick volatility, spread.
   - Medium / Slow (Candle Context): Lookback-specific trend return, momentum, and regime.
4. Noise & Spam Control:
   - Only active recommendations (LONG/SHORT) and explicit state transitions are written to ledgers.
   - 1.0-second minimum refractory period enforced on emitted recommendations.
   - Sub-tick deduplication suppresses noise jitter.
   - Stale quotes (>1000ms) and anomalous spreads (>5 bps) rejected.
5. Isolated Append-Only Ledgers:
   - Candidate A: live_capture/shadow_validation/crypto/event_driven/event_recs_candidate_16b.jsonl
   - Candidate B: live_capture/shadow_validation/crypto/event_driven/event_recs_candidate_32b.jsonl
   - Candidate C: live_capture/shadow_validation/crypto/event_driven/event_recs_candidate_64b.jsonl
   - Candidate D: live_capture/shadow_validation/crypto/event_driven/event_recs_candidate_120b.jsonl
6. Strictly 0.0% Order Authority: Read-only simulation. Canonical RP-608 remains untouched.
"""

from __future__ import annotations

import os
import sys
import json
import time
import math
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Optional, List, Dict, Any, Tuple
from collections import deque

WORKSPACE_DIR = Path(__file__).resolve().parent.parent
EVENT_DIR = WORKSPACE_DIR / "live_capture" / "shadow_validation" / "crypto" / "event_driven"

POLICY_FAMILY = "RP-608-EVENT-POLICY-V1"
CANDIDATE_CONFIGS = {
    16: {"id": "CANDIDATE_A_16B", "name": "16-Bar (4 Hours)", "bars": 16},
    32: {"id": "CANDIDATE_B_32B", "name": "32-Bar (8 Hours)", "bars": 32},
    64: {"id": "CANDIDATE_C_64B", "name": "64-Bar (16 Hours)", "bars": 64},
    120: {"id": "CANDIDATE_D_120B", "name": "120-Bar (30 Hours)", "bars": 120}
}

# Frozen Microstructure Thresholds
MAX_VALID_SPREAD_BPS = 5.0        # Quotes with spread > 5 bps rejected as anomalous
MAX_QUOTE_AGE_MS = 1000.0         # Stale quotes > 1000ms rejected
MIN_HISTORY_WARMUP_SEC = 5.0      # Minimum quote history required for fast velocity
MIN_REFRACTORY_MS = 1000.0        # Minimum 1.0 second between consecutive active recommendations

# Directional Microstructure Triggers
IMBALANCE_THRESHOLD = 0.30        # Top-of-book imbalance: +0.30 for LONG, -0.30 for SHORT
VELOCITY_5S_THRESHOLD_BPS = 1.0   # 5-second mid-price velocity in bps (+1.0 for LONG, -1.0 for SHORT)
VELOCITY_30S_CONFIRM_BPS = 0.0    # 30-second mid-price velocity confirmation

# Macro Candle Trend Filter
TREND_FILTER_BPS = 15.0           # Trend return gate: reject longs if macro trend < -15 bps; reject shorts if > +15 bps


def iso_utc_from_ms(ms: int) -> str:
    dt = datetime.fromtimestamp(ms / 1000.0, tz=timezone.utc)
    return dt.strftime("%Y-%m-%dT%H:%M:%S.") + f"{int(dt.microsecond / 1000):03d}Z"


@dataclass
class CompletedCandle:
    """Represents a validated completed 15-minute candle."""
    bar_idx: int
    open_time_ms: int
    close_time_ms: int
    open_price: float
    high_price: float
    low_price: float
    close_price: float
    volume: float = 0.0


@dataclass
class BookTickerQuote:
    """Represents a validated top-of-book Binance quote event."""
    update_id: int
    event_time_ms: int
    receive_time_ms: int
    bid_price: float
    bid_qty: float
    ask_price: float
    ask_qty: float
    symbol: str = "BTCUSDT"

    @property
    def mid_price(self) -> float:
        return (self.bid_price + self.ask_price) / 2.0

    @property
    def spread_bps(self) -> float:
        mid = self.mid_price
        if mid <= 0:
            return 0.0
        return ((self.ask_price - self.bid_price) / mid) * 10000.0

    @property
    def top_of_book_imbalance(self) -> float:
        denom = self.bid_qty + self.ask_qty
        if denom <= 0:
            return 0.0
        return (self.bid_qty - self.ask_qty) / denom

    @property
    def micro_price(self) -> float:
        denom = self.bid_qty + self.ask_qty
        if denom <= 0:
            return self.mid_price
        return (self.bid_price * self.ask_qty + self.ask_price * self.bid_qty) / denom

    def validate(self, prev_quote: Optional[BookTickerQuote] = None, max_age_ms: float = MAX_QUOTE_AGE_MS) -> Tuple[bool, str]:
        if self.bid_price <= 0 or self.ask_price <= 0:
            return False, "INVALID_PRICE_NON_POSITIVE"
        if self.ask_price <= self.bid_price:
            return False, "INVERTED_OR_CROSSED_SPREAD"
        if self.bid_qty <= 0 or self.ask_qty <= 0:
            return False, "INVALID_QUANTITY_NON_POSITIVE"
        if self.spread_bps > MAX_VALID_SPREAD_BPS:
            return False, f"EXCESSIVE_SPREAD_{self.spread_bps:.2f}_BPS"

        age_ms = self.receive_time_ms - self.event_time_ms
        if age_ms < -1000:
            return False, f"FUTURE_TIMESTAMP_{age_ms}MS"
        if age_ms > max_age_ms:
            return False, f"STALE_QUOTE_AGE_{age_ms}MS"

        if prev_quote is not None:
            if self.update_id < prev_quote.update_id:
                return False, f"OUT_OF_ORDER_UPDATE_ID_{self.update_id}_LT_{prev_quote.update_id}"
            if self.event_time_ms < prev_quote.event_time_ms:
                return False, f"OUT_OF_ORDER_EVENT_TIME_{self.event_time_ms}_LT_{prev_quote.event_time_ms}"

        return True, "VALID"


class CandleContextBuffer:
    """Maintains a rolling history of completed 15-minute candles up to 150 bars."""
    def __init__(self, max_bars: int = 150):
        self.max_bars = max_bars
        self.candles: deque[CompletedCandle] = deque(maxlen=max_bars)

    def append(self, candle: CompletedCandle):
        # Monotonicity check
        if self.candles and candle.close_time_ms <= self.candles[-1].close_time_ms:
            return
        self.candles.append(candle)

    def count(self) -> int:
        return len(self.candles)

    def is_warmed_up(self, lookback_bars: int) -> bool:
        return len(self.candles) >= lookback_bars

    def get_trend_return_bps(self, lookback_bars: int) -> Optional[float]:
        """Calculates return in bps from candle t-K close to latest candle close."""
        if len(self.candles) < lookback_bars:
            return None
        c_latest = self.candles[-1].close_price
        c_past = self.candles[-lookback_bars].close_price
        if c_past <= 0:
            return None
        return ((c_latest - c_past) / c_past) * 10000.0

    def get_regime(self, lookback_bars: int) -> str:
        ret = self.get_trend_return_bps(lookback_bars)
        if ret is None:
            return "UNWARMED"
        if ret > TREND_FILTER_BPS:
            return "BULLISH_TREND"
        elif ret < -TREND_FILTER_BPS:
            return "BEARISH_TREND"
        return "RANGING_NEUTRAL"


class MicroFeatureBuffer:
    """Maintains rolling buffer of quotes for fast microstructure features."""
    def __init__(self, window_sec: float = 65.0):
        self.window_sec = window_sec
        self.buffer: deque[BookTickerQuote] = deque()

    def append(self, quote: BookTickerQuote):
        self.buffer.append(quote)
        cutoff_ms = quote.event_time_ms - int(self.window_sec * 1000)
        while self.buffer and self.buffer[0].event_time_ms < cutoff_ms:
            self.buffer.popleft()

    def get_history_duration_sec(self) -> float:
        if len(self.buffer) < 2:
            return 0.0
        return (self.buffer[-1].event_time_ms - self.buffer[0].event_time_ms) / 1000.0

    def compute_velocity(self, target_lookback_sec: float) -> Optional[float]:
        if not self.buffer:
            return None
        latest = self.buffer[-1]
        target_time_ms = latest.event_time_ms - int(target_lookback_sec * 1000)

        if self.buffer[0].event_time_ms > target_time_ms:
            return None

        historical_quote = None
        for q in self.buffer:
            if q.event_time_ms >= target_time_ms:
                historical_quote = q
                break

        if historical_quote is None or historical_quote.mid_price <= 0:
            return None

        return ((latest.mid_price - historical_quote.mid_price) / historical_quote.mid_price) * 10000.0

    def compute_tick_volatility_bps(self) -> float:
        if len(self.buffer) < 5:
            return 0.0

        returns = []
        buf_list = list(self.buffer)
        for i in range(1, len(buf_list)):
            p0 = buf_list[i - 1].mid_price
            p1 = buf_list[i].mid_price
            if p0 > 0:
                ret = ((p1 - p0) / p0) * 10000.0
                returns.append(ret)

        if len(returns) < 2:
            return 0.0

        mean = sum(returns) / len(returns)
        var = sum((r - mean) ** 2 for r in returns) / (len(returns) - 1)
        return math.sqrt(var)


class CandidateLookbackEngine:
    """
    Independent recommendation engine configured for a specific candle lookback window
    (16, 32, 64, or 120 bars). Evaluates live quotes in the context of its own macro trend.
    """
    def __init__(self, lookback_bars: int, ledger_path: Optional[Path] = None, candle_buf: Optional[CandleContextBuffer] = None):
        self.lookback_bars = lookback_bars
        config = CANDIDATE_CONFIGS.get(lookback_bars, {"id": f"CANDIDATE_{lookback_bars}B", "name": f"{lookback_bars}-Bar"})
        self.candidate_id = config["id"]
        self.candidate_name = config["name"]
        self.policy_id = f"{POLICY_FAMILY}-{self.candidate_id}"

        default_file = EVENT_DIR / f"event_recs_candidate_{lookback_bars}b.jsonl"
        self.ledger_path = Path(ledger_path) if ledger_path else default_file
        self.ledger_path.parent.mkdir(parents=True, exist_ok=True)

        self.candle_buf = candle_buf if candle_buf is not None else CandleContextBuffer()
        self.feature_buf = MicroFeatureBuffer(window_sec=65.0)

        self.last_valid_quote: Optional[BookTickerQuote] = None
        self.last_emitted_rec: Optional[Dict[str, Any]] = None
        self.last_active_signal_ms: int = 0
        self.event_counter: int = 0
        self.config_hash = self.compute_config_hash()

        self.stats = {
            "quotes_received": 0,
            "quotes_valid": 0,
            "quotes_rejected": 0,
            "recs_emitted": 0,
            "recs_rate_limited": 0,
            "recs_deduplicated": 0,
            "recs_counter_trend_filtered": 0,
            "stream_reconnect_count": 0
        }

    def compute_config_hash(self) -> str:
        """Computes deterministic sha256 hash of frozen candidate configuration."""
        import hashlib
        cfg = {
            "lookback_bars": self.lookback_bars,
            "candidate_id": self.candidate_id,
            "policy_id": self.policy_id,
            "max_spread_bps": MAX_VALID_SPREAD_BPS,
            "max_quote_age_ms": MAX_QUOTE_AGE_MS,
            "min_warmup_sec": MIN_HISTORY_WARMUP_SEC,
            "min_refractory_ms": MIN_REFRACTORY_MS,
            "imbalance_threshold": IMBALANCE_THRESHOLD,
            "velocity_5s_threshold_bps": VELOCITY_5S_THRESHOLD_BPS,
            "velocity_30s_confirm_bps": VELOCITY_30S_CONFIRM_BPS,
            "trend_filter_bps": TREND_FILTER_BPS,
            "order_authority": 0.0
        }
        return hashlib.sha256(json.dumps(cfg, sort_keys=True).encode("utf-8")).hexdigest()

    def handle_reconnect(self, reason: str = "STREAM_RECONNECT"):
        """
        Handles feed disconnect/reconnect and sequence reset:
        1. Resets last_valid_quote baseline so update-ID resets don't trigger false out-of-order rejection.
        2. Clears fast microstructure buffer to mandate fresh 5.0-second warmup before emitting any new signals.
        3. Preserves candle buffer and historical context.
        """
        self.last_valid_quote = None
        self.feature_buf = MicroFeatureBuffer(window_sec=65.0)
        self.stats["stream_reconnect_count"] += 1

    def evaluate_quote(self, quote: BookTickerQuote, now_ms: int) -> Tuple[str, Dict[str, Any]]:
        """Evaluates decision logic combining fast microstructure with candidate candle trend."""
        spread = quote.spread_bps
        imbalance = quote.top_of_book_imbalance
        micro_price = quote.micro_price
        mid_price = quote.mid_price
        hist_dur = self.feature_buf.get_history_duration_sec()

        # Fast warmup check
        if hist_dur < MIN_HISTORY_WARMUP_SEC:
            return "NO_TRADE", {
                "decision": "NO_TRADE", "direction": "NEUTRAL", "strength": 0.0,
                "rationale": f"INSUFFICIENT_WARMUP_{hist_dur:.1f}S_LT_{MIN_HISTORY_WARMUP_SEC}S",
                "features": {"spread_bps": round(spread, 4), "imbalance": round(imbalance, 4), "history_sec": round(hist_dur, 2)}
            }

        # Velocity check
        v5 = self.feature_buf.compute_velocity(5.0)
        v30 = self.feature_buf.compute_velocity(30.0)
        vol_bps = self.feature_buf.compute_tick_volatility_bps()

        if v5 is None:
            return "NO_TRADE", {
                "decision": "NO_TRADE", "direction": "NEUTRAL", "strength": 0.0,
                "rationale": "MISSING_5S_VELOCITY",
                "features": {"spread_bps": round(spread, 4), "imbalance": round(imbalance, 4), "history_sec": round(hist_dur, 2)}
            }

        effective_v30 = v30 if v30 is not None else 0.0

        # Candidate Macro Trend Context
        candle_trend_bps = self.candle_buf.get_trend_return_bps(self.lookback_bars)
        regime = self.candle_buf.get_regime(self.lookback_bars)
        candles_available = self.candle_buf.count()

        features = {
            "candidate_lookback_bars": self.lookback_bars,
            "candles_available": candles_available,
            "candle_trend_bps": round(candle_trend_bps, 2) if candle_trend_bps is not None else None,
            "candle_regime": regime,
            "spread_bps": round(spread, 4),
            "imbalance": round(imbalance, 4),
            "micro_price": round(micro_price, 2),
            "velocity_5s_bps": round(v5, 4),
            "velocity_30s_bps": round(v30, 4) if v30 is not None else None,
            "tick_volatility_bps": round(vol_bps, 4),
            "history_sec": round(hist_dur, 2)
        }

        # Directional Criteria
        is_long_micro = (imbalance >= IMBALANCE_THRESHOLD) and (v5 >= VELOCITY_5S_THRESHOLD_BPS) and (effective_v30 >= VELOCITY_30S_CONFIRM_BPS) and (micro_price >= mid_price)
        is_short_micro = (imbalance <= -IMBALANCE_THRESHOLD) and (v5 <= -VELOCITY_5S_THRESHOLD_BPS) and (effective_v30 <= -VELOCITY_30S_CONFIRM_BPS) and (micro_price <= mid_price)

        if is_long_micro:
            # Check candidate macro filter: do not go long if candidate's lookback trend is strongly bearish
            if candle_trend_bps is not None and candle_trend_bps < -TREND_FILTER_BPS:
                self.stats["recs_counter_trend_filtered"] += 1
                return "NO_TRADE", {
                    "decision": "NO_TRADE", "direction": "NEUTRAL", "strength": 0.0,
                    "rationale": f"COUNTER_TREND_FILTER_K{self.lookback_bars}_BEARISH_{candle_trend_bps:.1f}BPS",
                    "features": features
                }
            
            # Boost strength if macro trend is concordant
            boost = 0.2 if (candle_trend_bps is not None and candle_trend_bps > TREND_FILTER_BPS) else 0.0
            strength = min(1.0, (imbalance / 0.8) * 0.4 + (v5 / 3.0) * 0.4 + boost)
            return "RECOMMEND_LONG", {
                "decision": "RECOMMEND_LONG", "direction": "LONG", "strength": round(strength, 4),
                "rationale": f"CONCORDANT_BUY_IMBALANCE_AND_VELOCITY_WITH_K{self.lookback_bars}_CONTEXT",
                "features": features
            }

        if is_short_micro:
            # Check candidate macro filter: do not go short if candidate's lookback trend is strongly bullish
            if candle_trend_bps is not None and candle_trend_bps > TREND_FILTER_BPS:
                self.stats["recs_counter_trend_filtered"] += 1
                return "NO_TRADE", {
                    "decision": "NO_TRADE", "direction": "NEUTRAL", "strength": 0.0,
                    "rationale": f"COUNTER_TREND_FILTER_K{self.lookback_bars}_BULLISH_{candle_trend_bps:.1f}BPS",
                    "features": features
                }

            boost = 0.2 if (candle_trend_bps is not None and candle_trend_bps < -TREND_FILTER_BPS) else 0.0
            strength = min(1.0, (abs(imbalance) / 0.8) * 0.4 + (abs(v5) / 3.0) * 0.4 + boost)
            return "RECOMMEND_SHORT", {
                "decision": "RECOMMEND_SHORT", "direction": "SHORT", "strength": round(strength, 4),
                "rationale": f"CONCORDANT_SELL_IMBALANCE_AND_VELOCITY_WITH_K{self.lookback_bars}_CONTEXT",
                "features": features
            }

        # Sub-threshold / conflict reasons
        if is_long_micro and v5 < 0:
            rationale = "CONFLICT_BUY_IMBALANCE_BUT_NEGATIVE_5S_VELOCITY"
        elif is_short_micro and v5 > 0:
            rationale = "CONFLICT_SELL_IMBALANCE_BUT_POSITIVE_5S_VELOCITY"
        elif abs(imbalance) < IMBALANCE_THRESHOLD and abs(v5) >= VELOCITY_5S_THRESHOLD_BPS:
            rationale = "SUB_THRESHOLD_IMBALANCE"
        elif abs(imbalance) >= IMBALANCE_THRESHOLD and abs(v5) < VELOCITY_5S_THRESHOLD_BPS:
            rationale = "SUB_THRESHOLD_VELOCITY"
        else:
            rationale = "MARKET_EQUILIBRIUM_NEUTRAL"

        return "NO_TRADE", {
            "decision": "NO_TRADE", "direction": "NEUTRAL", "strength": 0.0,
            "rationale": rationale, "features": features
        }

    def process_raw_dict(self, raw_data: Dict[str, Any], receive_time_ms: Optional[int] = None) -> Optional[Dict[str, Any]]:
        self.stats["quotes_received"] += 1
        now_ms = receive_time_ms if receive_time_ms is not None else int(time.time() * 1000)

        try:
            update_id = int(raw_data.get("u", 0))
            event_time_ms = int(raw_data.get("E", raw_data.get("T", raw_data.get("local_wallclock_utc_ms", now_ms))))
            bid_p = float(raw_data["b"])
            bid_q = float(raw_data["B"])
            ask_p = float(raw_data["a"])
            ask_q = float(raw_data["A"])
            symbol = raw_data.get("s", "BTCUSDT")

            quote = BookTickerQuote(
                update_id=update_id, event_time_ms=event_time_ms, receive_time_ms=now_ms,
                bid_price=bid_p, bid_qty=bid_q, ask_price=ask_p, ask_qty=ask_q, symbol=symbol
            )
        except (KeyError, ValueError, TypeError):
            self.stats["quotes_rejected"] += 1
            return None

        is_valid, reason = quote.validate(self.last_valid_quote)
        if not is_valid:
            self.stats["quotes_rejected"] += 1
            return None

        self.stats["quotes_valid"] += 1
        self.last_valid_quote = quote
        self.feature_buf.append(quote)

        # Evaluate policy
        decision, eval_res = self.evaluate_quote(quote, now_ms)
        direction = eval_res["direction"]

        # Invariant: Only emit and log ACTIVE RECOMMENDATIONS (LONG / SHORT)
        # Suppress NO_TRADE spam from flooding the recommendation ledger
        if direction not in ("LONG", "SHORT"):
            return None

        # 1.0-Second Minimum Refractory Period
        time_since_active = now_ms - self.last_active_signal_ms
        if time_since_active < MIN_REFRACTORY_MS:
            self.stats["recs_rate_limited"] += 1
            return None

        # Deduplication: Suppress repeated consecutive signals within 5s if price moved < 1 tick ($0.05)
        if self.last_emitted_rec is not None and self.last_emitted_rec["recommendation"]["direction"] == direction:
            prev_p = self.last_emitted_rec["quote"]["mid_price"]
            if abs(quote.mid_price - prev_p) < 0.05 and time_since_active < 5000:
                self.stats["recs_deduplicated"] += 1
                return None

        # Valid active recommendation emitted
        self.event_counter += 1
        event_id = f"EVT-K{self.lookback_bars:03d}-{datetime.fromtimestamp(now_ms/1000, tz=timezone.utc).strftime('%Y%m%d-%H%M%S')}-{self.event_counter:05d}"

        record = {
            "event_id": event_id,
            "policy_id": self.policy_id,
            "candidate_id": self.candidate_id,
            "candidate_lookback_bars": self.lookback_bars,
            "config_hash": self.config_hash,
            "symbol": quote.symbol,
            "binance_update_id": quote.update_id,
            "timestamp_utc": iso_utc_from_ms(quote.event_time_ms),
            "timestamp_epoch_ms": quote.event_time_ms,
            "receive_timestamp_epoch_ms": quote.receive_time_ms,
            "quote": {
                "bid_price": quote.bid_price,
                "ask_price": quote.ask_price,
                "mid_price": round(quote.mid_price, 2),
                "bid_qty": quote.bid_qty,
                "ask_qty": quote.ask_qty,
                "spread_bps": round(quote.spread_bps, 4)
            },
            "features": eval_res["features"],
            "recommendation": {
                "decision": eval_res["decision"],
                "direction": eval_res["direction"],
                "strength": eval_res["strength"],
                "rationale": eval_res["rationale"],
                "suggested_entry_price": quote.ask_price if direction == "LONG" else quote.bid_price,
                "invalidation_price": round(quote.mid_price * 0.997 if direction == "LONG" else quote.mid_price * 1.003, 2)
            },
            "order_authority": 0.0
        }

        # Write to candidate's isolated ledger
        with open(self.ledger_path, "a", encoding="utf-8") as f:
            f.write(json.dumps(record, separators=(",", ":")) + "\n")

        self.last_emitted_rec = record
        self.last_active_signal_ms = now_ms
        self.stats["recs_emitted"] += 1

        return record


class MultiLookbackCoordinator:
    """
    Coordinates ingestion across all four candidate lookback engines in parallel.
    Maintains shared candle buffer and routes quote events to each candidate.
    """
    def __init__(self, base_dir: Optional[Path] = None):
        self.base_dir = Path(base_dir) if base_dir else EVENT_DIR
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.candle_buf = CandleContextBuffer(max_bars=150)

        # Initialize the 4 independent candidate engines
        self.engines: Dict[int, CandidateLookbackEngine] = {}
        for bars in (16, 32, 64, 120):
            ledger = self.base_dir / f"event_recs_candidate_{bars}b.jsonl"
            self.engines[bars] = CandidateLookbackEngine(lookback_bars=bars, ledger_path=ledger, candle_buf=self.candle_buf)

    def ingest_completed_candle(self, candle: CompletedCandle):
        """Updates rolling candle buffer shared across candidate engines."""
        self.candle_buf.append(candle)

    def process_quote_event(self, raw_quote: Dict[str, Any], receive_time_ms: Optional[int] = None) -> Dict[int, Optional[Dict[str, Any]]]:
        """Routes a single quote event across all 4 candidate engines in parallel."""
        results = {}
        for bars, engine in self.engines.items():
            res = engine.process_raw_dict(raw_quote, receive_time_ms=receive_time_ms)
            results[bars] = res
        return results

    def handle_reconnect(self, reason: str = "STREAM_RECONNECT"):
        """Broadcasts stream reconnect reset to all candidate engines."""
        for engine in self.engines.values():
            engine.handle_reconnect(reason=reason)

    def get_summary_stats(self) -> Dict[str, Any]:
        return {
            "candle_count": self.candle_buf.count(),
            "candidates": {
                bars: {
                    "candidate_id": eng.candidate_id,
                    "warmed_up": self.candle_buf.is_warmed_up(bars),
                    "stats": eng.stats
                }
                for bars, eng in self.engines.items()
            }
        }


# Backwards compatibility alias for single-policy harness
EventEngine = CandidateLookbackEngine
POLICY_ID = f"{POLICY_FAMILY}-CANDIDATE_A_16B"


def main():
    print(f"[{POLICY_FAMILY}] Multi-Lookback Event Engine Loaded Cleanly.")
    print("  Candidate Engines:")
    for bars, cfg in CANDIDATE_CONFIGS.items():
        print(f"    - {cfg['id']}: {cfg['name']} ({bars} completed 15m candles)")
    print(f"  Max Valid Spread: {MAX_VALID_SPREAD_BPS} bps | Max Quote Age: {MAX_QUOTE_AGE_MS} ms")
    print(f"  Refractory Minimum: {MIN_REFRACTORY_MS} ms | Order Authority: 0.0 (Read-Only)")


if __name__ == "__main__":
    main()
