#!/usr/bin/env python3
"""Tests for deterministic dynamic stop/target from OHLCV structure."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(WORKSPACE))

from scripts.dynamic_trade_boundaries import (
    compute_dynamic_boundaries,
    propose_boundary_update,
    trend_scale_from_er_adx,
)


def _trend_bars(n: int = 80, start: float = 100.0, drift: float = 0.004):
    bars = []
    px = start
    for _ in range(n):
        o = px
        px = px * (1.0 + drift)
        bars.append({"open": o, "high": max(o, px) * 1.002, "low": min(o, px) * 0.998, "close": px})
    return bars


def _chop_bars(n: int = 80, start: float = 100.0):
    bars = []
    px = start
    for i in range(n):
        o = px
        px = px * (1.001 if i % 2 == 0 else 0.999)
        bars.append({"open": o, "high": max(o, px) * 1.001, "low": min(o, px) * 0.999, "close": px})
    return bars


class TestDynamicBoundaries(unittest.TestCase):
    def test_deterministic(self):
        bars = _trend_bars()
        a = compute_dynamic_boundaries(bars, side="LONG", entry_price=bars[-1]["close"])
        b = compute_dynamic_boundaries(bars, side="LONG", entry_price=bars[-1]["close"])
        self.assertEqual(a, b)

    def test_long_stop_below_entry_target_above(self):
        bars = _trend_bars()
        entry = bars[-1]["close"]
        res = compute_dynamic_boundaries(bars, side="LONG", entry_price=entry)
        self.assertTrue(res["ok"], res)
        self.assertLess(res["stop_price"], entry)
        self.assertGreater(res["target_price"], entry)
        self.assertGreaterEqual(res["net_rr"], 1.0)
        self.assertNotEqual(res["stop_bps"], 100.0)  # not the frozen Arm A constant by construction

    def test_short_geometry(self):
        bars = _trend_bars(drift=-0.004)
        entry = bars[-1]["close"]
        res = compute_dynamic_boundaries(bars, side="SHORT", entry_price=entry)
        self.assertTrue(res["ok"], res)
        self.assertGreater(res["stop_price"], entry)
        self.assertLess(res["target_price"], entry)

    def test_bounds_clamped(self):
        bars = _trend_bars(drift=0.02)
        res = compute_dynamic_boundaries(bars, side="LONG", entry_price=bars[-1]["close"])
        self.assertLessEqual(res["stop_bps"], 250.0)
        self.assertLessEqual(res["target_bps"], 400.0)
        self.assertGreaterEqual(res["stop_bps"], 40.0)

    def test_insufficient_history(self):
        bars = _trend_bars(n=10)
        res = compute_dynamic_boundaries(bars, side="LONG", entry_price=100.0)
        self.assertFalse(res["ok"])

    def test_trend_scale_labels(self):
        strong = trend_scale_from_er_adx(0.5, 30.0)
        weak = trend_scale_from_er_adx(0.1, 10.0)
        self.assertEqual(strong["label"], "STRONG_TREND")
        self.assertEqual(weak["label"], "WEAK_OR_CHOP")
        self.assertGreater(strong["target_atr_mult"], weak["target_atr_mult"])

    def test_propose_monotonic_long_stop(self):
        bars = _trend_bars()
        entry = bars[-1]["close"]
        base = compute_dynamic_boundaries(bars, side="LONG", entry_price=entry)
        # Propose with same bars — may reject NO_MATERIAL_CHANGE
        prop = propose_boundary_update(
            side="LONG",
            entry_price=entry,
            current_stop=base["stop_price"] * 0.99,  # looser than fresh → should tighten
            current_target=base["target_price"],
            bars=bars,
            last_price=entry,
        )
        self.assertIn(prop["state"], ("PROPOSED", "REJECTED"))
        if prop["state"] == "PROPOSED":
            self.assertGreaterEqual(prop["proposal"]["stop_price"], base["stop_price"] * 0.99)


if __name__ == "__main__":
    unittest.main()
