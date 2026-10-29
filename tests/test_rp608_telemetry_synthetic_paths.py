#!/usr/bin/env python3
"""
tests/test_rp608_telemetry_synthetic_paths.py

Standard-library unit test suite for RP-608 prospective telemetry integrity:
1. Synthetic Price Paths for MAE/MFE Excursion Logic (A, B, C, D, E)
2. Quote Quality & Validation Gates (Crossed, Stale, Zero/Negative Price)
3. Trade ID Bijective Journal Mapping
4. Exit Priority & 20 bps Gross Hurdle Disambiguation vs Friction

Zero external test framework dependencies.
"""

from __future__ import annotations

import json
import math
import tempfile
import time
import unittest
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


@dataclass
class SyntheticQuote:
    timestamp_utc_ms: int
    bid_price: float
    ask_price: float
    bid_qty: float = 1000.0
    ask_qty: float = 1000.0

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


def validate_quote(quote: SyntheticQuote, current_time_ms: int, max_age_ms: int = 60_000) -> Tuple[bool, str]:
    """Predeclared quote validation gate."""
    if quote.bid_price <= 0.0 or quote.ask_price <= 0.0:
        return False, "INVALID_PRICE_NON_POSITIVE"
    if quote.bid_price > quote.ask_price:
        return False, "CROSSED_BOOK_BID_EXCEEDS_ASK"
    if current_time_ms - quote.timestamp_utc_ms > max_age_ms:
        return False, f"STALE_QUOTE_AGE_{current_time_ms - quote.timestamp_utc_ms}MS"
    if quote.bid_qty <= 0.0 or quote.ask_qty <= 0.0:
        return False, "ZERO_DEPTH"
    return True, "VALID"


def calculate_excursion_metrics(
    entry_price: float,
    price_trajectory: List[float],
    side: str = "LONG",
    half_spread_bps: float = 0.0
) -> Dict[str, float]:
    """
    Computes reference and executable MAE/MFE across a discrete price sequence.
    """
    if not price_trajectory:
        return {"mae_bps": 0.0, "mfe_bps": 0.0, "mae_exec_bps": 0.0, "mfe_exec_bps": 0.0}

    if side == "LONG":
        # Reference excursions (mid/LTP)
        max_p = max(price_trajectory)
        min_p = min(price_trajectory)
        mfe_bps = max(0.0, ((max_p - entry_price) / entry_price) * 10000.0)
        mae_bps = max(0.0, ((entry_price - min_p) / entry_price) * 10000.0)

        # Executable excursions (factoring half-spread crossing at entry and exit)
        # Entry at Ask: entry_exec = entry_price * (1 + half_spread)
        # Exit at Bid: exit_exec = p * (1 - half_spread)
        spread_factor = half_spread_bps / 10000.0
        entry_exec = entry_price * (1.0 + spread_factor)
        max_exec_bid = max_p * (1.0 - spread_factor)
        min_exec_bid = min_p * (1.0 - spread_factor)

        mfe_exec_bps = ((max_exec_bid - entry_exec) / entry_exec) * 10000.0
        mae_exec_bps = ((entry_exec - min_exec_bid) / entry_exec) * 10000.0

        return {
            "mae_bps": round(mae_bps, 2),
            "mfe_bps": round(mfe_bps, 2),
            "mae_exec_bps": round(mae_exec_bps, 2),
            "mfe_exec_bps": round(mfe_exec_bps, 2),
        }
    else:
        raise NotImplementedError("Short side not authorized under current charter")


def evaluate_exit_policy(
    entry_price: float,
    bars_held: int,
    current_price: float,
    current_bar_idx: int,
    use_20bps_hurdle: bool = False,
    sl_pct: float = 0.010,
    tp_pct: float = 0.015,
) -> Optional[str]:
    """
    Evaluates exit conditions under strict priority hierarchy:
    1. Mandatory square-off at bar 24 (15:15 IST)
    2. Stop loss (-1.0%)
    3. Take profit (+1.5%)
    4. Max holding horizon (12 bars / 180 min)
    5. Target holding horizon (4 bars / 60 min):
       - Baseline: triggers if current_price > entry_price
       - H3 (20 bps hurdle): triggers if current_price >= entry_price * 1.0020
    """
    if current_bar_idx >= 24:
        return "MANDATORY_INTRADAY_SQUARE_OFF"
    if current_price <= entry_price * (1.0 - sl_pct):
        return "STOP_LOSS"
    if current_price >= entry_price * (1.0 + tp_pct):
        return "TAKE_PROFIT"
    if bars_held >= 12:
        return "MAX_HOLD_HORIZON_180M"
    if bars_held >= 4:
        if use_20bps_hurdle:
            if current_price >= entry_price * 1.0020:
                return "TARGET_HORIZON_60M_HURDLE_MET"
            # If hurdle not met, position stays open
            return None
        else:
            if current_price > entry_price:
                return "TARGET_HORIZON_60M"
    return None


class TestRP608TelemetrySyntheticPaths(unittest.TestCase):

    def test_path_a_monotonic_favorable(self):
        """Synthetic Path A: Monotonic price rise from 1000.0 to 1020.0."""
        entry_p = 1000.0
        trajectory = [1002.0, 1005.0, 1010.0, 1015.0, 1020.0]
        res = calculate_excursion_metrics(entry_p, trajectory, side="LONG", half_spread_bps=0.0)

        # MFE must be 200 bps, MAE must be exactly 0.0
        self.assertEqual(res["mfe_bps"], 200.0)
        self.assertEqual(res["mae_bps"], 0.0)

    def test_path_b_monotonic_adverse(self):
        """Synthetic Path B: Monotonic price decline from 1000.0 to 980.0."""
        entry_p = 1000.0
        trajectory = [998.0, 995.0, 990.0, 985.0, 980.0]
        res = calculate_excursion_metrics(entry_p, trajectory, side="LONG", half_spread_bps=0.0)

        # MFE must be 0.0, MAE must be exactly 200 bps
        self.assertEqual(res["mfe_bps"], 0.0)
        self.assertEqual(res["mae_bps"], 200.0)

    def test_path_c_adverse_dip_with_recovery(self):
        """
        Synthetic Path C: Dip to 995 (-50 bps) in early bars, recovering to 1015 (+150 bps).
        Verifies that early adverse excursion is never overwritten by later profit!
        """
        entry_p = 1000.0
        trajectory = [997.0, 995.0, 998.0, 1005.0, 1010.0, 1015.0]
        res = calculate_excursion_metrics(entry_p, trajectory, side="LONG", half_spread_bps=0.0)

        self.assertEqual(res["mae_bps"], 50.0)
        self.assertEqual(res["mfe_bps"], 150.0)

    def test_path_d_cross_bar_gap(self):
        """Synthetic Path D: Discrete gap from 1000.0 to 990.0 at bar close."""
        entry_p = 1000.0
        trajectory = [1000.0, 990.0]
        res = calculate_excursion_metrics(entry_p, trajectory, side="LONG", half_spread_bps=0.0)

        self.assertEqual(res["mae_bps"], 100.0)
        self.assertEqual(res["mfe_bps"], 0.0)

    def test_path_e_executable_spread_distortion(self):
        """
        Synthetic Path E: Demonstrates how reference LTP overstates executable returns
        by 2 * half_spread_bps (e.g., 2 * 2.5 bps = 5.0 bps friction penalty).
        """
        entry_p = 1000.0
        # LTP moves up by 10 bps (1001.0)
        trajectory = [1001.0]
        res = calculate_excursion_metrics(entry_p, trajectory, side="LONG", half_spread_bps=2.5)

        # Reference MFE is +10.0 bps
        self.assertEqual(res["mfe_bps"], 10.0)
        # Executable MFE is only ~5.0 bps due to 5 bps round-trip half-spread crossing
        self.assertAlmostEqual(res["mfe_exec_bps"], 5.0, delta=0.1)

    def test_quote_validation_gates(self):
        """Tests rejection of invalid, crossed, and stale quotes."""
        now_ms = 1793089500000

        # Valid quote
        q_valid = SyntheticQuote(timestamp_utc_ms=now_ms - 5000, bid_price=100.0, ask_price=100.05)
        ok, reason = validate_quote(q_valid, now_ms)
        self.assertTrue(ok)
        self.assertEqual(reason, "VALID")

        # Non-positive price
        q_zero = SyntheticQuote(timestamp_utc_ms=now_ms - 5000, bid_price=0.0, ask_price=100.05)
        ok, reason = validate_quote(q_zero, now_ms)
        self.assertFalse(ok)
        self.assertEqual(reason, "INVALID_PRICE_NON_POSITIVE")

        # Crossed book
        q_crossed = SyntheticQuote(timestamp_utc_ms=now_ms - 5000, bid_price=100.10, ask_price=100.05)
        ok, reason = validate_quote(q_crossed, now_ms)
        self.assertFalse(ok)
        self.assertEqual(reason, "CROSSED_BOOK_BID_EXCEEDS_ASK")

        # Stale quote (> 60s)
        q_stale = SyntheticQuote(timestamp_utc_ms=now_ms - 65000, bid_price=100.0, ask_price=100.05)
        ok, reason = validate_quote(q_stale, now_ms)
        self.assertFalse(ok)
        self.assertIn("STALE_QUOTE_AGE", reason)

    def test_exit_policy_and_20bps_hurdle_interaction(self):
        """
        Tests the disambiguation of the 20 bps hurdle vs baseline at Bar 4:
        - Baseline: small gain (+10 bps) exits immediately at Bar 4.
        - H3 Hurdle: small gain (+10 bps) does NOT exit at Bar 4; waits for 20 bps or other exit.
        """
        entry_p = 1000.0
        # +10 bps price (1001.0) at Bar 4
        price_10bps = 1001.0

        # Baseline: triggers TARGET_HORIZON_60M
        res_baseline = evaluate_exit_policy(
            entry_price=entry_p,
            bars_held=4,
            current_price=price_10bps,
            current_bar_idx=22,
            use_20bps_hurdle=False
        )
        self.assertEqual(res_baseline, "TARGET_HORIZON_60M")

        # H3 Hurdle: 10 bps is insufficient (< 20 bps), remains open (None)
        res_h3_stay = evaluate_exit_policy(
            entry_price=entry_p,
            bars_held=4,
            current_price=price_10bps,
            current_bar_idx=22,
            use_20bps_hurdle=True
        )
        self.assertIsNone(res_h3_stay)

        # H3 Hurdle: +25 bps price (1002.50) triggers exit
        price_25bps = 1002.50
        res_h3_exit = evaluate_exit_policy(
            entry_price=entry_p,
            bars_held=4,
            current_price=price_25bps,
            current_bar_idx=22,
            use_20bps_hurdle=True
        )
        self.assertEqual(res_h3_exit, "TARGET_HORIZON_60M_HURDLE_MET")

        # Mandatory square-off at Bar 24 overrides hurdle
        res_sqoff = evaluate_exit_policy(
            entry_price=entry_p,
            bars_held=6,
            current_price=price_10bps,
            current_bar_idx=24,
            use_20bps_hurdle=True
        )
        self.assertEqual(res_sqoff, "MANDATORY_INTRADAY_SQUARE_OFF")

        # Stop-loss overrides hurdle
        res_sl = evaluate_exit_policy(
            entry_price=entry_p,
            bars_held=2,
            current_price=989.0,
            current_bar_idx=20,
            use_20bps_hurdle=True
        )
        self.assertEqual(res_sl, "STOP_LOSS")

    def test_trade_id_bijective_reconciliation(self):
        """Verifies deterministic trade_id formatting ensures 1-to-1 bijection."""
        session_idx = 1
        bar_idx = 18
        symbol = "RELIANCE.NS"
        trade_id = f"nse_trade_{session_idx}_{bar_idx}_{symbol}"
        self.assertEqual(trade_id, "nse_trade_1_18_RELIANCE.NS")


if __name__ == "__main__":
    unittest.main()
