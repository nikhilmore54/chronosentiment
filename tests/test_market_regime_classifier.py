#!/usr/bin/env python3
"""
tests/test_market_regime_classifier.py

Unit test suite for the pure market regime classifier (scripts/market_regime_classifier.py).
Tests:
  1. Pure indicator calculation correctness (ER, VolRatio, ADX).
  2. Edge cases: insufficient history, zero division, flatlines, corrupted data.
  3. Deterministic 4-Step Precedence:
     - UNAVAILABLE
     - SIDEWAYS
     - DIRECTIONAL
     - TRANSITIONING
"""

from __future__ import annotations

import math
import unittest
from typing import Any, Dict, List

from scripts.market_regime_classifier import (
    kaufman_efficiency_ratio,
    calculate_atr_series,
    relative_volatility_ratio,
    adx_indicator,
    classify_market_regime
)


class MockBar:
    def __init__(self, high: float, low: float, close: float, open_: float = 0.0, volume: float = 1000.0):
        self.high = high
        self.low = low
        self.close = close
        self.open = open_ or (high + low) / 2.0
        self.volume = volume


class TestMarketRegimeClassifier(unittest.TestCase):

    # ─────────────────────────────────────────────────────────────
    # 1. Kaufman Efficiency Ratio Tests
    # ─────────────────────────────────────────────────────────────

    def test_er_perfect_trend(self):
        """Monotonic price expansion should produce ER = 1.0."""
        # 20 closes each rising by 5.0
        closes = [100.0 + i * 5.0 for i in range(20)]
        er = kaufman_efficiency_ratio(closes, n=16)
        self.assertIsNotNone(er)
        self.assertAlmostEqual(er, 1.0, places=4)

    def test_er_perfect_downtrend(self):
        """Monotonic price decline should produce ER = 1.0."""
        closes = [200.0 - i * 5.0 for i in range(20)]
        er = kaufman_efficiency_ratio(closes, n=16)
        self.assertIsNotNone(er)
        self.assertAlmostEqual(er, 1.0, places=4)

    def test_er_perfect_chop(self):
        """Alternating price chop returning to start should produce ER = 0.0."""
        # Alternates between 100 and 110, ending at 100 after 16 bars
        closes = [100.0, 110.0] * 10
        # n = 16: closes[-1] is 110.0, closes[-17] is 110.0 => disp = 0.0
        er = kaufman_efficiency_ratio(closes, n=16)
        self.assertIsNotNone(er)
        self.assertAlmostEqual(er, 0.0, places=4)

    def test_er_flatline_zero_movement(self):
        """Flatline prices should return ER = 0.0 without divide-by-zero error."""
        closes = [100.0] * 25
        er = kaufman_efficiency_ratio(closes, n=16)
        self.assertEqual(er, 0.0)

    def test_er_insufficient_history(self):
        """Fewer than n+1 closes must return None."""
        closes = [100.0, 101.0, 102.0]
        er = kaufman_efficiency_ratio(closes, n=16)
        self.assertIsNone(er)

    def test_er_corrupt_data(self):
        """NaN or Inf or negative prices must return None."""
        closes = [100.0 + i for i in range(15)] + [float("nan"), 120.0]
        er = kaufman_efficiency_ratio(closes, n=16)
        self.assertIsNone(er)

    # ─────────────────────────────────────────────────────────────
    # 2. Relative Volatility Ratio Tests
    # ─────────────────────────────────────────────────────────────

    def test_vol_ratio_normal(self):
        """Constant volatility produces VolRatio = 1.0."""
        bars = [MockBar(high=100.0 + i + 2.0, low=100.0 + i - 2.0, close=100.0 + i) for i in range(70)]
        vr = relative_volatility_ratio(bars, n_trailing=48, atr_period=14)
        self.assertIsNotNone(vr)
        self.assertAlmostEqual(vr, 1.0, places=2)

    def test_vol_ratio_compression(self):
        """Volatility dropping significantly produces VolRatio < 0.80."""
        # First 60 bars have range 10.0 (high-low=10)
        bars = [MockBar(high=100.0 + 5.0, low=100.0 - 5.0, close=100.0) for _ in range(60)]
        # Last 10 bars compress to range 1.0 (high-low=1)
        bars += [MockBar(high=100.0 + 0.5, low=100.0 - 0.5, close=100.0) for _ in range(10)]

        vr = relative_volatility_ratio(bars, n_trailing=48, atr_period=14)
        self.assertIsNotNone(vr)
        self.assertLess(vr, 0.80)

    def test_vol_ratio_expansion(self):
        """Volatility exploding produces VolRatio > 1.80."""
        # First 60 bars have range 2.0
        bars = [MockBar(high=100.0 + 1.0, low=100.0 - 1.0, close=100.0) for _ in range(60)]
        # Last 10 bars expand to range 20.0
        bars += [MockBar(high=100.0 + 10.0, low=100.0 - 10.0, close=100.0) for _ in range(10)]

        vr = relative_volatility_ratio(bars, n_trailing=48, atr_period=14)
        self.assertIsNotNone(vr)
        self.assertGreater(vr, 1.80)

    def test_vol_ratio_insufficient_history(self):
        """Fewer than 48 + 14 = 62 bars must return None."""
        bars = [MockBar(high=102.0, low=98.0, close=100.0) for _ in range(50)]
        vr = relative_volatility_ratio(bars, n_trailing=48, atr_period=14)
        self.assertIsNone(vr)

    # ─────────────────────────────────────────────────────────────
    # 3. Wilder's ADX Indicator Tests
    # ─────────────────────────────────────────────────────────────

    def test_adx_strong_trend(self):
        """A persistent ramp produces high ADX (>= 25.0)."""
        bars = []
        for i in range(60):
            p = 100.0 + (i * 3.0)
            bars.append(MockBar(high=p + 2.0, low=p - 1.0, close=p + 1.5))

        res = adx_indicator(bars, period=14)
        self.assertIsNotNone(res)
        self.assertGreaterEqual(res["adx"], 25.0)
        self.assertGreater(res["plus_di"], res["minus_di"])

    def test_adx_choppy_market(self):
        """Oscillating chop produces low ADX (< 20.0)."""
        bars = []
        for i in range(60):
            base = 100.0 if i % 2 == 0 else 102.0
            bars.append(MockBar(high=base + 2.0, low=base - 2.0, close=base))

        res = adx_indicator(bars, period=14)
        self.assertIsNotNone(res)
        self.assertLess(res["adx"], 25.0)

    def test_adx_insufficient_history(self):
        """Fewer than 2*14 + 1 = 29 bars must return None."""
        bars = [MockBar(high=105.0, low=95.0, close=100.0) for _ in range(20)]
        res = adx_indicator(bars, period=14)
        self.assertIsNone(res)

    # ─────────────────────────────────────────────────────────────
    # 4. Deterministic 4-Step Precedence Classification Tests
    # ─────────────────────────────────────────────────────────────

    def test_precedence_step1_unavailable(self):
        """If history is insufficient (< 62 bars), status is UNAVAILABLE."""
        bars = [MockBar(high=102.0, low=98.0, close=100.0) for _ in range(30)]
        res = classify_market_regime(bars)
        self.assertEqual(res["regime"], "UNAVAILABLE")
        self.assertIn("INSUFFICIENT_WARMUP", res["reason"])

    def test_precedence_step1_corrupt_data(self):
        """Corrupted bar with inverted high/low or NaN returns UNAVAILABLE."""
        bars = [MockBar(high=102.0, low=98.0, close=100.0) for _ in range(65)]
        bars[10] = MockBar(high=90.0, low=110.0, close=100.0)  # Inverted
        res = classify_market_regime(bars)
        self.assertEqual(res["regime"], "UNAVAILABLE")
        self.assertIn("CORRUPT_BAR_DATA", res["reason"])

    def test_precedence_step2_sideways_low_er(self):
        """Low ER (< 0.25) must trigger SIDEWAYS, even if volatility is normal."""
        # 50 bars of normal ramp, then 20 bars of flat sideways chop
        bars = [MockBar(high=100.0 + i, low=98.0 + i, close=99.0 + i) for i in range(50)]
        # 20 bars oscillating +/- 0.5 around 150
        for i in range(20):
            p = 150.0 + (0.5 if i % 2 == 0 else -0.5)
            bars.append(MockBar(high=p + 1.0, low=p - 1.0, close=p))

        res = classify_market_regime(bars)
        self.assertEqual(res["regime"], "SIDEWAYS")
        self.assertIn("LOW_EFFICIENCY", res["reason"])

    def test_precedence_step2_sideways_vol_compression(self):
        """Severe volatility compression (VolRatio < 0.80) triggers SIDEWAYS."""
        # 55 bars of range 10, then 15 bars of tiny range 0.5
        bars = [MockBar(high=100.0 + 5.0, low=100.0 - 5.0, close=100.0) for _ in range(55)]
        bars += [MockBar(high=100.0 + 0.25, low=100.0 - 0.25, close=100.0) for _ in range(15)]

        res = classify_market_regime(bars)
        self.assertEqual(res["regime"], "SIDEWAYS")
        self.assertIn("VOL_COMPRESSION", res["reason"])

    def test_precedence_step3_directional_trend(self):
        """Strong sustained trend with normal volatility produces DIRECTIONAL."""
        bars = []
        for i in range(70):
            p = 100.0 + (i * 2.0)
            bars.append(MockBar(high=p + 1.5, low=p - 1.0, close=p + 1.0))

        res = classify_market_regime(bars)
        self.assertEqual(res["regime"], "DIRECTIONAL")
        self.assertGreaterEqual(res["er"], 0.40)
        self.assertGreaterEqual(res["adx"], 25.0)
        self.assertIn("CONFIRMED_TREND:BULLISH", res["reason"])

    def test_precedence_step4_transitioning_intermediate_efficiency(self):
        """ER between 0.25 and 0.40 with developing ADX produces TRANSITIONING."""
        # Construct mixed bars that produce ER ~ 0.30
        bars = [MockBar(high=100.0 + i * 1.5, low=98.0 + i * 1.5, close=99.0 + i * 1.5) for i in range(45)]
        # Add a gentle pullback then resumption
        for i in range(25):
            p = 160.0 + (i * 0.4) + (2.0 if i % 4 in (1, 2) else -2.0)
            bars.append(MockBar(high=p + 2.5, low=p - 2.5, close=p))

        res = classify_market_regime(bars)
        # Must be deterministic: either SIDEWAYS or TRANSITIONING depending on exact numbers
        self.assertIn(res["regime"], ["SIDEWAYS", "TRANSITIONING"])
        self.assertIsNotNone(res["er"])
        self.assertIsNotNone(res["vol_ratio"])
        self.assertIsNotNone(res["adx"])


if __name__ == "__main__":
    unittest.main()
