"""Unit tests for RP-611 frozen absorption primitives."""

from __future__ import annotations

import unittest

from scripts.rp611_absorption_core import Candle, classify_absorption, confirm_absorption, rvol


class TestRp611Absorption(unittest.TestCase):
    def test_rvol_excludes_current(self):
        prior = [100.0] * 20
        self.assertAlmostEqual(rvol(150.0, prior), 1.5)

    def test_long_accumulation(self):
        # Red, high volume, small body, long lower wick
        prior = [100.0] * 20
        c = Candle(open=110, high=111, low=100, close=108, volume=200, open_time_ms=1)
        # body=2, range=11, eff≈0.18; lower wick=(108-100)/11≈0.727; red
        cls, d = classify_absorption(c, prior)
        self.assertEqual(cls, "ACCUMULATION_LONG")
        self.assertGreaterEqual(d["rvol"], 1.5)

    def test_confirmation_long(self):
        abs_c = Candle(open=110, high=111, low=100, close=108, volume=200, open_time_ms=1)
        nxt = Candle(open=108, high=110, low=107, close=109.6, volume=50, open_time_ms=2)
        # mid = 105.5; close 109.6 > mid
        conf = confirm_absorption("ACCUMULATION_LONG", abs_c, nxt)
        self.assertTrue(conf["passed"])


if __name__ == "__main__":
    unittest.main()
