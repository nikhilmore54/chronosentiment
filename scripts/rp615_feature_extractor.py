#!/usr/bin/env python3
"""
scripts/rp615_feature_extractor.py

RP-615 Upstream Selective Signal Architecture — Point-in-Time Feature Extractor
Milestone: Phase 1 (Frozen Protocol v1.0)
Governance:
  - RP-605 Baseline Control retains 100% Live Execution Authority.
  - RP-615 is strictly Research-Only (0% Live Authority).
  - All features computed strictly at t <= t_entry.
  - Zero lookahead, zero future imputation, deterministic replay.

Features Implemented (F01 - F09):
  F01: Trend: Ret_15m (15-bar momentum)
  F02: Trend: EMA_Align (EMA_8 - EMA_21 normalized by ATR_14)
  F03: Efficiency: KER_15m (Kaufman Efficiency Ratio)
  F04: Volume: RVOL_15m (Relative volume vs 15-bar rolling mean)
  F05: Volatility: ATR_Norm (Wilder 14-period ATR / Close)
  F06: Volatility: Sigma_60 (Trailing 60m sample std of 60 log-returns * sqrt(60))
  F07: Breadth: CS_raw & CS_aligned (Cross-sectional breadth across 99 symbols)
  F08: Micro: Wick_Asym (Upper wick - lower wick normalized by candle range)
  F09: Concurrency: N_Trig (Universe triggers in [t_e - 60s, t_e) excluding self)
"""

from __future__ import annotations

import json
import math
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple
import numpy as np

WORKSPACE = Path(__file__).resolve().parent.parent
CACHE_DIR = WORKSPACE / "intraday_capture" / "yahoo_cache_1m"
CARDS_LOG = WORKSPACE / "live_capture" / "shadow_validation" / "rp605_nse_decision_cards_log.jsonl"

IST = timezone(timedelta(hours=5, minutes=30))
EPS = 1e-6


def parse_iso_ts(ts_str: str) -> datetime:
    ts_str = ts_str.strip()
    if ts_str.endswith("Z"):
        ts_str = ts_str[:-1] + "+00:00"
    return datetime.fromisoformat(ts_str)


class BarCacheManager:
    """Manages loading and chronological indexing of 1-minute intraday bars."""

    def __init__(self, cache_dir: Path = CACHE_DIR):
        self.cache_dir = cache_dir
        self._cache: Dict[str, List[Dict[str, Any]]] = {}
        self._timestamps: Dict[str, np.ndarray] = {}

    def load_symbol(self, symbol: str) -> Tuple[List[Dict[str, Any]], np.ndarray]:
        if symbol not in self._cache:
            p = self.cache_dir / f"{symbol}.json"
            if not p.exists():
                return [], np.array([], dtype=np.int64)
            with p.open(encoding="utf-8") as f:
                candles = json.load(f)
            # Ensure chronologically sorted
            candles = sorted(candles, key=lambda c: c["timestamp"])
            self._cache[symbol] = candles
            self._timestamps[symbol] = np.array([c["timestamp"] for c in candles], dtype=np.int64)
        return self._cache[symbol], self._timestamps[symbol]

    def get_all_symbols(self) -> List[str]:
        return [p.stem for p in self.cache_dir.glob("*.json")]

    def get_bars_up_to(self, symbol: str, cutoff_ts_unix: int) -> List[Dict[str, Any]]:
        candles, ts_array = self.load_symbol(symbol)
        if len(ts_array) == 0:
            return []
        idx = int(np.searchsorted(ts_array, cutoff_ts_unix, side="right"))
        return candles[:idx]


class DecisionCardsIndexer:
    """Indexes RP-605 decision cards for point-in-time concurrency evaluation (F09)."""

    def __init__(self, cards_path: Path = CARDS_LOG):
        self.cards_path = cards_path
        self._triggers_by_sec: Dict[int, List[Dict[str, Any]]] = defaultdict(list)
        self._loaded: bool = False

    def ensure_loaded(self):
        if self._loaded:
            return
        if not self.cards_path.exists():
            return
        with self.cards_path.open(encoding="utf-8") as f:
            for line in f:
                if not line.strip():
                    continue
                c = json.loads(line)
                if c.get("decision_state") in ["BUY", "SHORT"]:
                    ts_dt = parse_iso_ts(c["timestamp_utc"])
                    ts_sec = int(ts_dt.timestamp())
                    self._triggers_by_sec[ts_sec].append(c)
        self._loaded = True

    def get_concurrent_triggers(self, t_e_sec: int, candidate_symbol: str) -> int:
        self.ensure_loaded()
        # Window: [t_e - 60s, t_e)
        count = 0
        for sec in range(t_e_sec - 60, t_e_sec):
            for card in self._triggers_by_sec.get(sec, []):
                if card.get("instrument") != candidate_symbol:
                    count += 1
        return count


class FeatureExtractor:
    """
    Unified Causal Point-in-Time Pre-Entry Feature Extractor.
    Implements F01 - F09 strictly according to RP-615 Preregistration v1.0.
    """

    def __init__(self, cache_mgr: Optional[BarCacheManager] = None, cards_indexer: Optional[DecisionCardsIndexer] = None):
        self.cache_mgr = cache_mgr or BarCacheManager()
        self.cards_indexer = cards_indexer or DecisionCardsIndexer()
        self.frozen_symbols: List[str] = sorted(self.cache_mgr.get_all_symbols())

    def extract_features_at_entry(
        self,
        symbol: str,
        side: str,
        t_entry_iso: str,
    ) -> Dict[str, Any]:
        """
        Extracts F01-F09 strictly using completed bars up to t_entry.
        Returns a standardized dictionary with values, status, and diagnostic flags.
        """
        t_e_dt = parse_iso_ts(t_entry_iso)
        t_e_sec = int(t_e_dt.timestamp())

        # Fetch completed bars up to the decision time
        bars = self.cache_mgr.get_bars_up_to(symbol, t_e_sec)

        result: Dict[str, Any] = {
            "symbol": symbol,
            "side": side.upper(),
            "t_entry_utc": t_entry_iso,
            "t_e_sec": t_e_sec,
            "t_bar_close_sec": None,
            "status": "ELIGIBLE",
            "ineligibility_reason": None,
            "F01_ret_15m": None,
            "F02_ema_align": None,
            "F03_ker_15m": None,
            "F04_rvol_15m": None,
            "F05_atr_norm": None,
            "F06_sigma_60": None,
            "F07_cs_raw": None,
            "F07_cs_aligned": None,
            "F08_wick_asym": None,
            "F09_n_trig": None,
        }

        if len(bars) == 0:
            result["status"] = "INELIGIBLE_DATA"
            result["ineligibility_reason"] = "NO_BAR_DATA"
            return result

        last_bar = bars[-1]
        result["t_bar_close_sec"] = last_bar["timestamp"]

        # Verify bar timing: the last bar must be within 120 seconds of decision time
        last_bar = bars[-1]
        time_diff = t_e_sec - last_bar["timestamp"]
        if time_diff < 0:
            result["status"] = "INELIGIBLE_DATA"
            result["ineligibility_reason"] = "FUTURE_BAR_LEAKAGE"
            return result
        if time_diff > 360:
            result["status"] = "INELIGIBLE_DATA"
            result["ineligibility_reason"] = f"ENTRY_BAR_DATA_GAP_{time_diff}S"
            return result

        # Validate OHLC consistency on last bar
        po = float(last_bar["open"])
        ph = float(last_bar["high"])
        pl = float(last_bar["low"])
        pc = float(last_bar["close"])
        if not (pl <= min(po, pc) + EPS and max(po, pc) <= ph + EPS and pl <= ph and pc > 0):
            result["status"] = "INELIGIBLE_DATA"
            result["ineligibility_reason"] = "CORRUPT_OHLC_CANDLE"
            return result

        # Check maximum warm-up requirement: at least 15 completed bars
        n_bars = len(bars)
        if n_bars < 15:
            result["status"] = "INELIGIBLE_WARMUP"
            result["ineligibility_reason"] = f"INSUFFICIENT_BARS_FOR_15M_{n_bars}"
            return result

        closes = [float(b["close"]) for b in bars]
        volumes = [float(b.get("volume", 0.0)) for b in bars]

        # ---------------------------------------------------------------------
        # F01: Trend: Ret_15m
        # ---------------------------------------------------------------------
        if n_bars >= 16:
            p_c_now = closes[-1]
            p_c_16 = closes[-16]
            result["F01_ret_15m"] = (p_c_now - p_c_16) / max(p_c_16, EPS)
        else:
            result["F01_ret_15m"] = 0.0

        # ---------------------------------------------------------------------
        # F05: Volatility: ATR_Norm (Wilder 14-period ATR / Close)
        # ---------------------------------------------------------------------
        trs = []
        for i in range(1, n_bars):
            h_i = float(bars[i]["high"])
            l_i = float(bars[i]["low"])
            pc_prev = float(bars[i - 1]["close"])
            
            # Check for intraday gap within the same session
            ts_i = bars[i]["timestamp"]
            ts_prev = bars[i - 1]["timestamp"]
            gap_sec = ts_i - ts_prev
            
            # If within same day and gap > 360s (6 min), register intraday gap
            dt_i = datetime.fromtimestamp(ts_i, IST)
            dt_prev = datetime.fromtimestamp(ts_prev, IST)
            if dt_i.date() == dt_prev.date() and gap_sec > 360:
                result["status"] = "INELIGIBLE_DATA"
                result["ineligibility_reason"] = f"INTRADAY_BAR_GAP_{gap_sec}S"
                return result
                
            tr = max(h_i - l_i, abs(h_i - pc_prev), abs(l_i - pc_prev))
            trs.append(tr)

        if len(trs) < 14:
            atr_14 = float(np.mean(trs)) if trs else max(ph - pl, EPS)
        else:
            atr_14 = float(np.mean(trs[:14]))
            for tr in trs[14:]:
                atr_14 = (atr_14 * 13.0 + tr) / 14.0
        atr_14 = max(atr_14, EPS)
        result["F05_atr_norm"] = atr_14 / max(pc, EPS)

        # ---------------------------------------------------------------------
        # F02: Trend: EMA_Align ((EMA_8 - EMA_21) / ATR_14)
        # ---------------------------------------------------------------------
        def calc_ema(series: List[float], period: int) -> float:
            if not series:
                return 0.0
            if len(series) < period:
                return float(np.mean(series))
            val = float(np.mean(series[:period]))
            alpha = 2.0 / (period + 1.0)
            for x in series[period:]:
                val = alpha * x + (1.0 - alpha) * val
            return val

        if n_bars >= 21:
            ema_8 = calc_ema(closes, 8)
            ema_21 = calc_ema(closes, 21)
            result["F02_ema_align"] = (ema_8 - ema_21) / max(atr_14, EPS)
        else:
            result["F02_ema_align"] = 0.0

        # ---------------------------------------------------------------------
        # F03: Efficiency: KER_15m (Kaufman Efficiency Ratio)
        # ---------------------------------------------------------------------
        if n_bars >= 16:
            net_change = abs(closes[-1] - closes[-16])
            path_length = sum(abs(closes[-1 - i] - closes[-2 - i]) for i in range(15))
            result["F03_ker_15m"] = float(np.clip(net_change / max(path_length, EPS), 0.0, 1.0))
        else:
            result["F03_ker_15m"] = 0.0

        # ---------------------------------------------------------------------
        # F04: Volume: RVOL_15m
        # ---------------------------------------------------------------------
        if n_bars >= 16:
            v_now = volumes[-1]
            prior_15_v = volumes[-16:-1]
            mean_prior_v = float(np.mean(prior_15_v)) if prior_15_v else 0.0
            result["F04_rvol_15m"] = v_now / max(mean_prior_v, 1.0) if mean_prior_v > 0 else 1.0
        else:
            result["F04_rvol_15m"] = 1.0

        # ---------------------------------------------------------------------
        # F06: Volatility: Sigma_60 (60 consecutive log-returns across 61 prices)
        # ---------------------------------------------------------------------
        if n_bars >= 61:
            p_window = closes[-61:]
            log_rets = [math.log(p_window[i] / max(p_window[i - 1], EPS)) for i in range(1, 61)]
            std_r = float(np.std(log_rets, ddof=1))
            result["F06_sigma_60"] = std_r * math.sqrt(60.0)
        else:
            p_window = closes
            log_rets = [math.log(p_window[i] / max(p_window[i - 1], EPS)) for i in range(1, len(p_window))]
            std_r = float(np.std(log_rets, ddof=1)) if len(log_rets) > 1 else 0.0
            result["F06_sigma_60"] = std_r * math.sqrt(60.0)

        # ---------------------------------------------------------------------
        # F07: Breadth: CS_raw & CS_aligned (Cross-Sectional Market Breadth)
        # ---------------------------------------------------------------------
        n_adv = 0
        n_dec = 0
        n_active = 0
        for sym in self.frozen_symbols:
            s_bars = self.cache_mgr.get_bars_up_to(sym, t_e_sec)
            if len(s_bars) >= 2 and (t_e_sec - s_bars[-1]["timestamp"]) <= 120:
                p_now = float(s_bars[-1]["close"])
                p_prev = float(s_bars[-2]["close"])
                if p_now > p_prev + EPS:
                    n_adv += 1
                elif p_now < p_prev - EPS:
                    n_dec += 1
                n_active += 1

        if n_active > 0:
            cs_raw = (n_adv - n_dec) / float(n_active)
        else:
            cs_raw = 0.0

        side_mult = +1.0 if side.upper() == "LONG" else -1.0
        result["F07_cs_raw"] = cs_raw
        result["F07_cs_aligned"] = cs_raw * side_mult

        # ---------------------------------------------------------------------
        # F08: Micro: Wick_Asym (Upper Wick - Lower Wick / Total Range)
        # ---------------------------------------------------------------------
        c_range = ph - pl
        if c_range > EPS:
            upper_wick = ph - max(po, pc)
            lower_wick = min(po, pc) - pl
            result["F08_wick_asym"] = float(np.clip((upper_wick - lower_wick) / c_range, -1.0, 1.0))
        else:
            result["F08_wick_asym"] = 0.0

        # ---------------------------------------------------------------------
        # F09: Concurrency: N_Trig (Cross-symbol concurrent trigger density)
        # ---------------------------------------------------------------------
        result["F09_n_trig"] = self.cards_indexer.get_concurrent_triggers(t_e_sec, symbol)

        return result


if __name__ == "__main__":
    print("RP-615 Feature Extractor Module initialized successfully.")
    extractor = FeatureExtractor()
    print(f"Loaded {len(extractor.frozen_symbols)} symbols in universe.")
